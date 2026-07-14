#!/usr/bin/env bash

set -euo pipefail

MYSQL_HOST="${MYSQL_HOST:-127.0.0.1}"
MYSQL_PORT="${MYSQL_PORT:-3306}"
MYSQL_ROOT_PASSWORD="${MYSQL_ROOT_PASSWORD:-root}"
MYSQL_USER="${MYSQL_USER:-robominer}"
MYSQL_PASSWORD="${MYSQL_PASSWORD:-password}"
MYSQL_DATABASE="${MYSQL_DATABASE:-RoboMiner}"
FORCE_REINIT="${ROBOMINER_FORCE_DB_REINIT:-0}"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

mysql_root() {
    mysql \
        -h "${MYSQL_HOST}" \
        -P "${MYSQL_PORT}" \
        -uroot \
        -p"${MYSQL_ROOT_PASSWORD}" \
        "$@"
}

mysql_app() {
    mysql \
        -h "${MYSQL_HOST}" \
        -P "${MYSQL_PORT}" \
        -u"${MYSQL_USER}" \
        -p"${MYSQL_PASSWORD}" \
        "$@"
}

wait_for_mysql() {
    echo "Waiting for MySQL at ${MYSQL_HOST}:${MYSQL_PORT}..."
    for _ in $(seq 1 60); do
        if mysqladmin ping \
            -h "${MYSQL_HOST}" \
            -P "${MYSQL_PORT}" \
            -uroot \
            -p"${MYSQL_ROOT_PASSWORD}" \
            --silent >/dev/null 2>&1
        then
            echo "MySQL is ready."
            return 0
        fi
        sleep 2
    done

    echo "MySQL did not become ready in time." >&2
    exit 1
}

require_command() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "Required command not found: $1" >&2
        exit 1
    fi
}

schema_ready() {
    mysql_app -N -e "SELECT 1 FROM User LIMIT 1" "${MYSQL_DATABASE}" >/dev/null 2>&1
}

needs_scan_time_migration() {
    mysql_app -N -e "SHOW COLUMNS FROM Robot LIKE 'scanSpeed'" "${MYSQL_DATABASE}" 2>/dev/null \
        | grep -q scanSpeed
}

apply_pending_migrations() {
    if needs_scan_time_migration; then
        echo "Applying scanSpeed -> scanTime migration..."
        load_sql "${ROOT}/resources/database/migrations/rename_scan_speed_to_scan_time.sql"
    fi
}

load_sql() {
    local sql_file="$1"
    echo "Loading ${sql_file#"${ROOT}/"}..."
    sed '/^SET storage_engine=/d' "${sql_file}" | mysql_root "${MYSQL_DATABASE}"
}

require_command mysql
require_command mysqladmin
require_command sed

wait_for_mysql

echo "Ensuring database user ${MYSQL_USER}@localhost exists..."
mysql_root <<EOF
CREATE DATABASE IF NOT EXISTS \`${MYSQL_DATABASE}\`;
CREATE USER IF NOT EXISTS '${MYSQL_USER}'@'localhost' IDENTIFIED BY '${MYSQL_PASSWORD}';
CREATE USER IF NOT EXISTS '${MYSQL_USER}'@'%' IDENTIFIED BY '${MYSQL_PASSWORD}';
GRANT ALL PRIVILEGES ON \`${MYSQL_DATABASE}\`.* TO '${MYSQL_USER}'@'localhost';
GRANT ALL PRIVILEGES ON \`${MYSQL_DATABASE}\`.* TO '${MYSQL_USER}'@'%';
FLUSH PRIVILEGES;
EOF

if [[ "${FORCE_REINIT}" == "1" ]]; then
    echo "Force reinit requested; reloading schema and seed data."
elif schema_ready; then
    apply_pending_migrations
    echo "RoboMiner schema already present in ${MYSQL_DATABASE}; skipping reinit."
    exit 0
fi

load_sql "${ROOT}/resources/database/createDatabase.sql"
load_sql "${ROOT}/resources/database/gameData.sql"

echo "RoboMiner database initialized in ${MYSQL_DATABASE}."
