use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;

use super::models::{AggregatedMetricRow, MetricNameRow};
use crate::modules::metrics::domain::{
    AggregatedMetric, MetricFilters, MetricPoint, MetricQueryResult, MetricsDomainError,
    MetricsRepository, RollupInterval,
};
use crate::modules::projects::domain::ProjectId;

pub struct TimescaleMetricsRepository {
    pool: Arc<PgPool>,
}

impl TimescaleMetricsRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MetricsRepository for TimescaleMetricsRepository {
    async fn save_batch(&self, metrics: &[MetricPoint]) -> Result<u32, MetricsDomainError> {
        if metrics.is_empty() {
            return Ok(0);
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| MetricsDomainError::InternalError(e.to_string()))?;

        let mut count = 0u32;

        for metric in metrics {
            let tags_json = json!(metric.tags());

            let (bucket_bounds, bucket_counts, histogram_sum, histogram_count, histogram_min, histogram_max) =
                if let Some(h) = metric.histogram_data() {
                    (
                        Some(h.bucket_bounds().to_vec()),
                        Some(h.bucket_counts().to_vec()),
                        Some(h.sum()),
                        Some(h.count()),
                        Some(h.min()),
                        Some(h.max()),
                    )
                } else {
                    (None, None, None, None, None, None)
                };

            sqlx::query(
                r#"
                INSERT INTO metrics (
                    id, project_id, name, metric_type, value, timestamp, received_at,
                    unit, description, tags,
                    bucket_bounds, bucket_counts, histogram_sum, histogram_count,
                    histogram_min, histogram_max, trace_id, span_id
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
                "#,
            )
            .bind(metric.id())
            .bind(metric.project_id().as_str())
            .bind(metric.name())
            .bind(metric.metric_type().as_str())
            .bind(metric.value())
            .bind(metric.timestamp())
            .bind(metric.received_at())
            .bind(metric.unit())
            .bind(metric.description())
            .bind(&tags_json)
            .bind(&bucket_bounds)
            .bind(&bucket_counts)
            .bind(histogram_sum)
            .bind(histogram_count)
            .bind(histogram_min)
            .bind(histogram_max)
            .bind(metric.trace_id())
            .bind(metric.span_id())
            .execute(&mut *tx)
            .await
            .map_err(|e| MetricsDomainError::InternalError(e.to_string()))?;

            count += 1;
        }

        tx.commit()
            .await
            .map_err(|e| MetricsDomainError::InternalError(e.to_string()))?;

        Ok(count)
    }

    async fn query(
        &self,
        project_id: &ProjectId,
        filters: &MetricFilters,
        rollup: RollupInterval,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<MetricQueryResult, MetricsDomainError> {
        let table = match rollup {
            RollupInterval::Raw => "metrics",
            RollupInterval::OneMinute => "metrics_1m",
            RollupInterval::OneHour => "metrics_1h",
            RollupInterval::OneDay => "metrics_1d",
        };

        let (timestamp_col, value_cols) = if rollup == RollupInterval::Raw {
            (
                "timestamp",
                "value AS avg_value, value AS min_value, value AS max_value, value AS sum_value, 1::bigint AS sample_count",
            )
        } else {
            (
                "bucket",
                "avg_value, min_value, max_value, sum_value, sample_count",
            )
        };

        let mut conditions = vec!["project_id = $1".to_string()];
        let mut param_idx = 2;

        // Build name filter
        let names_filter = if let Some(ref names) = filters.names {
            if !names.is_empty() {
                let placeholders: Vec<String> = names
                    .iter()
                    .enumerate()
                    .map(|(i, _)| format!("${}", param_idx + i))
                    .collect();
                param_idx += names.len();
                Some(format!("name IN ({})", placeholders.join(", ")))
            } else {
                None
            }
        } else {
            None
        };

        if let Some(filter) = names_filter {
            conditions.push(filter);
        }

        // Build time range filter
        if filters.start_time.is_some() {
            conditions.push(format!("{} >= ${}", timestamp_col, param_idx));
            param_idx += 1;
        }

        if filters.end_time.is_some() {
            conditions.push(format!("{} <= ${}", timestamp_col, param_idx));
            param_idx += 1;
        }

        // Build trace_id filter
        if filters.trace_id.is_some() && rollup == RollupInterval::Raw {
            conditions.push(format!("trace_id = ${}", param_idx));
            param_idx += 1;
        }

        let where_clause = conditions.join(" AND ");

        let limit_val = limit.unwrap_or(1000).min(10000);
        let offset_val = offset.unwrap_or(0);

        let query = format!(
            r#"
            SELECT project_id, name, metric_type, {} AS bucket, {}
            FROM {}
            WHERE {}
            ORDER BY bucket DESC
            LIMIT {} OFFSET {}
            "#,
            timestamp_col, value_cols, table, where_clause, limit_val, offset_val
        );

        let count_query = format!(
            r#"
            SELECT COUNT(*) as count
            FROM {}
            WHERE {}
            "#,
            table, where_clause
        );

        // Build the query with dynamic bindings
        let mut sql_query = sqlx::query_as::<_, AggregatedMetricRow>(&query);
        let mut count_sql = sqlx::query_scalar::<_, i64>(&count_query);

        // Bind project_id
        sql_query = sql_query.bind(project_id.as_str());
        count_sql = count_sql.bind(project_id.as_str());

        // Bind names if present
        if let Some(ref names) = filters.names {
            for name in names {
                sql_query = sql_query.bind(name);
                count_sql = count_sql.bind(name);
            }
        }

        // Bind time filters
        if let Some(start) = filters.start_time {
            sql_query = sql_query.bind(start);
            count_sql = count_sql.bind(start);
        }

        if let Some(end) = filters.end_time {
            sql_query = sql_query.bind(end);
            count_sql = count_sql.bind(end);
        }

        // Bind trace_id if present
        if let Some(ref trace_id) = filters.trace_id {
            if rollup == RollupInterval::Raw {
                sql_query = sql_query.bind(trace_id);
                count_sql = count_sql.bind(trace_id);
            }
        }

        let rows: Vec<AggregatedMetricRow> = sql_query
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(|e| MetricsDomainError::InternalError(e.to_string()))?;

        let total: i64 = count_sql
            .fetch_one(self.pool.as_ref())
            .await
            .map_err(|e| MetricsDomainError::InternalError(e.to_string()))?;

        let metrics = rows
            .into_iter()
            .map(|row| AggregatedMetric {
                project_id: row.project_id,
                name: row.name,
                metric_type: row.metric_type,
                bucket: row.bucket,
                avg_value: row.avg_value.unwrap_or(0.0),
                min_value: row.min_value.unwrap_or(0.0),
                max_value: row.max_value.unwrap_or(0.0),
                sum_value: row.sum_value.unwrap_or(0.0),
                sample_count: row.sample_count.unwrap_or(0),
            })
            .collect();

        Ok(MetricQueryResult { metrics, total })
    }

    async fn get_metric_names(&self, project_id: &ProjectId) -> Result<Vec<String>, MetricsDomainError> {
        let rows: Vec<MetricNameRow> = sqlx::query_as(
            r#"
            SELECT DISTINCT name
            FROM metrics
            WHERE project_id = $1
            ORDER BY name
            LIMIT 1000
            "#,
        )
        .bind(project_id.as_str())
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| MetricsDomainError::InternalError(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.name).collect())
    }

    async fn delete_before(
        &self,
        project_id: &ProjectId,
        before: DateTime<Utc>,
    ) -> Result<u64, MetricsDomainError> {
        let result = sqlx::query(
            r#"
            DELETE FROM metrics
            WHERE project_id = $1 AND timestamp < $2
            "#,
        )
        .bind(project_id.as_str())
        .bind(before)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| MetricsDomainError::InternalError(e.to_string()))?;

        Ok(result.rows_affected())
    }
}
