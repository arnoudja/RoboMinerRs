# Omarchy / Arch packaging design

**Date:** 2026-09-08  
**Status:** Approved (chat)

## Goals

- Document a working Omarchy/Arch **dev** path (`pacman` deps, Docker MySQL 8.4, `systemctl`).
- Ship an in-repo **source** package under `deploy/arch/` that installs the **same layout** as the Debian `.deb`, ready to copy into AUR later (no AUR submit in this work).

## Dev setup (docs)

Arch / Omarchy subsection in README and a short pointer in CONTRIBUTING:

- Packages: `base-devel`, Rust (pacman `rust` or rustup), `nodejs`, `npm`, `docker`, `python`, optional `mariadb-clients`.
- **Supported DB on Omarchy:** Docker `mysql:8.4` via `resources/scripts/ensure-test-mysql.sh` / `run-tests-with-db.sh` (host MariaDB remains best-effort).
- Day-to-day build stays `cargo build --workspace`; packaging is optional.

## Package layout (`deploy/arch/`)

| File | Role |
|------|------|
| `PKGBUILD` | `pkgname=robominer`, version synced to workspace, builds release binaries |
| `robominer.install` | Arch hooks mirroring Debian postinst behavior |
| `.SRCINFO` | Generated with `makepkg --printsrcinfo` for AUR |
| `README.md` | Local `makepkg -si` and AUR publish checklist |

**Install paths (parity with `.deb`):**

- `/opt/robominer/bin/{robominer-engine,robominer-web,robominer-wait-web-health}`
- `/opt/robominer/static/`
- `/usr/lib/systemd/system/robominer-{engine,web}.service`
- `/usr/lib/sysusers.d/robominer.conf`
- `/usr/share/robominer/gameData.sql`
- `/usr/share/robominer/package-post-install.sh`

**Depends:** `systemd`  
**Optdepends:** MySQL/MariaDB client, Docker, Python (URL parsing for gameData apply).

The package does **not** create `/etc/robominer/robominer.env`. Post-install migrates and starts services only when that file (or `ROBOMINER_DATABASE_URL`) already exists.

## Shared post-install hook

Extract migrate / gameData / sysusers / service-restart logic from `robominer-engine/debian/postinst` into `deploy/systemd/package-post-install.sh`, installed at `/usr/share/robominer/package-post-install.sh`.

Actions:

- `configure-pre` — sysusers + migrate + gameData (runs before Debian `#DEBHELPER#`)
- `configure-post` — start/restart enabled units (runs after `#DEBHELPER#`)
- `configure` — both phases (Arch `post_install` / `post_upgrade`)

## Out of scope

- Publishing to AUR
- Binary / GitHub Release packages
- Arch CI runners
- Making Pi cross-build linker names Arch-native
