#!/usr/bin/env bash

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

if ! command -v node >/dev/null 2>&1; then
    echo "node is required to run RoboMiner web JS tests" >&2
    exit 1
fi

cd "${ROOT}/robominer-web/static/js/rally_animation/tests"
node --test viewer.test.js

cd "${ROOT}/robominer-web/static/js/mining_queue/tests"
node --test clear_wallet.test.js page.test.js

cd "${ROOT}/robominer-web/static/js/mining_results/tests"
node --test page.test.js

cd "${ROOT}/robominer-web/static/js/common/tests"
node --test filter_restore.test.js

cd "${ROOT}/robominer-web/static/js/shop/tests"
node --test page.test.js

cd "${ROOT}/robominer-web/static/js/robot/tests"
node --test page.test.js

cd "${ROOT}/robominer-web/static/js/edit_code/tests"
node --test page.test.js

cd "${ROOT}/robominer-web/static/js/tests"
node --test page_scripts.test.js
