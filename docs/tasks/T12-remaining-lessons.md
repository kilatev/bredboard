# T12 — Guided RC and transistor experiments

Status: pending

## Dependencies

[T11 — Lesson engine and guided LED experiment](T11-led-lesson.md). Its acceptance criteria must be satisfied before starting this task.

## Outcome and commit boundary

Add English RC charge/discharge and transistor-switch lessons using the existing lesson engine and solver.

Suggested commit title: `feat: add capacitor and transistor lessons`.

The deliverable must stand on its own at this milestone. Unfinished successor tasks do not prevent this task from being accepted. Do not include unrelated refactors or publication steps.

## Scope and interfaces

Add built-in content/fixtures and reference conditions, not a second lesson engine or externally executable lesson scripts.

Follow the shared architecture, language, numerical, and persistence contracts in [the plan](../PLAN.md). Repository initialization, remote selection, and publication are not implicit parts of this task.

## Acceptance criteria

- [ ] RC lesson requires voltage thresholds in both charge and discharge directions.
- [ ] Transistor lesson requires measured load-current on and off ranges after button actions.
- [ ] All thresholds and fixture parameters are explicit and boundary-tested.
- [ ] Snapshots resume each lesson at an intermediate step without losing relevant progress.
- [ ] Both lessons provide short instructions, visual hints, and beginner explanations.

## Required verification

- Run baseline checks and complete headless action transcripts for both lessons.
- Test partial completion, incorrect ordering, reset, and snapshot continuation.
- Complete both lessons on Linux, Chromium, and Firefox and inspect English content and graph feedback.

“Baseline checks” means the actual project commands established in T01 and documented in README, not invented commands or an assumed passing suite. Record commands and evidence below. Required browser/UI checks cannot be replaced by successful compilation. A missing or failing required check blocks readiness.

## Codex Goal

```text
/goal Complete T12 in docs/tasks/T12-remaining-lessons.md according to docs/PLAN.md
and AGENTS.md. Deliver this outcome: Add English RC charge/discharge and transistor-switch lessons using the existing lesson engine and solver.
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
