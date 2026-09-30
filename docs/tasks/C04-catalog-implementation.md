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

- S06-01's passive-piezo component blocker is closed by T29:
  `PiezoPassive` is a fixed-resistance load whose visual/audio state is driven
  by recent calculated fixed-step current variation. The catalog row still
  needs its bounded fixture, menu entry, and source/buildability evidence;
  T29 does not authorize implementing that row here.
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
- S06-06 is admitted with the bounded reverse-breakdown NPN contract added for
  its first transistor, the existing audio-amplifier contract for the LM386
  stage, and a two-terminal rheostat approximation for the source pot wiper.
  S06-07 is admitted through the shared `OtherDeviceSpec` with explicit
  input/output jack pins and a calculated fixed-step DC input proxy; do not
  represent that proxy as a true guitar waveform or scripted audio.
- S06-09 uses LED/photoresistor optical coupling; the existing photoresistor
  control is not an optical component link, so the coupling must be defined
  before admission. S06-10, S06-12, and S06-13 require additional microphone,
  op-amp, or multi-stage audio contracts and remain blocked until those
  contracts exist. S06-11 is admitted with bounded microphone-proxy,
  transformer, and ring-modulator contracts; its recorded-audio, magnetic,
  and exact-timbre requirements remain explicit discrepancies.
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
- Record unsupported microphone, input-jack, op-amp,
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
- [x] Manual Linux interaction and representative breadboard checks are
  optional follow-up evidence; automated checks remain the acceptance gate.

## Implementation evidence so far

The first admitted C04 fixtures are `CAT-S06-01` (active and passive piezo),
`CAT-S06-02` (cricket), `CAT-S06-04`
(metronome), `CAT-S06-05` (doorbell), `CAT-S06-08` (police siren),
`CAT-S06-03` (electronic piano), `CAT-S06-06` (noise generator), and
`CAT-S06-07` (guitar fuzz). CAT-S06-01 uses one 9 V source and two
parallel 100 Ω fixed-resistance loads, preserving the source's direct-rail
comparison: the active buzzer draws the same 90 mA calculated DC current as
the passive piezo, while the passive-piezo history remains silent under
steady DC. Its fixture validates as 3 components, 4 wires, and 2 derived
nodes; the focused 64-step regression, 4,000-step simulation, full workspace
suite, and native/WASM builds passed. Exact physical load impedance and
audibility remain source/model discrepancies, and manual breadboard evidence
is optional follow-up work. S06-02 uses two calculated timer
stages and one adjustable burst control; it validates as 14 components, 9
wires, and 10 derived nodes, with a 4,000-step speaker-load regression. S06-04
uses one calculated 555 timing path, one adjustable control, one LED branch,
and an isolated speaker load. S06-05 uses a button-triggered 555 monostable
with diode and transistor companion branches. The admitted fixtures have structural
validation, fixed-step simulation, embedded app-menu entries, and focused core
regressions; optional browser evidence remains pending. S06-08 adds a second
calculated timer, an adjustable slow-rate control, a speaker load, and a finite
SPDT mode branch; its exact control-voltage sweep remains a source discrepancy.
S06-03 adds eight button/potentiometer key branches around one calculated 555
tone path and speaker load; exact one-key frequency selection remains a source
discrepancy.
S06-09 adds a calculated 555 tremolo with speed/depth controls, an LED-driven
voltage-controlled-resistance vactrol contract, and bounded calculated input
and output jack contracts. It validates as 14 components, 5 wires, and 11
derived nodes; thermoshrink remains a presentation discrepancy rather than an
electrical result.

The S06-09 focused gate passed with `cargo fmt --all --check`,
`cargo clippy --workspace --all-targets --locked -- -D warnings`, the focused
app regression `cargo test -p bredboard-app --locked
c04_s06_09_calculates_vactrol_and_audio_contracts`, unique-hole and menu
reachability tests, native and WASM app builds, fixture validation and
4,000-step simulation, `validate-catalog` (20 sections, 212 schematics), and
`git diff --check`. The full `cargo test --workspace --locked` invocation was
started but did not terminate in this run while two existing cycle-stability
tests remained active; it is not recorded as passed here.

S06-06 adds a deterministic fixed-step reverse-breakdown noise branch on Q1,
a common-emitter Q2 stage, and the existing bounded audio-amplifier contract.
Its fixture validates as 12 components, 3 wires, and 10 derived nodes; the
focused regression covers 2,000 steps of Q1 emitter variation and speaker
current. The 9 V source satisfies the source's reverse-breakdown threshold;
the LM386 package and three-terminal pot wiper remain explicit source
discrepancies represented by the existing bounded contracts.

The S06-06 focused gate passed with `cargo test -p bredboard-core --locked`
(135 tests), `cargo test -p bredboard-app --locked` (52 tests), the focused
reverse-breakdown property and fixture regressions, project validation (12
components, 3 wires, 10 derived nodes), 4,000-step simulation, and
`validate-catalog` (20 sections, 212 schematics). The app suite also passed
the unique lead/wire-hole invariant after the coupling capacitor was moved to
an unused hole on the same Q2 collector contact row.

The S06-06 full gate passed on 2026-09-30 with `cargo fmt --all --check`,
`cargo clippy --workspace --all-targets --locked -- -D warnings`,
`cargo test --workspace --locked` (52 app, 135 core, 3 tools, 0 doc-test
failures), native and WASM app builds, fixture validation and 4,000-step
simulation, `validate-catalog` (20 sections, 212 schematics), and `git diff
--check`.

