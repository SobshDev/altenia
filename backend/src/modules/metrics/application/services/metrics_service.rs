use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;

use crate::modules::auth::application::ports::IdGenerator;
use crate::modules::auth::domain::UserId;
use crate::modules::metrics::application::dto::*;
use crate::modules::metrics::domain::{
    HistogramData, MetricFilters, MetricPoint, MetricType, MetricsDomainError, MetricsRepository,
    RollupInterval,
};
use crate::modules::organizations::domain::{OrgId, OrganizationMemberRepository};
use crate::modules::projects::domain::{ProjectId, ProjectRepository};

pub struct MetricsService<MR, PR, OMR, ID>
where
    MR: MetricsRepository,
    PR: ProjectRepository,
    OMR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    metrics_repo: Arc<MR>,
    project_repo: Arc<PR>,
    member_repo: Arc<OMR>,
    id_generator: Arc<ID>,
}

impl<MR, PR, OMR, ID> MetricsService<MR, PR, OMR, ID>
where
    MR: MetricsRepository,
    PR: ProjectRepository,
    OMR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    pub fn new(
        metrics_repo: Arc<MR>,
        project_repo: Arc<PR>,
        member_repo: Arc<OMR>,
        id_generator: Arc<ID>,
    ) -> Self {
        Self {
            metrics_repo,
            project_repo,
            member_repo,
            id_generator,
        }
    }

    async fn verify_project_access(
        &self,
        project_id: &ProjectId,
        user_id: &str,
    ) -> Result<(), MetricsDomainError> {
        let project = self
            .project_repo
            .find_by_id(project_id)
            .await
            .map_err(|e| MetricsDomainError::InternalError(e.to_string()))?
            .ok_or(MetricsDomainError::ProjectNotFound)?;

        if project.is_deleted() {
            return Err(MetricsDomainError::ProjectNotFound);
        }

        let user_id_obj = UserId::new(user_id.to_string());
        let org_id = OrgId::new(project.organization_id().as_str().to_string());

        let membership = self
            .member_repo
            .find_by_org_and_user(&org_id, &user_id_obj)
            .await
            .map_err(|e| MetricsDomainError::InternalError(e.to_string()))?;

        if membership.is_none() {
            return Err(MetricsDomainError::NotAuthorized);
        }

        Ok(())
    }

    /// Ingest metrics (called via API key auth, no user verification needed)
    pub async fn ingest(
        &self,
        cmd: IngestMetricsCommand,
    ) -> Result<IngestMetricsResponse, MetricsDomainError> {
        let project_id = ProjectId::new(cmd.project_id);

        let mut metric_points = Vec::with_capacity(cmd.metrics.len());

        for input in cmd.metrics {
            let metric_type = MetricType::from_str(&input.metric_type)?;
            let timestamp = input.timestamp.unwrap_or_else(Utc::now);
            let id = self.id_generator.generate();

            let metric = if metric_type == MetricType::Histogram {
                // Validate histogram data is present
                let bucket_bounds = input.bucket_bounds.unwrap_or_default();
                let bucket_counts = input.bucket_counts.unwrap_or_default();
                let histogram_sum = input.histogram_sum.unwrap_or(0.0);
                let histogram_count = input.histogram_count.unwrap_or(0);
                let histogram_min = input.histogram_min.unwrap_or(0.0);
                let histogram_max = input.histogram_max.unwrap_or(0.0);

                let histogram_data = HistogramData::new(
                    bucket_bounds,
                    bucket_counts,
                    histogram_sum,
                    histogram_count,
                    histogram_min,
                    histogram_max,
                )?;

                MetricPoint::new_histogram(
                    id,
                    project_id.clone(),
                    input.name,
                    input.value,
                    timestamp,
                    input.unit,
                    input.description,
                    input.tags,
                    histogram_data,
                    input.trace_id,
                    input.span_id,
                )
            } else {
                MetricPoint::new(
                    id,
                    project_id.clone(),
                    input.name,
                    metric_type,
                    input.value,
                    timestamp,
                    input.unit,
                    input.description,
                    input.tags,
                    input.trace_id,
                    input.span_id,
                )
            };

            metric_points.push(metric);
        }

        let ingested = self.metrics_repo.save_batch(&metric_points).await?;

        Ok(IngestMetricsResponse { ingested })
    }

    /// Query metrics (requires user auth)
    pub async fn query(
        &self,
        cmd: QueryMetricsCommand,
    ) -> Result<MetricQueryResponse, MetricsDomainError> {
        let project_id = ProjectId::new(cmd.project_id);
        self.verify_project_access(&project_id, &cmd.requesting_user_id)
            .await?;

        let rollup = cmd
            .filters
            .rollup
            .as_deref()
            .map(RollupInterval::from_str)
            .unwrap_or_default();

        // Convert types to MetricType
        let metric_types = cmd.filters.types.map(|types| {
            types
                .iter()
                .filter_map(|t| MetricType::from_str(t).ok())
                .collect()
        });

        // Convert tags HashMap to Vec of tuples
        let tags = cmd.filters.tags.map(|t| t.into_iter().collect());

        let filters = MetricFilters {
            names: cmd.filters.names,
            metric_types,
            start_time: cmd.filters.start_time,
            end_time: cmd.filters.end_time,
            tags,
            trace_id: cmd.filters.trace_id,
        };

        let result = self
            .metrics_repo
            .query(
                &project_id,
                &filters,
                rollup,
                cmd.filters.limit,
                cmd.filters.offset,
            )
            .await?;

        let data = result
            .metrics
            .into_iter()
            .map(|m| MetricDataPoint {
                name: m.name,
                metric_type: m.metric_type,
                timestamp: m.bucket,
                avg_value: m.avg_value,
                min_value: m.min_value,
                max_value: m.max_value,
                sum_value: m.sum_value,
                sample_count: m.sample_count,
            })
            .collect();

        Ok(MetricQueryResponse {
            data,
            total: result.total,
        })
    }

    /// List metric names for a project (requires user auth)
    pub async fn list_names(
        &self,
        cmd: ListMetricNamesCommand,
    ) -> Result<MetricNamesResponse, MetricsDomainError> {
        let project_id = ProjectId::new(cmd.project_id);
        self.verify_project_access(&project_id, &cmd.requesting_user_id)
            .await?;

        let names = self.metrics_repo.get_metric_names(&project_id).await?;

        Ok(MetricNamesResponse { names })
    }
}
