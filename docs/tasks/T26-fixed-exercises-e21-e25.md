# T26 — Fixed exercises E21–E25

Status: ready_for_fukit

## Dependencies

[T21 — Scrollable exercise menu](T21-scrollable-exercise-menu.md) and
[docs/roadmap/LESSONS.md](../roadmap/LESSONS.md) (its "Extended exercise list
(E11–E30)" section). T21's acceptance criteria must be satisfied before
starting this task. Independent of T22/T23/T24/T25/T27.

## Outcome and commit boundary

Add 5 new fixed exercises to the menu (E21–E25), each buildable with kinds
already in the catalog: `dc_voltage_source`, `resistor`, `led`,
`momentary_button`, `npn_transistor`, `photoresistor`, `potentiometer`.

Suggested commit title: `feat: add five fixed practice exercises (E21-E25)`.

Do not add other exercises (other cards). Do not change the existing three
MVP benches, other exercise cards' work, or the menu mechanism itself (T21).

## Scope and interfaces

| Exercise | Wiring |
| --- | --- |
| E21 Adjustable Night-Light Threshold | `B1+ -> R2(photoresistor) -> N1;  N1 -> RV1 -> B1-;  N1 -> Q1(B);  B1+ -> R1 -> D1(A->K) -> Q1(C->E) -> B1-` |
| E22 Mixed Logic | `B1+ -> N1; N1 -> S1 -> S2 -> N2; N1 -> S3 -> N2; N2 -> R1 -> D1(A->K) -> B1-` |
| E23 Light and Sound Together | `B1+ -> R1 -> D1(A->K) -> N1;  B1+ -> BZ1(+->-) -> N1;  N1 -> Q1(C->E) -> B1-;  B1+ -> R2 -> Q1(B);  Q1(B) -> R3(photoresistor) -> B1-` |
| E24 Capacitor Charge and Discharge | `SW1(common) <- B1+;  SW1(A) -> R1 -> D1(A->K) -> N1;  SW1(B) -> R2 -> D2(A->K) -> N1;  N1 -> C1(+->-) -> B1-` |
| E25 Transistor AND | `B1+ -> R1 -> D1(A->K) -> Q1(C->E);  Q1(E) -> Q2(C->E) -> B1-;  B1+ -> R2 -> S1 -> Q1(B);  B1+ -> R3 -> S2 -> Q2(B)` |

- Write each fixture's English title, one-paragraph behavior explanation,
  parts list, and player task, adapted from the source mockup linked in
  `docs/roadmap/LESSONS.md`.
- E21 needs both the photoresistor's ambient-light dial and the
  potentiometer's dial simultaneously on one bench; confirm the existing
  dial-control mechanism (T19/T23) supports two independent dials on the same
  circuit, or extend it minimally if it currently assumes exactly one.
- E22 needs three independent momentary buttons on one bench; confirm the
  existing multi-button control row (generalized in T22 for E9's two buttons)
  scales to three without layout regressions (reuse T22's evidence about
  keeping `RESET`/`CIRCUITS` on-screen; verify for three buttons specifically,
  not just two).
- E24's `changeover_switch` routes into two different LED branches that share
  a capacitor; the capacitor is common to both switch positions, unlike E11
  where the branches are fully separate.
- E25's transistor chain (`Q1(E) -> Q2(C->E)`) stacks the two transistors'
  collector-emitter paths in series; verify the solver's transistor model
  handles a transistor's emitter driving another transistor's collector
  correctly (this differs from every prior transistor fixture, which put a
  transistor's collector-emitter path directly across a single LED branch to
  the negative rail).
- Reuse existing parameter values and ranges already validated by prior
  catalogs; do not invent new ranges.

## Acceptance criteria

- [ ] All 5 fixtures are valid, solvable, fixed projects, selectable from the
  menu and reachable via T21's scrolling.
- [ ] Each exercise shows its English title, explanation, parts list, and task
  text in the app, matching its fixture.
- [ ] E21's LED-on threshold (as a function of the ambient-light dial) shifts
  monotonically with the potentiometer dial, verified at multiple
  potentiometer settings.
- [ ] E22's LED lights iff `(S1 AND S2) OR S3`, verified against all eight
  button combinations.
- [ ] E23's LED and buzzer both switch on and off together as the
  ambient-light control crosses the transistor's threshold.
- [ ] E24's two LED branches show charge (brief bright flash, fast decay) and
  discharge (flash, slow decay) behavior distinctly per switch position,
  verified by simulated-step current traces.
- [ ] E25's LED lights only when both S1 and S2 are held, verified against all
  four combinations, and is contrasted in its task text with E9 (T22).
- [ ] Existing three MVP benches and other exercise cards' work are unchanged.
- [ ] Property/unit tests cover topology validity and the button-combination/
  monotonicity claims above for all 5 new fixtures.

## Required verification

- Run the baseline formatting, Clippy, workspace test, Linux build, and WASM
  build commands documented in `README.md`.
- Run fixture validation with the existing tools validation command for all 5
  new fixtures.
- Inspect all 5 exercises in the Linux executable: selection from the scrolled
  menu, all controls (three buttons for E22, two dials for E21), readings,
  and the specific check in each exercise's player task. Do not test browser
  interaction.

A missing or failing required check blocks readiness.

## Codex Goal

```text
/goal Complete T26 in docs/tasks/T26-fixed-exercises-e21-e25.md according to
docs/PLAN.md, AGENTS.md, and docs/roadmap/LESSONS.md. Add fixtures, menu
entries, and English text for exercises E21-E25 using only existing component
kinds and control mechanisms, through the scrollable menu from T21, extending
the multi-control and multi-dial mechanisms only as far as E21/E22 require.
Do not add other exercises, and do not change the existing three MVP benches
or other exercise cards' work.
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

- Added `fixtures/projects/e21-adjustable-night-light-threshold.json` through
  `e25-transistor-and.json`, with valid fixed projects using existing catalog
  kinds and ranges. E22 includes a 1 Ω resistor in the S3 branch to avoid the
  solver's underdetermined parallel-ideal-switch case while preserving the
  required `(S1 AND S2) OR S3` behavior.
- Added E21–E25 to the existing scrollable menu and added English title,
  explanation, parts list, and task text for each fixture. E21 renders two
  independent horizontal dials; E22 renders three button controls; E24 a
  changeover switch; E25 two buttons.
- Added regression tests for E21 threshold monotonicity and two-dial dispatch,
  E22's all-eight truth table, E23 synchronized LED/buzzer switching, E24
  distinct fast/slow capacitor current decay, and E25's all-four transistor
  AND combinations. Full app suite passed: 39 tests.
- Fixture validation passed for all ten E16–E25 projects with
  `cargo run -p bredboard-tools --locked -- validate <fixture>`; every output
  reported a valid project and derived topology nodes.
- Required baseline checks passed on 2026-09-27:
  `cargo fmt --all --check`; `cargo clippy --workspace --all-targets --locked
  -- -D warnings`; `cargo test --workspace --locked` (39 app, 59 core, 2
  tools tests); `cargo build -p bredboard-app --target
  x86_64-unknown-linux-gnu --locked`; and `cargo build -p bredboard-app
  --target wasm32-unknown-unknown --locked`.
- Existing menu selection/text/parts-list tests now cover all 28 menu entries;
  E21's two dials are independently updated through ordered core ratio
  actions, and the generic control row is exercised by the three-button E22
  truth-table test.
- Required manual Linux executable inspection remains blocked in this
  environment. In the normal task environment, launching
  `target/x86_64-unknown-linux-gnu/debug/bredboard-app` fails with
  `WaylandError(Connection(NoCompositor))`; forcing X11 fails with
  `XNotSupported(XOpenDisplayFailed)`. A host-display launch succeeded, but
  the desktop was locked and no native app surface was available to the UI
  connector, so no safe visual or input inspection was possible. No windowed
  inspection is claimed.
- Fresh desktop-session probe on 2026-09-27 confirmed
  `WAYLAND_DISPLAY=wayland-1`, `DISPLAY=:0`, and
  `XDG_RUNTIME_DIR=/run/user/1000`. `cargo run -p bredboard-app --locked`
  reached Vulkan and created a mapped `bredboard` window at Hyprland position
  `[0, 26]` with size `[1056, 1054]`; a compositor screenshot visibly showed
  the rendered MVP menu. The native-app CUA surface remained unavailable,
  and the desktop locked during the subsequent pointer-input attempt, so the
  required E21–E25 selection/control checks remain unclaimed.

- Fresh verification on 2026-09-28 passed `cargo fmt --all --check`,
  `cargo clippy --workspace --all-targets --locked -- -D warnings`,
  `cargo test --workspace --locked` (47 app, 59 core, 3 tools tests),
  `cargo build -p bredboard-app --target x86_64-unknown-linux-gnu --locked`,
  and `cargo build -p bredboard-app --target wasm32-unknown-unknown --locked`.
  All five fixture commands `cargo run -p bredboard-tools --locked -- validate
  fixtures/projects/e21-adjustable-night-light-threshold.json` through E25
  passed.
- The already-built Linux executable reached Vulkan and created a mapped
  `bredboard` window in the desktop session, but the current CUA runtime
  exposes no native-app binding or window screenshot/input surface. The
  required manual E21–E25 inspection therefore remains blocked and is not
  counted as passed.
- Owner verification on 2026-09-28 completed the required manual Linux
  inspection for E21–E25, including menu selection, controls, and exercise
  behavior. This supersedes the earlier environment-only inspection blocker.
