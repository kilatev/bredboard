# C06 — Motor and motion fixtures

Status: in_progress; source review is recorded and implementation starts with
the first switch-controlled DC-motor entry whose electrical load contract can
be calculated without pretending to simulate a physical propeller.

## Scope

Implement and disposition the 19 schematics in source section 08:

- `CAT-S08-01` through `CAT-S08-19`, motors and motion.

The source record is [`08-motors.md`](../../breadboard-circuits/spec/08-motors.md).

## Source and design review

- S08-01 is the first admissible slice: a 3 V source, SPDT switch, and a
  two-terminal DC motor. The fixture can calculate motor terminal voltage and
  current and expose a derived speed indicator from the electrical load. The
  source propeller/flag is presentation-only and remains an explicit
  discrepancy until a motion scene contract exists.
- S08-02 needs a manually driven motor-generator input and a speed-dependent
  generated-voltage contract; keep it blocked until that input is defined.
- S08-03 needs a DPDT switch, signed motor direction, and reverse-polarity
  presentation; keep it physical-scope until the two-pole switch contract
  exists.
- S08-04 needs a motor, TIP120 flyback behavior, and a transient overvoltage
  diagnostic; keep it blocked until the motor transient contract exists.
- S08-05 is admitted as an electrical motor-load fixture; the vibration motor
  base and physical vibration remain presentation discrepancies.
- S08-06 needs PWM duty-cycle behavior, MOSFET switching, flyback diode, and
  motor averaging; keep it blocked until those contracts exist.
- S08-07 needs a servo pulse-width-to-angle contract; keep it blocked until a
  servo actuator model exists.
- S08-08 needs a comparator, thermistor, MOSFET, fan actuator, and thermal
  input contract; keep it blocked.
- S08-09 needs a motor position model, two limit switches, DPDT behavior, and
  curtain mechanics; keep it blocked.
- S08-10 needs complementary transistor H-bridge behavior, interlock and
  shoot-through diagnostics, and signed motor direction; keep it blocked.
- S08-11 needs the L293D dual-driver and two-motor chassis contracts; keep it
  blocked.
- S08-12 is admitted as a calculated four-phase stepper fixture: the
  CD4017 Q0–Q3 outputs drive a four-channel ULN2003 open-collector contract,
  and the 28BYJ-48 is represented by a five-wire four-coil load. Shaft
  rotation remains presentation-only; no scripted position or motion outcome
  is used.
- S08-13 and S08-14 need two-motor chassis motion plus light or reflectance
  sensor contracts; keep them blocked.
- S08-15 needs a solar-panel source, energy-storage contract, and burst motor
  load; keep it blocked.
- S08-16 needs a dual-motor driver, timer, bumper state, and bounded reverse-
  and-turn motion; keep it blocked.
- S08-17 needs a reversible stepper driver, target-count comparator, display,
  and rotary-scene contract; keep it blocked.
- S08-18 needs an optical encoder, calibrated time window, motor speed, and
  multi-digit display contracts; keep it blocked.
- S08-19 combines the blocked chassis, sensor, bumper, timing, comparator, and
  priority-logic contracts; keep it blocked until those dependencies exist.

## Acceptance criteria

- [ ] All 19 source records have source, BOM, SVG, pin, supply, safety, and
  discrepancy evidence in the ledger.
- [ ] Every admitted BOM item maps to an implemented model or an explicit
  recorded discrepancy; no scripted motor speed or motion outcome is used.
- [ ] Each admitted fixture validates, derives topology from holes/pins/wires,
  has stable IDs, readable placement, and a reachable menu entry.
- [ ] Motor current, terminal voltage, direction, and any derived speed signal
  have calculated regressions and bounded diagnostics.
- [x] Manual Linux interaction and representative breadboard checks are
  optional follow-up evidence; automated checks remain the acceptance gate.

## Implementation evidence so far

The first admitted fixture is `CAT-S08-01`,
`fixtures/projects/c06-s08-01-motor-with-switch.json`. It uses a calculated
two-terminal motor load, exposes motor current and signed voltage-derived
no-load RPM, and keeps the source propeller as an explicit presentation
discrepancy. The app menu exposes an SPDT control and a motor-specific
current/RPM readout.

The C06-S08-01 full gate passed on 2026-09-29 with `cargo fmt --all --check`,
`cargo clippy --workspace --all-targets --locked -- -D warnings`,
`cargo test --workspace --locked` (49 app, 97 core, 3 tools, 0 doc-test
failures), native and WASM app builds, fixture validation (`3 components, 2
wires, 3 derived nodes`), 4,000-step simulation, `validate-catalog`
(`20 sections, 212 schematics`), and `git diff --check`. Manual browser and
real-breadboard evidence remain pending.

