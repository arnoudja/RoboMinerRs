# Crate module maps

Module ownership for large workspace crates. See [architecture.md](architecture.md) for layer
boundaries and the “where does this rule live?” decision tree.

## `robominer-db`

The crate root re-exports symbols for convenience (`robominer_db::enqueue_mining`, etc.). Prefer
explicit module paths for new APIs.

**Note:** `robominer-db` holds both persistence and **transactional game rules** (shop economics,
queue capacity, achievement unlock graphs, claim tax). Domain orchestrates simulation; db owns
invariants that must stay inside a SQL transaction.

| Module | Responsibility | Primary entry points |
| --- | --- | --- |
| `achievements/` | Achievement progress and claims | `claim_achievement_step`, page read models |
| `activity/` | Activity feed read models | `list_activity_recent_rally_feed` |
| `app_shell/` | Shared navigation/shell data | `load_app_shell` |
| `catalog/` | Ores, part types, shop catalog | `list_ores`, `list_shop_robot_part_catalog`, `part_type_id` |
| `config.rs` | Legacy config and CLI DB URL resolution | `connect_from_cli`, `resolve_database_url` |
| `leaderboard/` | Leaderboard snapshots | area/player/robot leaderboard reads |
| `migrate/` | Schema migration runner | `run_embedded_migrations`, `migration_status` |
| `mining_areas/` | Area metadata and costs | area overview reads |
| `mining_queue/` | Queue enqueue/cancel/fill | `enqueue_mining`, `cancel_mining_queue` |
| `pool.rs` | Pool rally loadouts and persist | pool read/write helpers |
| `program_sources/` | Program library CRUD | `create_program_source`, `update_program_source` |
| `rally/` | Rally lease, persist, wallet claim | `claim_user_results`, `persist_completed_rally` |
| `results/` | Mining result history | results list/detail reads |
| `robots/` | Robot CRUD and workshop reads | `update_robot_config`, `list_robot_config_states` |
| `shop/` | Part buy/sell | `buy_robot_part`, `sell_robot_part` |
| `users/` | Accounts and auth | `create_user`, `verify_login` |

## `robominer-domain`

Thin orchestration on top of db + sim + program. Not a general API gateway for CRUD.

| Module | Responsibility | Primary entry points |
| --- | --- | --- |
| `loadout/` | Assemble rally/pool/mining loadouts from db rows | `load_rally_loadout`, `load_next_rally_loadout_with_claim` |
| `simulation/` | Run rallies/pools, map outcomes to records | `run_rally_loadout_*`, `persist_rally_outcome` |
| `robot_config.rs` | Program create/update with compile verify | `create_program_source`, `update_program_source` |
| `rejection_messages/` | Player/CLI prose for typed db rejections | `*_rejection_message`, `Audience` |

## `robominer-web`

Axum transport shell with a custom HTML router. Page modules follow `mod.rs` + `actions.rs` +
`view_model.rs` + `render.rs` + tests (see [CONTRIBUTING.md](../CONTRIBUTING.md)).

| Module / folder | Responsibility |
| --- | --- |
| `router/` | Session gate, legacy redirects, route dispatch, auth policy |
| `routes.rs` | Canonical paths, `AppRoute`, `RoutePolicy` |
| `page_context.rs` | `PageSession`, `PageLoadError`, HUD render helpers |
| `csrf/`, `session/`, `rate_limit/` | Security and session management |
| `*_page/` | Per-route handlers, mutations, view models, HTML |
| `static/` | CSS, JS, help HTML fragments |
| `tests/` | HTTP + DB integration tests |

## `robominer-engine`

CLI and background rally/mining worker. Command modules mirror web page domains.

| Module | Responsibility |
| --- | --- |
| `cli/` | Clap entry and per-domain `*Command` enums (`mining`, `rally`, …) |
| `dispatch/` | Subcommand routing (`shop`, `mining`, `rally`, `user`, …) |
| `rally/` | Worker loop (`cycle.rs`), single-rally run (`run_single.rs`) |
| `shop.rs`, `mining/`, `robot.rs`, … | Thin command handlers calling db/domain |
| `shutdown.rs` | Shared ctrl-c `ShutdownSignal` for rally/mining workers |
| `db_outcome.rs` | Map `DbOutcome` to `anyhow` for operators |
| `database.rs` | Connect wrapper around `robominer_db::connect_from_cli` |

See [architecture.md](architecture.md) for how these layers connect.
