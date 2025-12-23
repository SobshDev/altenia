#!/bin/bash
set -e

# Migration runner script
# Tracks applied migrations in schema_migrations table and only runs new ones

DATABASE_URL="${DATABASE_URL:-postgres://altenia:altenia_dev_password@localhost:5432/altenia}"

# Parse DATABASE_URL
# Format: postgres://user:password@host:port/database
if [[ $DATABASE_URL =~ postgres://([^:]+):([^@]+)@([^:]+):([^/]+)/(.+) ]]; then
    DB_USER="${BASH_REMATCH[1]}"
    DB_PASS="${BASH_REMATCH[2]}"
    DB_HOST="${BASH_REMATCH[3]}"
    DB_PORT="${BASH_REMATCH[4]}"
    DB_NAME="${BASH_REMATCH[5]}"
else
    echo "ERROR: Could not parse DATABASE_URL"
    exit 1
fi

export PGPASSWORD="$DB_PASS"

MIGRATIONS_DIR="${MIGRATIONS_DIR:-/app/migrations}"

echo "==> Waiting for database to be ready..."
until pg_isready -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" > /dev/null 2>&1; do
    echo "    Database not ready, waiting..."
    sleep 2
done
echo "==> Database is ready"

# Run a SQL command and return trimmed output
run_sql() {
    psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -A -c "$1" 2>/dev/null | tr -d '[:space:]'
}

# Run a SQL command (for DDL, don't suppress output)
run_sql_ddl() {
    psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "$1"
}

# Run a SQL file
run_sql_file() {
    psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -v ON_ERROR_STOP=1 -f "$1"
}

# Create schema_migrations table if it doesn't exist
echo "==> Ensuring schema_migrations table exists..."
run_sql_ddl "CREATE TABLE IF NOT EXISTS schema_migrations (
    version VARCHAR(255) PRIMARY KEY,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);" > /dev/null

# Check if a migration has been applied
is_applied() {
    local version="$1"
    local count=$(run_sql "SELECT COUNT(*) FROM schema_migrations WHERE version = '$version';")
    [[ "$count" == "1" ]]
}

# Record a migration as applied
record_migration() {
    local version="$1"
    run_sql_ddl "INSERT INTO schema_migrations (version) VALUES ('$version');" > /dev/null
}

echo "==> Running migrations from $MIGRATIONS_DIR"

# Get all migration files sorted by name
migration_files=$(find "$MIGRATIONS_DIR" -name "*.sql" -type f | sort)

applied_count=0
skipped_count=0

for file in $migration_files; do
    filename=$(basename "$file")
    version="${filename%.sql}"

    # Skip the schema_migrations file itself (it's handled above)
    if [[ "$version" == "000_schema_migrations" ]]; then
        continue
    fi

    if is_applied "$version"; then
        echo "    [SKIP] $filename (already applied)"
        skipped_count=$((skipped_count + 1))
    else
        echo "    [RUN]  $filename"
        if run_sql_file "$file"; then
            record_migration "$version"
            echo "           Applied successfully"
            applied_count=$((applied_count + 1))
        else
            echo "ERROR: Failed to apply migration $filename"
            exit 1
        fi
    fi
done

echo "==> Migration complete: $applied_count applied, $skipped_count skipped"
