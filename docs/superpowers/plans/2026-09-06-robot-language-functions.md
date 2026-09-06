# Robot Language Functions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add user-defined functions to the robot language (`fn` / typed forms, by-value params, `return`, recursion, top-level var access) and make `ProgramStep::Fault` restart like `Done` in the sim bridge.

**Architecture:** Two-pass compile builds a `BTreeMap` function registry on `ExecutableProgram` (defs stripped from main statements). Calls are `ExecutableExpressionKind::Call`. The runner suspends expression eval, pushes a scoped statement frame for the body, and resumes with the return value. Call depth > 256 yields `Fault`; sim restarts on every `Fault`.

**Tech Stack:** Rust workspace crates `robominer-program`, `robominer-sim`; HTML help in `robominer-web`.

**Spec:** [docs/superpowers/specs/2026-09-06-robot-language-functions-design.md](../specs/2026-09-06-robot-language-functions-design.md)

## Global Constraints

- Declaration forms only: `fn name(...)`, `fn T name(...)`, `T name(...)` — reject `T fn name(...)`
- Top-level function defs only (outermost sequence after implicit `{ ... }` wrap)
- Params by value; optional `int`/`double`/`float`/`bool` prefix; untyped params take arg type per call
- Missing / bare `return` / fall-off → `int 0`; explicit return type coerces; omitted type requires all valued returns to agree
- Functions read/write all top-level program variables (any source order)
- Forward calls + recursion allowed; max call depth **256** → `Fault`
- Reserved names: keywords + builtins (full list in spec)
- Function names share top-level namespace with variables (no clash)
- `Fault` restarts like `Done` in sim (all Fault causes)
- Unparse: name-sorted functions first, then main statements
- No new GP mutation operators; update visitors/size/unparse only
- After Rust changes: `cargo fmt`, `cargo clippy -D warnings`, `resources/scripts/run-tests-with-db.sh` with full permissions

## File Structure

| File | Responsibility |
|------|----------------|
| `robominer-program/src/ast.rs` | `ExecutableFunction`, `FunctionParam`, `Return`, `Call`; `functions` on `ExecutableProgram` |
| `robominer-program/src/compile/reserved.rs` (new) | Reserved-name set + helpers |
| `robominer-program/src/compile/executable/functions.rs` (new) | Parse function defs, return inference, top-level split |
| `robominer-program/src/compile/executable/statements.rs` | `return` stmt; top-level `allow_functions` / reject nested |
| `robominer-program/src/compile/executable/expressions.rs` | Parse `name(args)` as `Call` when registered |
| `robominer-program/src/compile/executable/mod.rs` | Two-pass `parse_executable_program` |
| `robominer-program/src/compile/input.rs` | Optional: peek helpers if needed for `type name (` vs declare |
| `robominer-program/src/compile/program_size.rs` | Size for `Return` / `Call` / function bodies |
| `robominer-program/src/runner/mod.rs` | Hold `functions`, call depth, suspend/resume expression eval |
| `robominer-program/src/runner/step.rs` | Execute `Return`; end-of-body implicit return |
| `robominer-program/src/runner/expression_eval/schedule.rs` | Schedule `Call` → args + `InvokeCall` |
| `robominer-program/src/runner/expression_eval/step/ongoing.rs` | Handle `InvokeCall` (push call frame) |
| `robominer-program/src/runner/expression_eval/expr_helpers.rs` | Exhaustive match updates |
| `robominer-program/src/unparse.rs` | Emit functions + `return` + calls |
| `robominer-program/src/ast_visit.rs` | Visit function bodies |
| `robominer-program/src/gp.rs` | Exhaustive matches only |
| `robominer-program/src/tests/compile_functions.rs` (new) | Compile/verify tests |
| `robominer-program/src/tests/runner_functions.rs` (new) | Runner tests |
| `robominer-sim/src/simulation/program_bridge.rs` | Fault → restart like Done |
| `robominer-sim/src/tests/program_control.rs` | Fault-restart behavioral test |
| `robominer-web/static/help/robot_program.html` | Functions help section |

