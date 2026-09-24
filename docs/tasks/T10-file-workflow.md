# T10 — Complete project import and export experience

Status: pending

## Dependencies

[T09 — Educational board presentation and instruments](T09-board-instruments.md). Its acceptance criteria must be satisfied before starting this task.

## Outcome and commit boundary

Finish user-facing project/snapshot file workflows, schema authoring guidance, and useful validation feedback for externally authored assemblies.

Suggested commit title: `feat: support validated custom assembly import and export`.

The deliverable must stand on its own at this milestone. Unfinished successor tasks do not prevent this task from being accepted. Do not include unrelated refactors or publication steps.

## Scope and interfaces

Extend existing T06 adapters rather than adding competing persistence paths. Separate structural file rejection from solver diagnostics for valid but unsolvable benches.

Follow the shared architecture, language, numerical, and persistence contracts in [the plan](../PLAN.md). Repository initialization, remote selection, and publication are not implicit parts of this task.

## Acceptance criteria

- [ ] Custom supported assemblies load as unguided benches and can be exported.
- [ ] Snapshots restore the ongoing simulation through the user interface.
- [ ] Malformed references, unsupported versions/models, scope limits, and invalid parameters produce understandable English errors.
- [ ] Failed structural imports leave the currently open project unchanged.
- [ ] Valid but unsolvable imports display electrical diagnostics instead of silently fabricating results.

## Required verification

- Run baseline checks and integration tests covering accepted/rejected import fixtures.
- Perform native and Chromium/Firefox open/save/download/reopen round trips.
- Check boundary limits and corruption cases and compare the preserved active state after failed imports.

“Baseline checks” means the actual project commands established in T01 and documented in README, not invented commands or an assumed passing suite. Record commands and evidence below. Required browser/UI checks cannot be replaced by successful compilation. A missing or failing required check blocks readiness.

## Codex Goal

```text
/goal Complete T10 in docs/tasks/T10-file-workflow.md according to docs/PLAN.md
and AGENTS.md. Deliver this outcome: Finish user-facing project/snapshot file workflows, schema authoring guidance, and useful validation feedback for externally authored assemblies.
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
