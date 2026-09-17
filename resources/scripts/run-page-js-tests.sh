#!/usr/bin/env bash

# Run Node-based page JS tests (rally animation, shop, mining, atlas, …).
# Each listed tests/ directory is discovered via Node's test-file patterns
# (`*.test.js`, …) so new files are not omitted from CI.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

if ! command -v node >/dev/null 2>&1; then
    echo "node is required to run RoboMiner web JS tests" >&2
    exit 1
fi

run_dir_tests() {
    local dir="$1"
    (cd "${dir}" && node --test .)
}

run_dir_tests "${ROOT}/robominer-web/static/js/rally_animation/tests"
run_dir_tests "${ROOT}/robominer-web/static/js/mining_queue/tests"
run_dir_tests "${ROOT}/robominer-web/static/js/mining_results/tests"
run_dir_tests "${ROOT}/robominer-web/static/js/common/tests"
run_dir_tests "${ROOT}/robominer-web/static/js/shop/tests"
run_dir_tests "${ROOT}/robominer-web/static/js/robot/tests"
run_dir_tests "${ROOT}/robominer-web/static/js/edit_code/tests"
run_dir_tests "${ROOT}/robominer-web/static/js/mining_area_atlas/tests"
run_dir_tests "${ROOT}/robominer-web/static/js/tests"
