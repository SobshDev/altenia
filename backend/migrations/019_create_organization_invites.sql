-- Organization invites table for pending membership invitations
CREATE TABLE IF NOT EXISTS organization_invites (
    id VARCHAR(36) PRIMARY KEY,
    organization_id VARCHAR(36) NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    inviter_id VARCHAR(36) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    invitee_email VARCHAR(255) NOT NULL,
    invitee_id VARCHAR(36) REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(20) NOT NULL CHECK (role IN ('admin', 'member')),
    status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'accepted', 'declined', 'expired')),
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Ensure only one pending invite per org/email combination
CREATE UNIQUE INDEX IF NOT EXISTS idx_org_invites_unique_pending
    ON organization_invites(organization_id, invitee_email) WHERE status = 'pending';

-- Fast lookup for user's pending invites
CREATE INDEX IF NOT EXISTS idx_org_invites_invitee_id_pending
    ON organization_invites(invitee_id) WHERE status = 'pending';

-- For cleanup task to find expired invites
CREATE INDEX IF NOT EXISTS idx_org_invites_expires_pending
    ON organization_invites(expires_at) WHERE status = 'pending';

-- For listing org's invites
CREATE INDEX IF NOT EXISTS idx_org_invites_org_id
    ON organization_invites(organization_id, created_at DESC);
