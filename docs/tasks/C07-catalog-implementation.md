# C07 — Hardware-logic and module fixtures

Status: in_progress; source review is recorded and implementation starts with
the two-station telegraph entry whose electrical branches are calculable while
the long cable remains a presentation-only discrepancy.

## Scope

Implement and disposition the 19 schematics in source section 09:

- `CAT-S09-01` through `CAT-S09-19`, hardware logic and modules.

The source record is
[`09-hardware-logic-modules.md`](../../breadboard-circuits/spec/09-hardware-logic-modules.md).

## Source and design review

- S09-01 is admitted with the existing Schmitt inverter, digital counter,
  seven-segment display, and button-as-wire-ring substitute. The physical
  contact prop remains a presentation discrepancy.
- S09-02 needs an ISD1820 voice module, reed switch, and recorded audio; keep
  it blocked.
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
- S09-09 needs priority encoding, multiple flip-flops, a seven-segment
  display, and five-player arbitration; keep it blocked.
- S09-10 needs a crystal timebase, six countdown counters, six displays, and
  chess-clock state; keep it blocked.
- S09-11 needs four reversible BCD counters, four displays, and score-control
  semantics; keep it blocked.
- S09-12 needs paired RF modules, L293D, two motors, and chassis motion; keep
  it blocked.
- S09-13 needs a VCO/PLL, decade counter, capacitor ramp, and a bounded
  decelerating animation contract; keep it blocked.
- S09-14 needs three timer/counter/display chains, comparators, and bounded
  slot-machine state; keep it blocked.
- S09-15 needs reed/PIR sensor modules, several timer/flip-flop stages, and
  alarm state sequencing; keep it blocked.
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

## Current blockers

- Logic packages, module I/O contracts, RF/audio/IR/gas/PIR/ultrasonic sensor
  contracts, relay behavior, crystal timebases, displays, motor drivers,
  chassis/elevator mechanics, and multi-board scene semantics are not yet
  available.
- Manual interaction and real-breadboard evidence are pending for any new
  fixture.

## Completion and fukit handoff

Set `Status: ready_for_fukit` only after all 19 rows have reconciled source,
capability, fixture, electrical, automated, and release evidence. The card is
not complete merely because a two-button branch or a generic IC sprite renders.
