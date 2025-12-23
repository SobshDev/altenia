use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;

use super::models::{LevelCountRow, LogRow, LogStatsRow};
use crate::modules::logging::domain::{
    LogDomainError, LogEntry, LogFilters, LogId, LogLevel, LogQueryResult, LogRepository,
    LogStats, Pagination, SortOrder, SpanId, TraceId,
};
use crate::modules::projects::domain::ProjectId;

pub struct TimescaleLogRepository {
    pool: Arc<PgPool>,
}

impl TimescaleLogRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    fn row_to_log_entry(row: LogRow) -> Result<LogEntry, LogDomainError> {
        let id = LogId::new(row.id);
        let project_id = ProjectId::new(row.project_id);
        let level = LogLevel::from_str(&row.level)?;
        let trace_id = row.trace_id.and_then(TraceId::new);
        let span_id = row.span_id.and_then(SpanId::new);

        Ok(LogEntry::reconstruct(
            id,
            project_id,
            level,
            row.message,
            row.timestamp,
            row.received_at,
            row.source,
            row.metadata,
            trace_id,
            span_id,
        ))
    }

    /// Build WHERE clause from filters
    fn build_filter_clause(filters: &LogFilters, param_offset: usize) -> (String, Vec<String>) {
        let mut conditions = Vec::new();
        let mut params = Vec::new();
        let mut idx = param_offset;

        if let Some(ref levels) = filters.levels {
            if !levels.is_empty() {
                let placeholders: Vec<String> =
                    levels.iter().map(|_| format!("${}", { idx += 1; idx })).collect();
                conditions.push(format!("level IN ({})", placeholders.join(", ")));
                params.extend(levels.iter().map(|l| l.as_str().to_string()));
            }
        }

        if let Some(ref start_time) = filters.start_time {
            idx += 1;
            conditions.push(format!("timestamp >= ${}", idx));
            params.push(start_time.to_rfc3339());
        }

        if let Some(ref end_time) = filters.end_time {
            idx += 1;
            conditions.push(format!("timestamp <= ${}", idx));
            params.push(end_time.to_rfc3339());
        }

        if let Some(ref source) = filters.source {
            idx += 1;
            conditions.push(format!("source = ${}", idx));
            params.push(source.clone());
        }

        if let Some(ref search) = filters.search {
            idx += 1;
            conditions.push(format!("message ILIKE ${}", idx));
            params.push(format!("%{}%", search));
        }

        if let Some(ref trace_id) = filters.trace_id {
            idx += 1;
            conditions.push(format!("trace_id = ${}", idx));
            params.push(trace_id.clone());
        }

        let clause = if conditions.is_empty() {
            String::new()
        } else {
            format!(" AND {}", conditions.join(" AND "))
        };

        (clause, params)
    }
}

