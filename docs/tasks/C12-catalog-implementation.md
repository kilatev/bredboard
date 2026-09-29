# C12 — Find-the-bug fault-pair content

Status: in_progress; source review is recorded, but the section is not yet
representable by the current fixed-exercise product surface. Every source
entry is an intentionally faulty variant or a multi-fault diagnosis task;
none is admitted as a normal electrical fixture in this batch.

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

All 12 remain `out_of_scope` in the ledger until the fault-pair mode can
preserve a known-good variant, expose exactly the intended fault(s), identify
the learner-correctable cause, and apply a correction without scripted
electrical outcomes.

## Acceptance criteria

- [ ] All 12 source records have source, BOM, SVG, pin, supply, safety, and
  discrepancy evidence in the ledger.
- [ ] The product has an explicit fault-pair representation that preserves a
  known-good and faulty assembly without duplicating electrical authority.
- [ ] Fault highlighting identifies the actual learner-correctable cause at
  hole, component, pin, or board-reference level.
- [ ] A diagnosis/correction action is routed through the explicit action
  model and recalculates the electrical result.
- [ ] Unsafe variants (reversed supply/orientation, missing flyback, and
  heating claims) are bounded by safety diagnostics and never presented as a
  normal build recommendation.
- [ ] Required manual Linux interaction and representative breadboard checks
  are recorded separately; builds and tests are not treated as manual proof.

## Implementation evidence so far

No C12 fixture is accepted. The ledger records all 12 rows as `out_of_scope`
with the missing fault-mode, multi-board, actuator, IC, noise, and safety
contracts named per entry.

## Current blockers

- Fault-pair project/schema contracts, fault highlighting, diagnosis actions,
  and correction flow do not exist.
- Missing or unsafe source capabilities include 555/CMOS/counter/display
  models, motor/inductive behavior, switch bounce, multi-board references,
  power integrity, and rating/thermal diagnostics.
- Manual interaction and real-breadboard evidence are unavailable for a
  fault-mode fixture.

## Completion and fukit handoff

Set `Status: ready_for_fukit` only after the fault-pair mode and all 12 rows
have reconciled source, capability, fixture, electrical, manual, and release
evidence. The card is not complete merely because a good circuit plus a
static bad image exists.
