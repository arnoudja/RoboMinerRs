#!/usr/bin/env bash

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

export ROBOMINER_DATABASE_URL="$("${ROOT}/resources/scripts/ensure-test-mysql.sh")"

cd "${ROOT}"

# Page JS tests (Node `node:test`; no MySQL). Skip when CI already ran them
# in the fast-tests job (ROBOMINER_SKIP_PAGE_JS_TESTS=1).
if [[ "${ROBOMINER_SKIP_PAGE_JS_TESTS:-}" != "1" ]]; then
    "${ROOT}/resources/scripts/run-page-js-tests.sh"
fi

if cargo nextest --version >/dev/null 2>&1; then
    cargo nextest run --workspace --profile ci "$@"
else
    cargo test --workspace "$@" -- --test-threads 1
fi