---

### Task 1: AST types for functions, return, and call

**Files:**
- Modify: `robominer-program/src/ast.rs`
- Modify: every exhaustive `match` on `ExecutableStatementKind` / `ExecutableExpressionKind` / `ExecutableProgram` construction so the crate compiles (temporary `todo!` / `unreachable` only where listed below is unacceptable — add proper empty/zero arms)
- Test: `robominer-program/src/tests/compile.rs` (existing must still compile)

**Interfaces:**
- Produces:
  - `FunctionParam { name: String, value_type: Option<ValueType> }`
  - `ExecutableFunction { name: String, return_type: ValueType, params: Vec<FunctionParam>, body: Vec<ExecutableStatement> }`
  - `ExecutableProgram.functions: BTreeMap<String, ExecutableFunction>`
  - `ExecutableStatementKind::Return(Option<ExecutableExpression>)`
  - `ExecutableExpressionKind::Call { name: String, args: Vec<ExecutableExpression> }`

- [ ] **Step 1: Write a failing compile-API test that mentions functions map**

Add to a new file `robominer-program/src/tests/compile_functions.rs` and register it in `robominer-program/src/tests/mod.rs`:

```rust
use crate::*;

#[test]
fn executable_program_exposes_functions_map() {
    let program = compile_executable_source("fn answer() { return 42; } move(answer());")
        .expect("function program should compile");
    assert!(
        program.functions.contains_key("answer"),
        "compiled program must expose function registry"
    );
}
// Note: public APIs are `compile_executable_source` / `verify_source` (crate `robominer-program`).
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p robominer-program executable_program_exposes_functions_map -- --nocapture`

Expected: FAIL (no `functions` field and/or compile error on `fn`)

- [ ] **Step 3: Add AST types and wire `ExecutableProgram`**

In `ast.rs`:

```rust
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionParam {
    pub name: String,
    pub value_type: Option<ValueType>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutableFunction {
    pub name: String,
    pub return_type: ValueType,
    pub params: Vec<FunctionParam>,
    pub body: Vec<ExecutableStatement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutableProgram {
    pub statements: Vec<ExecutableStatement>,
    pub actions: Vec<ExecutableAction>,
    pub requires_runtime: bool,
    pub functions: BTreeMap<String, ExecutableFunction>,
}
```

Add variants:

```rust
// ExecutableStatementKind
Return(Option<ExecutableExpression>),

// ExecutableExpressionKind
Call {
    name: String,
    args: Vec<ExecutableExpression>,
},
```

Update `requires_runtime` for `Return` → `true` if expr present else `false` is fine as `true` always; for `Call` → always `true`.

Update `parse_executable_program` construction to set `functions: BTreeMap::new()` so non-function programs still build.

Fix exhaustiveness in: `unparse.rs`, `ast_visit.rs`, `gp.rs`, `compile/program_size.rs`, `compile/executable/mod.rs` (`collect_static_actions`), `runner/step.rs`, `runner/expression_eval/schedule.rs`, `runner/expression_eval/expr_helpers.rs`, and any other match the compiler lists. For this task only:
- `Return` / `Call` arms may `unimplemented!("functions")` in runner/schedule **or** treat `Return` as no-op Fault and `Call` schedule as push int 0 — prefer compile-time `match` arms that return safe defaults for size/unparse (`return;` / `name()`) so later tasks replace them.

Minimum size arms:

```rust
ExecutableStatementKind::Return(Some(expr)) => 1 + expression_size(expr),
ExecutableStatementKind::Return(None) => 1,
ExecutableExpressionKind::Call { args, .. } => 1 + args.iter().map(expression_size).sum::<usize>(),
```

Minimum unparse arms:

```rust
ExecutableStatementKind::Return(None) => out.push_str("return"),
ExecutableStatementKind::Return(Some(expr)) => {
    out.push_str("return ");
    unparse_expression(expr, out, 0);
}
ExecutableExpressionKind::Call { name, args } => {
    out.push_str(name);
    out.push('(');
    // comma-separate args via existing expression unparse
}
```

