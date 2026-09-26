# T20 — Buzzer component

Status: ready_for_fukit

## Dependencies

[T17 — Sprites for the remaining and future simple components](T17-more-component-sprites.md)
and [docs/roadmap/LESSONS.md](../roadmap/LESSONS.md). Independent of T19; can
be done in either order.

## Outcome and commit boundary

Add `buzzer` as a new `ComponentKind`: a fixed-resistance two-terminal load
with a current-derived "sounding" presentation state, plus its sprite design
and implementation.

Suggested commit title: `feat: add buzzer component`.

The buzzer has no audio output; the app has no audio system, and adding one is
out of scope. "Sounding" is a visual state only (for example, drawn sound-wave
marks), the same way `led` derives its glow purely from calculated current.

## Scope and interfaces

- Design the sprite first, following
  [the sprite design reference](../design/sprites/README.md)'s style rules and
  "Adding a component sprite" steps: body size, pins (`positive`, `negative`
  polarity, since real active buzzers are polarized), a silent state and a
  sounding state, and any new palette colors with unique legend symbols. Add
  the design to the reference's "Current parts" table (it is a real kind, not
  a Part B future design) with a golden `.txt` reference.
- Add `ComponentKind::Buzzer` with pins `positive`, `negative` and a documented
  `resistance` parameter and range in `crates/core/src/lib.rs`, alongside the
  other kinds.
- Model it in the solver as a fixed linear resistor (reuse the existing
  resistor stamp code path; do not add diode-style nonlinearity). Derive the
  sprite's sounding/silent state from calculated current against a documented
  threshold, the same pattern as the LED's dim/lit thresholds.
- Register the `PartArt` in `sprites::art_for` and extend the placement and
  same-size property tests to cover it.

## Acceptance criteria

- [ ] `buzzer` is a valid `ComponentKind` with documented pins, parameter, and
  range; JSON Schema output includes it.
- [ ] The solver treats it as a linear resistive load; a test confirms current
  and voltage match a plain resistor of the same value in an equivalent
  circuit.
- [ ] The sprite has silent and sounding states driven by calculated current,
  matches its committed golden reference, and follows the style rules
  (outline, lighting, palette reuse where possible).
- [ ] Placement and same-size property tests cover the new kind.
- [ ] Round-trip persistence for a project containing a buzzer.
- [ ] Existing fixtures, tests, and other kinds are unaffected.

## Required verification

- Run the baseline formatting, Clippy, workspace test, Linux build, and WASM
  build commands documented in `README.md`.
- Run the new solver-equivalence, golden, and property sprite tests; generate
  goldens with `BREDBOARD_BLESS_SPRITES=1` and review the new `.txt` file.
- Manually exercise the buzzer in a scratch fixture in the Linux executable to
  confirm the silent/sounding visual swap follows current.

A missing or failing required check blocks readiness.

## Codex Goal

```text
/goal Complete T20 in docs/tasks/T20-buzzer-component.md according to
docs/PLAN.md, AGENTS.md, docs/roadmap/LESSONS.md, and
docs/design/sprites/README.md. Design and add a buzzer component kind as a
linear resistive load with a current-derived silent/sounding sprite state.
Do not add audio output or the LESSONS.md exercises.
Satisfy every acceptance criterion and run every required check in the card.
Fix task-scoped findings without weakening tests or acceptance criteria.
Do not implement successor tasks. Prepare one coherent change for review;
do not commit or push. Record exact verification evidence in this card.
If a required check is unavailable, report the exact blocker and the input
or environment change needed; do not count it as passed.
```

## Completion and fukit handoff

When all criteria pass, record evidence and set `Status: ready_for_fukit`.
Stop without starting successor tasks. The user may then invoke `fukit` to
describe, commit, and push.

An existing jj repository and an unambiguous authorized remote/bookmark are
required for that workflow. Do not initialize or guess them. Commit
completion is evidenced by jj history; publication is evidenced by the actual
push result.

## Evidence

