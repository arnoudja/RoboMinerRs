# RoboMiner architecture

RoboMiner is a Rust workspace for a browser-based mining game backed by MySQL. Presentation layers call persistence directly for CRUD and route simulation-heavy work through the domain crate.

## Crate layers

```mermaid
flowchart TB
  subgraph presentation [Presentation]
    web[robominer-web]
    engine[robominer-engine]
  end
  subgraph domain [Domain]
    dom[robominer-domain]
  end
  subgraph core [Core]
    sim[robominer-sim]
    prog[robominer-program]
  end
  subgraph persistence [Persistence]
    db[robominer-db]
  end
  web --> dom
  web --> db
  engine --> dom
  engine --> db
  dom --> db
  dom --> sim
  sim --> prog
```

| Crate | Role |
| --- | --- |
| `robominer-program` | Robot language: parse, compile, interpret |
| `robominer-sim` | Mining physics, scoring, rally animation payloads |
| `robominer-db` | SQL, migrations, typed mutation contracts, read models |
| `robominer-domain` | Loadouts, simulation orchestration, program verify façades, rejection copy |
| `robominer-web` | Axum HTTP host, HTML pages, static assets |
| `robominer-engine` | CLI commands and background rally/mining worker |
| `robominer-optimize` | Offline genetic-algorithm tool (not on the production path) |
| `robominer-test-support` | Shared DB fixtures for integration tests |

Dependency direction is one-way: domain may depend on db; db must not depend on domain, sim, or program.

## Typical request flows

**Shop buy (web):** browser POST → `robominer-web` handler → `robominer_db::buy_robot_part` → typed rejection mapped through `robominer_domain::rejection_messages`.

**Rally worker (engine):** `robominer_domain::load_next_rally_loadout_with_claim` → `run_rally_loadout_*` → `persist_rally_outcome`; SQL stays in `robominer-db`.

**Program save (web):** handler → `robominer_domain::update_program_source` (compile verify) → db persist helpers.

See [CONTRIBUTING.md](../CONTRIBUTING.md) for crate-boundary rules and the route-to-test matrix.

## Frontend assets

Page HTML is rendered in Rust (`*_page/render.rs`). Client behaviour lives under `robominer-web/static/js/` with Node tests colocated in `*/tests/`. Rally replay uses a shared viewer in `static/js/rally_animation/`; its JSON contract is documented in [animation-payload.md](animation-payload.md).

## Database

Fresh environments load `resources/database/createDatabase.sql` plus incremental migrations under `resources/database/migrations/`. Runtime migrations are embedded at build time in `robominer-db` (see `build.rs`).

## Further reading

- [CONTRIBUTING.md](../CONTRIBUTING.md) — tests, crate boundaries, error handling
- [docs/crate-map.md](crate-map.md) — module map for large crates
- [docs/animation-payload.md](animation-payload.md) — rally replay JSON contract
- [AGENTS.md](../AGENTS.md) — cloud/dev environment notes
