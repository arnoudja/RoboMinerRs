# Rally animation payload contract

Rust serializes rally replay data in [`robominer-sim/src/animation_payload.rs`](../robominer-sim/src/animation_payload.rs). The browser viewer under [`robominer-web/static/js/rally_animation/`](../robominer-web/static/js/rally_animation/) loads and renders the same JSON.

## Versioning

| Field | Meaning |
| --- | --- |
| `v` | Payload schema version. Current version is generated into `generated/contract.js` from `ANIMATION_PAYLOAD_VERSION` in `robominer-sim/src/animation.rs`. |
| Supported versions | v1 (legacy fields only) and v2 (adds per-turn `cpu` micro-steps). The viewer rejects other values. |

## Top-level shape

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `v` | number | yes | Must be a supported version (see `contract.js`). |
| `robots` | object | yes | Wrapper with `robot` array. |
| `robots.robot[]` | array | yes | One entry per simulated robot. |
| `ground` | object | yes | Map size and per-cell ore timeline. |
| `oreTypes` | object | yes | Maps slot letters (`A`, `B`, …) to `{ id, max }`. |

## Robot entry (`robots.robot[i]`)

| Field | Type | Notes |
| --- | --- | --- |
| `robotnr` | number | Zero-based robot index. |
| `cpuspeed` | number | CPU speed stat used for replay timing. |
| `locations` | array | One sample per simulation turn. |
| `depotMaxA` … | number | Present when depot capacity is unlocked. |
| `homeX`, `homeY`, `homeSize` | number | Depot home square for replay UI. |

## Turn sample (`locations[j]`)

| Field | Type | Notes |
| --- | --- | --- |
| `t` | number | Turn index. |
| `x`, `y`, `o` | number | Position and orientation. |
| `a` | number | Action index (wait, scan, move, mine, …). |
| `l` | number | Source line highlight when no `cpu` spans are recorded. |
| `s` | string | Stuck/status label (`wait`, `scan`, `wall`, …). |
| `cpu` | array | v2: instruction micro-steps with spans, results, and locals. |
| `A`, `B`, … / `DA`, … | number | On-robot and depot ore counts per slot. |

## CI checks

- Rust: `robominer-sim` build script writes `generated/contract.js` for the viewer.
- Rust: `animation_payload::tests::golden_payload_v2_deserializes` validates [`resources/rally_animation/golden_payload_v2.json`](../resources/rally_animation/golden_payload_v2.json).
- Node: `robominer-web/static/js/rally_animation/tests/contract.test.js` checks version constants and golden payload acceptance.

## Build order (generated `contract.js`)

`robominer-web` does **not** depend on `robominer-sim`, so building web alone will not
regenerate the viewer contract. When changing the wire format:

1. Bump `ANIMATION_PAYLOAD_VERSION` (and legacy support if needed) in `robominer-sim/src/animation.rs`
2. Run `cargo build -p robominer-sim` so `build.rs` rewrites
   `robominer-web/static/js/rally_animation/generated/contract.js`
3. Commit the regenerated `contract.js`
4. Refresh golden JSON if the payload shape changed
5. Run `resources/scripts/run-page-js-tests.sh`
