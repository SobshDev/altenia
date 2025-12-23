-- Create filter_presets table for per-project saved filters
CREATE TABLE filter_presets (
    id VARCHAR(36) PRIMARY KEY,
    project_id VARCHAR(36) NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    user_id VARCHAR(36) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    filter_config JSONB NOT NULL,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for finding presets by project
CREATE INDEX idx_filter_presets_project ON filter_presets(project_id);

-- Index for finding presets by user within project
CREATE INDEX idx_filter_presets_user ON filter_presets(project_id, user_id);

-- Unique constraint: only one default preset per user per project
CREATE UNIQUE INDEX idx_filter_presets_default
    ON filter_presets(project_id, user_id)
    WHERE is_default = TRUE;

-- Unique constraint: name must be unique per user per project
CREATE UNIQUE INDEX idx_filter_presets_name
    ON filter_presets(project_id, user_id, LOWER(name));
