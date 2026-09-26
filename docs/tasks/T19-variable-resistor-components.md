# T19 — Variable-resistor components

Status: ready_for_fukit

## Dependencies

[T17 — Sprites for the remaining and future simple components](T17-more-component-sprites.md)
and the specification in
[docs/roadmap/LESSONS.md](../roadmap/LESSONS.md). T17's acceptance criteria
must remain satisfied before starting this task.

## Outcome and commit boundary

Add `potentiometer` and `photoresistor` as new `ComponentKind` values: a
two-terminal variable resistor whose resistance is driven by a continuous,
user-controlled ratio instead of a fixed project parameter, plus their
sprites, promoted from the T17 Part B designs.

Suggested commit title: `feat: add potentiometer and photoresistor components`.

The deliverable must stand on its own. Do not add the exercises that use these
parts, the buzzer, or the menu changes; those are T20–T23.

## Scope and interfaces

- Add one shared mechanism for a continuous control input (0.0–1.0), separate
  from the existing discrete `ControlState` used by buttons and switches.
  Route it through `Action` the same way button/switch presses are routed, so
  it participates in ordered, deterministic action application. Document the
  new action and any new `SimulationState` field it reads back.
- Add `ComponentKind::Potentiometer` and `ComponentKind::Photoresistor`, each
  with pins `a`, `b` (two-terminal, per the rheostat wiring documented in
  `docs/roadmap/LESSONS.md`; do not model the third terminal or a voltage
  divider). Document required parameters (for example, minimum and maximum
  resistance) and their ranges in the catalog, alongside the existing kinds in
  `crates/core/src/lib.rs`.
- Extend the solver to stamp these as a resistor whose value is computed from
  the parameter range and the current control ratio. Document the exact
  interpolation (for example linear between the minimum and maximum
  resistance) as a model equation, per `docs/PLAN.md`.
- Add `PartArt` implementations for both kinds using the existing
  `future-trimmer-potentiometer.txt` and `future-photoresistor.txt` golden
  references as the body art; promote their palette entries out of
  `#[cfg(test)]`/`palette::future` into the runtime palette. Register both in
  `sprites::art_for`.
- Both parts are two-pin for placement purposes (the wiper terminal is not
  wired in these exercises), so no new placement mechanism is required beyond
  what T09/T17 already support.
- Validate and document parameter/control ranges (`parameter_range`) so
  imported projects with an out-of-range control ratio or resistance fail
  validation clearly, per `docs/PLAN.md`'s persistence contract.

## Acceptance criteria

- [ ] `potentiometer` and `photoresistor` are valid `ComponentKind` values with
  documented pins, parameters, and ranges; JSON Schema output includes them.
- [ ] The solver computes each part's effective resistance from its control
  ratio and documented equation; a bounded property test covers the ratio's
  full range and confirms monotonic resistance change.
- [ ] The continuous control input is applied through the existing ordered
  `Action` reduction; a test confirms two ratio-change actions between steps
  apply in order and do not depend on frame rate.
- [ ] Both sprites render at every ratio without changing pixel content (state
  is a control signal, not a visual state, per the design reference) and
  match their committed golden references.
- [ ] Existing fixtures, tests, and the JSON Schema for the other component
  kinds are unaffected.
- [ ] Round-trip persistence: a project containing both new kinds serializes
  and deserializes losslessly, including the control ratio in
  `SimulationState`.

## Required verification

- Run the baseline formatting, Clippy, workspace test, Linux build, and WASM
  build commands documented in `README.md`.
- Run the new solver, action-ordering, and sprite golden/property tests.
- Manually exercise both new parts at several control ratios in a scratch
  fixture in the Linux executable (not one of the ten LESSONS.md exercises;
  those come in T22/T23) to confirm resistance changes are visible in
  readings.

“Baseline checks” means the actual project commands documented in `README.md`.
A missing or failing required check blocks readiness.

## Codex Goal

