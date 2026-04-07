#!/bin/bash
set -e

# Load environment variables from .env in project root
cd "$(dirname "$0")/.."
set -a
source .env
set +a

if [ -z "$DATABASE_URL" ]; then
    echo "ERROR: DATABASE_URL not set in .env"
    exit 1
fi

# Parse DATABASE_URL (postgres://user:password@host:port/database)
# Use pg_dump with the connection string directly
TIMESTAMP=$(date -u +"%Y%m%d_%H%M%S")
BACKUP_DIR="backups"
mkdir -p "$BACKUP_DIR"
BACKUP_FILE="$BACKUP_DIR/backup_$TIMESTAMP.sql"

echo "Backing up database to $BACKUP_FILE..."
pg_dump "$DATABASE_URL" --no-owner --no-acl --clean --if-exists -F p -f "$BACKUP_FILE"

if [ $? -eq 0 ]; then
    echo "Backup successful: $BACKUP_FILE"
    # Optional: compress the backup
    gzip -f "$BACKUP_FILE"
    echo "Compressed to $BACKUP_FILE.gz"
else
    echo "Backup failed"
    exit 1
fi

# Optional: keep only last 7 backups
ls -tp "$BACKUP_DIR/" | grep -v '/$' | tail -n +8 | xargs -I {} rm -- "$BACKUP_DIR/{}" 2>/dev/null || true

echo "Backup completed."