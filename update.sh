#!/usr/bin/env bash
# Rebuild and reinstall RoboMiner for the current OS from the working tree
# (uncommitted changes included). Version comes from workspace Cargo.toml.
# Ubuntu/Debian: cargo-deb + apt. Omarchy/Arch: makepkg + pacman.

set -euo pipefail

cd "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "### Syncing package version from Cargo.toml ###"
deploy/arch/sync-pkgver-from-cargo.sh

# shellcheck disable=SC1091
. /etc/os-release

if [[ "${ID}" == omarchy || "${ID}" == arch || " ${ID_LIKE:-} " == *" arch "* ]]; then
    echo "### Building and installing the Arch package ###"
    rm -f deploy/arch/robominer-*.pkg.tar.zst
    (cd deploy/arch && ./makepkg-local.sh -sfi --noconfirm)
elif [[ "${ID}" == ubuntu || "${ID}" == debian || "${ID}" == raspbian \
    || " ${ID_LIKE:-} " == *" debian "* || " ${ID_LIKE:-} " == *" ubuntu "* ]]; then
    echo "### Building the Debian package ###"
    rm -f ./target/debian/robominer_*_amd64.deb
    resources/scripts/build-deb.sh
    echo "### Installing the package ###"
    sudo apt-get install --reinstall ./target/debian/robominer_*_amd64.deb
else
    echo "Unsupported OS: ID=${ID:-unknown} ID_LIKE=${ID_LIKE:-unknown}" >&2
    exit 1
fi

echo "### Done ###"
