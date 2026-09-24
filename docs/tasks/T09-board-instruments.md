# T09 — Educational board presentation and instruments

Status: pending

## Dependencies

[T08 — NPN simulation and transistor bench](T08-npn-model.md). Its acceptance criteria must be satisfied before starting this task.

## Outcome and commit boundary

Complete rendering for the supported catalog, connected-hole and pin highlighting, measurement selection, and bounded parameter editing.

Suggested commit title: `feat: add connection highlighting and measurement instruments`.

The deliverable must stand on its own at this milestone. Unfinished successor tasks do not prevent this task from being accepted. Do not include unrelated refactors or publication steps.

## Scope and interfaces

Use derived core topology and readings for UI projection. No free placement, wiring editor, isometric view, or schematic view.

Follow the shared architecture, language, numerical, and persistence contracts in [the plan](../PLAN.md). Repository initialization, remote selection, and publication are not implicit parts of this task.

## Acceptance criteria

- [ ] Every supported catalog item has readable English labels and pin identification.
- [ ] Selecting a contact highlights its actual connected group.
- [ ] Selected voltage/current measurements and graphs show units and computed values.
- [ ] Parameter controls honor documented model bounds and dispatch actions.
- [ ] Overloads, calculation failures, and stale readings are visibly distinguishable.

## Required verification

- Run baseline checks and focused input-to-action/topology-to-highlight tests.
- Inspect all supported parts, measurements, parameter limits, and diagnostics on Linux, Chromium, and Firefox.
- Verify zoom/window resizing does not change electrical state or produce unreadable essential controls.

“Baseline checks” means the actual project commands established in T01 and documented in README, not invented commands or an assumed passing suite. Record commands and evidence below. Required browser/UI checks cannot be replaced by successful compilation. A missing or failing required check blocks readiness.

## Codex Goal

```text
/goal Complete T09 in docs/tasks/T09-board-instruments.md according to docs/PLAN.md
and AGENTS.md. Deliver this outcome: Complete rendering for the supported catalog, connected-hole and pin highlighting, measurement selection, and bounded parameter editing.
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
