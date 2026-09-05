# Security Policy

## Supported versions

Security fixes are applied to the latest `master` branch and to the most recent
tagged release when a release line is still in use.

## Reporting a vulnerability

Please **do not** open a public GitHub issue for security vulnerabilities.

Use GitHub’s private vulnerability reporting for this repository:

https://github.com/arnoudja/RoboMinerRs/security/advisories/new

Include:

- A description of the issue and its impact
- Steps to reproduce (PoC if available)
- Affected versions / commit SHAs when known

We aim to acknowledge reports within 7 days and to keep you informed of the
fix or disclosure timeline. Please give us a reasonable window to ship a
fix before any public disclosure.

## Scope notes

RoboMiner is a game server with intentional public social data (for example
robot stats by id and achievement overviews by user). Auth, CSRF, session, and
mutation authorization are in scope; those public read surfaces are not treated
as IDOR by design. See `deploy/INTERNET-HARDENING.md` for production hardening.

## Dependency advisories

CI runs `cargo audit` and `cargo deny` on every pull request. Advisory ignores,
if any are ever required, must be recorded in `.cargo/audit.toml` with a reason
and tracked for removal.
