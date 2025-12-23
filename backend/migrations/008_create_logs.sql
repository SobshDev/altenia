-- Create logs table as TimescaleDB hypertable for high-volume log storage
CREATE TABLE logs (
    id VARCHAR(36) NOT NULL,
    project_id VARCHAR(36) NOT NULL,
    level VARCHAR(10) NOT NULL,
    message TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    source VARCHAR(255),
    metadata JSONB,
    trace_id VARCHAR(64),
    span_id VARCHAR(64),
    -- Composite primary key for hypertable (project_id, timestamp, id)
    PRIMARY KEY (project_id, timestamp, id)
);

-- Convert to hypertable with 1-day chunks for high volume
-- Note: Foreign key to projects is intentionally omitted for hypertable compatibility
-- Referential integrity is enforced at application level
SELECT create_hypertable('logs', 'timestamp',
    chunk_time_interval => INTERVAL '1 day',
    if_not_exists => TRUE
);

-- Enable compression (compress chunks older than 7 days)
ALTER TABLE logs SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'project_id',
    timescaledb.compress_orderby = 'timestamp DESC'
);

-- Add compression policy
SELECT add_compression_policy('logs', INTERVAL '7 days', if_not_exists => TRUE);

-- Indexes for common query patterns
-- These are automatically created on each chunk by TimescaleDB
CREATE INDEX idx_logs_project_level ON logs(project_id, level, timestamp DESC);
CREATE INDEX idx_logs_trace ON logs(trace_id, timestamp DESC) WHERE trace_id IS NOT NULL;
CREATE INDEX idx_logs_source ON logs(project_id, source, timestamp DESC) WHERE source IS NOT NULL;

-- GIN index for JSONB metadata queries
CREATE INDEX idx_logs_metadata ON logs USING GIN (metadata);
