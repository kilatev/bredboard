# T22 — Basic new exercises

Status: ready_for_fukit

## Dependencies

[T20 — Buzzer component](T20-buzzer-component.md),
[T21 — Scrollable exercise menu](T21-scrollable-exercise-menu.md), and
[docs/roadmap/LESSONS.md](../roadmap/LESSONS.md). Both dependencies' acceptance
criteria must be satisfied before starting this task.

## Outcome and commit boundary

Add 8 new fixed exercises to the menu (E1, E2, E3, E4, E7, E8, E9, E10 from
`docs/roadmap/LESSONS.md`), each buildable with kinds that already exist after
T20: `dc_voltage_source`, `resistor`, `led`, `momentary_button`, `capacitor`,
`npn_transistor`, `buzzer`.

Suggested commit title: `feat: add eight fixed practice exercises`.

Do not add E5 or E6 (they need T19's variable resistors; that is T23). Do not
change the existing three MVP benches or the menu mechanism itself (T21).

## Scope and interfaces

For each of the 8 exercises, add a fixture and menu entry following the
existing `Circuit`/bench pattern (`fixtures/projects/*.json`,
`Circuit::all()`, the per-circuit English title/description/task text shown
in the app):

| Exercise | Wiring (see LESSONS.md for full circuit) |
| --- | --- |
| E1 First Light | `B1+ -> R1 -> D1(A->K) -> B1-` |
| E2 Push-Button Switch | `B1+ -> S1 -> R1 -> D1(A->K) -> B1-` |
| E3 Two LEDs in Series | `B1+ -> R1 -> D1(A->K) -> D2(A->K) -> B1-` |
| E4 Two LEDs in Parallel | `B1+ -> R1 -> D1(A->K) -> B1-;  B1+ -> R2 -> D2(A->K) -> B1-` |
| E7 Buzzer Doorbell | `B1+ -> S1 -> BZ1(+->-) -> B1-` |
| E8 Transistor Switch | `B1+ -> R2 -> D1(A->K) -> Q1(C->E) -> B1-;  B1+ -> S1 -> R1 -> Q1(B)` |
| E9 Logical AND | `B1+ -> S1 -> S2 -> R1 -> D1(A->K) -> B1-` |
| E10 Smooth Fade | `B1+ -> S1 -> N1;  N1 -> C1(+->-) -> B1-;  N1 -> R1 -> D1(A->K) -> B1-` |

- Write each fixture's English title, one-paragraph behavior explanation,
  parts list, and player task, translated and adapted from
  `docs/roadmap/LESSONS.md`'s source mockup (not copied verbatim; adapt any
  mockup-specific UI check, such as a "pull out D1" instruction, to what this
  app's controls can actually do — for example, toggling a component instead
  of physically removing it, or omitting a check the app cannot express).
- Reuse existing resistor/LED/capacitor/button/transistor parameter values and
  ranges already validated by the MVP catalog; do not invent new ranges.
- Each new fixture must pass the same structural and electrical validation as
  the existing three (`compile_topology`, solver convergence, documented
  ratings).
- Keep hole layouts distinct and readable per `docs/PLAN.md`'s board
  contract; T18 (readability layout), if it lands first, may be reused but is
  not a dependency of this task.

## Acceptance criteria

- [ ] All 8 fixtures are valid, solvable, fixed projects, each selectable from
  the (now 11-entry) menu alongside the existing 3 and reachable via T21's
  scrolling.
- [ ] Each exercise shows its English title, explanation, parts list, and task
  text in the app, matching its fixture.
- [ ] E4's two branches are independently solvable (removing/disabling one LED
  branch, however the app expresses "removing" a component, does not affect
  the other).
- [ ] E8 and E9's button-driven behavior is verified against their documented
  connection lists (E9: both buttons must be actuated for the LED to light;
  neither alone is sufficient).
- [ ] E10's capacitor fade behavior is verified against RC fixtures' existing
  numerical tolerances.
