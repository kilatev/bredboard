# 212-scheme catalog execution plan

This document turns the 212-entry reference catalog into an executable
content-delivery sequence. It is a planning document, not an implementation
task or an active Codex Goal. Before work starts on a batch, create a bounded
task card that copies the batch scope, evidence requirements, and acceptance
criteria below.

The source of truth for the input corpus is
[`breadboard-circuits/spec/catalog.json`](../../breadboard-circuits/spec/catalog.json),
its section Markdown files, and the matching SVG references. The source
corpus is reference material: a schematic or BOM is not accepted as
buildable until it passes the review and assembly gates in this document.

## Delivery outcome

The outcome is a catalog in which every entry has an explicit disposition:

- `ready`: implemented as a validated local fixture and accepted for the
  product's content release;
- `blocked_component`: waiting for a supported electrical model, pin contract,
  or required art;
- `blocked_design`: the source schematic, BOM, safety assumptions, or layout
  needs a design decision or correction;
- `physical_scope`: depends on a physical prop, multi-board system, or other
  experience not represented by the current board model;
- `out_of_scope`: explicitly rejected with a recorded reason.

No entry is silently dropped, replaced by a scripted outcome, or marked ready
only because its SVG renders. A blocked entry remains visible in the catalog
ledger until its disposition changes.

## Rules that apply to all batches

1. Keep the local Rust core authoritative for electrical behavior. Do not add
   circuit-specific simulation scripts to make a reference drawing appear to
   work.
2. Preserve the current board topology rule: connectivity is derived from
   holes, component pins, and wire endpoints. A schematic label or visual
   crossing is not a second editable netlist.
3. Keep the existing fixed-step solver and fixture architecture unless a
   concrete catalog entry demonstrates a blocker. Record the blocker before
   introducing a new abstraction.
4. Use English for all shipped titles, explanations, tasks, diagnostics, BOM
   descriptions, and implementation notes. Russian source text is input for
   translation, not shipped UI text.
5. Keep safety constraints from the source specification: battery or adapter
   supplies in the 5–12 V range, no mains voltage, and battery-only skin
   electrode experiments. Flag any source entry that violates these rules.
6. Preserve unrelated work and keep each batch small enough to produce one
   coherent, reviewable change.

## Per-scheme execution pipeline

Every scheme follows the same sequence. A batch may group related schemes, but
the evidence is recorded per scheme.

### 1. Freeze the source record

Record the stable source coordinates: section, source filename, catalog index,
scheme name, SVG path, source Markdown heading, and the source `catalog.json`
entry. Copy the source BOM and note whether the SVG is a complete schematic,
a block diagram, or one half of a find-the-bug pair.

### 2. Review the design

Check the schematic and BOM together, then compare both with the source notes.
The review must cover:

- component identity, quantity, value, rating, polarity, and package;
- every pin name, pin numbering convention, orientation, and intended
  connection;
- power source, rail continuity, common ground, current paths, and possible
  shorts;
- whether inputs have defined states and whether unused IC inputs are handled;
- whether the behavior described in the source can be produced by the listed
  parts without hidden firmware or a missing module;
- whether the layout fits one board or needs multiple boards or off-board
  props;
- learner safety, especially energy storage, motors, lasers, skin contact,
  and any source that is not actually low voltage.

Record each finding as `source_defect`, `implementation_blocker`,
`safety_blocker`, or `manual_assembly_finding`; do not mix these categories.

### 3. Decide scope and readiness

Map every BOM item to an existing component kind, a new component/model
requirement, a physical prop, or an explicit exclusion. Assign one of the
five dispositions above and record the next action. A scheme can proceed only
when its component contracts, safety assumptions, and board scope are clear.

### 4. Define the target content contract

For a scheme that proceeds, write the English content package:

- stable content ID and section/order metadata;
- title and one-paragraph behavior explanation;
- parts list with canonical kinds, values, quantities, and ratings;
- learner task, controls, expected observable states, and any hint text;
- board size, rail convention, hole-level placement, pin mapping, and wires;
- calculated readouts and the diagnostics that should be shown on failure;
- the source SVG link as a reference, without treating it as a second circuit
  definition.

If the current fixture format cannot express a required field, record the
smallest concrete format or app change needed. Do not pre-build a general
lesson engine or a second connectivity model for a hypothetical scheme.

### 5. Implement the electrical and visual fixture

Add or reuse the required component model, sprite, project fixture, and menu
entry. Route behavior through explicit actions and fixed simulation steps.
Use stable domain IDs and the existing project validation/topology compiler.
For a new component, include its pin contract, parameter range, model
equations or approximation, visual states, and attribution if an external
asset or source is introduced.

