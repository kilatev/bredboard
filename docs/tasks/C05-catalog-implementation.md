# C05 — Home-tool and power-energy fixtures

Status: in_progress; the source review is recorded and implementation has
admitted the Section 07 timer, pulse-generator, and adjustable-regulator
entries whose calculated contracts are sufficient.

## Scope

Implement and disposition the 24 schematics in source sections 07 and 10:

- `CAT-S07-01` through `CAT-S07-15`, home tools;
- `CAT-S10-01` through `CAT-S10-09`, power and energy.

Source records are [`07-home-tools.md`](../../breadboard-circuits/spec/07-home-tools.md)
and [`10-power-energy.md`](../../breadboard-circuits/spec/10-power-energy.md).

## Source and design review

- S07-01, S07-03, S07-04, S07-07, S07-11, and S07-12 require probes,
  unknown-device sockets, zener behavior, module contracts, or physical props.
- S07-13, S07-14, and S07-15 require comparator, counter,
  measurement, or module contracts not present in the current core. Keep them
  blocked rather than presenting scripted readings.
- S07-06 is admitted through the calculated adjustable-regulated-supply
  module contract. Its 12 V input, 2 V dropout limit, 10 V output clamp,
  ADJ feedback, reverse-protection diodes, and high-impedance voltmeter are
  explicit electrical behavior; the source's 5 kΩ label versus its 1.25–10 V
  range remains a bounded calibration discrepancy.
- S07-09 can use the existing timer/RC, button-as-reed-contact substitute,
  buzzer, resistor, and capacitor contracts. The physical magnet and door are
  presentation discrepancies.
- S07-10 can use the existing timer/RC, button, buzzer, LED, potentiometer,
  and PNP contracts. Its exact two-minute calibration remains a bounded timing
  discrepancy.
- S10-01, S10-02, and S10-04 require physical electrochemical cells, batteries,
  or inductors. S10-03 requires an AC source and bridge rectifier. S10-05
  requires a supercapacitor; the shared diode model now covers its Schottky
  junction. S10-06 requires a Li-ion charger module. S10-07 requires a
  low-voltage CMOS timer, MOSFET, inductor, and boost/clamp contract; the
  shared model covers its Schottky and zener junctions. S10-08 requires a shunt amplifier and bargraph;
  S10-09 requires a solar-panel source and charge/power contracts. Keep these
  entries blocked or physical-scope until those contracts exist.

## Acceptance criteria

- [ ] All 24 source records have source, BOM, SVG, pin, supply, safety, and
  discrepancy evidence in the ledger.
- [ ] Every admitted BOM item maps to an implemented model or an explicit
  recorded discrepancy; no scripted electrical result is used.
- [ ] Each admitted fixture validates, derives topology from holes/pins/wires,
  has stable IDs, readable placement, and a reachable menu entry.
- [ ] Timer/RC, load current, control ordering, and any new signal contract
  have calculated regressions and bounded diagnostics.
- [x] Manual Linux interaction and representative breadboard checks are
  optional follow-up evidence; automated checks remain the acceptance gate.

## Implementation evidence so far

The first admitted C05 fixtures are `CAT-S07-10` (two-minute timer) and
`CAT-S07-08` (pulse generator). S07-10 uses a calculated 555 monostable,
adjustable RC timing, button input, LED/buzzer load readouts, and a PNP
companion branch. S07-08 uses an adjustable calculated 555 astable and LED
pulse readout. The source timing calibration, passive-buzzer behavior, rotary
selector, and external output terminals remain explicit discrepancies; manual
browser and real-breadboard evidence remain pending.

S07-06 uses `fixtures/projects/c05-s07-06-adjustable-power-supply.json` and a
calculated LM317-style adjustable regulator with explicit IN/OUT/ADJ/GND pins,
240 Ω feedback, a 5 kΩ control, both 1N4007 protection paths, three
capacitors, and a calculated analog-transfer voltmeter. Its feedback
monotonicity/bounds regression, 4-node topology, unique lead/wire placement,
app menu reachability, fixture validation, and 4,000-step transient simulation
passed; manual browser and real-breadboard evidence remain pending.

