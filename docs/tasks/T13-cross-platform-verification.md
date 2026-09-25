# T13 — Integrated Linux and core verification

Status: ready_for_fukit

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

- [x] Native replay produces deterministic discrete core states for all three fixed circuits.
- [x] Native voltage/current traces meet the documented fixture tolerances.
- [x] Generated action/snapshot/replay sequences reproduce from logged seeds and retain minimized regression examples.
- [x] Representative malformed, floating, contradictory, and nonconvergent cases terminate with expected diagnostics.
- [x] The documented verification command fails when a comparison fails and records platform/tool versions.

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

Implementation and verification evidence on 2026-09-25:

- Added `cargo run -p bredboard-tools --locked -- verify-native`, which executes deterministic native transcripts for the LED, RC, and transistor fixtures, checks the documented LED/transistor ranges and 1% RC tolerance, replays each action log, and rejects malformed, floating, and contradictory fixtures. It prints OS/architecture, Rust version, core model/solver versions, and seed `0xA013_2026` with 32 property cases.
- Added `generated_action_logs_replay_exactly` with fixed seed `0xA013_2026` and 32 cases. Existing seeded property tests cover the solver, simulation, snapshots, and malformed/nonconvergent diagnostics; no minimized failures were produced. The intentionally mismatched comparison is covered by `intentionally_mismatched_fixture_is_detected` and is not retained as a defect.
- Baseline checks passed: `cargo fmt --all --check`; `cargo clippy --workspace --all-targets --locked -- -D warnings`; and `cargo test --workspace --locked` (18 app, 38 core, and 2 tools tests passed).
- Target builds passed: `cargo build -p bredboard-app --target x86_64-unknown-linux-gnu --locked` and `cargo build -p bredboard-app --target wasm32-unknown-unknown --locked`. Browser interaction was not run; T13 adds no browser behavior and does not claim Linux UI release acceptance.
