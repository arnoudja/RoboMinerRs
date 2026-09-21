#!/usr/bin/env bash
# Build (and optionally install) the Arch package from the current working tree
# (including uncommitted changes). Requires Arch makepkg (Omarchy/Arch).
#
# Package version is always taken from the workspace Cargo.toml.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
cd "${SCRIPT_DIR}"

# Omarchy /tmp is a tmpfs with a small per-user quota. Keep file lists,
# rustc scratch, and any inherited Cargo target off that filesystem.
export TMPDIR="${ROOT}/target/makepkg-local"
mkdir -p "${TMPDIR}"
case "${CARGO_TARGET_DIR:-}" in
    /tmp | /tmp/*) unset CARGO_TARGET_DIR ;;
esac

pkgver="$(sed -n 's/^version = "\(.*\)"/\1/p' "${ROOT}/Cargo.toml" | head -n1)"
if [[ -z "${pkgver}" ]]; then
    echo "Could not read version from ${ROOT}/Cargo.toml" >&2
    exit 1
fi

"${SCRIPT_DIR}/sync-pkgver-from-cargo.sh"

# Workspace version bumps leave Cargo.lock stale; --locked prepare/build need it current.
if ! (cd "${ROOT}" && cargo metadata --locked --format-version 1 >/dev/null 2>&1); then
    echo "Updating Cargo.lock to match Cargo.toml..."
    (cd "${ROOT}" && cargo metadata --format-version 1 >/dev/null)
fi

commit="$(git -C "${ROOT}" rev-parse HEAD)"
tarball="robominer-${pkgver}.tar.gz"
prefix="RoboMinerRs-${commit}"
file_list="$(mktemp)"
existing_list="$(mktemp)"
cleanup() {
    rm -f "${file_list}" "${existing_list}"
}
trap cleanup EXIT

# Tracked + untracked non-ignored files, as they exist on disk (dirty tree OK).
(
    cd "${ROOT}"
    git ls-files -z --cached --others --exclude-standard
) > "${file_list}"

: > "${existing_list}"
while IFS= read -r -d '' path; do
    if [[ -e "${ROOT}/${path}" || -L "${ROOT}/${path}" ]]; then
        printf '%s\0' "${path}" >> "${existing_list}"
    fi
done < "${file_list}"

echo "Archiving working tree (pkgver=${pkgver}, commit ${commit}) -> ${tarball}"
tar -C "${ROOT}" --null -T "${existing_list}" \
    --transform "s,^,${prefix}/," \
    -czf "${SCRIPT_DIR}/${tarball}"

sed \
  -e "s/^pkgver=.*/pkgver=${pkgver}/" \
  -e "s/^_commit=.*/_commit=${commit}/" \
  -e "s/^_srcdir=.*/_srcdir=\"RoboMinerRs-\${_commit}\"/" \
  -e "s|^source=.*|source=(\"${tarball}\")|" \
  -e "s/^sha256sums=.*/sha256sums=('SKIP')/" \
  PKGBUILD > PKGBUILD.local

echo "Wrote PKGBUILD.local (pkgver ${pkgver} from Cargo.toml)"
makepkg -p PKGBUILD.local "$@"
