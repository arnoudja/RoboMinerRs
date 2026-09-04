#!/usr/bin/env bash
# Compatibility shim: JS page tests live in run-page-js-tests.sh.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
exec "${ROOT}/resources/scripts/run-page-js-tests.sh" "$@"
