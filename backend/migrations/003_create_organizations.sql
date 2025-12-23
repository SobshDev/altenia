-- Create organizations table for multi-tenant support
CREATE TABLE IF NOT EXISTS organizations (
    id VARCHAR(36) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(110) NOT NULL,
    is_personal BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Unique slug index (only for non-deleted orgs)
CREATE UNIQUE INDEX IF NOT EXISTS idx_org_slug_active
    ON organizations(LOWER(slug)) WHERE deleted_at IS NULL;

-- Index for finding personal orgs quickly
CREATE INDEX IF NOT EXISTS idx_org_personal
    ON organizations(is_personal) WHERE is_personal = TRUE AND deleted_at IS NULL;
