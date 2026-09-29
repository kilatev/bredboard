# C01 — Level 1 and transistor reference fixtures

Status: in_progress; Section 01 fixture slice implemented, manual acceptance and
Sections 01/02 release decisions remain open.

## Dependencies

- [C00 — Catalog audit baseline](C00-catalog-audit.md) and its shared manual
  assembly protocol must be accepted.
- [C01 detailed readiness audit](../catalog/C01-READINESS-AUDIT.md) is the
  source of the per-scheme blockers and risks.
- Existing fixture architecture, stable board topology, and the current
  component contracts from the accepted MVP and post-MVP cards remain the
  boundary. This card does not authorize a second netlist or scripted result.
- The diode and PNP model/pin contracts are now present for S01-05 and
  S02-07; S02-07 still needs a bounded speaker-load fixture. The touch-pad and
  water-probe physical-input contracts are now explicit; their battery-only
  manual evidence is still required.

## Outcome and commit boundary

Review and, only after its gates pass, implement the 13 reference schemes in
source sections 01 and 02:

`C01-S01-01` through `C01-S01-06` and `C01-S02-01` through `C01-S02-07`.

Each admitted scheme becomes a calculated, hole-accurate local fixture with
English content metadata, stable IDs, readable placement, and the evidence
required by the catalog execution plan. Blocked schemes remain visible in the
ledger and do not receive a misleading menu entry.

Suggested commit title: `feat: add C01 catalog fixtures`.

This card does not authorize C02, any other catalog section, free assembly,
lesson scripting, project migrations, `.cir` export, ngspice checks, browser
interaction acceptance, or Steam work.

## Exact C01 scope

- Normalize and document the supply voltage for every fixture; the source
  drawings say 9 V while the product baseline presents a 5 V source.
- Define exact package/pin maps for every NPN and any future PNP/diode part.
- Reuse current source, resistor, LED, capacitor, NPN, button, SPDT,
  potentiometer, photoresistor, buzzer, and speaker contracts where their
  ranges cover the source BOM.
- Add only the smallest model/art contracts required by the admitted C01
  schemes. Do not add a generic lesson engine or physical simulation layer.
- Create project fixtures with derived connectivity, no duplicate pin holes,
  explicit polarity, and readable board/rail placement.
- Record English title, behavior, learner task, controls, expected readouts,
  hints, source SVG reference, BOM mapping, and failure diagnostics for each
  admitted fixture.

## Acceptance criteria

- [ ] Source integrity: all 13 entries have source Markdown, SVG, catalog
  coordinate, BOM, and source-note references recorded.
- [ ] Design and safety: the S01-01 destructive wording, S01-06 timing claim,
  and S02-01 BOM/text mismatch are resolved; supply, transistor, diode,
  touch/water, and speaker risks have owner-facing decisions.
- [ ] Capability readiness: every BOM item is mapped to an existing supported
  model, an accepted C01 model task, an explicitly accepted physical prop, or
  an out-of-scope disposition. No scripted electrical outcome substitutes for
  a missing model.
- [ ] Fixture correctness: every admitted scheme validates, derives topology
  from holes/pins/wires, uses stable IDs, has readable placement, and has no
  duplicate pin-hole occupancy.
- [ ] Electrical behavior: each admitted fixture has the applicable solver
  assertions, bounded-failure diagnostics, polarity/pin checks, and numerical
  or state checks. Oscillators have deterministic initial conditions.
- [ ] Automated evidence: exact commands and results are recorded in this
  card, including relevant property/regression tests and schema/reference
  checks.
- [ ] Manual buildability: representative assemblies cover each admitted
  component family and every unusual physical risk; the shared protocol's
  continuity, polarity, control-state, measurement, and visual evidence are
  recorded.
- [ ] Release decision: the 13 dispositions reconcile to the ledger, with
  ready/blocked/physical/out-of-scope counts and unresolved findings visible.

## Required verification

