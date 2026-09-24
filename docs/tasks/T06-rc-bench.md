# T06 — Buildable RC bench and shared app controls

Status: ready_for_fukit

## Dependencies

[T05 — Simulation snapshots and action replay](T05-snapshots-replay.md). Its acceptance criteria must be satisfied before starting this task.

## Outcome and commit boundary

Render a physically buildable RC assembly from its project hole IDs and connect shared Bevy controls to core actions.

Suggested commit title: `feat: expose an interactive RC bench in Bevy`.

The deliverable must stand on its own at this milestone. Unfinished successor tasks do not prevent this task from being accepted. Do not include unrelated refactors or publication steps.

## Scope and interfaces

Render four distinct power rails, A–E and F–J strips, the center gap, component pins, and wire endpoints at their actual project holes. The run/pause, reset, and switch controls must be shared by Linux and WASM builds. Keep hover and visual state outside the core snapshot. No user-facing file controls or browser-specific UI.

Follow the shared architecture, language, numerical, and persistence contracts in [the plan](../PLAN.md). Repository initialization, remote selection, and publication are not implicit parts of this task.

## Acceptance criteria

- [x] The RC bench opens from the shared menu and operates through visible controls in the Linux executable.
- [x] Run/pause, reset, and switch changes reflect calculated capacitor voltage.
- [x] The RC fixture has individually occupied holes and visible connections that match its project JSON.
- [x] Different frame pacing does not alter fixed-step results; the app does not discard calculation steps.
- [x] Measure RC accuracy and confirm the same app compiles for WASM; resolve Linux feasibility failures before T07.

## Required verification

- Run baseline checks and adapter/action integration tests.
- Exercise selection, run/pause, reset, and switch in the Linux window; record concrete outcomes.
- Compare a UI-driven RC trace with the headless reference and inspect board hole positions, units, stale-state display, and English labels. Do not test browser interaction.

“Baseline checks” means the actual project commands established in T01 and documented in README, not invented commands or an assumed passing suite. Record commands and evidence below. Required Linux UI checks cannot be replaced by successful compilation. A missing or failing required check blocks readiness.

## Codex Goal

```text
/goal Complete T06 in docs/tasks/T06-rc-bench.md according to docs/PLAN.md
and AGENTS.md. Deliver a buildable RC board with shared Bevy run/pause, reset, and switch controls driven by the core.
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

Automated checks passed on 2026-09-24 (Rust 1.95.0):

- `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo fmt --all --check` — passed.
- `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo clippy --workspace --all-targets --locked -- -D warnings` — passed.
- `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo test --workspace --locked` — passed (25 tests, including the app RC fixture test).
- `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo build -p bredboard-app --target x86_64-unknown-linux-gnu --locked` — passed.
- `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo build -p bredboard-app --target wasm32-unknown-unknown --locked` — passed.
- `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo run -q -p bredboard-tools -- simulate fixtures/projects/rc-bench.json 6240` — 4.990220277 V at 0.624 s. The Chrome bench showed 4.990 V at 0.624 s for the same trace.
- The app-level fixture test checks project-derived component labels, atomic rejection of a bad project version, charge/discharge through actual core actions, paused single-step graph sampling and 100 µs time display, 1% analytical RC accuracy, and snapshot continuation.

Manual UI checks:

- Linux: `cargo run -p bredboard-app --locked` opened the Bevy window successfully on Linux (Omarchy 4.0.4, Intel Iris Xe / Mesa Vulkan). Native window interaction remains unverified because native UI automation is unavailable; headless app tests exercise the actual mouse handler, visible button entities, and project/snapshot file round trips.
- Chrome only, per owner instruction (Firefox intentionally not tested): served `web/` at `http://127.0.0.1:8000/`, packaged with `wasm-bindgen-cli 0.2.128`, and opened the WASM app without startup console errors. Run, Pause, Step, Reset, and Switch operated; the voltage/time readout and graph followed calculated charge/discharge traces. A paused single step displayed `0.005 V | 0.0001 s`. Component labels and switch status identify their project IDs; graph labels show 0–5 V across 0–1 s. Save project and Save snapshot each reported a successful JSON download. System Chromium is `152.0.7977.82`.
- Chrome file import is blocked by the browser extension: `fileChooser.setFiles` returned `Not allowed` while testing Load project. Neither project nor snapshot loading could be verified in the browser. The owner cannot enable the extension's file-URL access and directed that this check be deferred to the next phase.

