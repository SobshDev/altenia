-- Create metrics table as TimescaleDB hypertable for time-series metrics storage
CREATE TABLE metrics (
    id VARCHAR(36) NOT NULL,
    project_id VARCHAR(36) NOT NULL,
    name VARCHAR(255) NOT NULL,
    metric_type VARCHAR(20) NOT NULL CHECK (metric_type IN ('counter', 'gauge', 'histogram')),
    value DOUBLE PRECISION NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    unit VARCHAR(50),
    description TEXT,
    tags JSONB DEFAULT '{}',
    -- Histogram-specific fields (NULL for counter/gauge)
    bucket_bounds DOUBLE PRECISION[],
    bucket_counts BIGINT[],
    histogram_sum DOUBLE PRECISION,
    histogram_count BIGINT,
    histogram_min DOUBLE PRECISION,
    histogram_max DOUBLE PRECISION,
    -- Correlation with traces
    trace_id VARCHAR(64),
    span_id VARCHAR(64),
    -- Composite primary key for hypertable
    PRIMARY KEY (project_id, timestamp, id)
);

-- Convert to hypertable with 1-hour chunks for high-frequency metrics
SELECT create_hypertable('metrics', 'timestamp',
    chunk_time_interval => INTERVAL '1 hour',
    if_not_exists => TRUE
);

-- Enable compression (compress chunks older than 1 day)
ALTER TABLE metrics SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'project_id, name',
    timescaledb.compress_orderby = 'timestamp DESC'
);

SELECT add_compression_policy('metrics', INTERVAL '1 day', if_not_exists => TRUE);

-- Indexes for common query patterns
CREATE INDEX idx_metrics_project_name ON metrics(project_id, name, timestamp DESC);
CREATE INDEX idx_metrics_project_type ON metrics(project_id, metric_type, timestamp DESC);
CREATE INDEX idx_metrics_trace ON metrics(trace_id) WHERE trace_id IS NOT NULL;

-- GIN index for tags queries
CREATE INDEX idx_metrics_tags ON metrics USING GIN (tags);