### 6. Verify automatically

At minimum, add the checks appropriate to the scheme:

- project/schema validation and reference integrity;
- topology and pin-orientation checks;
- expected solvability or bounded diagnostic behavior;
- numerical or state assertions for the documented observable behavior;
- current balance and deterministic action ordering where applicable;
- property tests for the component/model invariant when a new model is added;
- stable fixture loading and menu registration.

Keep reproducible seeds and minimized regressions for property-test failures.
Record exact commands and results in the task card. An unavailable required
check is a blocker, not a pass.

### 7. Perform the manual assembly check

Select representative schemes from each batch, plus every scheme with a new
physical risk, unusual pin mapping, multiple boards, or a source-design
finding. Assemble them on the documented solderless board using the source
BOM and the fixture's hole-level layout.

The checklist is:

1. confirm board count, rail convention, supply voltage, and polarity with
   power disconnected;
2. check continuity and the absence of an unintended supply short;
3. verify package orientation, pin numbering, jumper endpoints, and rail
   breaks;
4. power from the approved low-voltage source and observe startup behavior;
5. exercise every button, switch, sensor, or control state;
6. compare the physical output and measured voltage/current with the
   documented behavior and fixture readouts;
7. record photos or an equivalent visual record, deviations, unsafe behavior,
   missing parts, and ambiguous instructions;
8. feed findings back into the source-defect ledger or implementation task.

Manual assembly is independent acceptance evidence. Passing Rust tests or
rendering an SVG does not prove that a learner can reproduce the circuit.

### 8. Close the scheme and batch

A scheme becomes `ready` only when its source review, content contract,
automated checks, and required manual evidence are recorded. The batch card
then records the accepted count, blocked count by reason, deferred findings,
and the next dependency. Do not claim that the whole 212-entry catalog is
complete while any entry lacks a disposition.

## Foundation work before content batches

These are the shared preparation steps. They cover the whole catalog once and
must not be repeated as ad-hoc work in every batch.

### C00 — inventory and audit ledger

1. Validate `catalog.json` against `catalog.schema.json` and reconcile its
   section counts with the source README and SVG directories.
2. Assign a stable audit key to all 212 entries without changing the source
   catalog's meaning.
3. Create the ledger columns for source coordinates, scope disposition,
   component mapping, design defects, safety review, implementation status,
   fixture path, automated evidence, manual evidence, and release decision.
4. Review all 20 sections once for obvious missing files, duplicate entries,
   malformed BOMs, incomplete diagrams, and non-breadboard props.
5. Produce the first cross-section component inventory. Group synonyms such
   as resistor values and package variants, but do not collapse distinct
   electrical behavior.

### C00b — shared assembly protocol and component readiness

1. Fix the reference board size(s), rail convention, approved supply range,
   tools, measurement units, and photo/notes format.
2. Define the canonical pin and package naming rules used when translating a
   source schematic into a project fixture.
3. Make a readiness table for each required component family: existing model,
   model needed, sprite needed, pin ambiguity, and safety review.
4. Decide the scope of multi-board systems, physical props, and section 20's
   faulty/correct presentation before those sections get implementation cards.
5. Record which findings can be resolved in content and which require a
   solver, board, rendering, or interaction change.

The output of C00/C00b is an audit baseline, not 212 implemented levels.

## Implementation batches

The table assigns every source section to one execution batch. The section
count is the number of source schemes, not the number of schemes that will
necessarily be accepted as playable content. A batch can split into smaller
task cards after C00 if its entries require different component capabilities.

| Batch | Source sections | Schemes | Primary capability gate | Depends on |
| --- | --- | ---: | --- | --- |
| C01 | 01 — Level 1; 02 — Transistors | 13 | Basic discretes, PNP promotion where needed, safe board fixtures | C00b |
| C02 | 03 — First ICs | 7 | First timer/comparator/gate contracts; split analog and digital entries as needed | C00b, C01 |
| C03 | 04 — Counting/display; 05 — Systems | 17 | Digital primitives, counters, registers, displays, multi-board scope | C02 |
| C04 | 06 — Sound; 11 — Op-amps; 16 — Analog computer | 27 | Timer, op-amp, oscillator, audio, and analog-computing models | C02 |
| C05 | 07 — Home tools; 10 — Power/energy | 24 | Measurement fixtures, regulators, rectification, storage, and power safety | C01, C02, C04 |
| C06 | 08 — Motors | 19 | Motor/servo/stepper/solenoid behavior, drivers, motion presentation | C02, C03, C05 |
| C07 | 09 — Hardware logic/modules | 19 | Ready-made module contracts and digital/module interaction | C02, C03, C05 |
| C08 | 12 — Digital/analog; 14 — Light/effects; 17 — Light communication | 22 | ADC/DAC, displays, optical coupling, IR, and communication components | C03, C04, C07 |
| C09 | 13 — Biosignals/science; 15 — Relay logic | 14 | Instrumentation/sensor safety and relay coil/contact behavior | C04, C05, C06 |
| C10 | 18 — Hacks | 16 | Per-entry scope decisions for improvised materials and physical props | Relevant prior batch for each entry |
| C11 | 19 — World tasks | 22 | Capstone composition, multi-board systems, motors/modules and UI presentation | C05–C09 |
| C12 | 20 — Find the bug | 12 | Fault-pair content, fault highlighting, diagnosis and correction flow | The component phase of each paired scheme |
| **Total** | **20 sections** | **212** |  |  |

