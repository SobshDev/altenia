use axum::{
    extract::{Path, Query, State},
    response::sse::{Event, KeepAlive, Sse},
    Extension,
};
use futures::stream::Stream;
use serde::Deserialize;
use std::{convert::Infallible, sync::Arc, time::Duration};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt as TokioStreamExt;

use crate::modules::auth::domain::UserId;
use crate::modules::auth::infrastructure::http::extractors::AuthClaims;
use crate::modules::logging::infrastructure::broadcast::LogBroadcaster;
use crate::modules::organizations::domain::{OrgId, OrganizationMemberRepository};
use crate::modules::projects::domain::{ProjectId, ProjectRepository};

/// Query parameters for SSE stream filtering
#[derive(Debug, Deserialize)]
pub struct StreamFilters {
    #[serde(default)]
    pub levels: Option<String>, // Comma-separated list of levels to include
    #[serde(default)]
    pub source: Option<String>, // Filter by source
}

/// SSE handler for real-time log streaming
pub async fn stream_logs<PR, MR>(
    State((broadcaster, project_repo, member_repo)): State<(
        Arc<LogBroadcaster>,
        Arc<PR>,
        Arc<MR>,
    )>,
    Extension(claims): Extension<AuthClaims>,
    Path(project_id): Path<String>,
    Query(filters): Query<StreamFilters>,
) -> Result<
    Sse<impl Stream<Item = Result<Event, Infallible>>>,
    axum::http::StatusCode,
>
where
    PR: ProjectRepository,
    MR: OrganizationMemberRepository,
{
    // Verify user has access to project
    let project_id_vo = ProjectId::new(project_id.clone());

    let project = project_repo
        .find_by_id(&project_id_vo)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(axum::http::StatusCode::NOT_FOUND)?;

    if project.is_deleted() {
        return Err(axum::http::StatusCode::NOT_FOUND);
    }

    // Verify user is a member of the organization
    let user_id = UserId::new(claims.user_id);
    let org_id = OrgId::new(project.organization_id().as_str().to_string());

    let membership = member_repo
        .find_by_org_and_user(&org_id, &user_id)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    if membership.is_none() {
        return Err(axum::http::StatusCode::FORBIDDEN);
    }

    // Parse level filters
    let level_filter: Option<Vec<String>> = filters.levels.map(|s| {
        s.split(',')
            .map(|l| l.trim().to_lowercase())
            .filter(|l| !l.is_empty())
            .collect()
    });

    let source_filter = filters.source;

    // Subscribe to the project's log stream
    let rx = broadcaster.subscribe(&project_id).await;

    // Convert broadcast receiver to stream, applying filters using sync filter and map
    let stream = BroadcastStream::new(rx)
        // First, filter out errors (lagged messages)
        .filter_map(|result| result.ok())
        // Apply level filter
        .filter(move |notification| {
            if let Some(ref levels) = level_filter {
                levels.contains(&notification.level.to_lowercase())
            } else {
                true
            }
        })
        // Apply source filter
        .filter(move |notification| {
            if let Some(ref source) = source_filter {
                notification.source.as_ref() == Some(source)
            } else {
                true
            }
        })
        // Convert to SSE events
        .filter_map(|notification| {
            serde_json::to_string(&notification)
                .ok()
                .map(|event_data| Ok(Event::default().event("log").data(event_data)))
        });

    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    ))
}
