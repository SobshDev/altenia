-- Add display_name column to users table
-- Allows users to set a custom display name (1-30 characters)
ALTER TABLE users ADD COLUMN display_name VARCHAR(30);
