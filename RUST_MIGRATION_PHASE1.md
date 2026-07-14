# Rust Migration Phase 1: Original System Map

This is a migration reference document. The Java servlet/JSP web application and
C++ worker it maps have been removed; the Rust workspace and `robominer-web`
host are now the canonical application code. Phase 1 migration is complete.

It originally mapped the pre-Rust RoboMiner system so migration work could
replace behavior deliberately instead of doing a line-by-line translation. Keep
the legacy sections below as compatibility context, not as a description of the
current runtime. For current setup and commands, see `README.md`; for achievements
see `ACHIEVEMENTS.md`.

## Runtime Shape

RoboMiner is now a Rust workspace sharing one MySQL schema:

- `robominer-web`: Rust HTTP host for login, account, queue, results, rally
  view, robot configuration, shop, achievements, leaderboard, activity, mining
  area overview, help, and logoff routes.
- `robominer-engine`: Rust CLI/daemon for verification, simulation, rally
  execution, pool execution, result persistence, and worker-loop operation.
- `robominer-db`, `robominer-domain`, `robominer-program`, and
  `robominer-sim`: Rust library crates for database access, domain rules,
  robot-language compilation, and deterministic simulation.

The database remains the integration boundary between the web host, engine, and
maintenance scripts.

## Build And Deployment Inputs

- Rust workspace build: `cargo build --workspace`.
- Rust workspace tests: `cargo test --workspace` (database integration tests need
  `ROBOMINER_DATABASE_URL`; see `README.md`).
- Full local test run with MySQL: `resources/scripts/run-tests-with-db.sh`
  (reuses local MySQL or a persistent Docker container when possible).
- Clippy: `cargo clippy --workspace -- -D warnings`.
- Rust web host: `cargo run -p robominer-web`.
- Web session signing: set `ROBOMINER_SESSION_SECRET` for signed login cookies.
- Rust engine: `cargo run -p robominer-engine -- --help`.
- Engine and web config: pass `--database-url`, set `ROBOMINER_DATABASE_URL`, or
  use a config file such as `/etc/robominer/robominer.conf` (the engine still
  accepts the legacy path `/etc/robominer/robominer-engine.conf`).
- Linux services: `deploy/systemd/robominer-engine.service` and
  `deploy/systemd/robominer-web.service` (see `deploy/systemd/README.md`).
- Database schema and seed scripts:
  - `resources/database/createDatabase.sql`
  - `resources/database/gameData.sql`
  - `resources/database/balanceTestData.sql`

## Web Request Inventory

The current Rust web host redirects logged-out users from `/` and protected
pages to `/login`. Logged-in `/` requests redirect to `/miningQueue`. CSS is the
only static runtime web asset; browser behavior that remains JavaScript is
rendered from Rust source.

| Route | Current owner | Main behavior |
| --- | --- | --- |
| `/login` | `robominer-web` | Log in by username/email, create new users, set signed session cookie and remember-username cookie, redirect to queue/help. |
| `/logoff` | `robominer-web` | Expire signed session cookie, username cookie, and legacy session cookies. |
| `/account` | `robominer-web` | Process claimable assets, update username/email/password after current-password check. |
| `/miningQueue` | `robominer-web` | Process claimable assets, add/fill/remove queue entries, pay mining costs, render per-robot queue state. |
| `/miningResults` | `robominer-web` | Process claimable assets, show recent results or a user-owned rally animation. |
| `/robot` | `robominer-web` | Change robot name, program, and parts; create pending robot changes when needed. |
| `/editCode` | `robominer-web` | Create/update/delete program sources, run Rust verifier, apply verified code to idle robots. |
| `/shop` | `robominer-web` | Process claimable assets, buy/sell robot parts, update ore and part assets. |
| `/achievements` | `robominer-web` | Process claimable assets, claim achievement steps and rewards. |
| `/leaderboard` | `robominer-web` | Show top robots, top users, and per-area robot scores. |
| `/miningAreaOverview` | `robominer-web` | Show ore and mining-area statistics. |
| `/activity` | `robominer-web` | Show recent users/rallies or any rally animation by ID. |
| `/help` | `robominer-web` | Show help index and embedded help pages (`/helpTutorial`, `/helpProgramTips`, `/helpRobotProgram`, `/helpMechanics`). |

