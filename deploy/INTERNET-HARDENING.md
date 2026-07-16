# Internet exposure hardening

Checklist for running RoboMiner on the public internet. The application is a
small game server, not a hardened SaaS platform — treat this as **minimum viable
production hygiene**.

## Architecture

```text
Internet
   │
   ▼
Reverse proxy (TLS :443, rate limits, security headers)
   │
   ▼
robominer-web (127.0.0.1:8080 only)
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
```

| Key | Purpose |
| --- | --- |
| `host 127.0.0.1` | Web binds loopback only; proxy handles public traffic |
| `sessionsecret` | Signs session cookies; required if not on localhost |
| `securecookies 1` | `Secure` flag on cookies (use with HTTPS) |
| `allowsignup 0` | Disable public self-registration (invite-only) |

Environment overrides: `ROBOMINER_SESSION_SECRET`, `ROBOMINER_SECURE_COOKIES=1`,
`ROBOMINER_ALLOW_SIGNUP=0`.

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
- Apply pending schema migrations after deploy:

```bash
resources/scripts/migrate-database.sh
# or: cargo run -p robominer-engine -- migrate
```

## 7. Verification

| Check | Command / action |
| --- | --- |
| TLS works | Browser shows padlock on `https://your-host/login` |
| Secure cookie | DevTools → `robominer_session` has `Secure`, `HttpOnly`, `SameSite=Lax` |
| Loopback bind | `ss -ltnp \| grep 8080` → `127.0.0.1` |
| Signup off | `/login?signup=1` shows no sign-up tab when `allowsignup 0` |
| Engine private | No listening HTTP port for `robominer-engine` |
| App rate limit | Rapid `/login` POSTs return `429` |
| CSRF | Authenticated and login/signup POST forms include `csrfToken` |
| Static cache | `curl -I /css/robominer.css` shows `Cache-Control` and `ETag` |

## 8. Known gaps (app-level)

These are **not** fully solved. Accept the risk or plan follow-up work:

| Gap | Mitigation today |
| --- | --- |
| Open signup (default) | Set `allowsignup 0` for invite-only |
| Weak email validation | Manual account creation via engine CLI |
| No security headers in app | Proxy adds HSTS, `X-Frame-Options`, etc. |
| Auth CSRF token stable per user | Session-bound cookie + form for login; authenticated tokens HMAC’d to user id — rotate session secret on compromise |

### Already covered in-app

| Control | Notes |
| --- | --- |
| CSRF on authenticated mutations | Hidden `csrfToken` + HMAC, rejected with 403 |
| CSRF on login/signup | Double-submit cookie `robominer_csrf` |
| Body size limit | 1 MiB → HTTP 413 |
| Request timeouts | 30s |
| POST-only form mutations | GET cannot drive shop/queue/account writes |
| App login rate limit | Sliding window by IP and login name → 429 |
| Failed-login logging | Stable `auth_failure …` lines for fail2ban |
| Axum concurrency cap | In-flight request semaphore |
| Schema migrations | `SchemaMigration` + `migrate` / `migrate-database.sh` |

## Related docs

- [deploy/reverse-proxy/README.md](reverse-proxy/README.md) — TLS setup
- [deploy/systemd/README.md](systemd/README.md) — service install
- [README.md](../README.md) — build, test, run
