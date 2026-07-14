#!/usr/bin/env bash

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
INSTALL_PREFIX="/opt/robominer"
CONFIG_DIR="/etc/robominer"
SHARED_CONFIG_FILE="${CONFIG_DIR}/robominer.conf"
LEGACY_CONFIG_FILE="${CONFIG_DIR}/robominer-engine.conf"
STATIC_ROOT="${INSTALL_PREFIX}/static"
SYSUSERS_SOURCE="${ROOT}/deploy/systemd/robominer-engine.sysusers"

run_as_root() {
    if [[ "${EUID}" -eq 0 ]]; then
        "$@"
    else
        sudo "$@"
    fi
}

require_command() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "Required command not found: $1" >&2
        exit 1
    fi
}

create_service_user() {
    if [[ "${SKIP_USER:-false}" == true ]]; then
        echo "Skipping service account creation."
        return
    fi

    if getent passwd robominer >/dev/null 2>&1; then
        echo "Service account robominer already exists."
    elif command -v systemd-sysusers >/dev/null 2>&1; then
        echo "Creating service account with systemd-sysusers..."
        run_as_root install -D -m 0644 "${SYSUSERS_SOURCE}" /etc/sysusers.d/robominer-engine.conf
        run_as_root systemd-sysusers /etc/sysusers.d/robominer-engine.conf
    else
        echo "Creating service account with useradd..."
        run_as_root useradd --system --home-dir /var/lib/robominer --shell /usr/sbin/nologin robominer
    fi

    run_as_root install -d -o robominer -g robominer -m 0750 "${INSTALL_PREFIX}"
}

ensure_shared_config_file() {
    if [[ "${SKIP_CONFIG:-false}" == true ]]; then
        echo "Skipping config file creation."
        return
    fi

    run_as_root install -d -o root -g robominer -m 0750 "${CONFIG_DIR}"

    if [[ -f "${SHARED_CONFIG_FILE}" ]]; then
        echo "Shared config already exists: ${SHARED_CONFIG_FILE}"
        return
    fi

    if [[ -f "${LEGACY_CONFIG_FILE}" ]]; then
        echo "Migrating ${LEGACY_CONFIG_FILE} to ${SHARED_CONFIG_FILE}..."
        run_as_root cp "${LEGACY_CONFIG_FILE}" "${SHARED_CONFIG_FILE}"
        run_as_root chown root:robominer "${SHARED_CONFIG_FILE}"
        run_as_root chmod 0640 "${SHARED_CONFIG_FILE}"
        return
    fi

    if [[ -n "${ROBOMINER_DB_SERVER:-}" && -n "${ROBOMINER_DB_USER:-}" && -n "${ROBOMINER_DB_PASSWORD:-}" && -n "${ROBOMINER_DB_DATABASE:-}" ]]; then
        echo "Creating shared config from environment variables..."
        run_as_root tee "${SHARED_CONFIG_FILE}" >/dev/null <<EOF
dbserver ${ROBOMINER_DB_SERVER}
dbuser ${ROBOMINER_DB_USER}
dbpassword ${ROBOMINER_DB_PASSWORD}
dbdatabase ${ROBOMINER_DB_DATABASE}
EOF
        run_as_root chown root:robominer "${SHARED_CONFIG_FILE}"
        run_as_root chmod 0640 "${SHARED_CONFIG_FILE}"
        return
    fi

    cat <<EOF
Shared config file not found: ${SHARED_CONFIG_FILE}

Create it manually, for example:
  sudoedit ${SHARED_CONFIG_FILE}

Required database keys:
  dbserver <host>
  dbuser <user>
  dbpassword <password>
  dbdatabase <database>

Optional web keys:
  host <bind address>
  port <port>
  webroot <static asset directory>
  sessionsecret <session signing secret>
  sessionttlhours <login cookie lifetime in hours; default 24>
  sessionttlsecs <login cookie lifetime in seconds>
  securecookies <1 when served over HTTPS behind a reverse proxy>

Or rerun an install script with ROBOMINER_DB_* variables set.
EOF
}

