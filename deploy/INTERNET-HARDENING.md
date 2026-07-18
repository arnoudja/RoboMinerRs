# Internet exposure hardening

Checklist for running RoboMiner on the public internet. The application is a
small game server, not a hardened SaaS platform — treat this as **minimum viable
production hygiene**.

## Architecture

```text
Internet
   │
   ▼
Reverse proxy (TLS :443, rate limits, HSTS)
   │
   ▼
robominer-web (127.0.0.1:8080; CSRF, rate limits, security headers)
   │
   ├──► MySQL (private network / localhost only)
   └──► robominer-engine (no HTTP, DB access only)
```

Never expose `8080`, `3306`, or the engine process directly to the internet.

## 1. Application config

Edit `/etc/robominer/robominer.conf`:

```text
host 127.0.0.1
port 8080
webroot /opt/robominer/static
sessionsecret <run: openssl rand -hex 32>
securecookies 1
allowsignup 0
trustproxy 1
```

| Key | Purpose |
| --- | --- |
| `host 127.0.0.1` | Web binds loopback only; proxy handles public traffic |
| `sessionsecret` | Signs session cookies; required if not on localhost |
| `securecookies 1` | `Secure` flag on cookies (use with HTTPS) |
| `allowsignup 0` | Public self-registration off (default when unset); set `1` to open signup |
| `trustproxy 1` | Trust `X-Forwarded-For` / `X-Real-Ip` for login rate limits and auth logs |

Environment overrides: `ROBOMINER_SESSION_SECRET`, `ROBOMINER_SECURE_COOKIES=1`,
`ROBOMINER_ALLOW_SIGNUP=1`, `ROBOMINER_TRUST_PROXY=1`.

Create accounts while signup is disabled:

```bash
ROBOMINER_DATABASE_URL='mysql://…' \
  cargo run -p robominer-engine -- create-user \
  --username alice --email alice@example.com --password '…'
```

## 2. Firewall

Allow HTTPS (and HTTP only if you redirect to HTTPS). Block everything else.

```bash
sudo ufw default deny incoming
sudo ufw allow OpenSSH
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw enable
```

Confirm the app is not reachable on 8080 from outside:

```bash
ss -ltnp | grep 8080    # should show 127.0.0.1:8080
```

## 3. Reverse proxy

Use the hardened examples in `deploy/reverse-proxy/`:

- **Caddy** — automatic HTTPS + security headers (`Caddyfile`)
- **nginx** — login rate limiting + security headers (`nginx.conf`)

After editing, reload the proxy and verify:

```bash
curl -I https://robominer.example.com/login
```

Expect `Strict-Transport-Security` (after HTTPS is working) and a `200`/`302` from
the app.

## 4. Login rate limiting

Application-level rate limits apply to `POST /login` (login and signup) by client
IP and license plate (`login_name` / signup username), returning HTTP `429` when
exceeded. Prefer keeping the reverse-proxy limit as defense in depth.

`deploy/reverse-proxy/nginx.conf` also includes a `limit_req` zone on
`POST /login` (5 requests/minute per IP, burst 3). Tune for your player base.

Caddy does not ship HTTP rate limiting in the stock binary. Options:

- Rely on the **app-level** limiter (always on)
- Use **fail2ban** (below) against app stderr or proxy logs
- Build Caddy with a rate-limit plugin

## 5. fail2ban (optional)

Example filters for failed logins live in `deploy/fail2ban/`. Prefer matching the
stable app log lines emitted on stderr / systemd journal:

```text
auth_failure ip=<ip> login_name=<name> result=invalid_credentials
auth_failure ip=<ip> login_name=<name> result=rate_limited
```

```bash
sudo cp deploy/fail2ban/robominer-login.conf /etc/fail2ban/filter.d/
sudo cp deploy/fail2ban/robominer-jail.conf /etc/fail2ban/jail.d/robominer.local
# Point journalmatch / logpath at robominer-web (see jail comments)
sudo systemctl reload fail2ban
sudo fail2ban-client status robominer-login
```

## 6. MySQL

- Bind MySQL to `127.0.0.1` or a private VPC address only
- Dedicated `robominer` DB user with privileges for the `RoboMiner` schema
  (including `CREATE`/`ALTER` for schema migrations)
- Strong password; store only in `/etc/robominer/robominer.conf` (`chmod 0640`)
- Apply pending schema migrations after install or deploy (the systemd install
  script does not migrate unless you pass `--migrate`):

```bash
sudo /opt/robominer/bin/robominer-engine --config /etc/robominer/robominer.conf migrate
# from a checkout: resources/scripts/migrate-database.sh
# or: cargo run -p robominer-engine -- migrate
```

## 7. Verification

| Check | Command / action |
| --- | --- |
| TLS works | Browser shows padlock on `https://your-host/login` |
| Secure cookie | DevTools → `robominer_session` has `Secure`, `HttpOnly`, `SameSite=Lax` |
| Loopback bind | `ss -ltnp \| grep 8080` → `127.0.0.1` |
| Signup off (default) | `/login?signup=1` shows no sign-up tab unless `allowsignup 1` |
| Security headers | `curl -I /login` includes `X-Frame-Options`, `X-Content-Type-Options`, `Referrer-Policy` |
| Engine private | No listening HTTP port for `robominer-engine` |
| App rate limit | Rapid `/login` POSTs return `429` |
| CSRF | Authenticated and login/signup POST forms include `csrfToken` |
| Static cache | `curl -I /css/robominer.css` shows `Cache-Control` and `ETag` |

## 8. Known gaps (app-level)

These are **not** fully solved. Accept the risk or plan follow-up work:

| Gap | Mitigation today |
| --- | --- |
| No HSTS in app | Proxy adds `Strict-Transport-Security` (app serves plain HTTP on loopback) |

### Already covered in-app

| Control | Notes |
| --- | --- |
| CSRF on authenticated mutations | Hidden `csrfToken` bound to session nonce; nonce rotates after each successful POST |
| CSRF on login/signup | Double-submit cookie `robominer_csrf` |
| Body size limit | 1 MiB → HTTP 413 |
| Request timeouts | 30s |
| POST-only form mutations | GET cannot drive shop/queue/account writes |
| App login rate limit | Sliding window by IP and login name → 429; empty keys pruned |
| Account password rate limit | `/account` POSTs limited by IP and `user:{id}` before Argon2 verify |
| Client IP | Peer address by default; `trustproxy 1` enables proxy headers |
| Failed-login logging | Stable `auth_failure …` lines for fail2ban |
| Axum concurrency cap | In-flight request semaphore |
| Schema migrations | `SchemaMigration` + `migrate` / `migrate-database.sh` |
| Signup off by default | Set `allowsignup 1` / `ROBOMINER_ALLOW_SIGNUP=1` to open registration |
| Security headers | `X-Content-Type-Options`, `X-Frame-Options`, `Referrer-Policy` on all responses |
| Email validation | Local + domain with TLD on signup / account update |

## Related docs

- [deploy/reverse-proxy/README.md](reverse-proxy/README.md) — TLS setup
- [deploy/systemd/README.md](systemd/README.md) — service install
- [README.md](../README.md) — build, test, run
