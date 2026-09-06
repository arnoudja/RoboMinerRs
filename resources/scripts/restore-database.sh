#!/usr/bin/env bash
# Restore a MySQL/MariaDB dump created by backup-database.sh.
#
# Usage:
#   resources/scripts/restore-database.sh /path/to/backup.sql.gz
#   resources/scripts/restore-database.sh /path/to/backup.sql
#   ROBOMINER_DATABASE_URL='mysql://user:pass@host/db' \
#     resources/scripts/restore-database.sh backup.sql.gz
#   resources/scripts/restore-database.sh --env-file /etc/robominer/robominer.env backup.sql.gz
#
# Defaults:
#   env-file: /etc/robominer/robominer.env (used when ROBOMINER_DATABASE_URL is unset)
#
# Migrations are forward-only. Rolling back schema or data means restoring a
# backup taken before the change, then redeploying matching application binaries.
# After restore, run `robominer-engine migrate status --check` and confirm
# `GET /health/ready` (or `/health`) returns ok.

set -euo pipefail

ENV_FILE="/etc/robominer/robominer.env"
INPUT=""
YES=false

usage() {
    cat <<'EOF'
Usage:
  resources/scripts/restore-database.sh [options] <backup-file>

Options:
  --env-file PATH   Path to robominer.env (default: /etc/robominer/robominer.env)
  --yes             Skip the interactive confirmation prompt
  -h, --help        Show this help

Database URL resolution:
  1. ROBOMINER_DATABASE_URL in the process environment
  2. ROBOMINER_DATABASE_URL from --env-file / robominer.env

backup-file may be gzipped (.gz / .sql.gz) or plain .sql.

WARNING: This replaces the target database contents with the dump.
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --env-file)
            [[ $# -ge 2 ]] || { echo "--env-file requires a path" >&2; exit 1; }
            ENV_FILE="$2"
            shift 2
            ;;
        --yes)
            YES=true
            shift
            ;;
        -h | --help)
            usage
            exit 0
            ;;
        --)
            shift
            break
            ;;
        -*)
            echo "Unknown option: $1" >&2
            usage >&2
            exit 1
            ;;
        *)
            if [[ -n "${INPUT}" ]]; then
                echo "Unexpected argument: $1" >&2
                usage >&2
                exit 1
            fi
            INPUT="$1"
            shift
            ;;
    esac
done

if [[ $# -gt 0 ]]; then
    echo "Unexpected argument: $1" >&2
    usage >&2
    exit 1
fi

if [[ -z "${INPUT}" ]]; then
    echo "backup-file is required" >&2
    usage >&2
    exit 1
fi

if [[ ! -f "${INPUT}" ]]; then
    echo "Backup file not found: ${INPUT}" >&2
    exit 1
fi

require_command() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "Required command not found: $1" >&2
        exit 1
    fi
}

env_file_value() {
    local key="$1"
    [[ -f "${ENV_FILE}" ]] || return 0
    sed -n "s/^${key}=//p" "${ENV_FILE}" | head -n 1
}

parse_database_url() {
    local url="$1"
    require_command python3
    eval "$(
        python3 - "$url" <<'PY'
import sys
from urllib.parse import unquote, urlparse

u = urlparse(sys.argv[1])
if u.scheme not in ("mysql", "mysql2"):
    raise SystemExit("unsupported database URL scheme")
host = u.hostname or "127.0.0.1"
port = str(u.port) if u.port else ""
user = unquote(u.username or "")
password = unquote(u.password or "")
database = unquote((u.path or "").lstrip("/"))

def sh_escape(value: str) -> str:
    return "'" + value.replace("'", "'\"'\"'") + "'"

print(f"DB_SERVER={sh_escape(host)}")
print(f"DB_PORT={sh_escape(port)}")
print(f"DB_USER={sh_escape(user)}")
print(f"DB_PASSWORD={sh_escape(password)}")
print(f"DB_DATABASE={sh_escape(database)}")
PY
    )"
}

require_command mysql
require_command gzip

DATABASE_URL="${ROBOMINER_DATABASE_URL:-}"
if [[ -z "${DATABASE_URL}" ]]; then
    DATABASE_URL="$(env_file_value ROBOMINER_DATABASE_URL)"
fi
if [[ -z "${DATABASE_URL}" ]]; then
    echo "ROBOMINER_DATABASE_URL is not set (process env or ${ENV_FILE})." >&2
    exit 1
fi

DB_SERVER=""
DB_PORT=""
DB_USER=""
DB_PASSWORD=""
DB_DATABASE=""
parse_database_url "${DATABASE_URL}"

if [[ -z "${DB_SERVER}" || -z "${DB_USER}" || -z "${DB_DATABASE}" ]]; then
    echo "ROBOMINER_DATABASE_URL must include host, user, and database" >&2
    exit 1
fi

echo "About to restore into database '${DB_DATABASE}' on ${DB_SERVER}"
echo "Source: ${INPUT}"
echo
echo "Migrations are forward-only. Prefer restoring a pre-change backup and"
echo "matching application binaries rather than attempting SQL down-migrations."
echo

if [[ "${YES}" != true ]]; then
    if [[ ! -t 0 ]]; then
        echo "Refusing to restore without --yes when stdin is not a TTY." >&2
        exit 1
    fi
    read -r -p "Type the database name (${DB_DATABASE}) to confirm: " CONFIRM
    if [[ "${CONFIRM}" != "${DB_DATABASE}" ]]; then
        echo "Confirmation did not match; aborting." >&2
        exit 1
    fi
fi

# Password via env so it never appears on the process command line.
export MYSQL_PWD="${DB_PASSWORD}"

mysql_args=(
    --protocol=TCP
    -h "${DB_SERVER}"
    -u "${DB_USER}"
    --default-character-set=utf8mb4
    "${DB_DATABASE}"
)
if [[ -n "${DB_PORT}" ]]; then
    mysql_args+=(-P "${DB_PORT}")
fi

echo "Restoring..."
if [[ "${INPUT}" == *.gz ]]; then
    gzip -dc -- "${INPUT}" | mysql "${mysql_args[@]}"
else
    mysql "${mysql_args[@]}" < "${INPUT}"
fi

unset MYSQL_PWD

echo "Restore complete."
echo "Next steps:"
echo "  1. robominer-engine migrate status --check"
echo "  2. GET /health/ready (or /health) should report database=ok migrations=ok"
echo "  3. Redeploy application binaries that match the restored schema era if rolling back"
