#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

echo "==> Starting test database and redis..."
docker compose -f docker-compose.test.yml up -d

echo "==> Waiting for services to be ready..."
sleep 5

# Wait for PostgreSQL
until docker compose -f docker-compose.test.yml exec -T test-db pg_isready -U htyuc -d htyuc_test; do
    echo "Waiting for PostgreSQL..."
    sleep 2
done

# Wait for Redis
until docker compose -f docker-compose.test.yml exec -T test-redis redis-cli ping; do
    echo "Waiting for Redis..."
    sleep 2
done

echo "==> Running diesel migrations..."
cd htyuc_models
DATABASE_URL="postgres://htyuc:htyuc@localhost:5433/htyuc_test" diesel setup || true
DATABASE_URL="postgres://htyuc:htyuc@localhost:5433/htyuc_test" diesel migration run
cd "$PROJECT_ROOT"

echo "==> Initializing test data..."
PGPASSWORD=htyuc psql -h localhost -p 5433 -U htyuc -d htyuc_test -f htyuc/tests/fixtures/init_test_data.sql

echo "==> Running tests..."
export UC_DB_URL="postgres://htyuc:htyuc@localhost:5433/htyuc_test"
export REDIS_HOST="localhost"
export REDIS_PORT="6380"
export JWT_KEY="test_jwt_key_for_testing_only_1234567890"
export POOL_SIZE="5"
export TOKEN_EXPIRATION_DAYS="7"
export EXPIRATION_DAYS="7"
export SKIP_POST_LOGIN="true"
export SKIP_REGISTRATION="true"
export print_debug="true"

cargo test --package htyuc --test e2e_auth_tests -- --test-threads=1 --nocapture

TEST_EXIT_CODE=$?

echo "==> Cleaning up..."
docker compose -f docker-compose.test.yml down -v

exit $TEST_EXIT_CODE
