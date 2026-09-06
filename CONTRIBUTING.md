# Contributing

Orientation for contributors:

- [README.md](README.md) — build, run web/engine, deploy
- [docs/architecture.md](docs/architecture.md) — crate layers and request flows
- [docs/crate-map.md](docs/crate-map.md) — module ownership in large crates
- [docs/configuration.md](docs/configuration.md) — environment variable reference
- [AGENTS.md](AGENTS.md) — Cursor Cloud / VM environment caveats
- Agent automation also follows [`.cursor/rules/`](.cursor/rules/); keep process docs here rather than forking them into PRs

Supported database target is **MySQL 8.4** (CI and Docker test image). MariaDB may
work but is best-effort. Some Cloud Agent VMs ship a host MySQL 8.0 package; that
is a local convenience, not the supported dialect—see [AGENTS.md](AGENTS.md).

## Pull requests

Before opening or updating a PR:

1. Format and lint: `cargo fmt --all -- --check` and `cargo clippy --workspace -- -D warnings`
2. Run tests: `resources/scripts/run-tests-with-db.sh` (or `run-fast-tests.sh` when the change cannot affect DB paths)
3. Do not commit golden fixture updates unless the behavior change is deliberate
4. If you change SQL dialect assumptions, note them in the PR description
5. CI also runs a fast (no-DB) job, line coverage with floor
   `ROBOMINER_COVERAGE_FAIL_UNDER_LINES=93`, `cargo audit`, and `cargo deny`
   (see `.cargo/audit.toml` and `deny.toml`)

## Git hooks

This repo ships a pre-commit hook that runs the same rustfmt and Clippy checks as
CI when staged files include Rust sources:

```sh
git config core.hooksPath .githooks
```

That sets a repo-local `core.hooksPath` (not global). After that, commits that
stage `.rs` files run:

```sh
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
```

Bypass with `git commit --no-verify` only when you intentionally need to (CI will
still enforce both checks).

## Running tests

Use the same entry point locally and in CI:

```sh
resources/scripts/run-tests-with-db.sh
```

That script:

1. Resolves `ROBOMINER_DATABASE_URL` via `ensure-test-mysql.sh` (existing URL, local MySQL, or persistent Docker).
2. Runs rally animation JS tests (`resources/scripts/run-page-js-tests.sh`; requires Node).
3. Runs `cargo nextest run --workspace --profile ci` when nextest is installed, otherwise `cargo test --workspace` with a single test thread.

The `ci` profile uses a single test thread so DB integration binaries that share MySQL stay serialized via `#[serial]`.

CI initializes MySQL with `init-ci-database.sh`, sets `ROBOMINER_DATABASE_URL`, then calls
`run-tests-with-db.sh` so local and CI runs execute the same test command.

Pass extra arguments through to Cargo:

```sh
resources/scripts/run-tests-with-db.sh --lib -p robominer-domain
resources/scripts/run-tests-with-db.sh -p robominer-web -- login
```

Without a database URL, DB-backed integration tests skip themselves (they print a message and
return). Golden and unit tests still run. When `CI=true`, missing `ROBOMINER_DATABASE_URL` fails
the run instead of skipping.

Use `robominer_test_support::require_test_db()` in new DB integration tests instead of copying the
skip boilerplate.

### Updating golden fixtures

Golden JSON fixtures live under `*/tests/fixtures/`. To refresh them locally after an intentional
simulation or claim change, set the update env var and re-run the matching test binary:

| Test | Env var |
|------|---------|
| `robominer-domain/tests/rally_golden.rs` | `UPDATE_RALLY_GOLDEN=1` |
| `robominer-domain/tests/pool_golden.rs` | `UPDATE_POOL_GOLDEN=1` |
| `robominer-db/tests/claim_golden.rs` | `UPDATE_CLAIM_GOLDEN=1` |

Do not commit updated fixtures unless the behavior change is deliberate. CI never sets these vars.

### Compile-checked SQL (`query!`)

Claim, mining-queue enqueue/cancel, and user session bump / last-login hot paths use
`sqlx::query!` / `query_scalar!` with offline metadata under [`.sqlx/`](.sqlx/). After editing
those SQL strings or the tables they touch:

```sh
export DATABASE_URL=mysql://robominer:password@127.0.0.1:3306/RoboMiner
cargo sqlx prepare --workspace -- --package robominer-db --lib
```

