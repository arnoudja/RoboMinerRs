#!/usr/bin/env bash
# Wait until robominer-web answers 200 on GET /health (loopback readiness).
#
# Usage:
#   robominer-wait-web-health [/etc/robominer/robominer.conf]
#
# Reads host/port from the shared config when present (defaults: 127.0.0.1:8080).
# Intended for systemd ExecStartPost.

set -euo pipefail

CONFIG_FILE="${1:-/etc/robominer/robominer.conf}"
HOST="${ROBOMINER_WEB_HOST:-127.0.0.1}"
PORT="${ROBOMINER_WEB_PORT:-8080}"
ATTEMPTS="${ROBOMINER_HEALTH_ATTEMPTS:-30}"
SLEEP_SECS="${ROBOMINER_HEALTH_SLEEP_SECS:-1}"

if [[ -f "${CONFIG_FILE}" ]]; then
    while read -r key value; do
        case "${key,,}" in
            host) HOST="${value}" ;;
            port) PORT="${value}" ;;
        esac
    done < <(awk 'NF && $1 !~ /^#/ { print tolower($1), $2 }' "${CONFIG_FILE}")
fi

probe_health() {
    local response=""
    if command -v curl >/dev/null 2>&1; then
        response="$(
            curl -fsS --max-time 2 "http://${HOST}:${PORT}/health" 2>/dev/null || true
        )"
    else
        # Fallback without curl: raw HTTP over bash /dev/tcp.
        response="$(
            {
                exec 3<>"/dev/tcp/${HOST}/${PORT}" || exit 1
                printf 'GET /health HTTP/1.1\r\nHost: %s\r\nConnection: close\r\n\r\n' "${HOST}" >&3
                timeout 2 cat <&3 || true
                exec 3<&- 3>&- || true
            } 2>/dev/null || true
        )"
        if ! printf '%s' "${response}" | head -n 1 | grep -Eq 'HTTP/[0-9.]+ 200'; then
            return 1
        fi
        response="$(printf '%s' "${response}" | awk 'BEGIN{blank=0} blank{print} /^$/{blank=1}')"
    fi

    printf '%s' "${response}" | head -n 1 | grep -qx 'ok'
}

for _ in $(seq 1 "${ATTEMPTS}"); do
    if probe_health; then
        echo "robominer-web health ok at http://${HOST}:${PORT}/health"
        exit 0
    fi
    sleep "${SLEEP_SECS}"
done

echo "robominer-web health check failed at http://${HOST}:${PORT}/health after ${ATTEMPTS} attempts" >&2
exit 1
