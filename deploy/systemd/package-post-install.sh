#!/bin/sh
# Shared packaging post-install for Debian (.deb) and Arch (PKGBUILD).
# Installed as /usr/share/robominer/package-post-install.sh.
#
# Actions:
#   configure-pre  — sysusers + migrate + gameData (before Debian #DEBHELPER#)
#   configure-post — start/restart enabled units (after Debian #DEBHELPER#)
#   configure      — both phases (Arch post_install / post_upgrade)
set -e

ENV_FILE=/etc/robominer/robominer.env
ENGINE_BIN=/opt/robominer/bin/robominer-engine
GAMEDATA_SQL=/usr/share/robominer/gameData.sql
SYSUSERS_FILE=/usr/lib/sysusers.d/robominer.conf
INSTALL_PREFIX=/opt/robominer

env_value() {
    key="$1"
    if [ -f "$ENV_FILE" ]; then
        # Extract KEY=value without sourcing (avoids executing arbitrary shell).
        sed -n "s/^${key}=//p" "$ENV_FILE" | head -n 1
    fi
}

config_is_ready() {
    [ -f "$ENV_FILE" ] || [ -n "${ROBOMINER_DATABASE_URL:-}" ]
}

ensure_service_user() {
    if command -v systemd-sysusers >/dev/null 2>&1 && [ -f "$SYSUSERS_FILE" ]; then
        systemd-sysusers "$SYSUSERS_FILE" || true
    fi

    if getent passwd robominer >/dev/null 2>&1; then
        mkdir -p "$INSTALL_PREFIX" /var/lib/robominer
        chown robominer:robominer /var/lib/robominer 2>/dev/null || true
        chown robominer:robominer "$INSTALL_PREFIX" 2>/dev/null || true
        if [ -d "$INSTALL_PREFIX/static" ]; then
            chown -R robominer:robominer "$INSTALL_PREFIX/static" 2>/dev/null || true
        fi
    fi
}

run_engine() {
    if [ -f "$ENV_FILE" ]; then
        # Export env file into a subshell, then run the engine.
        bash -c '
            set -euo pipefail
            set -a
            # shellcheck disable=SC1090
            source "$1"
            set +a
            shift
            exec "$@"
        ' bash "$ENV_FILE" "$ENGINE_BIN" "$@"
        return
    fi

    if [ -n "${ROBOMINER_DATABASE_URL:-}" ]; then
        ROBOMINER_DATABASE_URL="$ROBOMINER_DATABASE_URL" "$ENGINE_BIN" "$@"
        return
    fi

    echo "RoboMiner: no database config ($ENV_FILE or ROBOMINER_DATABASE_URL)" >&2
    exit 1
}

parse_database_url() {
    # Sets dbserver/dbport/dbuser/dbpassword/dbdatabase from ROBOMINER_DATABASE_URL.
    url="$1"
    if command -v python3 >/dev/null 2>&1; then
        eval "$(python3 - "$url" <<'PY'
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
print(f"dbserver={sh_escape(host)}")
print(f"dbport={sh_escape(port)}")
print(f"dbuser={sh_escape(user)}")
print(f"dbpassword={sh_escape(password)}")
print(f"dbdatabase={sh_escape(database)}")
PY
)"
        return
    fi
    echo "RoboMiner: python3 required to parse ROBOMINER_DATABASE_URL for gameData.sql" >&2
    exit 1
}

# Prefer mariadb(1) on Arch/MariaDB hosts; mysql(1) is a deprecated alias there.
mysql_cli() {
    if command -v mariadb >/dev/null 2>&1; then
        command -v mariadb
        return 0
    fi
    if command -v mysql >/dev/null 2>&1; then
        command -v mysql
        return 0
    fi
    return 1
}

