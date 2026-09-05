# Legacy compatibility sunset plan

RoboMiner carries intentional compatibility layers from earlier engine and
URL conventions. Each surface below is covered by tests; remove only after the
sunset criteria are met.

## PascalCase URL redirects

| Item | Detail |
| --- | --- |
| Status | **Removed** — canonical camelCase paths only (`/shop`, `/miningQueue`, …) |
| Notes | Redirect middleware, PascalCase route aliases, and `/Health` alias deleted after sunset (zero production use) |

## Session tokens without `session_version`

| Item | Detail |
| --- | --- |
| Status | **Removed** — session cookies must embed `session_version` |
| Notes | Unversioned payloads (formerly defaulted to version `0`) are rejected after the remember-me TTL sunset window; explicit `0` remains valid for users whose DB `sessionVersion` is still `0` |

## Legacy rally `resultData` (JS executable)

| Item | Detail |
| --- | --- |
| Location | `robominer-web/src/rally_pages/view/payload.rs` |
| Behavior | Old rows are refused for replay; new runs store versioned JSON (`{"v":2,…}`) |
| Sunset | Document-only unless historical replay is required; optional DB migration to purge |
| Risk | Low — replay already unavailable for legacy rows |

## Legacy ore slot ordering

| Item | Detail |
| --- | --- |
| Location | `robominer-domain/src/simulation/legacy.rs` |
| Behavior | Ore animation slots follow descending ore id order from the original engine |
| Sunset | **Do not change without balance review** — golden tests lock behavior |
| Risk | High — affects rally scores and player expectations |

## `sha256:` password hashes

| Item | Detail |
| --- | --- |
| Status | **Removed** — Argon2id only |
| Notes | All accounts were migrated; login no longer accepts or upgrades `sha256:` rows |

## `/etc/robominer/robominer.conf`

| Item | Detail |
| --- | --- |
| Location | `robominer-web/src/startup.rs`, `robominer-db/src/config.rs` |
| Behavior | Legacy key/value file merged with environment variables |
| Sunset | Prefer `EnvironmentFile` / env vars (see `deploy/systemd/robominer.env.example`); deprecate file format in a major release |
| Risk | Medium for existing Pi/systemd installs — provide migration notes in `deploy/systemd/` |

## `__Host-` cookie prefix (not yet)

| Item | Detail |
| --- | --- |
| Location | `robominer-web/src/session/`, `robominer-web/src/csrf/` |
| Current | Cookies `robominer_session`, `robominer_username`, `robominer_csrf` use `Path=/`, no `Domain`; `Secure` only when `securecookies` is on |
| Sunset | Rename to `__Host-*` only when every real deploy forces Secure (HTTPS end-to-end). Checklist before flip: (1) `securecookies 1` required in all production env files / docs, (2) no plain-HTTP LAN deploys that still need cookies, (3) clear old cookie names on login/logoff, (4) proxy TLS terminates correctly |
| Risk | High for LAN/HTTP installs — browsers reject `__Host-` without Secure |

## Recommended order

1. ~~Tighten open-redirect and session rules~~ **Done** (`returnTo` whitelist, CSRF logoff, Secure+trustproxy, session secret floor, TTL cap)
2. ~~Trustproxy missing Real-IP footgun~~ **Done** (dedicated `proxy-missing-real-ip` key + error log)
3. ~~Mutation IP co-limit~~ **Done** (60 POSTs / 60s per client IP alongside user+action budgets)
4. ~~Remove PascalCase redirects~~ **Done** (canonical camelCase only)
5. ~~Bump minimum session version after TTL window~~ **Done** (unversioned tokens rejected; remember-me max age is 30 days)
6. `__Host-` cookies only after Secure is mandatory on every deploy path
7. Leave ore-ordering and conf-file paths until a deliberate migration release
8. Distributed / shared rate-limit store — only when running multiple web replicas (proxy limits remain the shared control plane today)
9. App-level HSTS — leave to the reverse proxy; do not emit HSTS on loopback HTTP binds
10. Mass JS `no-var` — continue page-scoped only (`common/`, `mining_queue/**`, shop/mining_results/robot pages done); defer `rally_animation` / `edit_code` until deliberately scheduled
