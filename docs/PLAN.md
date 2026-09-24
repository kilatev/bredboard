# bredboard — Product and technical plan

## Outcome and scope

Build an educational 2D breadboard simulator for a school-age beginner with no electronics background. Use Rust and Bevy, with an independently testable electrical core. Linux is the primary desktop target; validate the same application in Chromium and Firefox through WASM from the first runnable milestone.

English is canonical for all project artifacts: plans, task cards, documentation, code comments, UI, board labels, component descriptions, lessons, diagnostics, JSON Schema descriptions, examples, and commit messages. Keep user-facing strings separate from logic. Russian localization is outside the MVP.

The MVP provides three guided lessons, parameter editing, switches, voltage/current measurement, a voltage graph, run/pause/single-step/reset, connected-hole highlighting, and JSON import/export. Users can import new assemblies using the supported catalog even though interactive assembly is not yet available. Imported assemblies open as unguided benches.

Exclude free assembly, isometric rendering, programmable devices, imported lessons, accounts, a backend, and integrated AI chat. External AI tools can author documented JSON. Simulate overload warnings but not heating, production variability, wire parasitics, or irreversible component damage.

## Core and presentation

Organize a Cargo workspace into an independent core, a Bevy application, and verification tools. Pin compatible toolchain/dependency versions during T01 and retain the lockfile. Do not select moving versions during later tasks without a task-scoped reason.

The core owns these contracts:

- `Project`: board definition, components, parameters, pin placements, wires, and initial conditions.
- `SimulationState`: integer step/time, component internals, calculated readings, and diagnostics.
- `LessonState`: current lesson/step and satisfied conditions.
- `Action`: parameter edits, switches/buttons, simulation controls, and reset.
- Project validation, topology compilation, action reduction, and electrical stepping usable without a window.

Bevy converts input to ordered actions and renders core results. ECS entities reference stable domain IDs. Cursor, hover, and animation state are presentation concerns; do not serialize ECS as the circuit format. The core must not perform rendering, file access, or wall-clock reads. In-place Rust updates are allowed; Elm-style ownership of transitions does not require cloning the entire circuit each step.

Apply actions in order between fixed calculation steps. Frame rate does not change electrical time. When computation cannot keep up, slow simulation relative to wall time instead of dropping steps. Show calculation failures explicitly rather than displaying stale readings as current values.

## Board and electrical simulation

Support one documented breadboard model with explicit contact groups and power-rail continuity. Derive electrical nodes from connected holes, component pins, and wire endpoints. Screen-space crossings do not connect wires. Do not expose a second editable connectivity representation that can disagree with the board layout.

Initial catalog: DC voltage source, resistor, LED, capacitor, NPN transistor, momentary button, changeover switch, and wires. Document pin names, model parameters, units, ratings, and supported parameter ranges in the versioned catalog as each model is introduced. Use one common solver for arbitrary supported assemblies rather than circuit-specific scripted results.

Selected numerical baseline:

- Modified nodal analysis (MNA), `f64`, stable node/component ordering, sequential execution.
- Backward Euler capacitor integration with a fixed 100 microsecond step.
- Iterative nonlinear solution with a diode LED model and simplified Ebers–Moll NPN model without parasitic capacitances.
- Scope limits: 64 components, 128 compiled electrical nodes, and 256 wires. Check limits before expensive computation.
- Bounded nonlinear iterations and explicit nonconvergence diagnostics; implementation constants and supported model ranges must be documented and covered by tests when introduced.

Diagnose invalid parameters, floating circuits, contradictory sources, and nonconvergence. Distinguish an unresolvable ideal-source short from a calculable component overload. Stop simulated time on calculation failure, keep the last valid state, and mark its readings stale. Overloads in a solvable circuit produce warnings and continue without damage simulation.

T06 is the feasibility checkpoint for the fixed-step solver and web build. Resolve failures before adding the remaining lessons; changing the baseline requires an explicit plan update supported by measurements, not weaker assertions.

## Persistence and AI interface

Use `serde`, `serde_json`, and JSON Schema generated with Schemars. Version the format and publish English field descriptions, SI units, component IDs, pin IDs, hole IDs, limits, and complete examples.