S06-11 adds `fixtures/projects/c04-s06-11-robot-voice.json` and a reachable
menu entry. It uses a calculated electret-input proxy, one LM358 channel, a
fixed-step NE555 carrier, two bounded 1:1 transformer transfers, a four-diode
ring path, and a bounded LM386-style speaker load. The fixture validates as 23
components, 0 wires, and 12 derived nodes; the second LM358 channel,
recorded microphone waveform, magnetic transient behavior, and exact robot
timbre are not silently claimed.

The S06-05 full gate passed on 2026-09-29 with: `cargo fmt --all --check`,
`cargo clippy --workspace --all-targets --locked -- -D warnings`,
`cargo test --workspace --locked` (49 app, 92 core, 3 tools, 0 doc-test
failures), native and WASM app builds, fixture validation and 4,000-step
simulation, `validate-catalog` (20 sections, 212 schematics), and `git diff
--check`.

The S06-08 full gate passed on 2026-09-29 with `cargo fmt --all --check`,
`cargo clippy --workspace --all-targets --locked -- -D warnings`,
`cargo test --workspace --locked` (49 app, 93 core, 3 tools, 0 doc-test
failures), native and WASM app builds, fixture validation and 4,000-step
simulation, `validate-catalog` (20 sections, 212 schematics), and `git diff
--check`. The focused gate included `cargo test -p bredboard-core
c04_siren_composes_dual_timers_and_switchable_mode_load --locked`, the app
catalog test, fixture validation (`18 components, 9 wires, 13 derived nodes`),
and 4,000-step simulation.

The S06-03 focused gate passed with `cargo test -p bredboard-core
c04_piano_exposes_eight_calculated_key_branches --locked`, the app catalog test,
fixture validation (`24 components, 19 wires, 16 derived nodes`), and 4,000-step
simulation. The S06-03 full gate passed on 2026-09-29 with `cargo fmt --all
--check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`,
`cargo test --workspace --locked` (49 app, 94 core, 3 tools, 0 doc-test
failures), native and WASM app builds, fixture validation and 4,000-step
simulation, `validate-catalog` (20 sections, 212 schematics), and `git diff
--check`.

`CAT-S06-07` is now admitted with `fixtures/projects/c04-s06-07-guitar-fuzz.json`.
The fixture uses the shared `OtherDeviceSpec` for explicit two-pin input and
output jack contracts, preserves the source's two NPN stages and passive
values, and uses a bounded fixed-step DC input proxy because the current core
does not generate time-varying guitar audio. Its focused gate passed with
`cargo test -p bredboard-core
c04_guitar_fuzz_maps_jacks_and_calculates_clipped_signal_path --locked`,
fixture validation (`14 components, 12 wires, 7 derived nodes`), the app
catalog/embedded-fixture checks, and a 4,000-step transient regression.
True guitar waveform and audio playback remain explicit source discrepancies;
manual evidence is pending.

CAT-S06-10 now has a bounded candidate at
`fixtures/projects/c04-s06-10-light-music.json` and a menu entry. The candidate
validates as 34 components, 4 wires, and 16 derived nodes; its focused
three-LED regression passed. It remains `blocked_component` in the ledger:
`MIC1` is an explicit fixed Thevenin test source and `U1`–`U4` are generic
linear transfers, so this evidence does not claim live microphone capture,
TL074 package mapping, or frequency-selective light response.

The S06-11 focused gate passed on 2026-09-30 with the robot-voice core
regression, deterministic ring-modulator property tests, the app catalog
reachability/count test, fixture validation (`23 components, 0 wires, 12
derived nodes`), and a 4,000-step simulation. The S06-11 full gate passed on
2026-09-30 with `cargo fmt --all --check`,
`cargo clippy --workspace --all-targets --locked -- -D warnings`,
`cargo test --workspace --locked` (52 app, 135 core, 3 tools, 0 doc-test
failures), `cargo build -p bredboard-app --target x86_64-unknown-linux-gnu
--locked`, `cargo build -p bredboard-app --target wasm32-unknown-unknown
--locked`, fixture validation, 4,000-step simulation, catalog validation (20
sections, 212 schematics), and `git diff --check`.

## Current blockers

- External audio inputs, recorded microphone waveforms, full op-amp and
  transformer fidelity, analog multipliers, optical coupling, oscilloscope
  presentation, and the remaining multi-stage audio contracts are not yet
  available. T29's passive-piezo distinction and reverse-breakdown noise
  modeling are available; CAT-S06-01, CAT-S06-06, CAT-S06-07, CAT-S06-09,
  and CAT-S06-11 now have bounded fixtures and menu entries, with S06-07
  admitted only through its documented fixed-step DC input proxy (no true
  guitar waveform or scripted audio) and S06-11 admitted only through its
  bounded electret/transformer/ring-modulator proxies (no recorded
  microphone waveform or magnetic transient model).
- CAT-S06-10's candidate specifically remains blocked on a calculated
  microphone/audio-input contract and a TL074 four-channel filter contract;
  do not promote its fixed test-source candidate to `fixture_ready` without
  those findings being resolved.
- Manual interaction and real-breadboard evidence are pending for all new
  fixtures.

## Completion and fukit handoff

Set `Status: ready_for_fukit` only after all 27 rows have reconciled source,
capability, fixture, electrical, automated, and release evidence. The card is not
complete merely because a speaker sprite or timer fixture renders.