- [ ] **Step 4: Run existing compile tests plus the new test**

Run: `cargo test -p robominer-program executable_program_exposes_functions_map -- --nocapture`

Expected: still FAIL on parse of `fn` (map exists but compile rejects source) — that is OK for Task 1 if the test fails only on `compile_executable_source` Err. Alternatively temporarily assert `ExecutableProgram { functions: BTreeMap::new(), .. }` via a unit test on struct construction; keep the integration test for Task 2+.

Add this unit-style test instead if needed to make Task 1 green:

```rust
#[test]
fn executable_program_functions_default_empty() {
    let program = compile_executable_source("move(1);").expect("move compiles");
    assert!(program.functions.is_empty());
}
```

- [ ] **Step 5: Commit**

```bash
git add robominer-program/src/ast.rs robominer-program/src/tests/compile_functions.rs robominer-program/src/tests/mod.rs \
  robominer-program/src/unparse.rs robominer-program/src/ast_visit.rs robominer-program/src/gp.rs \
  robominer-program/src/compile/program_size.rs robominer-program/src/compile/executable/mod.rs \
  robominer-program/src/runner/step.rs robominer-program/src/runner/expression_eval
git commit -m "Add AST types for robot language functions, return, and call."
```

---

### Task 2: Reserved names + parse function definitions (no calls yet)

**Files:**
- Create: `robominer-program/src/compile/reserved.rs`
- Create: `robominer-program/src/compile/executable/functions.rs`
- Modify: `robominer-program/src/compile/mod.rs` (mod reserved)
- Modify: `robominer-program/src/compile/executable/mod.rs` (mod functions; two-pass entry)
- Modify: `robominer-program/src/compile/executable/statements.rs`
- Modify: `robominer-program/src/tests/compile_functions.rs`

**Interfaces:**
- Consumes: AST types from Task 1
- Produces:
  - `is_reserved_name(name: &str) -> bool`
  - `parse_function_header(...)` / top-level function parse
  - `parse_executable_program` returns filled `functions` map; main `statements` without defs

- [ ] **Step 1: Write failing tests for declaration forms**

```rust
#[test]
fn compiles_fn_forms_and_rejects_type_fn() {
    assert!(verify_source("fn f() { } move(1);").verified);
    assert!(verify_source("fn int f() { return 1; } move(1);").verified);
    assert!(verify_source("int f() { return 1; } move(1);").verified);
    let bad = verify_source("int fn f() { return 1; } move(1);");
    assert!(!bad.verified);
    assert!(
        bad.error_description.to_lowercase().contains("fn")
            || bad.error_description.to_lowercase().contains("syntax"),
        "{}",
        bad.error_description
    );
}

#[test]
fn rejects_nested_function_and_reserved_names() {
    assert!(!verify_source("fn outer() { fn inner() { } } move(1);").verified);
    assert!(!verify_source("fn move() { } move(1);").verified);
    assert!(!verify_source("fn while() { } move(1);").verified);
}
```

- [ ] **Step 2: Run tests — expect FAIL**

Run: `cargo test -p robominer-program compiles_fn_forms_and_rejects_type_fn rejects_nested_function_and_reserved_names -- --nocapture`

- [ ] **Step 3: Implement reserved set**

`compile/reserved.rs`:

```rust
pub(super) fn is_reserved_name(name: &str) -> bool {
    matches!(
        name,
        "fn" | "return" | "if" | "else" | "while" | "do" | "const"
            | "int" | "double" | "float" | "bool" | "true" | "false"
            | "move" | "rotate" | "mine" | "dump" | "dumpA" | "dumpB" | "dumpC"
            | "scan" | "time" | "ore" | "oreDistance" | "oreType"
            | "abs" | "sqrt" | "sin" | "cos" | "tan" | "min" | "max"
            | "robot" | "area"
    )
}
```

- [ ] **Step 4: Implement two-pass program parse**

In `parse_executable_program` (`compile/executable/mod.rs`):

