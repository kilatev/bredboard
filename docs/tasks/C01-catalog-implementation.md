# C01 — Level 1 and transistor reference fixtures

Status: pending; Section 01 fixture slice implemented, manual acceptance and
Sections 01/02 release decisions remain open.

## Dependencies

- [C00 — Catalog audit baseline](C00-catalog-audit.md) and its shared manual
  assembly protocol must be accepted.
- [C01 detailed readiness audit](../catalog/C01-READINESS-AUDIT.md) is the
  source of the per-scheme blockers and risks.
- Existing fixture architecture, stable board topology, and the current
  component contracts from the accepted MVP and post-MVP cards remain the
  boundary. This card does not authorize a second netlist or scripted result.
- New diode and PNP model/pin-contract tasks are prerequisites for S01-05 and
  S02-07. A physical-prop decision is a prerequisite for S02-03 and S02-04.

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

## Manual assembly checklist

Use [`MANUAL-ASSEMBLY-PROTOCOL.md`](../catalog/MANUAL-ASSEMBLY-PROTOCOL.md).
At minimum, record board/rail continuity, approved supply and current limit,
pin/polarity checks, no-short continuity, startup, every button/switch/control
state, measured readouts, deviations, and an overview plus close-up visual
record. S02-03 and S02-04 cannot be accepted without their physical-prop and
battery-only safety decisions.

## Known blockers at card creation

- The source catalog uses 9 V labels while the product presentation uses a
  5 V external source.
- The source alternates BC547/2N3904 and BC557/2N3906 without a single pin
  convention.
- No diode or PNP runtime model/pin contract is currently established for the
  C01 batch.
- Touch pads and water probes need a safe, reproducible physical-prop scope
  decision.
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
