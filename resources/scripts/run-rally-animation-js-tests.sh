#!/usr/bin/env bash

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

if ! command -v node >/dev/null 2>&1; then
    echo "node is required to run RoboMiner web JS tests" >&2
    exit 1
fi

cd "${ROOT}/robominer-web/static/js/rally_animation/tests"
node --test viewer.test.js

cd "${ROOT}/robominer-web/static/js/tests"
node --test page_scripts.test.js
