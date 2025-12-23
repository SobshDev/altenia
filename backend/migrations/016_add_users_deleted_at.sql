-- Add soft delete support for users
-- Data is retained for 30 days after deletion for recovery purposes
ALTER TABLE users ADD COLUMN deleted_at TIMESTAMPTZ;

-- Index for efficient filtering of active users
CREATE INDEX idx_users_deleted_at ON users(deleted_at) WHERE deleted_at IS NULL;
