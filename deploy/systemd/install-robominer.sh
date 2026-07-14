#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=install-common.sh
source "${SCRIPT_DIR}/install-common.sh"

ENABLE_SERVICE=false
SKIP_USER=false
SKIP_CONFIG=false
SKIP_SYSTEMD=false
INSTALL_ENGINE=true
INSTALL_WEB=true

usage() {
    cat <<'EOF'
Install the RoboMiner Rust engine and web host using deploy/systemd defaults.

Usage:
  deploy/systemd/install-robominer.sh [options]

Options:
  --enable          Enable and start both systemd services after installation.
  --engine-only     Install only robominer-engine.
  --web-only        Install only robominer-web.
  --skip-user       Skip service account creation.
  --skip-config     Do not create or update /etc/robominer/robominer.conf.
  --skip-systemd    Install binaries and assets only.
  -h, --help        Show this help.

Environment:
  Database keys (used when creating the shared config file):
    ROBOMINER_DB_SERVER
    ROBOMINER_DB_USER
    ROBOMINER_DB_PASSWORD
    ROBOMINER_DB_DATABASE

  Web keys (appended to the shared config when missing):
    ROBOMINER_WEB_HOST
    ROBOMINER_WEB_PORT
    ROBOMINER_SESSION_SECRET
    ROBOMINER_SECURE_COOKIES
    ROBOMINER_SESSION_TTL_HOURS
    ROBOMINER_SESSION_TTL_SECS

Examples:
  deploy/systemd/install-robominer.sh
  ROBOMINER_DB_SERVER=localhost ROBOMINER_DB_USER=robominer \
    ROBOMINER_DB_PASSWORD=secret ROBOMINER_DB_DATABASE=RoboMiner \
    ROBOMINER_SESSION_SECRET="$(openssl rand -hex 32)" \
    deploy/systemd/install-robominer.sh --enable
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --enable) ENABLE_SERVICE=true ;;
        --engine-only)
            INSTALL_ENGINE=true
            INSTALL_WEB=false
            ;;
        --web-only)
            INSTALL_ENGINE=false
            INSTALL_WEB=true
            ;;
        --skip-user) SKIP_USER=true ;;
        --skip-config) SKIP_CONFIG=true ;;
        --skip-systemd) SKIP_SYSTEMD=true ;;
        -h | --help) usage; exit 0 ;;
        *)
            echo "Unknown option: $1" >&2
            usage >&2
            exit 1
            ;;
    esac
    shift
done

if [[ "${INSTALL_ENGINE}" == false && "${INSTALL_WEB}" == false ]]; then
    echo "Nothing to install." >&2
    exit 1
fi

main() {
    local -a packages=()
    local -a units=()

    if [[ "${INSTALL_ENGINE}" == true ]]; then
        packages+=(robominer-engine)
        units+=(robominer-engine.service)
    fi
    if [[ "${INSTALL_WEB}" == true ]]; then
        packages+=(robominer-web)
        units+=(robominer-web.service)
    fi

    create_service_user
    build_release_packages "${packages[@]}"

    if [[ "${INSTALL_ENGINE}" == true ]]; then
        install_engine_binary
    fi
    if [[ "${INSTALL_WEB}" == true ]]; then
        install_web_binary
        install_static_assets
    fi

    ensure_shared_config_file
    if [[ "${INSTALL_WEB}" == true ]]; then
        ensure_web_config_keys
    fi

    install_systemd_units "${units[@]}"

    echo "RoboMiner installation finished."
    if [[ "${INSTALL_ENGINE}" == true ]]; then
        echo "Engine preflight dry run:"
        echo "  ${INSTALL_PREFIX}/bin/robominer-engine --config ${SHARED_CONFIG_FILE} run-rallies --once"
    fi
    if [[ "${INSTALL_WEB}" == true ]]; then
        echo "Web help page:"
        echo "  http://127.0.0.1:8080/help"
    fi
}

main "$@"
