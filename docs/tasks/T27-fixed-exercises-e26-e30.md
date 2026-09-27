# T27 — Fixed exercises E26–E30

Status: in_progress

## Dependencies

[T21 — Scrollable exercise menu](T21-scrollable-exercise-menu.md) and
[docs/roadmap/LESSONS.md](../roadmap/LESSONS.md) (its "Extended exercise list
(E11–E30)" section, including its note on E30's oscillation risk). T21's
acceptance criteria must be satisfied before starting this task. Independent
of T22/T23/T24/T25/T26.

## Outcome and commit boundary

Add 5 new fixed exercises to the menu (E26–E30), each buildable with kinds
already in the catalog: `dc_voltage_source`, `resistor`, `led`,
`momentary_button`, `changeover_switch`, `capacitor`, `npn_transistor`,
`potentiometer`.

Suggested commit title: `feat: add five fixed practice exercises (E26-E30)`.

Do not add other exercises (other cards). Do not change the existing three
MVP benches, other exercise cards' work, or the menu mechanism itself (T21).

## Scope and interfaces

| Exercise | Wiring |
| --- | --- |
| E26 Transistor OR | `B1+ -> R1 -> D1(A->K) -> N1;  N1 -> Q1(C->E) -> B1-;  N1 -> Q2(C->E) -> B1-;  B1+ -> S1 -> Q1(B);  B1+ -> S2 -> Q2(B)` |
| E27 Shared Brightness Control | `B1+ -> RV1 -> N1; N1 -> Q1(B); N1 -> Q2(B); B1+ -> R1 -> D1(A->K) -> Q1(C->E) -> B1-; B1+ -> R2 -> D2(A->K) -> Q2(C->E) -> B1-` |
| E28 Power Source Selector | `B1+ -> R1 -> SW1(A);  B2+ -> R2 -> SW1(B);  SW1(common) -> D1(A->K) -> B1-/B2-(shared negative)` |
| E29 Sensitivity Detector | `B1+ -> BZ1(+->-) -> N1;  N1 -> Q1(C->E);  Q1(E) -> Q2(B);  N1 -> Q2(C->E) -> B1-;  B1+ -> R1 -> S1 -> Q1(B)` |
| E30 Two-Transistor Flasher | `B1+ -> R1 -> D1(A->K) -> Q1(C->E) -> B1-;  B1+ -> R2 -> D2(A->K) -> Q2(C->E) -> B1-;  B1+ -> R3 -> N1 -> Q1(B);  B1+ -> R4 -> N2 -> Q2(B);  N1 -> C1 -> Q2(C);  N2 -> C2 -> Q1(C)` |

- Write each fixture's English title, one-paragraph behavior explanation,
  parts list, and player task, adapted from the source mockup linked in
  `docs/roadmap/LESSONS.md`.
- E28 needs a second `dc_voltage_source` component (`B2`), wired to the
  board's second rail pair (`BP+`/`BP-`), with `B1-`/`B2-` tied together by a
  wire per `docs/roadmap/LESSONS.md`'s note that this needs no new board or
  persistence support. Verify `compile_topology` and the solver accept two
  independent sources sharing a negative node without triggering the
  `conflicting-sources`-style validation error (see
  `fixtures/projects/conflicting-sources.json` for the case that error
  guards against, and confirm E28 is not that case).
- E29's transistor chain (`Q1(E) -> Q2(B)`) feeds one transistor's emitter
  into the next transistor's base — a Darlington pair, distinct from E25's
  emitter-to-collector stacking (T26); verify the solver's transistor model
  handles this topology and produces the expected current-gain
  multiplication (documented beta values times each other, not summed).
- E30 (astable multivibrator) is the hard case flagged in
  `docs/roadmap/LESSONS.md`: it is the only free-running, unforced-oscillation
  fixed exercise. Specify a concrete, deterministic symmetry-breaking choice
  (for example, `C1` and `C2` given slightly different capacitance, or
  documented unequal initial capacitor voltages in `initial_conditions`) and
  record it explicitly in the fixture and its evidence. Do not ship a
  perfectly symmetric fixture and merely hope floating-point rounding breaks
  the tie; that is not a documented, reproducible guarantee.
- Reuse existing parameter values and ranges already validated by prior
  catalogs; do not invent new ranges beyond what this task's new topologies
  require.

## Acceptance criteria

- [ ] All 5 fixtures are valid, solvable, fixed projects, selectable from the
  menu and reachable via T21's scrolling.
- [ ] Each exercise shows its English title, explanation, parts list, and task
  text in the app, matching its fixture.
- [ ] E26's LED lights with either button alone or both together, verified
  against all four combinations, and is contrasted in its task text with E14
  (T24).
- [ ] E27's two LED branches change brightness together and monotonically as
  the potentiometer dial moves across its full range.
- [ ] E28's LED reaches comparable brightness in both switch positions
  (documented currents within the same lit-brightness band for both the 5 V
  and 9 V paths), and the two sources' negatives are confirmed tied to one
  node without a solver or validation error.
- [ ] E29's buzzer sounds when the button is held, at a base-resistor value
  (1 MΩ) that would not trigger a single-transistor switch (contrast
  documented against E8's 10 kΩ), demonstrating the Darlington gain increase.
- [ ] E30 sustains alternating LED blinking over many simulated transient
  steps from its documented (deliberately asymmetric) initial state — a
  regression test must run enough steps to observe multiple full on/off
  cycles per LED, not just a single transition.
- [ ] Existing three MVP benches and other exercise cards' work are unchanged.
- [ ] Property/unit tests cover topology validity for all 5 new fixtures, plus
  the button-combination, monotonicity, dual-source, and sustained-oscillation
  claims above.

## Required verification

- Run the baseline formatting, Clippy, workspace test, Linux build, and WASM
  build commands documented in `README.md`.
- Run fixture validation with the existing tools validation command for all 5
  new fixtures.
- Inspect all 5 exercises in the Linux executable: selection from the scrolled
  menu, controls, readings, and the specific check in each exercise's player
  task — for E30, watch the alternating blink for long enough to see several
  cycles. Do not test browser interaction.

A missing or failing required check blocks readiness.

## Codex Goal

```text
/goal Complete T27 in docs/tasks/T27-fixed-exercises-e26-e30.md according to
docs/PLAN.md, AGENTS.md, and docs/roadmap/LESSONS.md. Add fixtures, menu
entries, and English text for exercises E26-E30 using only existing component
kinds, through the scrollable menu from T21. For E28, wire a second
dc_voltage_source to the board's second rail pair with a shared negative,
without adding new board or persistence support. For E30, choose and document
a concrete deterministic symmetry-breaking initial condition and prove
sustained oscillation with a regression test over many transient steps; do
not rely on undocumented floating-point asymmetry.
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

- Added `fixtures/projects/e26-transistor-or.json` through
  `e30-two-transistor-flasher.json`, with existing catalog kinds and parameter
  ranges. E28 contains `B1` on `TP+`/`TP-`, `B2` on `BP+`/`BP-`, and an
  explicit `TP-`–`BP-` tie; the solver reports one shared negative node and no
  conflicting-source diagnostic.
- Added E26–E30 to the existing scrollable menu with English title,
  explanation, parts list, and task text. E26 has two buttons, E27 has the
  existing potentiometer dial, E28 has the existing changeover switch, and
  E29 has the existing button control.
- E30 records a deterministic `C2` initial voltage of `0.5 V` in
  `initial_conditions`, uses two 10 µF cross-coupling capacitors, and adds
  high-value base-emitter bleed resistors to keep startup solvable. The core's
  existing bounded nonlinear solve now uses documented deterministic 25%
  under-relaxation; the E30 regression observes at least four transitions per
  LED across 60,000 fixed transient steps.
- Added regression tests for E26's four button combinations, E27's equal and
  monotonic LED currents, E28's dual-source/shared-negative behavior, E29's
  1 MΩ Darlington buzzer threshold, and E30's sustained alternating cycles.
  The focused T27 checks pass, and the full app suite contains 44 tests.
- Fixture validation passed for all five with
  `cargo run -p bredboard-tools --locked -- validate <fixture>`; E26–E30
  report valid topologies with 6, 7, 6, 6, and 8 derived nodes respectively.
- Final required baseline checks passed on 2026-09-27:
  `cargo fmt --all --check`; `cargo clippy --workspace --all-targets --locked
  -- -D warnings`; `cargo test --workspace --locked` (44 app, 59 core, 2
  tools tests); `cargo build -p bredboard-app --target
  x86_64-unknown-linux-gnu --locked`; and `cargo build -p bredboard-app
  --target wasm32-unknown-unknown --locked`.
- The Linux executable inspection remains blocked as documented for T25/T26.
  In the normal task environment, launching the executable reports
  `WaylandError(Connection(NoCompositor))`; forcing X11 reports
  `XNotSupported(XOpenDisplayFailed)`. A host-display launch succeeded, but
  the desktop was locked and no native app surface was available to the UI
  connector, so no safe visual or input inspection was possible. No windowed
  inspection is claimed.
