#!/usr/bin/env bash
# Apply pending RoboMiner schema migrations (or baseline a current schema).
#
# Usage:
#   resources/scripts/migrate-database.sh
#   ROBOMINER_DATABASE_URL=mysql://... resources/scripts/migrate-database.sh
#
# Prefer the Rust runner when available:
#   cargo run -p robominer-engine -- migrate
#
# DDL steps try the app user first, then fall back to MySQL root when
# MYSQL_ROOT_PASSWORD is set (needed on older installs before CREATE/ALTER grants).

set -euo pipefail

MYSQL_HOST="${MYSQL_HOST:-127.0.0.1}"
MYSQL_PORT="${MYSQL_PORT:-3306}"
MYSQL_ROOT_PASSWORD="${MYSQL_ROOT_PASSWORD:-root}"
MYSQL_USER="${MYSQL_USER:-robominer}"
MYSQL_PASSWORD="${MYSQL_PASSWORD:-password}"
MYSQL_DATABASE="${MYSQL_DATABASE:-RoboMiner}"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MIGRATIONS_DIR="${ROOT}/resources/database/migrations"

mysql_app() {
    if [[ -n "${ROBOMINER_DATABASE_URL:-}" ]]; then
        mysql "${ROBOMINER_DATABASE_URL}" "$@"
    else
        mysql \
            -h "${MYSQL_HOST}" \
            -P "${MYSQL_PORT}" \
            -u"${MYSQL_USER}" \
            -p"${MYSQL_PASSWORD}" \
            "${MYSQL_DATABASE}" \
            "$@"
    fi
}

mysql_root() {
    mysql \
        -h "${MYSQL_HOST}" \
        -P "${MYSQL_PORT}" \
        -uroot \
        -p"${MYSQL_ROOT_PASSWORD}" \
        "${MYSQL_DATABASE}" \
        "$@"
}

# Run SQL via stdin against app user, falling back to root for DDL/grants issues.
mysql_ddl() {
    local sql
    sql="$(cat)"
    if printf '%s\n' "${sql}" | mysql_app >/dev/null 2>&1; then
        return 0
    fi
    if printf '%s\n' "${sql}" | mysql_root >/dev/null 2>&1; then
        return 0
    fi
    printf '%s\n' "${sql}" | mysql_app
}

ensure_schema_migration_table() {
    if mysql_app -N -e "SELECT 1 FROM SchemaMigration LIMIT 1" >/dev/null 2>&1; then
        return 0
    fi

    mysql_ddl <<'SQL'
CREATE TABLE IF NOT EXISTS SchemaMigration (
    version VARCHAR(64) PRIMARY KEY,
    appliedAt TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
SQL
}

applied_count() {
    mysql_app -N -e "SELECT COUNT(*) FROM SchemaMigration" 2>/dev/null || echo 0
}

column_exists() {
    local table="$1"
    local column="$2"
    mysql_app -N -e \
        "SELECT COUNT(*) FROM information_schema.columns
         WHERE table_schema = DATABASE()
           AND table_name = '${table}'
           AND column_name = '${column}'" 2>/dev/null | grep -qx '1'
}

schema_already_current() {
    mysql_app -N -e "SELECT 1 FROM User LIMIT 1" >/dev/null 2>&1 \
        && column_exists Robot scanTime \
        && ! column_exists Robot scanSpeed
}

is_applied() {
    local version="$1"
    mysql_app -N -e \
        "SELECT COUNT(*) FROM SchemaMigration WHERE version = '${version}'" \
        | grep -qx '1'
}

record_applied() {
    local version="$1"
    mysql_app -e "INSERT INTO SchemaMigration (version) VALUES ('${version}')"
}

apply_sql_file() {
    local sql_file="$1"
    local sql
    sql="$(sed '/^SET storage_engine=/d' "${sql_file}")"
    if printf '%s\n' "${sql}" | mysql_app >/dev/null 2>&1; then
        return 0
    fi
    if printf '%s\n' "${sql}" | mysql_root >/dev/null 2>&1; then
        return 0
    fi
    echo "Failed to apply ${sql_file} as ${MYSQL_USER} or root." >&2
    echo "Grant CREATE/ALTER to ${MYSQL_USER}, or run with a privileged account." >&2
    return 1
}

baseline_all() {
    local sql_file version
    shopt -s nullglob
    for sql_file in "${MIGRATIONS_DIR}"/[0-9]*_*.sql; do
        version="$(basename "${sql_file}" .sql)"
        if ! is_applied "${version}"; then
            record_applied "${version}"
            echo "${version}	baselined"
        else
            echo "${version}	already-applied"
        fi
    done
}

apply_pending() {
    local sql_file version
    shopt -s nullglob
    local found=0
    for sql_file in "${MIGRATIONS_DIR}"/[0-9]*_*.sql; do
        found=1
        version="$(basename "${sql_file}" .sql)"
        if is_applied "${version}"; then
            echo "${version}	already-applied"
            continue
        fi
        echo "Applying ${version}..."
        apply_sql_file "${sql_file}"
        record_applied "${version}"
        echo "${version}	applied"
    done
    if [[ "${found}" -eq 0 ]]; then
        echo "no-migrations"
    fi
}

main() {
    if ! command -v mysql >/dev/null 2>&1; then
        echo "Required command not found: mysql" >&2
        exit 1
    fi

    ensure_schema_migration_table

    if [[ "$(applied_count | tr -d '[:space:]')" == "0" ]] && schema_already_current; then
        echo "Current schema detected with empty SchemaMigration; baselining..."
        baseline_all
        return 0
    fi

    apply_pending
}

main "$@"
