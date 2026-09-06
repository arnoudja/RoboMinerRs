# Robot language functions

## Problem

Robot programs have no user-defined functions. Repeated logic must be copy-pasted,
which inflates program size and makes structured control flow harder. Builtins
(`move`, `abs`, `scan`, …) are fixed callables; players cannot define their own.

## Goals

1. Add user-defined functions with call-by-value parameters and optional returns.
2. Support forward references and recursion.
3. Allow functions to read and write top-level program variables.
4. Fit the existing compile → `ExecutableProgram` → frame-stack runner model.
5. Keep existing programs working except for the new reserved words `fn` and `return`.

## Non-goals

- Nested / local function definitions.
- Closures over block-scoped locals (only top-level program variables are shared).
- First-class function values or passing functions as arguments.
- A full static typechecker beyond return-type inference and arity checks.
- New GP mutation operators for functions (only visitor / size / unparse support).

## Decision summary

| Topic | Choice |
|-------|--------|
| Declaration forms | `fn name(...)`, `fn T name(...)`, `T name(...)` — not `T fn name(...)` |
| Placement | Top-level only |
| Parameters | By value; optional type prefix; untyped → arg type per call |
| Return default | Missing / bare `return` / fall-off → `int 0` |
| Omitted return type | Infer iff all valued `return`s agree; else compile error |
| Explicit return type | Coerce return values (same as assign) |
| Outer variables | Read/write all top-level program vars (any source order) |
| Forward calls / recursion | Yes (hoisted registry) |
| Name conflicts | Reject keywords and builtins |
| Architecture | Function registry on `ExecutableProgram` + call-stack frames |
| Max call depth | 256 → `ProgramStep::Fault` |
| Fault behavior | Restart program like `Done` (sim bridge change for all Faults) |

## Language surface

### Definitions

Valid only as **top-level** statements of the player program (the outermost
sequence after the compiler’s implicit `{ ... }` wrap). Not allowed inside
`if` / `while` / nested `{ }` blocks or inside another function.

```text
fn name(param, ...) { body }
fn int name(param, ...) { body }
int name(param, ...) { body }
```

`int fn name(...)` is a compile error. The same applies with `double` / `float` /
`bool` in place of `int`.

`float` remains an alias of `double` for return and parameter type prefixes.

### Parameters

Comma-separated list of zero or more parameters. Each parameter is either:

- `name` — untyped; each call binds the parameter to that argument’s runtime type
  for that invocation (different call sites may pass different types).
- `T name` — typed (`int` / `double` / `float` / `bool`); the argument is coerced
  to `T` on entry (same coercion rules as variable assign).

Parameters are **passed by value**. Assigning to a parameter inside the function
does not change the caller’s bindings.

Arity must match at every call site (compile error on mismatch).

### Body and `return`

The body is a normal statement sequence and may use actions, control flow,
locals, and calls (including recursion).

- `return <expr>;` — exit with that value.
- `return;` — exit with `int 0`.
- Falling off the end of the body — same as `return;` → `int 0`.
- `return` outside a function — compile error.

### Return typing

**Explicit** return type (`fn T name` or `T name`):

- Every valued `return` is coerced to `T`.
- Bare / implicit return yields `int 0` coerced to `T`.

**Omitted** return type (`fn name`):

- If there is at least one valued `return`, all valued returns must share one
  concrete type (`Bool` / `Int` / `Float`); otherwise compile error.
- If there are no valued returns, the function’s return type is `int`.

### Calls

`name(arg0, arg1, ...)` is an expression (usable as a statement via the existing
expression-statement form). Arguments are evaluated left to right before the
call, consistent with other call-like builtins.

### Scoping

Inside a function:

- Parameters and locals declared in the function (and its nested blocks) are
  visible per normal block scope rules.
- All **top-level** program variables are readable and writable, including those
  declared textually after the function definition.
- Locals/parameters do not leak to the caller or to other functions.

### Reserved names

A function name must not be any of:

**Keywords / declarators:** `fn`, `return`, `if`, `else`, `while`, `do`, `const`,
`int`, `double`, `float`, `bool`, `true`, `false`

**Actions / queries:** `move`, `rotate`, `mine`, `dump`, `dumpA`, `dumpB`,
`dumpC`, `scan`, `time`, `ore`, `oreDistance`, `oreType`

**Math:** `abs`, `sqrt`, `sin`, `cos`, `tan`, `min`, `max`

**Objects:** `robot`, `area`

