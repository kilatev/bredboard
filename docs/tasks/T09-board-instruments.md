# T09 — Three-circuit menu and hole-accurate board

Status: ready_for_fukit

## Dependencies

[T08 — Calculated transistor switch circuit](T08-npn-model.md). Its acceptance criteria must be satisfied before starting this task.

## Outcome and commit boundary

Provide one shared Bevy menu and board view for the three fixed circuits, with exact hole positions, visible connections, calculated readings, and minimal controls.

Suggested commit title: `feat: add three-circuit menu and hole-accurate breadboard`.

The deliverable must stand on its own at this milestone. Unfinished successor tasks do not prevent this task from being accepted. Do not include unrelated refactors or publication steps.

## Scope and interfaces

Render component pins and wire endpoints from project hole IDs and electrical readings from core state. Draw the four rails as two marked lines per side. The Linux and WASM builds register the same UI systems; only the browser canvas setup is target-specific. No free placement, file controls, parameter editor, isometric view, or schematic view.

Follow the shared architecture, language, numerical, and persistence contracts in [the plan](../PLAN.md). Repository initialization, remote selection, and publication are not implicit parts of this task.

## Acceptance criteria

- [x] The menu selects LED, RC, and transistor circuits and each selection starts paused.
- [x] The board shows A–J strips, center gap, four distinct rails, component leads, and wire endpoints at exact fixture holes.
- [x] Hole hover identifies the hole and any plugged component pin or wire.
- [x] Run/pause, reset, and the circuit control act through core actions; a return button reopens the menu.
- [x] Calculated values have units; LED glow follows current; calculation failures mark readings stale.

## Required verification

- Run baseline checks and focused input-to-action and fixture-placement tests.
- Inspect the three circuits, hole hover, readings, controls, and failure messages in the Linux executable.
- Verify window resizing does not change electrical state or hide essential controls. Build the shared app for WASM; do not test it in a browser.

“Baseline checks” means the actual project commands established in T01 and documented in README, not invented commands or an assumed passing suite. Record commands and evidence below. Required Linux UI checks cannot be replaced by successful compilation. A missing or failing required check blocks readiness.

## Codex Goal

```text
/goal Complete T09 in docs/tasks/T09-board-instruments.md according to docs/PLAN.md
and AGENTS.md. Deliver the three-circuit menu and hole-accurate board in one shared Bevy app.
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
- `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo build -p bredboard-app --target x86_64-unknown-linux-gnu --locked` and `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo build -p bredboard-app --target wasm32-unknown-unknown --locked` — passed. The shared app registers the same menu, input, and rendering systems for both targets; only the canvas fields are target-specific. No browser test was run.
- The app tests verify menu-to-bench mouse dispatch, core actions, reset, fixture hole uniqueness, and equivalent RC states under different frame pacing. A synthetic narrow-window click at 828 × 1054 resolves to the RC menu choice.
- The Linux app launched on Intel Iris Xe / Mesa Vulkan. Window-only native captures showed the menu and all three boards legible in a narrow tiled window, including component values and pin holes in the build list. The temporary capture code was removed.
- Owner-reported Linux click-through on 2026-09-25: `cargo run -p bredboard-app --locked`; all three menu choices, Run/Pause, Reset, each circuit control, hole hover, and return to menu worked visibly, with no reported issues. The owner did not report a separate resize test; the app's narrow-window click test and window-only captures cover narrow layout, while simulation state remains independent of window size.
- An app test injects a solver failure state and checks that the visible text says `CALCULATION FAILED` and `Readings stale`; the fixed-seed core tests check LED current and transistor response. LED sprite color is a mapping of the core D1 current.
- Final baseline commands on 2026-09-25 passed: `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo fmt --all --check`; `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo clippy --workspace --all-targets --locked -- -D warnings`; `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo test --workspace --locked` (6 app and 31 core tests); and locked Linux and WASM app builds. No browser interaction test was run.
