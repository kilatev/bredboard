# C07 — Hardware-logic and module fixtures

Status: in_progress; source review is recorded and implementation includes
the admitted talking-card and two-station telegraph entries whose electrical
contracts are calculable while their physical/audio details remain explicit
discrepancies.

## Scope

Implement and disposition the 19 schematics in source section 09:

- `CAT-S09-01` through `CAT-S09-19`, hardware logic and modules.

The source record is
[`09-hardware-logic-modules.md`](../../breadboard-circuits/spec/09-hardware-logic-modules.md).

## Source and design review

- S09-01 is admitted with the existing Schmitt inverter, digital counter,
  seven-segment display, and button-as-wire-ring substitute. The physical
  contact prop remains a presentation discrepancy.
- S09-02 is admitted with a calculated reed-switch contract and an
  ISD1820-style threshold-output module contract. The 4.5 V trigger path and
  8 ohm speaker load are calculated; recorded speech, magnetic mechanics, and
  one-shot playback timing remain presentation discrepancies.
- S09-03 is the first admissible slice: two buttons, two active buzzer loads,
  two LEDs, two resistors, and one source. The long cable is represented by
  calculated wires and remains an explicit presentation discrepancy.
- S09-04 needs a sound sensor module, D flip-flop edge contract, relay coil,
  flyback diode, and lamp actuator; keep it blocked.
- S09-05 needs an IR receiver, pulse-decoding behavior, relay, and remote
  control input; keep it blocked.
- S09-06 needs a PIR sensor, voice module, and one-shot retrigger contract;
  keep it blocked.
- S09-07 needs paired RF encoder/decoder modules, an RF-link contract, and
  address jumpers; keep it blocked.
- S09-08 needs an MQ-2 gas module, warm-up/analog sensor contract, and a
  comparator-driven alarm path; keep it blocked.
- S09-09 is admitted with four calculated D flip-flops, a calculated NOR
  allow gate, a calculated multi-input winner OR, two BCD bit OR gates, and
  the existing seven-segment display model. The source package boundaries
  (74HC20, 74HC148, and CD4511) are an explicit model-level discrepancy; a
  simultaneous same-step button press is resolved by stable simulation order.
- S09-10 is admitted as a calculated chess-clock fixture: a fixed-step clock
  source and D flip-flop divide stage provide a deterministic 1 Hz tick; six
  reversible BCD counters drive two three-digit countdown readouts; button
  pulses transfer the active turn; and calculated zero detectors drive the
  timeout LEDs and buzzer. The source's 32,768 Hz crystal, CD4060/CD4013/
  CD4510/CD4511/CD4011 package internals, 9 V adapter, and exact display
  segment-current wiring remain explicit model or layout discrepancies.
- S09-11 is admitted as a calculated score-board fixture: four compact BCD
  counters drive four displays, electrical direction inputs distinguish the
  plus and minus controls, and carry/borrow/reset behavior is covered by the
  app regression. The source's CD4510/CD4511/CD4093 package internals and the
  exact physical board layout remain explicit model or buildability
  discrepancies.
- S09-12 needs paired RF modules, L293D, two motors, and chassis motion; keep
  it blocked.
- S09-13 needs a VCO/PLL, decade counter, capacitor ramp, and a bounded
  decelerating animation contract; keep it blocked.
- S09-14 needs three timer/counter/display chains, comparators, and bounded
  slot-machine state; keep it blocked.
- S09-15 is admitted as a calculated security-alarm fixture: the SPDT key
  controls an armed latch, a finite reed contact and threshold-output PIR
  module feed the sensor path, and a D flip-flop latches the alarm. Bounded
  555 stages drive the entry, siren, and status branches. The source's exact
  package/timing behavior, apartment scene, PIR/magnetic mechanics, and audio
  presentation remain explicit discrepancies; the 5 V source is a low-voltage
  proxy and the fixture is not a mains safety design.
- S09-16 needs an HC-SR04 distance/time-of-flight contract, analog scaling,
  bargraph output, and distance-dependent tone; keep it blocked.
