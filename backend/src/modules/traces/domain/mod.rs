pub mod errors;
pub mod span;

pub use errors::TracesDomainError;
pub use span::{
    Pagination, Span, SpanEvent, SpanKind, SpanLink, SpansRepository, SpanStatusCode,
    TraceFilters, TraceSearchResult, TraceSummary, MAX_ATTRIBUTES_PER_SPAN, MAX_SPANS_PER_TRACE,
};
