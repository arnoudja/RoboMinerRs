#!/usr/bin/env bash

# Workspace LLVM coverage against MySQL. Set ROBOMINER_COVERAGE_FAIL_UNDER_LINES to enforce a
# minimum line-coverage percentage (CI uses 87).

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

export ROBOMINER_DATABASE_URL="$("${ROOT}/resources/scripts/ensure-test-mysql.sh")"

if ! cargo llvm-cov --version >/dev/null 2>&1; then
    cat >&2 <<'EOF'
cargo-llvm-cov is not installed.

Install it with:
  cargo install cargo-llvm-cov --locked

Then re-run:
  resources/scripts/run-coverage-with-db.sh
EOF
    exit 1
fi

THRESHOLD_ARGS=()
if [[ -n "${ROBOMINER_COVERAGE_FAIL_UNDER_LINES:-}" ]]; then
    THRESHOLD_ARGS+=(--fail-under-lines "${ROBOMINER_COVERAGE_FAIL_UNDER_LINES}")
fi

cd "${ROOT}"

# llvm-cov uses its own target directory, so integration tests that shell out to
# robominer-engine need a stable binary path on fresh CI runners.
(
    cd "${ROOT}"
    unset CARGO_TARGET_DIR
    cargo build -p robominer-engine --quiet
)
ENGINE_BIN="${ROOT}/target/debug/robominer-engine"

env "CARGO_BIN_EXE_robominer-engine=${ENGINE_BIN}" \
    cargo llvm-cov --workspace "${THRESHOLD_ARGS[@]}" "$@"
