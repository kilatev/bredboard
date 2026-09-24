# T03 — Resistive DC circuit simulation

Status: pending

## Dependencies

[T02 — Project JSON and breadboard connectivity](T02-project-topology.md). Its acceptance criteria must be satisfied before starting this task.

## Outcome and commit boundary

Add stable-order MNA for DC sources, resistors, buttons, and switches, usable from a headless verification tool.

Suggested commit title: `feat: solve resistive DC circuits with explicit diagnostics`.

The deliverable must stand on its own at this milestone. Unfinished successor tasks do not prevent this task from being accepted. Do not include unrelated refactors or publication steps.

## Scope and interfaces

Introduce solver readings and typed electrical diagnostics; keep structure validation separate from electrical solvability. No capacitors or nonlinear models yet.

Follow the shared architecture, language, numerical, and persistence contracts in [the plan](../PLAN.md). Repository initialization, remote selection, and publication are not implicit parts of this task.

## Acceptance criteria

- [ ] Series/parallel resistor and divider reference circuits produce analytically correct voltages and currents.
- [ ] Open switches and ideal-source shorts have documented, tested outcomes.
- [ ] Floating circuits and contradictory sources fail explicitly without hanging or presenting invalid readings.
- [ ] Document supported parameter bounds and numerical tolerances.
- [ ] Generated solvable resistor networks satisfy current balance and ordering invariance.

## Required verification

- Run baseline checks and analytical reference tests.
- Exercise CLI diagnostics for floating networks, conflicting sources, and limit boundaries.
- Run property tests on constrained solvable circuits, including current-balance and topology-order invariants.

“Baseline checks” means the actual project commands established in T01 and documented in README, not invented commands or an assumed passing suite. Record commands and evidence below. Required browser/UI checks cannot be replaced by successful compilation. A missing or failing required check blocks readiness.

## Codex Goal

```text
/goal Complete T03 in docs/tasks/T03-dc-solver.md according to docs/PLAN.md
and AGENTS.md. Deliver this outcome: Add stable-order MNA for DC sources, resistors, buttons, and switches, usable from a headless verification tool.
Satisfy every acceptance criterion and run every required check in the card.
Fix task-scoped findings without weakening tests or acceptance criteria.
Do not implement successor tasks. Prepare one coherent change for review;
do not commit or push. Record exact verification evidence in this card.
If a required check is unavailable, report the exact blocker and the input
or environment change needed; do not count it as passed.
```

## Completion and fukit handoff

When all criteria pass, record evidence and set `Status: ready_for_fukit`. Stop without starting the next task. The user may then invoke `fukit` for task-scoped compliance review, code review, required checks, fixes, commit, and targeted push.

An existing jj repository and an unambiguous authorized remote/bookmark are required for that workflow. Do not initialize or guess them. Commit completion is evidenced by jj history; publication is evidenced by the actual push result, not a checkbox pre-written in this change.

## Evidence

Not run yet. Record exact commands or manual procedures, results, environment/browser versions when relevant, and limitations here before marking the task ready.
