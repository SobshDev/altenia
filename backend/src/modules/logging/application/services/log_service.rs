use std::io::{Cursor, Write};
use std::sync::Arc;

use chrono::Utc;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::modules::auth::application::ports::IdGenerator;
use crate::modules::auth::domain::UserId;
use crate::modules::logging::application::dto::*;
use crate::modules::logging::domain::{
    LogDomainError, LogEntry, LogFilters, LogId, LogLevel, LogRepository, LogStats,
    MetadataFilter, MetadataOperator, Pagination, SortOrder, SpanId, TraceId,
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

    /// Get a reference to the log repository (for use by alert evaluator)
    pub fn log_repo(&self) -> Arc<LR> {
        self.log_repo.clone()
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

        // Convert metadata filter inputs to domain MetadataFilter
        let metadata_filters = filters
            .metadata_filters
            .unwrap_or_default()
            .into_iter()
            .map(|input| {
                let operator = MetadataOperator::from_str(&input.operator)?;
                MetadataFilter::new(input.key, operator, input.value)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(LogFilters {
            levels,
            start_time: filters.start_time,
            end_time: filters.end_time,
            source: filters.source,
            search: filters.search,
            trace_id: filters.trace_id,
            metadata_filters,
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

    /// Get metrics for dashboard charts
    pub async fn get_metrics(&self, query: MetricsQuery) -> Result<MetricsResponse, LogDomainError> {
        let project_id = ProjectId::new(query.project_id.clone());

        // Verify user access
        self.verify_project_access(&project_id, &query.requesting_user_id)
            .await?;

        let bucket_interval = query.bucket.to_interval();

        // Fetch all metrics data in parallel
        let (volume_result, levels_result, sources_result, stats_result) = tokio::join!(
            self.log_repo.get_volume_over_time(
                &project_id,
                bucket_interval,
                query.start_time,
                query.end_time
            ),
            self.log_repo.get_levels_over_time(
                &project_id,
                bucket_interval,
                query.start_time,
                query.end_time
            ),
            self.log_repo.get_top_sources(
                &project_id,
                query.top_sources_limit,
                query.start_time,
                query.end_time
            ),
            self.log_repo.get_stats(&project_id)
        );

        let volume_data = volume_result?;
        let levels_data = levels_result?;
        let sources_data = sources_result?;
        let stats = stats_result?;

        // Convert volume data
        let volume_over_time: Vec<TimeBucketCount> = volume_data
            .into_iter()
            .map(|(bucket, count)| TimeBucketCount { bucket, count })
            .collect();

        // Calculate error rate over time from levels data
        // Group by bucket, calculate error percentage
        let mut bucket_totals: std::collections::HashMap<chrono::DateTime<chrono::Utc>, i64> =
            std::collections::HashMap::new();
        let mut bucket_errors: std::collections::HashMap<chrono::DateTime<chrono::Utc>, i64> =
            std::collections::HashMap::new();

        for (level, bucket, count) in &levels_data {
            *bucket_totals.entry(*bucket).or_insert(0) += count;
            if level == "error" || level == "fatal" {
                *bucket_errors.entry(*bucket).or_insert(0) += count;
            }
        }

        let mut error_rate_over_time: Vec<ErrorRatePoint> = bucket_totals
            .into_iter()
            .map(|(bucket, total)| {
                let errors = bucket_errors.get(&bucket).copied().unwrap_or(0);
                let rate = if total > 0 {
                    (errors as f64 / total as f64) * 100.0
                } else {
                    0.0
                };
                ErrorRatePoint { bucket, rate }
            })
            .collect();
        error_rate_over_time.sort_by_key(|p| p.bucket);

        // Group levels data by level for time series
        let mut levels_map: std::collections::HashMap<String, Vec<TimeBucketCount>> =
            std::collections::HashMap::new();
        for (level, bucket, count) in levels_data {
            levels_map
                .entry(level)
                .or_default()
                .push(TimeBucketCount { bucket, count });
        }

        let levels_over_time: Vec<LevelTimeSeries> = levels_map
            .into_iter()
            .map(|(level, mut data)| {
                data.sort_by_key(|d| d.bucket);
                LevelTimeSeries { level, data }
            })
            .collect();

        // Convert top sources
        let top_sources: Vec<SourceCount> = sources_data
            .into_iter()
            .map(|(source, count, error_count)| SourceCount {
                source,
                count,
                error_count,
            })
            .collect();

        // Convert stats to response
        let counts_by_level = stats
            .counts_by_level
            .into_iter()
            .map(|(level, count)| LevelCount {
                level: level.to_string(),
                count,
            })
            .collect();

        let summary = LogStatsResponse {
            total_count: stats.total_count,
            counts_by_level,
            oldest_log: stats.oldest_log,
            newest_log: stats.newest_log,
        };

        Ok(MetricsResponse {
            volume_over_time,
            levels_over_time,
            error_rate_over_time,
            top_sources,
            summary,
        })
    }

    /// Export logs to a ZIP file containing JSON
    pub async fn export_logs(
        &self,
        project_id: &str,
        request: ExportLogsRequest,
        requesting_user_id: &str,
    ) -> Result<Vec<u8>, LogDomainError> {
        let project_id_typed = ProjectId::new(project_id.to_string());

        // Verify user access
        self.verify_project_access(&project_id_typed, requesting_user_id)
            .await?;

        // Get project for metadata
        let project = self
            .project_repo
            .find_by_id(&project_id_typed)
            .await
            .map_err(|e| LogDomainError::InternalError(e.to_string()))?
            .ok_or(LogDomainError::ProjectNotFound)?;

        // Build filters from request
        let filters = LogFilters {
            levels: request
                .levels
                .clone()
                .map(|levels| {
                    levels
                        .into_iter()
                        .filter_map(|l| LogLevel::from_str(&l).ok())
                        .collect()
                })
                .filter(|v: &Vec<LogLevel>| !v.is_empty()),
            start_time: request.start_time,
            end_time: request.end_time,
            source: request.source.clone(),
            search: request.search.clone(),
            trace_id: request.trace_id.clone(),
            metadata_filters: Vec::new(),
        };

        let max_logs = request.max_logs.unwrap_or(100_000);
        let batch_size = 10_000i64;
        let mut all_logs: Vec<LogResponse> = Vec::new();
        let mut offset = 0i64;

        // Fetch logs in batches
        loop {
            let pagination = Pagination {
                limit: batch_size.min(max_logs - offset),
                offset,
            };

            let result = self
                .log_repo
                .query(&project_id_typed, &filters, &pagination, SortOrder::Descending)
                .await?;

            let batch_len = result.logs.len() as i64;

            // Convert logs to response format
            for log in result.logs {
                all_logs.push(LogResponse {
                    id: log.id().as_str().to_string(),
                    level: log.level().to_string(),
                    message: log.message().to_string(),
                    timestamp: log.timestamp(),
                    received_at: log.received_at(),
                    source: log.source().map(|s| s.to_string()),
                    metadata: log.metadata().cloned(),
                    trace_id: log.trace_id().map(|t| t.as_str().to_string()),
                    span_id: log.span_id().map(|s| s.as_str().to_string()),
                });
            }

            offset += batch_len;

            // Stop if we've fetched all logs or reached the limit
            if batch_len < batch_size || offset >= max_logs {
                break;
            }
        }

        // Create metadata
        let metadata = ExportMetadata {
            project_id: project_id.to_string(),
            project_name: project.name().as_str().to_string(),
            exported_at: Utc::now(),
            total_logs: all_logs.len() as i64,
            filters: ExportFiltersMetadata {
                levels: request.levels,
                start_time: request.start_time,
                end_time: request.end_time,
                source: request.source,
                search: request.search,
                trace_id: request.trace_id,
            },
        };

        // Create ZIP file in memory
        let mut buffer = Cursor::new(Vec::new());
        {
            let mut zip = ZipWriter::new(&mut buffer);
            let options = SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);

            // Add metadata.json
            zip.start_file("metadata.json", options)
                .map_err(|e| LogDomainError::InternalError(format!("ZIP error: {}", e)))?;
            let metadata_json = serde_json::to_string_pretty(&metadata)
                .map_err(|e| LogDomainError::InternalError(format!("JSON error: {}", e)))?;
            zip.write_all(metadata_json.as_bytes())
                .map_err(|e| LogDomainError::InternalError(format!("ZIP write error: {}", e)))?;

            // Add logs.json
            zip.start_file("logs.json", options)
                .map_err(|e| LogDomainError::InternalError(format!("ZIP error: {}", e)))?;
            let logs_json = serde_json::to_string_pretty(&all_logs)
                .map_err(|e| LogDomainError::InternalError(format!("JSON error: {}", e)))?;
            zip.write_all(logs_json.as_bytes())
                .map_err(|e| LogDomainError::InternalError(format!("ZIP write error: {}", e)))?;

            zip.finish()
                .map_err(|e| LogDomainError::InternalError(format!("ZIP finish error: {}", e)))?;
        }

        Ok(buffer.into_inner())
    }
}