The S07-10 full gate passed on 2026-09-29 with `cargo fmt --all --check`,
`cargo clippy --workspace --all-targets --locked -- -D warnings`,
`cargo test --workspace --locked` (49 app, 95 core, 3 tools, 0 doc-test
failures), native and WASM app builds, fixture validation and 4,000-step
simulation, `validate-catalog` (20 sections, 212 schematics), and `git diff

The S07-08 full gate passed on 2026-09-29 with the same workspace checks (49
app, 96 core, 3 tools, 0 doc-test failures), native and WASM app builds,
fixture validation (`11 components, 4 wires, 7 derived nodes`), 4,000-step
simulation, `validate-catalog` (20 sections, 212 schematics), and `git diff
--check`.

The parallel C05-S07-09 slice adds
`fixtures/projects/c05-s07-09-refrigerator-guard.json` and registers it in the
app menu. It uses a calculated 555 monostable, explicit button control as a
reed-contact substitute, a 330 kΩ delay resistor, a 100 µF capacitor, and a
bounded buzzer load. The physical magnet and refrigerator door remain
presentation discrepancies; no scripted electrical outcome is used.

The CAT-S07-02 slice admits
`fixtures/projects/c05-s07-02-diode-tester.json` through the calculated
`diode_test_socket` behavior. The fixture preserves the source's 9 V source,
1 kΩ limiter, and two-terminal socket, and selects an LED subject in the
working forward-polarity state. The solver reuses the shared diode equation;
focused regression coverage also checks reversed, open, and shorted subject
states. Physical insertion remains a presentation discrepancy, not a scripted
electrical result.

CAT-S07-02 verification on 2026-10-01: `cargo fmt --all --check`, `cargo
clippy --workspace --all-targets --locked -- -D warnings`, `cargo test
--workspace --locked` (56 app, 144 core, 3 tools, 0 doc-test failures),
`cargo build -p bredboard-app --target x86_64-unknown-linux-gnu --locked`,
`cargo build -p bredboard-app --target wasm32-unknown-unknown --locked`,
`cargo run -p bredboard-tools --locked -- validate-catalog
breadboard-circuits/spec/catalog.json`, and `git diff --check` all passed.
The CAT-S07-02 fixture validation and 4,000-step simulation also passed; the
focused results are recorded below.

Focused evidence for the S07-09 slice:

```text
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c05-s07-09-refrigerator-guard.json — passed; 7 components, 5 wires, 5 derived nodes
cargo run -p bredboard-tools --locked -- simulate fixtures/projects/c05-s07-09-refrigerator-guard.json 4000 — passed; transient state advanced without diagnostics
cargo test -p bredboard-app all_embedded_boards_have_unique_lead_and_wire_holes --locked — passed
```

Focused CAT-S07-02 evidence on 2026-10-01:

```text
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c05-s07-02-diode-tester.json — passed; 3 components, 3 wires, 3 derived nodes
cargo run -p bredboard-tools --locked -- simulate fixtures/projects/c05-s07-02-diode-tester.json 4000 — passed; transient state advanced without diagnostics
cargo test -p bredboard-app c05_s07_02_diode_socket_derives_polarity_and_fault_current --locked — passed; working/reversed/open/shorted calculated currents
cargo test -p bredboard-app all_embedded_boards_have_unique_lead_and_wire_holes --locked — passed
```

The CAT-S07-04 slice admits `fixtures/projects/c05-s07-04-transistor-tester.json`
through a calculated `bjt_test_socket` behavior. Its two `Other` instances
declare explicit NPN and PNP socket polarity, matching working subject state,
and bounded open/short failure states; the solver reuses the existing NPN/PNP
equations and reports terminal currents. The source's physical swappable-part
operation remains a presentation discrepancy because the fixture exposes two
fixture-selected test channels.

Focused CAT-S07-04 evidence on 2026-10-01:

```text
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c05-s07-04-transistor-tester.json — passed; 11 components, 14 wires, 10 derived nodes
cargo run -p bredboard-tools --locked -- simulate fixtures/projects/c05-s07-04-transistor-tester.json 4000 — passed; transient state advanced without diagnostics
cargo test -p bredboard-app c05_s07_04 --locked — passed; calculated green/red LED and socket currents
cargo test -p bredboard-app all_embedded_boards_have_unique_lead_and_wire_holes --locked — passed
cargo test -p bredboard-app exercise_catalog_keeps_search_groups_and_circuits_separate --locked — passed; 107 circuits
```

The CAT-S07-05 slice admits
`fixtures/projects/c05-s07-05-usb-5v-supply.json` through the existing
calculated `regulated_supply` module contract. Its 9 V source, 5 V target,
2 V dropout limit, three decoupling capacitors, 330 Ω/green-LED load, and
two-terminal high-impedance USB adapter are represented in the solved board
topology. USB protocol, attached-device charging current, and regulator
thermal/current limits remain explicit discrepancies; the fixture does not
claim those behaviors.

Focused CAT-S07-05 evidence on 2026-10-01:

```text
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c05-s07-05-usb-5v-supply.json — passed; 8 components, 1 wire, 4 derived nodes
cargo run -p bredboard-tools --locked -- simulate fixtures/projects/c05-s07-05-usb-5v-supply.json 4000 — passed; transient state advanced without diagnostics
cargo test -p bredboard-core c05_battery_supply_regulates_five_volts_and_limits_usb_load --locked — passed; 5 V regulation, LED load, high-impedance USB connector, and bounded dropout
cargo test -p bredboard-app c05_s07_05_regulates_five_volts_and_keeps_usb_connector_high_impedance --locked — passed
cargo test -p bredboard-app all_embedded_boards_have_unique_lead_and_wire_holes --locked — passed
```

The full workspace gate passed on 2026-10-01 after this slice: `cargo fmt
--all --check`, Clippy, workspace tests (57 app, 145 core, 3 tools, 0
doc-test failures), native Linux and WASM app builds, catalog validation, and
`git diff --check` all passed. The required Cargo commands used the exported
shared target and temporary directories; no build output was redirected to
`/tmp`.

## Current blockers

- Physical probes, the remaining unknown-device socket (S07-12), AC sources, battery and lemon-cell
  models, comparators, counters, measurement modules, inductors,
  MOSFETs, supercapacitors, charger modules, solar sources, and bargraph
  instrumentation are not yet available. The shared diode model is available;
  row-specific diode wiring and power-stage behavior remain fixture work.
- Manual interaction and real-breadboard evidence are pending for all new
  fixtures.

## Planned: remaining device-under-test socket models (S07-02, S07-12)

Re-reviewed 2026-09-30 after two independent blocked_component passes on
S07-02 confirmed the same gap. Before this slice,
`crates/core/src/other_device.rs` (`OtherDeviceSpec`/`OtherDeviceBehavior`)
modeled one static, continuously computed electrical behavior per instance
(resistive, voltage source, linear transfer, transformer, ring modulator,
voltage-controlled resistance). The new `bjt_test_socket` behavior adds an
explicit fixture-selected BJT polarity/state mapping for S07-04; it does not
provide a runtime swappable-part editor. The remaining runtime-variable input
in core is the continuous `control_ratios: BTreeMap<ComponentId, f64>` used for
potentiometer/photoresistor-style interpolation.

The remaining ledger row shares this exact shape — "insert an unknown/swappable
part of a known family into a fixed test harness, read out pass/fail or a
bounded measurement":

- `CAT-S07-12` — network-cable tester: needs RJ45 terminal/pair mapping and
  calculated continuity faults.

`CAT-S07-03` (battery tester) and `CAT-S07-13` (battery charge gauge) are
adjacent but distinct: they need a bounded *measurement* contract (voltage
threshold → LED/bargraph readout), which is closer to a parameterized
`LinearTransfer` than to a new discrete-state type, though both would
consume a device-under-test socket if one existed. `CAT-S07-06` (adjustable
PSU) is unrelated — it needs a three-terminal regulator transfer model with
safety limits, not a swappable test subject, and must stay out of this
slice.

The shared contract is now implemented for CAT-S07-04 and CAT-S07-02.
`DiodeTestSocket` covers the two-terminal LED/diode family, polarity, and
working/open/shorted fixture states. The remaining S07-12 model work still
needs a concrete starting point, per AGENTS.md's "no scripted electrical
outcomes" and "derive connectivity from contacts/pins/wires" rules:

1. Extend the existing `bjt_test_socket` pattern or add a related
   `TestSubjectSpec` type in
   `crates/core/src/other_device.rs`, parallel to `OtherDeviceBehavior`,
   describing an RJ45 socket with the pin roles a plugged cable exposes, a
   closed set of named pair mappings and continuity faults, and per variant
   the electrical branches that variant contributes to the solver — reusing
   the existing resistive contract rather than inventing new electrical math.
2. Selecting a variant is a fixture-authoring-time or `InitialConditions`
   choice (mirroring how `control_ratios` is already threaded through),
   never a per-frame or animation-driven change.
3. Property tests: for each cable variant, assert the solver produces the
   electrically-correct continuity readout; use reproducible seeds where
   randomness picks the presented variant.
4. Once the remaining cable type lands, revisit `CAT-S07-12` as its own
   scheme-implementation slice with its own ledger/evidence update, not
   bundled into the model-landing commit.

This is a plan record, not an active Codex Goal — per AGENTS.md, do not
start implementing it until explicitly requested.

## Completion and fukit handoff

Set `Status: ready_for_fukit` only after all 24 rows have reconciled source,
capability, fixture, electrical, automated, and release evidence. The card is not
complete merely because a timer or LED fixture renders.
