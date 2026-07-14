# Early-game flow (current balance)

Design notes for the first hour of play, aligned with current seed data in
`gameData.sql` and wallet cap `INITIAL_ORE_WALLET_MAX = 5`. See `ACHIEVEMENTS.md`
for the full achievement mechanism.

## New player (after signup)

Signup auto-claims achievement **1 / step 1**:

- 1 robot with standard parts (container **101**, mining unit **201**, battery
  **301**, memory **401**, CPU **501**, engine **601**, scanner **701**)
- Default program: `move(1);` + `mine();` (fits memory size **4**)
- Ore container capacity **2** on the robot (wallet is separate)
- Mining speed **1**, CPU **1** i/c, engine forward **15** / backward **3** /
  rotate **8** → effective move speed **1.0**
- Battery capacity **110**, recharge time **5** s (~15 short mining cycles)
- **1** mining queue slot
- Access to **Cerbonium-mini** (`1001`)
- Cerbonium **wallet** cap **5** (first row created when ore is first claimed)

## Cerbonium-mini (`1001`)

- 10×10 area, **15** move cycles, **5** s mining time per action, **25%** tax
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
   Save for shop upgrades.

5. **Achievement 2 step 3** — score ≥ **70** in Cerbonium-mini → unlock
   **Cerbonium-Starter** (`1002`, 15×15, 25 cycles, 20% tax).

6. **Step 4** — mine **50** Cerbonium → wallet cap **50** (can afford enhanced
   container **102**, capacity **5**).

7. **Step 5** — mine **75** Cerbonium → **+1 queue** (3 slots).

8. **Step 6** — score ≥ **120** in Cerbonium-Starter → **Cerbonium-Advanced**
   (`1003`).

9. **Step 7** — mine **100** Cerbonium → wallet cap **100**; unlocks
   **Oxaria Mastery** (achievement 3).

10. **Oxaria Mastery step 1** — unlock **Oxaria-Light** (`1101`, costs 3
    Cerbonium to queue).

11. Continue Cerbonium Mastery steps **8–10** for higher wallet caps and, at
    step **10**, unlock **New robot** (achievement 99) after **1 000** Lithabine
    mined.

## Part upgrades (shop)

Early targets from standard → enhanced lines (part IDs in parentheses):

| Goal | Part | Ore cost (approx.) | Effect |
| --- | --- | --- | --- |
| Bigger container | Enhanced Ore Container (102) | 5 Cerbonium | Capacity **5** |
| Longer runs | Cerbonium Battery (303) | 15 Cerbonium | More cycles per rally |
| Better program | Enhanced Memory (402) | 6 Cerbonium | Memory **6** (`while (mine());`) |
| Faster mining | Enhanced CPU (502) | 5 Cerbonium | More instructions per turn |

Exact prices are in `gameData.sql` (`OrePrice` / `OrePriceAmount`).