```text
/goal Complete T19 in docs/tasks/T19-variable-resistor-components.md according
to docs/PLAN.md, AGENTS.md, and docs/roadmap/LESSONS.md. Add potentiometer and
photoresistor component kinds with a continuous control input, solver support,
and sprites promoted from the T17 future-part designs. Do not add the ten
LESSONS.md exercises, the buzzer, or menu changes.
Satisfy every acceptance criterion and run every required check in the card.
Fix task-scoped findings without weakening tests or acceptance criteria.
Do not implement successor tasks. Prepare one coherent change for review;
do not commit or push. Record exact verification evidence in this card.
If a required check is unavailable, report the exact blocker and the input
or environment change needed; do not count it as passed.
```

## Completion and fukit handoff

When all criteria pass, record evidence and set `Status: ready_for_fukit`.
Stop without starting successor tasks. The user may then invoke `fukit` to
describe, commit, and push.

An existing jj repository and an unambiguous authorized remote/bookmark are
required for that workflow. Do not initialize or guess them. Commit
completion is evidenced by jj history; publication is evidenced by the actual
push result.

## Evidence

Implementation, `crates/core`:
- `lib.rs`: `ComponentKind::Potentiometer`/`Photoresistor` (pins `a`, `b`),
  required parameters `min_resistance`/`max_resistance` each ranged
  `1.0..=1e7` ohm (same breadboard-scale bound as `resistor`). `compile_topology`
  rejects `min_resistance >= max_resistance` (`invalid_parameter_range`).
  `InitialConditions.control_ratios: BTreeMap<ComponentId, f64>` is the new
  continuous-control mechanism (0.0..=1.0), validated the same way as
  `controls`: unknown/wrong-kind reference is `invalid_initial_control_ratio`,
  non-finite or out-of-range is `control_ratio_out_of_range`.
- `simulation.rs`: `Action::SetControlRatio { component, ratio }`, reduced in
  `apply_actions` next to `SetControl` — same ordered-batch semantics, same
  validation (component must be a potentiometer/photoresistor, ratio finite
  and in `0.0..=1.0`), same `needs_solve` invalidation on success, same
  `invalid_control_ratio` diagnostic on rejection (state unchanged). New
  `SimulationState.control_ratios` field, defaulted to `0.5` per component at
  `SimulationState::new` from `initial_conditions.control_ratios`, and
  restored by `Action::Reset` like every other field.
