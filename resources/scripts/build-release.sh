#!/usr/bin/env bash

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PACKAGES=(-p robominer-engine -p robominer-web)
AARCH64_TARGET="aarch64-unknown-linux-gnu"

log() {
    echo "$@" >&2
}

require_command() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "Required command not found: $1" >&2
        exit 1
    fi
}

ensure_rust_target() {
    if ! rustup target list --installed | grep -qx "$1"; then
        log "Installing Rust target $1..."
        rustup target add "$1"
    fi
}

require_command cargo
require_command rustc
require_command rustup

cd "${ROOT}"

TARGET_DIR="${CARGO_TARGET_DIR:-${ROOT}/target}"
HOST_TARGET="$(rustc -vV | awk '/^host: / { print $2 }')"

log "Building release binaries for native target ${HOST_TARGET}..."
cargo build --release "${PACKAGES[@]}"

log ""
log "Built native binaries:"
log "  ${TARGET_DIR}/release/robominer-engine"
log "  ${TARGET_DIR}/release/robominer-web"

if [[ "${HOST_TARGET}" == "${AARCH64_TARGET}" ]]; then
    log ""
    log "Host is already ${AARCH64_TARGET}; no cross build needed."
    exit 0
fi

ensure_rust_target "${AARCH64_TARGET}"
require_command aarch64-linux-gnu-gcc

log ""
log "Building release binaries for ${AARCH64_TARGET}..."
cargo build --release "${PACKAGES[@]}" --target "${AARCH64_TARGET}"

log ""
log "Built Raspberry Pi (64-bit) binaries:"
log "  ${TARGET_DIR}/${AARCH64_TARGET}/release/robominer-engine"
log "  ${TARGET_DIR}/${AARCH64_TARGET}/release/robominer-web"
