use std::sync::Arc;

use crate::modules::auth::application::ports::IdGenerator;
use crate::modules::auth::domain::UserId;
use crate::modules::logging::application::dto::*;
use crate::modules::logging::domain::{
    LogDomainError, LogEntry, LogFilters, LogId, LogLevel, LogRepository, LogStats, Pagination,
    SortOrder, SpanId, TraceId,
};
use crate::modules::organizations::domain::{OrgId, OrganizationMemberRepository};
use crate::modules::projects::domain::{ProjectId, ProjectRepository};

/// Log service - orchestrates all logging use cases
pub struct LogService<LR, PR, MR, ID>
where
    LR: LogRepository,
    PR: ProjectRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    log_repo: Arc<LR>,
    project_repo: Arc<PR>,
    member_repo: Arc<MR>,
    id_generator: Arc<ID>,
}

impl<LR, PR, MR, ID> LogService<LR, PR, MR, ID>
where
    LR: LogRepository,
    PR: ProjectRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    pub fn new(
        log_repo: Arc<LR>,
        project_repo: Arc<PR>,
        member_repo: Arc<MR>,
        id_generator: Arc<ID>,
    ) -> Self {
        Self {
            log_repo,
            project_repo,
            member_repo,
            id_generator,
        }
    }

    /// Verify user has access to project via org membership
    async fn verify_project_access(
        &self,
        project_id: &ProjectId,
        user_id: &str,
    ) -> Result<(), LogDomainError> {
        // Get project
        let project = self
            .project_repo
            .find_by_id(project_id)
            .await
            .map_err(|e| LogDomainError::InternalError(e.to_string()))?
            .ok_or(LogDomainError::ProjectNotFound)?;

        if project.is_deleted() {
            return Err(LogDomainError::ProjectDeleted);
        }

        // Verify user is member of the org
        let user_id = UserId::new(user_id.to_string());
        let org_id = OrgId::new(project.organization_id().as_str().to_string());

        let membership = self
            .member_repo
            .find_by_org_and_user(&org_id, &user_id)
            .await
            .map_err(|e| LogDomainError::InternalError(e.to_string()))?;

        if membership.is_none() {
            return Err(LogDomainError::NotOrgMember);
        }

        Ok(())
    }

    /// Ingest a batch of logs
    /// Called after API key validation (project_id comes from validated key)
    pub async fn ingest(&self, cmd: IngestLogsCommand) -> Result<IngestResponse, LogDomainError> {
        let project_id = ProjectId::new(cmd.project_id);
        let mut accepted = 0u32;
        let mut rejected = 0u32;
        let mut errors = Vec::new();
        let mut valid_logs = Vec::new();

        // Validate and convert each log entry
        for (idx, input) in cmd.logs.into_iter().enumerate() {
            match self.validate_and_convert_log(&project_id, input) {
                Ok(log_entry) => {
                    valid_logs.push(log_entry);
                }
                Err(e) => {
                    rejected += 1;
                    errors.push(format!("Log {}: {}", idx, e));
                }
            }
        }

        // Save valid logs in batch
        if !valid_logs.is_empty() {
            match self.log_repo.save_batch(&valid_logs).await {
                Ok(count) => {
                    accepted = count;
                }
                Err(e) => {
                    // If batch save fails, all are rejected
                    rejected += valid_logs.len() as u32;
                    errors.push(format!("Batch save failed: {}", e));
                }
            }
        }

        Ok(IngestResponse {
            accepted,
            rejected,
            errors,
        })
    }

    /// Validate and convert LogInput to LogEntry
    fn validate_and_convert_log(
        &self,
        project_id: &ProjectId,
        input: LogInput,
    ) -> Result<LogEntry, LogDomainError> {
        // Validate level
        let level = LogLevel::from_str(&input.level)?;

        // Validate message
        if input.message.is_empty() {
            return Err(LogDomainError::InvalidMessage(
                "Message cannot be empty".to_string(),
            ));
        }

        // Parse trace_id and span_id (they return Option, so flatten)
        let trace_id = input.trace_id.and_then(TraceId::new);
        let span_id = input.span_id.and_then(SpanId::new);

        // Create log entry
        let log_id = LogId::new(self.id_generator.generate());

        Ok(LogEntry::new(
            log_id,
            project_id.clone(),
            level,
            input.message,
            input.timestamp,
            input.source,
            input.metadata,
            trace_id,
            span_id,
        ))
    }

    /// Query logs with filters
    pub async fn query(&self, cmd: QueryLogsCommand) -> Result<LogQueryResponse, LogDomainError> {
        let project_id = ProjectId::new(cmd.project_id);

        // Verify user access
        self.verify_project_access(&project_id, &cmd.requesting_user_id)
            .await?;

        // Convert query filters
        let filters = self.convert_query_filters(cmd.filters)?;

        // Build pagination
        let pagination = Pagination {
            limit: cmd.limit.unwrap_or(100),
            offset: cmd.offset.unwrap_or(0),
        };

        // Parse sort order
        let sort = match cmd.sort.as_deref() {
            Some("asc") | Some("ascending") => SortOrder::Ascending,
            _ => SortOrder::Descending,
        };

        // Execute query
        let result = self
            .log_repo
            .query(&project_id, &filters, &pagination, sort)
            .await?;

        // Convert to response
        let logs = result
            .logs
            .into_iter()
            .map(|log| LogResponse {
                id: log.id().as_str().to_string(),
                level: log.level().to_string(),
                message: log.message().to_string(),
                timestamp: log.timestamp(),
                received_at: log.received_at(),
                source: log.source().map(|s| s.to_string()),
                metadata: log.metadata().cloned(),
                trace_id: log.trace_id().map(|t| t.as_str().to_string()),
                span_id: log.span_id().map(|s| s.as_str().to_string()),
            })
            .collect();

        Ok(LogQueryResponse {
            logs,
            total: result.total,
            has_more: result.has_more,
        })
    }

    /// Convert DTO filters to domain filters
    fn convert_query_filters(&self, filters: QueryFilters) -> Result<LogFilters, LogDomainError> {
        // Convert level strings to LogLevel enums
        let levels = filters
            .levels
            .map(|levels| {
                levels
                    .into_iter()
                    .map(|l| LogLevel::from_str(&l))
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?;

        Ok(LogFilters {
            levels,
            start_time: filters.start_time,
            end_time: filters.end_time,
            source: filters.source,
            search: filters.search,
            trace_id: filters.trace_id,
        })
    }

    /// Get log statistics for a project
    pub async fn get_stats(
        &self,
        project_id: &str,
        requesting_user_id: &str,
    ) -> Result<LogStatsResponse, LogDomainError> {
        let project_id = ProjectId::new(project_id.to_string());

        // Verify user access
        self.verify_project_access(&project_id, requesting_user_id)
            .await?;

        // Get stats
        let stats: LogStats = self.log_repo.get_stats(&project_id).await?;

        // Convert to response
        let counts_by_level = stats
            .counts_by_level
            .into_iter()
            .map(|(level, count)| LevelCount {
                level: level.to_string(),
                count,
            })
            .collect();

        Ok(LogStatsResponse {
            total_count: stats.total_count,
            counts_by_level,
            oldest_log: stats.oldest_log,
            newest_log: stats.newest_log,
        })
    }
}
