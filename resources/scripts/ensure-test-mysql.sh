#!/usr/bin/env bash

set -euo pipefail

MYSQL_HOST="${MYSQL_HOST:-127.0.0.1}"
MYSQL_PORT="${MYSQL_PORT:-3306}"
MYSQL_ROOT_PASSWORD="${MYSQL_ROOT_PASSWORD:-root}"
MYSQL_USER="${MYSQL_USER:-robominer}"
MYSQL_PASSWORD="${MYSQL_PASSWORD:-password}"
MYSQL_DATABASE="${MYSQL_DATABASE:-RoboMiner}"
DOCKER_CONTAINER="${ROBOMINER_TEST_MYSQL_CONTAINER:-robominer-test-mysql}"
DOCKER_IMAGE="${ROBOMINER_TEST_MYSQL_IMAGE:-mysql:8.4}"
DOCKER_VOLUME="${ROBOMINER_TEST_MYSQL_VOLUME:-robominer-test-mysql-data}"
DOCKER_PORT="${ROBOMINER_TEST_DOCKER_PORT:-3307}"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

database_url() {
    printf 'mysql://%s:%s@%s:%s/%s' \
        "${MYSQL_USER}" "${MYSQL_PASSWORD}" "${MYSQL_HOST}" "${MYSQL_PORT}" "${MYSQL_DATABASE}"
}

log() {
    echo "$@" >&2
}

mysql_app() {
    mysql \
        -h "${MYSQL_HOST}" \
        -P "${MYSQL_PORT}" \
        -u"${MYSQL_USER}" \
        -p"${MYSQL_PASSWORD}" \
        "$@"
}

mysql_root_ping() {
    mysqladmin ping \
        -h "${MYSQL_HOST}" \
        -P "${MYSQL_PORT}" \
        -uroot \
        -p"${MYSQL_ROOT_PASSWORD}" \
        --silent >/dev/null 2>&1
}

schema_ready() {
    mysql_app -N -e "SELECT 1 FROM User LIMIT 1" "${MYSQL_DATABASE}" >/dev/null 2>&1
}

schema_migration_ready() {
    mysql_app -N -e "SELECT 1 FROM SchemaMigration LIMIT 1" "${MYSQL_DATABASE}" >/dev/null 2>&1
}

apply_migrations_for_url() {
    local database_url="$1"
    ROBOMINER_DATABASE_URL="${database_url}" \
        "${ROOT}/resources/scripts/migrate-database.sh" >&2
}

try_existing_database_url() {
    if [[ -z "${ROBOMINER_DATABASE_URL:-}" ]]; then
        return 1
    fi

    if mysql "${ROBOMINER_DATABASE_URL}" -N -e "SELECT 1 FROM User LIMIT 1" >/dev/null 2>&1; then
        if apply_migrations_for_url "${ROBOMINER_DATABASE_URL}" \
            && mysql "${ROBOMINER_DATABASE_URL}" -N -e "SELECT 1 FROM SchemaMigration LIMIT 1" >/dev/null 2>&1
        then
            log "Using existing ROBOMINER_DATABASE_URL."
            echo "${ROBOMINER_DATABASE_URL}"
            return 0
        fi
        log "ROBOMINER_DATABASE_URL schema is present but migrations cannot be applied; trying another MySQL."
    fi

    return 1
}

init_database() {
    MYSQL_HOST="${MYSQL_HOST}" MYSQL_PORT="${MYSQL_PORT}" \
        "${ROOT}/resources/scripts/init-ci-database.sh" >&2
}

try_local_mysql() {
    if ! mysql_root_ping; then
        return 1
    fi

    if schema_ready; then
        apply_migrations_for_url "$(database_url)" || true
        init_database || true
        if schema_migration_ready; then
            log "Using MySQL already running at ${MYSQL_HOST}:${MYSQL_PORT}."
            database_url
            return 0
        fi
        log "Local MySQL at ${MYSQL_HOST}:${MYSQL_PORT} lacks SchemaMigration privileges; trying Docker."
        return 1
    fi

    log "MySQL is running at ${MYSQL_HOST}:${MYSQL_PORT}, but RoboMiner schema is missing."
    if init_database && schema_migration_ready; then
        database_url
        return 0
    fi
    return 1
}

start_docker_mysql() {
    if ! command -v docker >/dev/null 2>&1; then
        return 1
    fi

    MYSQL_HOST="127.0.0.1"
    MYSQL_PORT="${DOCKER_PORT}"

    if docker ps --format '{{.Names}}' | grep -qx "${DOCKER_CONTAINER}"; then
        log "Reusing running Docker container ${DOCKER_CONTAINER} on port ${MYSQL_PORT}."
        if schema_ready; then
            init_database
            database_url
            return 0
        fi

        init_database
        database_url
        return 0
    fi

    if docker ps -a --format '{{.Names}}' | grep -qx "${DOCKER_CONTAINER}"; then
        log "Starting existing Docker container ${DOCKER_CONTAINER}..."
        docker start "${DOCKER_CONTAINER}" >/dev/null
    else
        log "Creating persistent Docker container ${DOCKER_CONTAINER} on port ${MYSQL_PORT}..."
        docker run -d \
            --name "${DOCKER_CONTAINER}" \
            -e MYSQL_ROOT_PASSWORD="${MYSQL_ROOT_PASSWORD}" \
            -e MYSQL_DATABASE="${MYSQL_DATABASE}" \
            -e MYSQL_USER="${MYSQL_USER}" \
            -e MYSQL_PASSWORD="${MYSQL_PASSWORD}" \
            -p "${MYSQL_PORT}:3306" \
            -v "${DOCKER_VOLUME}:/var/lib/mysql" \
            "${DOCKER_IMAGE}" >/dev/null
    fi

    init_database
    database_url
}

main() {
    local url=""

    if url="$(try_existing_database_url)"; then
        echo "${url}"
        return 0
    fi

    if url="$(try_local_mysql)"; then
        echo "${url}"
        return 0
    fi

    if url="$(start_docker_mysql)"; then
        echo "${url}"
        return 0
    fi

    cat >&2 <<EOF
Could not find a usable MySQL instance for RoboMiner tests.

Options:
  1. Start local MySQL and run: resources/scripts/init-ci-database.sh
  2. Set ROBOMINER_DATABASE_URL to an initialized RoboMiner database
  3. Install Docker so a persistent test container can be created automatically

Default test URL when using the Docker helper:
  mysql://${MYSQL_USER}:${MYSQL_PASSWORD}@127.0.0.1:${DOCKER_PORT}/${MYSQL_DATABASE}
EOF
    exit 1
}

main "$@"