Run the repository baseline commands documented by the current workspace
README, plus the C01-specific checks below. These commands are planned for
C01 implementation; they were not run by C00.

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build -p bredboard-app --target x86_64-unknown-linux-gnu --locked
cargo build -p bredboard-app --target wasm32-unknown-unknown --locked
```

C01-specific evidence must include catalog/schema/reference integrity,
fixture validation and derived-topology checks, model/property/regression
tests, deterministic action/step behavior where applicable, and a manual
Linux interaction check for every admitted menu fixture. A successful build is
not interaction or manual-assembly evidence. Browser interaction is not part
of this card's acceptance.

## Section 01 implementation evidence

The current Section 01 slice adds six embedded fixtures under
`fixtures/projects/c01-s01-*.json`, registers them in the app menu, and adds
the smallest missing runtime contract for the 1N4007 reference: a smooth
two-pin `diode` with `anode/cathode`, bounded parameters, calculated current,
and a cathode-band sprite. The source 9 V labels are normalized to the
product's 5 V supply. The destructive S01-01 wording is replaced with a
bounded current observation, and the S01-06 timing claim is replaced with the
documented 470 uF / 470 ohm time constant.

Checks run for this slice:

```text
cargo test -p bredboard-core — passed
cargo test -p bredboard-app — passed
```

The Section 01 rows remain release-blocked until manual continuity, polarity,
measurement, and visual evidence is recorded. C01 is not `ready_for_fukit`.

The current Section 02 slice adds five fixtures under
`fixtures/projects/c01-s02-*.json`, selects the BC547 pin convention, and
removes the source's unlisted motor behavior from the player-facing contract.
S02-06 follows the five-NPN topology visible in its SVG; the three-NPN BOM
count is recorded as a source discrepancy rather than dropping stages. Its
calculated AND/OR/NOT truth table passes all four button combinations.
S02-03 and S02-04 now use explicit `touch_pad` and `water_probe` two-terminal
controlled-resistance contracts. Ratio 0 is the dry/open endpoint and ratio 1
is the documented contact/wet endpoint; the app exposes both controls as
dials, and the core tests compare calculated currents at both endpoints. The
physical pads, cup, and probes still require the battery-only manual protocol.
The diode and PNP runtime contracts are also present, but no S02-05 candidate
is admitted: the catalog's 47 kΩ/47 µF cross-coupled oscillator remains
nonconvergent at a later polarity transition under the fixed-step solver.
Only S02-05 and S02-07 remain blocked in this slice; touch and water are
fixture-ready with manual physical evidence pending.

Latest automated evidence for this slice:

```text
cargo fmt --all --check — passed
cargo clippy --workspace --all-targets --locked -- -D warnings — passed
cargo test --workspace --locked — passed (49 app, 109 core, 3 tools tests)
cargo build -p bredboard-app --target x86_64-unknown-linux-gnu --locked — passed
cargo build -p bredboard-app --target wasm32-unknown-unknown --locked — passed
cargo run -p bredboard-tools --locked -- validate-catalog breadboard-circuits/spec/catalog.json breadboard-circuits/spec/catalog.schema.json — passed; 20 sections, 212 schematics
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c01-s01-01-first-safe-light.json — passed; 3 components, 2 wires, 3 derived nodes
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c01-s01-02-button-and-switch.json — passed; 7 components, 3 wires, 7 derived nodes
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c01-s01-03-series-and-parallel.json — passed; 8 components, 8 wires, 5 derived nodes
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c01-s01-04-potentiometer-dimmer.json — passed; 4 components, 2 wires, 4 derived nodes
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c01-s01-05-reverse-polarity.json — passed; 4 components, 2 wires, 4 derived nodes
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c01-s01-06-smooth-fade.json — passed; 6 components, 3 wires, 5 derived nodes
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c01-s02-01-transistor-key.json — passed; 6 components, 3 wires, 6 derived nodes
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c01-s02-02-dusk-night-light.json — passed; 7 components, 6 wires, 5 derived nodes
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c01-s02-03-touch-button.json — passed; 7 components, 4 wires, 7 derived nodes
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c01-s02-04-water-sensor.json — passed; 5 components, 3 wires, 5 derived nodes
cargo run -p bredboard-tools --locked -- validate fixtures/projects/c01-s02-06-transistor-logic.json — passed; 20 components, 12 wires, 14 derived nodes
cargo test -p bredboard-core c01_transistor_logic_matches_all_four_input_combinations --locked — passed
cargo test -p bredboard-core c01_touch_and_water_inputs_follow_explicit_resistance_controls --locked — passed
cargo test -p bredboard-app --locked — passed (49 app tests)
cargo test -p bredboard-app e30_asymmetric_initial_state_sustains_multiple_alternating_led_cycles --locked — passed
```

The current solver remains bounded at 80 nonlinear iterations. A 2026-09-29
admission attempt for S02-05 reused the E30 topology and the source's 47 µF
capacitors: structural validation passed, but
`cargo run -p bredboard-tools --locked -- simulate
fixtures/projects/c01-s02-05-two-transistor-flasher.json 4000` stopped with
`nonconvergence` at step 428. The candidate fixture was not retained, and
lowering the capacitor or cross-coupling values was rejected as a source-model
substitution. The row remains `blocked_component` until the fixed-step
transient solve can sustain the calculated polarity transitions.

A separate 2026-09-29 S02-07 attempt followed the source's NPN/PNP/capacitor
feedback topology with the normalized 5 V supply, 100 kΩ potentiometer, 10 kΩ
resistor, 47 nF capacitor, and 8 Ω speaker. It converged for 10,000 fixed
steps, but the speaker current settled near 0.556 A (about 2.47 W) and the
capacitor did not sustain an output transition. The candidate was deleted;
the row remains `blocked_component` until a bounded speaker-output contract
and a calculated startup/oscillation result are available.

The current workspace baseline for the already admitted catalog surface is
`cargo test --workspace --locked` — 49 app tests, 109 core tests, 3 tool tests,
and 0 doc-test failures; C12's 12 fault-pair fixtures are covered separately
by its published repair-loop regression.

## Manual assembly checklist

Use [`MANUAL-ASSEMBLY-PROTOCOL.md`](../catalog/MANUAL-ASSEMBLY-PROTOCOL.md).
At minimum, record board/rail continuity, approved supply and current limit,
pin/polarity checks, no-short continuity, startup, every button/switch/control
state, measured readouts, deviations, and an overview plus close-up visual
record. S02-03 and S02-04 cannot be accepted without their physical-prop and
battery-only safety evidence.

## Known blockers at card creation

- The source catalog uses 9 V labels while the product presentation uses a
  5 V external source.
- The source alternates BC547/2N3904 and BC557/2N3906 without a single pin
  convention.
- The diode and PNP runtime models now have named pin contracts; S02-07 still
  lacks a bounded speaker-load and oscillator-startup fixture.
- A candidate S02-05 project with the source's 47 kΩ/47 µF values validated
  structurally but stopped with `nonconvergence` during a later polarity
  transition; it was deleted and is not counted as an implementation.
- S02-06 uses the five-transistor SVG topology; the three-transistor BOM count
  remains an explicit source discrepancy in the ledger and fixture contract.
- Touch pads and water probes now have explicit controlled-resistance fixture
  contracts; battery-only physical assembly, spill protection, and
  reproducible manual evidence remain open.
- S01-06's nominal RC time constant does not support its “couple seconds”
  wording without a documented observable threshold.
- S02-01 mentions a motor that is absent from its BOM.
- S02-07 needs an 8 Ω speaker power/current bound before manual operation.

## Codex Goal

```text
/goal Complete C01 in docs/tasks/C01-catalog-implementation.md after C00 is
accepted. Audit and implement only the 13 schemes in source sections 01 and
02 as calculated, validated, hole-accurate fixtures, resolving or recording
all source, safety, component, pin, sprite, fixture, and manual-assembly
findings. Preserve the current core/topology architecture and use explicit
actions and fixed steps. Do not start C02 or implement free assembly, lesson
scripting, migrations, .cir/ngspice, browser interaction acceptance, or Steam
work. Run every required check, record exact evidence, and leave blocked
schemes visible rather than inventing scripted outcomes. Do not commit or push.
```

## Completion and fukit handoff

Set `Status: ready_for_fukit` only after every acceptance gate and required
check has evidence in this card. A ready card is not a commit or publication
claim. Do not mark C01 ready merely because its source files parse or its SVGs
render.