Scope correction on 2026-09-24: the owner clarified that the Linux executable is the primary product and must contain every control. Browser interaction and file adapters are deferred to the post-MVP web application. The earlier Chrome observations above are historical evidence from the previous scope, not T06 acceptance evidence. The web file toolbar and WASM file bridge were removed; all interactive controls are now native-only, including visible Load/Save Project and Load/Save Snapshot buttons in the Linux Bevy window. File buttons use fixed names in the process working directory for this minimal milestone; T10 completes the workflow.

After the correction:

- `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo fmt --all --check` — passed.
- `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo clippy --workspace --all-targets --locked -- -D warnings` — passed.
- `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo test --workspace --locked` — passed (26 tests, including two app tests).
- `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo test -p bredboard-app --locked` — passed after native-only control gating (2 tests). The new test checks that all four file buttons are spawned and that native project and snapshot file round trips restore state.
- `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo build -p bredboard-app --target x86_64-unknown-linux-gnu --locked` — passed after native-only control gating.
- `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo build -p bredboard-app --target wasm32-unknown-unknown --locked` — passed without warnings after native-only control gating.
- `git diff --check` — passed.

Scope updated on 2026-09-25 for the single-app MVP: the earlier browser observations, graph, step button, and file-button checks above are historical evidence only. The replacement RC fixture uses unique occupied holes and the app draws its wires and components from project hole IDs. No browser interaction is an MVP check. A sandboxed Linux launch initially failed to connect to Wayland (`NoCompositor`); a desktop-access launch later succeeded.

On 2026-09-25, a regression test initially showed that Bevy's default 250 ms virtual-time cap reduced a one-second frame to 2,500 of the expected 10,000 fixed steps. The app now disables that cap. `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo test -p bredboard-app --locked different_frame_pacing_preserves_fixed_simulation_results` passes: a one-second frame and 100/400/500 ms frames produce identical 10,000-step simulation states. The T06 WASM/page artifacts remain historical and are not browser acceptance evidence.

Current verification after the clock fix (2026-09-25):

- `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo fmt --all --check` — passed.
- `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo clippy --workspace --all-targets --locked -- -D warnings` — passed.
- `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo test --workspace --locked` — passed (28 tests).
- `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo test -p bredboard-app --locked linux_mouse_controls_dispatch_visible_bench_actions` — passed; synthetic window clicks exercise Run, Pause, Step, Switch, and Reset through the native mouse handler.
- `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo build -p bredboard-app --target x86_64-unknown-linux-gnu --locked` — passed.
- `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo build -p bredboard-app --target wasm32-unknown-unknown --locked` — passed.
- `git diff --check` — passed.

Current shared-app verification on 2026-09-25 (supersedes the older UI and pacing scope above):

- `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo fmt --all --check` — passed.
- `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo clippy --workspace --all-targets --locked -- -D warnings` — passed.
- `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo test --workspace --locked` — passed (5 app and 29 core tests). The app tests cover RC charge/discharge through the fixture, unique lead/wire holes, synthetic mouse controls, and equivalent 1,000-step results for one 1 s frame versus 100/400/500 ms frames.
- `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo build -p bredboard-app --target x86_64-unknown-linux-gnu --locked` — passed.
- `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo build -p bredboard-app --target wasm32-unknown-unknown --locked` — passed; compile check only, with no browser test.
- `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo run -q -p bredboard-tools --locked -- validate fixtures/projects/rc-bench.json` — accepted 5 components, 6 wires, 5 nodes.
- `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo run -q -p bredboard-tools --locked -- simulate fixtures/projects/rc-bench.json 1000` — 3.159683479 V at 0.100000 s, within 1% of the 3.160602794 V analytical value.
- The Linux window opened with Mesa Vulkan. A window-only native capture showed the RC board, distinct rails, center gap, wires, build list, and controls readable in a narrow tiled window. The temporary capture code was removed.
- Owner-reported Linux click-through on 2026-09-25: `cargo run -p bredboard-app --locked`; all three menu choices, Run/Pause, Reset, each circuit control, hole hover, and return to menu worked visibly, with no reported issues. This is user observation rather than an automated assertion.
- The app RC integration test confirms the UI control actions produce 3.159683479 V after 0.1 s of charging, discharge on switch change, and 0 V on reset. The stale-readings app test confirms the explicit failure presentation.
- Final commands on 2026-09-25: `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo fmt --all --check`, `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo clippy --workspace --all-targets --locked -- -D warnings`, `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo test --workspace --locked` (6 app and 31 core tests), `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo build -p bredboard-app --target x86_64-unknown-linux-gnu --locked`, and `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo build -p bredboard-app --target wasm32-unknown-unknown --locked` all passed. No browser interaction test was run.
