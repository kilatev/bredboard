# C12 — Find-the-bug fault-pair content

Status: in_progress; source review is recorded and all 12 fault-pair fixtures
are now represented by calculated projects. Manual interaction and physical
assembly evidence remain pending.

## Scope

Implement and disposition the 12 schematics in source section 20:

- `CAT-S20-01` through `CAT-S20-12`.

The source record is `20-find-the-bug.md`.

## Source and design review

- S20-01 reverses an LED in an otherwise simple source/resistor circuit; it
  needs a fault-state presentation and a learner correction action.
- S20-02 breaks a long power rail at its midpoint; it needs explicit rail
  continuity/fault highlighting rather than a silently disconnected fixture.
- S20-03 places one resistor lead on the wrong row; it needs hole-level fault
  highlighting and a correction flow.
- S20-04 shows a 22 kΩ resistor where 220 Ω is intended; it needs a bad-part
  variant and replacement diagnosis.
- S20-05 swaps NPN emitter and collector; it needs a pin-order fault variant
  and a safe comparison of the calculated behavior.
- S20-06 rotates a 555 by 180° and explicitly warns of heating in real life;
  the current product has no orientation-fault or rating/thermal contract.
- S20-07 leaves a CMOS gate input floating; it needs an undefined-input fault
  state and a pull-up correction contract.
- S20-08 omits the motor flyback diode; it needs a motor/inductive transient
  model, fault highlighting, and a safe correction state.
- S20-09 omits decoupling capacitors around a counter and motor system; it
  needs power-integrity/noise behavior rather than a scripted fault outcome.
- S20-10 connects a button directly to a counter clock; it needs switch
  bounce, debounce, and seven-segment display contracts.
- S20-11 omits the ground link between two powered boards; it needs explicit
  multi-board reference semantics and signal diagnosis.
- S20-12 is a level-5 composition with two or three simultaneous faults; it
  needs a bounded multi-fault diagnosis mode and a source project baseline.

The implementation uses a single faulty project plus explicit structural
repair metadata. `Action::RepairFault` applies pin, parameter, or wire edits,
removes the repaired fault from the active project, invalidates prior
readings, recompiles topology, and recalculates through the common solver.
There is no duplicate editable netlist or scripted electrical result.

## Acceptance criteria

- [x] All 12 source records have source, BOM, SVG, pin, supply, safety, and
  discrepancy evidence in the ledger.
- [x] The product has an explicit fault-pair representation that preserves a
  known-good and faulty assembly without duplicating electrical authority.
- [x] Fault highlighting identifies the actual learner-correctable cause at
  hole, component, pin, or board-reference level.
- [x] A diagnosis/correction action is routed through the explicit action
  model and recalculates the electrical result.
- [x] Unsafe variants (reversed supply/orientation, missing flyback, and
  heating claims) are bounded by safety diagnostics and never presented as a
  normal build recommendation.
- [ ] Required manual Linux interaction and representative breadboard checks
  are recorded separately; builds and tests are not treated as manual proof.

## Implementation evidence so far

All 12 fixtures are accepted for the current calculated fault-pair surface:
`c12-s20-01-reversed-led.json` through `c12-s20-12-broken-project.json`.
Core regressions cover the polarity/value/wiring repairs and the complete
12-fixture repair loop; the app menu exposes each exercise with the shared
REPAIR FAULT control. Source-specific thermal, noise, actuator, and display
behavior remain bounded discrepancies rather than scripted outcomes.

The automated C12 gate passed locally on 2026-09-29 with:

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked` — 49 app tests, 108 core tests, 3 tool
  tests, and 0 doc-test failures
- `cargo build -p bredboard-app --target x86_64-unknown-linux-gnu --locked`
- `cargo build -p bredboard-app --target wasm32-unknown-unknown --locked`
- `cargo run -p bredboard-tools --locked -- validate` for all 12 C12 fixtures
  — all schema/topology validations passed
- `cargo run -p bredboard-tools --locked -- validate-catalog
  breadboard-circuits/spec/catalog.json
  breadboard-circuits/spec/catalog.schema.json` — 20 sections, 212 schematics
- `git diff --check`

The 12 validation results were: S20-01 (3 components, 3 wires, 3 nodes),
S20-02 (9, 9, 9), S20-03 (4, 2, 5), S20-04 (3, 3, 3), S20-05 (6, 2, 5),
S20-06 (8, 4, 7), S20-07 (5, 1, 5), S20-08 (6, 2, 5), S20-09 (8, 1, 7),
S20-10 (8, 2, 10), S20-11 (6, 6, 6), and S20-12 (5, 6, 4).

## Current blockers

- Manual Linux interaction and real-breadboard evidence are pending.
- Source-specific thermal, noise, actuator, and physical display behavior are
  recorded discrepancies for the bounded fixtures.

## Completion and fukit handoff

Set `Status: ready_for_fukit` only after the fault-pair mode and all 12 rows
have reconciled source, capability, fixture, electrical, manual, and release
evidence. The card is not complete merely because a good circuit plus a
static bad image exists.
