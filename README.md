# RoboMiner

RoboMiner is an online programming game. Improve the program for your robot to mine more efficiently.

## Prerequisites

- Rust toolchain with Cargo.
- **MySQL 8.4** (CI and supported target). MariaDB may work but is best-effort /
  untested against the current schema and SQL dialect.

The database scripts are kept under `resources/database/`:

- `createDatabase.sql` — database schema
- `gameData.sql` — seed data (ores, parts, areas, achievements)

Further docs: [CONTRIBUTING.md](CONTRIBUTING.md) (tests, coverage, **db vs domain
boundary**), [docs/architecture.md](docs/architecture.md) (crate layers and request
flows), [docs/crate-map.md](docs/crate-map.md) (module ownership),
[ACHIEVEMENTS.md](ACHIEVEMENTS.md) (progression and claim flow),
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

Build installable Debian packages for this machine and for 64-bit Raspberry Pi
(requires [`cargo-deb`](https://crates.io/crates/cargo-deb) and `dpkg-deb`;
Pi builds also need `gcc-aarch64-linux-gnu`):

```sh
cargo install cargo-deb --locked   # once
resources/scripts/build-deb.sh
```

Install on the target host (example for a native build artifact):

```sh
sudo apt install ./target/debian/robominer_*.deb
```

On a Pi, copy the `aarch64` `.deb` from the build output and install the same way.
If `/etc/robominer/robominer.env` already exists, the package `postinst` runs
`migrate apply`, applies `gameData.sql`, and starts the systemd units.

The main binaries are:

- `target/debug/robominer-engine`
- `target/debug/robominer-web`
- `target/release/robominer-engine`
- `target/release/robominer-web`



## Test And Check

See [CONTRIBUTING.md](CONTRIBUTING.md) for the full test workflow, route-to-test
matrix, coverage floor (93), golden fixtures, git hooks, and crate-boundary rules.

```sh
resources/scripts/run-tests-with-db.sh   # same entry point as CI (MySQL 8.4)
resources/scripts/run-fast-tests.sh      # no database
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo check --workspace
```

Enable the repo pre-commit hook (rustfmt + Clippy when `.rs` files are staged):

```sh
git config core.hooksPath .githooks
```

## Database

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
cargo run -p robominer-engine -- migrate apply
cargo run -p robominer-engine -- migrate status
cargo run -p robominer-engine -- migrate status --check
```

`migrate status --check` exits non-zero while migrations are pending. The web host
exposes loopback readiness at `GET /health` (database ping + migration currency).

Versioned SQL lives under `resources/database/migrations/` (`NNN_description.sql`).
`run-tests-with-db.sh` resolves MySQL via an existing URL, local schema, or a
persistent Docker container—details in [CONTRIBUTING.md](CONTRIBUTING.md).



## Run The Engine

`robominer-engine` is the Rust command-line replacement for the legacy native
engine. It accepts the database connection in this order:

1. Pass `--database-url`.
2. Set `ROBOMINER_DATABASE_URL` (for example via `/etc/robominer/robominer.env`).

Example with an explicit database URL:

```sh
cargo run -p robominer-engine -- \
  --database-url mysql://robominer:password@localhost/RoboMiner \
  mining queue-page-states --user-id 1
```

Example using the environment variable:

```sh
ROBOMINER_DATABASE_URL=mysql://robominer:password@localhost/RoboMiner \
  cargo run -p robominer-engine -- leaderboard states --max-entries 10
```

Run the rally worker loop (`--persist` also credits finished mining runs into
player wallets after each poll cycle). Adaptive sleep uses
`min(next rally claim, next wallet claim)` so wallet credits are not delayed
when mining finishes but no rally is due. For single-worker deploys,
`rally rallies --persist` covers wallet claims without a separate
`mining claim-all --loop`:

```sh
ROBOMINER_DATABASE_URL=mysql://robominer:password@localhost/RoboMiner \
  cargo run -p robominer-engine -- rally rallies --loop --sleep-seconds 5 --persist
```

Show all available engine commands:

```sh
cargo run -p robominer-engine -- --help
```

## Offline tooling

`robominer-optimize` is an offline genetic-programming optimizer for experimenting
with robot programs against the live database schema. It is not part of the production
web/engine path:

```sh
cargo run -p robominer-optimize -- --help
```



## Run The Rust Web Host

`robominer-web` is the Rust web host for RoboMiner. It owns the application
routes, renders the remaining browser behavior from Rust source, and serves CSS
from `robominer-web/static`.

It handles `/help`, `/helpTutorial`, `/helpProgramTips`, `/helpRobotProgram`,
`/helpMechanics`, `/logoff`, `/leaderboard`, `/miningAreaOverview`,
`/activity`, `/miningQueue`, `/miningResults`, `/account`, `/achievements`,
`/editCode`, `/login`, `/robot`, `/robotStats`, and `/shop`.

Run it on the default address, `127.0.0.1:8080`:

```sh
cargo run -p robominer-web
```

Or with a systemd-style env file already exported:

```sh
set -a && source /etc/robominer/robominer.env && set +a
cargo run -p robominer-web
```

The web host accepts the database connection in this order:

1. Pass `--database-url`.
2. Set `ROBOMINER_DATABASE_URL`.

Override host, port, or static asset root:

```sh
HOST=0.0.0.0 PORT=8080 ROBOMINER_WEB_ROOT=robominer-web/static cargo run -p robominer-web
```

Set `ROBOMINER_DATABASE_URL` to enable database-backed pages such as
`/leaderboard`, `/miningAreaOverview`, `/activity`, `/miningQueue`,
`/miningResults`, `/account`, `/achievements`, `/editCode`, `/login`, `/robot`,
`/robotStats`, and `/shop`:

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
`ROBOMINER_ALLOW_SIGNUP=1`; otherwise create
users with `robominer-engine user create`.

Logged-in users are identified by a signed session cookie minted at login
(`__Host-robominer_session` when Secure cookies are on; `robominer_session` for
local loopback HTTP). The legacy plain `robominer_user_id` cookie is no longer
accepted.

New users receive Argon2id password hashes. Legacy `sha256:` password hashes are
no longer accepted.

Logged-off sessions are invalidated server-side: `POST /logoff` bumps
`User.sessionVersion` so stolen cookies stop working. The account page also
offers **Log out all devices** (bumps the version and reissues the current
cookie).

Install the engine and web host for production with:

```sh
deploy/systemd/install-robominer.sh --migrate --enable
```

Or install from a Debian package built with `resources/scripts/build-deb.sh`:

```sh
sudo apt install ./target/debian/robominer_*.deb
```

Omit `--migrate` only if you will apply schema changes yourself afterward
(`robominer-engine migrate apply`). The install script prints a reminder when it
skips that step. The `.deb` `postinst` migrates and reloads `gameData.sql`
automatically when `/etc/robominer/robominer.env` is already present.

For HTTPS, put Caddy or nginx in front of the web host. See
`deploy/reverse-proxy/README.md` for example configs and
`deploy/INTERNET-HARDENING.md` for firewall, rate limits, and invite-only
signup (public registration is off by default; set `allowsignup 1` to open it).
Bind `robominer-web` to `127.0.0.1`, set `sessionsecret`, and enable
`securecookies 1` when users reach the site over HTTPS.

See `deploy/systemd/README.md` for `/etc/robominer/robominer.env`.
Environment variables are listed in [`docs/configuration.md`](docs/configuration.md).

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