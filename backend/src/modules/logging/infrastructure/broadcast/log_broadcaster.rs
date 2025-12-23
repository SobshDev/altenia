use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgListener;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Payload from pg_notify for new logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogNotification {
    pub project_id: String,
    pub id: String,
    pub level: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub source: Option<String>,
}

/// Broadcaster for real-time log streaming via SSE
/// Listens to PostgreSQL LISTEN/NOTIFY and broadcasts to project subscribers
pub struct LogBroadcaster {
    /// Map of project_id -> broadcast channel sender
    channels: RwLock<HashMap<String, broadcast::Sender<LogNotification>>>,
    /// Channel capacity
    capacity: usize,
}

impl LogBroadcaster {
    pub fn new(capacity: usize) -> Self {
        Self {
            channels: RwLock::new(HashMap::new()),
            capacity,
        }
    }

    /// Subscribe to a project's log stream
    pub async fn subscribe(&self, project_id: &str) -> broadcast::Receiver<LogNotification> {
        let mut channels = self.channels.write().await;

        if let Some(sender) = channels.get(project_id) {
            sender.subscribe()
        } else {
            // Create new channel for this project
            let (tx, rx) = broadcast::channel(self.capacity);
            channels.insert(project_id.to_string(), tx);
            rx
        }
    }

    /// Broadcast a log notification to all subscribers of a project
    pub async fn broadcast(&self, notification: LogNotification) {
        let channels = self.channels.read().await;

        if let Some(sender) = channels.get(&notification.project_id) {
            // Ignore send errors (no receivers)
            let _ = sender.send(notification);
        }
    }

    /// Get the number of active subscribers for a project
    pub async fn subscriber_count(&self, project_id: &str) -> usize {
        let channels = self.channels.read().await;
        channels
            .get(project_id)
            .map(|s| s.receiver_count())
            .unwrap_or(0)
    }

    /// Clean up empty channels (no subscribers)
    pub async fn cleanup_empty_channels(&self) {
        let mut channels = self.channels.write().await;
        channels.retain(|_, sender| sender.receiver_count() > 0);
    }
}

/// Start the PostgreSQL LISTEN/NOTIFY listener
/// This should be spawned as a background task
pub async fn start_log_listener(
    pool: Arc<PgPool>,
    broadcaster: Arc<LogBroadcaster>,
) -> Result<(), sqlx::Error> {
    let mut listener = PgListener::connect_with(pool.as_ref()).await?;
    listener.listen("new_log").await?;

    tracing::info!("Log listener started, listening for new_log notifications");

    loop {
        match listener.recv().await {
            Ok(notification) => {
                let payload = notification.payload();
                match serde_json::from_str::<LogNotification>(payload) {
                    Ok(log_notification) => {
                        broadcaster.broadcast(log_notification).await;
                    }
                    Err(e) => {
                        tracing::warn!(
                            error = %e,
                            payload = %payload,
                            "Failed to parse log notification"
                        );
                    }
                }
            }
            Err(e) => {
                tracing::error!(error = %e, "Error receiving notification");
                // Try to reconnect after a delay
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

                // Attempt to re-listen
                if let Err(e) = listener.listen("new_log").await {
                    tracing::error!(error = %e, "Failed to re-listen after error");
                }
            }
        }
    }
}

/// Periodic cleanup task for empty channels
pub async fn start_cleanup_task(broadcaster: Arc<LogBroadcaster>) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));

    loop {
        interval.tick().await;
        broadcaster.cleanup_empty_channels().await;
    }
}
