-- Create api_keys table
CREATE TABLE api_keys (
    id VARCHAR(36) PRIMARY KEY,
    project_id VARCHAR(36) NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    key_prefix VARCHAR(20) NOT NULL,
    key_hash VARCHAR(64) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ
);

-- Index for finding API keys by project
CREATE INDEX idx_api_keys_project ON api_keys(project_id);

-- Index for validating API keys by hash (only active keys)
CREATE INDEX idx_api_keys_hash ON api_keys(key_hash) WHERE revoked_at IS NULL;

-- Index for identifying keys by prefix
CREATE INDEX idx_api_keys_prefix ON api_keys(key_prefix);
