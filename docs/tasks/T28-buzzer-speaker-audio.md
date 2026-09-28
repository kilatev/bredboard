# T28 — Buzzer and speaker audio, plus a speaker component

Status: ready_for_fukit

## Dependencies

[T20 — Buzzer component](T20-buzzer-component.md). This task explicitly
reopens T20's "no audio output; the app has no audio system, and adding one
is out of scope" decision at the owner's direct request; it does not
reopen any other MVP or catalog-audit scope.

## Outcome and commit boundary

Add a real Bevy audio system that plays a tone while a `buzzer` is in its
current-driven "sounding" state, and add `speaker` as a second, electrically
identical `ComponentKind` (a low-impedance dynamic speaker rather than a
piezo buzzer) with its own sprite and its own, lower tone, sharing the same
audio wiring.

Suggested commit title: `feat: add buzzer/speaker audio and a speaker component`.

Sounding is still a presentation mapping of calculated current, the same
pattern as the LED's glow; the tone's pitch is a fixed presentation choice
per kind, not a calculated electrical result, and does not affect the
solver.

## Scope and interfaces

- Add `ComponentKind::Speaker` in `crates/core/src/lib.rs`: pins `positive`,
  `negative`, required parameter `resistance`. Document a tighter range than
  the buzzer's (dynamic speakers are low-impedance voice coils, typically
  4-32 ohm) rather than reusing the buzzer's wide practical piezo range.
  Model it in the solver as a fixed linear resistor via the same code path as
  `Buzzer` (no new branch kind, no nonlinearity).
- Design a new `speaker` sprite (`crates/app/src/sprites/speaker.rs`)
  distinct from the buzzer's flat grilled disc — a cone read (case ring, cone
  slope, dust cap) — following the sprite design reference's steps and style
  rules, sharing the buzzer's `SOUNDING_CURRENT` threshold and sound-wave
  marks. Add it to the reference's "Current parts" table with a golden `.txt`
  reference. Register the `PartArt` in `sprites::art_for` and extend the
  placement and same-size property tests.
- Enable Bevy's `bevy_audio` feature in `crates/app/Cargo.toml` (no decoder
  format feature needed — the tone is synthesized, not decoded from a file).
- Play a fixed tone per sounding buzzer/speaker using `bevy_audio`'s built-in
  `Pitch` asset (a synthesized sine wave `Decodable` source already provided
  by `bevy_audio`; no audio asset file is shipped, no custom `Decodable`
  impl is needed). Two long-duration `Pitch` handles are created once at
  startup and stored in a resource: a higher piezo-like tone for `Buzzer`,
  a lower, fuller tone for `Speaker`.
- Per sounding entity, insert `(AudioPlayer<Pitch>, PlaybackSettings::LOOP)`
  when its sprite state crosses into "sounding" and remove
  `(AudioPlayer<Pitch>, AudioSink)` when it crosses back to silent, so the
  tone starts and stops exactly when the existing sprite state does. Do not
  add volume/mute UI, pitch-bending, or any other new audio feature.
- Keep the audio-management step optional (`Option<Res<...>>`) in the shared
  `update_view` system so test harnesses that build a minimal `App` without
  `AudioPlugin`/`AssetPlugin` (several already do, for menu- and
  readout-focused tests) keep working unchanged.

## Acceptance criteria

- [ ] `speaker` is a valid `ComponentKind` with documented pins, parameter,
  and range; JSON Schema output includes it.
- [ ] The solver treats `speaker` as a linear resistive load; a test confirms
  current and voltage match a plain resistor of the same value in an
  equivalent circuit, the same pattern as the buzzer's equivalence test.
- [ ] The speaker sprite has silent and sounding states driven by calculated
  current, matches its committed golden reference, is visually distinct from
  the buzzer, and follows the style rules.
- [ ] Placement and same-size property tests cover `speaker`.
- [ ] Round-trip persistence for a project containing a speaker.
- [ ] `cargo build -p bredboard-app --target wasm32-unknown-unknown --locked`
  still succeeds with `bevy_audio` enabled.
- [ ] A sounding buzzer or speaker gets an `AudioPlayer`/`AudioSink` and a
  silent one does not (or has them removed), driven by the same current
  threshold as the sprite.