ensure_web_config_keys() {
    if [[ "${SKIP_CONFIG:-false}" == true || ! -f "${SHARED_CONFIG_FILE}" ]]; then
        return
    fi

    local append=""
    if ! grep -Eq '^[Hh][Oo][Ss][Tt][[:space:]]' "${SHARED_CONFIG_FILE}"; then
        append+="host ${ROBOMINER_WEB_HOST:-127.0.0.1}"$'\n'
    fi
    if ! grep -Eq '^[Pp][Oo][Rr][Tt][[:space:]]' "${SHARED_CONFIG_FILE}"; then
        append+="port ${ROBOMINER_WEB_PORT:-8080}"$'\n'
    fi
    if ! grep -Eq '^[Ww][Ee][Bb][Rr][Oo][Oo][Tt][[:space:]]' "${SHARED_CONFIG_FILE}"; then
        append+="webroot ${STATIC_ROOT}"$'\n'
    fi
    if [[ -n "${ROBOMINER_SESSION_SECRET:-}" ]] && ! grep -Eq '^[Ss][Ee][Ss][Ss][Ii][Oo][Nn][Ss][Ee][Cc][Rr][Ee][Tt][[:space:]]' "${SHARED_CONFIG_FILE}"; then
        append+="sessionsecret ${ROBOMINER_SESSION_SECRET}"$'\n'
    fi
    if [[ "${ROBOMINER_SECURE_COOKIES:-}" == "1" ]] && ! grep -Eq '^[Ss][Ee][Cc][Uu][Rr][Ee][Cc][Oo][Oo][Kk][Ii][Ee][Ss][[:space:]]' "${SHARED_CONFIG_FILE}"; then
        append+="securecookies 1"$'\n'
    fi
    if [[ -n "${ROBOMINER_SESSION_TTL_SECS:-}" ]] && ! grep -Eq '^[Ss][Ee][Ss][Ss][Ii][Oo][Nn][Tt][Tt][Ll][Ss][Ee][Cc][Ss][[:space:]]' "${SHARED_CONFIG_FILE}"; then
        append+="sessionttlsecs ${ROBOMINER_SESSION_TTL_SECS}"$'\n'
    elif [[ -n "${ROBOMINER_SESSION_TTL_HOURS:-}" ]] && ! grep -Eq '^[Ss][Ee][Ss][Ss][Ii][Oo][Nn][Tt][Tt][Ll][Hh][Oo][Uu][Rr][Ss][[:space:]]' "${SHARED_CONFIG_FILE}"; then
        append+="sessionttlhours ${ROBOMINER_SESSION_TTL_HOURS}"$'\n'
    fi

    if [[ -n "${append}" ]]; then
        echo "Adding web settings to ${SHARED_CONFIG_FILE}..."
        printf '%s' "${append}" | run_as_root tee -a "${SHARED_CONFIG_FILE}" >/dev/null
        run_as_root chown root:robominer "${SHARED_CONFIG_FILE}"
        run_as_root chmod 0640 "${SHARED_CONFIG_FILE}"
    fi
}

install_static_assets() {
    echo "Installing static assets to ${STATIC_ROOT}..."
    run_as_root install -d -o robominer -g robominer -m 0755 "${STATIC_ROOT}/css"
    run_as_root install -m 0644 "${ROOT}/robominer-web/static/css/robominer.css" \
        "${STATIC_ROOT}/css/robominer.css"
}

build_release_packages() {
    require_command cargo

    local -a cargo_args=()
    for package in "$@"; do
        cargo_args+=(-p "${package}")
    done

    echo "Building release packages: $*"
    (
        cd "${ROOT}"
        cargo build --release "${cargo_args[@]}"
    )
}

install_engine_binary() {
    local binary_dest="${INSTALL_PREFIX}/bin/robominer-engine"
    echo "Installing binary to ${binary_dest}..."
    run_as_root install -D -m 0755 "${ROOT}/target/release/robominer-engine" "${binary_dest}"
}

install_web_binary() {
    local binary_dest="${INSTALL_PREFIX}/bin/robominer-web"
    echo "Installing binary to ${binary_dest}..."
    run_as_root install -D -m 0755 "${ROOT}/target/release/robominer-web" "${binary_dest}"
}

install_systemd_units() {
    if [[ "${SKIP_SYSTEMD:-false}" == true ]]; then
        echo "Skipping systemd unit installation."
        return
    fi

    if [[ "$#" -eq 0 ]]; then
        return
    fi

    require_command systemctl

    local unit
    for unit in "$@"; do
        echo "Installing systemd unit ${unit}..."
        run_as_root install -D -m 0644 "${ROOT}/deploy/systemd/${unit}" \
            "/etc/systemd/system/${unit}"
    done

    run_as_root systemctl daemon-reload

    if [[ "${ENABLE_SERVICE:-false}" == true ]]; then
        if [[ ! -f "${SHARED_CONFIG_FILE}" ]]; then
            echo "Cannot enable services without ${SHARED_CONFIG_FILE}." >&2
            exit 1
        fi

        for unit in "$@"; do
            echo "Enabling and starting ${unit}..."
            run_as_root systemctl enable --now "${unit}"
            echo "Follow logs with: journalctl -u ${unit} -f"
        done
    else
        echo "Systemd units installed. Enable them with:"
        for unit in "$@"; do
            echo "  sudo systemctl enable --now ${unit}"
        done
    fi
}
