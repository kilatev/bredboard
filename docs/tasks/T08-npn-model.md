# T08 — NPN simulation and transistor bench

Status: pending

## Dependencies

[T07 — LED simulation and overload warnings](T07-led-model.md). Its acceptance criteria must be satisfied before starting this task.

## Outcome and commit boundary

Add a simplified Ebers-Moll NPN model without parasitic capacitances and a reference button-controlled LED-load assembly.

Suggested commit title: `feat: simulate NPN transistor switching`.

The deliverable must stand on its own at this milestone. Unfinished successor tasks do not prevent this task from being accepted. Do not include unrelated refactors or publication steps.

## Scope and interfaces

Extend model/catalog/schema contracts with documented NPN pin order, parameters, ratings, and model version. Reuse the existing nonlinear solver and core actions.

Follow the shared architecture, language, numerical, and persistence contracts in [the plan](../PLAN.md). Repository initialization, remote selection, and publication are not implicit parts of this task.

## Acceptance criteria

- [ ] Off, active, and saturation cases are covered by reference circuits.
- [ ] Button press/release changes calculated load current and visible output.
- [ ] Equivalent-model ngspice comparisons document fixture provenance and numerical tolerance.
- [ ] Nonconvergence and invalid parameters remain bounded and explicit.
- [ ] NPN snapshots restore model internals correctly and continue equivalently.

## Required verification

- Run baseline checks, NPN reference and snapshot-continuation tests, and equivalent ngspice comparisons.
- Verify current balance on a constrained generated NPN circuit family.
- Exercise button switching, readings, and diagnostics on Linux and both browsers.

“Baseline checks” means the actual project commands established in T01 and documented in README, not invented commands or an assumed passing suite. Record commands and evidence below. Required browser/UI checks cannot be replaced by successful compilation. A missing or failing required check blocks readiness.

## Codex Goal

```text
/goal Complete T08 in docs/tasks/T08-npn-model.md according to docs/PLAN.md
and AGENTS.md. Deliver this outcome: Add a simplified Ebers-Moll NPN model without parasitic capacitances and a reference button-controlled LED-load assembly.
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