Historical shared servlet behavior in `RoboMinerServletBase`:

- Session user id helpers.
- Form id parsing through `getItemId`.
- `processAssets`, which claims completed mining results before several pages.
- `payMiningCosts`, which deducts ore before queue insertion.

Historical static browser assets:

- JavaScript modules in `RoboMinerWeb/web/js`: common helpers, login/account
  validation, edit-code behavior, mining queue timers, robot form behavior,
  shop behavior, achievements behavior, rally animation. These runtime JS files
  have been removed; the rally animation renderer now lives in Rust source.
- JSP tag files in `RoboMinerWeb/web/WEB-INF/tags`: default page frame,
  menu bar, user asset display, footer, duration formatting. The Rust web host
  now owns page layout and formatting.

## Java Domain And Service Inventory

### Service/Facade Layer

- `AbstractFacade<T>` provides generic JPA create/edit/remove/find/list/count.
- `UserFacade` handles user lookup, duplicate username/email checks, recent
  users, and top achievement users.
- `UserAssets` is the central transactional service for:
  - creating a user with defaults and initial achievement progress;
  - claiming finished mining queue results into user ore assets;
  - calculating tax/reward from mining results;
  - updating robot lifetime totals and mining-area lifetime totals;
  - deducting mining costs.
- `RoboMinerCppBean` invokes the native verifier binary from the web app.
- Other facades wrap table-specific queries:
  - `MiningQueueFacade`
  - `RobotFacade`
  - `ProgramSourceFacade`
  - `RobotPartFacade`
  - `RobotPartTypeFacade`
  - `OreFacade`
  - `UserOreAssetFacade`
  - `UserRobotPartAssetFacade`
  - `AchievementFacade`
  - `MiningAreaFacade`
  - `MiningAreaLifetimeResultFacade`
  - `MiningOreResultFacade`
  - `PendingRobotChangesFacade`
  - `RallyResultFacade`
  - `RobotMiningAreaScoreFacade`
  - `TopRobotsViewFacade`

### Domain Clusters

- Accounts and ownership: `User`, `UserOreAsset`, `UserRobotPartAsset`.
- Robot configuration: `Robot`, `RobotPart`, `RobotPartType`,
  `PendingRobotChanges`, `ProgramSource`.
- Mining/rally workflow: `MiningArea`, `MiningAreaOreSupply`, `MiningQueue`,
  `RallyResult`, `MiningOreResult`, `RobotActionsDone`.
- Scoring/statistics: `RobotMiningAreaScore`, `RobotLifetimeResult`,
  `MiningAreaLifetimeResult`, `TopRobotsView`, `RobotStatistics`.
- Economy/prices: `Ore`, `OrePrice`, `OrePriceAmount`.
- Achievements: `Achievement`, `AchievementStep`, `AchievementPredecessor`,
  `AchievementStepMiningTotalRequirement`,
  `AchievementStepMiningScoreRequirement`, `UserAchievement`.
- Pool simulation tables are present in the schema and used by C++:
  `Pool`, `PoolItem`, `PoolItemMiningTotals`.

## Historical C++ Command And Job Inventory

The removed C++ binary supported three command modes:

- `robominercpp verify <program_source_id>`
  - Reads `ProgramSource.sourceCode`.
  - Parses/compiles the robot language.
  - Updates `ProgramSource.verified`, `compiledSize`, and `errorDescription`.
