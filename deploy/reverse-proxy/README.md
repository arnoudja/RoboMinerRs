# Reverse proxy for RoboMiner

Run `robominer-web` on localhost and terminate TLS with a reverse proxy on the
public interface. The Rust web host serves plain HTTP only; it does not handle
certificates or HTTPS directly.

## Recommended layout

| Layer | Address | Role |
| --- | --- | --- |
| Reverse proxy | `0.0.0.0:443` (public) | TLS termination |
| `robominer-web` | `127.0.0.1:8080` | Application HTTP |

Set these keys in `/etc/robominer/robominer.conf`:

```text
host 127.0.0.1
port 8080
webroot /opt/robominer/static
sessionsecret <long random secret>
securecookies 1
allowsignup 0
```

`allowsignup 0` disables public self-registration (invite-only). Omit the key or
set `allowsignup 1` to keep sign-up open. Override with `ROBOMINER_ALLOW_SIGNUP=0`.

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
3. After logging in, inspect the `robominer_session` cookie in browser devtools.
   It should include `HttpOnly`, `SameSite=Lax`, and `Secure` when
   `securecookies 1` is set.

## Notes

- The proxy forwards `Host`, `X-Forwarded-For`, and `X-Forwarded-Proto`.
  RoboMiner does not require these headers today, but they are included for
  compatibility with standard proxy setups.
- Static CSS is served by `robominer-web` from `webroot`; the proxy does not
  need a separate static file root unless you choose to offload assets later.
- Keep `robominer-engine` off the public internet. It only needs database access
  and does not serve HTTP traffic.

For a full internet-exposure checklist (firewall, rate limits, fail2ban), see
[INTERNET-HARDENING.md](../INTERNET-HARDENING.md).

See also `deploy/systemd/README.md` for installing the web and engine services.
