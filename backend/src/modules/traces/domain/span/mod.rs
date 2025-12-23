pub mod entity;
pub mod repository;
pub mod value_objects;

pub use entity::Span;
pub use repository::{Pagination, SpansRepository, TraceFilters, TraceSearchResult, TraceSummary};
pub use value_objects::{
    SpanEvent, SpanKind, SpanLink, SpanStatusCode, MAX_ATTRIBUTES_PER_SPAN, MAX_SPANS_PER_TRACE,
};
