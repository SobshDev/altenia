-- Create organization_members table for membership management
CREATE TABLE IF NOT EXISTS organization_members (
    id VARCHAR(36) PRIMARY KEY,
    organization_id VARCHAR(36) NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id VARCHAR(36) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(20) NOT NULL CHECK (role IN ('owner', 'admin', 'member')),
    last_accessed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(organization_id, user_id)
);

-- Indexes for efficient lookups
CREATE INDEX IF NOT EXISTS idx_org_members_org ON organization_members(organization_id);
CREATE INDEX IF NOT EXISTS idx_org_members_user ON organization_members(user_id);

-- Index for finding last accessed org efficiently
CREATE INDEX IF NOT EXISTS idx_org_members_user_last_accessed
    ON organization_members(user_id, last_accessed_at DESC NULLS LAST);

-- Partial index for owners (useful for "last owner" checks)
CREATE INDEX IF NOT EXISTS idx_org_members_owners
    ON organization_members(organization_id) WHERE role = 'owner';