- S09-17 needs floor sensors, multi-stage logic, L293D, motor motion, and a
  physical elevator shaft; keep it blocked.
- S09-18 needs a crystal timebase, counters, six displays, and binary-clock
  presentation; keep it blocked.
- S09-19 needs three boards, many logic stages, and a multi-state ping-pong
  controller; keep it blocked.

## Acceptance criteria

- [ ] All 19 source records have source, BOM, SVG, pin, supply, safety, and
  discrepancy evidence in the ledger.
- [ ] Every admitted BOM item maps to an implemented model or an explicit
  recorded discrepancy; no scripted communication or module outcome is used.
- [ ] Each admitted fixture validates, derives topology from holes/pins/wires,
  has stable IDs, readable placement, and a reachable menu entry.
- [ ] Button, LED, buzzer, and any module substitute behavior has calculated
  regressions and bounded diagnostics.
- [x] Manual Linux interaction and representative breadboard checks are
  optional follow-up evidence; automated checks remain the acceptance gate.

## Implementation evidence so far

The first admitted fixture is `CAT-S09-03`,
`fixtures/projects/c07-s09-03-two-station-telegraph.json`. It uses two
calculated button-controlled LED and active-buzzer stations; the source cable
is an explicit presentation discrepancy. The app menu exposes both button
controls and a station-by-station current readout.

The C07-S09-03 full gate passed on 2026-09-29 with `cargo fmt --all --check`,
`cargo clippy --workspace --all-targets --locked -- -D warnings`,
`cargo test --workspace --locked` (49 app, 98 core, 3 tools, 0 doc-test
failures), native and WASM app builds, fixture validation (`9 components, 4
wires, 6 derived nodes`), 4,000-step simulation, `validate-catalog`
(`20 sections, 212 schematics`), and `git diff --check`. Manual browser and
real-breadboard evidence remain pending.

The CAT-S09-09 slice adds
`fixtures/projects/c07-s09-09-quiz-buttons.json` and registers it in the app
menu. It preserves the source BOM counts for buttons, LEDs, resistors,
capacitors, buzzer, and source. Four D flip-flops latch player inputs; the
calculated NOR allow path blocks later inputs, the multi-input OR drives the
buzzer, and calculated BCD outputs drive the seven-segment display. The
source's package-level 74HC20/74HC148/CD4511 behavior is represented by the
existing bounded pin-level primitives, with the discrepancy recorded in the
ledger.

Focused evidence for the slice:

```text
cargo run -q -p bredboard-tools --locked -- validate fixtures/projects/c07-s09-09-quiz-buttons.json — passed; 41 components, 5 wires, 28 derived nodes
cargo test -p bredboard-app --locked c07_s09_09_latches_first_player_blocks_later_inputs_and_resets -- --nocapture — passed
cargo test -p bredboard-app --locked all_embedded_boards_have_unique_lead_and_wire_holes -- --nocapture — passed
```

The CAT-S09-09 full automated gate passed on 2026-10-02: the 4,000-step
simulation completed, `validate-catalog` reported 20 sections and 212
schematics, `cargo fmt --all --check`, Clippy, `cargo test --workspace
--locked` (59 app, 147 core, 3 tools, 0 doc-test failures), native Linux and
WASM app builds, and `git diff --check` all passed. Manual browser and
real-breadboard evidence remain pending.

The parallel C07-S09-01 slice adds
`fixtures/projects/c07-s09-01-hot-wire-button-counter.json` and registers it
in the app menu. It uses a calculated debounce path, decimal counter, and
seven-segment display with resistor-limited segment loads. The wire-ring prop
is represented by an explicit button control; no scripted counter result is
used.

Focused evidence for the slice:

```text
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c07-s09-01-hot-wire-button-counter.json — passed; 16 components, 0 wires, 16 derived nodes
cargo run -p bredboard-tools --locked -- simulate fixtures/projects/c07-s09-01-hot-wire-button-counter.json 500 — passed; transient state advanced without diagnostics
cargo test -p bredboard-app all_embedded_boards_have_unique_lead_and_wire_holes --locked — passed
```

