# C03 — Counting, display, and systems fixtures

Status: in_progress; the Section 04 and Section 05 source review is recorded
and implementation is proceeding in bounded capability slices. Six Section 04
fixtures are admitted; the remaining 11 entries and all manual evidence remain
open.

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
- S04-06 and S05-04 need edge/reset semantics stronger than the current C02
  level-sensitive D-flip-flop approximation. They can only be admitted with a
  documented bounded behavior or remain blocked.
- S04-07 and S04-08 depend on LM358, LM3914, LM386, microphone, bargraph, and
  audio contracts. Audio output remains presentation-only and cannot be
  claimed as an electrical amplifier result without the corresponding model.
- S04-09 and S05-07 reuse 555 timing but require multiple timers and a
  calculated control-voltage path; speaker frequency is a readout contract,
  not a wall-clock or audio-script result.
- S05-03, S05-05, and S05-06 contain state machines, clock division, or
  crystal assumptions. Educational bounded timing is admissible only when the
  observable state is calculated from pins and fixed simulation steps.
- S05-08 explicitly requires approximately 30 ICs, SRAM, three boards, and a
  5 V / 2 A supply. It remains a multi-board scope blocker until board
  identity, supply limits, SRAM behavior, and the presentation surface are
  defined.

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
- Record unsupported audio, crystal, SRAM, op-amp, bargraph, physical-prop,
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
- [ ] Required manual Linux interaction and representative breadboard checks
  are recorded separately; builds and tests are not treated as manual proof.

## Implementation evidence so far

The current slice adds calculated `digital_counter`, `shift_register`, and
`seven_segment_display` contracts, six embedded Section 04 fixtures, menu
entries, and fixed-step regressions for rising-edge counting, BCD segment
decoding, serial shifting, and output latching.

```text
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c03-s04-03-button-counter.json — passed; 5 components, 0 wires, 20 derived nodes
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c03-s04-05-shift-register.json — passed; 8 components, 0 wires, 13 derived nodes
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c03-s04-04-binary-counter.json — passed; 12 components, 4 wires, 7 derived nodes
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c03-s04-01-running-lights.json — passed; 24 components, 10 wires, 13 derived nodes
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c03-s04-02-electronic-dice.json — passed; 18 components, 7 wires, 10 derived nodes
cargo test -p bredboard-core c03_dice_counter_calculates_the_one_dot_pattern --locked — passed
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c03-s04-09-dual-555-noisemaker.json — passed; 16 components, 9 wires, 10 derived nodes
cargo run -p bredboard-tools --locked -- simulate fixtures/projects/c03-s04-09-dual-555-noisemaker.json 10 — passed; transient state advanced for C1/C2/C3/C4
cargo test -p bredboard-core digital_counter_advances_once_per_calculated_rising_edge --locked — passed
cargo test -p bredboard-core c03_counter_fixture_drives_a_calculated_display_digit --locked — passed
cargo test -p bredboard-core c03_shift_register_calculates_shift_then_latch --locked — passed
cargo test -p bredboard-app all_embedded_boards_have_unique_lead_and_wire_holes --locked — passed
cargo test -p bredboard-app each_circuit_uses_core_controls_and_reset --locked — passed
cargo fmt --all --check — passed after `cargo fmt --all`
cargo clippy --workspace --all-targets --locked -- -D warnings — passed
cargo test --workspace --locked — passed (49 app, 76 core, 3 tools, 0 doc-tests)
cargo build -p bredboard-app --target x86_64-unknown-linux-gnu --locked — passed
cargo build -p bredboard-app --target wasm32-unknown-unknown --locked — passed
cargo run -p bredboard-tools --locked -- validate-catalog breadboard-circuits/spec/catalog.json breadboard-circuits/spec/catalog.schema.json — passed; 20 sections, 212 schematics
```

## Current blockers

- The current D flip-flop is a C02 level-sensitive capacitor-backed
  approximation; S04-06 and S05-04 cannot be release-ready without a stronger
  edge/reset contract or an explicit bounded lesson disposition.
- LM358/LM3914/LM386, microphone, bargraph, crystal, SRAM, and multi-board
  supply/presentation contracts are not yet available.
- Manual interaction and real-breadboard evidence are pending for every new
  fixture.

## Completion and fukit handoff

Set `Status: ready_for_fukit` only after each of the 17 rows has reconciled
source, capability, fixture, electrical, manual, and release evidence. The
card is not complete merely because a generic IC sprite renders or a fixture
serializes.
