#!/usr/bin/env bash

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

cd "${ROOT}"

# Headless rally replay viewer logic (no MySQL).
"${ROOT}/resources/scripts/run-rally-animation-js-tests.sh"

if cargo nextest --version >/dev/null 2>&1; then
    cargo nextest run --workspace --profile fast "$@"
else
    cat >&2 <<'EOF'
cargo-nextest is not installed; falling back to cargo test --lib.

Install nextest for faster parallel runs:
  cargo install cargo-nextest --locked
EOF
    cargo test --workspace --lib "$@"
    cargo test -p robominer-domain --test rally_golden --test pool_golden "$@"
    cargo test -p robominer-engine --test verify_source_cli "$@"
fi
