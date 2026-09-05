#!/usr/bin/env bash
# Fail CI when migration baseline probes drift across:
#   - resources/database/migrations/*.sql
#   - robominer-db/src/migrate/mod.rs (BASELINE_PROBE_* lists)
#   - robominer-db/src/migrate/schema.rs (schema_already_current)
#   - resources/scripts/migrate-database.sh (schema_already_current)
#   - resources/database/createDatabase.sql (fresh-schema markers)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MIGRATE_MOD="${ROOT}/robominer-db/src/migrate/mod.rs"
SCHEMA_RS="${ROOT}/robominer-db/src/migrate/schema.rs"
MIGRATE_SH="${ROOT}/resources/scripts/migrate-database.sh"
CREATE_SQL="${ROOT}/resources/database/createDatabase.sql"
MIGRATIONS_DIR="${ROOT}/resources/database/migrations"

die() {
  echo "check-migration-baseline-sync: $*" >&2
  exit 1
}

[[ -f "${MIGRATE_MOD}" ]] || die "missing ${MIGRATE_MOD}"
[[ -f "${SCHEMA_RS}" ]] || die "missing ${SCHEMA_RS}"
[[ -f "${MIGRATE_SH}" ]] || die "missing ${MIGRATE_SH}"
[[ -f "${CREATE_SQL}" ]] || die "missing ${CREATE_SQL}"
[[ -d "${MIGRATIONS_DIR}" ]] || die "missing ${MIGRATIONS_DIR}"

mapfile -t MIGRATION_FILES < <(
  find "${MIGRATIONS_DIR}" -maxdepth 1 -type f -name '*.sql' -printf '%f\n' \
    | sed 's/\.sql$//' \
    | sort
)
[[ "${#MIGRATION_FILES[@]}" -gt 0 ]] || die "no migration SQL files found"

# Every on-disk migration must appear in exactly one of the two policy lists.
for version in "${MIGRATION_FILES[@]}"; do
  if ! grep -Fq "\"${version}\"" "${MIGRATE_MOD}"; then
    die "migration ${version} is not listed in BASELINE_PROBE_MIGRATIONS or BASELINE_COVERED_BY_LATER_PROBE (${MIGRATE_MOD})"
  fi
done

# Shared markers that schema_already_current and the shell helper must both check,
# and that createDatabase.sql must contain for a fresh install.
REQUIRED_MARKERS=(
  "scanTime"
  "scanSpeed"
  "sessionVersion"
  "scoreOreTarget"
  "AIRobot"
  "depotTaxRate"
  "MiningOreResult"
  "depotAmount"
  "MiningAreaLifetimeResult"
  "totalRuns"
  "AchievementStepDepotTotalRequirement"
  "processingLeaseUntil"
  "idx_mining_queue_claimable"
  "RobotLifetimeResult"
)

for marker in "${REQUIRED_MARKERS[@]}"; do
  grep -Fq "${marker}" "${SCHEMA_RS}" || die "schema.rs missing probe marker ${marker}"
  grep -Fq "${marker}" "${MIGRATE_SH}" || die "migrate-database.sh missing probe marker ${marker}"
  # scanSpeed is intentionally absent from a current fresh schema (renamed away).
  if [[ "${marker}" == "scanSpeed" ]]; then
    if grep -Eq '[[:space:]]scanSpeed[[:space:]]' "${CREATE_SQL}"; then
      die "createDatabase.sql still defines scanSpeed (expected rename to scanTime only)"
    fi
    continue
  fi
  grep -Fq "${marker}" "${CREATE_SQL}" || die "createDatabase.sql missing fresh-schema marker ${marker}"
done

# Shell helper and Rust function must share the same entry-point name.
grep -Eq 'fn[[:space:]]+schema_already_current' "${SCHEMA_RS}" \
  || die "schema.rs missing schema_already_current"
grep -Eq '^schema_already_current\(\)' "${MIGRATE_SH}" \
  || die "migrate-database.sh missing schema_already_current()"

echo "Migration baseline probes are in sync (${#MIGRATION_FILES[@]} migrations)."
