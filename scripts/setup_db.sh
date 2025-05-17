#!/bin/bash

# Load environment variables
source .env

# Create database if it doesn't exist
createdb task_manager 2>/dev/null || true

# Run migrations
sqlx database create
sqlx migrate run

echo "✅ Database setup complete!" 