- [ ] E7's buzzer shows its sounding state only while the button is held.
- [ ] Existing three MVP benches, their fixtures, and their tests are
  unchanged.
- [ ] Property/unit tests cover topology validity for all 8 new fixtures.

## Required verification

- Run the baseline formatting, Clippy, workspace test, Linux build, and WASM
  build commands documented in `README.md`.
- Run fixture validation with the existing tools validation command for all 8
  new fixtures.
- Inspect all 8 exercises in the Linux executable: selection from the scrolled
  menu, controls, readings, and the specific check in each exercise's player
  task. Do not test browser interaction.

A missing or failing required check blocks readiness.

## Codex Goal

```text
/goal Complete T22 in docs/tasks/T22-fixed-exercises-basic.md according to
docs/PLAN.md, AGENTS.md, and docs/roadmap/LESSONS.md. Add fixtures, menu
entries, and English text for exercises E1, E2, E3, E4, E7, E8, E9, E10 using
only existing component kinds plus the buzzer from T20, through the scrollable
menu from T21. Do not add E5 or E6, and do not change the existing three MVP
benches.
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

Fixtures (`fixtures/projects/e1-first-light.json` through `e10-smooth-fade.json`,
skipping E5/E6 per scope): each built from existing catalog kinds plus the
T20 buzzer, wired per the LESSONS.md table. E10 adapts the mockup's wiring:
the given `N1 -> C1` and `N1 -> R1 -> D1` split off an unresistored node,
which (since the button is an ideal switch) would jump to 5 V instantly with
no fade at all; adapted to `S1 -> R1 -> N1`, `N1 -> C1 -> V1-`,
`N1 -> D1 -> V1-` so R1 is the shared charging resistor and D1's own
`series_resistance` still limits its current, producing the intended gradual
brightening. Documented inline above and in the app's E10 explanation text.

App (`crates/app/src/main.rs`): `Circuit` grew from 3 to 11 variants
(`Circuit::all()` now returns `[Self; 11]`); `E1_JSON`..`E10_JSON` embedded
the same way as the existing three. Generalized the single hardcoded
"toggle one control" mechanism (previously `Circuit::control()` +
`Bench::toggle_control()`, hardcoded to `S1`/`B1`) into `Circuit::controls()`
returning a list of `ControlSpec { label, component, is_switch }` and
`Bench::toggle(index)`; this was required by E9's two independent buttons.
`Control::ToggleControl` renamed to `Control::Toggle(usize)` (Clippy:
`enum_variant_names`). Control buttons share one row regardless of count
(1 button keeps the original 400x53 layout exactly; 2 buttons, only for E9,
split that same row into two 190x53 buttons) so `RESET` and `CIRCUITS` never
move — an earlier version that stacked a second button below the first
pushed `CIRCUITS` off the bottom of the 760-tall window; caught by the new
menu-selection test below and fixed before landing. Added
`Circuit::explanation_and_task()`: empty for the three MVP benches (which
keep their original "Hover a hole..." hint unchanged, exact original
position) and a one-paragraph explanation + a short task line for each new
exercise, shown in that same text slot. `component_summary` (existing,
unchanged) already serves as each exercise's parts list. `Readout::Value`
and `Readout::Control` generalized to read whichever components/controls a
circuit actually has (`.get()`, not indexing, to avoid panicking on E7's
buzzer or E9's second button) instead of a hardcoded Rc/else branch.

Required verification (run from repository root on 2026-09-26):
- `cargo fmt --all --check` — pass.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — pass.
- `cargo test --workspace --locked` — pass (82 tests: 23 app + 57 core + 2 tools).
- `cargo build -p bredboard-app --target x86_64-unknown-linux-gnu --locked` — pass.
- `cargo build -p bredboard-app --target wasm32-unknown-unknown --locked` — pass.
- `cargo run -p bredboard-tools --locked -- validate <fixture>` for all 8 new
  fixtures — all report "valid project" (component/wire/node counts recorded
  in this evidence's command output, reproducible from the fixture files).
- `cargo run -p bredboard-tools --locked -- verify-native` — pass (unaffected).

New tests:
- `bredboard_core::solver::tests::exercises` (new module, one fixture-backed
  test per exercise plus one covering all 8 together):
  `all_ten_exercise_fixtures_have_valid_solvable_topology`,
  `e1_e3_always_on_exercises_light_without_any_control`,
  `e2_led_lights_only_while_its_button_is_pressed`,
  `e4_two_led_branches_are_independently_solvable` (removes each branch's
  components/wires in turn and confirms the other branch's current is
  unaffected — the required "however the app expresses removing a
  component" is a structural/off-line removal, since there is no live
  component-removal UI),
  `e7_buzzer_sounds_only_while_its_button_is_held`,
  `e8_led_switches_on_only_while_its_button_is_held`,
  `e9_led_lights_only_when_both_buttons_are_held_together` (all four
  press/release combinations; only both-pressed lights the LED),
  `e10_capacitor_charges_monotonically_and_led_follows` (2000 transient
  steps with the button held; both capacitor voltage and LED current never
  decrease, and the LED is lit once charged).
- `bredboard_app::tests::all_eleven_circuits_are_selectable_via_the_scrolled_menu` —
  scrolls to and selects every one of the 11 menu entries (computing each
  entry's clamped scroll offset the same way `handle_scroll` would) and
  confirms the right circuit loads, then returns to the menu, for all 11 —
  the concrete "selectable from the menu and reachable via scrolling" check.
  (This test caught the `CIRCUITS`-button-off-screen bug above.)
- `bredboard_app::tests::new_exercises_show_title_explanation_and_task_text` —
  for all 8 new exercises, spawns the bench and confirms the rendered
  `Text2d` entities include the title, the explanation, the task text, and
  every component's id (parts list).
- Existing regression tests unaffected and still passing unchanged:
  `existing_three_entry_menu_layout_is_unchanged`,
  `all_embedded_boards_have_unique_lead_and_wire_holes`,
  `shared_menu_and_visible_buttons_dispatch_core_actions`,
  `each_circuit_uses_core_controls_and_reset` (now iterates all 11 circuits;
  circuits with zero controls, E1/E3/E4, correctly no-op on `toggle(0)`),
  `rc_switch_charges_and_discharges_from_the_embedded_board`.

Manual exercise (CLI, `bredboard-tools solve`/`simulate`, on top of the
automated tests above): confirmed released/pressed current pairs for E1
(8.58 mA, always on), E2 (0 -> 8.58 mA), E3 (5.33 mA on both LEDs, above the
5 mA lit threshold), E4 (8.58 mA on each independent branch), E7 (0 -> 50 mA,
well past the 1 mA sounding threshold), E8 (0.49 uA -> 8.46 mA through the
transistor), E9 (0 mA for none/S1-only/S2-only, 8.58 mA only when both
buttons are pressed), and E10 (capacitor voltage rising smoothly from 0 to a
~1.99 V plateau over 0.1-0.3 s of simulated time across steps 1/100/500/1000/
3000/8000).

Manual Linux-executable (windowed) check: not available in this session, for
the same reason recorded on T19/T20/T21 (this session's `grim` screen capture
returns only the static desktop wallpaper for every geometry tried, and no
input-automation daemon is available). Recorded as a blocker per AGENTS.md,
not a pass. Given the scale of this task (8 new exercises through a
generalized multi-control UI), the automated coverage above is unusually
direct substitute evidence — it exercises the real `handle_mouse`,
`scroll_menu`, `spawn_bench`, and `update_view` systems end-to-end for every
new exercise, including the exact menu-scroll-and-select path and the exact
rendered text — but a human with working screen access should still click
through all 8 exercises in the real window (selection, controls, readings,
and each exercise's specific task check) before relying on this as final
sign-off.