- `robominercpp runpool <pool_id>`
  - Loads one `Pool` and its `PoolItem` rows.
  - Runs repeated four-robot rallies until required runs are complete.
  - Updates `PoolItem.totalScore`, `PoolItem.runsDone`, and
    `PoolItemMiningTotals`.
- `robominercpp` with no command
  - Runs indefinitely.
  - Loads mining areas once at startup.
  - Every loop selects eligible queue items per mining area.
  - Starts a rally when four distinct users are available or the earliest item
    is close to expiry.
  - Fills empty rally slots with the area AI robot.

C++ database responsibilities:

- Reads: `ProgramSource`, `MiningArea`, `MiningAreaOreSupply`, `Robot`,
  `MiningQueue`, `RobotMiningAreaScore`, `Pool`, `PoolItem`.
- Inserts: `RallyResult`, `MiningOreResult`, `RobotActionsDone`,
  `RobotMiningAreaScore`, `PoolItemMiningTotals`.
- Updates: `ProgramSource`, `MiningQueue`, `Robot`, `RobotMiningAreaScore`,
  `PendingRobotChanges`, `PoolItem`.
- Deletes old claimed queue data: `MiningQueue`, `MiningOreResult`,
  `RobotActionsDone`, and unused `RallyResult`.

C++ code modules:

- Robot language parser/compiler: `RoboMinerCpp/robotcode`.
- Rally simulation: `Rally`, `Robot`, `Ground`, `GroundUnit`, `Position`.
- Animation serialization: `Animation`, `AnimationArrayData`,
  `AnimationStep`, `GroundChangeStep`.
- Database integration: `Database`, `DatabaseStatement`, `ConfigFile`.

## Historical Database Ownership Map

Treat this table as the original compatibility contract for Rust migration. It
records the Java/C++ ownership that Rust replaced or mirrored.

| Table/View | Historical writers | Historical readers | Notes |
| --- | --- | --- | --- |
| `User` | Java | Java, C++ indirectly through joins | Authentication, profile, achievements, queue size. |
| `UserOreAsset` | Java | Java | User wallet and max ore caps. Updated when claiming results and buying/selling. |
| `ProgramSource` | Java, C++ verifier | Java, C++ verifier | Java writes source; C++ writes verification status. |
| `Robot` | Java, C++ daemon | Java, C++ daemon | Java submits config; C++ updates active mining/recharge state and applies pending changes. |
| `PendingRobotChanges` | Java, C++ daemon, Java asset claim cleanup | Java, C++ daemon | Bridge for robot changes that wait for mining completion. |
| `RobotPartType`, `RobotPart` | Seed data | Java | Shop/configuration catalog. |
| `UserRobotPartAsset` | Java | Java | Owned robot parts. |
| `Ore`, `OrePrice`, `OrePriceAmount` | Seed data | Java, C++ | Economy and mining cost data. |
| `MiningArea`, `MiningAreaOreSupply` | Seed data | Java, C++ | Mining environment and AI robot reference. |
| `UserMiningArea` | Java achievement claiming/seed data | Java | Unlocks available mining areas. |
| `MiningQueue` | Java, C++ daemon | Java, C++ daemon | Queue entries are created by Java, executed/scored by C++, claimed by Java. |
| `RallyResult` | C++ daemon | Java, C++ cleanup | Stores serialized animation data. |
| `MiningOreResult` | C++ daemon, Java claim tax update | Java, C++ cleanup | Raw amount from C++; tax/reward calculated in Java on claim. |
| `RobotActionsDone` | C++ daemon | Java, C++ cleanup | Per-rally action counters. |
| `RobotMiningAreaScore` | C++ daemon | Java, C++ daemon | Rolling per-robot score per mining area. |
| `RobotLifetimeResult` | Java claim flow | Java | Per-robot mined totals/taxes after user claims results. |
| `MiningAreaLifetimeResult` | Java claim flow | Java | Area-level mined totals and container-size totals. |
| Achievement tables | Seed data, Java claim flow | Java | Claiming grants points, queue size, robots, areas, and ore caps. |
| `TopRobotsView` | Derived view | Java | Leaderboard view over robot lifetime totals. |
| Pool tables | C++ pool command, seed/manual setup | C++ | CLI-only in Rust too; no web UI. |

