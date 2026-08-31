# `gameData.sql` structural map

Canonical seed data lives in [`resources/database/gameData.sql`](../resources/database/gameData.sql).
The file opens with a short comment TOC; this page is the prose map for humans.

## Major sections

| Section | Approx. IDs | Notes |
| --- | --- | --- |
| Ores | 1–11 | `Ore` rows (Cerbonium … Etaxy) |
| Robot part types | 1–7 | Container through ore scanner |
| Shop ore prices | 101–1102 | Per-ore price tiers referenced by parts |
| Robot parts | type×100 ranges (101–1100 … 701–7100) | One tier ladder per part type |
| AI opponent robots | 1002–2002 | Separate from player `Robot` rows |
| Mining areas | 1001–2002 | Area entry costs use `OrePrice` 10001–20002 |
| Achievements | 1–14 | Steps, predecessors, and requirement tables follow each achievement |
| Derived updates | (end of file) | Tier levels, wallet caps, depot, area access, achievement points |

## Related narrative docs

- [ROBOTS.md](../ROBOTS.md) — sample robot programs
- [ACHIEVEMENTS.md](../ACHIEVEMENTS.md) — stepped achievement progression and rewards

Schema creation and incremental changes are outside this file (`createDatabase.sql` and `resources/database/migrations/`).
