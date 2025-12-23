use std::sync::Arc;

use serde_json::json;

use crate::modules::auth::application::ports::IdGenerator;
use crate::modules::auth::domain::UserId;
use crate::modules::organizations::domain::{OrgId, OrganizationMemberRepository};
use crate::modules::projects::domain::{ProjectId, ProjectRepository};
use crate::modules::traces::application::dto::*;
use crate::modules::traces::domain::{
    Pagination, Span, SpanEvent, SpanKind, SpanLink, SpanStatusCode, SpansRepository,
    TraceFilters, TracesDomainError,
};

pub struct TraceService<SR, PR, OMR, ID>
where
    SR: SpansRepository,
    PR: ProjectRepository,
    OMR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    spans_repo: Arc<SR>,
    project_repo: Arc<PR>,
    member_repo: Arc<OMR>,
    id_generator: Arc<ID>,
}

impl<SR, PR, OMR, ID> TraceService<SR, PR, OMR, ID>
where
    SR: SpansRepository,
    PR: ProjectRepository,
    OMR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    pub fn new(
        spans_repo: Arc<SR>,
        project_repo: Arc<PR>,
        member_repo: Arc<OMR>,
        id_generator: Arc<ID>,
    ) -> Self {
        Self {
            spans_repo,
            project_repo,
            member_repo,
            id_generator,
        }
    }

    async fn verify_project_access(
        &self,
        project_id: &ProjectId,
        user_id: &str,
    ) -> Result<(), TracesDomainError> {
        let project = self
            .project_repo
            .find_by_id(project_id)
            .await
            .map_err(|e| TracesDomainError::InternalError(e.to_string()))?
            .ok_or(TracesDomainError::ProjectNotFound)?;

        if project.is_deleted() {
            return Err(TracesDomainError::ProjectNotFound);
        }

        let user_id_obj = UserId::new(user_id.to_string());
        let org_id = OrgId::new(project.organization_id().as_str().to_string());

        let membership = self
            .member_repo
            .find_by_org_and_user(&org_id, &user_id_obj)
            .await
            .map_err(|e| TracesDomainError::InternalError(e.to_string()))?;

        if membership.is_none() {
            return Err(TracesDomainError::NotAuthorized);
        }

        Ok(())
    }

    fn span_to_response(span: &Span) -> SpanResponse {
        let duration_ms = span.duration_ns().map(|ns| ns as f64 / 1_000_000.0);

        SpanResponse {
            id: span.id().to_string(),
            trace_id: span.trace_id().to_string(),
            span_id: span.span_id().to_string(),
            parent_span_id: span.parent_span_id().map(String::from),
            name: span.name().to_string(),
            kind: span.kind().as_str().to_string(),
            start_time: span.start_time(),
            end_time: span.end_time(),
            duration_ms,
            status: span.status().as_str().to_string(),
            status_message: span.status_message().map(String::from),
            service_name: span.service_name().map(String::from),
            service_version: span.service_version().map(String::from),
            resource_attributes: span.resource_attributes().clone(),
            attributes: span.attributes().clone(),
            events: span
                .events()
                .iter()
                .map(|e| SpanEventResponse {
                    name: e.name.clone(),
                    timestamp: e.timestamp,
                    attributes: e.attributes.clone(),
                })
                .collect(),
            links: span
                .links()
                .iter()
                .map(|l| SpanLinkResponse {
                    trace_id: l.trace_id.clone(),
                    span_id: l.span_id.clone(),
                    attributes: l.attributes.clone(),
                })
                .collect(),
        }
    }

    /// Ingest spans (called via API key auth, no user verification needed)
    pub async fn ingest(
        &self,
        cmd: IngestSpansCommand,
    ) -> Result<IngestSpansResponse, TracesDomainError> {
        let project_id = ProjectId::new(cmd.project_id);

        let mut spans = Vec::with_capacity(cmd.spans.len());

        for input in cmd.spans {
            let kind = input
                .kind
                .as_deref()
                .map(SpanKind::from_str)
                .transpose()?
                .unwrap_or_default();

            let status = input
                .status
                .as_deref()
                .map(SpanStatusCode::from_str)
                .transpose()?
                .unwrap_or_default();

            let events: Vec<SpanEvent> = input
                .events
                .into_iter()
                .map(|e| SpanEvent {
                    name: e.name,
                    timestamp: e.timestamp,
                    attributes: e.attributes,
                })
                .collect();

            let links: Vec<SpanLink> = input
                .links
                .into_iter()
                .map(|l| SpanLink {
                    trace_id: l.trace_id,
                    span_id: l.span_id,
                    attributes: l.attributes,
                })
                .collect();

            let span = Span::new(
                self.id_generator.generate(),
                project_id.clone(),
                input.trace_id,
                input.span_id,
                input.parent_span_id,
                input.name,
                kind,
                input.start_time,
                input.end_time,
                status,
                input.status_message,
                input.service_name,
                input.service_version,
                input.resource_attributes,
                input.attributes,
                events,
                links,
            );

            spans.push(span);
        }

        let ingested = self.spans_repo.save_batch(&spans).await?;

        Ok(IngestSpansResponse { ingested })
    }

    /// Search traces (requires user auth)
    pub async fn search_traces(
        &self,
        cmd: SearchTracesCommand,
    ) -> Result<TraceSearchResponse, TracesDomainError> {
        let project_id = ProjectId::new(cmd.project_id);
        self.verify_project_access(&project_id, &cmd.requesting_user_id)
            .await?;

        let status = cmd
            .filters
            .status
            .as_deref()
            .map(SpanStatusCode::from_str)
            .transpose()?;

        let filters = TraceFilters {
            service_name: cmd.filters.service_name,
            span_name: cmd.filters.span_name,
            status,
            start_time: cmd.filters.start_time,
            end_time: cmd.filters.end_time,
            min_duration_ns: cmd.filters.min_duration_ms.map(|ms| ms * 1_000_000),
            max_duration_ns: cmd.filters.max_duration_ms.map(|ms| ms * 1_000_000),
        };

        let pagination = Pagination {
            limit: cmd.filters.limit.unwrap_or(100).min(1000),
            offset: cmd.filters.offset.unwrap_or(0),
        };

        let result = self
            .spans_repo
            .search_traces(&project_id, &filters, &pagination)
            .await?;

        let traces = result
            .traces
            .into_iter()
            .map(|t| TraceSummaryResponse {
                trace_id: t.trace_id,
                root_span_name: t.root_span_name,
                services: t.service_names,
                span_count: t.span_count,
                error_count: t.error_count,
                start_time: t.start_time,
                end_time: t.end_time,
                duration_ms: t.duration_ns.map(|ns| ns as f64 / 1_000_000.0),
            })
            .collect();

        Ok(TraceSearchResponse {
            traces,
            total: result.total,
        })
    }

    /// Get a specific trace with all spans (requires user auth)
    pub async fn get_trace(&self, cmd: GetTraceCommand) -> Result<TraceResponse, TracesDomainError> {
        let project_id = ProjectId::new(cmd.project_id);
        self.verify_project_access(&project_id, &cmd.requesting_user_id)
            .await?;

        let spans = self.spans_repo.get_trace(&project_id, &cmd.trace_id).await?;

        if spans.is_empty() {
            return Err(TracesDomainError::TraceNotFound);
        }

        // Collect unique service names
        let services: Vec<String> = spans
            .iter()
            .filter_map(|s| s.service_name())
            .map(String::from)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        // Calculate trace duration from min start_time to max end_time
        let min_start = spans.iter().map(|s| s.start_time()).min();
        let max_end = spans.iter().filter_map(|s| s.end_time()).max();

        let duration_ms = match (min_start, max_end) {
            (Some(start), Some(end)) => {
                let duration_ns = (end - start).num_nanoseconds().unwrap_or(0);
                Some(duration_ns as f64 / 1_000_000.0)
            }
            _ => None,
        };

        let span_responses: Vec<SpanResponse> = spans.iter().map(Self::span_to_response).collect();

        Ok(TraceResponse {
            trace_id: cmd.trace_id,
            spans: span_responses,
            services,
            duration_ms,
        })
    }

    /// List service names for a project (requires user auth)
    pub async fn list_services(
        &self,
        cmd: ListServicesCommand,
    ) -> Result<ServicesResponse, TracesDomainError> {
        let project_id = ProjectId::new(cmd.project_id);
        self.verify_project_access(&project_id, &cmd.requesting_user_id)
            .await?;

        let services = self.spans_repo.get_service_names(&project_id).await?;

        Ok(ServicesResponse { services })
    }
}
