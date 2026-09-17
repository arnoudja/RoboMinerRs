#!/usr/bin/env bash
# Set deploy/arch PKGBUILD + .SRCINFO pkgver from the workspace Cargo.toml.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
PKGBUILD="${SCRIPT_DIR}/PKGBUILD"
SRCINFO="${SCRIPT_DIR}/.SRCINFO"

pkgver="$(sed -n 's/^version = "\(.*\)"/\1/p' "${ROOT}/Cargo.toml" | head -n1)"
if [[ -z "${pkgver}" ]]; then
    echo "Could not read version from ${ROOT}/Cargo.toml" >&2
    exit 1
fi

if [[ ! -f "${PKGBUILD}" ]]; then
    echo "Missing ${PKGBUILD}" >&2
    exit 1
fi

# GNU sed (Arch/Ubuntu).
sed -i "s/^pkgver=.*/pkgver=${pkgver}/" "${PKGBUILD}"

if command -v makepkg >/dev/null 2>&1; then
    (cd "${SCRIPT_DIR}" && makepkg --printsrcinfo > .SRCINFO)
else
    # Debian/Ubuntu hosts may not have makepkg; keep .SRCINFO in sync by hand.
    if [[ ! -f "${SRCINFO}" ]]; then
        echo "Missing ${SRCINFO} and makepkg is unavailable to regenerate it." >&2
        exit 1
    fi
    sed -i "s/^[[:space:]]*pkgver = .*/\tpkgver = ${pkgver}/" "${SRCINFO}"
    sed -i "s|^[[:space:]]*source = robominer-[^:]*::|	source = robominer-${pkgver}.tar.gz::|" "${SRCINFO}"
fi

echo "Synced Arch pkgver=${pkgver} from Cargo.toml"
