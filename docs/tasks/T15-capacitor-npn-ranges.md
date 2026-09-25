# T15 — Breadboard-scale capacitor and transistor ranges

Status: ready_for_fukit

## Dependencies

[T09 — Three-circuit menu and hole-accurate board](T09-board-instruments.md), including the [T07 range amendment](T07-led-model.md#amendment-2026-09-25--breadboard-scale-parameter-ranges). Their acceptance criteria must be satisfied before starting this task.

## Outcome and commit boundary

Narrow the supported capacitor and NPN parameter ranges to parts found in hobby breadboard kits, and guarantee that valid circuits built from them solve within the documented bounds.

Suggested commit title: `fix: limit capacitor and transistor parameters to breadboard scale`.

The deliverable must stand on its own at this milestone. Unfinished successor tasks do not prevent this task from being accepted. Do not include unrelated refactors or publication steps.

## Scope and interfaces

The current ranges admit values with no breadboard meaning. A 1000 F capacitor stamps a 1e7 S companion conductance at the 100 µs step. A `saturation_current` of 1 A gives a negative base-emitter threshold (`0.026*ln(0.001/Is)` ≈ -0.18 V), so the transistor conducts with no base drive. Change only `parameter_range` in `crates/core/src/lib.rs`, the range table in `docs/PROJECT-FORMAT.md`, and the tests that cover them. Do not change the model equations, the 80-iteration bound, or the ranges settled by the T07 amendment.

Follow the shared architecture, language, numerical, and persistence contracts in [the plan](../PLAN.md). Repository initialization, remote selection, and publication are not implicit parts of this task.

## Acceptance criteria

- [x] Supported ranges are: `capacitor.capacitance` 1e-10 to 1e-2 F (100 pF to 10 mF); `npn_transistor.beta` 10 to 1000; `npn_transistor.saturation_current` 1e-16 to 1e-12 A (base-emitter threshold of about 0.78 V down to 0.54 V). `docs/PROJECT-FORMAT.md` documents each range and the resulting NPN threshold span.
- [x] Out-of-range values are rejected by validation with the existing clear parameter diagnostic, before solving. All built-in fixtures and committed snapshots remain valid without edits.
- [x] A fixed-seed property test samples RC assemblies across the full capacitance range with T07-amendment resistor and voltage ranges. Every step solves, and the capacitor voltage stays between the initial voltage and the source voltage.
- [x] A fixed-seed property test samples transistor-switch assemblies across the full beta and saturation-current ranges. Every case converges within 80 iterations, base current is under 1 µA with the button released, and the load current is higher with the button pressed than released.
- [x] No project format version bump: the format is unpublished and all committed fixtures stay valid. Record this decision in `docs/PROJECT-FORMAT.md`.

## Required verification

- Run baseline checks and the new range-validation and property tests.
- Validate all three built-in fixtures with the tools `validate` command.
- Confirm in the Linux executable that the three circuits still behave as before. Do not test browser interaction.

“Baseline checks” means the actual project commands established in T01 and documented in README, not invented commands or an assumed passing suite. Record commands and evidence below. Required Linux UI checks cannot be replaced by successful compilation. A missing or failing required check blocks readiness.

## Codex Goal

```text
/goal Complete T15 in docs/tasks/T15-capacitor-npn-ranges.md according to docs/PLAN.md
and AGENTS.md. Narrow capacitor and transistor parameter ranges to breadboard scale.
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

Verified on 2026-09-25 with Rust 1.95.0 (`rustc 1.95.0 (59807616e 2026-04-14)`, Cargo 1.95.0). Commands used `mise exec rust@1.95.0 --` because Rust is installed through mise in this environment.

- `mise exec rust@1.95.0 -- cargo fmt --all --check` — passed.
- `mise exec rust@1.95.0 -- cargo clippy --workspace --all-targets --locked -- -D warnings` — passed.
- `mise exec rust@1.95.0 -- cargo test --workspace --locked` — passed: 6 app tests, 37 core tests, tool crate harness, and core doctests; all passed.
- `mise exec rust@1.95.0 -- cargo build -p bredboard-app --target x86_64-unknown-linux-gnu --locked` — passed.
- `mise exec rust@1.95.0 -- cargo build -p bredboard-app --target wasm32-unknown-unknown --locked` — passed; no browser interaction test was run.
- `mise exec rust@1.95.0 -- cargo test -p bredboard-core --locked enforces_breadboard_scale_capacitor_and_npn_ranges` — passed; both accepted endpoints and all six out-of-range boundary cases use `parameter_out_of_range` at the expected parameter path before solving.
- `mise exec rust@1.95.0 -- cargo test -p bredboard-core --locked breadboard_rc_ranges_solve_each_step -- --nocapture` — passed; fixed seed `0xA004_2026`, 48 cases, 16 fixed 100 µs steps per case, voltage 0–12 V, resistor 1–10,000,000 Ω, and capacitance 1e-10–1e-2 F.
- `mise exec rust@1.95.0 -- cargo test -p bredboard-core --locked breadboard_transistor_ranges_converge_and_switch -- --nocapture` — passed; fixed seed `0xDC03_2026`, 96 cases, beta 10–1000, saturation current 1e-16–1e-12 A, released base current under 1 µA, and pressed load current greater than released.
- `mise exec rust@1.95.0 -- cargo run -q -p bredboard-tools --locked -- validate fixtures/projects/led-bench.json` — passed: 4 components, 2 wires, 4 derived nodes.
- `mise exec rust@1.95.0 -- cargo run -q -p bredboard-tools --locked -- validate fixtures/projects/rc-bench.json` — passed: 5 components, 6 wires, 5 derived nodes.
- `mise exec rust@1.95.0 -- cargo run -q -p bredboard-tools --locked -- validate fixtures/projects/transistor-bench.json` — passed: 7 components, 7 wires, 6 derived nodes.
- `mise exec rust@1.95.0 -- cargo run -q -p bredboard-tools --locked -- validate-snapshot fixtures/snapshots/rc-charging-1000.json` — passed: valid snapshot at step 1000.
- `git diff --check` — passed; no fixtures or snapshots were edited.
- Linux executable: `target/x86_64-unknown-linux-gnu/debug/bredboard-app` launched with desktop access and created the `bredboard` Bevy window on Omarchy 4.0.4 / Linux 7.2.5-3-omarchy using Intel Iris Xe, Mesa 26.2.2, Vulkan; the live session was stopped with Ctrl-C after window creation. The T09 owner-reported Linux click-through on 2026-09-25 remains applicable because this change does not touch app/UI code: all three menu choices, Run/Pause, Reset, each circuit control, hole hover, and return to menu worked visibly. The current full workspace tests also pass the shared menu/control, RC, LED, and transistor fixture behavior tests. No browser interaction was run.

The implementation changes only the shared `parameter_range` table, its focused validation/property coverage, and the documented catalog ranges. The project format remains version 1 because it is unpublished and all committed fixtures/snapshots remain valid.
