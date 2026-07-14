#!/usr/bin/env bash

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

export ROBOMINER_DATABASE_URL="$("${ROOT}/resources/scripts/ensure-test-mysql.sh")"

cd "${ROOT}"

if cargo nextest --version >/dev/null 2>&1; then
    cargo nextest run --workspace --profile ci "$@"
else
    cargo test --workspace "$@"
fi