1. **Signature scan:** parse the outer sequence with a mode that, at top-level only, recognizes:
   - `fn` / `fn T` / `T` followed by `name` then `(` → function: parse params + `{ body }`, stash in `Vec`/map; do **not** put in main statements
   - else existing statement parse
   - During this pass, register each function name before parsing bodies is not enough for forward calls — so for Task 2, bodies may parse without resolving `Call` yet (calls still fail as undeclared variables). Register all signatures in a `BTreeMap<String, Signature>` **before** parsing any body by using brace-skipping for phase 1a:

**Phase 1a (signatures only):** walk top-level items; on function header parse name/params/optional return type; skip body with brace depth counter; reject reserved/duplicate names; collect top-level `Declare` names into `program_globals: BTreeMap<String, ValueType>` by parsing declare statements normally OR by a second full parse.

**Practical approach that fits this parser:** full parse of outer sequence where function defs are parsed completely into `ExecutableFunction` values stored on the side, with `ParseContext { allow_function_defs: bool, function_signatures: ..., program_globals: ..., in_function_body: bool }`. Do it as:

1. Parse once with `allow_function_defs=true` only for the outermost `parse_executable_sequence` call (thread a flag). Nested sequences pass `false`.
2. When parsing a function: parse header; push signature into a mutable registry on `CompileInput` immediately; parse body with `in_function_body=true`, `allow_function_defs=false`.
3. After outer parse, move registry into `ExecutableProgram.functions`.
4. Forward calls: temporarily allow `Call` parse for any name in registry **or** any identifier followed by `(` that is not a builtin (resolve existence in a post-pass). For Task 2, post-pass can require functions exist; Task 3 adds call expression parsing.

**Disambiguating `int name(` vs `int name`:** in `parse_executable_variable_statement` / new top-level hook, after reading type+name, if `peek() == '('` and `allow_function_defs`, parse as function (no `fn` keyword form `T name(`). If `fn` seen first: optional type, then name, then `(`.

Reject `int fn name` explicitly when after type you see `fn`.

**`return`:** in `parse_executable_statement`, if `use_next_word("return")`: parse optional expression; if not `in_function_body`, error. Emit `Return(...)`.

**Return-type inference (can complete in Task 2 for defs without calls):** after body AST exists, walk for `Return(Some(expr))`; determine literal/expression result types where possible; for Task 2, infer only from literal returns (`return 1` → Int, `return 1.0` → Double, `return true` → Bool). Full expression typing can stay limited: if any valued return’s type cannot be known statically, require explicit return type OR treat non-literal as requiring explicit type. **Spec says all valued returns must agree on concrete type** — implement a small `infer_return_type(body) -> Result<ValueType>` that uses a recursive expression type guesser (literals, variables via declared types in scope, binary promotions matching runtime). If guess fails for a valued return and return type omitted → compile error asking for explicit type OR unify when all succeed.

Simpler spec-compliant approach: during inference, classify each valued return expression with `expression_value_type(expr, vars) -> Option<ValueType>`; if any is `None`, compile error `"cannot infer return type"`; if disagree, error `"conflicting return types"`.

- [ ] **Step 5: Run Task 2 tests — expect PASS**

Run: `cargo test -p robominer-program compiles_fn_forms_and_rejects_type_fn rejects_nested_function_and_reserved_names executable_program_exposes_functions_map -- --nocapture`

- [ ] **Step 6: Commit**

```bash
git add robominer-program/src/compile robominer-program/src/tests/compile_functions.rs
git commit -m "Parse robot language function definitions and reserved names."
```

---

### Task 3: Parse calls + arity + namespace clash checks

**Files:**
- Modify: `robominer-program/src/compile/executable/expressions.rs`
- Modify: `robominer-program/src/compile/executable/functions.rs` (post-pass validate)
- Modify: `robominer-program/src/tests/compile_functions.rs`

**Interfaces:**
- Consumes: function registry on `CompileInput` during expression parse
- Produces: `ExecutableExpressionKind::Call { name, args }`

- [ ] **Step 1: Write failing call/arity/clash tests**

