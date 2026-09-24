# bredboard — Product and technical plan

## Outcome and scope

Build an educational 2D breadboard simulator for a school-age beginner with no electronics background. Use Rust and Bevy, with an independently testable electrical core. One Bevy app provides the same menu, controls, and board view for Linux and WASM deployments. The Linux executable is the MVP interaction target; the WASM target is compilation-checked only. The browser wrapper adds a canvas and startup call, with no separate product UI.

English is canonical for all project artifacts: plans, task cards, documentation, code comments, UI, board labels, component descriptions, lessons, diagnostics, JSON Schema descriptions, examples, and commit messages. Keep user-facing strings separate from logic. Russian localization is outside the MVP.

The MVP menu selects one of three fixed, physically buildable circuits: LED and resistor, capacitor charge/discharge, or transistor-controlled LED. Each opens paused with a run/pause control, reset, and its switch or press/release button. Show calculated voltage/current, calculation failures, and LED brightness driven by calculated current. Hovering a hole reveals its exact ID and plugged leads. No guided lesson engine, parameter editor, graph, or user-facing JSON file workflow is required for the MVP.

Exclude free assembly, isometric rendering, programmable devices, imported lessons, accounts, a backend, and integrated AI chat. External AI tools can author documented JSON through the core and CLI. The MVP does not model heating, production variability, wire parasitics, or irreversible component damage; component rating warnings can be added with editable assembly.

## Core and presentation

Organize a Cargo workspace into an independent core, a Bevy application, and verification tools. Pin compatible toolchain/dependency versions during T01 and retain the lockfile. Do not select moving versions during later tasks without a task-scoped reason.

The core owns these contracts:

- `Project`: board definition, components, parameters, pin placements, wires, and initial conditions.
- `SimulationState`: integer step/time, component internals, calculated readings, and diagnostics.
- `Action`: parameter edits, switches/buttons, simulation controls, and reset.
- Project validation, topology compilation, action reduction, and electrical stepping usable without a window.

Bevy converts input to ordered actions and renders core results. ECS entities reference stable domain IDs. Cursor, hover, and animation state are presentation concerns; do not serialize ECS as the circuit format. The core must not perform rendering, file access, or wall-clock reads. In-place Rust updates are allowed; Elm-style ownership of transitions does not require cloning the entire circuit each step.

Apply actions in order between fixed calculation steps. Frame rate does not change electrical results. The shared app schedules 1,000 core steps per wall-clock second (0.1 seconds of simulation time); when computation cannot keep up, slow simulation relative to wall time instead of dropping steps. Show calculation failures explicitly rather than displaying stale readings as current values.

## Board and electrical simulation

Support one documented breadboard model with explicit contact groups and power-rail continuity. Derive electrical nodes from connected holes, component pins, and wire endpoints. Screen-space crossings do not connect wires. Do not expose a second editable connectivity representation that can disagree with the board layout.

Initial catalog: DC voltage source, resistor, LED, capacitor, NPN transistor, momentary button, changeover switch, and wires. Document pin names, model parameters, units, ratings, and supported parameter ranges in the versioned catalog as each model is introduced. Use one common solver for arbitrary supported assemblies rather than circuit-specific scripted results.

Selected numerical baseline:

- Modified nodal analysis (MNA), `f64`, stable node/component ordering, sequential execution.
- Backward Euler capacitor integration with a fixed 100 microsecond step.
- Bounded iterative solution with a smooth forward LED diode and a base-controlled NPN collector-emitter conductance. These are educational approximations; the NPN model does not predict precise device-specific curves.
- Scope limits: 64 components, 128 compiled electrical nodes, and 256 wires. Check limits before expensive computation.
- Bounded nonlinear iterations and explicit nonconvergence diagnostics; implementation constants and supported model ranges must be documented and covered by tests when introduced.

Diagnose invalid parameters, floating circuits, contradictory sources, and nonconvergence. Stop simulated time on calculation failure, keep the last valid state, and mark its readings stale. The three fixed fixtures use safe nominal values; a component-rating system is outside this MVP.

T06 is the feasibility checkpoint for the fixed-step solver and Linux RC bench. Resolve failures before adding the other two circuits. Model limitations and reference measurements must be recorded with the relevant task evidence.

## Persistence and AI interface

Use `serde`, `serde_json`, and JSON Schema generated with Schemars. Version the format and publish English field descriptions, SI units, component IDs, pin IDs, hole IDs, limits, and complete examples.

- A **project** represents an assembly and initial conditions for AI authoring and human sharing.
- A **snapshot** includes its project, model/solver versions, current step, and component internals needed to continue.
- An **action log** records ordered actions and their simulation-step boundaries for replay from a specified initial state.

Validate structure, references, placement, catalog parameters, limits, and supported versions before replacing the current project. Structural validity is separate from electrical solvability. A structurally valid but unsolvable bench should show the solver diagnostic; malformed files must leave the current project untouched. Regenerate derived connectivity rather than trusting serialized caches.

Stable ordering and explicit internal state must make save/restore and replay testable in the core and CLI. Unsupported versions fail with useful errors; do not invent silent migrations. User-facing file adapters are deferred until interactive assembly exists.

## Board and UI

Render the documented 30-row board with A–E and F–J contact strips, a center gap, and two visually distinct continuous power rails on each side. Every displayed lead and wire endpoint must correspond to its project hole ID; built-in layouts cannot put two leads in one hole. Use readable labels and hole-hover identification so a learner can replicate the assembly on a standard solderless breadboard. The 5 V source is shown as an external supply attached to rail holes.

The shared Bevy app owns selection, run/pause, reset, circuit controls, calculated readouts, warnings, and LED glow. Do not gate these systems by deployment target. Platform-specific bootstrap may select the browser canvas. The MVP does not provide a schematic view or assembly editor.

## Verification and release criteria

Add tests with each feature, not only at final integration. Use Proptest generators restricted to the class of circuits appropriate to each electrical invariant; arbitrary malformed or floating graphs are not valid inputs to conservation assertions.

- Analytical resistive reference cases and RC transients; RC error at control samples must stay within 1% of source voltage.
- Test LED and NPN switching responses against documented fixture ranges and a separate analytical or numerical reference appropriate to the implemented educational models. Document the model equations and limits; do not promise exact results for a specific physical part.
- Current balance on solvable generated circuits, ID-renaming invariance, JSON-array-order invariance, project round trips, snapshot continuation, and replay.
- Deterministic action ordering and discrete states independent of rendering FPS.
- Preserve a compiling WASM target for the same app. Browser runtime and cross-platform interaction comparisons belong to a later deployment phase; no browser testing is part of MVP acceptance.
- Invalid imports, contradictory sources, floating nodes, limit violations, and nonlinear failure produce bounded diagnostics without hangs or misleading live readings.
- Select and operate all three fixed circuits in the Linux executable. A successful build alone is not Linux interaction acceptance.

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
