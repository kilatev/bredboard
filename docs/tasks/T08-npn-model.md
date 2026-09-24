# T08 — Calculated transistor switch circuit

Status: ready_for_fukit

## Dependencies

[T07 — Calculated LED circuit and current-driven light](T07-led-model.md). Its acceptance criteria must be satisfied before starting this task.

## Outcome and commit boundary

Add a base-controlled NPN approximation and a buildable button-controlled LED-load assembly.

Suggested commit title: `feat: simulate NPN transistor switching`.

The deliverable must stand on its own at this milestone. Unfinished successor tasks do not prevent this task from being accepted. Do not include unrelated refactors or publication steps.

## Scope and interfaces

Use the existing NPN pin order and parameters. Document the base-junction and collector-conductance equations and their limits. Reuse the common solver and core actions. Do not script the LED outcome.

Follow the shared architecture, language, numerical, and persistence contracts in [the plan](../PLAN.md). Repository initialization, remote selection, and publication are not implicit parts of this task.

## Acceptance criteria

- [x] Off and on cases are covered by calculated reference circuits.
- [x] Button press/release changes calculated load current and visible output.
- [x] The fixture has unique occupied holes, including three adjacent transistor lead holes.
- [x] Nonconvergence and invalid parameters remain bounded and explicit.
- [x] NPN snapshots restore model internals correctly and continue equivalently.

## Required verification

- Run baseline checks, NPN reference and snapshot-continuation tests.
- Verify current response over a constrained generated resistor/beta family.
- Exercise button switching, readings, and diagnostics in the Linux executable. Do not test browser interaction.

“Baseline checks” means the actual project commands established in T01 and documented in README, not invented commands or an assumed passing suite. Record commands and evidence below. Required Linux UI checks cannot be replaced by successful compilation. A missing or failing required check blocks readiness.

## Codex Goal

```text
/goal Complete T08 in docs/tasks/T08-npn-model.md according to docs/PLAN.md
and AGENTS.md. Deliver a calculated transistor switch circuit with a buildable board fixture.
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

Implementation and automated evidence on 2026-09-25:

- `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo fmt --all --check`, `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo clippy --workspace --all-targets --locked -- -D warnings`, and `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo test --workspace --locked` — passed (5 app and 29 core tests).
- `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo build -p bredboard-app --target x86_64-unknown-linux-gnu --locked` and `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo build -p bredboard-app --target wasm32-unknown-unknown --locked` — passed. No browser test was run.
- `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo run -q -p bredboard-tools --locked -- validate fixtures/projects/transistor-bench.json` — accepted 7 components, 7 wires, 6 nodes. App placement test confirms unique occupied holes and adjacent Q1 pins F12/F13/F14.
- `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo run -q -p bredboard-tools --locked -- solve /tmp/bredboard-transistor-on.json` — B1 pressed gives 8.461094 mA through D1 and Q1 collector, 0.426794 mA in the base-feed resistor, and 0.040341 V collector-emitter. The temporary input changed only the built-in button state; the unmodified fixture has under 1 µA LED current. Fixed-seed Proptest (`0xDC03_2026`, 96 cases) checks beta response and current balance. The snapshot continuation test passed.
- A Linux window-only capture showed the transistor, leads, wires, and build list. Owner-reported Linux click-through on 2026-09-25 found the three circuits and their controls worked visibly with no reported issue. The B1 action changes calculated D1 and Q1 currents in the core regression tests, and `update_view` maps D1 current to LED sprite color.
- Invalid parameter edits are atomically rejected by the core topology validation. Nonconvergence is limited to 80 solver iterations; the regression test checks that failure stops stepping and marks readings stale, while the app test checks the diagnostic display.
- Final baseline commands on 2026-09-25 passed: `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo fmt --all --check`; `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo clippy --workspace --all-targets --locked -- -D warnings`; `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo test --workspace --locked` (6 app and 31 core tests); and locked Linux and WASM app builds. No browser interaction test was run.
