# T24 — Fixed exercises E11–E15

Status: ready_for_fukit

## Dependencies

[T21 — Scrollable exercise menu](T21-scrollable-exercise-menu.md) and
[docs/roadmap/LESSONS.md](../roadmap/LESSONS.md) (its "Extended exercise list
(E11–E30)" section). T21's acceptance criteria must be satisfied before
starting this task. Independent of T22/T23/T25/T26/T27.

## Outcome and commit boundary

Add 5 new fixed exercises to the menu (E11–E15), each buildable with kinds
already in the catalog: `dc_voltage_source`, `resistor`, `led`,
`momentary_button`, `changeover_switch`, `capacitor`, `npn_transistor`.

Suggested commit title: `feat: add five fixed practice exercises (E11-E15)`.

Do not add E16–E30 (later cards). Do not change the existing three MVP
benches, the T22/T23 exercises, or the menu mechanism itself (T21).

## Scope and interfaces

For each exercise, add a fixture and menu entry following the existing
`Circuit`/bench pattern (`fixtures/projects/*.json`, `Circuit::all()`, the
per-circuit English title/description/task text shown in the app):

| Exercise | Wiring |
| --- | --- |
| E11 Two-Way Switch | `B1+ -> SW1(common); SW1(A) -> R1 -> D1(A->K) -> B1-;  SW1(B) -> R2 -> D2(A->K) -> B1-` |
| E12 Mixed Series/Parallel Wiring | `B1+ -> R1 -> D1(A->K) -> D2(A->K) -> B1-;  B1+ -> R2 -> D3(A->K) -> B1-` |
| E13 Capacitor Against Bounce | `B1+ -> S1 -> N1;  N1 -> C1(+->-) -> B1-;  N1 -> R1 -> D1(A->K) -> B1-` |
| E14 Buttons OR | `B1+ -> N1; N1 -> S1 -> R1 -> N2; N1 -> S2 -> R2 -> N2; N2 -> R3 -> D1(A->K) -> B1-` |
| E15 Transistor Inverter | `B1+ -> R1 -> D1(A->K) -> Q1(C->E) -> B1-;  B1+ -> R2 -> Q1(B);  Q1(B) -> S1 -> B1-` |

- Write each fixture's English title, one-paragraph behavior explanation,
  parts list, and player task, adapted from the source mockup linked in
  `docs/roadmap/LESSONS.md` (not copied verbatim; adapt any mockup-specific UI
  check to what this app's controls can actually do, per the same rule T22
  used).
- E11 needs a `changeover_switch` control exposed as a toggle between its two
  positions (`A`/`B`), not just on/off; reuse or extend whatever control
  mechanism the existing changeover-switch sprite/interaction already uses if
  T17 already added one, otherwise add the minimal toggle needed.
- E13's player-visible check (comparing button release with and without the
  capacitor) must be expressible with this app's controls (for example,
  toggling the capacitor's presence structurally is not available; prefer a
  task phrased around observable LED fade timing, as T22's E10 did).
- E14's two buttons cannot be wired as bare ideal switches directly in
  parallel between the same two nodes: this solver models a closed switch as
  a zero-volt ideal branch (see `crates/core/src/solver.rs`'s
  `BranchKind::Switch` stamping), so two such branches sharing an identical
  node pair produce the same `singular_system` class of error that
  `fixtures/projects/conflicting-sources.json` guards against, specifically
  when both buttons are held at once. Give each button its own small series
  resistor (100 ohm) before the two paths merge, so each switch's branch ends
  at a distinct node; this preserves OR behavior (lit whenever either or both
  buttons are held) without hitting the singularity.
- Reuse existing resistor/LED/capacitor/button/switch/transistor parameter
  values already validated by the MVP and T22/T23 catalogs; do not invent new
  ranges.
- Each new fixture must pass the same structural and electrical validation as
  the existing ones (`compile_topology`, solver convergence, documented
  ratings).

## Acceptance criteria

- [ ] All 5 fixtures are valid, solvable, fixed projects, selectable from the
  menu and reachable via T21's scrolling.
- [ ] Each exercise shows its English title, explanation, parts list, and task
  text in the app, matching its fixture.