```rust
#[test]
fn compiles_calls_and_forward_reference() {
    assert!(verify_source("move(f()); fn int f() { return 2; }").verified);
    assert!(verify_source("fn int add(int a, b) { return a + b; } move(add(1, 2));").verified);
}

#[test]
fn rejects_arity_mismatch_and_name_clash() {
    assert!(!verify_source("fn int f(a) { return a; } move(f());").verified);
    assert!(!verify_source("fn int f() { return 1; } int f; move(1);").verified);
    assert!(!verify_source("int f; fn int f() { return 1; } move(1);").verified);
}

#[test]
fn rejects_return_outside_function() {
    assert!(!verify_source("return 1;").verified);
}

#[test]
fn omitted_return_type_requires_agreement() {
    assert!(verify_source("fn f() { return 1; } move(f());").verified);
    assert!(!verify_source("fn f() { if (true) { return 1; } return 1.5; } move(f());").verified);
}
```

- [ ] **Step 2: Run — expect FAIL on calls**

Run: `cargo test -p robominer-program compiles_calls_and_forward_reference rejects_arity_mismatch_and_name_clash -- --nocapture`

- [ ] **Step 3: Implement Call parsing**

In `expressions.rs`, where an identifier is read via `use_next_word_any()`:

Before `expect_declared_variable`, if `input.eat_char('(', false)` was not consumed yet — **peek** for `(`:
- If next char is `(` and `input.functions.contains(name)` (or name is registered): parse argument list (zero or more expressions separated by `,`), expect `)`, build `Call { name, args }`.
- Else existing variable path.

Thread `functions: BTreeMap<String, FunctionSignature>` on `CompileInput` (signature = param count + param type options + optional explicit return type; full body not required for arity check).

Because of forward references, ensure **all** signatures are registered before parsing main statements and before parsing function bodies (phase 1a brace-skip, then phase 1b full parse with registry populated).

Post-pass: clash between function names and top-level variable names.

- [ ] **Step 4: Run Task 3 tests — expect PASS**

Run: `cargo test -p robominer-program compile_functions -- --nocapture`

- [ ] **Step 5: Commit**

```bash
git add robominer-program/src/compile robominer-program/src/tests/compile_functions.rs
git commit -m "Parse and validate robot language function calls."
```

---

### Task 4: Runner — call frames, return, depth limit

**Files:**
- Modify: `robominer-program/src/runner/mod.rs`
- Modify: `robominer-program/src/runner/step.rs`
- Modify: `robominer-program/src/runner/expression_eval/schedule.rs`
- Modify: `robominer-program/src/runner/expression_eval/step/ongoing.rs`
- Modify: `robominer-program/src/runner/expression_eval/step/work.rs` (if needed)
- Modify: `robominer-program/src/runner/expression_eval/resume.rs` (if needed)
- Create: `robominer-program/src/tests/runner_functions.rs`
- Modify: `robominer-program/src/tests/mod.rs`

**Interfaces:**
- Consumes: `ExecutableProgram.functions`, `Call`, `Return`
- Produces: runtime call semantics; `Fault` on depth > 256

- [ ] **Step 1: Write failing runner tests**