Commit the regenerated `.sqlx/` JSON. CI and local builds without a live DB should set
`SQLX_OFFLINE=true` (CI already does). Dynamic `IN (...)` lists and other built SQL must use
[`robominer_db::assert_sql_safe`](robominer-db/src/query_util.rs) (sqlx 0.9 `SqlSafeStr`).
See [docs/SQLX-0.9-UPGRADE.md](docs/SQLX-0.9-UPGRADE.md).

### Fast tests (no database)

For library unit tests and simulation goldens that do not need MySQL:

```sh
resources/scripts/run-fast-tests.sh
```

That also runs the rally animation JS tests (Node `node:test`, no npm packages).

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
`ROBOMINER_COVERAGE_FAIL_UNDER_LINES` (currently 93 in CI).

Set the threshold locally:

```sh
ROBOMINER_COVERAGE_FAIL_UNDER_LINES=93 resources/scripts/run-coverage-with-db.sh
```

## Page-scoped CSS

Styles live under `robominer-web/static/css/pages/`. Every page loads the shared layout partials
(`layout_shell.css`, `layout_dialogs.css`, `layout_tables.css`) plus only the page file(s) it needs
via `robominer_stylesheet_tags(&[PageStylesheet::…])` (see `static_assets.rs`). Pass that slice as
the last argument to `html::layout`, or call the helper directly for auth/logoff shells.

When adding a page: create `static/css/pages/<name>.css`, add a `PageStylesheet` variant, and
link only that variant (do not reintroduce “load every CSS file” in layout). Shared strips used by
more than one page (for example `PageStylesheet::PageWallet` → `page_wallet.css`) are requested
alongside the page file, not baked into every layout. Truly cross-page chrome that every page may
show (help hints) belongs in `layout_shell.css`.

Some variants emit more than one file (for example `MiningQueue` also loads
`mining_queue_robots.css`; `Rally` also loads `rally_sidebar.css`). Treat
`PageStylesheet` in `static_assets.rs` as the source of truth for which files each
variant includes.

| Page / shell | Stylesheets (`PageStylesheet` …) |
|--------------|----------------------------------|
| Auth / logoff | layout + `Auth` |
| Account | layout + `Auth` + `Account` |
| Mining queue | layout + `PageWallet` + `MiningQueue` |
| Mining area atlas | layout + `MiningAreaAtlas` |
| Mining results | layout + `MiningResults` |
| Activity | layout + `Activity` |
| Rally replay | layout + `Rally` |
| Edit code | layout + `EditCode` |
| Robot workshop | layout + `Robot` |
| Achievements | layout + `Achievements` |
| Shop | layout + `PageWallet` + `Shop` |
| Help | layout + `Help` |
| Leaderboard | layout + `Leaderboard` |
| Robot stats | layout + `RobotStats` |

## HTML escaping

Untrusted strings in server-built HTML must go through `EscapedHtml` / `html_attr`
(`robominer-web/src/html/format.rs`). Do not interpolate raw user or DB text into
`format!` HTML.

JS `innerHTML` is allowed only for:

- Server-rendered fragments that were already escaped in Rust (for example HUD /
  robot rows in `static/js/mining_queue/view.js`), or
- Non-user integers / fixed markup (for example line numbers in
  `static/js/edit_code/editor.js`).

When adding a new `format!` HTML path or `innerHTML` assignment, confirm every
dynamic field is escaped server-side (or is a trusted integer / constant).

## Splitting a web page module

**Naming:** use singular `*_page` for one primary route (`shop_page`, `mining_queue_page`).
Use plural `*_pages` for a family of related routes in one folder (`auth_pages`, `rally_pages`,
`help_pages`). Prefer renaming only when already touching that module.

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

Existing splits follow this layout:

| File | Responsibility |
| --- | --- |
| `mod.rs` | Handler entry, page state structs, load orchestration |
| `actions.rs` | POST mutation dispatch (`submitType`, form keys, buy/sell, etc.) |
| `view_model.rs` | DB record → view struct mapping (when mapping is non-trivial) |
| `render.rs` | HTML only |
| `tests.rs` or `tests/` | Pure render/helper tests |

Examples: `shop_page/` (with `actions.rs`), `mining_queue_page/` (with `actions.rs` +
`view_model.rs`), `edit_code_page/` (with `actions.rs`), `account_page/` (with `actions.rs`),
`achievements_page/` (with `actions.rs`), `robot_page/` (with `actions.rs`), `auth_pages/`,
`rally_pages/`, `leaderboard_page/`, `mining_results_page/`, `mining_area_overview_page/`,
`robot_stats_page/`, `help_pages/`.

New web pages should use the same split (start with `mod.rs` + `render.rs` + `tests.rs`;
add `actions.rs` when POST branches appear; add `view_model.rs` when DB→view mapping grows).