### Batch-specific steps

Each batch uses the per-scheme pipeline above and adds these focus checks:

- **C01:** establish the canonical fixture/layout pattern and verify that
  novice-readable wiring remains possible before adding denser circuits.
- **C02:** define DIP pin numbering, unused-input handling, timing behavior,
  and the boundary between analog and digital state.
- **C03:** verify clock/reset states, display segment mapping, debouncing, and
  the scope of multi-board systems before accepting counters.
- **C04:** document approximation equations, time-step requirements,
  oscillator startup conditions, audio presentation limits, and stable initial
  conditions.
- **C05:** check current/voltage limits, polarity, stored energy, charger
  safety, and whether a measurement exercise describes a real measurable
  quantity rather than a scripted number.
- **C06:** define the electrical-to-motion contract, stall/start behavior,
  driver protection, direction control, and deterministic presentation state.
- **C07:** define every module's pin-level input/output contract and ensure
  that no module depends on hidden firmware, bus commands, or network access.
- **C08:** verify analog/digital boundary conditions, display timing,
  optical alignment assumptions, and the distinction between light coupling
  and a direct wire.
- **C09:** keep biosignal experiments battery-only and safety-reviewed; verify
  relay isolation, coil flyback handling, contact state, and mechanical
  presentation.
- **C10:** decide entry by entry whether an improvised material is representable
  as a board component, needs a physical-prop mode, or is out of scope. Do not
  invent a visual substitute that changes the lesson's electrical meaning.
- **C11:** split large systems into explicit subassemblies, document extra
  boards and off-board props, and accept only the parts that fit the current
  product surface.
- **C12:** preserve the known-good and faulty variants, identify exactly one
  intended fault unless the task explicitly teaches multiple faults, and test
  that the diagnostic points to the learner-correctable cause.

## Batch acceptance gates

Each batch card must report these gates separately:

| Gate | Required evidence | Result if missing |
| --- | --- | --- |
| Source integrity | All assigned entries have source coordinates and reviewed SVG/BOM/notes | The entry is blocked |
| Design and safety | Findings classified; no unresolved critical safety or wiring contradiction | The entry is blocked |
| Capability readiness | Every BOM item has a supported model, an approved new-model task, or a recorded scope disposition | The entry is blocked or scoped out |
| Fixture correctness | Valid project, stable IDs, derived topology, readable placement, menu/content registration | The entry is not ready |
| Electrical behavior | Solver/property/regression checks and bounded diagnostics pass | The entry is not ready |
| Manual buildability | Required representative assembly evidence and findings recorded | The batch is not ready |
| Release decision | Ready/blocked/physical/out-of-scope counts reconcile with the ledger | The batch remains open |

## Completion of the 212-scheme initiative

The initiative is complete only when:

1. all 212 source entries appear in the ledger;
2. all 212 have a current disposition and no unexplained omissions;
3. every `ready` entry has the source review, English content package, valid
   fixture, automated evidence, and required manual evidence;
4. every blocked or out-of-scope entry has a written reason, owner-facing
   next decision, and no misleading menu entry;
5. each component family introduced by the catalog has documented pins,
   ranges, visual states, model limitations, tests, and attribution review;
6. batch evidence names exact commands and results, and unavailable checks
   remain visible as blockers;
7. the final release decision distinguishes implemented content from the
   reference corpus and from deferred product capabilities.

This plan does not authorize free assembly, a lesson scripting engine,
browser interaction acceptance, `.cir` export, ngspice differential testing,
or Steam packaging. Those remain separate roadmap decisions unless a concrete
catalog blocker causes the owner to request a new bounded task.
