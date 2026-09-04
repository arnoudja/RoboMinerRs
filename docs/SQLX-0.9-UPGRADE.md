# sqlx 0.9 upgrade

RoboMiner pins **sqlx 0.9.0** in the workspace [`Cargo.toml`](../Cargo.toml) with
features `runtime-tokio`, `tls-rustls-ring`, `mysql`, and `macros` (the old
combined `runtime-tokio-rustls` feature was removed in 0.9).

## Compile-checked queries

Hot paths in claim, mining-queue enqueue/cancel, and user session bump /
last-login use `sqlx::query!` / `query_scalar!` with offline metadata under
[`.sqlx/`](../.sqlx/). After changing those SQL strings (or the schema they
touch), refresh metadata:

```sh
export DATABASE_URL=mysql://robominer:password@127.0.0.1:3306/RoboMiner
cargo sqlx prepare --workspace -- --package robominer-db --lib
```

Commit the updated `.sqlx/` JSON. CI sets `SQLX_OFFLINE=true` so builds do not
need a live MySQL connection just to compile macros.

## Dynamic SQL (`SqlSafeStr`)

sqlx 0.9 requires `query*` SQL to be `&'static str` or wrapped in
`AssertSqlSafe`. Prefer literals + binds. When the SQL string must be built
(placeholder counts, fixed column fragments), use
[`robominer_db::assert_sql_safe`](../robominer-db/src/query_util.rs) after
auditing that all user values are bound parameters. Dynamic `IN (...)` batches
and `FOR UPDATE` locks still use runtime `sqlx::query` this way.

## Validation checklist (after future sqlx bumps)

1. Read the [sqlx changelog](https://github.com/launchbadge/sqlx/blob/main/CHANGELOG.md).
2. Update the workspace dependency pin and matching `sqlx-cli`.
3. `cargo update -p sqlx` and fix compile errors crate-by-crate.
4. Re-run `cargo sqlx prepare --workspace -- --package robominer-db --lib` against MySQL 8.4.
5. `cargo fmt --all` and `cargo clippy --workspace -- -D warnings`.
6. `resources/scripts/run-tests-with-db.sh`.

## High-risk areas to re-test

- Rally claim batch upserts (`robominer-db/src/rally/claim/`)
- Mining queue enqueue/cancel concurrency (`robominer-db/tests/db_mining_queue.rs`)
- Shop sell-all batch paths (`robominer-db/src/shop/`)
- Migration runner (`robominer-db/src/migrate/`) against MySQL 8.4 (CI target)

## CI note

CI targets **MySQL 8.4** only. MariaDB compatibility is not exercised in CI.
