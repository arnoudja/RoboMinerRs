# RoboMiner systemd deployment

This directory contains systemd unit templates and install scripts for the Rust
engine and web host.

Both services read the shared config file `/etc/robominer/robominer.conf`.
The engine still accepts the legacy path `/etc/robominer/robominer-engine.conf`
when the shared file is absent.

## Install

Automated install for both the engine and web host:

```bash
deploy/systemd/install-robominer.sh
```

Install only one component:

```bash
deploy/systemd/install-engine.sh
deploy/systemd/install-web.sh
```

Set `ROBOMINER_DB_SERVER`, `ROBOMINER_DB_USER`, `ROBOMINER_DB_PASSWORD`, and
`ROBOMINER_DB_DATABASE` to create `/etc/robominer/robominer.conf` automatically.
Set `ROBOMINER_SESSION_SECRET` before install to add a web session signing key.
Pass `--enable` to install and start the systemd services.
Run `deploy/systemd/install-robominer.sh --help` for all options.

## Shared config

Create `/etc/robominer/robominer.conf` with database keys:

```text
dbserver <host>
dbuser <user>
dbpassword <password>
dbdatabase <database>
```

Optional web keys in the same file:

```text
host 127.0.0.1
port 8080
webroot /opt/robominer/static
sessionsecret <random secret>
securecookies 1
allowsignup 0
trustproxy 1
```

`allowsignup 0` disables public sign-up (invite-only). See
[INTERNET-HARDENING.md](../INTERNET-HARDENING.md) for the full public-deployment
checklist.

Environment variables still override config values for local development:

- `ROBOMINER_DATABASE_URL` or `--database-url`
- `HOST`, `PORT`, `ROBOMINER_WEB_ROOT`, `ROBOMINER_SESSION_SECRET`, `ROBOMINER_SECURE_COOKIES`, `ROBOMINER_ALLOW_SIGNUP`, `ROBOMINER_TRUST_PROXY`

## Install the engine

`deploy/systemd/install-engine.sh` is a wrapper around `install-robominer.sh
--engine-only`.

Manual install steps:

```bash
sudo install -D -m 0644 deploy/systemd/robominer-engine.sysusers \
  /etc/sysusers.d/robominer-engine.conf
sudo systemd-sysusers /etc/sysusers.d/robominer-engine.conf
sudo install -d -o robominer -g robominer -m 0750 /opt/robominer
cargo build --release -p robominer-engine
sudo install -D -m 0755 target/release/robominer-engine /opt/robominer/bin/robominer-engine
sudo install -d -o root -g robominer -m 0750 /etc/robominer
sudoedit /etc/robominer/robominer.conf
sudo chown root:robominer /etc/robominer/robominer.conf
sudo chmod 0640 /etc/robominer/robominer.conf
sudo install -D -m 0644 deploy/systemd/robominer-engine.service \
  /etc/systemd/system/robominer-engine.service
sudo systemctl daemon-reload
sudo systemctl enable --now robominer-engine.service
```

## Install the web host

`deploy/systemd/install-web.sh` is a wrapper around `install-robominer.sh
--web-only`.

Manual install steps:

```bash
cargo build --release -p robominer-web
sudo install -D -m 0755 target/release/robominer-web /opt/robominer/bin/robominer-web
sudo install -d -o robominer -g robominer -m 0755 /opt/robominer/static/css
sudo install -m 0644 robominer-web/static/css/robominer.css \
  /opt/robominer/static/css/robominer.css
sudo install -D -m 0644 deploy/systemd/robominer-web.service \
  /etc/systemd/system/robominer-web.service
sudo systemctl daemon-reload
sudo systemctl enable --now robominer-web.service
```

The web unit runs:

```bash
/opt/robominer/bin/robominer-web --config /etc/robominer/robominer.conf
```

Put a reverse proxy in front of the web host for TLS on production systems.
See `deploy/reverse-proxy/README.md` for Caddy and nginx examples. Bind the web
host to `127.0.0.1` and set `securecookies 1` when serving users over HTTPS.

## Engine operation

The engine unit runs:

```bash
/opt/robominer/bin/robominer-engine \
  --config /etc/robominer/robominer.conf \
  run-rallies --loop --sleep-seconds 5 --persist
```

Logs go to journald:

```bash
journalctl -u robominer-engine.service -f
journalctl -u robominer-web.service -f
```

Graceful engine shutdown:

```bash
sudo systemctl stop robominer-engine.service
```

The unit sends `SIGINT`, which lets `robominer-engine` finish the current poll
cycle or transaction and exit before starting the next cycle.

## Deployment runbook

Use this runbook when deploying the Rust rally service.

### 1. Preflight

Confirm the Rust binary can connect to the production database:

```bash
/opt/robominer/bin/robominer-engine \
  --config /etc/robominer/robominer.conf \
  run-rallies --once
```

This is a dry run. It should print how many mining areas were processed and
which ready rallies would run, without writing results.

### 2. Manual persisted trial

Run one persisted Rust pass manually before enabling the long-running service:

```bash
/opt/robominer/bin/robominer-engine \
  --config /etc/robominer/robominer.conf \
  run-rallies --once --persist
```

Verify recent rows in `MiningQueue`, `RallyResult`, `MiningOreResult`,
`RobotActionsDone`, and `RobotMiningAreaScore`, and open at least one rally in
the web UI to confirm the generated animation payload renders.

### 3. Start Rust services

Start and monitor the services:

```bash
sudo systemctl enable --now robominer-engine.service
sudo systemctl enable --now robominer-web.service
journalctl -u robominer-engine.service -f
```

Watch for repeated errors, unexpectedly high skipped counts, or missing
`Persisted rally result` messages when the mining queue has ready work.

### 4. Rollback

To roll back this deployment:

```bash
sudo systemctl stop robominer-engine.service robominer-web.service
sudo systemctl disable robominer-engine.service robominer-web.service
```

Restore the previously deployed binaries and config, then restart the services.
Confirm only one rally worker is active after rollback.

## Notes

- Adjust `User`, `Group`, `WorkingDirectory`, `ExecStart`, and database service
  names to match the deployment host.
