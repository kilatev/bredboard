# Post-MVP roadmap

The MVP is complete at T14. These phases are post-MVP work and are not acceptance requirements for the released MVP. Before implementing each phase, prepare a dedicated specification and commit-sized task cards using the same Goal and verification conventions as the MVP. Do not reopen MVP scope while executing them.

## Browser deployment verification

Package and verify the same Bevy app in a browser after Linux MVP acceptance. The wrapper remains a canvas and startup call; menu, controls, and rendering stay in the shared app. Test complete interactions in Chromium and Firefox then. Compare native and WASM discrete states exactly and voltage/current values with `abs(a-b) <= atol + 1e-6 * max(abs(a), abs(b))`, using `atol = 1e-6 V` or `1e-9 A` respectively. The current WASM build is a compatibility check, not browser acceptance.

## 1. Free assembly and files

Add placement of supported components, wire creation, deletion, undo/redo, and user-facing project JSON import/export. Reuse the existing board topology, project JSON, actions, and solver. Define placement conflicts and simulation behavior during edits in this phase's specification. Do not introduce a second connectivity source of truth. Add snapshot file controls only if the editing workflow needs resumable sessions.

## 2. AI-authored lessons

Add guided built-in lessons if teaching needs more than the three fixed experiments. Extend project sharing with declarative lesson steps, hints, and measurable completion conditions when imported lessons are requested. Validate references and conditions before use; imported content must not execute arbitrary code. Provide English schemas and complete AI-authoring examples.

## 3. Analog expansion

Add signal sources, RC filters, operational amplifiers, and oscilloscope functionality, then oscillators. Re-evaluate numerical integration, time-step requirements, and model accuracy using frequency and transient reference cases before implementing broader claims of analog accuracy.

## 4. Digital logic — separate plan

Support gates, flip-flops, counters, and shift registers without firmware or user programming. Design event scheduling and analog/digital time coordination explicitly. Do not pre-implement a mixed-signal solver in the MVP.

## Optional presentation work

[T18 — Readable automatic breadboard layout](../tasks/T18-readable-layout.md) adds a deterministic readability-focused placement heuristic for existing projects. It changes placements while preserving the same board topology; it does not add an editor or a schematic view.

Isometric rendering may become another projection of the same board model. Additional localizations may reuse separated English UI and lesson strings. Neither is a prerequisite for the phases above.
