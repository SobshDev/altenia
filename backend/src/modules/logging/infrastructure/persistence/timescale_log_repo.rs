use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;

use super::models::{LevelBucketRow, LevelCountRow, LogRow, LogStatsRow, SourceCountRow, TimeBucketRow};
use crate::modules::logging::domain::{
    LogDomainError, LogEntry, LogFilters, LogId, LogLevel, LogQueryResult, LogRepository,
    LogStats, MetadataFilter, MetadataOperator, Pagination, SortOrder, SpanId, TraceId,
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
    /// Returns the SQL clause and the count of parameters used for binding
    fn build_filter_clause(filters: &LogFilters, param_offset: usize) -> (String, usize) {
        let mut conditions = Vec::new();
        let mut idx = param_offset;

        if let Some(ref levels) = filters.levels {
            if !levels.is_empty() {
                let placeholders: Vec<String> =
                    levels.iter().map(|_| format!("${}", { idx += 1; idx })).collect();
                conditions.push(format!("level IN ({})", placeholders.join(", ")));
            }
        }

        if filters.start_time.is_some() {
            idx += 1;
            conditions.push(format!("timestamp >= ${}", idx));
        }

        if filters.end_time.is_some() {
            idx += 1;
            conditions.push(format!("timestamp <= ${}", idx));
        }

        if filters.source.is_some() {
            idx += 1;
            conditions.push(format!("source = ${}", idx));
        }

        if filters.search.is_some() {
            idx += 1;
            conditions.push(format!("message ILIKE ${}", idx));
        }

        if filters.trace_id.is_some() {
            idx += 1;
            conditions.push(format!("trace_id = ${}", idx));
        }

        // Add metadata filter conditions
        for filter in &filters.metadata_filters {
            let metadata_condition = Self::build_metadata_condition(filter, &mut idx);
            conditions.push(metadata_condition);
        }

        let clause = if conditions.is_empty() {
            String::new()
        } else {
            format!(" AND {}", conditions.join(" AND "))
        };

        (clause, idx - param_offset)
    }

    /// Build SQL condition for a single metadata filter
    fn build_metadata_condition(filter: &MetadataFilter, idx: &mut usize) -> String {
        let key_path = Self::build_jsonb_path(&filter.key);

        match filter.operator {
            MetadataOperator::Exists => {
                // Check if key exists in JSONB
                format!("metadata ? '{}'", filter.key.replace('\'', "''"))
            }
            MetadataOperator::Eq => {
                *idx += 1;
                // Exact match using @> containment
                format!("metadata @> jsonb_build_object('{}', ${}::jsonb)",
                    filter.key.replace('\'', "''"), *idx)
            }
            MetadataOperator::Neq => {
                *idx += 1;
                // Not equal
                format!("NOT (metadata @> jsonb_build_object('{}', ${}::jsonb))",
                    filter.key.replace('\'', "''"), *idx)
            }
            MetadataOperator::Contains => {
                *idx += 1;
                // String contains (case-insensitive)
                format!("({} IS NOT NULL AND {}::text ILIKE ${})",
                    key_path, key_path, *idx)
            }
            MetadataOperator::Gt => {
                *idx += 1;
                // Greater than (numeric comparison)
                format!("({}::numeric > ${}::numeric)", key_path, *idx)
            }
            MetadataOperator::Lt => {
                *idx += 1;
                // Less than (numeric comparison)
                format!("({}::numeric < ${}::numeric)", key_path, *idx)
            }
            MetadataOperator::Gte => {
                *idx += 1;
                // Greater than or equal (numeric comparison)
                format!("({}::numeric >= ${}::numeric)", key_path, *idx)
            }
            MetadataOperator::Lte => {
                *idx += 1;
                // Less than or equal (numeric comparison)
                format!("({}::numeric <= ${}::numeric)", key_path, *idx)
            }
        }
    }

    /// Build JSONB path accessor for nested keys (e.g., "request.path" -> metadata->'request'->'path')
    fn build_jsonb_path(key: &str) -> String {
        let parts: Vec<&str> = key.split('.').collect();
        if parts.len() == 1 {
            format!("metadata->'{}'", key.replace('\'', "''"))
        } else {
            let mut path = "metadata".to_string();
            for part in parts.iter() {
                path.push_str(&format!("->'{}'", part.replace('\'', "''")));
            }
            path
        }
    }

    /// Bind metadata filter value to query builder
    fn bind_metadata_filter_value<'q, O>(
        query_builder: sqlx::query::QueryAs<'q, sqlx::Postgres, O, sqlx::postgres::PgArguments>,
        filter: &'q MetadataFilter,
    ) -> sqlx::query::QueryAs<'q, sqlx::Postgres, O, sqlx::postgres::PgArguments> {
        match filter.operator {
            MetadataOperator::Exists => {
                // No value needed for exists operator
                query_builder
            }
            MetadataOperator::Eq | MetadataOperator::Neq => {
                // Bind the JSON value as text for JSONB comparison
                if let Some(ref value) = filter.value {
                    query_builder.bind(value.to_string())
                } else {
                    query_builder
                }
            }
            MetadataOperator::Contains => {
                // Bind as ILIKE pattern
                if let Some(ref value) = filter.value {
                    let pattern = format!("%{}%", value.as_str().unwrap_or(&value.to_string()));
                    query_builder.bind(pattern)
                } else {
                    query_builder
                }
            }
            MetadataOperator::Gt | MetadataOperator::Lt | MetadataOperator::Gte | MetadataOperator::Lte => {
                // Bind numeric value as string for casting
                if let Some(ref value) = filter.value {
                    let num_str = match value {
                        serde_json::Value::Number(n) => n.to_string(),
                        _ => value.to_string().trim_matches('"').to_string(),
                    };
                    query_builder.bind(num_str)
                } else {
                    query_builder
                }
            }
        }
    }

    /// Bind metadata filter value to scalar query builder
    fn bind_metadata_filter_value_scalar<'q>(
        query_builder: sqlx::query::QueryScalar<'q, sqlx::Postgres, i64, sqlx::postgres::PgArguments>,
        filter: &'q MetadataFilter,
    ) -> sqlx::query::QueryScalar<'q, sqlx::Postgres, i64, sqlx::postgres::PgArguments> {
        match filter.operator {
            MetadataOperator::Exists => {
                query_builder
            }
            MetadataOperator::Eq | MetadataOperator::Neq => {
                if let Some(ref value) = filter.value {
                    query_builder.bind(value.to_string())
                } else {
                    query_builder
                }
            }
            MetadataOperator::Contains => {
                if let Some(ref value) = filter.value {
                    let pattern = format!("%{}%", value.as_str().unwrap_or(&value.to_string()));
                    query_builder.bind(pattern)
                } else {
                    query_builder
                }
            }
            MetadataOperator::Gt | MetadataOperator::Lt | MetadataOperator::Gte | MetadataOperator::Lte => {
                if let Some(ref value) = filter.value {
                    let num_str = match value {
                        serde_json::Value::Number(n) => n.to_string(),
                        _ => value.to_string().trim_matches('"').to_string(),
                    };
                    query_builder.bind(num_str)
                } else {
                    query_builder
                }
            }
        }
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

        let (filter_clause, _param_count) = Self::build_filter_clause(filters, 1);

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

        // Bind metadata filter values
        for filter in &filters.metadata_filters {
            query_builder = Self::bind_metadata_filter_value(query_builder, filter);
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

        // Bind metadata filter values
        for filter in &filters.metadata_filters {
            query_builder = Self::bind_metadata_filter_value_scalar(query_builder, filter);
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

    // ==================== Metrics Methods ====================

    async fn get_volume_over_time(
        &self,
        project_id: &ProjectId,
        bucket_interval: &str,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> Result<Vec<(DateTime<Utc>, i64)>, LogDomainError> {
        let rows: Vec<TimeBucketRow> = sqlx::query_as(
            r#"
            SELECT
                time_bucket($1::interval, timestamp) AS bucket,
                COUNT(*) AS count
            FROM logs
            WHERE project_id = $2
              AND ($3::timestamptz IS NULL OR timestamp >= $3)
              AND ($4::timestamptz IS NULL OR timestamp <= $4)
            GROUP BY bucket
            ORDER BY bucket ASC
            "#,
        )
        .bind(bucket_interval)
        .bind(project_id.as_str())
        .bind(start_time)
        .bind(end_time)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| LogDomainError::InternalError(e.to_string()))?;

        Ok(rows.into_iter().map(|r| (r.bucket, r.count)).collect())
    }

    async fn get_levels_over_time(
        &self,
        project_id: &ProjectId,
        bucket_interval: &str,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> Result<Vec<(String, DateTime<Utc>, i64)>, LogDomainError> {
        let rows: Vec<LevelBucketRow> = sqlx::query_as(
            r#"
            SELECT
                level,
                time_bucket($1::interval, timestamp) AS bucket,
                COUNT(*) AS count
            FROM logs
            WHERE project_id = $2
              AND ($3::timestamptz IS NULL OR timestamp >= $3)
              AND ($4::timestamptz IS NULL OR timestamp <= $4)
            GROUP BY level, bucket
            ORDER BY bucket ASC, level ASC
            "#,
        )
        .bind(bucket_interval)
        .bind(project_id.as_str())
        .bind(start_time)
        .bind(end_time)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| LogDomainError::InternalError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| (r.level, r.bucket, r.count))
            .collect())
    }

    async fn get_top_sources(
        &self,
        project_id: &ProjectId,
        limit: i32,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> Result<Vec<(String, i64, i64)>, LogDomainError> {
        let rows: Vec<SourceCountRow> = sqlx::query_as(
            r#"
            SELECT
                COALESCE(source, 'unknown') AS source,
                COUNT(*) AS total,
                COUNT(*) FILTER (WHERE level IN ('error', 'fatal')) AS error_count
            FROM logs
            WHERE project_id = $1
              AND ($2::timestamptz IS NULL OR timestamp >= $2)
              AND ($3::timestamptz IS NULL OR timestamp <= $3)
            GROUP BY source
            ORDER BY total DESC
            LIMIT $4
            "#,
        )
        .bind(project_id.as_str())
        .bind(start_time)
        .bind(end_time)
        .bind(limit)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| LogDomainError::InternalError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| (r.source, r.total, r.error_count))
            .collect())
    }
}