Duplicate function names → compile error. A function must not reuse a top-level
variable name and vice versa (same global namespace for top-level bindings).

## Architecture

```text
Source
  → two-pass compile
  → ExecutableProgram { statements, functions, actions, requires_runtime }
  → ExecutableRunner (statement frames + call frames + shared globals)
  → ProgramStep { Cpu | Action | Done | Fault }
  → sim program_bridge (Done and Fault both restart runner)
```

### AST

Add:

- `ExecutableFunction { name, return_type: Option<ValueType> /* None = infer then store resolved */, params: Vec<FunctionParam>, body: Vec<ExecutableStatement> }`
- `FunctionParam { name, value_type: Option<ValueType> }`
- `ExecutableStatementKind::Return(Option<ExecutableExpression>)`
- `ExecutableExpressionKind::Call { name: String, args: Vec<ExecutableExpression> }`

`ExecutableProgram` gains `functions: BTreeMap<String, ExecutableFunction>` (or
equivalent ordered map). Parsed function definitions are **removed** from the
main `statements` list used as the entry script.

Store the **resolved** return `ValueType` on each function after inference so the
runner does not re-infer.

### Compile (two-pass)

1. **Collect:** parse the outer sequence; split top-level items into main
   statements vs function definitions. Reject nested function definitions and
   `T fn name` forms. Register each function signature (name, params, optional
   explicit return type). Collect top-level variable declaration names/types
   into a program-global symbol table. Enforce reserved names and uniqueness
   against other functions and top-level variables.

2. **Fill:** parse/validate each function body with scope =
   program-globals + parameters; allow `return`; resolve `Call` against the
   full function registry (forward references OK). Parse/validate the main
   statement list the same way (calls allowed; `return` forbidden). Check call
   arity. Infer omitted return types from valued `return` expressions; on
   conflict, compile error.

Implementation may use a dedicated scan of already-built body ASTs for return
inference rather than a third full parse, as long as results match the rules
above.

### Runner

- Keep the existing statement frame stack.
- Add call frames (or tagged frames) that:
  - push a new variable scope for parameters/locals;
  - leave top-level globals in the outermost scope shared across calls;
  - record where the expression evaluator should receive the return value.
- On `Call`: evaluate args → push call frame → bind params → run body statements.
- On `Return` or end of body: coerce/produce return value → pop call frame →
  resume the caller’s expression evaluation or statement.
- Max **call depth 256** (nested user calls). Exceeding it yields
  `ProgramStep::Fault` via the existing fault path.

Program restart (entry) clears call state the same way it clears statement
frames today.

### Fault = restart

Today `ProgramStep::Fault` in `robominer-sim` halts the robot without restarting
(`Wait` forever for that program). Change that so **Fault restarts like Done**:

- Reset the runner with the same path as `Done` (`program.runner()`).
- Clear pending action handshake / motion as needed for a clean restart.
- Apply the same CPU-budget charge used on `Done` so fault loops cannot spin
  forever.

Update the `ProgramStep::Fault` docs: Fault remains “runner could not continue
this activation,” but callers **must restart** rather than permanently halt.
Call-depth overflow uses this path; other invariant faults do too.

### Program size, unparse, GP

- Function bodies and `Call` / `Return` nodes count toward program size like
  other AST nodes.
- Unparse emits all function definitions first (name-sorted, matching
  `BTreeMap` iteration), then the main `statements` list.
- Update `ast_visit` / GP walkers to visit `functions` so size and transforms do
  not ignore them. No new mutation operators in this change.

## Help text

Extend `robominer-web/static/help/robot_program.html` with a functions section
covering declaration forms, parameters, return typing, scoping, reserved names,
and recursion.

## Testing

### `robominer-program`

- Accept `fn f()`, `fn int f()`, `int f()`; reject `int fn f()` and nested defs.
- Typed/untyped params; by-value isolation; arity mismatch errors.
- Return: valued, bare, fall-through → 0; inference agreement vs conflict;
  explicit-type coercion.
- Calls as expression and statement; forward reference; recursion.
- Outer top-level variable read/write from a function (including var declared
  after the function in source).
- Reserved names rejected; duplicate function / clash with top-level var.
- Call depth > 256 → `Fault`.

### `robominer-sim`

- `Fault` restarts the program (assert runner reset / continued execution after
  a fault), covering the new call-depth case and preserving the same restart
  policy for other Faults.

## Compatibility

- Existing programs that never used `fn` / `return` as identifiers keep working.
- `fn` and `return` become reserved words (breaking for any program that used
  them as variable names).
