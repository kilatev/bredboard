# T11 — Lesson engine and guided LED experiment

Status: pending

## Dependencies

[T10 — Complete project import and export experience](T10-file-workflow.md). Its acceptance criteria must be satisfied before starting this task.

## Outcome and commit boundary

Add core-owned lesson progress and condition evaluation, with English instructions/hints and a complete LED/resistor lesson.

Suggested commit title: `feat: teach LED current control with a guided lesson`.

The deliverable must stand on its own at this milestone. Unfinished successor tasks do not prevent this task from being accepted. Do not include unrelated refactors or publication steps.

## Scope and interfaces

Define built-in LessonState and declarative internal conditions tied to simulation readings and steps. Include progress in snapshots; external lesson import remains excluded.

Follow the shared architecture, language, numerical, and persistence contracts in [the plan](../PLAN.md). Repository initialization, remote selection, and publication are not implicit parts of this task.

## Acceptance criteria

- [ ] The learner enables power and increases resistance; completion uses calculated current decrease.
- [ ] Instructions, hint highlighting, and explanations are understandable to a beginner.
- [ ] Document explicit fixture parameters and condition thresholds; test boundary and false-positive cases.
- [ ] Pause/reset and snapshot restore preserve their documented lesson semantics.
- [ ] Unguided imported assemblies are not evaluated against built-in lesson assumptions.

## Required verification

- Run baseline checks and headless lesson progression, threshold, reset, and snapshot tests.
- Complete the lesson through the UI on Linux and both browsers, including an incorrect action and recovery.
- Inspect English text placement and hint association with actual board components.

“Baseline checks” means the actual project commands established in T01 and documented in README, not invented commands or an assumed passing suite. Record commands and evidence below. Required browser/UI checks cannot be replaced by successful compilation. A missing or failing required check blocks readiness.

## Codex Goal

```text
/goal Complete T11 in docs/tasks/T11-led-lesson.md according to docs/PLAN.md
and AGENTS.md. Deliver this outcome: Add core-owned lesson progress and condition evaluation, with English instructions/hints and a complete LED/resistor lesson.
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
