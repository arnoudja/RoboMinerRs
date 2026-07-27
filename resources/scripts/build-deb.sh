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
        if [[ "$1" == "cargo-deb" ]]; then
            echo "Install with: cargo install cargo-deb --locked" >&2
        fi
        exit 1
    fi
}

ensure_rust_target() {
    if ! rustup target list --installed | grep -qx "$1"; then
        log "Installing Rust target $1..."
        rustup target add "$1"
    fi
}

find_deb_for_arch() {
    local arch_label="$1"
    local search_dirs=()
    local deb

    case "$arch_label" in
        native)
            search_dirs+=("${TARGET_DIR}/debian")
            ;;
        aarch64)
            search_dirs+=(
                "${TARGET_DIR}/${AARCH64_TARGET}/debian"
                "${TARGET_DIR}/debian"
            )
            ;;
    esac

    for dir in "${search_dirs[@]}"; do
        [[ -d "$dir" ]] || continue
        deb="$(find "$dir" -maxdepth 1 -type f -name 'robominer_*.deb' -printf '%T@ %p\n' 2>/dev/null \
            | sort -nr \
            | head -n 1 \
            | cut -d' ' -f2- || true)"
        if [[ -n "${deb}" ]]; then
            printf '%s\n' "$deb"
            return 0
        fi
    done
    return 1
}

build_deb() {
    local target_args=()
    local label="$1"
    shift

    if [[ "$#" -gt 0 ]]; then
        target_args=(--target "$1")
    fi

    log ""
    log "Building release binaries (${label})..."
    cargo build --release "${PACKAGES[@]}" "${target_args[@]}"

    log "Packaging robominer .deb (${label})..."
    # --no-build: include the prebuilt robominer-web binary from the other crate.
    cargo deb -p robominer-engine --no-build "${target_args[@]}"
}

require_command cargo
require_command rustc
require_command rustup
require_command cargo-deb
require_command dpkg-deb

cd "${ROOT}"

TARGET_DIR="${CARGO_TARGET_DIR:-${ROOT}/target}"
HOST_TARGET="$(rustc -vV | awk '/^host: / { print $2 }')"

build_deb "native ${HOST_TARGET}"
NATIVE_DEB="$(find_deb_for_arch native || true)"

if [[ "${HOST_TARGET}" == "${AARCH64_TARGET}" ]]; then
    log ""
    log "Host is already ${AARCH64_TARGET}; skipping separate Pi cross package."
    log ""
    log "Built package:"
    log "  ${NATIVE_DEB:-<not found>}"
    exit 0
fi

ensure_rust_target "${AARCH64_TARGET}"
require_command aarch64-linux-gnu-gcc

# Ensure the linker is discoverable for cross builds.
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="${CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER:-aarch64-linux-gnu-gcc}"

build_deb "Raspberry Pi ${AARCH64_TARGET}" "${AARCH64_TARGET}"
AARCH64_DEB="$(find_deb_for_arch aarch64 || true)"

log ""
log "Built packages:"
log "  native:  ${NATIVE_DEB:-<not found>}"
log "  aarch64: ${AARCH64_DEB:-<not found>}"

if [[ -z "${NATIVE_DEB}" || -z "${AARCH64_DEB}" ]]; then
    log "Warning: could not locate one or more .deb artifacts under ${TARGET_DIR}"
    exit 1
fi