Sprite design (`crates/app/src/sprites/buzzer.rs`, golden references
`docs/design/sprites/buzzer-silent.txt` / `buzzer-sounding.txt`, README
updated): 12 x 16 px (12 x 12 case plus 4 px headroom for sound-wave marks).
Round dark-plastic case reusing `changeover_switch`'s `SWITCH_CASE`/
`SWITCH_CASE_SHADE`, a silver metal grille reusing `METAL`/`METAL_LIGHT`, and
a polarity stripe on the positive (left) side reusing the capacitor's
`ELCAP_STRIPE`, per the "polarized... positive, negative" requirement. One
new palette colour, `SOUND_WAVE` (legend symbol `'`, the only unused ASCII
legend character left — checked mechanically since nearly the full printable
range is already assigned). Silent state has an empty headroom band; sounding
adds `SOUND_WAVE` arcs there. Two states, same size (enforced by
`every_state_of_a_part_has_the_same_size`).

Core (`crates/core/src/lib.rs`): `ComponentKind::Buzzer`, pins `positive`,
`negative`, required parameter `resistance` ranged `1.0..=1e7` ohm (same
breadboard-scale bound as `resistor`).

Solver (`crates/core/src/solver.rs`): stamped as `BranchKind::Resistor` via
the existing resistor code path (no new branch kind, no nonlinearity) —
`buzzer_matches_a_plain_resistor_of_the_same_value` confirms current and both
pin voltages exactly match a plain resistor of the same value in an
equivalent circuit (the divider fixture with R1 swapped for a buzzer).

Presentation (`crates/app`): `PartContext.buzzer_current` (new field,
alongside the existing `led_current`), populated in `main.rs::update_view`
from `readings.resistor_currents.get(&part.id)` (the buzzer's own current,
since it stamps into that map like any resistor). `buzzer::SOUNDING_CURRENT`
(1 mA) is the silent/sounding threshold, the same current-threshold pattern
as the LED's `DIM_CURRENT`/`LIT_CURRENT`. Registered in `sprites::art_for`;
`main.rs::component_summary` has a `Buzzer` build-list line.

Required verification (run from repository root on 2026-09-26):
- `cargo fmt --all --check` — pass.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — pass.
- `cargo test --workspace --locked` — pass (72 tests: 21 app + 49 core + 2 tools).
- `cargo build -p bredboard-app --target x86_64-unknown-linux-gnu --locked` — pass.
- `cargo build -p bredboard-app --target wasm32-unknown-unknown --locked` — pass.

New tests:
- `bredboard_core::tests::buzzer_is_a_valid_kind_with_documented_range_and_round_trips` —
  schema includes `"buzzer"`, range diagnostics at both ends, lossless JSON round trip.
- `solver::tests::buzzer_matches_a_plain_resistor_of_the_same_value` — the
  card's required solver-equivalence check.
- `sprites::tests::body_sprites_match_design_references` — both new golden
  bodies, generated with `BREDBOARD_BLESS_SPRITES=1` and reviewed above.
- `sprites::tests::every_state_of_a_part_has_the_same_size` and
  `placements_cover_every_pin_hole` (property test) extended to cover `Buzzer`.

Manual exercise (CLI, `bredboard-tools`): a scratch project (5 V source, a
momentary button in series with the buzzer, `resistance = 32`) solved with
the button released (`resistor BZ1: 0.000000000 A`, below
`SOUNDING_CURRENT` — silent) and with `initial_conditions.controls.S1 =
button_pressed` (`resistor BZ1: 0.156250000 A` = 5 V / 32 ohm exactly, well
above `SOUNDING_CURRENT` — sounding), confirming the silent/sounding current
threshold responds to the button as E7 (T22) will require.

Manual Linux-executable (windowed) check: not available in this session, for
the same reason recorded on T19/T21 — this session's `grim` screen capture
returns only the static desktop wallpaper for every geometry tried, despite
`hyprctl` confirming the target window renders, and no input-automation
daemon is available. Recorded as a blocker per AGENTS.md, not a pass: a human
with working screen access should place a buzzer on a scratch fixture in the
Linux executable and confirm the silent/sounding visual swap follows the
button before relying on this as final sign-off.
