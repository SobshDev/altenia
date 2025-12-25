-- Add privacy setting for allowing/blocking incoming organization invites
ALTER TABLE users ADD COLUMN IF NOT EXISTS allow_invites BOOLEAN NOT NULL DEFAULT TRUE;