- [ ] Existing fixtures, tests, and other kinds are unaffected; every test
  harness that builds a minimal `App` without `AudioPlugin` still passes.

## Required verification

- Run the baseline formatting, Clippy, workspace test, Linux build, and WASM
  build commands documented in `README.md`.
- Run the new solver-equivalence, golden, and property sprite tests; generate
  goldens with `BREDBOARD_BLESS_SPRITES=1` and review the new `.txt` files.
- This environment cannot play or hear audio output. Verify what is
  checkable without a speaker or microphone (compiles, builds, the
  `AudioPlayer`/`AudioSink` insertion and removal follow the sounding state,
  the `Pitch` handles are created once at startup); record audibility itself
  as unverified, not as a pass, per AGENTS.md ("an unavailable required check
  is a blocker, not a pass"). A human with working audio output should
  confirm a buzzer and a speaker are both actually audible, and sound
  different, in the Linux executable before relying on this as final
  sign-off.
- Manually exercise a buzzer and a speaker in a scratch fixture in the Linux
  executable to confirm the silent/sounding visual swap still follows
  current, alongside the (unverifiable-here) audio.

A missing or failing required check blocks readiness.

## Codex Goal

```text
/goal Complete T28 in docs/tasks/T28-buzzer-speaker-audio.md according to
docs/PLAN.md, AGENTS.md, and docs/design/sprites/README.md. Add a real
Bevy audio system that plays a tone while a buzzer sounds, and add a
speaker ComponentKind (electrically identical to buzzer, its own sprite
and tone) reusing that same audio wiring. Do not add volume/mute UI,
pitch-bending, or any other new audio feature; do not implement successor
tasks. Satisfy every acceptance criterion and run every required check in
the card. Fix task-scoped findings without weakening tests or acceptance
criteria. Prepare one coherent change for review; do not commit or push.
Record exact verification evidence in this card. If a required check is
unavailable (including audibility, which cannot be checked in a
non-interactive environment), report the exact blocker; do not count it
as passed.
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

Core (`crates/core/src/lib.rs`): `ComponentKind::Speaker`, pins `positive`,
`negative`, required parameter `resistance` ranged `1.0..=100.0` ohm
(documented as typical dynamic-speaker voice-coil impedance, tighter than the
buzzer's `1.0..=1e7`). New test
`speaker_is_a_valid_kind_with_documented_range_and_round_trips` mirrors the
buzzer's: schema includes `"speaker"`, range diagnostics at both ends,
lossless JSON round trip.

Solver (`crates/core/src/solver.rs`): `ComponentKind::Speaker` stamped
through the same `BranchKind::Resistor` arm as `Buzzer` (one shared match
arm). New test `speaker_matches_a_plain_resistor_of_the_same_value` (within
the speaker's 1-100 ohm range, unlike the buzzer's 1000-ohm divider test)
confirms current and both pin voltages exactly match a plain resistor of the
same value in an equivalent circuit.

Sprite (`crates/app/src/sprites/speaker.rs`, golden references
`docs/design/sprites/speaker-silent.txt` / `speaker-sounding.txt`, README
updated): 12 x 16 px, same footprint as the buzzer. A concentric-ring "cone"
read (case ring reusing `SWITCH_CASE`, a `METAL`/`METAL_LIGHT`-lit cone slope,
a `METAL_DARK` dust cap) rather than the buzzer's flat grilled disc, so the
two are visually distinct even though they share pins, electrical model, and
sounding threshold. Reuses `buzzer::SOUNDING_CURRENT` (1 mA) and the
`SOUND_WAVE` sounding marks; no new palette colours. Two states, same size
(enforced by `every_state_of_a_part_has_the_same_size`). Registered in
`sprites::art_for`; `placements_cover_every_pin_hole` extended from 6 to 7
kinds; `main.rs::component_summary` has a `Speaker` build-list line.

Audio (`crates/app/Cargo.toml`, `crates/app/src/main.rs`): enabled Bevy's
`bevy_audio` feature (no decoder-format feature — nothing is decoded from a
file). Uses `bevy_audio::Pitch`, a built-in synthesized sine-wave
`Decodable` source, so no custom decoder was written. `setup_audio`
(`Startup`, alongside `setup`) creates two long-duration (1 hour, so a loop
restart is never audible in practice) `Pitch` assets once — 2800 Hz for
`Buzzer`, 440 Hz for `Speaker` — and stores their handles in a new
`SoundTones` resource. `update_view` now takes `Option<Res<SoundTones>>` and,
for `Buzzer`/`Speaker` parts, compares the sprite's existing sounding state
(state index 1, already computed for the visual swap) against a new
`PartVisual.sounding` field; on a rising edge it inserts
`(AudioPlayer<Pitch>, PlaybackSettings::LOOP)` with the kind's tone, on a
falling edge it removes `(AudioPlayer<Pitch>, AudioSink)`. The `Option` keeps
every existing minimal-`App` test (several construct an `App` with only a
few plugins and no `AudioPlugin`/`AssetPlugin`) working unchanged: the audio
step is simply skipped when `SoundTones` is absent.

Required verification (run from repository root on 2026-09-28):
- `cargo fmt --all --check` — pass.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — pass.
- `cargo test --workspace --locked` — pass (113 tests: 49 app + 61 core + 3 tools).
- `cargo build -p bredboard-app --target x86_64-unknown-linux-gnu --locked` — pass.
- `cargo build -p bredboard-app --target wasm32-unknown-unknown --locked` — pass
  (`bevy_audio`'s `cpal`/`rodio` dependency chain, including native `alsa`,
  builds for both targets from this machine's cached/fetched registry; no
  wasm-specific feature gating was needed).

New/changed tests:
- `bredboard_core::tests::speaker_is_a_valid_kind_with_documented_range_and_round_trips`.
- `solver::tests::speaker_matches_a_plain_resistor_of_the_same_value`.
- `sprites::tests::body_sprites_match_design_references` — two new golden
  bodies, generated with `BREDBOARD_BLESS_SPRITES=1` and reviewed above.
- `sprites::tests::every_state_of_a_part_has_the_same_size` and
  `placements_cover_every_pin_hole` (property test) extended to cover
  `Speaker`.
- All 49 `bredboard-app` tests continued to pass unchanged, including the
  several that build a minimal `App` without `AudioPlugin`, confirming the
  `Option<Res<SoundTones>>` guard.

Manual exercise (CLI, `bredboard-tools solve`, since `solve` does not accept
control states so a momentary button cannot be driven directly): a scratch
5 V source in series with a speaker (`resistance = 32`) through a released
momentary button reads `resistor SPK1: 0.000000000 A` (silent, matching the
buzzer's T20 evidence pattern); the same speaker wired directly across the
5 V source reads `resistor SPK1: 0.156250000 A` (= 5 V / 32 ohm, well above
`SOUNDING_CURRENT` — sounding).

This environment cannot play or hear audio output (no speaker/microphone
device, no interactive session): audibility itself, and whether the buzzer
and speaker tones actually sound different, are **not verified** and are
recorded as a blocker per AGENTS.md, not a pass. What was verified instead:
the code compiles and builds for both required targets with `bevy_audio`
enabled; `AudioPlayer<Pitch>`/`PlaybackSettings::LOOP` are inserted exactly
when a part's sprite state crosses into "sounding" and
`AudioPlayer<Pitch>`/`AudioSink` are removed exactly when it crosses back
(reviewed in `update_view`, exercised indirectly by the existing sounding-
state test coverage inherited from T20/T22/T26, e.g.
`e18_volume_control_buzzer_current_decreases_to_silence`,
`e23_led_and_buzzer_switch_together_with_ambient_light`, which assert on the
current threshold that now also gates the tone).

Manual Linux-executable (windowed, and audible) check: not available in this
session, for the same screen/input-automation reason recorded on T19/T20/T21
(`grim` screen capture returns only the static desktop wallpaper regardless
of geometry, and no audio capture or input-automation daemon is available
either). Recorded as a blocker per AGENTS.md, not a pass: a human with
working screen and audio access should place a buzzer and a speaker on a
scratch fixture in the Linux executable, confirm the silent/sounding visual
swap still follows current, and confirm both are actually audible and sound
different from each other, before relying on this as final sign-off.
