//! # Program ↔ simulation pending-action protocol
//!
//! Move, rotate, and scan coordinate program execution with the mining-cycle
//! simulator. Move/rotate use paired pending state across crate boundaries; scan
//! uses robot scan state on the simulation side plus runner expression suspend/resume.
//!
//! | Layer | Type | Owner | Role |
//! |---|---|---|---|
//! | Program | `ExecutableRunner::pending_physical` | `robominer-program` | Hold the logical move/rotate and **defer advancing until the sim reports completion**. |
//! | Simulation | `PendingExpressionAction` | `robominer-sim` | Split one logical move/rotate into **per-cycle speed chunks** and accumulate distance/angle traveled. |
//! | Simulation | `Robot::scan_state` | `robominer-sim` | Track in-progress or completed scan; feed `ExecutionContext` scan fields. |
//!
//! Do not merge move/rotate pending structs with scan state: the runner tracks
//! *where* execution is in source code; the simulator tracks *how much* physical
//! work remains and whether a scan has finished.
//!
//! ## Execution path
//!
//! [`robominer_sim::ScriptedRobot::from_executable_program`] always installs
//! `ActionSource::Program` (live runner). Static expansion of literal action lists
//! is a mapper/test helper only and must not drive player or rally robots: it drops
//! scan/control-flow and cannot feed action results back into expressions.
//!
//! ## End-to-end cycle flow
//!
//! ```text
//! Mining cycle N
//! ──────────────
//! 1. Simulation::next_robot_action
//!      └─ if pending_expression_actions[i] is Some → emit next speed chunk (skip program)
//!      └─ else run_program_cpu_loop → ExecutableRunner::step
//!
//! 2. Runner emits `ProgramStep::Action(Move(total))` once
//!      └─ `start_pending_physical`: `awaits_action_result = true`, `pending_physical` set
//!      └─ statement index NOT advanced yet for chunked actions
//!
//! 3. Simulation::start_expression_action
//!      └─ pending_expression_actions[i] = Move { remaining: total, accumulated: 0 }
//!      └─ first RobotAction chunk executed; walls/collisions applied
//!
//! 4. Simulation::record_action_result
//!      └─ partial chunk: action_results[i] = None, sim pending kept
//!      └─ final chunk:   action_results[i] = Some(accumulated), sim pending cleared
//!
//! Mining cycle N+1 (partial move still running)
//! ─────────────────────────────────────────────
//! 5. next_robot_action serves the next chunk from pending_expression_actions
//!    without calling the runner (program counter unchanged, `pending_physical` kept)
//!
//! Mining cycle M (move finished)
//! ──────────────────────────────
//! 6. next_robot_action → run_program_cpu_loop
//! 7. build_execution_context copies action_results[i] into ExecutionContext::action_result
//! 8. Runner::step → `handle_continue_physical`
//!      └─ requires `pending_physical` Some AND `action_result` Some
//!      └─ clears `pending_physical`, advances frame index or pushes expression result
//! 9. Runner continues same cycle (CPU budget) or waits next cycle for next statement
//! ```
//!
//! ## Move/rotate initiation (unified)
//!
//! All move/rotate paths call `PendingPhysicalAction::start` with a
//! `PhysicalCompletion` and resume through `PendingPhysicalAction::continue_action`:
//!
//! | Source | Example | Completion |
//! |---|---|---|
//! | Statement | `move(4);` | `PhysicalCompletion::Statement` |
//! | Dynamic statement | `move(robot.yPos);` | `PhysicalCompletion::Statement` |
//! | Expression | `if (move(2) >= 1)` | `PhysicalCompletion::Expression` |
//!
//! Literal moves in expressions (`move(1.5)`) and dynamic moves (`move(x)`) both use
//! the expression completion path via `step_expression_move_or_rotate`.
//!
//! ## Scan coordination
//!
//! `scan()` compiles to an expression statement. The runner emits
//! `ProgramStep::Action(StartScan(direction))`, queues `pending_action`, and waits
//! for `action_result` to receive the robot's `scan_time` before advancing. Reads
//! such as `oreDistance()` and `oreType()` use separate expression work items that
//! consult `ExecutionContext` scan fields built from `Robot::scan_snapshot()`.
//!
//! ### Simulation scan state
//!
//! | `ScanState` | `scan_started` | `scan_complete` | Meaning |
//! |---|---|---|---|
//! | `Idle` | false | false | No scan this rally step chain yet |
//! | `Scanning { direction, cycles_remaining }` | true | false | Scan initiated; counting down |
//! | `Complete(result)` | true | true | Ray-march done; distance/ore_type available |
//!
//! - **`start_scan`** — called on `StartScan`; sets `Scanning` with
//!   `cycles_remaining = robot.spec.scan_time`, increments scan action counter,
//!   sets `action_results[i] = Some(scan_time)`.
//! - **`tick_scan`** — called on every `ProgramStep::Cpu` in the CPU loop; decrements
//!   `cycles_remaining` and ray-marches when it reaches zero.
//! - **`complete_scan_now`** — called on `AwaitScanResult`; ray-marches immediately
//!   and sets `Complete`, charging `cpu_used += cycles_remaining.max(1)`.
//!
//! Passive countdown (`tick_scan` on CPU steps) and active wait (`AwaitScanResult`)
//! can both finish a scan. Programs that call `oreDistance()` while a scan is still
//! `Scanning` take the active path so the read does not return stale data.
//!
//! ### Scan end-to-end flow
//!
//! ```text
//! scan();                          // expression statement
//! ────────────────────────────────
//! 1. Runner emits StartScan(dir)
//! 2. Sim start_scan → Scanning, action_results = Some(scan_time)
//! 3. Runner re-emits until action_result consumed, then pushes scan_time as value
//!
//! oreDistance();                   // while Scanning
//! ────────────────────────────────
//! 4. Runner sees !scan_complete → Action(AwaitScanResult)
//! 5. Sim complete_scan_now → Complete, cpu_used += remaining cycles
//! 6. Runner continues eval; context.scan_complete true → pushes distance
//!
//! oreDistance();                   // with no prior scan
//! ────────────────────────────────
//! → pushes -1.0 without emitting an action (scan_started false)
//! ```
//!
//! ### CPU budget and cross-cycle scan results
//!
//! Each mining cycle runs at most `robot.spec.cpu_speed` program instructions unless
//! the CPU loop sets `extend_budget`:
//!
//! ```text
//! extend_budget =
//!     runner.awaits_scan_result()
//!     || runner.has_pending_scan_completion()
//! ```
//!
//! - **`awaits_scan_result`** — `pending_action == AwaitScanResult` after step (4).
//! - **`has_pending_scan_completion`** — ongoing expression eval is at
//!   `PushOreDistance` or `PushOreType` (about to read scan results).
//!
//! Extend the budget while a read is pending so `oreDistance()` / `oreType()` can
//! finish in the same mining cycle even after `AwaitScanResult` sets
//! `ScanState::Complete` (extend must not require `Scanning`; completion happens
//! before the read consumes context).
//!
//! When `cpu_speed` is exhausted mid-`scan()`, the runner keeps
//! `pending_action == StartScan` and the sim keeps `action_results = Some(scan_time)`.
//! **`should_preserve_program_action_result`** prevents `process_robot_action(Wait)`
//! from clearing that result so the next mining cycle can consume it and advance
//! past the `scan()` statement. The same preservation applies while
//! `awaits_scan_result()` is true.
//!
//! ## Await kinds
//!
//! Every emitted [`ExecutableAction`] has an [`crate::ActionAwaitKind`] from
//! [`crate::await_kind`]:
//!
//! | Kind | Examples | Awaits `action_result`? |
//! |---|---|---|
//! | `None` | `move(0)`, `AwaitScanResult`, statement side effects | No |
//! | `Scalar` | expression `mine()` / `dump(n)` | Yes (one cycle) |
//! | `Motion` | chunked `move` / `rotate` | Yes (via pending physical + sim chunks) |
//! | `ScanStart` | `scan()` | Result written in the CPU loop, not via Wait |
//!
//! **Invariant:** never set `awaits_action_result` / `action_result_expected` for a
//! Wait-mapped action (`ActionAwaitKind::None` for motion amounts within
//! [`crate::motion::MOTION_EPSILON`]). The sim maps those to `Wait`, which produces
//! `ActionResult::None` and would otherwise livelock the runner.
//!
//! If sim pending motion still exists when a cycle yields `Wait` (for example zero
//! engine speed with remaining distance), `record_action_result` force-completes the
//! pending action with the accumulated travel so the runner can resume.
//!
//! ## Invariants
//!
//! - While `pending_expression_actions[i]` is `Some`, the simulation must **not** call
//!   `ExecutableRunner::step` for that robot; chunk delivery is sim-driven.
//! - While `ExecutableRunner::pending_physical` is `Some` for a chunked move/rotate, the
//!   runner must **not** advance past that statement until `action_result` is `Some`.
//! - `move(0)` / `rotate(0)` (and amounts within [`crate::motion::MOTION_EPSILON`]) are not
//!   chunked: expression forms complete immediately with result `0` (no pending), and dynamic
//!   statements advance the frame when emitting them. The sim maps zero motion to `Wait` and
//!   does not return an action result.
//! - `ExecutionContext::action_result` is `None` between partial sim chunks so the
//!   runner does not treat an incomplete move as finished.
//! - `ExecutableRunner::step` clears `awaits_action_result` at entry; it is set again
//!   only when `start_pending_physical` or `queue_pending_action` runs for the newly
//!   emitted action.
//! - While `pending_action == StartScan` or `AwaitScanResult`, the simulation must
//!   **not** clear `action_results[i]` on `RobotAction::Wait` if a scan result is
//!   still to be delivered to the runner.
//! - `ExecutionContext::scan_*` fields come from `build_execution_context` /
//!   `scan_snapshot` and must reflect `ScanState` before each `runner.step` call.
//!
//! ## Return values and wall clipping
//!
//! Each sim chunk reports actual distance/rotation via `action_results`. Chunk
//! completion is computed by [`motion::record_motion_step`]:
//!
//! - [`MotionStepOutcome::Continue`] — full chunk traveled and more distance remains.
//! - [`MotionStepOutcome::Complete`] — final chunk; remaining distance is zero.
//! - [`MotionStepOutcome::Blocked`] — travel fell short of the requested chunk
//!   (for example a wall); the whole pending action ends and the accumulated total
//!   becomes the expression return value for path (3). Paths (1) and (2) ignore it
//!   when advancing the frame.
//!
//! ## Related code
//!
//! - Runner: `pending_await`, `pending_physical_action`, `start_pending_physical`, `handle_continue_physical`
//! - Runner scan eval: `expression_eval::step` (`PushStartScan`, `PushOreDistance`, `PushOreType`)
//! - Motion chunking: [`motion`]
//! - Sim bridge: `run_program_cpu_loop`, `start_scan`, `tick_scan`, `complete_scan_now`,
//!   `build_execution_context`
//! - Sim pending move/rotate: `pending_expression_actions`, `record_action_result`,
//!   `should_preserve_program_action_result`, `next_robot_action`