The parallel C06-S08-05 slice adds
`fixtures/projects/c06-s08-05-vibration-bot.json` and registers it in the app
menu. It uses a calculated 3 V motor load and an SPDT branch; motor current
and voltage-derived no-load speed remain core results. The toothbrush body,
base, and vibration are presentation discrepancies.

Focused evidence for the slice:

```text
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c06-s08-05-vibration-bot.json — passed; 3 components, 2 wires, 3 derived nodes
cargo run -p bredboard-tools --locked -- simulate fixtures/projects/c06-s08-05-vibration-bot.json 4000 — passed; transient state advanced without diagnostics
cargo test -p bredboard-app all_embedded_boards_have_unique_lead_and_wire_holes --locked — passed
```

The CAT-S08-08 slice adds
`fixtures/projects/c06-s08-08-thermostatic-fan.json` and registers it in the
app menu. It maps the source's LM393/NTC threshold circuit to the existing
comparator and thermistor kinds, uses a calculated voltage-controlled
resistance contract for the IRLZ44N MOSFET terminals, a bounded 35 ohm fan
load, and the shared reverse diode model. Temperature and threshold are
explicit control ratios; no fan airflow or closed-loop thermal scene is
scripted.

Focused evidence for the slice:

```text
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c06-s08-08-thermostatic-fan.json — passed; 10 components, 7 wires, 6 derived nodes
cargo run -p bredboard-tools --locked -- simulate fixtures/projects/c06-s08-08-thermostatic-fan.json 4000 — passed; step 4000 at 0.400000 s without diagnostics
cargo test -p bredboard-core c06_ --locked — passed; cold/hot switching regression and 64-case threshold property regression
cargo test -p bredboard-app all_embedded_boards_have_unique_lead_and_wire_holes --locked — passed
cargo test -p bredboard-app exercise_catalog_keeps_search_groups_and_circuits_separate --locked — passed; 111 catalog circuits
cargo run -p bredboard-tools --locked -- validate-catalog breadboard-circuits/spec/catalog.json breadboard-circuits/spec/catalog.schema.json — passed; 20 sections, 212 schematics
```

The CAT-S08-12 slice adds
`fixtures/projects/c06-s08-12-stepper-motor.json` and registers it in the app
menu. It maps the source NE555 clock, CD4017 modulo-five sequence with Q4
reset, ULN2003 four-channel open-collector driver, and 28BYJ-48 five-wire
stepper load. The stepper contract calculates each coil current from the
common and phase voltages; the app exposes the four phase currents and active
phase. The source schematic's carry/reset behavior is made explicit with a
carry-to-ground wire, avoiding a floating digital input under the core
contract.

Focused evidence for the slice:

```text
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c06-s08-12-stepper-motor.json — passed; 9 components, 2 wires, 20 derived nodes
cargo run -p bredboard-tools --locked -- simulate fixtures/projects/c06-s08-12-stepper-motor.json 4000 — passed; step 4000 at 0.400000 s without diagnostics
cargo test -p bredboard-core c06_stepper_fixture_calculates_four_phase_load_and_driver_mapping --locked — passed; four direct phases, current balance, 20-node topology
cargo test -p bredboard-app all_embedded_boards_have_unique_lead_and_wire_holes --locked — passed
cargo test -p bredboard-app exercise_catalog_keeps_search_groups_and_circuits_separate --locked — passed; 113 catalog circuits
```

The full verification gate also passed: workspace formatting, Clippy,
workspace tests, native and WASM app builds, catalog validation, and
`git diff --check` across 2026-10-01–02. Manual browser and real-breadboard
evidence remain pending.

## Current blockers

- Motor-generator input, DPDT polarity switching, PWM/MOSFET/flyback behavior,
  servo angle, physical fan airflow/closed-loop thermal scene, mechanical
  position, H-bridge safety, L293D, chassis motion,
  optical/reflectance sensors, solar energy storage, and display/measurement
  contracts are not yet available.
- Manual interaction and real-breadboard evidence are pending for any new
  fixture.

## Completion and fukit handoff

Set `Status: ready_for_fukit` only after all 19 rows have reconciled source,
capability, fixture, electrical, automated, and release evidence. The card is
not complete merely because a motor sprite or current readout renders.
