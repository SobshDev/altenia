pub mod application;
pub mod domain;
pub mod infrastructure;

// Re-export commonly used types
pub use application::{
    IngestLogsCommand, IngestResponse, LevelCount, LogInput, LogQueryResponse, LogResponse,
    LogService, LogStatsResponse, QueryFilters, QueryLogsCommand,
};
pub use domain::{
    LogDomainError, LogEntry, LogFilters, LogId, LogLevel, LogQueryResult, LogRepository,
    LogStats, Pagination, SortOrder, SpanId, TraceId,
};
pub use infrastructure::{
    ingest_routes, log_query_routes, sse_routes, start_cleanup_task, start_log_listener,
    ApiKeyContext, LogBroadcaster, LogNotification, TimescaleLogRepository,
};