Rust now owns all live database writers for the tables above. The Java/C++
columns record the original ownership for compatibility testing and fixture
context.

## Behavior To Preserve First

These are the high-risk behavioral contracts to test before replacing code:

- User creation initializes defaults, first achievement progress, initial robot,
  initial parts, default program, mining areas, and ore caps according to seed
  data behavior.
- Program verification exactly matches current accepted/rejected robot language
  and compiled-size calculation.
- Robot configuration enforces ownership, memory capacity, pending-change rules,
  and verified-source application.
- Queue insertion deducts mining costs and respects per-user queue size.
- Rally selection allows at most one queued robot per user per rally and fills
  missing slots with the mining area's AI robot.
- Rally output preserves the legacy `RallyResult.resultData` payload shape for
  compatibility with existing stored results and Rust-rendered animation.
- Claiming results must calculate tax/reward identically and update user assets,
  robot lifetime totals, mining-area lifetime totals, achievements, and pending
  robot change cleanup.
- Score updates must preserve the rolling score formula and initial score
  discount.

## Current Rust Boundary

The current Rust crates mirror the existing schema boundary:

- `robominer-db`: typed MySQL access and transactions for the existing schema.
- `robominer-domain`: users, robots, parts, ore, mining areas, queues,
  achievements, and scoring rules.
- `robominer-program`: parser/compiler/interpreter for the robot language.
- `robominer-sim`: pure Rust rally/world mechanics for deterministic tests.
- `robominer-engine`: CLI/daemon replacement for `RoboMinerCpp`.
- `robominer-web`: HTTP handlers, templates, signed session cookies, forms, CSS
  serving, and Rust-owned browser behavior rendering.

Phase 1 migration is complete. The initial migration path delivered:

1. `robominer-engine verify <program_source_id>` against the `ProgramSource`
   table.
2. Golden tests for parser/compiler behavior from known source snippets.
3. `robominer-engine verify-source <path>` for Rust-only parser/compiler
   development without MySQL.
4. One-shot and continuous rally execution with DB persistence.
5. Rust web and engine binaries replacing the Java servlet host and C++
   worker.
6. Legacy behavior preserved through golden tests, fixture programs, and the
   unchanged MySQL schema.

## Engine And CLI Usage

For Rust-only verification, run the parser/compiler directly against a source
file:

```bash
cargo run -p robominer-engine -- verify-source path/to/program.txt
```

For a Rust-only simulation run, execute one program on a deterministic map with
one ore heap:

```bash
cargo run -p robominer-engine -- simulate-source path/to/program.txt \
  --turns 10 \
  --size-x 5 \
  --size-y 5 \
  --ore-x 1 \
  --ore-y 1 \
  --ore-type 0 \
  --ore-amount 8
```

For a multi-robot Rust-only simulation, pass repeated `--robot` files instead of
the positional source file:

```bash
cargo run -p robominer-engine -- simulate-source \
  --robot seed-ai-1.txt \
  --robot seed-ai-2.txt \
  --robot seed-ai-3.txt \
  --turns 20
```

The command prints one summary block per robot and pairwise final distances.

For a manual DB-backed rally run, execute one mining area through the Rust
read/run path:

```bash
ROBOMINER_DATABASE_URL='mysql://robominer:password@localhost/RoboMiner' \
cargo run -p robominer-engine -- run-rally \
  --mining-area-id 1001 \
  --seed 0
```