## Test layout conventions

| Layer | Location | When to use |
|-------|----------|-------------|
| Page render/helpers | `robominer-web/src/<page>/tests.rs` or `…/tests/` | Pure HTML and helper logic; no live HTTP or DB |
| Help content | `robominer-web/static/help/*.html` | Guide bodies loaded with `include_str!`; rendering in `help_pages/` |
| HTTP + DB integration | `robominer-web/tests/*.rs` | POST/GET through `route()` with real MySQL |
| Engine CLI integration | `robominer-engine/tests/*_db_cli.rs` | Subprocess `robominer-engine` against MySQL |
| DB mutations | `robominer-db/tests/` | Direct SQL helpers without CLI or HTTP (`db_mutations_*.rs`, `db_users.rs`, `db_rally_*.rs`, `db_activity.rs`, `db_pool.rs`, `db_program_sources.rs`, `db_mining_areas.rs`, `db_mining_queue.rs`, `db_robots.rs`, `db_achievements.rs`, `db_migrate.rs`, `claim_golden.rs`) |
| Domain goldens | `robominer-domain/tests/*_golden.rs` | Deterministic simulation fixtures |
| Rally animation JS | `robominer-web/static/js/rally_animation/tests/` | Headless Node tests of viewer payload/draw helpers |
| Shared fixtures | `robominer-test-support/` | SQL setup reused by web and engine tests; composable `Scenario::user_with_robot_and_wallet()` wraps common user+robot+wallet setup |

Engine integration tests use `mod support; use support::*;` and `#[serial]` because they share
one MySQL instance.

## Route-to-test matrix

“Page unit” = tests in `robominer-web/src/<page>/tests.rs` or `…/tests/` (or inline
`#[cfg(test)]` in the page module). “Web DB” = `robominer-web/tests/`. “Engine CLI” =
matching `*_db_cli.rs` binary.

| Route / feature | Page unit | Web DB | Engine CLI | Notes |
|-----------------|-----------|--------|------------|-------|
| `/` redirect | `router` tests | `web_db_smoke` | — | Logged-in → mining queue |
| `/login`, signup | `auth_pages/tests.rs` | `login.rs` | `user_create_db_cli.rs`, `user_login_db_cli.rs` | Session cookie minted at login; signup POST is `/login` |
| `/logoff` | `auth_pages/tests.rs`, `router` | `security_hardening.rs` | — | POST + CSRF clears session; GET shows page without clearing |
| `/account` | `account_page` | `account_actions.rs` | `user_account_update_db_cli.rs` | Profile/password updates |
| `/achievements` | `achievements_page/tests/` | `achievement_claim.rs` | `achievement_db_cli.rs` | Claim rewards |
| `/editCode` | `edit_code_page/tests/` | `edit_code_actions.rs` | `program_source_db_cli.rs` | Create, apply, and delete sources |
| `/robot` | `robot_page/tests/` | `robot_apply.rs` | `robot_config_db_cli.rs`, `claim_robot_config_db_cli.rs` | Apply config (wallet claim is engine-side) |
| `/robotStats` | `robot_stats_page/tests.rs` | `read_model_pages.rs` | — | No dedicated engine CLI |
| `/shop` | `shop_page/tests/` | `shop_actions.rs` | `shop_db_cli.rs` | Buy/sell parts |
| `/miningQueue` | `mining_queue_page/tests/` | `mining_queue_actions_*.rs` | `mining_queue_db_cli.rs`, `mining_queue_read_model_db_cli.rs` | Enqueue, fill, cancel + read model |
| `/miningResults` | `mining_results_page/tests/` | `read_model_pages.rs` | `mining_area_read_model_db_cli.rs` | Rally replay also via `?rallyResultId=` |
| `/leaderboard` | `leaderboard_page/tests/` | `read_model_pages.rs` | `leaderboard_read_model_db_cli.rs` | |
| `/miningAreaOverview` | `mining_area_overview_page/tests.rs` | `read_model_pages.rs` | `mining_area_overview_read_model_db_cli.rs` | |
| `/activity` | `rally_pages/tests/` | `read_model_pages.rs` | `activity_read_model_db_cli.rs`, `rally_read_model_db_cli.rs` | Activity feed + rally replay UI; JS viewer in `rally_animation/tests/` |
| `/help*` | `help_pages/tests.rs`, `help_pages/render.rs` | — | — | Routes + content/rendering in `help_pages/`; bodies in `static/help/`; smoke hits `/help` |
| `/health` | — | `health.rs` | — | DB ping + migration readiness |
| Rally worker / wallet claim | — | `web_db_smoke` (indirect) | `rally_db_cli_*.rs`, `pool_db_cli.rs`, `claim_robot_config_db_cli.rs` | `rally rallies --persist` runs a wallet claim pass; standalone `mining claim-all` |
| Program compile | `robominer-program` unit | — | `verify_source_cli.rs` | No DB |
| Migrate CLI | — | — | `migrate_db_cli.rs` | Schema migrations |
| Simulation goldens | — | — | — | `robominer-domain/tests/rally_golden.rs`, `pool_golden.rs` |

