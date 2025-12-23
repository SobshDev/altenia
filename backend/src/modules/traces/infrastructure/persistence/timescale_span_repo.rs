use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;

use crate::modules::projects::domain::ProjectId;
use crate::modules::traces::domain::{
    Pagination, Span, SpanEvent, SpanKind, SpanLink, SpanStatusCode, SpansRepository,
    TraceFilters, TraceSearchResult, TracesDomainError, TraceSummary,
};
use crate::modules::traces::infrastructure::persistence::models::{SpanRow, TraceSummaryRow};

pub struct TimescaleSpanRepository {
    pool: PgPool,
}

impl TimescaleSpanRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_span(row: SpanRow) -> Result<Span, TracesDomainError> {
        let kind = SpanKind::from_str(&row.kind)?;
        let status = SpanStatusCode::from_str(&row.status)?;

        let events: Vec<SpanEvent> = serde_json::from_value(row.events)
            .map_err(|e| TracesDomainError::InternalError(e.to_string()))?;

        let links: Vec<SpanLink> = serde_json::from_value(row.links)
            .map_err(|e| TracesDomainError::InternalError(e.to_string()))?;

        Ok(Span::new(
            row.id,
            ProjectId::new(row.project_id),
            row.trace_id,
            row.span_id,
            row.parent_span_id,
            row.name,
            kind,
            row.start_time,
            row.end_time,
            status,
            row.status_message,
            row.service_name,
            row.service_version,
            row.resource_attributes,
            row.attributes,
            events,
            links,
        ))
    }
}

