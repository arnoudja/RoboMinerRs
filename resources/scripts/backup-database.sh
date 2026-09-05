#!/usr/bin/env bash
# Create a MySQL/MariaDB dump using credentials from robominer.conf
# (legacy; soft-deprecated — prefer ROBOMINER_DATABASE_URL / robominer.env in a follow-up).
#
# Usage:
#   resources/scripts/backup-database.sh
#   resources/scripts/backup-database.sh /path/to/backup.sql.gz
#   resources/scripts/backup-database.sh --config /etc/robominer/robominer.conf
#   resources/scripts/backup-database.sh --config ./robominer.conf ./backups/RoboMiner.sql
#
# Defaults:
#   config: /etc/robominer/robominer.conf
#   output: ./robominer-<database>-<UTC-timestamp>.sql.gz (gzip unless output ends in .sql)

set -euo pipefail

CONFIG_FILE="/etc/robominer/robominer.conf"
OUTPUT=""

usage() {
    cat <<'EOF'
Usage:
  resources/scripts/backup-database.sh [options] [output-file]

Options:
  --config PATH   Path to robominer.conf (default: /etc/robominer/robominer.conf)
  -h, --help      Show this help

If output-file is omitted, writes a timestamped .sql.gz in the current directory.
If output-file ends with .sql, the dump is left uncompressed; otherwise it is gzipped
(and .gz is appended when missing).
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --config)
            [[ $# -ge 2 ]] || { echo "--config requires a path" >&2; exit 1; }
            CONFIG_FILE="$2"
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

conf_value() {
    local key="$1"
    # Match "key value" lines; ignore comments and blanks.
    sed -n "s/^${key}[[:space:]]\{1,\}//p" "${CONFIG_FILE}" | head -n 1
}

require_command mysqldump
require_command gzip
require_command sed
require_command date

if [[ ! -f "${CONFIG_FILE}" ]]; then
    echo "Config file not found: ${CONFIG_FILE}" >&2
    exit 1
fi

DB_SERVER="$(conf_value dbserver)"
DB_USER="$(conf_value dbuser)"
DB_PASSWORD="$(conf_value dbpassword)"
DB_DATABASE="$(conf_value dbdatabase)"

if [[ -z "${DB_SERVER}" || -z "${DB_USER}" || -z "${DB_DATABASE}" ]]; then
    echo "Config ${CONFIG_FILE} must define dbserver, dbuser, and dbdatabase" >&2
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

echo "Backing up database '${DB_DATABASE}' from ${DB_SERVER} using ${CONFIG_FILE}"
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

if [[ "${COMPRESS}" == true ]]; then
    mysqldump "${dump_args[@]}" | gzip -c > "${OUTPUT}"
else
    mysqldump "${dump_args[@]}" > "${OUTPUT}"
fi

unset MYSQL_PWD

BYTES="$(wc -c < "${OUTPUT}" | tr -d ' ')"
echo "Backup complete (${BYTES} bytes): ${OUTPUT}"
