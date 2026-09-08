#!/usr/bin/env bash
# Build (and optionally install) the Arch package from the current git checkout
# without fetching sources from GitHub. Requires Arch makepkg (Omarchy/Arch).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
cd "${SCRIPT_DIR}"

pkgver="$(sed -n 's/^pkgver=//p' PKGBUILD | head -n1)"
commit="$(git -C "${ROOT}" rev-parse HEAD)"
tarball="robominer-${pkgver}.tar.gz"
prefix="RoboMinerRs-${commit}"

echo "Archiving ${commit} -> ${tarball}"
git -C "${ROOT}" archive --format=tar.gz --prefix="${prefix}/" -o "${SCRIPT_DIR}/${tarball}" HEAD

sed \
  -e "s/^_commit=.*/_commit=${commit}/" \
  -e "s/^_srcdir=.*/_srcdir=\"RoboMinerRs-\${_commit}\"/" \
  -e "s|^source=.*|source=(\"${tarball}\")|" \
  -e "s/^sha256sums=.*/sha256sums=('SKIP')/" \
  PKGBUILD > PKGBUILD.local

echo "Wrote PKGBUILD.local (commit ${commit})"
makepkg -p PKGBUILD.local "$@"