```rust
use crate::*;
use super::helpers::*;

#[test]
fn function_return_value_used_by_move() {
    let program = compile_executable_source("fn int f() { return 2; } move(f());")
        .expect("compile");
    let mut runner = program.runner();
    let mut ctx = test_context(20, None);
    assert!(matches!(
        runner.next_action(&mut ctx),
        Some(ExecutableAction::Move(d)) if (d - 2.0).abs() < 1e-9
    ));
}

#[test]
fn untyped_param_is_by_value_and_dynamic() {
    let program = compile_executable_source(
        "fn int id(x) { return x; } move(id(3)); move(id(4));",
    )
    .expect("compile");
    let mut runner = program.runner();
    let mut ctx = test_context(30, None);
    assert!(matches!(runner.next_action(&mut ctx), Some(ExecutableAction::Move(d)) if (d-3.0).abs()<1e-9));
    let mut ctx = test_context(30, Some(3.0));
    assert!(matches!(runner.next_action(&mut ctx), Some(ExecutableAction::Move(d)) if (d-4.0).abs()<1e-9));
}

#[test]
fn function_reads_and_writes_top_level_var() {
    let program = compile_executable_source(
        "fn bump() { x = x + 1; } int x = 0; bump(); move(x);",
    )
    .expect("compile");
    let mut runner = program.runner();
    let mut ctx = test_context(40, None);
    // Drain CPU until Move
    let action = runner.next_action(&mut ctx);
    assert!(matches!(action, Some(ExecutableAction::Move(d)) if (d-1.0).abs()<1e-9));
}

#[test]
fn recursion_and_depth_fault() {
    let ok = compile_executable_source(
        "fn int sum(int n) { if (n <= 0) { return 0; } return n + sum(n - 1); } move(sum(3));",
    )
    .expect("compile");
    let mut runner = ok.runner();
    let mut ctx = test_context(200, None);
    assert!(matches!(runner.next_action(&mut ctx), Some(ExecutableAction::Move(d)) if (d-6.0).abs()<1e-9));

    let deep = compile_executable_source(
        "fn int rec(int n) { return rec(n); } move(rec(1));",
    )
    .expect("compile");
    let mut runner = deep.runner();
    let mut ctx = test_context(10_000, None);
    let mut saw_fault = false;
    for _ in 0..10_000 {
        match runner.step(&mut ctx) {
            ProgramStep::Fault => {
                saw_fault = true;
                break;
            }
            ProgramStep::Done => break,
            ProgramStep::Action(_) => {
                // should not need actions
            }
            ProgramStep::Cpu => {}
        }
    }
    assert!(saw_fault, "infinite recursion must Fault at depth 256");
}

#[test]
fn fallthrough_returns_zero() {
    let program = compile_executable_source("fn int f() { } move(f());").expect("compile");
    let mut runner = program.runner();
    let mut ctx = test_context(20, None);
    assert!(matches!(runner.next_action(&mut ctx), Some(ExecutableAction::Move(d)) if d.abs()<1e-9));
}
```

- [ ] **Step 2: Run — expect FAIL**

Run: `cargo test -p robominer-program runner_functions -- --nocapture`

- [ ] **Step 3: Implement runner call machinery**

On `ExecutableRunner`:

```rust
functions: BTreeMap<String, ExecutableFunction>,
call_depth: usize,
/// Suspended expression evaluations waiting for a user-call return (stack).
suspended_expression_evals: Vec<OngoingExpressionEval>,
```

`ExecutableRunner::new(program)` stores `program.functions` and starts `call_depth = 0`.

**Scheduling** (`schedule.rs`): for `Call { name, args }`, schedule each arg left-to-right, then `ExpressionWork::InvokeCall { name: name.clone(), argc: args.len() }`.

**Invoke** (`ongoing.rs`): when work is `InvokeCall { name, argc }`:
1. Pop `argc` values from `eval.values` (last arg on top — pop into a `Vec` then reverse, or pop into front).
2. If `call_depth >= 256` → `abort_with_fault()`.
3. Look up function; coerce/bind params:
   - typed param → `arg.coerce_to(param_type)` then `declare` in new scope
   - untyped → `declare` with `ValueType` derived from arg (`Bool`/`Int`/`Float`→`Double`)
4. `call_depth += 1`
5. Take current `expression_eval` and push onto `suspended_expression_evals`
6. `variables.push_scope()`; declare params in that scope
7. `push_frame(function.body, None, None, true)` — body statements, scoped
8. Mark frame as a **call frame** (add field `is_function_call: bool` on `ExecutionFrame`) so end-of-body triggers return

**Return statement** (`step.rs`): on `Return(expr)`:
- If `Some(expr)`, start expression eval with new resume `ExpressionResume::Return`
- If `None`, complete return with `CpuStepResult::Int(0)`

`ExpressionResume::Return` handler: coerce to current function `return_type`, then `complete_function_return(value)`.