- A **project** represents an assembly and initial conditions for AI authoring and human sharing.
- A **snapshot** includes its project, model/solver versions, current step, component internals, and any built-in lesson progress needed to continue.
- An **action log** records ordered actions and their simulation-step boundaries for replay from a specified initial state.

Validate structure, references, placement, catalog parameters, limits, and supported versions before replacing the current project. Structural validity is separate from electrical solvability. A structurally valid but unsolvable bench should show the solver diagnostic; malformed files must leave the current project untouched. Regenerate derived connectivity rather than trusting serialized caches.

Stable ordering and explicit internal state must make save/restore and replay testable. Unsupported versions fail with useful errors; do not invent silent migrations. Browser file selection/download and native file access belong in application adapters.

## Lessons and UI

Show a top-down board, pin labels, connected-hole highlighting, measurement points, readable values/units, and voltage-over-time plotting. Parameters are editable within documented model bounds. The MVP does not provide a schematic view or drag-and-drop wiring editor.

Each lesson contains a short English instruction, visual hint, expected effect, and a simple explanation. Conditions are evaluated in the core from calculated measurements and simulated time, not animations or elapsed wall time.

| Lesson | Learner actions | Observable completion condition |
| --- | --- | --- |
| LED and resistor | Enable power and increase resistance | Calculated current decreases; displayed brightness follows current |
| Capacitor charging/discharging | Operate the charge/discharge switch | Voltage crosses documented thresholds in both directions |
| Transistor switch | Press and release the button | Load current reaches documented on/off ranges |

Use explicit fixture parameters and thresholds in lesson content, tested at their boundaries. Save and restore lesson progress with snapshots. Custom imported assemblies run without built-in lesson assumptions.

## Verification and release criteria

Add tests with each feature, not only at final integration. Use Proptest generators restricted to the class of circuits appropriate to each electrical invariant; arbitrary malformed or floating graphs are not valid inputs to conservation assertions.

- Analytical resistive reference cases and RC transients; RC error at control samples must stay within 1% of source voltage.
- Nonlinear references compared with ngspice using equivalent model parameters. Document fixture provenance, model equivalence, and tolerance; this does not promise precision for every real component.
- Current balance on solvable generated circuits, ID-renaming invariance, JSON-array-order invariance, project round trips, snapshot continuation, and replay.
- Deterministic action ordering and discrete states independent of rendering FPS.
- Native/WASM discrete states match exactly. For voltage/current comparisons use `abs(a-b) <= atol + 1e-6 * max(abs(a), abs(b))`, with `atol = 1e-6 V` or `1e-9 A` respectively.
- Invalid imports, contradictory sources, floating nodes, limit violations, and nonlinear failure produce bounded diagnostics without hangs or misleading live readings.
- Complete all lessons and test file import/export in Linux, Chromium, and Firefox. A successful WASM build alone is not browser acceptance.

T01 establishes exact formatting-check, Clippy, test, Linux-build, and WASM-build commands. Later cards add numerical, property, and interaction checks. Report unavailable required checks as blockers rather than passes.

The project uses the owner-selected [MIT License](../LICENSE) from the initial GitHub commit. English contributor/build instructions and dependency/asset license and attribution review are prerequisites for the MVP release. Public hosting and repository publication require an established destination and authorization.

## Delivery

Execute [the task sequence](TASKS.md), one bounded Goal and coherent change at a time. The current preparation task only creates these documents. Repository initialization, remote selection, implementation, commit, and publication are separate actions.

See [the post-MVP roadmap](roadmap/POST-MVP.md) for free assembly, AI-authored lessons, expanded analog simulation, and a separate digital-logic plan.

## Design references

- [Bevy features](https://bevy.org/)
- [Schemars](https://docs.rs/crate/schemars/latest)
- [Proptest](https://proptest-rs.github.io/proptest/intro.html)
- [ngspice documentation](https://ngspice.sourceforge.io/docs.html)
- [Codex Goals](https://developers.openai.com/cookbook/examples/codex/using_goals_in_codex)
