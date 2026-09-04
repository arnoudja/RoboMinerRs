# Typed program values (int / bool / float)

> **Status: implemented** (merged in #41). The runtime uses a tagged
> `CpuStepResult` union with `i64` ints, real `bool`, and `f64` floats; help
> text and AST literals match. Keep this document as design history — do not
> re-implement.

## Problem

Robot programs declare `int`, `double`/`float`, and `bool` variables, but the Rust VM stores every runtime value as `f64` with a sticky display kind. Arithmetic, comparisons, and logic all run in floating point; the kind is only used when formatting rally debug output (JS rounds ints / formats bools).

That means:

- `int` math is not integer math (e.g. large ints lose precision; `%` already special-cases via `i32` casts).
- Bools are `0.0` / `1.0` with a tag, not real `bool`.
- Help text claims mismatched assigns are “converted,” but the implementation just stores the `f64` and keeps the declaration kind.

## Goals

1. Process `int` values as Rust integers and `bool` values as Rust booleans inside the program runner.
2. Keep `double`/`float` as `f64`.
3. Support silent automatic type conversion where needed (declare, assign, mixed operators).
4. Preserve the animation wire format and JS debugger formatting.

## Non-goals

- Compile-time typechecker or rejecting mismatched types.
- Explicit cast syntax in the language.
- Changing JS to re-evaluate expressions (it only displays wired results).
- Changing player-visible action APIs beyond the numeric meaning of int division.

## Decision summary

| Topic | Choice |
|-------|--------|
| Representation | Tagged union on `CpuStepResult` |
| Int width | `i64` (wrapping overflow) |
| Assign / declare mismatch | Silent coerce |
| `int / int` | Truncating integer division |
| Bool in arithmetic | Coerce to int (`true`→1, `false`→0) |
| Wire JSON | Unchanged `{ k, v: number }` |

## Value model

Replace the current shape:

```rust
pub struct CpuStepResult {
    pub kind: CpuStepResultKind,
    pub value: f64,
}
```

with:

```rust
pub enum CpuStepResult {
    Bool(bool),
    Int(i64),
    Float(f64),
}
```

Keep `CpuStepResultKind` as a derived display/wire discriminant via a `kind()` method (`Bool` / `Int` / `Float`, with AST `ValueType::Double` ≡ `Float`).

Runtime variable bindings store:

- sticky `ValueType` from the declaration (coerce target on later assigns);
- current `CpuStepResult` payload (always coerced to that type on write).

### Coercion

`coerce_to(ValueType)` always succeeds:

| From → To | Bool | Int | Float |
|-----------|------|-----|-------|
| Bool | identity | `true`→`1`, `false`→`0` | `0.0` / `1.0` |
| Int | nonzero → `true` | identity | `as f64` |
| Float | nonzero → `true` | truncate toward 0 | identity |

Used for: declare initializer, assign, compound-assign write-back, and as the building block for operator promotion.

Helpers such as `as_bool()`, `as_i64()`, `as_f64()`, and `is_truthy()` implement the same rules for call sites that need a concrete Rust number (move distance, rotate, dump ore index, conditions, etc.).

### Literals (AST)

Split `ExecutableExpressionKind::Number(f64)` into:

- `Int(i64)` — lexer token with no `.`
- `Float(f64)` — token containing `.`

Keep `Bool(bool)`. Unparse emits integers without a decimal point and floats in the existing style.

## Operators and builtins

### Binary arithmetic (`+`, `-`, `*`, `/`, `%`) and `min` / `max`

Promotion:

1. If either operand is `Float` → coerce both to `Float`; result `Float` (except `%`, see below).
2. Otherwise → coerce both to `Int` (bool→int); result `Int`.

Division:

- Both ints (after bool→int) → truncating `i64` division → `Int`.
- Any float involved → `f64` division → `Float`.

Modulo (`%`):

- Coerce both operands to `Int` (float truncated toward 0; bool→int).
- Result always `Int`.

Division or modulo by zero (integer): return `Int(0)` (game-safe; avoid panics). Float division by zero keeps IEEE behavior (`±inf` / `NaN`).

### Comparisons (`>`, `<`, `>=`, `<=`, `==`, `!=`)

- If either operand is `Float` → compare as `f64`.
- Else → compare as `i64` (bool→int).
- Result always `Bool`.

### Logic (`&&`, `||`, `!`)

Truthy = `Bool` value / `Int != 0` / `Float != 0`. Result `Bool`.

### Unary and math builtins

- Unary `-` / `abs`: bool→int first; preserve `Int` or `Float`.
- `sqrt` / `sin` / `cos` / `tan`: coerce to float; result `Float`.

### Increment / decrement

Apply `±1` in the variable’s declared type (int stays int; double stays double). For `bool`, coerce through normal assign rules after the numeric step (effectively bool←int coerce of the updated value).

### Control flow

`if` / `while` conditions use `is_truthy()` on the typed condition result.

### Actions and properties

Existing kind maps (`for_action`, `for_robot_property`, `for_area_property`, ore distance/type, time) return the matching enum variant with a real `i64` / `f64` / `bool` payload instead of `f64` + tag.

## Serialization boundary

`robominer-sim` `AnimationCpuStepResult` stays `{ k: "b"|"i"|"f", v: number }`.

Mapping:

- `Bool(b)` → `{ k: "b", v: 0.0 or 1.0 }`
- `Int(n)` → `{ k: "i", v: n as f64 }`
- `Float(x)` → `{ k: "f", v: x }`

JS `formatRallySourceStepResult` unchanged (`true`/`false`, `Math.round`, `toFixed(2)`).

## Player-facing docs

Update `robominer-web/static/help/robot_program.html` Variables section to state:

- Mismatched values are converted (truncate float→int toward zero; nonzero↔true; bool↔0/1).
- `int / int` uses integer division; involving a `double` uses real division.

## Testing

- Update existing assertions that read `.value` / `.kind` fields on `CpuStepResult`.
- Add focused unit/runner tests for: coerce table, `int/int` division, mixed float promote, bool-in-arithmetic, assign coerce, div-by-zero int → 0.
- Keep JS viewer tests as-is (wire contract unchanged).
- Full workspace suite: `resources/scripts/run-tests-with-db.sh`.

## Primary code touch points

| Area | Files |
|------|--------|
| Value type | `robominer-program/src/cpu_step_result.rs` |
| AST literals | `robominer-program/src/ast.rs`, compile expressions/input, `unparse.rs` |
| Eval | `runner/expression_eval/schedule.rs`, `step/work.rs`, `runtime_variables.rs`, `resume.rs`, related step paths |
| Sim bridge | `robominer-sim/src/animation.rs` (and any direct `.value` / constructor call sites) |
| Help | `robominer-web/static/help/robot_program.html` |
| Tests | `robominer-program/src/tests/*`, any sim tests asserting CPU step payloads |

## Risks

- **Behavior change:** `5/2` becomes `2` instead of `2.5` — intentional; document in help; may affect existing player programs that relied on float division of ints.
- **Precision:** ints above `2^53` no longer round-trip perfectly through animation `v: f64`; acceptable for debug display; internal eval stays exact `i64`.
- **Wide API surface:** many call sites use `.value` / `int_value(f64)` — migrate systematically with helper constructors.
