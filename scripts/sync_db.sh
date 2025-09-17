#!/usr/bin/env bash

set -e

if [ -f .env ]; then
  export DATABASE_URL_DESKTOP=$(grep '^DATABASE_URL_DESKTOP=' .env | cut -d '=' -f2-)
  export DATABASE_URL_SERVER=$(grep '^DATABASE_URL_SERVER=' .env | cut -d '=' -f2-)
fi

mkdir -p ./data

echo "===> Applying migrations for DESKTOP DB"
export DATABASE_URL="$DATABASE_URL_DESKTOP"
sqlx database create || true
sqlx migrate run --source crates/db/migrations/desktop

echo "===> Applying migrations for SERVER DB"
export DATABASE_URL="$DATABASE_URL_SERVER"
sqlx database create || true
sqlx migrate run --source crates/db/migrations/server

echo "🚀 Both desktop and server DBs are synced!"

