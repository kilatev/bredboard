# C04 — Sound, op-amp, and analog-computer fixtures

Status: in_progress; the source review is recorded and implementation starts
with the Section 06 entries whose timer, RC, speaker, LED, and control
contracts already exist. The 27 source entries are split into admitted bounded
fixtures and explicit component/physical-scope dispositions.

## Scope

Implement and disposition the 27 schematics in source sections 06, 11, and 16:

- `CAT-S06-01` through `CAT-S06-13`, sound circuits;
- `CAT-S11-01` through `CAT-S11-08`, op-amp circuits;
- `CAT-S16-01` through `CAT-S16-06`, analog-computer circuits.

Source records are [`06-sound.md`](../../breadboard-circuits/spec/06-sound.md),
[`11-op-amps.md`](../../breadboard-circuits/spec/11-op-amps.md), and
[`16-analog-computer.md`](../../breadboard-circuits/spec/16-analog-computer.md).
SVG references are in the matching `breadboard-circuits/spec/svg/` folders.

This batch preserves hole-derived topology, fixed-step simulation, stable JSON
contracts, and English player-facing content. It does not add a hidden audio
script, an oscilloscope-only result, or a physical prop that the board model
cannot represent.

## Source and design review

- S06-01 requires active and passive buzzer behavior; the current electrical
  models can represent current-driven loads but do not yet distinguish a
  passive piezo's need for an oscillating drive. Keep this as a component
  blocker until the distinction is explicit.
- S06-02, S06-04, S06-05, and S06-08 use timer/RC paths that can be expressed
  with existing `timer_555`, capacitor, resistor, button, switch, LED, NPN,
  PNP, diode, and speaker contracts. Dual-timer entries may compose two
  existing timer primitives when every output remains voltage-derived.
- S06-04 is admitted first as a bounded metronome fixture. The source's
  1 MΩ/1 µF/10 nF timing values are recorded as a solver-stability
  discrepancy; the fixture uses the existing stable fixed-step timer contract
  and a 470 Ω speaker isolation resistor.
- S06-03 needs eight electrical key inputs and a bounded frequency/readout
  contract; the source's exact musical tuning remains a documented
  approximation.
- S06-06 and S06-07 require reverse-breakdown noise, external audio input, or
  input-jack contracts not present in the core. Keep them blocked rather than
  scripting noise or guitar audio.
- S06-09 uses LED/photoresistor optical coupling; the existing photoresistor
  control is not an optical component link, so the coupling must be defined
  before admission. S06-10 through S06-13 require microphone, op-amp,
  transformer, or multi-stage audio contracts and remain blocked until those
  contracts exist.
- All S11 entries require op-amp, signal-source, filter, or oscilloscope
  presentation contracts beyond the current DC/RC model. S16 additionally
  requires analog multipliers, integrator presentation, and multi-output
  oscilloscope behavior. These are component or physical-scope dispositions,
  not scripted fixture outcomes.

## Required capability slice

- Reuse existing timer, capacitor, resistor, potentiometer, button, switch,
  LED, diode, transistor, buzzer, speaker, and fixed-step controls where their
  pin-level behavior is sufficient.
- Add only the smallest new model needed by an admitted source entry, with
  named pins, parameter ranges, calculated behavior, visual state, and a
  focused regression.
- Keep presentation frequency and audio tone as readouts of calculated
  electrical state; do not use wall-clock timing or scripted sound results.
- Record unsupported passive-piezo, microphone, input-jack, op-amp,
  transformer, analog-multiplier, oscilloscope, and optical-coupling
  requirements in the ledger.

## Acceptance criteria

- [ ] All 27 source records have source, BOM, SVG, pin, supply, safety, and
  discrepancy evidence in the ledger.
- [ ] Every admitted BOM item maps to an implemented model or an explicit
  recorded discrepancy; no scripted electrical result is used.
- [ ] Each admitted fixture validates, derives topology from holes/pins/wires,
  has stable IDs, readable placement, and a reachable menu entry.
- [ ] Timer/RC, load current, control ordering, and any new signal contract
  have calculated regressions and bounded diagnostics.
- [ ] Required manual Linux interaction and representative breadboard checks
  are recorded separately; builds and tests are not treated as manual proof.

## Implementation evidence so far

The first admitted C04 fixture is `CAT-S06-04` (metronome), using one
calculated 555 timing path, one adjustable control, one LED branch, and an
isolated speaker load. It validates as 11 components, 5 wires, and 8 derived
nodes; its 4,000-step regression observes calculated LED and speaker pulses.

## Current blockers

- Passive-piezo distinction, reverse-breakdown noise, external audio inputs,
  microphones, op-amps, transformers, analog multipliers, optical coupling,
  oscilloscope presentation, and the remaining multi-stage audio contracts
  are not yet available.
- Manual interaction and real-breadboard evidence are pending for all new
  fixtures.

## Completion and fukit handoff

Set `Status: ready_for_fukit` only after all 27 rows have reconciled source,
capability, fixture, electrical, manual, and release evidence. The card is not
complete merely because a speaker sprite or timer fixture renders.
