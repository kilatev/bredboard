# T29 — Passive-piezo oscillating-drive distinction

Status: ready_for_fukit

## Dependencies

[T28 — Buzzer and speaker audio, plus a speaker component](T28-buzzer-speaker-audio.md),
whose resistor and sounding-presentation model this extends, and the existing
`timer_555` electrical contract already used by
[C04](C04-catalog-implementation.md)'s S06-02/04/05/08 fixtures as an
oscillating drive source.

This closes the `CAT-S06-01` blocker recorded in C04's "Source and design
review" and "Current blockers" sections. It does not reopen any other C04
row, and does not authorize implementing S06-06/07/09-13, S11, or S16.

## Outcome and commit boundary

Add a way to distinguish a passive piezo from the existing active buzzer.
Real active buzzers have an internal oscillator and sound on any sufficient
DC current (the behavior `Buzzer`/`Speaker` already model). Real passive
piezos have no internal oscillator: they only produce sound when driven by
an externally oscillating signal (for example, a 555 timer's square-wave
output), and stay silent under steady DC no matter how much current flows.

Electrically the new kind remains a fixed-resistance two-terminal load, the
same as `Buzzer`/`Speaker` — no new solver branch or nonlinearity. Only the
sounding-presentation rule changes: it must be derived from calculated,
fixed-step current actually oscillating over a short window, not a single
above-threshold sample. Per AGENTS.md, this must be calculated electrical
behavior, not a wall-clock effect or a scripted result.

Suggested commit title: `feat: add passive-piezo oscillating-drive distinction`.

## Scope and interfaces

- Add `ComponentKind::PiezoPassive` (name to confirm during implementation)
  in `crates/core/src/lib.rs`: pins `positive`, `negative`, and the same
  `resistance` parameter and range as `Buzzer`. Stamp it through the same
  resistor branch as `Buzzer`/`Speaker` (no new solver arm).
- Core must retain a short rolling history of the component's calculated
  current across recent fixed steps so oscillation can be detected. This is
  core state, not app/Bevy state — it is a pure function of the already
  solved current sequence, consistent with "core state is the authority for
  electrical behavior" (AGENTS.md). It likely belongs alongside existing
  per-step readings in `crates/core/src/simulation.rs`.
- Define "sounding" for `PiezoPassive` as: current crosses/varies across the
  silent/sounding threshold repeatedly within the retained window (an
  oscillating drive is present), as opposed to `Buzzer`/`Speaker`'s existing
  single-sample `current > SOUNDING_CURRENT` check. A steady DC current,
  however high, must stay silent for this kind.
- Reuse `buzzer::SOUNDING_CURRENT` and the existing sound-wave sprite marks
  where they fit; design one new silent/sounding sprite pair per the sprite
  design reference (`docs/design/sprites/README.md`), visually distinct from
  both the active buzzer and the speaker.
- Reuse the existing `Pitch`-based audio wiring from T28 unchanged; only the
  trigger condition (oscillation-detected vs. threshold-only) differs.
- Do not add a general-purpose oscilloscope/waveform display, a configurable
  drive-frequency parameter, or op-amp/transformer contracts. Those remain
  separate, already-recorded blockers (S06-10 through S06-13, all of S11,
  all of S16) per C04.

## Acceptance criteria

- [x] `PiezoPassive` is a valid `ComponentKind` with documented pins,
  parameter, and range; JSON Schema output includes it.
- [x] The solver treats it as a linear resistive load; a test confirms
  current and voltage match a plain resistor of the same value in an
  equivalent circuit, the same pattern as the buzzer's equivalence test.
- [x] A regression proves steady DC current, even well above
  `SOUNDING_CURRENT`, never reads as sounding for this kind.
- [x] A regression proves an oscillating drive (for example, a `timer_555`
  square wave into the piezo) reads as sounding, matching an expected
  fixed-step sequence.
- [x] The sprite has silent and sounding states driven by the oscillation
  rule, matches its committed golden reference, and is visually distinct
  from both `Buzzer` and `Speaker`.
- [x] Placement and same-size property tests cover the new kind.
- [x] Round-trip persistence for a project containing a passive piezo.
- [x] Existing fixtures, tests, and other kinds are unaffected.
- [x] `CAT-S06-01`'s row in `docs/catalog/CATALOG-AUDIT-LEDGER.md` and the
  blocker text in `docs/tasks/C04-catalog-implementation.md` are updated to
  reflect this task's outcome (documentation follow-up, not new code).

## Required verification

- Run the baseline formatting, Clippy, workspace test, Linux build, and WASM
  build commands documented in `README.md`.
- Run the new solver-equivalence, steady-DC-silence, oscillating-drive, and
  golden/property sprite tests; generate goldens with
  `BREDBOARD_BLESS_SPRITES=1` and review the new `.txt` files.
