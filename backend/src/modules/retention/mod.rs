//! Retention cleanup module for metrics and traces
//!
//! This module provides background tasks to clean up old data based on
//! per-project retention settings.

use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;

use crate::modules::metrics::domain::MetricsRepository;
use crate::modules::projects::domain::ProjectRepository;
use crate::modules::traces::domain::SpansRepository;

/// Start the metrics retention cleanup background task.
/// Runs every hour and deletes metrics older than the project's retention period.
pub async fn start_metrics_cleanup<MR, PR>(
    metrics_repo: Arc<MR>,
    project_repo: Arc<PR>,
    interval_secs: u64,
) where
    MR: MetricsRepository + 'static,
    PR: ProjectRepository + 'static,
{
    let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));

    loop {
        interval.tick().await;

        // Get all active projects
        let projects = match project_repo.find_all_active().await {
            Ok(p) => p,
            Err(e) => {
                tracing::error!(error = %e, "Failed to fetch projects for metrics cleanup");
                continue;
            }
        };

        for project in projects {
            let retention_days = project.metrics_retention_days().value();
            let cutoff = Utc::now() - chrono::Duration::days(retention_days as i64);

            match metrics_repo.delete_before(project.id(), cutoff).await {
                Ok(deleted) if deleted > 0 => {
                    tracing::info!(
                        project_id = %project.id().as_str(),
                        deleted_count = deleted,
                        retention_days = retention_days,
                        "Cleaned up old metrics"
                    );
                }
                Err(e) => {
                    tracing::error!(
                        error = %e,
                        project_id = %project.id().as_str(),
                        "Failed to cleanup metrics"
                    );
                }
                _ => {}
            }
        }
    }
}

/// Start the traces retention cleanup background task.
/// Runs every hour and deletes spans older than the project's retention period.
pub async fn start_traces_cleanup<SR, PR>(
    spans_repo: Arc<SR>,
    project_repo: Arc<PR>,
    interval_secs: u64,
) where
    SR: SpansRepository + 'static,
    PR: ProjectRepository + 'static,
{
    let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));

    loop {
        interval.tick().await;

        // Get all active projects
        let projects = match project_repo.find_all_active().await {
            Ok(p) => p,
            Err(e) => {
                tracing::error!(error = %e, "Failed to fetch projects for traces cleanup");
                continue;
            }
        };

        for project in projects {
            let retention_days = project.traces_retention_days().value();
            let cutoff = Utc::now() - chrono::Duration::days(retention_days as i64);

            match spans_repo.delete_before(project.id(), cutoff).await {
                Ok(deleted) if deleted > 0 => {
                    tracing::info!(
                        project_id = %project.id().as_str(),
                        deleted_count = deleted,
                        retention_days = retention_days,
                        "Cleaned up old traces"
                    );
                }
                Err(e) => {
                    tracing::error!(
                        error = %e,
                        project_id = %project.id().as_str(),
                        "Failed to cleanup traces"
                    );
                }
                _ => {}
            }
        }
    }
}
