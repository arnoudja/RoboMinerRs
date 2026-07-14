# Contributing

## Running tests

Use the same entry point locally and in CI:

```sh
resources/scripts/run-tests-with-db.sh
```

That script:

1. Resolves `ROBOMINER_DATABASE_URL` via `ensure-test-mysql.sh` (existing URL, local MySQL, or persistent Docker).
2. Runs `cargo nextest run --workspace --profile ci` when nextest is installed, otherwise `cargo test --workspace`.

The `ci` profile uses a single test thread so DB integration binaries that share MySQL stay serialized via `#[serial]`.

CI initializes MySQL with `init-ci-database.sh`, sets `ROBOMINER_DATABASE_URL`, then calls
`run-tests-with-db.sh` so local and CI runs execute the same test command.

Pass extra arguments through to Cargo:

```sh
resources/scripts/run-tests-with-db.sh --lib -p robominer-domain
resources/scripts/run-tests-with-db.sh -p robominer-web -- login
```

Without a database URL, DB-backed integration tests skip themselves (they print a message and
return). Golden and unit tests still run.

### Fast tests (no database)

For library unit tests and simulation goldens that do not need MySQL:

```sh
resources/scripts/run-fast-tests.sh
```

Install [`cargo-nextest`](https://nexte.st/) for faster runs. `run-fast-tests.sh` uses the `fast` profile; `run-tests-with-db.sh` uses the `ci` profile when nextest is present. Both scripts fall back to `cargo test` when nextest is absent.

## Coverage

Install [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov) once:

```sh
cargo install cargo-llvm-cov --locked
```

Generate a workspace report against MySQL:

```sh
resources/scripts/run-coverage-with-db.sh
```

Write LCOV for upload or local inspection:

```sh
resources/scripts/run-coverage-with-db.sh --lcov --output-path lcov.info
```

HTML summary:

```sh
resources/scripts/run-coverage-with-db.sh --html --output-dir target/coverage-html
```

CI uploads `lcov.info` as a workflow artifact on every push and pull request. The coverage job
also uploads to Codecov when configured and fails when line coverage drops below
`ROBOMINER_COVERAGE_FAIL_UNDER_LINES` (currently 52 in CI).

Set the threshold locally:

```sh
ROBOMINER_COVERAGE_FAIL_UNDER_LINES=52 resources/scripts/run-coverage-with-db.sh
```

## Splitting a web page module

Use `resources/scripts/split-web-page.py` when a `robominer-web/src/<page>.rs` file grows past
handler + render + inline tests. The script moves code into `<page>/mod.rs`, `render.rs`, and
`tests.rs` using line-number boundaries you pass for:

- `render_start` — first line of the render function
- `helper_start` — first line after render (handler helpers)
- `tests_start` — first `#[cfg(test)]` module

Edit the script's `if __name__ == "__main__"` block: uncomment and fill in the
`split_page(...)` example with your page path, line boundaries, and imports (place it
above `sys.exit(1)`), then run:

```sh
python3 resources/scripts/split-web-page.py
```

Existing splits (`shop_page/`, `robot_page/`, `edit_code_page/`, `auth_pages/`, `rally_pages/`,
`achievements_page/`, `account_page/`, …)
follow this layout: handlers and state in `mod.rs`, HTML in `render.rs`, pure tests in `tests.rs`.

## Test layout conventions

| Layer | Location | When to use |
|-------|----------|-------------|
| Page render/helpers | `robominer-web/src/<page>/tests.rs` | Pure HTML and helper logic; no live HTTP or DB |
| Help content | `robominer-web/static/help/*.html` | Guide bodies loaded with `include_str!`; rendering in `help_pages/render.rs` |
| HTTP + DB integration | `robominer-web/tests/*.rs` | POST/GET through `route()` with real MySQL |
| Engine CLI integration | `robominer-engine/tests/*_db_cli.rs` | Subprocess `robominer-engine` against MySQL |
| DB mutations | `robominer-db/tests/` | Direct SQL helpers without CLI or HTTP (`db_mutations.rs`, `db_users.rs`, `db_rally.rs`, `db_activity.rs`, `db_pool.rs`, `db_program_sources.rs`, `db_mining_areas.rs`, `db_robots.rs`) |
| Domain goldens | `robominer-domain/tests/*_golden.rs` | Deterministic simulation fixtures |
| Shared fixtures | `robominer-test-support/` | SQL setup reused by web and engine tests |

New web pages should follow the `mod.rs` + `render.rs` + `tests.rs` split used by `shop_page/`,
`mining_queue_page/`, `robot_page/`, `leaderboard_page/`, `mining_results_page/`,
`achievements_page/`, `account_page/`, and `mining_area_overview_page/`.

Engine integration tests use `mod support; use support::*;` and `#[serial]` because they share
one MySQL instance.

## Route-to-test matrix

“Page unit” = tests in `robominer-web/src/<page>/tests.rs` or inline `#[cfg(test)]` in the
page module. “Web DB” = `robominer-web/tests/`. “Engine CLI” = matching `*_db_cli.rs` binary.

| Route / feature | Page unit | Web DB | Engine CLI | Notes |
|-----------------|-----------|--------|------------|-------|
| `/` redirect | `router` tests | `web_db_smoke` | — | Logged-in → mining queue |
| `/login`, signup | `auth_pages/tests.rs` | `login.rs` | `user_create_db_cli.rs`, `user_login_db_cli.rs` | Session cookie minted at login; signup POST covered |
| `/logoff` | `auth_pages/tests.rs`, `router` | — | — | Router test clears session cookie |
| `/account` | `account_page` | `account_actions.rs` | `user_account_update_db_cli.rs` | Profile/password updates |
| `/achievements` | `achievements_page` | `achievement_claim.rs` | `achievement_db_cli.rs` | Claim rewards |
| `/editCode` | `edit_code_page/tests.rs` | `edit_code_actions.rs` | `program_source_db_cli.rs` | Create, apply, and delete sources |
| `/robot` | `robot_page/tests.rs` | `robot_apply.rs` | `robot_config_db_cli.rs`, `claim_robot_config_db_cli.rs` | Apply config + claim pending |
| `/shop` | `shop_page/tests.rs` | `shop_actions.rs` | `shop_db_cli.rs` | Buy/sell parts |
| `/miningQueue` | `mining_queue_page/tests.rs` | `mining_queue_actions.rs` | `mining_queue_db_cli.rs` | Enqueue, fill, cancel |
| `/miningResults` | `mining_results_page/tests.rs` | `read_model_pages.rs` | `mining_area_read_model_db_cli.rs` | |
| `/leaderboard` | `leaderboard_page/tests.rs` | `read_model_pages.rs` | `leaderboard_read_model_db_cli.rs` | |
| `/miningAreaOverview` | `mining_area_overview_page/tests.rs` | `read_model_pages.rs` | `mining_area_overview_read_model_db_cli.rs` | |
| `/activity` | `rally_pages/tests.rs` | `read_model_pages.rs` | `activity_read_model_db_cli.rs`, `rally_read_model_db_cli.rs` | Activity feed + rally replay UI |
| `/help*` | `help_page/tests.rs` | — | — | Static help content in `static/help/` |
| Rally worker / claim | — | `web_db_smoke` (indirect) | `rally_db_cli.rs`, `pool_db_cli.rs` | Background engine, not a page POST |
| Program compile | `robominer-program` unit | — | `verify_source_cli.rs` | No DB |
| Simulation goldens | — | — | — | `robominer-domain/tests/rally_golden.rs`, `pool_golden.rs` |

Gaps worth knowing when adding tests:

- `/logoff` — covered by `auth_pages/tests.rs` and `router` tests; no DB integration needed.
- `/editCode` apply/delete — covered by `edit_code_actions.rs`.

## User-facing rejection messages

Player-visible web copy and engine CLI diagnostics both come from
`robominer_domain::rejection_messages`:

- Web pages call the `*_player_message` helpers (often via thin `pub(super)` wrappers in the page module).
- Engine CLI commands call the matching `*_cli_message` helpers.

When changing copy, update the central module and keep the page-module parity tests and
`*_db_cli.rs` integration tests green.

## Benchmarking robot programs

When comparing programs or validating balance advice:

```sh
cargo test -p robominer-domain benchmark_recommended_programs -- --nocapture
```

Harness: `robominer-domain/tests/program_recommendations.rs`.