## Crate boundary: `robominer-db` vs `robominer-domain`

`robominer-db` is persistence, typed mutation contracts, **and transactional
multi-table game rules** (shop affordability, queue capacity, achievement claim
graphs, claim tax). `robominer-domain` is loadouts, simulation/verify façades,
and shared rejection copy on top of db (plus `robominer-program` / `robominer-sim`).

Use the decision tree in [docs/architecture.md](docs/architecture.md) when adding
behaviour: transactional invariants across tables → db write module; simulation /
loadout / program verify → domain; player/CLI copy → `rejection_messages`.

Dependency direction is one-way: **domain may depend on db; db must not depend on
domain, sim, or program.**

| Put it in… | When… |
| --- | --- |
| `robominer-db` | SQL, migrations, pool/config, record DTOs, typed `*Request` / `*Rejection` / read models, **and transactional game rules** |
| `robominer-domain` | Loadout assembly, rally/pool run + persist façades, program create/update with verify, player/CLI rejection strings |
| `robominer-web` / `robominer-engine` | HTTP/CLI presentation, routing, formatting beyond shared rejection strings |

Prefer `robominer_db::module::…` paths for **new** APIs and when updating hot
production call sites; **no new root wildcards** (`pub use module::*` at the
crate root). Existing flat re-exports stay for compatibility. See
[docs/crate-map.md](docs/crate-map.md).

### Schema changes

Migrations are **forward-only**. There are no down migrations. To roll back a
failed schema change, restore a pre-migration database dump (see
`resources/scripts/restore-database.sh`) and redeploy application binaries that
match that schema era. Take a backup before `migrate apply` in production.

Prefer `robominer-engine migrate apply` as the primary migrate path; treat
`resources/scripts/migrate-database.sh` as bootstrap/fallback.

A schema change needs three synchronized artifacts:

1. Fresh DDL in `resources/database/createDatabase.sql`
2. An incremental file under `resources/database/migrations/`
3. A probe in `robominer-db/src/migrate/schema.rs` `schema_already_current` (and the matching checklist in `resources/scripts/migrate-database.sh` when that script is used)

Missing a probe can silently baseline an empty `SchemaMigration` table without applying the new change.

### Rules

These rules still describe how the workspace is structured. Prefer them when adding
features; do not introduce a parallel “domain gateway” for ordinary CRUD.

1. **All production SQL lives in `robominer-db`.** Domain `src` may call db helpers and map results; it must not contain `sqlx::query` or raw SQL. Integration tests under `robominer-domain/tests/` may use SQL for fixtures.
2. **Prefer `sqlx::query_as` with small `FromRow` structs for new SQL readers.** Map rows into typed records under `robominer-db/src/types/records/` rather than manual column indexing; migrate legacy readers only when touching the module.
3. **Db returns typed rejections and records, not player-facing prose.** Enums such as `EnqueueMiningRejection` live with the mutation; strings live in `robominer-domain` (`rejection_messages`).
4. **Loadout assembly and simulation belong in domain.** Build `RallyLoadout` / `PoolLoadout`, run them, map outcomes to completed records, then call db persist helpers.
5. **Use a domain façade only when a write spans db + non-db rules.** Program create/update must go through `robominer_domain::create_program_source` / `update_program_source` so compile verification runs. Do not call the bare db helpers from web/engine for that path.
6. **Otherwise prefer direct `robominer_db` from web/engine.** Shop buy, enqueue mining, claim achievement, page read models, and similar CRUD call db, then map rejections through domain message helpers.
7. **Do not push sim/compile into db.** Db may store verification flags; domain/engine owns invoking `robominer_program::verify_source` (domain façades for program save; engine `verify` CLI may mark validity without going through the façade).
8. **Do not grow a general “domain API gateway.”** Thin façades that only forward to db without extra rules are noise—call db from the edge instead.
9. **Web page loaders and handlers use `PageLoadError`, not `DomainError`.** HTML page modules return `crate::page_context::PageLoadError` for SQL/load failures (including after domain program writes via `PageLoadError::from_domain`). Reserve `robominer_domain::DomainError` for loadout/simulation and other domain rule failures.
10. **Test-only re-exports in release builds must stay cfg-gated.** Helpers such as `reset_*_rate_limiter_for_tests` in `robominer-web/src/rate_limit/` are defined under `#[cfg(any(test, debug_assertions))]`; any `pub use` of them must use the same cfg so `cargo build --release` stays green.

