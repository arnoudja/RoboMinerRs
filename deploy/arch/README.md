# Arch / Omarchy package (AUR-ready)

Source package that installs RoboMiner with the same layout as the Debian
`.deb` (`/opt/robominer`, systemd units, sysusers, `gameData.sql`).

This directory is ready to copy into an AUR package repo. This project does
**not** publish to the AUR automatically.

## Layout

| Path | Purpose |
|------|---------|
| `/opt/robominer/bin/` | `robominer-engine`, `robominer-web`, `robominer-wait-web-health` |
| `/opt/robominer/static/` | Web static assets |
| `/usr/lib/systemd/system/` | `robominer-engine.service`, `robominer-web.service` |
| `/usr/lib/sysusers.d/robominer.conf` | `robominer` system user |
| `/usr/share/robominer/gameData.sql` | Seed data applied on install when config exists |
| `/usr/share/robominer/package-post-install.sh` | Shared Debian/Arch post-install hook |

The package does **not** create `/etc/robominer/robominer.env`. When that file
(or `ROBOMINER_DATABASE_URL`) already exists, post-install runs migrations,
applies `gameData.sql`, and starts enabled units (same behavior as the `.deb`).

## Build from a local checkout (Omarchy / Arch)

Requires `base-devel` and `rust` (provides Cargo):

```sh
sudo pacman -S --needed base-devel rust
cd deploy/arch
./makepkg-local.sh -si
```

`makepkg-local.sh` packs the **current working tree** (including uncommitted
edits; respects `.gitignore`), sets `pkgver` from the workspace `Cargo.toml`
(via `sync-pkgver-from-cargo.sh` into `PKGBUILD` / `.SRCINFO`), writes
`PKGBUILD.local`, and runs `makepkg`. Pass any `makepkg` flags after the script
name (`-s` installs make-deps, `-i` installs the package). For local builds and
`./update.sh`, bump the version only in the root `Cargo.toml`.

## Build from the AUR-style PKGBUILD

1. Set `_commit` in `PKGBUILD` to the git revision you want to package.
2. Refresh checksums: `updpkgsums` (replaces `SKIP`).
3. Refresh metadata: `makepkg --printsrcinfo > .SRCINFO`
4. Build: `makepkg -si`

GitHub source archives use the prefix `RoboMinerRs-<commit>/` (see `_srcdir`).

## After install

```sh
sudo install -d /etc/robominer
sudo cp /opt/robominer/deploy/systemd/README.md /tmp/ # reference only
# Create /etc/robominer/robominer.env from deploy/systemd/robominer.env.example
# (copy from the git tree or package docs), then:
sudo /usr/share/robominer/package-post-install.sh configure
# or:
sudo systemctl start robominer-engine robominer-web
```

Supported database targets remain **MySQL 8.4** and **MariaDB 10.11+**. On
Omarchy / Arch, prefer **host MariaDB** (`pacman -S mariadb`); Docker MySQL 8.4
remains a supported alternative via the repo test helpers.

## AUR publish checklist (manual)

1. Copy `PKGBUILD`, `robominer.install`, and `.SRCINFO` into your AUR clone
   (`ssh://aur@aur.archlinux.org/robominer.git`).
2. Set `_commit` to the release revision; run `updpkgsums`.
3. Run `makepkg --printsrcinfo > .SRCINFO`.
4. Test with `makepkg -si` on a clean Arch/Omarchy system.
5. Commit and `git push` to the AUR (not done from this repository).

## Pi / aarch64 note

`arch=('x86_64' 'aarch64')` builds natively on each architecture. Cross-building
the Debian Pi `.deb` still uses `resources/scripts/build-deb.sh` and the
Debian-named `aarch64-linux-gnu-gcc` linker; that path is separate from this
PKGBUILD.
