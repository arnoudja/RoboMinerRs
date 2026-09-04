# Legacy compatibility sunset plan

RoboMiner carries intentional compatibility layers from earlier engine and
URL conventions. Each surface below is covered by tests; remove only after the
sunset criteria are met.

## PascalCase URL redirects

| Item | Detail |
| --- | --- |
| Location | `robominer-web/src/router/` (`canonical_path_redirect`) |
| Behavior | GET/HEAD `/Shop` → `/shop`, etc.; POST stays on legacy path for CSRF |
| Sunset | Log redirect hits in production; remove redirects after zero use for 3+ months |
| Risk | Old bookmarks and external links break if removed prematurely |

## Session tokens without `session_version`

| Item | Detail |
| --- | --- |
| Location | `robominer-web/src/session/mod.rs` |
| Behavior | Tokens missing version field are treated as version `0` |
| Sunset | After max session TTL, require re-login for all legacy tokens; then reject version `0` |
| Risk | Active players with long-lived “remember me” cookies would be logged out |

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

## Recommended order

1. ~~Tighten open-redirect and session rules~~ **Done** (`returnTo` whitelist, CSRF logoff, Secure+trustproxy, session secret floor, TTL cap)
2. Remove PascalCase redirects when analytics show zero use
3. Bump minimum session version after TTL window (remember-me max age is 30 days)
4. Leave ore-ordering and conf-file paths until a deliberate migration release