The CAT-S09-02 slice adds
`fixtures/projects/c07-s09-02-talking-card.json` and registers it in the app
menu. The source's ISD1820 pins map to the shared module contract: PLAYE is a
threshold input, SPK+ is a bounded output, and SPK− is grounded through the
board. The new calculated reed-switch contract uses ordered button-style
control actions while retaining the source BOM's one reed component. The
speaker is presented from solved current using the existing fixed-tone audio
scope; no recorded clip or scripted phrase result is used.

Focused evidence for the slice:

```text
cargo run -q -p bredboard-tools --locked -- validate fixtures/projects/c07-s09-02-talking-card.json — passed; 4 components, 1 wire, 4 derived nodes
cargo run -q -p bredboard-tools --locked -- simulate fixtures/projects/c07-s09-02-talking-card.json 500 — passed; step 500 at 0.050000 s without diagnostics
cargo test -p bredboard-core --locked talking_card_reed_triggers_calculated_module_and_speaker_load -- --nocapture — passed
cargo test -p bredboard-app --locked c07_s09_02_reed_control_drives_the_talking_card_speaker -- --nocapture — passed
cargo test -p bredboard-app --locked all_embedded_boards_have_unique_lead_and_wire_holes -- --nocapture — passed
```

The CAT-S09-02 full automated gate passed on 2026-10-02: `cargo fmt --all
--check`, Clippy with `-D warnings`, `cargo test --workspace --locked` (62
app, 151 core, 3 tools, 0 doc-test failures), native Linux and WASM app
builds, `validate-catalog` (20 sections, 212 schematics), and `git diff
--check`. Manual browser, audio, and real-breadboard evidence remain pending.

The shared board-model slice now provides a validated multi_board contract
for up to eight named half_size_solderless boards. Qualified
board_id/local_hole references preserve per-board strips and rails, while
explicit wire endpoints can connect boards; deterministic topology and a
calculated cross-board resistor path are covered by fixed-seed core tests.
S09-19 remains blocked because its logic packages, controller behavior, and
three-board fixture/layout are not implemented.

The CAT-S09-10 slice adds
`fixtures/projects/c07-s09-10-chess-clock.json` and registers it in the app
menu. It contains 88 components, 88 explicit wires, and 50 derived nodes on
three named half-size boards. The clock source is a fixed-step 2 Hz proxy,
the divide stage produces the calculated 1 Hz counter clock, and the six
reversible counters start at 5:00 for each player. The active-turn gates keep
one player's counters enabled; button pulses clock the turn flip-flop. The
source's 44 segment resistors are all retained, but 42 are explicit VCC–GND
loads because the current placement does not couple them to display segment
pins; this is recorded in the ledger instead of being presented as a complete
display-current path.

Focused evidence for the slice:

```text
cargo run -q -p bredboard-tools --locked -- validate fixtures/projects/c07-s09-10-chess-clock.json — passed; 88 components, 88 wires, 50 derived nodes
cargo run -q -p bredboard-tools --locked -- simulate fixtures/projects/c07-s09-10-chess-clock.json 10 — passed; step 10 at 0.001000 s without diagnostics
cargo test -p bredboard-core --locked c07_s09_10_clock_fixture_counts_down_and_transfers_turn -- --nocapture — passed
cargo test -p bredboard-app --locked c07_s09_10_counts_down_and_transfers_the_active_turn -- --nocapture — passed
cargo test -p bredboard-app --locked all_embedded_boards_have_unique_lead_and_wire_holes -- --nocapture — passed
```

The full automated gate passed on 2026-10-02: `cargo fmt --all -- --check`,
Clippy with `-D warnings`, `cargo test --workspace --locked` (60 app, 149
core, 3 tools, 0 doc-test failures), native Linux and WASM app builds, and
`git diff --check`. Manual browser and real-breadboard evidence remain
pending.

