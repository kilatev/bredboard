# Post-MVP roadmap

These phases are not acceptance requirements for any MVP task. Before implementing each phase, prepare a dedicated specification and commit-sized task cards using the same Goal and verification conventions as the MVP.

## 1. Free assembly

Add placement of supported components, wire creation, deletion, and undo/redo. Reuse the existing board topology, project JSON, actions, and solver. Define placement conflicts and simulation behavior during edits in this phase's specification. Do not introduce a second connectivity source of truth.

## 2. AI-authored lessons

Extend project sharing with declarative lesson steps, hints, and measurable completion conditions. Validate references and conditions before use; imported content must not execute arbitrary code. Provide English schemas and complete AI-authoring examples. Reuse the built-in lesson semantics without requiring imported lessons in the MVP.

## 3. Analog expansion

Add signal sources, RC filters, operational amplifiers, and oscilloscope functionality, then oscillators. Re-evaluate numerical integration, time-step requirements, and model accuracy using frequency and transient reference cases before implementing broader claims of analog accuracy.

## 4. Digital logic — separate plan

Support gates, flip-flops, counters, and shift registers without firmware or user programming. Design event scheduling and analog/digital time coordination explicitly. Do not pre-implement a mixed-signal solver in the MVP.

## Optional presentation work

Isometric rendering may become another projection of the same board model. Additional localizations may reuse separated English UI and lesson strings. Neither is a prerequisite for the phases above.