### Examples

- **Rejection split:** `robominer_db::claim_achievement_step` returns `ClaimAchievementStepRejection`; web/engine use `robominer_domain::claim_achievement_step_rejection_message`.
- **Sim pipeline:** engine `rally rallies` uses domain `load_next_rally_loadout_with_claim` (persist path) → `run_rally_loadout_*` → `persist_rally_outcome`; SQL for claim/lease + persist stays in `robominer-db`. Concurrent workers use `FOR UPDATE SKIP LOCKED` (MariaDB-compatible; no MySQL-only `OF table`) leases and conditional `miningEndTime IS NULL` updates.
- **Anti-pattern:** Calling `robominer_db::create_program_source` from web/engine and skipping domain drops verify-and-mark. Embedding `"Unknown robot"`-style strings inside db mutation modules likewise breaks the split. Returning `DomainError` from a page `list_*` loader (via `From<sqlx::Error>`) is the same anti-pattern—use `PageLoadError`.

See also [User-facing rejection messages](#user-facing-rejection-messages) below and the achievements-specific Code and UI table in [ACHIEVEMENTS.md](ACHIEVEMENTS.md).

## Error handling

Use the narrowest error type for each boundary:

| Situation | Return type | Notes |
| --- | --- | --- |
| HTML page loader / handler (read model or post-then-reload) | `robominer_web::PageLoadError` | SQL/load failures; do not surface `DomainError` as the page `Result` error type |
| Loadout assembly / simulation orchestration | `robominer_domain::DomainError` | Includes opaque `DatabaseError` for unexpected SQL failures |
| Typed DB mutation (web/engine) | `DbOutcome<T, R>` | Success value or typed rejection enum from `robominer-db` |
| Player/CLI copy for rejections | `robominer_domain::rejection_messages` | Map rejections with `Audience::Player` or `Audience::Cli` helpers |
| Engine CLI convenience | `anyhow::Result` + `finish_db_outcome` | String diagnostics for operators |

```mermaid
flowchart TD
  pageGet[Page GET loader] --> pageLoadError[PageLoadError]
  domainSim[Domain loadout/sim] --> domainError[DomainError]
  dbMutation[DB mutation] --> dbOutcome[DbOutcome T R]
  dbOutcome --> rejectionCopy[rejection_messages Audience]
  engineCli[Engine CLI] --> anyhowErr[anyhow Result]
  dbOutcome --> anyhowErr
```

Architecture overview: [docs/architecture.md](docs/architecture.md). Module ownership in large crates: [docs/crate-map.md](docs/crate-map.md).

## User-facing rejection messages

Player-visible web copy and engine CLI diagnostics both come from
`robominer_domain::rejection_messages` (see crate boundary above):

- Web pages call the `*_player_message` helpers (often via thin `pub(super)` wrappers in the page module).
- Engine CLI commands call the matching `*_cli_message` helpers.

When changing copy, update the central module and keep the page-module parity tests and
`*_db_cli.rs` integration tests green.

**Account enumeration trade-off:** signup and account-update intentionally return
distinct `DuplicateUsername` / `DuplicateEmail` messages so players get actionable
UX (“username taken” vs “email already registered”). That reveals account existence.
Login failures stay generic (`login_failure_message`) and do not distinguish unknown
user vs bad password. Keep this split unless product requirements change.

## Generated assets

Rally animation version constants are generated by `robominer-sim/build.rs` into
`robominer-web/static/js/rally_animation/generated/contract.js`. See
[docs/animation-payload.md](docs/animation-payload.md) for the build-order checklist
when bumping `ANIMATION_PAYLOAD_VERSION`.

## Benchmarking robot programs

When comparing programs or validating balance advice:

```sh
cargo test -p robominer-domain benchmark_recommended_programs -- --nocapture
```

Harness: `robominer-domain/tests/program_recommendations.rs`. Scores are printed after tax; the default run averages 20 seeds (0..19) per program/area/robot combination. This harness is diagnostic output only—it is not part of the golden fixture suite and does not require a database.