**`complete_function_return`:**
1. Pop call frame(s) until the function’s scoped call frame is popped (or simply pop the call frame and any inner frames — typically Return only runs inside the call’s statement stack; pop until `is_function_call` frame popped).
2. `call_depth -= 1`
3. Restore `expression_eval` from `suspended_expression_evals`, push return value onto `values`, advance invoke work index, continue

**End of call body without Return:** when popping an `is_function_call` frame because index past end, behave like `return` with `Int(0)` coerced to return type.

**Globals:** root frame stays `scoped: false` with the single outer `RuntimeVariables` scope holding top-level declares; each call `push_scope` for params/locals so `set` on a global name still finds the outer binding (existing `set` walks scopes reverse).

- [ ] **Step 4: Run runner_functions — expect PASS**

Run: `cargo test -p robominer-program runner_functions -- --nocapture`

- [ ] **Step 5: Commit**

```bash
git add robominer-program/src/runner robominer-program/src/tests/runner_functions.rs robominer-program/src/tests/mod.rs
git commit -m "Execute robot language function calls, returns, and depth limits."
```

---

### Task 5: Fault restarts like Done in sim

**Files:**
- Modify: `robominer-sim/src/simulation/program_bridge.rs`
- Modify: `robominer-program/src/runtime.rs` (doc comment on `ProgramStep::Fault`)
- Modify: `robominer-sim/src/tests/program_control.rs`

**Interfaces:**
- Consumes: `ProgramStep::Fault`
- Produces: same restart behavior as `Done`

- [ ] **Step 1: Write failing sim test**

```rust
#[test]
fn program_fault_restarts_like_done() {
    // Infinite recursion → Fault at depth 256; after restart, should Fault again but keep simulating.
    let program = robominer_program::compile_executable_source(
        "fn int rec(int n) { return rec(n); } move(rec(1));",
    )
    .expect("compile");

    let mut spec = RobotSpec::test_robot();
    spec.max_turns = 5;
    let mut simulation = Simulation::new(
        Ground::new(5, 5),
        5,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );
    simulation.run();
    // If Fault halted forever, the sim would still finish turns via Wait; assert robot stayed alive
    // and did not permanently stick without CPU charging. Mirror empty-program Done budgeting:
    // after run, turns elapsed normally.
    assert_eq!(simulation.robot(0).actions_done().iter().sum::<i32>(), 0);
}
```

Prefer a tighter test if helpers can step one robot cycle: assert that after a Fault, `runner` is replaced (e.g. call depth back to 0) by exposing or observing that a subsequent Fault can occur again within the same `run()` rather than infinite Wait with no CPU progress. Inspect `empty_program.rs` Done budgeting test and mirror it for Fault.

Also add a unit-level approach: if `program_bridge` logic is hard to hook, extract restart into a small helper used by both Done and Fault and test indirectly via animation/CPU charge like `empty_program.rs`.

- [ ] **Step 2: Run — expect FAIL (Fault still halts)**

Run: `cargo test -p robominer-sim program_fault_restarts_like_done -- --nocapture`

- [ ] **Step 3: Change Fault handling**

In `program_bridge.rs`, replace the Fault arm so it matches Done:

```rust
ProgramStep::Fault => {
    let ActionSource::Program { program, runner, .. } =
        &mut self.action_sources[robot_index]
    else {
        unreachable!("ActionSource::Program checked above");
    };
    **runner = program.runner();
    self.action_results[robot_index] = None;
    self.last_cpu_highlight[robot_index] = None;
    self.cpu_highlight_seed_floor[robot_index] = cpu_steps.len();
    cpu_used += 1;
}
```

(Adapt exact field names to the file’s current identifiers — mirror the existing `Done` arm literally.)

Update `ProgramStep::Fault` docs in `runtime.rs` to say callers must restart like `Done`, not halt permanently.

- [ ] **Step 4: Run sim test — expect PASS**

Run: `cargo test -p robominer-sim program_fault_restarts_like_done -- --nocapture`

- [ ] **Step 5: Commit**

```bash
git add robominer-sim/src/simulation/program_bridge.rs robominer-sim/src/tests/program_control.rs robominer-program/src/runtime.rs
git commit -m "Restart robot programs on Fault, matching Done behavior."
```

