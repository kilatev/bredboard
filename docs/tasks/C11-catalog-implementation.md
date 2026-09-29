# C11 — World-task capstone fixtures

Status: in_progress; source review is recorded and implementation starts with
the beacon composition. Most world tasks remain physical scope or blocked on
modules, actuators, and multi-board presentation.

## Scope

Implement and disposition the 22 schematics in source section 19:

- `CAT-S19-01` through `CAT-S19-22`.

The source record is `19-world-tasks.md`.

## Source and design review

- S19-01 is the first admissible slice: existing photoresistor, timer 555,
  potentiometer, NPN, capacitors, resistors, and LED models can calculate a
  dark-enabled beacon. The ship, ambient scene, and exact white LED optical
  output remain presentation discrepancies.
- S19-02 needs PIR and MOSFET module contracts plus a timed night-motion scene;
  keep it blocked.
- S19-03 needs two servo outputs, a dual timer, and a crane prop; keep it
  physical scope and blocked.
- S19-04 needs a ten-step light sequence, counter, flip-flop, and dense LED
  presentation; keep it blocked.
- S19-05 needs comparator, thermistor, MOSFET, and heater safety contracts;
  keep it blocked.
- S19-06 needs two IR modules, timed retriggering, a servo, and a barrier prop;
  keep it physical scope.
- S19-07 needs motor-driver, limit-switch, state, and elevator-prop contracts;
  keep it physical scope.
- S19-08 needs two IR modules, alternating lights, buzzer, servo, and crossing
  presentation; keep it physical scope and blocked.
- S19-09 needs an IR module, timed motor drive, limit switch, and door prop;
  keep it physical scope.
- S19-10 needs water-level probes, an RS latch, pump, and two water vessels;
  keep it physical scope.
- S19-11 needs coin/cam limit switches, relay/motor sequencing, and a donor
  dispenser prop; keep it physical scope.
- S19-12 needs long timing, servo motion, and feeder presentation; keep it
  blocked and physical scope.
- S19-13 needs a countdown display, cancel/start logic, motor, and rocket
  presentation; keep it blocked.
- S19-14 needs PWM ramping, motor, buzzer, and carousel motion; keep it
  physical scope and blocked.
- S19-15 needs two light sensors, comparator, motor tracking, and spotlight
  presentation; keep it physical scope.
- S19-16 needs radio encoder/decoder modules, motor control, limit switches,
  obstacle sensing, and a garage prop; keep it physical scope.
- S19-17 needs water-level probes, bridge motion, interlocks, and multi-board
  safety presentation; keep it physical scope.
- S19-18 needs a reflectance sensor, timed conveyor, solenoid, and high-current
  supply; keep it physical scope and safety-blocked.
- S19-19 needs directional dual-beam sensing, up/down counters, displays, and
  a capacity presentation; keep it blocked.
- S19-20 needs thermistor/comparator, relay, alarm, latch, and evacuation
  interlocks; keep it blocked until those contracts are complete.
- S19-21 needs water vessels, level probes, two servos, pump, and interlocks;
  keep it physical scope.
- S19-22 is a four-board city composition with eight actuators and twenty IC/
  module contracts; keep it physical scope.

## Acceptance criteria

- [ ] All 22 source records have source, BOM, SVG, pin, supply, safety, and
  discrepancy evidence in the ledger.
- [ ] The admitted beacon fixture maps every electrical BOM item to a
  calculated model and records the scene/optical discrepancies.
- [ ] The fixture validates, preserves derived topology, has stable IDs,
  readable placement, and a reachable menu entry.
- [ ] Darkness control, timer output, and LED/NPN current have a calculated
  regression plus a bounded property check over supported control values.
- [ ] Required manual Linux interaction and representative breadboard checks
  are recorded separately; builds and tests are not treated as manual proof.

## Implementation evidence so far

The first admitted fixture is `CAT-S19-01`,
`fixtures/projects/c11-s19-01-beacon.json`. It is a 9 V photoresistor-gated
555 beacon with a calculated NPN and current-limited LED load. The photoresistor
holds reset low in bright conditions and enables the fixed-step oscillator in
darkness; the source ship scene and exact white-LED optical behavior remain
explicit presentation discrepancies.

The fixture's C11 implementation gate passed locally on 2026-09-29 with:

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked` — 49 app tests, 105 core tests, 3 tool
  tests, and 0 doc-test failures
- `cargo build -p bredboard-app --target x86_64-unknown-linux-gnu --locked`
- `cargo build -p bredboard-app --target wasm32-unknown-unknown --locked`
- `cargo run -p bredboard-tools --locked -- validate
  fixtures/projects/c11-s19-01-beacon.json` — 12 components, 7 wires, 10
  derived nodes
- `cargo run -p bredboard-tools --locked -- simulate
  fixtures/projects/c11-s19-01-beacon.json 4000` — step 4000 completed
- `cargo run -p bredboard-tools --locked -- validate-catalog
  breadboard-circuits/spec/catalog.json
  breadboard-circuits/spec/catalog.schema.json` — 20 sections, 212 schematics
- `git diff --check`

Manual Linux interaction and real-breadboard evidence remain pending.

## Current blockers

- PIR, IR, Hall/reflectance, water-level, radio, comparator, MOSFET, servo,
  pump, solenoid, multi-board, and world-scene contracts are unavailable.
- Manual interaction and real-breadboard evidence are pending for any new
  fixture.

## Completion and fukit handoff

Set `Status: ready_for_fukit` only after all 22 rows have reconciled source,
capability, fixture, electrical, manual, and release evidence. The card is not
complete merely because one beacon fixture validates.
