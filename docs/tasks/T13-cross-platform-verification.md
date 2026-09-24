# T13 — Integrated Linux and core verification

Status: pending

## Dependencies

[T15 — Breadboard-scale capacitor and transistor ranges](T15-capacitor-npn-ranges.md). Its acceptance criteria must be satisfied before starting this task.

## Outcome and commit boundary

Add an automated runner for the three fixed circuit/action transcripts on native builds, plus integrated property regression coverage.

Suggested commit title: `test: verify native simulation replay and regressions`.

The deliverable must stand on its own at this milestone. Unfinished successor tasks do not prevent this task from being accepted. Do not include unrelated refactors or publication steps.

## Scope and interfaces

Reuse core fixtures and action formats. This task expands earlier feature tests; it is not permission to postpone those tests until now.

Follow the shared architecture, language, numerical, and persistence contracts in [the plan](../PLAN.md). Repository initialization, remote selection, and publication are not implicit parts of this task.

## Acceptance criteria

- [ ] Native replay produces deterministic discrete core states for all three fixed circuits.
- [ ] Native voltage/current traces meet the documented fixture tolerances.
- [ ] Generated action/snapshot/replay sequences reproduce from logged seeds and retain minimized regression examples.
- [ ] Representative malformed, floating, contradictory, and nonconvergent cases terminate with expected diagnostics.
- [ ] The documented verification command fails when a comparison fails and records platform/tool versions.

## Required verification

- Run baseline checks and the new native replay runner.
- Run property suites with recorded seeds/case counts, including snapshot and replay sequences.
- Demonstrate that an intentionally mismatched test fixture is detected, without retaining the intentional defect.

“Baseline checks” means the actual project commands established in T01 and documented in README, not invented commands or an assumed passing suite. Record commands and evidence below. Required Linux UI checks cannot be replaced by successful compilation. A missing or failing required check blocks readiness.

## Codex Goal

```text
/goal Complete T13 in docs/tasks/T13-cross-platform-verification.md according to docs/PLAN.md
and AGENTS.md. Deliver an automated native runner for the three fixed circuits and integrated property regression coverage. Do not run browser tests.
Satisfy every acceptance criterion and run every required check in the card.
Fix task-scoped findings without weakening tests or acceptance criteria.
Do not implement successor tasks. Prepare one coherent change for review;
do not commit or push. Record exact verification evidence in this card.
If a required check is unavailable, report the exact blocker and the input
or environment change needed; do not count it as passed.
```

## Completion and fukit handoff

When all criteria pass, record evidence and set `Status: ready_for_fukit`. Stop without starting the next task. The user may then invoke `fukit` to describe, commit, and push.

An existing jj repository and an unambiguous authorized remote/bookmark are required for that workflow. Do not initialize or guess them. Commit completion is evidenced by jj history; publication is evidenced by the actual push result, not a checkbox pre-written in this change.

## Evidence

Not run yet. Record exact commands or manual procedures, results, environment/browser versions when relevant, and limitations here before marking the task ready.
