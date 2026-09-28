# C02 — First integrated-circuit fixtures

Status: pending; the Section 03 source audit is recorded and implementation is
blocked on the smallest calculated IC contracts listed below.

## Scope

Implement only the seven schematics in source section 03, “First ICs”:

`CAT-S03-01` through `CAT-S03-07`.

The source records are in
[`03-level-3-first-ics.md`](../../breadboard-circuits/spec/03-level-3-first-ics.md)
and the SVG references are under
`breadboard-circuits/spec/svg/03-level-3-first-ics/`.

This batch must preserve the existing hole-derived topology, fixed-step
simulation, stable project JSON contract, and English player-facing content.
It does not authorize C03, free assembly, a generic lesson engine, or
scripted outputs.

## Source and design review

- S03-01 and S03-02 use an NE555 in astable and monostable configurations.
  The fixture supply is normalized to 5 V; the source's 9 V label and the
  “10 seconds” claim need bounded timing text and a documented observable.
- S03-03 uses 74HC00, 74HC08, 74HC32, and 74HC86 with two button inputs.
  Gate pin maps and unused inputs must be explicit; each gate output must be
  calculated from its electrical input levels.
- S03-04 uses a 74HC00 cross-coupled RS latch. It needs deterministic reset
  state and explicit set/reset action semantics; it must not be a button-only
  scripted toggle.
- S03-05 uses 74HC14 and 74HC74. The “before/after” debounce claim needs a
  fixed-step observable and a documented initial state; a D-flip-flop state
  contract is required.
- S03-06 uses an LM393 and an NTC. The open-collector output, comparator
  polarity, and threshold control must be calculated; the source's
  “heater” is represented only by a bounded LED/readout load.
- S03-07 uses an NE555, photoresistor, and speaker. Frequency is a calculated
  timing/readout contract; audio presentation remains separate from the core,
  and the 8 ohm load must stay within the existing speaker bounds.

## Required capability slice

- Add explicit component contracts and named pins for `timer_555`,
  `logic_gate`, `schmitt_inverter`, `d_flip_flop`, and `comparator` only if
  the source fixtures require them. Parameters and supported ranges must be
  validated in the core schema.
- Calculate logic levels from node voltages and expose deterministic state
  transitions through fixed simulation steps. Do not encode a circuit ID or
  fixture-specific truth table in the solver.
- Model 555 timing through its pins and connected RC components, with bounded
  output/current behavior and no wall-clock reads.
- Model LM393's open-collector output and NTC control as calculated electrical
  behavior. Record the educational approximation and limits.
- Add sprites or reuse a clearly documented IC package sprite, with pin
  labels and placement tests covering every occupied pin hole.

## Acceptance criteria

- [ ] All seven source records have reviewed Markdown, BOM, SVG, pin map,
  supply, safety, and source-discrepancy evidence in the ledger.
- [ ] Every admitted BOM item maps to an implemented model or an explicit
  recorded disposition; no scripted electrical result is used.
- [ ] Each admitted fixture validates, derives topology from holes/pins/wires,
  has stable IDs, readable placement, and a reachable menu entry.
- [ ] Logic truth tables, latch/reset behavior, debounce observables, 555
  timing, comparator threshold behavior, and speaker bounds have calculated
  regression/property coverage with reproducible seeds where applicable.
- [ ] Exact formatting, Clippy, workspace tests, Linux/WASM builds,
  catalog/schema validation, and per-fixture validation results are recorded.
- [ ] Required manual Linux interaction and representative breadboard checks
  are recorded separately; builds and tests are not treated as manual proof.

## Current blockers

- No IC component kind or pin contract exists in the core model.
- The current simulation state has capacitor internals but no general
  sequential IC state contract for an RS latch or D flip-flop.
- The source describes analog timing and audio behavior that must be bounded
  before fixture acceptance.

## Completion and fukit handoff

Set `Status: ready_for_fukit` only after all seven entries have reconciled
source, capability, fixture, electrical, manual, and release evidence. The
card is not complete merely because the new component enum serializes or an
SVG renders.
