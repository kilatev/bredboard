# C05 — Home-tool and power-energy fixtures

Status: in_progress; the source review is recorded and implementation starts
with the Section 07 timer entry whose existing timer, RC, control, buzzer, LED,
and transistor contracts are sufficient.

## Scope

Implement and disposition the 24 schematics in source sections 07 and 10:

- `CAT-S07-01` through `CAT-S07-15`, home tools;
- `CAT-S10-01` through `CAT-S10-09`, power and energy.

Source records are [`07-home-tools.md`](../../breadboard-circuits/spec/07-home-tools.md)
and [`10-power-energy.md`](../../breadboard-circuits/spec/10-power-energy.md).

## Source and design review

- S07-01, S07-02, S07-03, S07-04, S07-07, S07-11, and S07-12 require probes,
  unknown-device sockets, zener behavior, module contracts, or physical props.
- S07-05, S07-06, S07-08, S07-13, S07-14, and S07-15 require regulator,
  comparator, counter, measurement, or module contracts not present in the
  current core. Keep them blocked rather than presenting scripted readings.
- S07-09 can use the existing timer/RC, button-as-reed-contact substitute,
  buzzer, resistor, and capacitor contracts. The physical magnet and door are
  presentation discrepancies.
- S07-10 can use the existing timer/RC, button, buzzer, LED, potentiometer,
  and PNP contracts. Its exact two-minute calibration remains a bounded timing
  discrepancy.
- S10-01, S10-02, and S10-04 require physical electrochemical cells, batteries,
  or inductors. S10-03 requires an AC source and bridge rectifier. S10-05
  requires a supercapacitor and Schottky diode. S10-06 requires a Li-ion
  charger module. S10-07 requires a low-voltage CMOS timer, MOSFET, inductor,
  Schottky diode, and zener. S10-08 requires a shunt amplifier and bargraph;
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

Focused evidence for the slice:

```text
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c05-s07-09-refrigerator-guard.json — passed; 7 components, 5 wires, 5 derived nodes
cargo run -p bredboard-tools --locked -- simulate fixtures/projects/c05-s07-09-refrigerator-guard.json 4000 — passed; transient state advanced without diagnostics
cargo test -p bredboard-app all_embedded_boards_have_unique_lead_and_wire_holes --locked — passed
```

## Current blockers

- Physical probes, unknown-device sockets, AC sources, battery and lemon-cell
  models, regulators, comparators, counters, measurement modules, inductors,
  MOSFETs, zeners, Schottky diodes, supercapacitors, charger modules, solar
  sources, and bargraph instrumentation are not yet available.
- Manual interaction and real-breadboard evidence are pending for all new
  fixtures.

## Completion and fukit handoff

Set `Status: ready_for_fukit` only after all 24 rows have reconciled source,
capability, fixture, electrical, automated, and release evidence. The card is not
complete merely because a timer or LED fixture renders.
