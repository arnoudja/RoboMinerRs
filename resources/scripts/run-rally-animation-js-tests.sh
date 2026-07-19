#!/usr/bin/env bash

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TEST_DIR="${ROOT}/robominer-web/static/js/rally_animation/tests"

if ! command -v node >/dev/null 2>&1; then
    echo "node is required to run rally animation JS tests" >&2
    exit 1
fi

cd "${TEST_DIR}"
exec node --test viewer.test.js
