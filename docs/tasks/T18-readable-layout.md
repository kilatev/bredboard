# T18 — Readable automatic breadboard layout

Status: deferred_v3

## Dependencies

[T14 — Verified Linux MVP release artifact](T14-mvp-release.md) and [T17 — Sprites for the remaining and future simple components](T17-more-component-sprites.md). The MVP fixtures, board model, sprites, and verification commands must remain accepted before starting this task.

## Outcome and commit boundary

Add a deterministic Rust layout solver that rearranges valid breadboard projects for readability rather than compactness. It must reduce visible component and wire overlap in the LED, RC, and transistor fixtures while preserving their electrical topology and simulation behavior.

Suggested commit title: `feat: add readability-focused breadboard layout`.

The deliverable is a post-MVP presentation improvement. Do not add free assembly, drag-and-drop editing, undo/redo, user-facing file controls, schematic rendering, or new electrical models.

## Scope and interfaces

- Add a platform-independent layout API in `bredboard-core` that accepts a valid `Project` and returns a new valid `Project` or a bounded layout diagnostic. The input project must not be mutated or partially replaced on failure.
- The result may change component pin hole assignments and wire endpoints, but must preserve component IDs, kinds, pins, parameters, initial conditions, and electrical node equivalence. Reuse `compile_topology`; do not introduce a second editable netlist.
- Support the existing `HalfSizeSolderless` board and all valid projects within the documented component, node, wire, and hole limits. Projects that cannot fit must fail clearly instead of returning a partial layout.
- Use only deterministic Rust code and existing dependencies. Do not embed ELK/elkjs, a JavaScript runtime, Graphviz, or a new optimization framework.
- Start with a layered graph layout: derive stable connectivity groups, handle cycles as stable strongly connected groups, order layers with barycenter/median sweeps, assign legal holes, then run bounded local swaps/flips to improve the score.
- Score readability in this priority order: component/wire overlap, wire crossings, wire bends, backwards flow, then wire length. Keep the scoring function pure and testable; it is a readability heuristic, not a claim of global optimality.
- Add a tools entry point or equivalent verification command that lays out a project and reports its score and structural validation result. Do not silently rewrite fixture files as part of ordinary validation.

Follow the shared architecture, language, persistence, and verification contracts in [the plan](../PLAN.md) and [the post-MVP roadmap](../roadmap/POST-MVP.md). Repository publication and `fukit` are not part of this task.

## Acceptance criteria

- [ ] The layout API returns a new valid project for every embedded LED, RC, and transistor fixture.
- [ ] The three embedded fixtures have no component/component, component/wire, or wire/wire overlaps under the app's board geometry, and their readability score is lower than the current fixture layout.
- [ ] Re-layout preserves component and wire counts, IDs, kinds, parameters, initial conditions, and the partition of electrical contacts into derived nodes.
- [ ] Solving the original and laid-out fixtures produces equivalent calculated readings within the existing numerical tolerances.
- [ ] Running layout twice with the same project and seed produces identical JSON and score. Any seed is explicit and reproducible; no wall-clock or frame-rate state is used.
- [ ] Property tests cover topology preservation, unique occupied holes, deterministic output, and bounded diagnostics when the board cannot fit a project.
- [ ] A regression test covers the documented transistor fixture overlap between R1 and R2.
- [ ] The app renders all three laid-out fixtures legibly in the Linux executable. WASM remains a compilation check; browser interaction is out of scope.
- [ ] The documentation records the heuristic's known ceiling: it is a deterministic breadboard placement heuristic, not an exact optimizer or a general schematic layout engine.

## Required verification

- Run the baseline formatting, Clippy, workspace test, Linux build, and WASM build commands documented in `README.md`.
- Run the focused layout unit, property, topology-equivalence, fixture-score, and tool-command tests.
- Validate all committed fixtures with the existing tools validation command.
- Inspect the LED, RC, and transistor fixtures in the Linux executable after layout, including readable component bodies, wires, pin holes, and controls. Do not test browser interaction.
- Record exact commands and results below. An unavailable required check is a blocker, not a pass.

## Codex Goal

```text
/goal Complete T18 in docs/tasks/T18-readable-layout.md according to
docs/PLAN.md and AGENTS.md. Add a deterministic Rust layout solver that
rearranges valid breadboard projects for readability while preserving
electrical topology and simulation behavior. Use the existing board model and
fixtures; do not embed ELK/elkjs or add a free-assembly editor.
Satisfy every acceptance criterion and run every required check in the card.
Fix task-scoped findings without weakening tests or acceptance criteria.
Do not implement successor tasks. Prepare one coherent change for review;
do not commit or push. Record exact verification evidence in this card.
If a required check is unavailable, report the exact blocker and the input
or environment change needed; do not count it as passed.
```

## Completion and fukit handoff

When all criteria pass, record evidence and set `Status: ready_for_fukit`. Stop without starting successor tasks. The user may then invoke `fukit` to describe, commit, and push.

An existing jj repository and an unambiguous authorized remote/bookmark are required for that workflow. Do not initialize or guess them. Commit completion is evidenced by jj history; publication is evidenced by the actual push result.

## Evidence

Not started.

## Deferred findings from the abandoned first implementation attempt

- Contact-group row swaps optimized topology and generic score fields, but made the rendered circuits less readable.
- Two-segment L-routing produced long rectangular loops and could still cross unrelated wires.
- Generic core footprints did not match sprite geometry closely enough to predict visual collisions.
- Wire thickness, draw order, and endpoint markers materially affected readability but were not represented consistently in the score.
- External supply leads used a separate visual routing path from internal wires.
- The next attempt should begin from rendered schematic composition and preserve the existing fixture geometry until a route/placement feedback loop is proven visually.
