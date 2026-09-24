# T06 — End-to-end RC bench on Linux and web

Status: pending

## Dependencies

[T05 — Simulation snapshots and action replay](T05-snapshots-replay.md). Its acceptance criteria must be satisfied before starting this task.

## Outcome and commit boundary

Render an RC assembly with its switch, voltage graph, and time controls; connect the UI to the core action interface and provide minimal project/snapshot load/save.

Suggested commit title: `feat: expose an interactive RC bench in Bevy`.

The deliverable must stand on its own at this milestone. Unfinished successor tasks do not prevent this task from being accepted. Do not include unrelated refactors or publication steps.

## Scope and interfaces

Add ECS-to-domain ID mapping and native/browser file adapters. Render from core state; keep hover/cursor/animation state outside the simulation snapshot.

Follow the shared architecture, language, numerical, and persistence contracts in [the plan](../PLAN.md). Repository initialization, remote selection, and publication are not implicit parts of this task.

## Acceptance criteria

- [ ] The same RC bench operates on Linux, Chromium, and Firefox.
- [ ] Run/pause/step/reset, switch changes, and graph readings reflect calculated state.
- [ ] Project and snapshot files load/save through native file access and browser selection/download.
- [ ] Different frame pacing does not alter fixed-step results; the app does not discard calculation steps.
- [ ] Measure the RC accuracy and web feasibility checkpoint; resolve failures before T07.

## Required verification

- Run baseline checks and adapter/action integration tests.
- Exercise all bench controls plus save/restore on Linux and both browsers; record concrete outcomes.
- Compare a UI-driven RC trace with the headless reference and inspect graph, units, stale-state display, and English labels.

“Baseline checks” means the actual project commands established in T01 and documented in README, not invented commands or an assumed passing suite. Record commands and evidence below. Required browser/UI checks cannot be replaced by successful compilation. A missing or failing required check blocks readiness.

## Codex Goal

```text
/goal Complete T06 in docs/tasks/T06-rc-bench.md according to docs/PLAN.md
and AGENTS.md. Deliver this outcome: Render an RC assembly with its switch, voltage graph, and time controls; connect the UI to the core action interface and provide minimal project/snapshot load/save.
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