# Local packaged installs typically use plain TCP to loopback; skip TLS noise.
mysql_local_ssl_args() {
    host="$1"
    case "$host" in
        localhost | 127.0.0.1 | ::1 | "")
            cli="$(mysql_cli)" || return 0
            if "$cli" --help 2>/dev/null | grep -q -- '--skip-ssl'; then
                printf '%s\n' --skip-ssl
            elif "$cli" --help 2>/dev/null | grep -q -- '--ssl-mode'; then
                printf '%s\n' --ssl-mode=DISABLED
            fi
            ;;
    esac
}

apply_database_updates() {
    if ! config_is_ready; then
        echo "RoboMiner: no $ENV_FILE; skipping migrate/gameData."
        echo "Create $ENV_FILE (see deploy/systemd/robominer.env.example), initialize MySQL,"
        echo "then: systemctl start robominer-engine robominer-web"
        return 0
    fi

    if [ ! -x "$ENGINE_BIN" ]; then
        echo "RoboMiner: engine binary missing at $ENGINE_BIN" >&2
        exit 1
    fi

    echo "RoboMiner: applying schema migrations..."
    run_engine migrate apply
    run_engine migrate status --check

    dbserver=""
    dbport=""
    dbuser=""
    dbpassword=""
    dbdatabase=""

    database_url="${ROBOMINER_DATABASE_URL:-}"
    if [ -z "$database_url" ]; then
        database_url="$(env_value ROBOMINER_DATABASE_URL)"
    fi
    if [ -z "$database_url" ]; then
        echo "RoboMiner: $ENV_FILE is missing ROBOMINER_DATABASE_URL" >&2
        exit 1
    fi
    parse_database_url "$database_url"

    if [ -z "$dbserver" ] || [ -z "$dbuser" ] || [ -z "$dbdatabase" ]; then
        echo "RoboMiner: database connection details incomplete for gameData.sql" >&2
        exit 1
    fi

    if [ ! -f "$GAMEDATA_SQL" ]; then
        echo "RoboMiner: game data SQL missing at $GAMEDATA_SQL" >&2
        exit 1
    fi

    mysql_bin="$(mysql_cli)" || {
        echo "RoboMiner: mariadb/mysql client not found; cannot apply gameData.sql" >&2
        exit 1
    }

    echo "RoboMiner: applying gameData.sql..."
    # Strip obsolete SET storage_engine= (removed in modern MySQL/MariaDB).
    ssl_args="$(mysql_local_ssl_args "$dbserver")"
    # shellcheck disable=SC2086 # intentional optional flag expansion
    if [ -n "$dbport" ]; then
        sed '/^SET storage_engine=/d' "$GAMEDATA_SQL" | MYSQL_PWD="$dbpassword" "$mysql_bin" \
            --protocol=TCP \
            -h "$dbserver" \
            -P "$dbport" \
            -u "$dbuser" \
            $ssl_args \
            "$dbdatabase"
    else
        sed '/^SET storage_engine=/d' "$GAMEDATA_SQL" | MYSQL_PWD="$dbpassword" "$mysql_bin" \
            --protocol=TCP \
            -h "$dbserver" \
            -u "$dbuser" \
            $ssl_args \
            "$dbdatabase"
    fi
}

start_services_if_configured() {
    if ! config_is_ready; then
        return 0
    fi
    if [ ! -d /run/systemd/system ]; then
        return 0
    fi

    for unit in robominer-engine.service robominer-web.service; do
        if systemctl is-enabled "$unit" >/dev/null 2>&1; then
            # Prefer restart on upgrade so a running daemon picks up new binaries.
            if systemctl is-active --quiet "$unit"; then
                systemctl try-restart "$unit" || systemctl restart "$unit"
            else
                systemctl start "$unit" || true
            fi
        fi
    done
}

configure_pre() {
    ensure_service_user
    apply_database_updates
}

configure_post() {
    start_services_if_configured
}

case "${1:-}" in
    configure-pre)
        configure_pre
        ;;
    configure-post)
        configure_post
        ;;
    configure)
        configure_pre
        configure_post
        ;;
    *)
        echo "Usage: package-post-install.sh configure|configure-pre|configure-post" >&2
        exit 1
        ;;
esac

exit 0
