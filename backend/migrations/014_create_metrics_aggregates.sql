-- Create continuous aggregates for automatic metric rollups
-- These provide efficient querying at different time resolutions
-- Note: Each aggregate must query the raw hypertable directly

-- Drop existing aggregates if they exist (for idempotent migration)
DROP MATERIALIZED VIEW IF EXISTS metrics_1d CASCADE;
DROP MATERIALIZED VIEW IF EXISTS metrics_1h CASCADE;
DROP MATERIALIZED VIEW IF EXISTS metrics_1m CASCADE;

-- 1-minute rollups (from raw metrics)
CREATE MATERIALIZED VIEW metrics_1m
WITH (timescaledb.continuous) AS
SELECT
    project_id,
    name,
    metric_type,
    time_bucket('1 minute', timestamp) AS bucket,
    AVG(value) AS avg_value,
    MIN(value) AS min_value,
    MAX(value) AS max_value,
    SUM(value) AS sum_value,
    COUNT(*) AS sample_count
FROM metrics
GROUP BY project_id, name, metric_type, bucket
WITH NO DATA;

SELECT add_continuous_aggregate_policy('metrics_1m',
    start_offset => INTERVAL '1 hour',
    end_offset => INTERVAL '1 minute',
    schedule_interval => INTERVAL '1 minute',
    if_not_exists => TRUE
);

-- 1-hour rollups (from raw metrics)
CREATE MATERIALIZED VIEW metrics_1h
WITH (timescaledb.continuous) AS
SELECT
    project_id,
    name,
    metric_type,
    time_bucket('1 hour', timestamp) AS bucket,
    AVG(value) AS avg_value,
    MIN(value) AS min_value,
    MAX(value) AS max_value,
    SUM(value) AS sum_value,
    COUNT(*) AS sample_count
FROM metrics
GROUP BY project_id, name, metric_type, bucket
WITH NO DATA;

SELECT add_continuous_aggregate_policy('metrics_1h',
    start_offset => INTERVAL '1 day',
    end_offset => INTERVAL '1 hour',
    schedule_interval => INTERVAL '1 hour',
    if_not_exists => TRUE
);

-- 1-day rollups (from raw metrics)
CREATE MATERIALIZED VIEW metrics_1d
WITH (timescaledb.continuous) AS
SELECT
    project_id,
    name,
    metric_type,
    time_bucket('1 day', timestamp) AS bucket,
    AVG(value) AS avg_value,
    MIN(value) AS min_value,
    MAX(value) AS max_value,
    SUM(value) AS sum_value,
    COUNT(*) AS sample_count
FROM metrics
GROUP BY project_id, name, metric_type, bucket
WITH NO DATA;

SELECT add_continuous_aggregate_policy('metrics_1d',
    start_offset => INTERVAL '7 days',
    end_offset => INTERVAL '1 day',
    schedule_interval => INTERVAL '1 day',
    if_not_exists => TRUE
);