---

### Task 6: Unparse, program size, ast_visit polish + help docs

**Files:**
- Modify: `robominer-program/src/unparse.rs`
- Modify: `robominer-program/src/compile/program_size.rs`
- Modify: `robominer-program/src/ast_visit.rs`
- Modify: `robominer-program/src/tests/unparse.rs` (or compile_functions)
- Modify: `robominer-web/static/help/robot_program.html`

- [ ] **Step 1: Write failing unparse round-trip test**

```rust
#[test]
fn unparse_emits_functions_before_main() {
    let source = "fn int f() { return 1; } move(f());";
    let program = compile_executable_source(source).expect("compile");
    let text = unparse_program(&program);
    let f_pos = text.find("fn int f").or_else(|| text.find("int f")).expect("function in unparse");
    let m_pos = text.find("move").expect("move in unparse");
    assert!(f_pos < m_pos, "functions must unparse before main statements: {text}");
    compile_executable_source(&text).expect("unparsed program must recompile");
}
```

- [ ] **Step 2: Run — expect FAIL if order wrong**

Run: `cargo test -p robominer-program unparse_emits_functions_before_main -- --nocapture`

- [ ] **Step 3: Implement unparse of `program.functions` first (BTreeMap order), then statements; visit bodies in `ast_visit`; ensure size includes function bodies in total program size used by verify**

If `program_size` only walks `statements`, add:

```rust
pub fn program_instruction_size(program: &ExecutableProgram) -> usize {
    let main: usize = program.statements.iter().map(statement_size).sum();
    let functions: usize = program
        .functions
        .values()
        .map(|f| f.body.iter().map(statement_size).sum::<usize>())
        .sum();
    main + functions
}
```

Help section: add `<h2 id="functions">Functions</h2>` after Flow control (before Expressions) covering forms, params, return typing, top-level vars, reserved names, recursion/depth.

- [ ] **Step 4: Run unparse + compile_functions + runner_functions**

Run: `cargo test -p robominer-program unparse_emits_functions_before_main compile_functions runner_functions -- --nocapture`

- [ ] **Step 5: Commit**

```bash
git add robominer-program/src/unparse.rs robominer-program/src/compile/program_size.rs \
  robominer-program/src/ast_visit.rs robominer-program/src/tests robominer-web/static/help/robot_program.html
git commit -m "Unparse and document robot language functions."
```

---

### Task 7: Format, clippy, full workspace tests

**Files:** none intended (fixes only if tools report issues)

- [ ] **Step 1: Format + clippy**

```bash
cargo fmt --all
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
```

Expected: clean

- [ ] **Step 2: Full test suite**

```bash
resources/scripts/run-tests-with-db.sh
```

(Use Shell `required_permissions: ["all"]`.)

Expected: all tests pass

- [ ] **Step 3: Commit any fmt/clippy fixes**

```bash
git add -A
git commit -m "Fix fmt/clippy after robot language functions."
```

(Skip commit if tree clean.)

---

## Spec coverage (self-review)

| Spec requirement | Task |
|------------------|------|
| `fn` / `fn T` / `T name` forms; reject `T fn` | 2 |
| Top-level only | 2 |
| Params by value; optional types; untyped per-call | 3–4 |
| Return / bare / fall-off → int 0 | 2, 4 |
| Omitted return type agreement | 2–3 |
| Explicit return coerce | 4 |
| Outer top-level var R/W any order | 2 (globals collect), 4 |
| Forward calls + recursion | 3–4 |
| Reserved names | 2 |
| Registry + call stack | 1, 4 |
| Depth 256 → Fault | 4 |
| Fault restarts like Done | 5 |
| Unparse name-sorted functions first | 6 |
| Help docs | 6 |
| Visitor/size/GP exhaustiveness | 1, 6 |
| No new GP mutations | 6 (explicit non-goal) |

## Placeholder scan

No TBD/TODO steps. Exact commands and concrete code included. Field names in sim Done/Fault arm must be copied from the live `Done` arm when editing (names verified as `ProgramStep` in `robominer-program`).
