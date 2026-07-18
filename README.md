# RoboMiner

RoboMiner is an online programming game. Improve the program for your robot to mine more efficiently.

## Prerequisites

- Rust toolchain with Cargo.
- MySQL or MariaDB.

The database scripts are kept under `resources/database/`:

- `createDatabase.sql` — database schema
- `gameData.sql` — seed data (ores, parts, areas, achievements)
- `balanceTestData.sql` — optional test fixtures

Further docs: [CONTRIBUTING.md](CONTRIBUTING.md) (tests, coverage, **db vs domain
boundary**), [ACHIEVEMENTS.md](ACHIEVEMENTS.md) (progression and claim flow),
[gameflow.md](gameflow.md) (early-game balance notes), [ROBOTS.md](ROBOTS.md)
(sample robot programs).

## Build

Build every Rust crate in the workspace:

```sh
cargo build --workspace
```

Build optimized release binaries:

```sh
cargo build --workspace --release
```

Build native release binaries and cross-compile for 64-bit Raspberry Pi
(`aarch64-unknown-linux-gnu`):

```sh
resources/scripts/build-release.sh
```

The main binaries are:

- `target/debug/robominer-engine`
- `target/debug/robominer-web`
- `target/release/robominer-engine`
- `target/release/robominer-web`



## Test And Check

See [CONTRIBUTING.md](CONTRIBUTING.md) for the route-to-test matrix, test layout conventions,
and coverage instructions.

Run the workspace test suite the same way CI does:

```sh
resources/scripts/run-tests-with-db.sh
```

For a quicker loop without MySQL (library unit tests and simulation goldens):

```sh
resources/scripts/run-fast-tests.sh
```

`run-tests-with-db.sh` resolves `ROBOMINER_DATABASE_URL` (existing database, local MySQL, or persistent
Docker) and runs `cargo test --workspace`. Without a usable database, DB integration tests skip
themselves; golden and unit tests still run.

Generate an LLVM coverage report (requires `[cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)`):

```sh
resources/scripts/run-coverage-with-db.sh
resources/scripts/run-coverage-with-db.sh --lcov --output-path lcov.info
ROBOMINER_COVERAGE_FAIL_UNDER_LINES=93 resources/scripts/run-coverage-with-db.sh
```

CI enforces the line-coverage floor via `ROBOMINER_COVERAGE_FAIL_UNDER_LINES` and uploads LCOV to
Codecov when configured.

Run the Rust formatter check:

```sh
cargo fmt --check
```

Run Clippy with warnings denied (same as CI):

```sh
cargo clippy --workspace -- -D warnings
```

Run a compile check without producing final binaries:

```sh
cargo check --workspace
```

GitHub Actions runs `cargo fmt --check`, Clippy, and `resources/scripts/run-tests-with-db.sh`
against MySQL 8. A separate coverage job uploads an LCOV artifact built with
`resources/scripts/run-coverage-with-db.sh`.

Fastest local workflow: reuse an existing MySQL instance or a persistent Docker
container instead of creating a fresh database every run:

```sh
resources/scripts/run-tests-with-db.sh
```

That helper:

1. Uses `ROBOMINER_DATABASE_URL` when it already points at an initialized database.
2. Otherwise reuses MySQL on `127.0.0.1:3306` when the RoboMiner schema is present.
3. Otherwise starts or reuses a persistent Docker container named
  `robominer-test-mysql` on port `3307` with volume `robominer-test-mysql-data`.

After the first Docker setup (~30–40 s), later runs typically add only about
0.1 s of database setup before the ~6 s integration test suite runs.

Manual setup:

```sh
ROBOMINER_DATABASE_URL=mysql://robominer:password@localhost/RoboMiner cargo test --workspace
```

To initialize or refresh schema manually (`createDatabase.sql` then
`gameData.sql`, then schema migrations):

```sh
resources/scripts/init-ci-database.sh
ROBOMINER_FORCE_DB_REINIT=1 resources/scripts/init-ci-database.sh
```

Apply pending schema migrations to an existing database (also auto-baselines a
schema that already matches `createDatabase.sql`):

```sh
resources/scripts/migrate-database.sh
# or:
cargo run -p robominer-engine -- migrate
cargo run -p robominer-engine -- migrate-status
cargo run -p robominer-engine -- migrate-status --check
```

`migrate-status --check` exits non-zero while migrations are pending. The web host
exposes loopback readiness at `GET /health` (database ping + migration currency).

Versioned SQL lives under `resources/database/migrations/` (`NNN_description.sql`).



## Run The Engine

`robominer-engine` is the Rust command-line replacement for the legacy native
engine. It accepts the database connection in three ways, in this order:

