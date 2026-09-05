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

- This Cloud VM installs a **host** MySQL **8.0** package; CI and the supported
  target use **MySQL 8.4** (see README / CONTRIBUTING). Prefer dialect features
  that work on 8.4; treat the host 8.0 as a local convenience.
- There is no systemd/auto-start in the VM. Start MySQL with
  `sudo service mysql start` at the beginning of a session (check with
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
- **InnoDB OS error 22** (`Invalid argument` on file `close` in
  `/var/log/mysql/error.log`): snapshot/overlay-backed datadir files can become
  unusable after a kernel or image change. Docker is often unavailable on these
  VMs, so recover host MySQL by reinitializing the datadir, then reloading
  schema:

  ```sh
  sudo service mysql stop || true
  sudo mv /var/lib/mysql "/var/lib/mysql.broken-$(date +%s)"
  sudo mkdir -p /var/lib/mysql /var/run/mysqld
  sudo chown mysql:mysql /var/lib/mysql /var/run/mysqld
  sudo mysqld --initialize-insecure --user=mysql --datadir=/var/lib/mysql
  sudo service mysql start
  sudo mysql -e "ALTER USER 'root'@'localhost' IDENTIFIED WITH caching_sha2_password BY 'root'; CREATE USER IF NOT EXISTS 'root'@'127.0.0.1' IDENTIFIED WITH caching_sha2_password BY 'root'; CREATE USER IF NOT EXISTS 'root'@'%' IDENTIFIED WITH caching_sha2_password BY 'root'; GRANT ALL PRIVILEGES ON *.* TO 'root'@'localhost' WITH GRANT OPTION; GRANT ALL PRIVILEGES ON *.* TO 'root'@'127.0.0.1' WITH GRANT OPTION; GRANT ALL PRIVILEGES ON *.* TO 'root'@'%' WITH GRANT OPTION; FLUSH PRIVILEGES;"
  resources/scripts/init-ci-database.sh
  mysqladmin ping -h127.0.0.1 -urobominer -ppassword
  ```

  Fresh files on the same overlay root work; only the stale snapshot datadir
  fails. Prefer Docker MySQL 8.4 via `ensure-test-mysql.sh` when `docker` is
  available.

### Running the web host

- Run: `ROBOMINER_DATABASE_URL=mysql://robominer:password@127.0.0.1:3306/RoboMiner cargo run -p robominer-web`
  (listens on `127.0.0.1:8080`; `GET /health` reports DB + migration readiness).
- Public self-registration is OFF by default. For local testing set
  `ROBOMINER_ALLOW_SIGNUP=1` to enable the signup form, or create users with
  `robominer-engine user create`.
- Binding to loopback allows an insecure dev session secret; any non-loopback
  bind requires `ROBOMINER_SESSION_SECRET` or the process exits at startup.

### Running the engine

- The engine reads the DB from `--database-url` or `ROBOMINER_DATABASE_URL`
  (in that order). Example:
  `ROBOMINER_DATABASE_URL=... cargo run -p robominer-engine -- leaderboard states --max-entries 10`.
  The background worker loop is `robominer-engine ... rally rallies`. Each persist
  cycle also claims finished mining runs into player wallets; sleep is
  `min(rally claim delay, wallet claim delay)`. Standalone:
  `robominer-engine ... mining claim-all --loop` (optional when a single
  `rally rallies --persist` worker is enough).
