# Post-MVP roadmap

The MVP is complete at T14. These phases are post-MVP work and are not acceptance requirements for the released MVP. Before implementing each phase, prepare a dedicated specification and commit-sized task cards using the same Goal and verification conventions as the MVP. Do not reopen MVP scope while executing them.

## Current direction and checkpoint

The active priority is the 212-scheme catalog in
[`CIRCUIT-CATALOG.md`](CIRCUIT-CATALOG.md). Work proceeds in bounded batches:

1. inspect source schematics, BOMs, and visual references;
2. record design defects and missing implementation prerequisites;
3. implement the next supported schemes as validated local fixtures;
4. manually assemble representative schemes on a real breadboard and record
   buildability findings.

The existing local solver, JSON fixtures, and fixed menu architecture remain
the default implementation surface. A new abstraction should be added only
when a concrete catalog scheme requires it.

The detailed delivery sequence is in
[CATALOG-EXECUTION-PLAN.md](CATALOG-EXECUTION-PLAN.md): one per-scheme
pipeline, shared audit prerequisites, 12 implementation batches covering all
20 sections, and explicit batch acceptance gates.

The following are explicitly deferred and are not current priorities:

- Steam packaging and release work;
- `.cir` export and ngspice differential checks;
- data-driven level metadata and project-format migration;
- free assembly, user-facing files, and AI-authored lesson machinery;
- browser interaction verification;
- broad UI, rendering, and presentation improvements.

These items remain documented for later and are not removed from the roadmap.

## Deferred: browser deployment verification

Package and verify the same Bevy app in a browser after Linux MVP acceptance. The wrapper remains a canvas and startup call; menu, controls, and rendering stay in the shared app. Test complete interactions in Chromium and Firefox then. Compare native and WASM discrete states exactly and voltage/current values with `abs(a-b) <= atol + 1e-6 * max(abs(a), abs(b))`, using `atol = 1e-6 V` or `1e-9 A` respectively. The current WASM build is a compatibility check, not browser acceptance.

## Deferred: free assembly and files

Add placement of supported components, wire creation, deletion, undo/redo, and user-facing project JSON import/export. Reuse the existing board topology, project JSON, actions, and solver. Define placement conflicts and simulation behavior during edits in this phase's specification. Do not introduce a second connectivity source of truth. Add snapshot file controls only if the editing workflow needs resumable sessions.

## Deferred: AI-authored lessons

Add guided built-in lessons if teaching needs more than the three fixed experiments. Extend project sharing with declarative lesson steps, hints, and measurable completion conditions when imported lessons are requested. Validate references and conditions before use; imported content must not execute arbitrary code. Provide English schemas and complete AI-authoring examples.

[docs/roadmap/LESSONS.md](LESSONS.md) specifies a concrete slice of this phase: 30 new fixed exercises added to the menu (still fixed fixtures, not a lesson-scripting engine). The first ten need the potentiometer, photoresistor, and buzzer component kinds; decomposed into [T19–T23](../TASKS.md#post-mvp-presentation-tasks). The next 20 (E11–E30) need no new component kinds; decomposed into [T24–T27](../TASKS.md#post-mvp-presentation-tasks). The declarative lesson-step/hint/completion-condition engine described above remains unimplemented after T19–T27.

## Deferred: analog expansion

Add signal sources, RC filters, operational amplifiers, and oscilloscope functionality, then oscillators. Re-evaluate numerical integration, time-step requirements, and model accuracy using frequency and transient reference cases before implementing broader claims of analog accuracy.

## Deferred: digital logic — separate plan

Support gates, flip-flops, counters, and shift registers without firmware or user programming. Design event scheduling and analog/digital time coordination explicitly. Do not pre-implement a mixed-signal solver in the MVP.

## Deferred: optional presentation work

[T18 — Readable automatic breadboard layout](../tasks/T18-readable-layout.md) adds a deterministic readability-focused placement heuristic for existing projects. It changes placements while preserving the same board topology; it does not add an editor or a schematic view.

Isometric rendering may become another projection of the same board model. Additional localizations may reuse separated English UI and lesson strings. Neither is a prerequisite for the phases above.

## Imported circuit catalog

`breadboard-circuits/spec/` holds a 212-circuit, 20-section source corpus
(schematics, BOMs, and a machine-readable `catalog.json`) that is far larger
than the E11–E30 slice above and does not fit the fixed-exercise pattern by
itself. [docs/roadmap/CIRCUIT-CATALOG.md](CIRCUIT-CATALOG.md) specifies how
its sections map onto phases 1/3/4 above plus new phases those don't cover
yet (electromechanical/motors, ready-made behavioral modules, power/energy
expansion). It is a specification only; no phase in it is an active card.
