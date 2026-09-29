# T29 — Passive-piezo oscillating-drive distinction

Status: pending

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

- [ ] `PiezoPassive` is a valid `ComponentKind` with documented pins,
  parameter, and range; JSON Schema output includes it.
- [ ] The solver treats it as a linear resistive load; a test confirms
  current and voltage match a plain resistor of the same value in an
  equivalent circuit, the same pattern as the buzzer's equivalence test.
- [ ] A regression proves steady DC current, even well above
  `SOUNDING_CURRENT`, never reads as sounding for this kind.
- [ ] A regression proves an oscillating drive (for example, a `timer_555`
  square wave into the piezo) reads as sounding, matching an expected
  fixed-step sequence.
- [ ] The sprite has silent and sounding states driven by the oscillation
  rule, matches its committed golden reference, and is visually distinct
  from both `Buzzer` and `Speaker`.
- [ ] Placement and same-size property tests cover the new kind.
- [ ] Round-trip persistence for a project containing a passive piezo.
- [ ] Existing fixtures, tests, and other kinds are unaffected.
- [ ] `CAT-S06-01`'s row in `docs/catalog/CATALOG-AUDIT-LEDGER.md` and the
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

Not started.