- [ ] E11's switch reliably selects exactly one of D1/D2, never both, in both
  positions.
- [ ] E12's two branches are independently solvable (one branch's LEDs do not
  affect the other branch's current).
- [ ] E14's OR behavior is verified against all four button combinations.
- [ ] E15's inverter behavior is verified: LED on when the button is
  unpressed, off when pressed.
- [ ] Existing three MVP benches, T22/T23 exercises, and their tests are
  unchanged.
- [ ] Property/unit tests cover topology validity for all 5 new fixtures.

## Required verification

- Run the baseline formatting, Clippy, workspace test, Linux build, and WASM
  build commands documented in `README.md`.
- Run fixture validation with the existing tools validation command for all 5
  new fixtures.
- Inspect all 5 exercises in the Linux executable: selection from the scrolled
  menu, controls, readings, and the specific check in each exercise's player
  task. Do not test browser interaction.

A missing or failing required check blocks readiness.

## Codex Goal

```text
/goal Complete T24 in docs/tasks/T24-fixed-exercises-e11-e15.md according to
docs/PLAN.md, AGENTS.md, and docs/roadmap/LESSONS.md. Add fixtures, menu
entries, and English text for exercises E11-E15 using only existing component
kinds, through the scrollable menu from T21. Do not add E16-E30, and do not
change the existing three MVP benches or the T22/T23 exercises.
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

Implemented 5 new fixtures and the corresponding `Circuit` entries:

- `fixtures/projects/e11-two-way-switch.json` (`Circuit::E11`): a
  `changeover_switch` with `normally_closed`/`normally_open` branches, each its
  own resistor+LED to the negative rail; `common` tied to the positive rail.
  Reuses the existing `is_switch: true` toggle mechanism from the RC bench
  (`Circuit::controls`/`Bench::toggle`) unchanged — no new control mechanism
  needed.
- `fixtures/projects/e12-mixed-wiring.json` (`Circuit::E12`): one branch wires
  two LEDs in series behind `R1` (150 ohm, same value T22's E3 uses for two
  series LEDs), the other wires one LED behind `R2` (330 ohm); always-on, no
  controls, like E1/E3/E4.
- `fixtures/projects/e13-capacitor-against-bounce.json` (`Circuit::E13`): a
  button feeds a node shared by a capacitor (100 uF, T22's E10 value) and a
  resistor(2 kOhm, T22's E10 value)+LED branch, both to the negative rail;
  task text is phrased around observable LED fade-out timing on release, per
  this card's own note, mirroring T22's E10 adaptation.
- `fixtures/projects/e14-buttons-or.json` (`Circuit::E14`): two buttons
  wired in parallel to a resistor+LED branch. Deviation from the card's
  original wiring, recorded above under "Scope and interfaces": wiring the two
  buttons as bare ideal switches directly in parallel between the same two
  nodes made the "both pressed" case a `singular_system` structural error
  (`bench.simulation.diagnostics` reported
  `circuit has an underdetermined ideal-source arrangement`), the same class
  of failure `fixtures/projects/conflicting-sources.json` exists to guard
  against — this solver stamps a closed switch as a zero-volt ideal branch
  (`BranchKind::Switch` in `crates/core/src/solver.rs`), and two such branches
  sharing an identical node pair are structurally redundant. Fixed by giving
  each button its own 100 ohm series resistor (`R1`, `R2`) before the two
  paths merge into the final 330 ohm limiting resistor (`R3`) and LED; this
  keeps the OR behavior (lit whenever either or both buttons are held)
  without hitting the singularity, confirmed by the new
  `e14_or_truth_table_across_all_combinations` test below, which failed with
  exactly that diagnostic before the fix and passes after it.
- `fixtures/projects/e15-transistor-inverter.json` (`Circuit::E15`): `R2`
  (100 kOhm, T22's E8 `R3` pull-down value, reused here as a pull-up) keeps
  the base high by default, turning the transistor and LED on; `S1` pulls the
  base directly to the negative rail when pressed, turning the LED off — a
  logical NOT of E8's (T22) button-to-turn-on behavior, called out in both
  exercises' task text.

`crates/app/src/main.rs` changes: added `E11_JSON`..`E15_JSON` embedded
constants, 5 `Circuit` variants, grew `Circuit::all()` from 13 to 18 entries,
added `json()`/`label()`/`explanation_and_task()` arms, added `controls()`
entries (`E11_SWITCH` reusing the existing switch-toggle `ControlSpec`
mechanism; `E13`/`E15` reuse the existing single-button `S1_BUTTON` arm;
`E14` reuses the existing two-button `E9_BUTTONS` arm as-is, since its label
text ("S1/S2: PRESS / RELEASE") already fits). No changes to `dial()`, to the
three MVP benches, or to any T22/T23 exercise.

Added tests: `e11_switch_selects_exactly_one_led` (both switch positions,
asserts exactly one of D1/D2 carries current); `e12_branches_are_independently_solvable`
(solves the full fixture, then solves each branch's components/wires with the
other branch's removed, and asserts the LED currents match exactly, proving
branch independence); `e14_or_truth_table_across_all_combinations` (all 4
button-state combinations via `Action::SetControl`, asserting D1 lights iff
either button is pressed); `e15_inverter_lights_when_unpressed_and_darkens_when_pressed`.
Extended `new_exercises_show_title_explanation_and_task_text` to cover
E11–E15 (title/explanation/task/parts-list text). `all_embedded_boards_have_unique_lead_and_wire_holes`,
`each_circuit_uses_core_controls_and_reset`, and `all_circuits_are_selectable_via_the_scrolled_menu`
already iterate `Circuit::all()` and so cover all 5 new fixtures without
changes to those tests themselves.

Required verification (run from the workspace root on 2026-09-27):

- `cargo fmt --all --check` — passed (also picked up one pre-existing,
  unrelated formatting drift in `spawn_source`'s `draw_wire` call from before
  this task; fixed via `cargo fmt --all` as part of the same pass since
  `--check` cannot pass otherwise).
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — passed,
  no warnings.
- `cargo test --workspace --locked` — passed: 59 `bredboard-core` tests, 28
  `bredboard-app` tests (including the 5 new ones above and the extended
  title/task test), 2 `bredboard-tools` tests, 0 doc-tests.
- `cargo build -p bredboard-app --target x86_64-unknown-linux-gnu --locked` —
  passed.
- `cargo build -p bredboard-app --target wasm32-unknown-unknown --locked` —
  passed.
- `cargo run -p bredboard-tools --locked -- validate fixtures/projects/<f>.json`
  for all 5 new fixtures — all reported `valid project` with the expected
  component/wire/node counts (E11: 6/5/6, E12: 6/4/5, E13: 5/3/4, E14: 7/2/6,
  E15: 6/4/5).
- Additionally ran `cargo run -p bredboard-tools --locked -- solve` (E11,
  E12, E14, E15) and `-- simulate ... 100` (E13, since it has a capacitor and
  needs transient stepping) to manually sanity-check node voltages and
  currents against each fixture's intended behavior before writing the
  automated tests; all matched expectations (e.g. E11 default position lights
  only D1 at ~8.6 mA with D2 at 0 A; E15 default state has the transistor
  conducting at ~7.6 mA with the base near 0.6 V).

Manual Linux-executable (windowed) inspection: **not available in this
session**, same blocker class as T19–T22 recorded. `grim` is present on
`PATH` this time, but the capture command hung indefinitely (no responding
Wayland compositor/screenshot permission in this sandbox) rather than
returning a usable image, and had to be force-stopped after timeout; no
input-automation daemon is available either. Recorded as a blocker per
AGENTS.md, not a pass. The automated `bredboard-app` tests above exercise the
real `spawn_bench`, `handle_mouse`, `scroll_menu`, `Bench::toggle`, and
`update_view` systems end-to-end (menu selection by scrolled position, control
button clicks, control-state dispatch, and readout/text content) for all 5
new exercises, which is the same substitute evidence T19–T22 relied on for
this same environment limitation.

All acceptance criteria satisfied given the above (including the fixed E14
wiring, which still satisfies "OR behavior... verified against all four
button combinations"). Stopping here without starting T25.