#[async_trait]
impl SpansRepository for TimescaleSpanRepository {
    async fn save_batch(&self, spans: &[Span]) -> Result<u32, TracesDomainError> {
        if spans.is_empty() {
            return Ok(0);
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| TracesDomainError::InternalError(e.to_string()))?;

        let mut count = 0u32;

        for span in spans {
            let events_json = json!(span.events());
            let links_json = json!(span.links());

            sqlx::query(
                r#"
                INSERT INTO spans (
                    id, project_id, trace_id, span_id, parent_span_id, name, kind,
                    start_time, end_time, duration_ns, status, status_message, received_at,
                    service_name, service_version, resource_attributes, attributes, events, links
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
                ON CONFLICT (project_id, start_time, id) DO NOTHING
                "#,
            )
            .bind(span.id())
            .bind(span.project_id().as_str())
            .bind(span.trace_id())
            .bind(span.span_id())
            .bind(span.parent_span_id())
            .bind(span.name())
            .bind(span.kind().as_str())
            .bind(span.start_time())
            .bind(span.end_time())
            .bind(span.duration_ns())
            .bind(span.status().as_str())
            .bind(span.status_message())
            .bind(Utc::now())
            .bind(span.service_name())
            .bind(span.service_version())
            .bind(span.resource_attributes())
            .bind(span.attributes())
            .bind(&events_json)
            .bind(&links_json)
            .execute(&mut *tx)
            .await
            .map_err(|e| TracesDomainError::InternalError(e.to_string()))?;

            count += 1;
        }

        tx.commit()
            .await
            .map_err(|e| TracesDomainError::InternalError(e.to_string()))?;

        Ok(count)
    }

    async fn get_trace(
        &self,
        project_id: &ProjectId,
        trace_id: &str,
    ) -> Result<Vec<Span>, TracesDomainError> {
        let rows: Vec<SpanRow> = sqlx::query_as(
            r#"
            SELECT id, project_id, trace_id, span_id, parent_span_id, name, kind,
                   start_time, end_time, duration_ns, status, status_message, received_at,
                   service_name, service_version, resource_attributes, attributes, events, links
            FROM spans
            WHERE project_id = $1 AND trace_id = $2
            ORDER BY start_time ASC
            "#,
        )
        .bind(project_id.as_str())
        .bind(trace_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| TracesDomainError::InternalError(e.to_string()))?;

        rows.into_iter().map(Self::row_to_span).collect()
    }

    async fn search_traces(
        &self,
        project_id: &ProjectId,
        filters: &TraceFilters,
        pagination: &Pagination,
    ) -> Result<TraceSearchResult, TracesDomainError> {
        // Build dynamic WHERE clause
        let mut conditions = vec!["project_id = $1".to_string()];
        let mut param_idx = 2;

        if filters.service_name.is_some() {
            conditions.push(format!("service_name = ${}", param_idx));
            param_idx += 1;
        }
        if filters.span_name.is_some() {
            conditions.push(format!("name ILIKE ${}", param_idx));
            param_idx += 1;
        }
        if filters.status.is_some() {
            conditions.push(format!("status = ${}", param_idx));
            param_idx += 1;
        }
        if filters.start_time.is_some() {
            conditions.push(format!("start_time >= ${}", param_idx));
            param_idx += 1;
        }
        if filters.end_time.is_some() {
            conditions.push(format!("start_time <= ${}", param_idx));
            param_idx += 1;
        }
        if filters.min_duration_ns.is_some() {
            conditions.push(format!("duration_ns >= ${}", param_idx));
            param_idx += 1;
        }
        if filters.max_duration_ns.is_some() {
            conditions.push(format!("duration_ns <= ${}", param_idx));
            param_idx += 1;
        }

        let where_clause = conditions.join(" AND ");

        // Query for trace summaries
        let query = format!(
            r#"
            WITH filtered_spans AS (
                SELECT trace_id, name, service_name, start_time, end_time, duration_ns, status, parent_span_id
                FROM spans
                WHERE {}
            ),
            trace_stats AS (
                SELECT
                    trace_id,
                    MIN(CASE WHEN parent_span_id IS NULL THEN name END) as root_span_name,
                    ARRAY_AGG(DISTINCT service_name) FILTER (WHERE service_name IS NOT NULL) as service_names,
                    COUNT(*) as span_count,
                    COUNT(*) FILTER (WHERE status = 'error') as error_count,
                    MIN(start_time) as start_time,
                    MAX(end_time) as end_time,
                    EXTRACT(EPOCH FROM (MAX(end_time) - MIN(start_time))) * 1000000000 as duration_ns
                FROM filtered_spans
                GROUP BY trace_id
            )
            SELECT trace_id, root_span_name, service_names, span_count, error_count,
                   start_time, end_time, duration_ns::BIGINT
            FROM trace_stats
            ORDER BY start_time DESC
            LIMIT ${} OFFSET ${}
            "#,
            where_clause,
            param_idx,
            param_idx + 1
        );

        let mut query_builder = sqlx::query_as::<_, TraceSummaryRow>(&query);
        query_builder = query_builder.bind(project_id.as_str());

        if let Some(ref service_name) = filters.service_name {
            query_builder = query_builder.bind(service_name);
        }
        if let Some(ref span_name) = filters.span_name {
            query_builder = query_builder.bind(format!("%{}%", span_name));
        }
        if let Some(ref status) = filters.status {
            query_builder = query_builder.bind(status.as_str());
        }
        if let Some(start_time) = filters.start_time {
            query_builder = query_builder.bind(start_time);
        }
        if let Some(end_time) = filters.end_time {
            query_builder = query_builder.bind(end_time);
        }
        if let Some(min_duration) = filters.min_duration_ns {
            query_builder = query_builder.bind(min_duration);
        }
        if let Some(max_duration) = filters.max_duration_ns {
            query_builder = query_builder.bind(max_duration);
        }

        query_builder = query_builder.bind(pagination.limit).bind(pagination.offset);

        let rows: Vec<TraceSummaryRow> = query_builder
            .fetch_all(&self.pool)
            .await
            .map_err(|e| TracesDomainError::InternalError(e.to_string()))?;

        // Count total distinct traces
        let count_query = format!(
            r#"
            SELECT COUNT(DISTINCT trace_id) as total
            FROM spans
            WHERE {}
            "#,
            where_clause
        );

        let mut count_builder = sqlx::query_scalar::<_, i64>(&count_query);
        count_builder = count_builder.bind(project_id.as_str());

        if let Some(ref service_name) = filters.service_name {
            count_builder = count_builder.bind(service_name);
        }
        if let Some(ref span_name) = filters.span_name {
            count_builder = count_builder.bind(format!("%{}%", span_name));
        }
        if let Some(ref status) = filters.status {
            count_builder = count_builder.bind(status.as_str());
        }
        if let Some(start_time) = filters.start_time {
            count_builder = count_builder.bind(start_time);
        }
        if let Some(end_time) = filters.end_time {
            count_builder = count_builder.bind(end_time);
        }
        if let Some(min_duration) = filters.min_duration_ns {
            count_builder = count_builder.bind(min_duration);
        }
        if let Some(max_duration) = filters.max_duration_ns {
            count_builder = count_builder.bind(max_duration);
        }

        let total: i64 = count_builder
            .fetch_one(&self.pool)
            .await
            .map_err(|e| TracesDomainError::InternalError(e.to_string()))?;

        let traces: Vec<TraceSummary> = rows
            .into_iter()
            .map(|row| TraceSummary {
                trace_id: row.trace_id,
                root_span_name: row.root_span_name,
                service_names: row.service_names,
                span_count: row.span_count,
                error_count: row.error_count,
                start_time: row.start_time,
                end_time: row.end_time,
                duration_ns: row.duration_ns,
            })
            .collect();

        Ok(TraceSearchResult { traces, total })
    }

    async fn get_service_names(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<String>, TracesDomainError> {
        let services: Vec<String> = sqlx::query_scalar(
            r#"
            SELECT DISTINCT service_name
            FROM spans
            WHERE project_id = $1 AND service_name IS NOT NULL
            ORDER BY service_name
            "#,
        )
        .bind(project_id.as_str())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| TracesDomainError::InternalError(e.to_string()))?;

        Ok(services)
    }

    async fn delete_before(
        &self,
        project_id: &ProjectId,
        before: chrono::DateTime<chrono::Utc>,
    ) -> Result<u64, TracesDomainError> {
        let result = sqlx::query(
            r#"
            DELETE FROM spans
            WHERE project_id = $1 AND start_time < $2
            "#,
        )
        .bind(project_id.as_str())
        .bind(before)
        .execute(&self.pool)
        .await
        .map_err(|e| TracesDomainError::InternalError(e.to_string()))?;

        Ok(result.rows_affected())
    }
}
