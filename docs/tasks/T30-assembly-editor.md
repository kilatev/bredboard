# T30 — Interactive 2D assembly editor

Status: ready_for_fukit

## Owner request

Bring thegame's breadboard assembly interaction to bredboard's existing 2D
presentation. This explicitly authorizes free assembly ahead of the catalog
roadmap; the closed MVP and its fixed exercises remain unchanged.

## Scope

- A blank 5 V bench and an editable copy of a selected single-board fixed exercise.
- Click contacts to place wires and catalog parts, move individual leads,
  select parts, change nominal values, reverse two-terminal polarity and remove
  parts/wires. Escape/right click cancels an unfinished gesture.
- Clear and undo structural edits. No user-facing file workflow in this task.
- Preserve the existing core Project/JSON contracts. Compile each complete
  candidate atomically; core diagnostics remain authoritative for calculation.
- Keep fixed exercises available; editing does not overwrite their fixtures.

## Acceptance and evidence

- Real Linux UI interaction must cover assembly, moving, deleting, cancellation,
  nominal changes and actual calculated LED behavior.
- Reproducible property tests cover rejected edits, unique contacts/IDs and undo.
- Run README's fmt, Clippy, workspace tests, Linux and WASM builds.
- Record actual commands, observed outcomes and remaining limitations below.

## Implementation

The app owns only editing gestures and an undo history (32 Projects). Structural
changes clone the current Project, validate topology and unique physical contact
occupancy, then replace it atomically and request a real core SingleStep.
Rejected edits preserve both the Project and SimulationState. Accepted edits
restart simulation; the latest edited Project is the Reset Time baseline.
Electrical solve failures remain explicit diagnostics, rather than fabricated
readings. Numeric input consumes ordered keyboard messages, including repeated
digits arriving in one rendered frame.

The existing sprites, embedded component templates and core contracts are reused.
No dependency, fixture, schema or core source changes were required. Only the
core crate is optimized in development builds to keep interactive recalculation
fast. Slow-frame wall-time catch-up is capped at 50 ms; fixed electrical steps
remain consecutive, with slower-than-wall-time playback on slow machines.
Inspector text uses finite layout bounds for reliable native rendering.

## Verification environment

Rust 1.95.0; Linux x86_64; Bevy 0.19.1; locked dependencies. Commands used:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build -p bredboard-app --target x86_64-unknown-linux-gnu --locked
cargo build -p bredboard-app --target wasm32-unknown-unknown --locked
git diff --check
```

The managed environment used `CARGO_BUILD_JOBS=2`,
`CARGO_PROFILE_DEV_DEBUG=0` and `CARGO_PROFILE_TEST_DEBUG=0` (debug symbols disabled
for disk space). Native development library metadata and software Vulkan drivers
were unpacked into workspace-local tooling directories; no project dependency or
system package installation was needed. Native interaction uses a real Bevy
window on task-owned Xorg :98 with Mesa llvmpipe/Vulkan and XTest pointer/key events.
WASM runtime interaction remains deferred by PLAN; the required WASM check is
compilation only.

## Results

- Full workspace tests: 232 passed (74 app, 155 core, 3 tools; no doctests).
  This run preceded the final frame-time cap and finite text-bound fixes; focused
  tests for those final changes are recorded separately below.
- Two new property tests use 48 cases each and fixed seed `20261025`: invalid
  candidate atomicity and generated wire/edit/undo sequences.
- Integration tests exercise actual Bevy pointer and ordered keyboard events,
  assembled LED current, nominal edits, invalid values, removal/undo, cancellation,
  clear/undo and removal of a stateful capacitor from an editable RC copy.
- Solver benchmark: 200 validated, calculated lead moves in 123.24 ms in the
  recorded run. This measures CPU editing work, not rendered frame rate.
- The initial unoptimized workspace test run was interrupted; it is not counted
  as a passing run. The complete result above uses the documented core-only
  development optimization.

## Limits

Single-board assembly only. The palette exposes the seven existing component
kinds listed in README; this task does not add every catalog component to the
palette. Multi-board exercises retain their fixed view. No user-facing save,
load or export workflow; assemblies last for the current session. Closing or
leaving the editor discards the current unsaved assembly.

### Final regressions and native interaction

- `cargo test -p bredboard-app slow_frame_bounds --locked`: passed. A simulated
  ten-second frame executes only 50 consecutive core steps; its complete state
  equals explicitly advancing those same 50 steps.
- Final app test binary: all six `assembly::` tests, the actual pointer/keyboard
  integration test and the editable RC-copy regression passed (8 focused tests,
  plus the slow-frame regression above). Both seeded property tests ran again.
- Final Linux build passed. In the real native window, placed a resistor and LED,
  connected both supply wires, entered resistance `1000` and inspected the LED:
  **2.959 mA**, anode **2.04 V**, cathode **0.00 V**.
- Moving the cathode from A11 to A12 produced **0.000 mA** and an unlit LED;
  Undo restored **2.959 mA**. Removing the wire at B5 produced **0.000 mA**;
  Undo restored the circuit. Escape cancelled an unfinished wire from A20.
- Inspector and status text rendered correctly after replacing effectively
  unbounded layout height with finite bounds. Native evidence:
  [assembled circuit and calculated readings](../design/assembly/editor.png).
- The headless environment has no sound device; ALSA warnings were observed.
  Audio output was outside this editor task's verification scope.

### Final check results

- Formatting and `git diff --check`: passed.
- Final workspace/all-targets Clippy with `-D warnings`: passed.
- Linux x86_64 build: passed after all source changes.
- WASM build: passed after all source changes (5 min 11 s including cold
  dependencies). Browser runtime was not exercised and is not claimed.
- Ready status records verification only; publication is reported separately
  from this task card after the actual remote operation.

## Recorded demonstration

[40-second native screen recording](../design/assembly/editor-demo.mp4)
(1200 × 760, H.264 MP4, silent): blank board, resistor and LED placement,
supply wiring, calculated current, resistance entry, moving a lead, removing
a wire and Undo restoring the real circuit. Recorded directly from the app
window; no app implementation changes were needed for recording.
