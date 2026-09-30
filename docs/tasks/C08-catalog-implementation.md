# C08 — Digital-analog, light-effect, and optical communication fixtures

Status: in_progress; source review is recorded and implementation starts with
the PC817-style optocoupler entry whose two isolated electrical domains can be
calculated without simulating an external light path.

## Scope

Implement and disposition the 22 schematics in source sections 12, 14, and 17:

- `CAT-S12-01` through `CAT-S12-07`;
- `CAT-S14-01` through `CAT-S14-08`;
- `CAT-S17-01` through `CAT-S17-07`.

The source records are `12-digital-analog.md`, `14-light-effects.md`, and
`17-light-communication.md`.

## Source and design review

- S12-01 through S12-07 need DAC/ADC, op-amp, comparator, counter, diode-ROM,
  or oscilloscope/measurement contracts; keep them blocked.
- S14-02 through S14-05 need oscillator, diode steering, RGB/array, or mode
  sequencing contracts; keep them blocked. S14-01 is admitted as a bounded
  two-timer/diode/LED fixture; its candle appearance and exact random flicker
  remain presentation discrepancies.
- S14-06 and S14-08 additionally need a 3D LED cube or POV motion scene; keep
  them physical-scope.
- S14-07 needs multi-board display and large diode-matrix contracts; keep it
  blocked.
- S17-01 needs a light sensor plus comparator and two electrical domains; keep
  it blocked.
- S17-02 needs an IR emitter/receiver path and barrier scene; keep it blocked.
- S17-03 is the first admissible slice: a button, LED, resistors, two sources,
  and a calculated four-terminal optocoupler. The optocoupler's transfer is a
  voltage/current-derived electrical contract; no direct wire joins its two
  source domains.
- S17-04 through S17-07 need modulated IR, optical receiver, laser/photodiode,
  or multi-bit light-link contracts; keep them blocked or physical-scope.

## Acceptance criteria

- [ ] All 22 source records have source, BOM, SVG, pin, supply, safety, and
  discrepancy evidence in the ledger.
- [ ] Every admitted BOM item maps to an implemented model or an explicit
  recorded discrepancy; no scripted optical output is used.
- [ ] Each admitted fixture validates, preserves isolated topology, has stable
  IDs, readable placement, and a reachable menu entry.
- [ ] Optocoupler input current and output transfer have calculated regressions
  and bounded diagnostics.
- [x] Manual Linux interaction and representative breadboard checks are
  optional follow-up evidence; automated checks remain the acceptance gate.

## Implementation evidence so far

The first admitted fixture is `CAT-S17-03`,
`fixtures/projects/c08-s17-03-optocoupler.json`. It adds a calculated
four-terminal optocoupler: input LED current controls a bounded
collector-emitter conductance while the two source domains remain isolated.
The app menu exposes the button and isolated input/output current readout.

The C08-S17-03 full gate passed on 2026-09-29 with `cargo fmt --all --check`,
`cargo clippy --workspace --all-targets --locked -- -D warnings`,
`cargo test --workspace --locked` (49 app, 99 core, 3 tools, 0 doc-test
failures), native and WASM app builds, fixture validation (`7 components, 5
wires, 7 derived nodes`), 4,000-step simulation, `validate-catalog`
(`20 sections, 212 schematics`), and `git diff --check`. Manual browser and
real-breadboard evidence remain pending.

The parallel C08-S14-01 slice adds
`fixtures/projects/c08-s14-01-candle-flicker.json` and registers it in the app
menu. It uses two calculated 555 timing stages, diode isolation, a current-
limited LED branch, and fixed-step capacitor state. The source candle prop
and exact random flicker are explicit presentation discrepancies; the fixture
does not script an output.

Focused evidence for the slice:

```text
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c08-s14-01-candle-flicker.json — passed; 13 components, 9 wires, 8 derived nodes
cargo run -p bredboard-tools --locked -- simulate fixtures/projects/c08-s14-01-candle-flicker.json 1000 — passed; transient state advanced without diagnostics
cargo test -p bredboard-app all_embedded_boards_have_unique_lead_and_wire_holes --locked — passed
```

The shared board-model slice now provides a validated multi_board contract
for up to eight named half_size_solderless boards. Qualified
board_id/local_hole references preserve per-board strips and rails, while
explicit wire endpoints can connect boards; deterministic topology and a
calculated cross-board resistor path are covered by fixed-seed core tests.
S14-07 remains blocked because its diode matrix, decoder/driver mapping,
sequencing behavior, and two-board fixture/layout are not implemented.

## Current blockers

- DAC/ADC, op-amp, comparator, oscillator, diode-ROM, measurement, optical
  sensor, modulated-light, multi-bit light-link, large-array, and motion-scene
  contracts and S14-07's fixture-specific multi-board display semantics are
  not yet available.
- Manual interaction and real-breadboard evidence are pending for any new
  fixture.

## Completion and fukit handoff

Set `Status: ready_for_fukit` only after all 22 rows have reconciled source,
capability, fixture, electrical, automated, and release evidence. The card is
not complete merely because an isolated LED branch renders.
