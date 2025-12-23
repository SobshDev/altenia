-- Create spans table as TimescaleDB hypertable for distributed tracing
CREATE TABLE spans (
    id VARCHAR(36) NOT NULL,
    project_id VARCHAR(36) NOT NULL,
    trace_id VARCHAR(64) NOT NULL,
    span_id VARCHAR(32) NOT NULL,
    parent_span_id VARCHAR(32),
    name VARCHAR(255) NOT NULL,
    kind VARCHAR(20) NOT NULL DEFAULT 'internal'
        CHECK (kind IN ('internal', 'server', 'client', 'producer', 'consumer')),
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ,
    duration_ns BIGINT,
    status VARCHAR(20) NOT NULL DEFAULT 'unset'
        CHECK (status IN ('unset', 'ok', 'error')),
    status_message TEXT,
    received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    service_name VARCHAR(255),
    service_version VARCHAR(100),
    resource_attributes JSONB DEFAULT '{}',
    attributes JSONB DEFAULT '{}',
    events JSONB DEFAULT '[]',
    links JSONB DEFAULT '[]',
    -- Composite primary key for hypertable
    PRIMARY KEY (project_id, start_time, id)
);

-- Convert to hypertable with 1-day chunks
SELECT create_hypertable('spans', 'start_time',
    chunk_time_interval => INTERVAL '1 day',
    if_not_exists => TRUE
);

-- Enable compression (compress chunks older than 3 days)
ALTER TABLE spans SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'project_id, trace_id',
    timescaledb.compress_orderby = 'start_time DESC'
);

SELECT add_compression_policy('spans', INTERVAL '3 days', if_not_exists => TRUE);

-- Indexes for common query patterns
-- Primary trace lookup
CREATE INDEX idx_spans_trace ON spans(project_id, trace_id, start_time DESC);

-- Service-based queries
CREATE INDEX idx_spans_service ON spans(project_id, service_name, start_time DESC)
    WHERE service_name IS NOT NULL;

-- Error spans for debugging
CREATE INDEX idx_spans_status_error ON spans(project_id, start_time DESC)
    WHERE status = 'error';

-- Parent-child relationship queries
CREATE INDEX idx_spans_parent ON spans(project_id, parent_span_id, start_time DESC)
    WHERE parent_span_id IS NOT NULL;

-- GIN index for attributes queries
CREATE INDEX idx_spans_attributes ON spans USING GIN (attributes);

-- GIN index for resource attributes queries
CREATE INDEX idx_spans_resource_attrs ON spans USING GIN (resource_attributes);