#[async_trait]
impl LogRepository for TimescaleLogRepository {
    async fn save_batch(&self, logs: &[LogEntry]) -> Result<u32, LogDomainError> {
        if logs.is_empty() {
            return Ok(0);
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| LogDomainError::InternalError(e.to_string()))?;

        let mut count = 0u32;
        for log in logs {
            let result = sqlx::query(
                r#"
                INSERT INTO logs (id, project_id, level, message, timestamp, received_at,
                                  source, metadata, trace_id, span_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                "#,
            )
            .bind(log.id().as_str())
            .bind(log.project_id().as_str())
            .bind(log.level().as_str())
            .bind(log.message())
            .bind(log.timestamp())
            .bind(log.received_at())
            .bind(log.source())
            .bind(log.metadata())
            .bind(log.trace_id().map(|t| t.as_str()))
            .bind(log.span_id().map(|s| s.as_str()))
            .execute(&mut *tx)
            .await;

            match result {
                Ok(_) => count += 1,
                Err(e) => {
                    tracing::warn!("Failed to insert log: {}", e);
                }
            }
        }

        tx.commit()
            .await
            .map_err(|e| LogDomainError::InternalError(e.to_string()))?;

        Ok(count)
    }

    async fn query(
        &self,
        project_id: &ProjectId,
        filters: &LogFilters,
        pagination: &Pagination,
        sort: SortOrder,
    ) -> Result<LogQueryResult, LogDomainError> {
        let order = match sort {
            SortOrder::Ascending => "ASC",
            SortOrder::Descending => "DESC",
        };

        let (filter_clause, filter_params) = Self::build_filter_clause(filters, 1);

        // Build query with dynamic filters
        let query = format!(
            r#"
            SELECT id, project_id, level, message, timestamp, received_at,
                   source, metadata, trace_id, span_id
            FROM logs
            WHERE project_id = $1 {}
            ORDER BY timestamp {}
            LIMIT {} OFFSET {}
            "#,
            filter_clause, order, pagination.limit, pagination.offset
        );

        // We need to use raw query since we have dynamic parameters
        let mut query_builder = sqlx::query_as::<_, LogRow>(&query).bind(project_id.as_str());

        // Bind filter params dynamically based on filter types
        // Note: This is a simplified version - in production you might use a query builder
        if let Some(ref levels) = filters.levels {
            for level in levels {
                query_builder = query_builder.bind(level.as_str());
            }
        }
        if let Some(ref start_time) = filters.start_time {
            query_builder = query_builder.bind(start_time);
        }
        if let Some(ref end_time) = filters.end_time {
            query_builder = query_builder.bind(end_time);
        }
        if let Some(ref source) = filters.source {
            query_builder = query_builder.bind(source);
        }
        if let Some(ref search) = filters.search {
            query_builder = query_builder.bind(format!("%{}%", search));
        }
        if let Some(ref trace_id) = filters.trace_id {
            query_builder = query_builder.bind(trace_id);
        }

        let rows: Vec<LogRow> = query_builder
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(|e| LogDomainError::InternalError(e.to_string()))?;

        // Get total count
        let total = self.count(project_id, filters).await?;
        let fetched_count = rows.len() as i64;
        let has_more = pagination.offset + fetched_count < total;

        let logs = rows
            .into_iter()
            .map(Self::row_to_log_entry)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(LogQueryResult {
            logs,
            total,
            has_more,
        })
    }

    async fn count(
        &self,
        project_id: &ProjectId,
        filters: &LogFilters,
    ) -> Result<i64, LogDomainError> {
        let (filter_clause, _) = Self::build_filter_clause(filters, 1);

        let query = format!(
            r#"SELECT COUNT(*) as count FROM logs WHERE project_id = $1 {}"#,
            filter_clause
        );

        let mut query_builder = sqlx::query_scalar::<_, i64>(&query).bind(project_id.as_str());

        if let Some(ref levels) = filters.levels {
            for level in levels {
                query_builder = query_builder.bind(level.as_str());
            }
        }
        if let Some(ref start_time) = filters.start_time {
            query_builder = query_builder.bind(start_time);
        }
        if let Some(ref end_time) = filters.end_time {
            query_builder = query_builder.bind(end_time);
        }
        if let Some(ref source) = filters.source {
            query_builder = query_builder.bind(source);
        }
        if let Some(ref search) = filters.search {
            query_builder = query_builder.bind(format!("%{}%", search));
        }
        if let Some(ref trace_id) = filters.trace_id {
            query_builder = query_builder.bind(trace_id);
        }

        let count = query_builder
            .fetch_one(self.pool.as_ref())
            .await
            .map_err(|e| LogDomainError::InternalError(e.to_string()))?;

        Ok(count)
    }

    async fn get_stats(&self, project_id: &ProjectId) -> Result<LogStats, LogDomainError> {
        // Get total count and time range
        let stats_row: LogStatsRow = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) as total_count,
                MIN(timestamp) as oldest_log,
                MAX(timestamp) as newest_log
            FROM logs
            WHERE project_id = $1
            "#,
        )
        .bind(project_id.as_str())
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| LogDomainError::InternalError(e.to_string()))?;

        // Get counts by level
        let level_counts: Vec<LevelCountRow> = sqlx::query_as(
            r#"
            SELECT level, COUNT(*) as count
            FROM logs
            WHERE project_id = $1
            GROUP BY level
            ORDER BY count DESC
            "#,
        )
        .bind(project_id.as_str())
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| LogDomainError::InternalError(e.to_string()))?;

        let counts_by_level = level_counts
            .into_iter()
            .filter_map(|row| {
                LogLevel::from_str(&row.level)
                    .ok()
                    .map(|level| (level, row.count))
            })
            .collect();

        Ok(LogStats {
            total_count: stats_row.total_count,
            counts_by_level,
            oldest_log: stats_row.oldest_log,
            newest_log: stats_row.newest_log,
        })
    }

    async fn delete_before(
        &self,
        project_id: &ProjectId,
        before: DateTime<Utc>,
    ) -> Result<u64, LogDomainError> {
        let result = sqlx::query(
            r#"
            DELETE FROM logs
            WHERE project_id = $1 AND timestamp < $2
            "#,
        )
        .bind(project_id.as_str())
        .bind(before)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| LogDomainError::InternalError(e.to_string()))?;

        Ok(result.rows_affected())
    }
}
