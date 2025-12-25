-- Create organization_activities table for audit logging
CREATE TABLE IF NOT EXISTS organization_activities (
    id VARCHAR(36) PRIMARY KEY,
    organization_id VARCHAR(36) NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    activity_type VARCHAR(50) NOT NULL CHECK (activity_type IN (
        'org_created',
        'member_added',
        'member_removed',
        'member_role_changed',
        'org_name_changed'
    )),
    actor_id VARCHAR(36) NOT NULL REFERENCES users(id) ON DELETE SET NULL,
    target_id VARCHAR(36) REFERENCES users(id) ON DELETE SET NULL,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for efficient lookups by org (primary query pattern)
CREATE INDEX IF NOT EXISTS idx_org_activities_org_created ON organization_activities(organization_id, created_at DESC);
