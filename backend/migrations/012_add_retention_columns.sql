-- Add retention settings for metrics and traces to projects table
ALTER TABLE projects
ADD COLUMN IF NOT EXISTS metrics_retention_days INTEGER NOT NULL DEFAULT 90
    CHECK (metrics_retention_days >= 1 AND metrics_retention_days <= 365);

ALTER TABLE projects
ADD COLUMN IF NOT EXISTS traces_retention_days INTEGER NOT NULL DEFAULT 14
    CHECK (traces_retention_days >= 1 AND traces_retention_days <= 90);