The command loads the next ready rally for that area, runs it in memory, prints
queued and AI participant summaries, and defaults to a dry run. To write the
result transaction, pass `--persist`. Rust now generates the legacy
`RallyResult.resultData` JavaScript payload (`myRobots`, `myGround`, and
`myOreTypes`) during simulation. `--result-data-file <path>` remains available
as an override for compatibility experiments.

For a manual DB-backed pool rally run, execute one pool through the Rust
read/run path:

```bash
ROBOMINER_DATABASE_URL='mysql://robominer:password@localhost/RoboMiner' \
cargo run -p robominer-engine -- run-pool \
  --pool-id 1 \
  --seed 0
```

The command loads the next legacy-ordered pool item cohort, skips missing,
empty, or already-complete pools, runs one pool rally in memory, and defaults to
a dry run. Add `--persist` to increment `PoolItem.totalScore`,
`PoolItem.runsDone`, and `PoolItemMiningTotals`. To mirror legacy
`robominercpp runpool <pool_id>`, add `--until-complete --persist`; the command
reloads the pool after each persisted rally and stops when the pool is complete
or no runnable items remain. `--max-rallies <n>` defaults to 100 as a guard.

To process every mining area once:

```bash
ROBOMINER_DATABASE_URL='mysql://robominer:password@localhost/RoboMiner' \
cargo run -p robominer-engine -- run-rallies \
  --once \
  --seed 0
```

`run-rallies` scans all `MiningArea` rows, runs the same Rust read/run path for
areas with a ready rally, skips areas without one, and prints ran/skipped
counts. Add `--persist` to write completed rallies for all processed areas.
Continuous polling is available but intentionally explicit:

```bash
ROBOMINER_DATABASE_URL='mysql://robominer:password@localhost/RoboMiner' \
cargo run -p robominer-engine -- run-rallies \
  --loop \
  --sleep-seconds 5 \
  --persist
```

`--loop` requires `--persist`, and `--once`/`--loop` are mutually exclusive. Each
poll cycle reloads mining areas, prints per-cycle ran/skipped counts, and sleeps
for the configured interval before checking again. Ctrl+C/SIGINT is handled
gracefully: the process records the shutdown request, lets the current poll
cycle finish, and exits before starting another sleep/poll cycle.

For Linux service deployment, use the systemd units in
`deploy/systemd/robominer-engine.service` and
`deploy/systemd/robominer-web.service`. The companion
`deploy/systemd/README.md` documents installing release binaries, shared config
at `/etc/robominer/robominer.conf`, enabling services, viewing journald logs,
and graceful `SIGINT` shutdown for the engine loop.

## Current Architecture

`robominer-db` exposes typed read/write access for the existing schema: users,
user ore assets, user robot-part assets, program sources, robot part catalogs,
robots, mining areas, mining area ore supplies, mining queue rows, rally
results, pool tables, and related result rows. The schema remains the
compatibility boundary; Rust code preserves legacy column semantics and payload
shapes.

`robominer-domain` assembles loadouts and domain rules above raw table rows:

- `RobotLoadout` from one robot and its six optional part slots.
- `MiningAreaLoadout` from a mining area, ore supplies, and AI robot.
- `RallyLoadout` from the mining area, queued robots, and AI fill count.
- `PoolLoadout` from a pool, mining area, and pool item robots.

It runs rallies in memory via `robominer-sim`, compiles robot programs through
`robominer-program`, and maps outcomes to write rows. Rally persistence inserts
`RallyResult`, updates `MiningQueue` and queued robots, applies pending robot
changes, inserts `MiningOreResult` and `RobotActionsDone`, updates
`RobotMiningAreaScore` with the legacy score-smoothing formula, and cleans up
old claimed queue rows. Pool persistence increments `PoolItem.totalScore`,
`PoolItem.runsDone`, and upserts `PoolItemMiningTotals`.

`robominer-web` owns all player-facing HTTP routes listed above. It calls
`robominer-domain` and `robominer-db` for queue management, result claiming,
robot configuration, shop transactions, achievements, and leaderboard data.