The CAT-S09-11 slice adds
`fixtures/projects/c07-s09-11-score-board.json` and registers it in the app
menu. It contains 56 components, 11 explicit wires, and 57 derived nodes on
two named half-size boards. The four counters use compact q0–q3 BCD outputs;
their optional direction pins are driven by the calculated plus/minus control
branches, while calculated carry/borrow and reset behavior feed the four
seven-segment displays. The two CD4093 packages are bounded by four calculated
OR/AND gate contracts, so Schmitt-trigger debounce and exact package current
behavior remain explicit discrepancies. Two boards are required by the
current contact model: one half-size board cannot provide the 63 distinct
signal/ground groups needed by this fixture; this is recorded as a
buildability/layout finding rather than hidden in the schematic mapping.

Focused evidence for the slice:

```text
cargo run -q -p bredboard-tools --locked -- validate fixtures/projects/c07-s09-11-score-board.json — passed; 56 components, 11 wires, 57 derived nodes
cargo run -q -p bredboard-tools --locked -- simulate fixtures/projects/c07-s09-11-score-board.json 4000 — passed; step 4000 at 0.400000 s, capacitors C1–C5 at 9.000000000 V, no diagnostics
cargo test -p bredboard-app --locked c07_s09_11_score_buttons_count_up_down_carry_and_reset -- --nocapture — passed
cargo test -p bredboard-app --locked all_embedded_boards_have_unique_lead_and_wire_holes -- --nocapture — passed
```

The CAT-S09-11 automated gate passed on 2026-10-02: `cargo fmt --all --
--check`, Clippy with `-D warnings`, `cargo test --workspace --locked` (64
app, 152 core, 3 tools, 0 doc-test failures), native Linux and WASM app
builds, `validate-catalog` (20 sections, 212 schematics), and `git diff
--check`. Manual browser and real-breadboard evidence remain pending.

The CAT-S09-15 slice adds
`fixtures/projects/c07-s09-15-security-alarm.json` and registers it in the
app menu. It contains 49 components, 22 explicit wires, and 23 derived nodes.
The SPDT key arms and resets a calculated D flip-flop latch; the finite reed
contact and threshold-output PIR module feed the calculated sensor path; a
second D flip-flop latches the alarm; and bounded timer stages drive the entry,
siren, red LED, and speaker branches. The source's eight IC packages are
represented by ten bounded logical primitives, and the 5 V supply, exact
timings, apartment scene, PIR/magnetic mechanics, and audio remain explicit
discrepancies rather than scripted outcomes.

Focused evidence for the slice:

```text
cargo run -q -p bredboard-tools --locked -- validate fixtures/projects/c07-s09-15-security-alarm.json — passed; 49 components, 22 wires, 23 derived nodes
cargo run -q -p bredboard-tools --locked -- simulate fixtures/projects/c07-s09-15-security-alarm.json 4000 — passed; step 4000 at 0.400000 s without diagnostics
cargo test -p bredboard-core --locked c07_s09_15_key_and_reed_drive_deterministic_alarm_sequence -- --nocapture — passed
cargo test -p bredboard-app --locked c07_s09_15_arms_latches_alarm_and_resets_from_key -- --nocapture — passed
cargo test -p bredboard-app --locked all_embedded_boards_have_unique_lead_and_wire_holes -- --nocapture — passed
```

The CAT-S09-15 automated gate passed on 2026-10-02: `cargo fmt --all
--check`, Clippy with `-D warnings`, `cargo test --workspace --locked` (65
app, 153 core, 3 tools, 0 doc-test failures), native Linux and WASM app
builds, `validate-catalog breadboard-circuits/spec/catalog.json
breadboard-circuits/spec/catalog.schema.json` (20 sections, 212 schematics),
and `git diff --check`. Manual browser, real-breadboard, and audio evidence
remain pending.

## Current blockers

- Logic packages, module I/O contracts, RF/audio/IR/gas/PIR/ultrasonic sensor
  contracts, relay behavior, exact crystal timebases and package-level
  displays, motor drivers,
  chassis/elevator mechanics, and S09-19's fixture-specific multi-board scene
  semantics are not yet available.
- Manual interaction and real-breadboard evidence are pending for any new
  fixture.

## Completion and fukit handoff

Set `Status: ready_for_fukit` only after all 19 rows have reconciled source,
capability, fixture, electrical, automated, and release evidence. The card is
not complete merely because a two-button branch or a generic IC sprite renders.
