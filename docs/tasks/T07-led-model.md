# T07 — LED simulation and overload warnings

Status: pending

## Dependencies

[T06 — End-to-end RC bench on Linux and web](T06-rc-bench.md). Its acceptance criteria must be satisfied before starting this task.

## Outcome and commit boundary

Add a bounded iterative nonlinear solution path and diode-based LED model, with rating diagnostics and current-driven brightness in the existing bench UI.

Suggested commit title: `feat: simulate LEDs and report component overloads`.

The deliverable must stand on its own at this milestone. Unfinished successor tasks do not prevent this task from being accepted. Do not include unrelated refactors or publication steps.

## Scope and interfaces

Version the LED model and extend catalog/schema data. Document solver iteration limits, convergence tolerances, parameter ranges, and overload ratings. No damage state.

Follow the shared architecture, language, numerical, and persistence contracts in [the plan](../PLAN.md). Repository initialization, remote selection, and publication are not implicit parts of this task.

## Acceptance criteria

- [ ] LEDs work in arbitrary structurally valid supported assemblies, not only the built-in fixture.
- [ ] Reference forward/reverse operating cases and resistor changes give the expected currents.
- [ ] Brightness is a presentation mapping of current; overload warnings continue a solvable simulation.
- [ ] Nonconvergence terminates within a documented bound and marks readings stale.
- [ ] Equivalent-model ngspice comparisons have documented parameters and tolerances.

## Required verification

- Run baseline checks, nonlinear references, and convergence/failure-bound tests.
- Generate solvable LED/resistor cases and verify current balance and parameter-response expectations.
- Check LED brightness and readable overload/nonconvergence warnings on Linux and both browsers.

“Baseline checks” means the actual project commands established in T01 and documented in README, not invented commands or an assumed passing suite. Record commands and evidence below. Required browser/UI checks cannot be replaced by successful compilation. A missing or failing required check blocks readiness.

## Codex Goal

```text
/goal Complete T07 in docs/tasks/T07-led-model.md according to docs/PLAN.md
and AGENTS.md. Deliver this outcome: Add a bounded iterative nonlinear solution path and diode-based LED model, with rating diagnostics and current-driven brightness in the existing bench UI.
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
