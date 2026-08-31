# sqlx 0.9 upgrade plan

RoboMiner currently pins **sqlx 0.8.6** in the workspace [`Cargo.toml`](../Cargo.toml). A future dedicated PR should upgrade to **sqlx 0.9** after a full validation pass.

## Compile-checked queries (current)

Hot paths in claim and mining-queue enqueue/cancel use `sqlx::query!` / `query_scalar!` with offline metadata under [`.sqlx/`](../.sqlx/). After changing those SQL strings (or the schema they touch), refresh metadata:

```sh
export DATABASE_URL=mysql://robominer:password@127.0.0.1:3306/RoboMiner
cargo sqlx prepare --workspace -- --package robominer-db --lib
```

Commit the updated `.sqlx/` JSON. CI sets `SQLX_OFFLINE=true` so builds do not need a live MySQL connection just to compile macros. Dynamic `IN (...)` batches and `FOR UPDATE` locks still use runtime `sqlx::query`.

## Scope

Crates using sqlx:

- `robominer-db` (primary — queries, migrations, `FromRow` derives)
- `robominer-web`, `robominer-engine`, `robominer-domain`, `robominer-test-support` (pool consumers / test helpers)

## Pre-upgrade checklist

1. Read the [sqlx 0.9 changelog](https://github.com/launchbadge/sqlx/blob/main/CHANGELOG.md) for breaking API and MySQL driver changes.
2. Update the workspace dependency in `Cargo.toml` (single version pin) and reinstall matching `sqlx-cli`.
3. Run `cargo update -p sqlx` and fix compile errors crate-by-crate.
4. Re-run `cargo sqlx prepare --workspace -- --package robominer-db --lib` against MySQL 8.4 and commit `.sqlx/`.
5. Run formatting and Clippy:
   ```sh
   cargo fmt --all
   cargo clippy --workspace -- -D warnings
   ```
6. Run the full DB-backed suite:
   ```sh
   resources/scripts/run-tests-with-db.sh
   ```
7. Run coverage locally if the PR touches query-heavy paths:
   ```sh
   resources/scripts/run-coverage-with-db.sh --lcov --output-path lcov.info
   ```

## High-risk areas to re-test manually

- Rally claim batch upserts (`robominer-db/src/rally/claim/`)
- Mining queue enqueue/cancel concurrency (`robominer-db/tests/db_mining_queue.rs`)
- Shop sell-all batch paths (`robominer-db/src/shop/`)
- Migration runner (`robominer-db/src/migrate/`) against MySQL 8.4 (CI target)

## Rollback

Keep the upgrade isolated in one PR. If CI fails on subtle query/runtime regressions, revert the dependency bump rather than patching forward without tests.

## CI note

CI targets **MySQL 8.4** only (see [`.github/workflows/ci.yml`](../.github/workflows/ci.yml)). MariaDB compatibility is not exercised in CI; document any driver assumptions in the PR description if they change.
