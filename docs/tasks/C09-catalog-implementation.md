# C09 — Biosignal/science and relay-logic fixtures

Status: in_progress; source review is recorded and implementation starts with
the first relay switch entry. Biosignal entries remain blocked or physical
scope until their sensor, instrumentation, and safety contracts are explicit.

## Scope

Implement and disposition the 14 schematics in source sections 13 and 15:

- `CAT-S13-01` through `CAT-S13-07`;
- `CAT-S15-01` through `CAT-S15-07`.

The source records are `13-biosignals-science.md` and `15-relay-logic.md`.

## Source and design review

- S13-01 needs a J-FET and a physical antenna/charged-object interaction; keep
  it blocked until a sensor/control contract exists.
- S13-02 needs skin electrodes and a 555/speaker signal path; keep it
  physical scope because the source behavior depends on a real skin-contact
  prop and a safety-reviewed battery-only experiment.
- S13-03 needs a piezo sensor, op-amp, rectifier, and bargraph driver; keep it
  blocked until those measurement contracts exist.
- S13-04 needs a Hall sensor, magnetic prop, and bargraph driver; keep it
  physical scope until a deterministic field-control contract is defined.
- S13-05 needs an IR emitter/phototransistor optical path and op-amp signal
  conditioning; keep it blocked until the light-sensor contract exists.
- S13-06 needs a random-delay interaction, 555 timing, 4013 state, three
  counters, and three seven-segment displays; keep it blocked until its timing
  and measurement contract is defined.
- S13-07 needs battery-only skin electrodes and an instrumentation amplifier;
  keep it physical scope and preserve the source safety restriction.
- S15-01 through S15-07 require relay coil/contact behavior. The relay model
  will be electrical and deterministic: a resistive coil energizes at a
  threshold and selects a common-to-NO or common-to-NC contact. Mechanical
  click, contact bounce, and real coil inductance remain documented
  discrepancies, not scripted electrical outcomes.
- S15-03 intentionally omits a flyback diode so the relay can self-interrupt;
  preserve that source-design exception and do not silently add protection.
- S15-05 and S15-07 exceed one-board density and remain physical scope even
  after relay behavior is modeled.

## Acceptance criteria

- [ ] All 14 source records have source, BOM, SVG, pin, supply, safety, and
  discrepancy evidence in the ledger.
- [ ] Every admitted relay BOM item maps to the implemented relay contract or
  an explicit recorded discrepancy; no click or relay outcome is scripted.
- [ ] Each admitted fixture validates, preserves derived topology, has stable
  IDs, readable placement, and a reachable menu entry.
- [ ] Relay coil current and contact selection have calculated regressions;
  the unprotected self-interrupting source design remains called out.
- [x] Manual Linux interaction and representative breadboard checks are
  optional follow-up evidence; automated checks remain the acceptance gate.

## Implementation evidence so far

The first admitted fixture is `CAT-S15-01`,
`fixtures/projects/c09-s15-01-relay-switch.json`. It adds a calculated
five-terminal relay: a momentary button drives a 4.5 V resistive coil, the
coil threshold selects its common-to-NC/NO contact, a flyback diode is present
in the source-listed coil path, and the contact side switches an isolated 9 V
LED load. Mechanical click, bounce, and inductance remain explicit
discrepancies.

The C09-S15-01 full gate passed on 2026-09-29 with `cargo fmt --all --check`,
`cargo clippy --workspace --all-targets --locked -- -D warnings`,
`cargo test --workspace --locked` (49 app, 101 core, 3 tools, 0 doc-test
failures), native and WASM app builds, fixture validation (`8 components, 6
wires, 8 derived nodes`), 4,000-step simulation, `validate-catalog`
(`20 sections, 212 schematics`), and `git diff --check`. Manual browser and
real-breadboard evidence remain pending.

## Current blockers

- J-FET/antenna, skin electrode, piezo, Hall, optical sensor, instrumentation
  amplifier, and reaction-timing contracts are not available.
- Relay mechanical sound, bounce, inductance, and multi-board density are not
  represented by the current product surface.
- Manual interaction and real-breadboard evidence are pending for any new
  fixture.

## Completion and fukit handoff

Set `Status: ready_for_fukit` only after all 14 rows have reconciled source,
capability, fixture, electrical, automated, and release evidence. The card is not
complete merely because one relay fixture validates.
