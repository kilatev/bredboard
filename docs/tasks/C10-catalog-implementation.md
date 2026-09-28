# C10 — Hacks and improvised-material fixtures

Status: in_progress; source review is recorded and implementation starts with
the diode-input logic entry. Entries that depend on physical materials, donor
hardware, probes, or unsupported IC contracts remain explicitly blocked or
physical scope.

## Scope

Implement and disposition the 16 schematics in source section 18:

- `CAT-S18-01` through `CAT-S18-16`.

The source record is `18-hacks.md`.

## Source and design review

- S18-01 needs a drawn graphite track, two crocodile probes, and continuous
  user-authored resistance; keep it physical scope until a drawing/control
  contract exists.
- S18-02 needs a foil/cardboard pressure prop; keep it physical scope.
- S18-03 needs two external probes and an unknown polarity input; keep it
  physical scope until an instrument/control contract exists.
- S18-04 needs an LED used as a light sensor and LM358 buffering; keep it
  blocked until reverse-photoresponse and measurement contracts exist.
- S18-05 needs a speaker-as-microphone input and LM386 signal path; keep it
  blocked until an audio-input/amplifier contract exists.
- S18-06 needs a self-flashing LED timing model; keep it blocked until that
  component contract exists.
- S18-07 is the first admissible slice: two buttons, two diodes, an NPN,
  resistors, and an LED can calculate a diode-input OR path. The source title
  mentions both AND and OR but lists one output; this fixture accepts the OR
  behavior and records the missing second-output interpretation.
- S18-08 needs fruit touch props, four Darlington channels, and a 555/audio
  path; keep it physical scope and blocked on the touch contract.
- S18-09 needs a handmade LED/photocell optical path and heat-shrink prop;
  keep it blocked until the optical coupling contract exists.
- S18-10 needs six independent Schmitt oscillators and their capacitors; keep
  it blocked until the source's six-channel contract is specified.
- S18-11 needs a donor talking toy and soldering/black-box presentation; keep
  it physical scope.
- S18-12 needs an eight-button resistor ladder and LM3914/bargraph display;
  keep it blocked until analog measurement/display behavior is available.
- S18-13 needs an unbuffered inverter's analog feedback behavior and audio
  jacks; keep it blocked until that contract exists.
- S18-14 needs a 555 used as a threshold device without timing capacitors;
  keep it blocked until the source-specific threshold contract is recorded.
- S18-15 needs a Peltier element, ferrite transformer, thermal props, and a
  boost-converter model; keep it physical scope and blocked.
- S18-16 is a donor-hardware disassembly and multi-board composition; keep it
  physical scope. Laser-diode reuse remains explicitly excluded.

## Acceptance criteria

- [ ] All 16 source records have source, BOM, SVG, pin, supply, safety, and
  discrepancy evidence in the ledger.
- [ ] The admitted diode-logic fixture maps every electrical BOM item to a
  calculated model and records the source's AND/OR ambiguity.
- [ ] The fixture validates, preserves derived topology, has stable IDs,
  readable placement, and a reachable menu entry.
- [ ] Diode-input OR behavior and NPN/LED current have a calculated regression
  plus a bounded property check over supported values.
- [ ] Required manual Linux interaction and representative breadboard checks
  are recorded separately; builds and tests are not treated as manual proof.

## Implementation evidence so far

The first admitted fixture is `CAT-S18-07`,
`fixtures/projects/c10-s18-07-diode-logic.json`. It adds a calculated 4.5 V
two-button diode-input OR path driving an NPN and current-limited LED. All
electrical BOM items are represented; the source's missing second output for
the AND/OR description remains an explicit content discrepancy.

The C10-S18-07 full gate passed on 2026-09-29 with `cargo fmt --all --check`,
`cargo clippy --workspace --all-targets --locked -- -D warnings`,
`cargo test --workspace --locked` (49 app, 103 core, 3 tools, 0 doc-test
failures), native and WASM app builds, fixture validation (`11 components, 5
wires, 9 derived nodes`), 4,000-step simulation, `validate-catalog`
(`20 sections, 212 schematics`), and `git diff --check`. Manual browser and
real-breadboard evidence remain pending.

## Current blockers

- User-authored resistance drawing, physical pressure/probe/touch/thermal
  props, optical/audio input, donor hardware, self-flashing LED, and missing
  analog/IC contracts are not represented by the current product surface.
- Manual interaction and real-breadboard evidence are pending for any new
  fixture.

## Completion and fukit handoff

Set `Status: ready_for_fukit` only after all 16 rows have reconciled source,
capability, fixture, electrical, manual, and release evidence. The card is not
complete merely because one diode-logic fixture validates.
