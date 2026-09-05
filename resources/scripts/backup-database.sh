#!/usr/bin/env bash
# Create a MySQL/MariaDB dump using ROBOMINER_DATABASE_URL or /etc/robominer/robominer.env.
#
# Usage:
#   resources/scripts/backup-database.sh
#   resources/scripts/backup-database.sh /path/to/backup.sql.gz
#   ROBOMINER_DATABASE_URL='mysql://user:pass@host/db' resources/scripts/backup-database.sh
#   resources/scripts/backup-database.sh --env-file /etc/robominer/robominer.env
#
# Defaults:
#   env-file: /etc/robominer/robominer.env (used when ROBOMINER_DATABASE_URL is unset)
#   output: ./robominer-<database>-<UTC-timestamp>.sql.gz (gzip unless output ends in .sql)

set -euo pipefail

ENV_FILE="/etc/robominer/robominer.env"
OUTPUT=""

usage() {
    cat <<'EOF'
Usage:
  resources/scripts/backup-database.sh [options] [output-file]

Options:
  --env-file PATH   Path to robominer.env (default: /etc/robominer/robominer.env)
  -h, --help        Show this help

Database URL resolution:
  1. ROBOMINER_DATABASE_URL in the process environment
  2. ROBOMINER_DATABASE_URL from --env-file / robominer.env

If output-file is omitted, writes a timestamped .sql.gz in the current directory.
If output-file ends with .sql, the dump is left uncompressed; otherwise it is gzipped
(and .gz is appended when missing).
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --env-file)
            [[ $# -ge 2 ]] || { echo "--env-file requires a path" >&2; exit 1; }
            ENV_FILE="$2"
            shift 2
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
            if [[ -n "${OUTPUT}" ]]; then
                echo "Unexpected argument: $1" >&2
                usage >&2
                exit 1
            fi
            OUTPUT="$1"
            shift
            ;;
    esac
done

if [[ $# -gt 0 ]]; then
    echo "Unexpected argument: $1" >&2
    usage >&2
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

require_command mysqldump
require_command gzip
require_command date

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

TIMESTAMP="$(date -u +%Y%m%dT%H%M%SZ)"
COMPRESS=true

if [[ -z "${OUTPUT}" ]]; then
    OUTPUT="./robominer-${DB_DATABASE}-${TIMESTAMP}.sql.gz"
elif [[ "${OUTPUT}" == *.sql ]]; then
    COMPRESS=false
elif [[ "${OUTPUT}" != *.gz ]]; then
    OUTPUT="${OUTPUT}.gz"
fi

OUTPUT_DIR="$(dirname -- "${OUTPUT}")"
mkdir -p "${OUTPUT_DIR}"

echo "Backing up database '${DB_DATABASE}' from ${DB_SERVER}"
echo "Writing ${OUTPUT}"

# Password via env so it never appears on the process command line.
export MYSQL_PWD="${DB_PASSWORD}"

dump_args=(
    --protocol=TCP
    -h "${DB_SERVER}"
    -u "${DB_USER}"
    --single-transaction
    --routines
    --triggers
    --hex-blob
    --default-character-set=utf8mb4
    "${DB_DATABASE}"
)
if [[ -n "${DB_PORT}" ]]; then
    dump_args+=(-P "${DB_PORT}")
fi

if [[ "${COMPRESS}" == true ]]; then
    mysqldump "${dump_args[@]}" | gzip -c > "${OUTPUT}"
else
    mysqldump "${dump_args[@]}" > "${OUTPUT}"
fi

unset MYSQL_PWD

BYTES="$(wc -c < "${OUTPUT}" | tr -d ' ')"
echo "Backup complete (${BYTES} bytes): ${OUTPUT}"
