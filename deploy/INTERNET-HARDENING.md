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

## 4. Login rate limiting (nginx)

`deploy/reverse-proxy/nginx.conf` includes a `limit_req` zone on `POST /login`
(5 requests/minute per IP, burst 3). Tune for your player base.

Caddy does not ship HTTP rate limiting in the stock binary. Options:

- Use **fail2ban** (below) against Caddy or nginx logs
- Build Caddy with a rate-limit plugin
- Prefer **nginx** if you want built-in `limit_req`

## 5. fail2ban (optional)

Example filters for repeated failed logins live in `deploy/fail2ban/`.

```bash
sudo cp deploy/fail2ban/robominer-login.conf /etc/fail2ban/filter.d/
sudo cp deploy/fail2ban/robominer-jail.conf /etc/fail2ban/jail.d/robominer.local
sudo systemctl reload fail2ban
sudo fail2ban-client status robominer-login
```

Adjust `logpath` in the jail to match your proxy access log (nginx or Caddy).

## 6. MySQL

- Bind MySQL to `127.0.0.1` or a private VPC address only
- Dedicated `robominer` DB user with least privilege on the `RoboMiner` database
- Strong password; store only in `/etc/robominer/robominer.conf` (`chmod 0640`)

## 7. Verification

| Check | Command / action |
| --- | --- |
| TLS works | Browser shows padlock on `https://your-host/login` |
| Secure cookie | DevTools → `robominer_session` has `Secure`, `HttpOnly`, `SameSite=Lax` |
| Loopback bind | `ss -ltnp \| grep 8080` → `127.0.0.1` |
| Signup off | `/login?signup=1` shows no sign-up tab when `allowsignup 0` |
| Engine private | No listening HTTP port for `robominer-engine` |
| Rate limit | Rapid failed logins get `503` (nginx) or fail2ban ban |

## 8. Known gaps (app-level)

These are **not** solved by proxy config alone. Accept the risk or plan follow-up
work:

| Gap | Mitigation today |
| --- | --- |
| No CSRF tokens | `SameSite=Lax` cookies; keep sessions short; HTTPS only |
| No app-level rate limit | Proxy `limit_req` + fail2ban |
| Open signup (default) | Set `allowsignup 0` for invite-only |
| Weak email validation | Manual account creation via engine CLI |
| No security headers in app | Proxy adds HSTS, `X-Frame-Options`, etc. |

## Related docs

- [deploy/reverse-proxy/README.md](reverse-proxy/README.md) — TLS setup
- [deploy/systemd/README.md](systemd/README.md) — service install
- [README.md](../README.md) — build, test, run