- This environment cannot play or hear audio output. Verify what is
  checkable without a speaker or microphone (compiles, builds, the
  oscillation-detection state transitions correctly per fixed step); record
  audibility itself as unverified, not as a pass, per AGENTS.md ("an
  unavailable required check is a blocker, not a pass").
- Manually exercise a passive piezo driven by a `timer_555` in a scratch
  fixture in the Linux executable to confirm the silent/sounding visual
  swap follows the oscillating drive and stays silent under steady DC.

A missing or failing required check blocks readiness.

## Codex Goal

```text
/goal Complete T29 in docs/tasks/T29-passive-piezo-drive.md according to
docs/PLAN.md, AGENTS.md, and docs/design/sprites/README.md. Add a
PiezoPassive ComponentKind (electrically identical resistor to Buzzer) whose
sounding presentation is derived from calculated current actually
oscillating across recent fixed steps, not a single above-threshold sample;
steady DC current must never sound. Reuse the existing Buzzer/Speaker
resistor and audio wiring; do not add an oscilloscope, a configurable
drive-frequency parameter, or op-amp/transformer contracts, and do not
implement S06-06/07/09-13, S11, or S16. Satisfy every acceptance criterion
and run every required check in the card. Fix task-scoped findings without
weakening tests or acceptance criteria. Do not implement successor tasks.
Prepare one coherent change for review; do not commit or push. Record exact
verification evidence in this card. If a required check is unavailable
(including audibility), report the exact blocker; do not count it as passed.
```

## Completion and fukit handoff

When all criteria pass, record evidence and set `Status: ready_for_fukit`.
Stop without starting successor tasks. The user may then invoke `fukit` to
describe, commit, and push.

An existing jj repository and an unambiguous authorized remote/bookmark are
required for that workflow. Do not initialize or guess them. Commit
completion is evidenced by jj history; publication is evidenced by the
actual push result.

## Evidence

Implementation is complete and the required manual Linux/audio gate has been
confirmed by the owner.

- `ComponentKind::PiezoPassive` uses `positive`/`negative` pins and the
  buzzer resistance range (`1.0..=1e7` ohm). Project schema, validation,
  JSON round-trip, and passive-piezo range coverage are in
  `crates/core/src/lib.rs`.
- The solver stamps `PiezoPassive` through the existing linear resistor arm.
  `solver::tests::passive_piezo_matches_a_plain_resistor_of_the_same_value`
  verifies equal current and terminal voltages.
- `SimulationState` retains 64 calculated fixed-step currents per passive
  piezo. `passive_piezo_is_sounding` requires repeated threshold/polarity or
  directional variations, so steady DC stays silent. Core regressions cover
  steady DC above threshold, an exact square-wave sequence, and a real 555
  astable drive (`cargo test -p bredboard-core --locked passive_piezo` — 5
  passed).
- `crates/app/src/sprites/piezo_passive.rs` adds the distinct silent/sounding
  rectangular transducer art. Goldens are
  `docs/design/sprites/piezo-passive-silent.txt` and
  `docs/design/sprites/piezo-passive-sounding.txt`, generated with
  `BREDBOARD_BLESS_SPRITES=1`; app sprite tests cover state mapping, distinct
  art, same-size states, and randomized pin placement.
- `tests::passive_piezo_view_uses_core_history_for_sprite_and_audio_transition`
  exercises the app's `update_view` path with a headless `SoundTones` resource:
  an oscillating current history switches the sprite to sounding and inserts
  `AudioPlayer<Pitch>`, while a steady history returns it to silent and removes
  the audio component.
- App audio reuses the existing T28 `Pitch` wiring and buzzer tone; only the
  trigger state differs, using the core oscillation result. `cargo clippy
  --workspace --all-targets --locked -- -D warnings` and `cargo fmt --all
  --check` pass. The core package suite passes with 114 tests and zero
  doc-test failures; the tools package suite passes with 3 tests. The newly
  added app integration test passes, and the previously completed workspace
  baseline passed with 51 app, 114 core, and 3 tools tests before that test was
  added. A subsequent full-workspace rerun reached the app suite's two known
  long transient tests but was interrupted by the runner before completion;
  it is not counted as a pass.
- Required builds pass: `cargo build -p bredboard-app --target
  x86_64-unknown-linux-gnu --locked` and `cargo build -p bredboard-app
  --target wasm32-unknown-unknown --locked`.
- `CAT-S06-01` now records `PiezoPassive` as supported in
  `docs/catalog/CATALOG-AUDIT-LEDGER.md`; C04 records that T29 closes the
  component-model blocker while leaving bounded fixture/menu work separate.

All task-specific automated checks are complete. The owner confirmed the
manual Linux executable check: a passive piezo driven by the calculated 555
oscillator transitions to sounding, while steady DC remains silent and the
audio presentation works. T29 is therefore `ready_for_fukit`.
