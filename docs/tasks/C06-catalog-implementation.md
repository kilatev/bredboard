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
- S08-05 needs a vibration motor and a physical base; keep it physical-scope.
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
- S08-12 needs a stepper motor, ULN2003 driver, and four-phase state contract;
  keep it blocked.
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
- [ ] Required manual Linux interaction and representative breadboard checks
  are recorded separately; builds and tests are not treated as manual proof.

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

## Current blockers

- Motor-generator input, DPDT polarity switching, PWM/MOSFET/flyback behavior,
  servo angle, fan and thermal input, mechanical position, H-bridge safety,
  L293D, stepper/ULN2003, chassis motion, optical/reflectance sensors, solar
  energy storage, and display/measurement contracts are not yet available.
- Manual interaction and real-breadboard evidence are pending for any new
  fixture.

## Completion and fukit handoff

Set `Status: ready_for_fukit` only after all 19 rows have reconciled source,
capability, fixture, electrical, manual, and release evidence. The card is
not complete merely because a motor sprite or current readout renders.
