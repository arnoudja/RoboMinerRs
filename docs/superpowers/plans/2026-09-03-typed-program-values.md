# Typed Program Values Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Process robot-program `int` and `bool` as real Rust types (`i64` / `bool`) with silent coercion and truncating `int/int` division, per `docs/superpowers/specs/2026-09-03-typed-program-values-design.md`.

**Architecture:** Replace `CpuStepResult { kind, value: f64 }` with a tagged union. Expression stack and variables store the enum; operators coerce then compute in the native type; sim maps to `{k,v:f64}` at the wire boundary.

**Tech Stack:** Rust workspace (`robominer-program`, `robominer-sim`), existing cargo tests, help HTML.

## Global Constraints

- Silent coerce on declare/assign (float→int truncates toward 0; nonzero↔true; bool↔0/1).
- `int / int` truncating integer division; any float operand → float division.
- Bool participates in arithmetic via coerce to int.
- Int width `i64`, wrapping overflow; int div/mod by zero → `Int(0)`.
- Animation JSON stays `{ k: "b"|"i"|"f", v: number }`.
- No compile-time typechecker.

---

### Task 1: Tagged `CpuStepResult` + coerce helpers

**Files:**
- Modify: `robominer-program/src/cpu_step_result.rs`
- Create: `robominer-program/src/tests/cpu_step_result.rs` (or module under `tests/`)
- Modify: `robominer-program/src/tests/mod.rs` (register module)

**Interfaces:**
- Produces: `enum CpuStepResult { Bool(bool), Int(i64), Float(f64) }`
- Produces: `kind(&self) -> CpuStepResultKind`, `is_truthy`, `as_bool`, `as_i64`, `as_f64`, `coerce_to(ValueType)`, constructors `bool_value(bool)`, `int_value(i64)`, `float_value(f64)`
- Produces: updated `for_*` helpers returning enum variants; `for_binary_operator` may move to eval later — keep or replace with `evaluate_binary`

- [ ] **Step 1: Write failing unit tests** for coerce table, truthiness, `as_*`, kind()

- [ ] **Step 2: Run** `cargo test -p robominer-program cpu_step_result -- --nocapture` — expect compile/fail on missing API

- [ ] **Step 3: Implement enum + helpers** in `cpu_step_result.rs`; temporarily keep bridge methods if needed for compile, or fix call sites in same commit if the type change forces it

- [ ] **Step 4: Tests pass**

- [ ] **Step 5: Commit** `feat(program): tagged CpuStepResult with coerce helpers`

---

### Task 2: AST int/float literals + parse/unparse

**Files:**
- Modify: `robominer-program/src/ast.rs` — `Number(f64)` → `Int(i64)` + `Float(f64)`
- Modify: `robominer-program/src/compile/input.rs` — return int vs float from lexer
- Modify: `robominer-program/src/compile/executable/expressions.rs`
- Modify: `robominer-program/src/compile/executable/actions.rs` (default `Number(0.0)` → `Int(0)`)
- Modify: `robominer-program/src/unparse.rs`, `gp.rs`, `compile/program_size.rs`, `ast_visit.rs`, `expr_helpers.rs`
- Modify: tests matching `ExecutableExpressionKind::Number`

- [ ] **Step 1: Update parser** so tokens without `.` become `Int`, with `.` become `Float`

- [ ] **Step 2: Fix all match arms / size / gp / unparse**

- [ ] **Step 3: `cargo test -p robominer-program` compile+tests for compile/unparse**

- [ ] **Step 4: Commit** `feat(program): split AST number literals into Int and Float`

---

### Task 3: Typed operators and expression eval

**Files:**
- Modify: `robominer-program/src/runner/expression_eval/schedule.rs` — `ExpressionWork::PushNumber` → `PushInt`/`PushFloat`; rewrite `evaluate_operator` to take `CpuStepResult`×2 → `CpuStepResult`
- Modify: `robominer-program/src/runner/expression_eval/step/work.rs` — all push/apply paths
- Modify: `robominer-program/src/runner/expression_eval/runtime_variables.rs` — store `CpuStepResult`; `set` coerces to declared type; `update` uses typed ±1
- Modify: `resume.rs`, `step.rs`, motion/completion paths that read `.value`

- [ ] **Step 1: Add runner tests** for `5/2==2`, `5/2.0==2.5`, `true+1==2`, `double x=3.9` then assign to int → 3, bool from int

- [ ] **Step 2: Implement operator eval + variable coerce + work.rs**

- [ ] **Step 3: Fix remaining program crate compile errors**

- [ ] **Step 4: Commit** `feat(program): evaluate ints and bools as native types`

---

### Task 4: Sim wire mapping + call sites

**Files:**
- Modify: `robominer-sim/src/animation.rs` — map enum → `{k, v:f64}`
- Modify: `robominer-sim/src/simulation/program_bridge.rs` — constructors
- Modify: any sim tests building `ExecutableExpressionKind::Number` or asserting `.value`

- [ ] **Step 1: Update mapping helper**

- [ ] **Step 2: `cargo test -p robominer-sim`**

- [ ] **Step 3: Commit** `feat(sim): map typed CpuStepResult to animation JSON`

---

### Task 5: Help docs + full suite

**Files:**
- Modify: `robominer-web/static/help/robot_program.html`

- [ ] **Step 1: Document coerce + integer division**

- [ ] **Step 2: Run** `resources/scripts/run-tests-with-db.sh` (with full permissions)

- [ ] **Step 3: `cargo fmt --all` + `cargo clippy --workspace -- -D warnings`**

- [ ] **Step 4: Commit** `docs(help): document typed value conversion and int division`

---

## Spec coverage

| Spec item | Task |
|-----------|------|
| Tagged union | 1 |
| Coerce table | 1, 3 |
| AST Int/Float literals | 2 |
| Operators / div / mod / logic | 3 |
| Wire format | 4 |
| Help text | 5 |
| Full tests | 5 |
