# T07 — Calculated LED circuit and current-driven light

Status: ready_for_fukit

## Dependencies

[T06 — Buildable RC bench and shared app controls](T06-rc-bench.md). Its acceptance criteria must be satisfied before starting this task.

## Outcome and commit boundary

Add a bounded smooth-diode LED model and a buildable LED/resistor/button fixture. Show calculated current and drive visible LED brightness from it.

Suggested commit title: `feat: simulate LED current and show current-driven light`.

The deliverable must stand on its own at this milestone. Unfinished successor tasks do not prevent this task from being accepted. Do not include unrelated refactors or publication steps.

## Scope and interfaces

Use the existing catalog parameters and common solver. Document the model equation, iteration bound, and limits. The Bevy controls and visuals are shared across build targets; Linux is the only interaction test target.

Follow the shared architecture, language, numerical, and persistence contracts in [the plan](../PLAN.md). Repository initialization, remote selection, and publication are not implicit parts of this task.

## Acceptance criteria

- [x] LEDs work in arbitrary structurally valid supported assemblies, not only the built-in fixture.
- [x] Forward/reverse cases and resistor changes give the expected calculated currents.
- [x] Brightness is a presentation mapping of current; the off and on states are visibly distinct.
- [x] Nonconvergence terminates within a documented bound and marks readings stale.
- [x] The embedded fixture has unique occupied holes and produces a measurable on current when pressed.

### Amendment 2026-09-25 — breadboard-scale parameter ranges

The documented ranges admitted valid projects the LED solver cannot solve (1000 V, 0.001 Ω), so "supported" and "converges" disagreed. Bredboard targets hobby breadboards powered by batteries or USB, so narrow the supported ranges to that scale and make convergence a guarantee inside them.

- [x] `parameter_range` in `crates/core/src/lib.rs` and the table in `docs/PROJECT-FORMAT.md` use: `dc_voltage_source.voltage` 0 to 12 V; `resistor.resistance` 1 to 10,000,000 Ω; `led.series_resistance` 1 to 10,000,000 Ω. Other ranges are unchanged by this amendment.
- [x] Out-of-range values are rejected by validation with the existing clear parameter diagnostic, before solving. All three built-in fixtures remain valid without edits.
- [x] A fixed-seed property test samples LED/resistor/button assemblies across the full new ranges, including the worst case (12 V, 1 Ω resistor, 1 Ω LED series resistance, and an LED wired directly across the source), and every case converges within the documented 80-iteration bound.
- [x] The nonconvergence failure path stays covered. If no valid project can reach it, drive it with a test-only lower iteration bound or an internal solver input, not by widening the public ranges. The stop/keep-last-valid/stale behavior and the app warning test are retained.
- [x] The 1000 V boundary evidence below is replaced with evidence for the new ranges. No project format version bump: the format is unpublished and all committed fixtures stay valid. Record this decision in `docs/PROJECT-FORMAT.md`.

## Required verification

- Run baseline checks, nonlinear references, and convergence/failure-bound tests.
- Generate solvable LED/resistor cases and verify current response to resistance and polarity.
- Check LED brightness and readable failure warnings in the Linux executable. Do not test browser interaction.

“Baseline checks” means the actual project commands established in T01 and documented in README, not invented commands or an assumed passing suite. Record commands and evidence below. Required Linux UI checks cannot be replaced by successful compilation. A missing or failing required check blocks readiness.

## Codex Goal

```text
/goal Complete T07 in docs/tasks/T07-led-model.md according to docs/PLAN.md
and AGENTS.md. Deliver a calculated LED circuit with current-driven light and a buildable board fixture.
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

- Baseline checks passed exactly as documented in `README.md`: `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo fmt --all --check`; `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo clippy --workspace --all-targets --locked -- -D warnings`; and `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo test --workspace --locked` (6 app tests and 34 core tests passed).
- Locked builds passed: `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo build -p bredboard-app --target x86_64-unknown-linux-gnu --locked` and `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo build -p bredboard-app --target wasm32-unknown-unknown --locked`. No browser interaction test was run, as required by this card.
- `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo run -q -p bredboard-tools --locked -- validate fixtures/projects/led-bench.json`; `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo run -q -p bredboard-tools --locked -- validate fixtures/projects/rc-bench.json`; and `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo run -q -p bredboard-tools --locked -- validate fixtures/projects/transistor-bench.json` passed with respectively 4 components/2 wires/4 nodes, 5/6/5, and 7/7/6. The app placement test confirms unique occupied holes for all embedded fixtures.
- `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo run -q -p bredboard-tools --locked -- solve fixtures/projects/led-bench.json` reports released LED current `0.000000000 A`; the pressed temporary case `/tmp/bredboard-led-on.json` reports `0.008576130 A` (8.576130 mA). The fixed-seed LED tests (`0xDC03_2026`, 96 cases) cover current response to resistance, polarity, and the full logarithmic 1–10,000,000 Ω ranges; the explicit 12 V / 1 Ω / 1 Ω series and LED-direct-across-source cases converge.
- `enforces_breadboard_scale_voltage_and_resistance_ranges` verifies the existing `parameter_out_of_range` diagnostic for values just outside all three amended ranges before solving. `docs/PROJECT-FORMAT.md` records the battery/USB breadboard rationale and explicitly records that the unpublished format version remains unchanged.
- The nonconvergence regression now uses the crate-internal test iteration limit `0`, because valid public-range LED projects are required to converge. It confirms no step advancement, last-valid-result retention, stale readings, and the `nonconvergence` diagnostic. The app test `failure_state_marks_visible_readings_stale` retains checks for visible `CALCULATION FAILED` and `Readings stale` text.
- Linux UI check: `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo run -p bredboard-app --locked` initially failed in the sandbox with `WaylandError(Connection(NoCompositor))`; the same command with desktop access succeeded on Linux (Omarchy 4.0.4, Intel Iris Xe, Mesa 26.2.2, Vulkan) and created a visible `bredboard` window. The existing owner-reported click-through on 2026-09-25 exercised the menu, three boards, controls, calculated LED glow, and failure warning with no reported issue. Browser interaction was not tested.
