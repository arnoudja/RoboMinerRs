# AGENTS.md

## Cursor Cloud specific instructions

RoboMiner is a Rust workspace: a web game backed by MySQL. The two runnable
binaries are `robominer-web` (the HTTP host) and `robominer-engine` (a CLI +
rally/mining worker). Standard build/test/lint/run commands live in `README.md`
and `CONTRIBUTING.md` — use those; only the non-obvious environment caveats are
noted here.

### Toolchain

- The workspace uses Rust edition 2024, so it needs `rustc >= 1.85`. The default
  `rustup` toolchain is set to `stable` (currently 1.97). `cargo-nextest` is
  installed (the test scripts use it via the `ci`/`fast` profiles and otherwise
  fall back to `cargo test`).

### MySQL (must be started manually each session)

- MySQL 8.0 is installed but there is no systemd/auto-start in the VM. Start it
  with `sudo service mysql start` at the beginning of a session (check with
  `sudo mysqladmin ping`).
- Credentials configured in this environment: root over TCP is `root`/`root`
  (host `127.0.0.1`/`%`), and the app user is `robominer`/`password`. The
  `RoboMiner` database is already initialized (schema + `gameData.sql` seed +
  migrations) and persists in the MySQL data directory.
- The canonical dev/test connection string is:
  `mysql://robominer:password@127.0.0.1:3306/RoboMiner`
- `resources/scripts/run-tests-with-db.sh` auto-resolves the database: it reuses
  local MySQL on `127.0.0.1:3306` when the schema is present, so Docker is not
  required here. If a fresh DB is ever needed, run
  `resources/scripts/init-ci-database.sh` (or with `ROBOMINER_FORCE_DB_REINIT=1`).

### Running the web host

- Run: `ROBOMINER_DATABASE_URL=mysql://robominer:password@127.0.0.1:3306/RoboMiner cargo run -p robominer-web`
  (listens on `127.0.0.1:8080`; `GET /health` reports DB + migration readiness).
- Public self-registration is OFF by default. For local testing set
  `ROBOMINER_ALLOW_SIGNUP=1` to enable the signup form, or create users with
  `robominer-engine user create`.
- Binding to loopback allows an insecure dev session secret; any non-loopback
  bind requires `ROBOMINER_SESSION_SECRET` or the process exits at startup.

### Running the engine

- The engine reads the DB from `--database-url`, `ROBOMINER_DATABASE_URL`, or a
  config file (in that order). Example:
  `ROBOMINER_DATABASE_URL=... cargo run -p robominer-engine -- leaderboard states --max-entries 10`.
  The background worker loop is `robominer-engine ... rally rallies`. Each persist
  cycle also claims finished mining runs into player wallets; sleep is
  `min(rally claim delay, wallet claim delay)`. Standalone:
  `robominer-engine ... mining claim-all --loop` (optional when a single
  `rally rallies --persist` worker is enough).
