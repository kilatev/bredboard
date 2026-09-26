# T23 — Variable-resistor exercises

Status: ready_for_fukit

## Dependencies

[T19 — Variable-resistor components](T19-variable-resistor-components.md),
[T21 — Scrollable exercise menu](T21-scrollable-exercise-menu.md), and
[docs/roadmap/LESSONS.md](../roadmap/LESSONS.md). Both dependencies'
acceptance criteria must be satisfied before starting this task. Independent
of T20/T22; can be done before or after them.

## Outcome and commit boundary

Add the remaining 2 fixed exercises from `docs/roadmap/LESSONS.md` (E5, E6),
each requiring T19's new component kinds.

Suggested commit title: `feat: add potentiometer and photoresistor exercises`.

## Scope and interfaces

| Exercise | Wiring | New part exercised |
| --- | --- | --- |
| E5 Brightness Dial | `B1+ -> RV1 -> R1 -> D1(A->K) -> B1-` | `potentiometer` |
| E6 Light-Reactive LED | `B1+ -> R2(photoresistor) -> R1 -> D1(A->K) -> B1-` | `photoresistor` |

- Write each fixture's English title, one-paragraph explanation, parts list,
  and player task, translated and adapted from `docs/roadmap/LESSONS.md`'s
  source mockup, per the same adaptation rule as T22 (do not copy a
  mockup-specific UI check the app cannot express).
- E5 exposes T19's continuous control input as a visible in-app control (for
  example, a draggable dial or slider bound to the potentiometer's ratio),
  distinct from the discrete button/switch controls already in the app.
- E6 exposes the same mechanism as an "ambient light" control the player can
  change (per `docs/roadmap/LESSONS.md`, a slider is an acceptable
  presentation of "cover the sensor").
- Both fixtures must pass the same structural and electrical validation as
  the existing fixtures.

## Acceptance criteria

- [ ] Both fixtures are valid, solvable, fixed projects, selectable from the
  menu (now 13 entries total) and reachable via T21's scrolling.
- [ ] E5's LED brightness changes continuously and monotonically as the
  in-app dial control moves across its range.
- [ ] E6's LED brightness changes continuously and monotonically as the
  in-app "ambient light" control moves across its range, and reaches the LED's
  off threshold at the darkest setting.
- [ ] Each exercise shows its English title, explanation, parts list, and task
  text in the app, matching its fixture.
- [ ] Existing fixtures, the 8 T22 exercises (if landed), and the three MVP
  benches are unaffected.
- [ ] Property/unit tests cover topology validity for both new fixtures and
  the monotonic-brightness claims above.

## Required verification

- Run the baseline formatting, Clippy, workspace test, Linux build, and WASM
  build commands documented in `README.md`.
- Run fixture validation with the existing tools validation command for both
  new fixtures.
- Inspect both exercises in the Linux executable: selection from the scrolled
  menu, the dial/slider controls across their full range, and LED brightness
  response. Do not test browser interaction.

A missing or failing required check blocks readiness.

## Codex Goal

```text
/goal Complete T23 in docs/tasks/T23-fixed-exercises-variable.md according to
docs/PLAN.md, AGENTS.md, and docs/roadmap/LESSONS.md. Add fixtures, menu
entries, in-app dial/slider controls, and English text for exercises E5 and
E6, using the potentiometer and photoresistor kinds from T19 through the
scrollable menu from T21.
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

Fixtures (`fixtures/projects/e5-brightness-dial.json`,
`e6-light-reactive-led.json`): `V1+ -> RV1 -> R1 -> D1(A->K) -> V1-`, `RV1`
a `potentiometer` (E5, 100-5000 ohm) or `photoresistor` (E6, 100-20000 ohm),
default `initial_conditions.control_ratios.RV1 = 0.5`. `R1` (100 ohm) and
`D1`'s own `series_resistance` keep current in range across the whole ratio
sweep.

App (`crates/app/src/main.rs`): added `Circuit::E5`/`E6` (`Circuit::all()` is
now 13, appended after E10 so the extension stays append-only per T21's
contract). New `Circuit::dial()` returns an optional `DialSpec { label,
component }` for the bench's continuous control, shown instead of a button
row when present (E5/E6 have empty `controls()`). Spawns a `DialTrack`
(the draggable region, same position/size as the button row it replaces)
and a `DialHandle` sprite positioned from the live ratio. Two new systems:
`handle_dial` (while the left mouse button is held over the track, computes
`ratio` from the cursor's horizontal position and issues
`Action::SetControlRatio`, exactly the same action path T19 added) and
`update_dial_handle` (repositions the handle from
`SimulationState.control_ratios` every frame). Both registered in the
`Update` chain after `handle_mouse`. This is a distinct control mechanism
from the discrete button/switch `Control::Toggle`, per the card's
requirement.

Required verification (run from repository root on 2026-09-26):
- `cargo fmt --all --check` — pass.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — pass.
- `cargo test --workspace --locked` — pass (85 tests: 24 app + 59 core + 2 tools).
- `cargo build -p bredboard-app --target x86_64-unknown-linux-gnu --locked` — pass.
- `cargo build -p bredboard-app --target wasm32-unknown-unknown --locked` — pass.
- `cargo run -p bredboard-tools --locked -- validate` for both new fixtures —
  both report "valid project".

New tests:
- `bredboard_core::solver::tests::exercises::e5_led_brightness_changes_continuously_and_monotonically_with_the_dial` —
  41-point ratio sweep (0.0..=1.0), current never rises as ratio rises
  (potentiometer: higher ratio -> higher resistance -> lower current).
- `e6_led_brightness_changes_continuously_and_monotonically_and_reaches_off` —
  same sweep, current never falls as ratio rises, and the darkest setting
  (ratio 0.0) is below the LED's dim threshold (0.156 mA < 0.5 mA), i.e. off.
- `all_ten_exercise_fixtures_have_valid_solvable_topology` extended to cover
  E5/E6 alongside the T22 fixtures.
- `bredboard_app::tests::dragging_the_dial_sets_the_control_ratio_and_moves_its_handle` —
  for both E5 and E6: a synthetic drag to the track's left edge sets the
  ratio near 0.0, a drag to the right edge sets it near 1.0, and the handle
  sprite's X position tracks the resulting ratio, all through the real
  `handle_dial`/`update_dial_handle` systems.
- `new_exercises_show_title_explanation_and_task_text` and
  `all_circuits_are_selectable_via_the_scrolled_menu` (renamed from
  `..._eleven_..._menu` since the count is no longer literally eleven)
  extended to include E5/E6, so both are also confirmed selectable via
  T21's scrolling and show their title/explanation/task/parts-list text.

Manual exercise (CLI, `bredboard-tools solve`, a ratio sweep at
0.0/0.25/0.5/0.75/1.0 via `initial_conditions.control_ratios`): E5's D1
current fell monotonically from 13.6 mA to 0.60 mA across the sweep (a
"dial" from bright toward dim, matching its title; LESSONS.md does not
require it to reach fully off). E6's D1 current rose monotonically from
0.156 mA (below the 0.5 mA dim threshold — off, the darkest setting) to
13.6 mA (brightest) across the same sweep.

Manual Linux-executable (windowed) check: not available in this session, for
the same reason recorded on T19-T22 (this session's `grim` screen capture
returns only the static desktop wallpaper for every geometry tried, and no
input-automation daemon is available to drive a real mouse drag). Recorded
as a blocker per AGENTS.md, not a pass. The automated drag test above
exercises the real drag-handling and handle-repositioning systems end to
end, but a human with working screen access should still drag both dials in
the real window and watch the LED brightness respond before relying on this
as final sign-off.
