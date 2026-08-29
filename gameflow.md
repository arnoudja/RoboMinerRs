# Early-game flow (current balance)

Design notes for the first hour of play, aligned with current seed data in
`gameData.sql` and wallet cap `INITIAL_ORE_WALLET_MAX = 5`. See `ACHIEVEMENTS.md`
for the full achievement mechanism and later tracks.

## New player (after signup)

Signup auto-claims achievement **1 / step 1**:

- 1 robot with standard parts (container **101**, mining unit **201**, battery
  **301**, memory **401**, CPU **501**, engine **601**, scanner **701**)
- Default program: `move(1);` + `mine();` (fits memory size **4**)
- Ore container capacity **2** on the robot (wallet is separate)
- Mining speed **1**, CPU **1** i/t, engine forward **15** / backward **3** /
  rotate **8** → effective move speed **1.0**
- Battery capacity **140**, recharge time **5** s
- **1** mining queue slot
- Access to **Cerbonium-mini** (`1001`)
- Cerbonium **wallet** cap **5** (first row created when ore is first claimed)

## Cerbonium-mini (`1001`)

- 10×10 area, **20** turns (`maxMoves`), **5** s mining time per action, **25%**
  container tax, **10%** depot tax, mining target **2**
- One Cerbonium heap (supply 4, radius 4)
- Queue cost: **2** Cerbonium (from `OrePrice` 101) — needs wallet cap ≥ 2 after
  first claim

## Suggested early progression

1. **Queue one session** in Cerbonium-mini. After tax, first claim may add ~1–2
   Cerbonium to the wallet (capped at **5** until achievements raise it).

2. **Achievement 2 step 1** — mine **1** Cerbonium lifetime → **+1 queue** (2
   slots total).

3. **Fill the queue** with two more runs → more Cerbonium in wallet; container
   still holds **2** per rally.

4. **Achievement 2 step 2** — mine **20** Cerbonium lifetime → wallet cap **20**.
   The starting cap of **5** already covers Enhanced Memory.

5. **Shop: Enhanced Memory Module (402)** — **5** Cerbonium, memory **8**. Equip
   it and apply, then change the program to `move(1); while (mine());` (compiled
   size **5**). Starter memory **4** rejects that program.

6. **Achievement 2 step 3** — mine **25** Cerbonium and score ≥ **300** in
   Cerbonium-mini → unlock **Cerbonium-Starter** (`1002`, 15×15, 40 turns, 20%
   tax). With mining target **2**, one high-tier ore already scores **450**.

7. **Step 4** — mine **50** Cerbonium → wallet cap **50**, depot cap **5** (can
   afford Enhanced Ore Container **102**, capacity **5**).

8. **Step 5** — mine **75** Cerbonium → **+1 queue** (3 slots).

9. **Step 6** — mine **100** Cerbonium; scores ≥ **500** in mini and ≥ **300** in
   Starter → unlock **Cerbonium-Advanced** (`1003`).

10. **Step 7** — mine **120** Cerbonium → wallet cap **100**, depot cap **10**;
    unlocks **Oxaria Mastery** (achievement 3).

11. **Oxaria Mastery step 1** — mine **125** Cerbonium and score ≥ **550** in
    Starter → unlock **Oxaria-Light** (`1101`, queue cost **1** Cerbonium).

12. Continue Cerbonium Mastery steps **8–12** for higher wallet and depot caps.
    The second robot is achievement **6**, unlocked after Neudralion Mastery
    step 5 — not a Cerbonium-track reward.

The in-game tutorial matches this shop-then-program order (help steps 4–5).

## Part upgrades (shop)

Early targets from standard → enhanced lines (part IDs in parentheses):

| Goal | Part | Ore cost | Effect |
| --- | --- | --- | --- |
| Better program | Enhanced Memory Module (402) | 5 Cerbonium | Memory **8** (`move(1); while (mine());`) |
| Bigger container | Enhanced Ore Container (102) | 5 Cerbonium | Capacity **5** |
| Faster CPU | Fast CPU (502) | 5 Cerbonium | CPU **3** i/t |
| Longer runs | Cerbonium Battery (303) | 15 Cerbonium | Battery **420** |

Exact prices are in `gameData.sql` (`OrePrice` / `OrePriceAmount`). Shared
`orePriceId` **102** is 5 Cerbonium (Enhanced Memory, Enhanced Container, Fast
CPU).
