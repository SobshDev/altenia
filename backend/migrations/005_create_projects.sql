-- Create projects table
CREATE TABLE projects (
    id VARCHAR(36) PRIMARY KEY,
    organization_id VARCHAR(36) NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    retention_days INTEGER NOT NULL DEFAULT 30 CHECK (retention_days >= 1 AND retention_days <= 365),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Index for finding projects by organization
CREATE INDEX idx_projects_org ON projects(organization_id) WHERE deleted_at IS NULL;

-- Unique constraint on name within organization (case-insensitive)
CREATE UNIQUE INDEX idx_projects_name_org ON projects(organization_id, LOWER(name)) WHERE deleted_at IS NULL;