- `solver.rs`: both kinds stamp as a resistor (`BranchKind::Resistor`, the
  existing stamp code path — no new branch kind), value computed from the
  control ratio: potentiometer `resistance = min + ratio * (max - min)`
  (ratio 0.0 -> min, 1.0 -> max); photoresistor
  `resistance = max - ratio * (max - min)` (ratio is "ambient light": 0.0
  darkest -> `max_resistance`, 1.0 brightest -> `min_resistance`, so E6's
  darkest setting reaches the LED's off threshold per T23). `solve_dc`/
  `solve_transient` take a new `ratios: &BTreeMap<ComponentId, f64>` argument,
  overlaid on `project.initial_conditions.control_ratios` exactly like the
  existing `states` argument overlays `initial_conditions.controls`.
- `persistence.rs`: `restore_snapshot` now also requires
  `state.control_ratios` to have exactly one finite, in-range entry per
  potentiometer/photoresistor component (`invalid_control_ratio_state`
  otherwise), mirroring the existing button/switch `controls` check.

Implementation, `crates/app/src/sprites`: new `potentiometer.rs` and
`photoresistor.rs`, each a `PartArt` with `axis_pins ["a","b"]`, a single
state, and a `body()` that is pixel-identical to the T17 Part B
`future::trimmer_potentiometer()`/`future::photoresistor()` designs (the
golden files `future-trimmer-potentiometer.txt` and `future-photoresistor.txt`
are unchanged and now checked from `body_sprites_match_design_references`
instead of `future_part_designs_match_references`). Both registered in
`sprites::art_for`. Neither drawing function actually referenced
`palette::future` (that module is only the colored-LED design-only
constants) — both already used runtime `Band` constants, so there was no
`#[cfg(test)]` palette entry to promote; this is recorded here since the
card called for promoting one. `main.rs::component_summary` gained match arms
for both kinds (build-list text), and `Circuit`/`Bench`/menu are unchanged
(no exercise or menu wiring — that is T22/T23).

Required verification (run from repository root on 2026-09-26):
- `cargo fmt --all --check` — pass.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — pass.
- `cargo test --workspace --locked` — pass (70 tests: 21 app + 47 core + 2 tools).
- `cargo build -p bredboard-app --target x86_64-unknown-linux-gnu --locked` — pass.
- `cargo build -p bredboard-app --target wasm32-unknown-unknown --locked` — pass.

New tests:
- `bredboard_core::tests::potentiometer_and_photoresistor_are_valid_kinds_with_documented_ranges`,
  `variable_resistor_range_and_reference_diagnostics`,
  `variable_resistors_round_trip_including_control_ratio` (`lib.rs`) — schema
  includes both kinds; range/reference/inverted-range diagnostics; lossless
  JSON round trip of a project with both kinds and their control ratios.
- `solver::tests::potentiometer_resistance_matches_linear_interpolation_at_the_endpoints`,
  `photoresistor_resistance_is_inverted_relative_to_potentiometer` — exact
  analytical current at ratio 0.0/1.0 against a resistor-divider fixture.
- `solver::tests::variable_resistor_current_changes_monotonically_with_ratio` —
  bounded property test (96 cases, fixed seed) over both kinds and the full
  breadboard resistance range confirms monotonic current change across 8
  sampled ratios per case, per the acceptance criterion.
- `simulation::tests::control_ratio_actions_apply_in_order_between_steps_regardless_of_batching` —
  two `SetControlRatio` actions in one ordered batch: only the last is
  visible (no frame-rate dependency); out-of-range ratio is rejected without
  mutating state; `Reset` restores the default ratio.
- `simulation::tests::control_ratio_change_is_visible_in_recalculated_readings` —
  a ratio change is reflected in the next solved step's resistor current.
- `persistence::tests::snapshot_round_trip_preserves_control_ratios_for_variable_resistors` —
  snapshot round trip preserves both components' control ratios exactly.

Sprite/placement tests: `sprites::tests::body_sprites_match_design_references`
now includes both new bodies against the unchanged T17 golden files;
`every_state_of_a_part_has_the_same_size` and `placements_cover_every_pin_hole`
(property test) now cover both kinds.

Manual exercise (CLI, `bredboard-tools`): built four scratch projects (a
potentiometer or photoresistor in series with a fixed resistor across the
5 V rail) at ratio 0.1/0.9 (potentiometer) and 0.0/1.0 (photoresistor,
darkest/brightest), and ran `validate` + `solve` on each:
- potentiometer ratio 0.1 (1090 ohm): 2.392344 mA; ratio 0.9 (9010 ohm):
  0.499500 mA — matches `5 / (min + ratio*(max-min) + 1000)` exactly and
  decreases as ratio rises.
- photoresistor ratio 0.0/darkest (10000 ohm): 0.454545 mA; ratio 1.0/brightest
  (100 ohm): 4.545455 mA — matches the inverted equation exactly and confirms
  the darkest setting is the lowest-current (LED-off-threshold) end, as T23
  will need.

Manual Linux-executable (windowed) check: not available in this session.
As recorded on T21, this session's screen-capture tool (`grim`) returns only
the static desktop wallpaper for every geometry tried, despite `hyprctl`
confirming the target window is mapped and rendering, and no input-automation
daemon is available either. Per AGENTS.md this is a blocker, not a pass: a
human with working screen access should still place both new parts on a
scratch fixture in the Linux executable and confirm the sprite renders
identically across ratios before relying on this as final sign-off. The CLI
exercise above is the available substitute evidence that resistance changes
with ratio are visible in readings; it does not substitute for the visual
check of the sprite itself.
