# `gameData.sql` structural map

Canonical seed data lives in [`resources/database/gameData.sql`](../resources/database/gameData.sql).
The SQL file opens with a short comment TOC; this page is the prose map for humans navigating
that file. It is **not** a full ID catalog — use the SQL (or in-game / shop UI) for exact rows.

Schema creation and incremental changes are outside this file
(`createDatabase.sql` and `resources/database/migrations/`).

## How to use this map

1. Jump via the comment landmarks in the SQL (quoted below) or the approximate line ranges.
2. Treat ID ranges as orientation aids; gaps and per-ore ladders are intentional.
3. Prefer editing within an existing section’s pattern (same `INSERT` shape and ID scheme).

## Major sections

| Section | SQL landmark (approx. lines) | ID orientation | Tables / notes |
| --- | --- | --- | --- |
| Preamble cleanup | `-- Cleaning unreferenced…` (~15) | — | Deletes junction/requirement rows that this file fully owns (`OrePriceAmount`, area supplies, achievement step tables, …) before re-seeding |
| Ores | `-- The ore type names` (~25) | 1–11 | `Ore` — Cerbonium … Etaxy progression |
| Robot part types | `-- The robot part names` (~38) | 1–7 | `RobotPartType` — container → mining unit → battery → memory → CPU → engine → scanner |
| Shop ore prices | `-- Shop prices - Cerbonium` (~47) | 101–1102 | `OrePrice` + `OrePriceAmount`; per-ore tier blocks (standard / enhanced / …). Parts and areas reference these IDs |
| Robot parts | `-- Ore containers - Cerbonium` (~166) through scanners (~575+) | type×100 ladders (101–1100 … 701–7100) | `RobotPart` (+ price FKs). One ore-tier ladder per part type; comment headers name each type |
| AI opponent robots | `-- AI opponent robots…` (~643) | 1002–2002 | Separate from player `Robot` rows; areas point at these for NPC opposition |
| Mining areas | `-- Mining areas` (~950) | area 1001–2002; entry `OrePrice` 10001–20002 | `MiningArea`, supplies, area prices — entry cost, size, moves, tax, AI robot |
| Achievements | `-- Achievements - Initial…` (~1340) through Etaxy Mastery | achievement 1–14 | `Achievement`, predecessors, `AchievementStep`, and requirement tables (score / mining total / depot) interleaved per achievement |
| Derived updates | `-- Calculate the tier levels` (~2578) to EOF | — | Post-seed `UPDATE`s: part tier levels, wallet/depot caps, area access, achievement points, area runtime resets |

## ID schemes (orientation only)

- **Shop prices:** roughly `oreIndex * 100 + tier` within the 101–1102 band (not every integer is used).
- **Parts:** `partType * 100` … `partType * 1000` style ladders (containers `1xx`/`1xxx`, mining units `2xx`/`2xxx`, … scanners `7xx`/`7xxx`).
- **Areas vs area prices:** mining area ids stay in the 1xxx–2xxx band; their entry-cost `OrePrice` rows use 10001–20002 so they do not collide with shop part prices.
- **AI robots:** high ids in the same numeric neighborhood as late areas, but they are not player robots.

## Related narrative docs

- [ROBOTS.md](../ROBOTS.md) — sample robot programs
- [ACHIEVEMENTS.md](../ACHIEVEMENTS.md) — stepped achievement progression and rewards
- [architecture.md](architecture.md) — where seed data sits relative to migrations and runtime
