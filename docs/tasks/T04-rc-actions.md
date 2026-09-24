# T04 — RC simulation and deterministic actions

Status: pending

## Dependencies

[T03 — Resistive DC circuit simulation](T03-dc-solver.md). Its acceptance criteria must be satisfied before starting this task.

## Outcome and commit boundary

Add capacitor state, Backward Euler integration at 100 microseconds, and ordered actions for parameters, switches, and time controls.

Suggested commit title: `feat: advance RC circuits through deterministic actions`.

The deliverable must stand on its own at this milestone. Unfinished successor tasks do not prevent this task from being accepted. Do not include unrelated refactors or publication steps.

## Scope and interfaces

Introduce SimulationState and action reduction; expose headless stepping with integer step counts. No snapshots or Bevy bench yet.

Follow the shared architecture, language, numerical, and persistence contracts in [the plan](../PLAN.md). Repository initialization, remote selection, and publication are not implicit parts of this task.

## Acceptance criteria

- [ ] Run, pause, single-step, and reset have explicit tested semantics; actions apply between steps.
- [ ] RC charging/discharging reference samples stay within 1% of source voltage.
- [ ] Simulation advancement depends on requested step count, not elapsed wall time or rendering FPS.
- [ ] Reset restores documented project initial conditions; failed calculation preserves the last valid state and reports stale readings.

## Required verification

- Run baseline checks and analytical RC references.
- Replay equivalent action schedules under different step batching patterns and compare states.
- Run generated valid RC/action sequences and check deterministic reduction and reset behavior.

“Baseline checks” means the actual project commands established in T01 and documented in README, not invented commands or an assumed passing suite. Record commands and evidence below. Required browser/UI checks cannot be replaced by successful compilation. A missing or failing required check blocks readiness.

## Codex Goal

```text
/goal Complete T04 in docs/tasks/T04-rc-actions.md according to docs/PLAN.md
and AGENTS.md. Deliver this outcome: Add capacitor state, Backward Euler integration at 100 microseconds, and ordered actions for parameters, switches, and time controls.
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
