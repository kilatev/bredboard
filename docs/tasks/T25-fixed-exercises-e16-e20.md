# T25 — Fixed exercises E16–E20

Status: in_progress

## Dependencies

[T21 — Scrollable exercise menu](T21-scrollable-exercise-menu.md) and
[docs/roadmap/LESSONS.md](../roadmap/LESSONS.md) (its "Extended exercise list
(E11–E30)" section). T21's acceptance criteria must be satisfied before
starting this task. Independent of T22/T23/T24/T26/T27.

## Outcome and commit boundary

Add 5 new fixed exercises to the menu (E16–E20), each buildable with kinds
already in the catalog: `dc_voltage_source`, `resistor`, `led`, `capacitor`,
`npn_transistor`, `photoresistor`, `buzzer`.

Suggested commit title: `feat: add five fixed practice exercises (E16-E20)`.

Do not add other exercises (other cards). Do not change the existing three
MVP benches, other exercise cards' work, or the menu mechanism itself (T21).

## Scope and interfaces

| Exercise | Wiring |
| --- | --- |
| E16 Automatic Night Light | `B1+ -> R1 -> D1(A->K) -> Q1(C->E) -> B1-;  B1+ -> R2 -> Q1(B);  Q1(B) -> R3(photoresistor) -> B1-` |
| E17 Light Alarm | `B1+ -> BZ1(+->-) -> Q1(C->E) -> B1-;  B1+ -> R2(photoresistor) -> Q1(B);  Q1(B) -> R1 -> B1-` |
| E18 Volume Control | `B1+ -> RV1 -> BZ1(+->-) -> B1-` |
| E19 Three Independent Branches | `B1+ -> R1 -> D1(A->K) -> B1-;  B1+ -> R2 -> D2(A->K) -> B1-;  B1+ -> R3 -> D3(A->K) -> B1-` |
| E20 Turn-On Delay | `B1+ -> R2 -> N1 -> Q1(B);  N1 -> C1(+->-) -> B1-;  B1+ -> R1 -> D1(A->K) -> Q1(C->E) -> B1-` |

- Write each fixture's English title, one-paragraph behavior explanation,
  parts list, and player task, adapted from the source mockup linked in
  `docs/roadmap/LESSONS.md` (not copied verbatim; adapt any mockup-specific UI
  check, such as "cover the sensor with your hand," to this app's existing
  "ambient light" control input, the same mechanism T23's E6 uses).
- E16 and E17 both need the `photoresistor`'s continuous "ambient light"
  control exposed as an in-app dial/slider, exactly as T23 (E6) already did;
  reuse that mechanism rather than duplicating it.
- E18 needs the potentiometer's continuous control dial, exactly as T23 (E5)
  already did; reuse that mechanism.
- E18's task (finding where the buzzer cuts off) only needs the existing
  "sounding" presentation state; no new instrument is required.
- E20's player task references elapsed real time; state the expected delay
  in simulated seconds (consistent with the fixed timestep the app already
  uses) rather than wall-clock time, since the app has no wall-clock display.
- Reuse existing parameter values and ranges already validated by prior
  catalogs; do not invent new ranges beyond what T19/T20 already validated
  for `photoresistor`/`buzzer`.

## Acceptance criteria

- [ ] All 5 fixtures are valid, solvable, fixed projects, selectable from the
  menu and reachable via T21's scrolling.
- [ ] Each exercise shows its English title, explanation, parts list, and task
  text in the app, matching its fixture.
- [ ] E16's LED brightness responds inversely to E6's across the same
  ambient-light control sweep (darker → brighter, the opposite of E6).
- [ ] E17's buzzer sounding state responds to the same ambient-light control,
  going from silent (dark) to sounding (bright), past its documented
  threshold.
- [ ] E18's buzzer sounding state is monotonic (non-increasing) as the
  potentiometer dial ratio increases, and reaches silent at some ratio before
  1.0.
- [ ] E19's three branches are independently solvable and their currents are
  each individually verified against their documented resistor/LED pairing.
- [ ] E20's LED turns on only after the capacitor's simulated charging time
  crosses the transistor's base threshold, verified by simulated-step count,
  not wall-clock time.
- [ ] Existing three MVP benches and other exercise cards' work are unchanged.
- [ ] Property/unit tests cover topology validity and the monotonic/threshold
  claims above for all 5 new fixtures.

## Required verification

- Run the baseline formatting, Clippy, workspace test, Linux build, and WASM
  build commands documented in `README.md`.
- Run fixture validation with the existing tools validation command for all 5
  new fixtures.
- Inspect all 5 exercises in the Linux executable: selection from the scrolled
  menu, the dial/slider controls where present, and the specific check in each
  exercise's player task. Do not test browser interaction.

A missing or failing required check blocks readiness.

## Codex Goal

```text
/goal Complete T25 in docs/tasks/T25-fixed-exercises-e16-e20.md according to
docs/PLAN.md, AGENTS.md, and docs/roadmap/LESSONS.md. Add fixtures, menu
entries, and English text for exercises E16-E20 using only existing component
kinds and the existing dial/slider control mechanisms from T19/T23, through
the scrollable menu from T21. Do not add other exercises, and do not change
the existing three MVP benches or other exercise cards' work.
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

- Added `fixtures/projects/e16-automatic-night-light.json` through
  `e20-turn-on-delay.json`, with the specified fixed topologies and existing
  catalog parameter ranges. The existing core topology/solver accepts all
  five; the app tests also assert unique lead/wire holes and menu reachability.
- Added E16–E20 to the existing scrollable menu and reused the existing dial
  mechanism for E16's `R3` photoresistor, E17's `R2` photoresistor, and E18's
  `RV1` potentiometer. Added English title, explanation, parts list, and task
  text for each fixture.
- Added regression tests for E16 inverse light response, E17 light threshold,
  E18 monotonic buzzer current and silence, E19 independent branch currents,
  and E20 fixed-step capacitor delay. `cargo test -p bredboard-app --locked`
  passed: 33 tests.
- Fixture validation passed for all five:
  `cargo run -p bredboard-tools --locked -- validate
  fixtures/projects/e16-automatic-night-light.json` (6 components, 4 wires,
  5 nodes), E17 (5, 4, 4), E18 (3, 2, 3), E19 (7, 6, 5), and E20 (6, 4, 5).
- Required baseline checks passed on 2026-09-27:
  `cargo fmt --all --check`; `cargo clippy --workspace --all-targets --locked
  -- -D warnings`; `cargo test --workspace --locked` (33 app, 59 core, 2
  tools tests); `cargo build -p bredboard-app --target
  x86_64-unknown-linux-gnu --locked`; and `cargo build -p bredboard-app
  --target wasm32-unknown-unknown --locked`.
- Menu selection, dial dispatch, exercise text, fixed-step behavior, and
  control/reset paths are covered by the Bevy app tests, including all five
  new circuits in `all_circuits_are_selectable_via_the_scrolled_menu` and
  `new_exercises_show_title_explanation_and_task_text`.
- Required manual Linux executable inspection remains blocked in this
  environment. In the normal task environment, launching
  `target/x86_64-unknown-linux-gnu/debug/bredboard-app` fails with
  `WaylandError(Connection(NoCompositor))`; forcing X11 fails with
  `XNotSupported(XOpenDisplayFailed)`. A host-display launch succeeded, but
  the desktop was locked and no native app surface was available to the UI
  connector, so no safe visual or input inspection was possible. No windowed
  inspection is claimed.
