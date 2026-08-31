# `robominer-db` module map

The crate root re-exports symbols for convenience (`robominer_db::enqueue_mining`, etc.). Prefer explicit module paths for new APIs.

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

See [architecture.md](architecture.md) for how this layer connects to `robominer-domain` and presentation crates.
