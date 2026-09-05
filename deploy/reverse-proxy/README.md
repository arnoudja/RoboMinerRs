# Reverse proxy for RoboMiner

Run `robominer-web` on localhost and terminate TLS with a reverse proxy on the
public interface. The Rust web host serves plain HTTP only; it does not handle
certificates or HTTPS directly.

## Recommended layout

| Layer | Address | Role |
| --- | --- | --- |
| Reverse proxy | `0.0.0.0:443` (public) | TLS termination |
| `robominer-web` | `127.0.0.1:8080` | Application HTTP |

Set these in `/etc/robominer/robominer.env` (preferred):

```bash
HOST=127.0.0.1
PORT=8080
ROBOMINER_WEB_ROOT=/opt/robominer/static
ROBOMINER_SESSION_SECRET=<long random secret>
ROBOMINER_SECURE_COOKIES=1
ROBOMINER_ALLOW_SIGNUP=0
ROBOMINER_TRUST_PROXY=1
```

Public self-registration is off by default. Set `ROBOMINER_ALLOW_SIGNUP=1`
(or legacy `allowsignup 1`) to open sign-up; keep it `0` for invite-only.

`trustproxy 1` trusts **only** `X-Real-Ip` for login rate limits and auth
failure logs (spoofable `X-Forwarded-For` is ignored). Enable only when a
reverse proxy **overwrites** `X-Real-Ip` with the connecting client address
(the example nginx/Caddy configs set it); leave unset (default off) if the app
is reachable directly. Override with `ROBOMINER_TRUST_PROXY=1`. Do not use
`$proxy_add_x_forwarded_for` — that prepends a client-controlled hop.

`sessionsecret` is required whenever the web host binds outside localhost.
Behind a reverse proxy, keep `host` on loopback so the application is not
exposed directly on the network.

Enable secure cookies when users reach the site over HTTPS so session cookies
include the `Secure` attribute. Set `securecookies 1` in config or export
`ROBOMINER_SECURE_COOKIES=1`.

## Caddy (automatic HTTPS)

1. Install [Caddy](https://caddyserver.com/docs/install).
2. Copy and edit `deploy/reverse-proxy/Caddyfile`:

   ```bash
   sudo cp deploy/reverse-proxy/Caddyfile /etc/caddy/Caddyfile
   sudoedit /etc/caddy/Caddyfile
   ```

3. Replace `robominer.example.com` with your hostname and ensure DNS points at
   the server.
4. Reload Caddy:

   ```bash
   sudo systemctl enable --now caddy
   sudo systemctl reload caddy
   ```

Caddy obtains Let's Encrypt certificates automatically for public hostnames.

## nginx (bring your own certificates)

1. Install nginx and obtain a certificate (for example with
   [certbot](https://certbot.eff.org/)).
2. Copy and edit `deploy/reverse-proxy/nginx.conf`:

   ```bash
   sudo cp deploy/reverse-proxy/nginx.conf /etc/nginx/sites-available/robominer.conf
   sudo ln -s /etc/nginx/sites-available/robominer.conf /etc/nginx/sites-enabled/
   sudoedit /etc/nginx/sites-available/robominer.conf
   ```

3. Update `server_name` and the `ssl_certificate` paths.
4. Test and reload:

   ```bash
   sudo nginx -t
   sudo systemctl reload nginx
   ```

## Verify

1. Confirm the web service listens only on localhost:

   ```bash
   ss -ltnp | grep 8080
   ```

2. Open `https://robominer.example.com/login` in a browser.
3. After logging in, inspect the session cookie in browser devtools
   (`__Host-robominer_session` when Secure cookies are on). It should include
   `HttpOnly`, `SameSite=Lax`, `Path=/`, no `Domain`, and `Secure` when
   `securecookies 1` is set.

## Notes

- The proxy **must** set `X-Real-IP` to the connecting client (`$remote_addr` in
  nginx, `{remote_host}` in Caddy). With `trustproxy 1`, RoboMiner uses only
  that header for rate-limit keys; missing/blank Real-IP uses the dedicated key
  `proxy-missing-real-ip` (and logs an error) instead of collapsing onto the
  loopback peer. `X-Forwarded-For` / `X-Forwarded-Proto` are still useful for
  other tooling but are not used for app rate limits.
- Static CSS is served by `robominer-web` from `webroot`; the proxy does not
  need a separate static file root unless you choose to offload assets later.
- Keep `robominer-engine` off the public internet. It only needs database access
  and does not serve HTTP traffic.

For a full internet-exposure checklist (firewall, rate limits, fail2ban), see
[INTERNET-HARDENING.md](../INTERNET-HARDENING.md).

See also `deploy/systemd/README.md` for installing the web and engine services.
