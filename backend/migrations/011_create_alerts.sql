-- Alert channels (notification destinations)
CREATE TABLE alert_channels (
    id VARCHAR(36) PRIMARY KEY,
    project_id VARCHAR(36) NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    channel_type VARCHAR(50) NOT NULL,  -- 'webhook' for now
    config JSONB NOT NULL,              -- {"url": "...", "headers": {...}}
    is_enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Alert rules
CREATE TABLE alert_rules (
    id VARCHAR(36) PRIMARY KEY,
    project_id VARCHAR(36) NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    rule_type VARCHAR(50) NOT NULL,     -- 'error_rate', 'log_count', 'pattern_match'
    config JSONB NOT NULL,              -- Type-specific config
    threshold_value DOUBLE PRECISION NOT NULL,
    threshold_operator VARCHAR(10) NOT NULL,  -- 'gt', 'gte', 'lt', 'lte'
    time_window_seconds INT NOT NULL DEFAULT 300,
    is_enabled BOOLEAN NOT NULL DEFAULT true,
    last_evaluated_at TIMESTAMPTZ,
    last_triggered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by VARCHAR(36) NOT NULL REFERENCES users(id)
);

-- Rule-channel mapping (many-to-many)
CREATE TABLE alert_rule_channels (
    rule_id VARCHAR(36) NOT NULL REFERENCES alert_rules(id) ON DELETE CASCADE,
    channel_id VARCHAR(36) NOT NULL REFERENCES alert_channels(id) ON DELETE CASCADE,
    PRIMARY KEY (rule_id, channel_id)
);

-- Alert history (triggered alerts)
CREATE TABLE alerts (
    id VARCHAR(36) PRIMARY KEY,
    rule_id VARCHAR(36) NOT NULL REFERENCES alert_rules(id) ON DELETE CASCADE,
    project_id VARCHAR(36) NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    status VARCHAR(20) NOT NULL,  -- 'firing', 'resolved'
    triggered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ,
    trigger_value DOUBLE PRECISION,
    message TEXT,
    metadata JSONB
);

-- Indexes
CREATE INDEX idx_alert_channels_project ON alert_channels(project_id);
CREATE INDEX idx_alert_rules_project ON alert_rules(project_id);
CREATE INDEX idx_alert_rules_enabled ON alert_rules(is_enabled) WHERE is_enabled = true;
CREATE INDEX idx_alerts_project ON alerts(project_id, triggered_at DESC);
CREATE INDEX idx_alerts_rule ON alerts(rule_id, triggered_at DESC);
CREATE INDEX idx_alerts_status ON alerts(status) WHERE status = 'firing';