1. Pass `--database-url`.
2. Set `ROBOMINER_DATABASE_URL`.
3. Pass `--config`, or read `/etc/robominer/robominer.conf`, then
  `/etc/robominer/robominer-engine.conf`, then `robominer-engine.conf` beside
   the binary.

Example with an explicit database URL:

```sh
cargo run -p robominer-engine -- \
  --database-url mysql://robominer:password@localhost/RoboMiner \
  mining-queue-page-states --user-id 1
```

Example using the environment variable:

```sh
ROBOMINER_DATABASE_URL=mysql://robominer:password@localhost/RoboMiner \
  cargo run -p robominer-engine -- leaderboard-states --max-entries 10
```

Run the rally worker loop:

```sh
ROBOMINER_DATABASE_URL=mysql://robominer:password@localhost/RoboMiner \
  cargo run -p robominer-engine -- run-rallies
```

Show all available engine commands:

```sh
cargo run -p robominer-engine -- --help
```



## Run The Rust Web Host

`robominer-web` is the Rust web host for RoboMiner. It owns the application
routes, renders the remaining browser behavior from Rust source, and serves CSS
from `robominer-web/static`.

It handles `/help`, `/helpTutorial`, `/helpProgramTips`, `/helpRobotProgram`,
`/helpMechanics`, `/logoff`, `/leaderboard`, `/miningAreaOverview`,
`/activity`, `/miningQueue`, `/miningResults`, `/account`, `/achievements`,
`/editCode`, `/login`, `/robot`, and `/shop`.

Run it on the default address, `127.0.0.1:8080`:

```sh
cargo run -p robominer-web
```

Or with the shared legacy config file:

```sh
cargo run -p robominer-web -- --config /etc/robominer/robominer.conf
```

The web host accepts the database connection in this order:

1. Pass `--database-url`.
2. Set `ROBOMINER_DATABASE_URL`.
3. Pass `--config` or read `/etc/robominer/robominer.conf`, then
  `robominer-web.conf` beside the binary.

Override host, port, or static asset root:

```sh
HOST=0.0.0.0 PORT=8080 ROBOMINER_WEB_ROOT=robominer-web/static cargo run -p robominer-web
```

Set `ROBOMINER_DATABASE_URL` to enable database-backed pages such as
`/leaderboard`, `/miningAreaOverview`, `/activity`, `/miningQueue`,
`/miningResults`, `/account`, `/achievements`, `/editCode`, `/login`, `/robot`,
and `/shop`:

```sh
ROBOMINER_DATABASE_URL=mysql://robominer:password@localhost/RoboMiner cargo run -p robominer-web
```

Set `ROBOMINER_SESSION_SECRET` to sign login session cookies. When binding to
`127.0.0.1`, `localhost`, or `::1`, the web host allows an insecure development
default if no secret is configured. Any other bind address requires a secret and
the process exits on startup if one is missing. Use a long random value in any
shared or production deployment:

```sh
ROBOMINER_SESSION_SECRET="$(openssl rand -hex 32)" \
ROBOMINER_DATABASE_URL=mysql://robominer:password@localhost/RoboMiner \
cargo run -p robominer-web
```

Public self-registration is off by default. For local development, set
`ROBOMINER_ALLOW_SIGNUP=1` or `allowsignup 1` in the config file; otherwise create
users with `robominer-engine create-user`.

Logged-in users are identified by a signed `robominer_session` cookie minted at
login. The legacy plain `robominer_user_id` cookie is no longer accepted.

New users receive Argon2id password hashes. Existing `sha256:` hashes remain
valid until the user logs in successfully, at which point the hash is upgraded
automatically.

Install the engine and web host for production with:

```sh
deploy/systemd/install-robominer.sh --migrate --enable
```

Omit `--migrate` only if you will apply schema changes yourself afterward
(`robominer-engine migrate`). The install script prints a reminder when it
skips that step.

For HTTPS, put Caddy or nginx in front of the web host. See
`deploy/reverse-proxy/README.md` for example configs and
`deploy/INTERNET-HARDENING.md` for firewall, rate limits, and invite-only
signup (public registration is off by default; set `allowsignup 1` to open it).
Bind `robominer-web` to `127.0.0.1`, set `sessionsecret`, and enable
`securecookies 1` when users reach the site over HTTPS.

See `deploy/systemd/README.md` for the shared `/etc/robominer/robominer.conf`
format used by both `robominer-web` and `robominer-engine`.

Then open:

```text
http://127.0.0.1:8080/login
```



## License

RoboMiner is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
[http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
- MIT license ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

at your option.