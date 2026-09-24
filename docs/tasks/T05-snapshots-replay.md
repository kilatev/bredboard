# T05 — Simulation snapshots and action replay

Status: pending

## Dependencies

[T04 — RC simulation and deterministic actions](T04-rc-actions.md). Its acceptance criteria must be satisfied before starting this task.

## Outcome and commit boundary

Add versioned snapshots and action logs including all core state needed to continue an RC experiment.

Suggested commit title: `feat: restore snapshots and replay simulation actions`.

The deliverable must stand on its own at this milestone. Unfinished successor tasks do not prevent this task from being accepted. Do not include unrelated refactors or publication steps.

## Scope and interfaces

Persistence is pure serialization/validation in the core with file I/O in tools/adapters. Include model/solver versions, project, step count, and internal component state.

Follow the shared architecture, language, numerical, and persistence contracts in [the plan](../PLAN.md). Repository initialization, remote selection, and publication are not implicit parts of this task.

## Acceptance criteria

- [ ] A saved/restored run agrees with an uninterrupted run under the same later actions.
- [ ] Action logs identify an initial state and record ordered step boundaries.
- [ ] Unsupported versions and corrupted snapshots fail without replacing active state.
- [ ] Project and snapshot schemas have English descriptions and complete examples.
- [ ] Derived topology is rebuilt and validated rather than trusted as an authoritative cache.

## Required verification

- Run baseline checks and snapshot round-trip/continuation tests.
- Use generated valid action sequences and random save boundaries to compare uninterrupted and restored runs.
- Exercise malformed and version-incompatible files through the headless tool.

“Baseline checks” means the actual project commands established in T01 and documented in README, not invented commands or an assumed passing suite. Record commands and evidence below. Required browser/UI checks cannot be replaced by successful compilation. A missing or failing required check blocks readiness.

## Codex Goal

```text
/goal Complete T05 in docs/tasks/T05-snapshots-replay.md according to docs/PLAN.md
and AGENTS.md. Deliver this outcome: Add versioned snapshots and action logs including all core state needed to continue an RC experiment.
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
