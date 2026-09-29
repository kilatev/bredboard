# C03 — Counting, display, and systems fixtures

Status: in_progress; the Section 04 and Section 05 source review is recorded
and implementation is proceeding in bounded capability slices. Nine Section 04
and eight Section 05 fixtures are admitted; optional physical validation is
tracked separately.

## Scope

Implement and disposition the 17 schematics in source sections 04 and 05:

- `CAT-S04-01` through `CAT-S04-09`, counting and display circuits;
- `CAT-S05-01` through `CAT-S05-08`, digital systems circuits.

The source records are [`04-level-4-counting-display.md`](../../breadboard-circuits/spec/04-level-4-counting-display.md)
and [`05-level-5-systems.md`](../../breadboard-circuits/spec/05-level-5-systems.md).
SVG references are under `breadboard-circuits/spec/svg/04-level-4-counting-display/`
and `breadboard-circuits/spec/svg/05-level-5-systems/`.

This batch preserves hole-derived topology, fixed-step simulation, stable JSON
contracts, and English player-facing content. It does not authorize free
assembly, a lesson engine, or a hidden script that substitutes for calculated
electrical behavior.

## Source and design review

- S04-01, S04-02, S04-03, and S04-04 need explicit clock/reset contracts,
  counter modulus behavior, and output pin maps. The source uses CD4017,
  CD4026, and 74HC393 packages that are not represented by the current core.
- S04-05 and S05-01/S05-02 need shift-register and arithmetic contracts;
  DIP inputs must be electrical switch states and outputs must be calculated.
- S04-06 is covered by a bounded two-player first-press lockout using the
  edge/reset contract and diode-OR buzzer path. S05-04 needs ordered sequence
  memory and reset behavior; those are covered by a bounded four-stage
  D-flip-flop chain.
- S04-07 is admitted with a bounded bargraph and photoresistor input; the
  source microphone/LM358/LM3914 path remains an explicit discrepancy. S04-08
  is admitted with a bounded gain-transfer amplifier; the source LM386 and
  input-jack contracts remain explicit discrepancies.
- S04-09 reuses 555 timing, while S05-07 is covered by a bounded eight-step
  control-voltage sequencer; speaker frequency is a readout contract, not a
  wall-clock or audio-script result.
- S05-03, S05-05, and S05-06 contain state machines, clock division, or
  crystal assumptions. Educational bounded timing is admissible only when the
  observable state is calculated from pins and fixed simulation steps.
- S05-08 explicitly requires approximately 30 ICs, SRAM, three boards, and a
  5 V / 2 A supply. It is admitted with a bounded two-word, eight-bit SRAM
  contract; the full SAP-1 CPU, board identity, supply limits, and presentation
  surface remain explicit discrepancies.

## Required capability slice

- Add named, validated contracts for a digital counter, an eight-bit shift
  register, and a seven-segment display only where the fixtures require them.
- Keep digital state in the core and advance it by ordered fixed simulation
  steps. Clock, reset, enable, latch, and carry behavior must come from node
  voltages and actions, not circuit IDs or scripted outcome tables.
- Add a bounded arithmetic/decoder contract only if composing existing gates
  would make the fixture less transparent or exceed the core limits.
- Reuse the existing timer, button, switch, LED, resistor, capacitor, and
  potentiometer models where their pin-level behavior is sufficient.
- Add an IC/display sprite and placement checks for every occupied lead hole.
- Record unsupported audio, crystal, full SAP-1 CPU, op-amp, bargraph, physical-prop,
  and multi-board requirements as ledger dispositions instead of faking them.

## Acceptance criteria

- [ ] All 17 source records have reviewed Markdown, BOM, SVG, pin map, supply,
  safety, and source-discrepancy evidence in the ledger.
- [ ] Every admitted BOM item maps to an implemented model or an explicit
  recorded disposition; no scripted electrical result is used.
- [ ] Each admitted fixture validates, derives topology from holes/pins/wires,
  has stable IDs, readable placement, and a reachable menu entry.
- [ ] Counter modulus/carry, register shift/latch, display segment mapping,
  clock/reset behavior, and any arithmetic truth tables have calculated
  regression/property coverage with reproducible seeds where applicable.
- [ ] Unsupported entries have concrete blockers and are not marked ready.
- [ ] Exact formatting, Clippy, workspace tests, Linux/WASM builds,
  catalog/schema validation, and per-fixture validation results are recorded.
- [x] Manual Linux interaction and representative breadboard checks are
  optional follow-up evidence; automated checks remain the acceptance gate.

## Implementation evidence so far

The current slice adds calculated `digital_counter`, `shift_register`,
`seven_segment_display`, `four_bit_adder`, and stateful edge-triggered
`d_flip_flop`, `bargraph_display`, `audio_amplifier`, and `step_sequencer` contracts, nine embedded Section 04 fixtures, seven embedded
Section 05 fixtures, menu entries, and fixed-step regressions for rising-edge
counting, cascaded decimal carry, BCD segment decoding, serial shifting,
output latching, binary addition, ordered code entry, sequencer selection, and
reset. The final Section 05 fixture also has a calculated two-word SRAM write
and read regression.