`robominer-engine` exposes the operational CLI:

- `verify` / `verify-source` for program verification.
- `simulate-source` for deterministic offline simulation.
- `run-rally` for one mining-area rally (dry-run by default; `--persist` writes).
- `run-rallies --once` or `--loop --persist` for the continuous rally worker.
- `run-pool` with optional `--until-complete --persist` for pool balancing.

`robominer-sim` records legacy animation data while it runs, including robot
position/ore timelines, initial and changed ground ore, and ore metadata. Rust
generates the legacy `RallyResult.resultData` payload (`myRobots`, `myGround`,
and `myOreTypes`) during simulation.

DB-backed integration tests live in `robominer-engine/tests/run_rally_db_cli.rs`
and are gated by `ROBOMINER_DATABASE_URL`. They cover rally and pool
persistence, queue cleanup, and most engine/web CLI read models. Golden tests in
`robominer-domain/tests/rally_golden.rs`, `robominer-db/tests/claim_golden.rs`,
and `robominer-program` compatibility fixtures guard behavioral parity.

## Robot Language And Simulation

`robominer-program` implements the full robot language verifier and an
executable interpreter used during rallies.

Verification (`verify_source`) runs both the legacy compiled-size parser and
executable compilation, so programs must parse and compile before they are
marked valid.

The executable runtime supports:

- Actions: `move`, `rotate`, `mine`, `dump`, and `scan`.
- Control flow: `if`/`else` (including `else if` chains), `while`, and
  `do-while`.
- Expressions: `time()`, `ore(n)`, `scan([direction])`, `oreDistance()`, `oreType()`,
  arithmetic, comparisons,
  boolean operators, `!`, and action return values such as `while (mine())`.
- Variables: declarations (including `const`), assignments, reads,
  increments/decrements, block scopes, and variable action arguments such as
  `rotate(rot)`.
- CPU budget: one instruction per expression node per mining cycle, bounded by
  the robot's `cpuSpeed`; `scan()` initiates a background scan that completes
  after the scanner's `scanTime` CPU cycles (passive progression each CPU tick).

Long `move(...)` and `rotate(...)` expressions continue across turns until
complete or blocked; collision-limited move return values feed back into
programs. `scan()` returns the remaining scan time in CPU cycles; use
`oreDistance()` and `oreType()` to read results once the scan completes.
`oreDistance()` returns -1 when no ore is found; `oreType()` returns the ore
quality index (0 = none, 1+ = quality slot), matching `ore()` and `dump()`.
Scan actions are recorded in `RobotActionsDone`.

Static-only programs (no runtime control flow or scan) take a fast path that
expands literal action sequences; everything else runs through the resumable
interpreter in `robominer-sim`.

`robominer-sim` implements rally mechanics: initial placement, movement, wall
clamping, multi-robot collision clipping, rotation, ore heap generation, mining
distribution, dumping, scoring, and action counting. Deterministic simulator
tests cover the seeded AI programs, multi-robot rallies, and compatibility
fixtures.

## Deployment Configuration

For current deployments, place shared config in
`/etc/robominer/robominer.conf` or set `ROBOMINER_DATABASE_URL`. If no
`--database-url` argument or `ROBOMINER_DATABASE_URL` environment variable is
provided, the Rust binaries read the `--config` file and build the database URL
from `dbserver`, `dbuser`, `dbpassword`, and `dbdatabase`.

## Open Questions For Phase 2

- Which existing database contents should become fixtures/golden tests?
- Should pool completion become part of a scheduled service, or stay an
  operator-triggered maintenance command?
- Should the Rust web layer stay server-rendered, or should it expose an API for
  a later frontend rewrite?
- Is preserving `RallyResult.resultData` mandatory long-term, or only during the
  compatibility phase?
- Should the Rust engine keep database polling, or should queue execution become
  an explicit scheduled job/worker model?
