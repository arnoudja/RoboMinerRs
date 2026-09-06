# Environment configuration reference

RoboMiner is configured with environment variables (for example via
`/etc/robominer/robominer.env` and systemd `EnvironmentFile=`). The legacy
key/value `robominer.conf` format has been removed.

This table is the single inventory of supported knobs. Deploy examples live in
[`deploy/systemd/robominer.env.example`](../deploy/systemd/robominer.env.example)
and hardening notes in [`deploy/INTERNET-HARDENING.md`](../deploy/INTERNET-HARDENING.md).

## Database

| Variable | Used by | Default | Purpose |
| --- | --- | --- | --- |
| `ROBOMINER_DATABASE_URL` | web, engine, scripts | _(required)_ | MySQL URL (`mysql://user:pass@host:3306/RoboMiner`) |
| `ROBOMINER_DB_MAX_CONNECTIONS` | web, engine | implementation default | sqlx pool size |
| `ROBOMINER_ALLOW_INSECURE_MYSQL` | db connect | off | Allow non-TLS MySQL to a remote host (loopback is fine without this) |

CLI override: `--database-url` on engine/web when supported.

## Web server

| Variable | Default | Purpose |
| --- | --- | --- |
| `HOST` | `127.0.0.1` | Bind address |
| `PORT` | `8080` | Bind port |
| `ROBOMINER_WEB_ROOT` | crate `static/` | Static asset root |
| `ROBOMINER_SESSION_SECRET` | _(required off loopback)_ | HMAC session secret (≥32 chars) |
| `ROBOMINER_ALLOW_INSECURE_DEV_SECRET` | off | Permit built-in insecure secret on loopback only |
| `ROBOMINER_SESSION_TTL_SECS` | _(unset)_ | Session TTL in seconds (wins over hours when set) |
| `ROBOMINER_SESSION_TTL_HOURS` | _(implementation default)_ | Session TTL in hours |
| `ROBOMINER_SECURE_COOKIES` | off on loopback | Set `Secure` on cookies; **required** for non-loopback binds and when trusting a proxy |
| `ROBOMINER_ALLOW_SIGNUP` | off | Enable public self-registration (signup uses PoW + stronger password rules) |
| `ROBOMINER_TRUST_PROXY` | off | Trust `X-Real-Ip` only; requires loopback bind + secure cookies |

## Logging / tests

| Variable | Used by | Purpose |
| --- | --- | --- |
| `ROBOMINER_LOG_FORMAT` | web, engine | text | Set to `json` for structured JSON logs on stderr |
| `ROBOMINER_USER_PASSWORD` | engine user commands | _(unset)_ | Password for `user create` / verify without argv |
| `RUST_LOG` | web, engine | `tracing` filter (default is quiet/`warn`-oriented) |
| `SQLX_OFFLINE` | CI / local offline builds | Use committed `.sqlx/` metadata |
| `ROBOMINER_COVERAGE_FAIL_UNDER_LINES` | CI coverage | Line coverage floor |
| `ROBOMINER_FORCE_DB_REINIT` | `init-ci-database.sh` | Force schema reload |

## Boolean parsing

Truthy values for flag env vars: `1`, `true`, `TRUE`, `yes`, `YES`, `on`, `ON`
(with optional surrounding whitespace). Anything else is false when the variable
is set; unset usually means the documented default.