Section 05 is reconciled in the ledger as eight bounded fixtures. S05-08
records the full SAP-1 CPU, four-board layout, and 5 V / 2 A supply as source
discrepancies rather than hiding them in the electrical result.

```text
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c03-s04-03-button-counter.json — passed; 5 components, 0 wires, 20 derived nodes
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c03-s04-05-shift-register.json — passed; 8 components, 0 wires, 13 derived nodes
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c03-s04-04-binary-counter.json — passed; 12 components, 4 wires, 7 derived nodes
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c03-s04-01-running-lights.json — passed; 24 components, 10 wires, 13 derived nodes
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c03-s04-02-electronic-dice.json — passed; 18 components, 7 wires, 10 derived nodes
cargo test -p bredboard-core c03_dice_counter_calculates_the_one_dot_pattern --locked — passed
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c03-s04-09-dual-555-noisemaker.json — passed; 16 components, 9 wires, 10 derived nodes
cargo run -p bredboard-tools --locked -- simulate fixtures/projects/c03-s04-09-dual-555-noisemaker.json 10 — passed; transient state advanced for C1/C2/C3/C4
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c03-s05-01-four-bit-adder.json — passed; 28 components, 15 wires, 23 derived nodes
cargo test -p bredboard-core c03_four_bit_adder_calculates_five_plus_nine --locked — passed
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c03-s05-02-four-bit-subtractor.json — passed; 28 components, 15 wires, 23 derived nodes
cargo test -p bredboard-core c03_four_bit_subtractor_calculates_nine_minus_five --locked — passed
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c03-s05-03-pedestrian-signal.json — passed; 14 components, 5 wires, 8 derived nodes
cargo test -p bredboard-core c03_pedestrian_signal_advances_calculated_phases --locked — passed
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c03-s04-06-reaction-game.json — passed; 18 components, 5 wires, 12 derived nodes
cargo test -p bredboard-core c03_reaction_game_latches_the_first_player_and_resets_both_outputs --locked — passed
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c03-s04-07-level-indicator.json — passed; 24 components, 10 wires, 23 derived nodes
cargo test -p bredboard-core c03_level_indicator_calculates_segment_count_from_input_ratio --locked — passed
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c03-s04-08-audio-amplifier.json — passed; 10 components, 1 wire, 5 derived nodes
cargo test -p bredboard-core c03_audio_amplifier_transfers_calculated_input_level_to_speaker_load --locked — passed
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c03-s05-04-code-lock.json — passed; 24 components, 6 wires, 17 derived nodes
cargo test -p bredboard-core d_flip_flop_captures_data_only_on_a_calculated_rising_edge --locked — passed
cargo test -p bredboard-core c03_code_lock_requires_ordered_edges_and_calculates_reset_and_unlock_outputs --locked — passed
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c03-s05-05-digital-stopwatch.json — passed; 21 components, 0 wires, 60 derived nodes
cargo test -p bredboard-core c03_stopwatch_cascades_clock_carry_and_reset --locked — passed
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c03-s05-06-digital-clock.json — passed; 15 components, 0 wires, 61 derived nodes
cargo test -p bredboard-core c03_clock_core_cascades_six_calculated_stages --locked — passed
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c03-s05-07-step-sequencer.json — passed; 37 components, 0 wires, 28 derived nodes
cargo test -p bredboard-core c03_step_sequencer_advances_led_and_control_selection --locked — passed
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c03-s05-08-bounded-sram.json — passed; 33 components, 8 wires, 30 derived nodes
cargo test -p bredboard-core c03_bounded_sram_writes_and_reads_a_calculated_byte --locked — passed
cargo test -p bredboard-core digital_counter_advances_once_per_calculated_rising_edge --locked — passed
cargo test -p bredboard-core c03_counter_fixture_drives_a_calculated_display_digit --locked — passed
cargo test -p bredboard-core c03_shift_register_calculates_shift_then_latch --locked — passed
cargo test -p bredboard-app all_embedded_boards_have_unique_lead_and_wire_holes --locked — passed
cargo test -p bredboard-app each_circuit_uses_core_controls_and_reset --locked — passed
cargo fmt --all --check — passed after `cargo fmt --all`
cargo clippy --workspace --all-targets --locked -- -D warnings — passed
cargo test --workspace --locked — passed (49 app, 89 core, 3 tools, 0 doc-tests)
cargo build -p bredboard-app --target x86_64-unknown-linux-gnu --locked — passed
cargo build -p bredboard-app --target wasm32-unknown-unknown --locked — passed
cargo run -p bredboard-tools --locked -- validate-catalog breadboard-circuits/spec/catalog.json breadboard-circuits/spec/catalog.schema.json — passed; 20 sections, 212 schematics
```

## Current blockers

- LM358/LM3914/LM386, microphone, full SAP-1 CPU, bargraph, crystal, and
  multi-board supply/presentation contracts are not yet available.
- Manual interaction and real-breadboard evidence are pending for every new
  fixture.

## Completion and fukit handoff

Set `Status: ready_for_fukit` only after each of the 17 rows has reconciled
source, capability, fixture, electrical, automated, and release evidence. The
card is not complete merely because a generic IC sprite renders or a fixture
serializes.
