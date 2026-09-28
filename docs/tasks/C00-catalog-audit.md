# C00 — Catalog audit baseline and shared assembly protocol

Status: ready_for_fukit

## Dependencies

The source corpus and the current product/architecture documents in
`docs/PLAN.md`, `docs/TASKS.md`, `docs/roadmap/CIRCUIT-CATALOG.md`, and
`docs/roadmap/CATALOG-EXECUTION-PLAN.md`. No application implementation task
is a dependency of this documentation baseline.

## Outcome and commit boundary

Establish one auditable baseline for all 212 reference schemes and define the
shared manual breadboard protocol needed before implementation batches. Create
the bounded C01 card, but do not implement C01.

The deliverables are:

- [`CATALOG-AUDIT-LEDGER.md`](../catalog/CATALOG-AUDIT-LEDGER.md), with one
  stable row per source entry and explicit disposition, readiness, findings,
  implementation evidence, and manual evidence fields;
- [`CATALOG-SECTION-TRIAGE.md`](../catalog/CATALOG-SECTION-TRIAGE.md), with
  coarse findings for all 20 sections;
- [`C01-READINESS-AUDIT.md`](../catalog/C01-READINESS-AUDIT.md), with detailed
  pin/model/sprite/fixture/manual risks for all 13 C01 schemes;
- [`MANUAL-ASSEMBLY-PROTOCOL.md`](../catalog/MANUAL-ASSEMBLY-PROTOCOL.md),
  defining the shared board, pin, safety, assembly, and evidence rules; and
- [`C01-catalog-implementation.md`](C01-catalog-implementation.md), the next
  bounded implementation card in `pending` state.

No circuit, fixture, menu entry, solver change, component kind, sprite, or UI
change belongs in this card.

## Acceptance criteria

- [x] `catalog.json` satisfies every constraint expressed by
  `catalog.schema.json`; the validation method and the unavailable generic
  validator limitation are recorded below.
- [x] The corpus reconciles to 20 sections and 212 entries, with 212 existing
  SVG references and matching Markdown headings, SVG paths, and per-circuit
  BOM rows.
- [x] The ledger contains all 212 entries with stable keys and all required
  audit/evidence fields.
- [x] All 20 sections have coarse design, safety, capability, and buildability
  triage with explicit disposition counts.
- [x] All 13 C01 entries have detailed mapping, pin ambiguity, model, sprite,
  fixture, and manual-assembly findings.
- [x] The shared manual assembly and evidence protocol is defined.
- [x] C01 is bounded by its own card and no C01 implementation was started.
- [x] English is used for new repository-authored text; source-language names
  remain only as source coordinates and preserved catalog values.

## Required verification

This is a documentation-only foundation task. Cargo formatting, Clippy,
workspace tests, Linux builds, WASM builds, and browser interaction are not
acceptance checks for C00 and were not claimed as passed.

The source and document checks are:

```text
python3 - <<'PY'
import json, re
from pathlib import Path
root = Path("breadboard-circuits/spec")
data = json.loads((root / "catalog.json").read_text())
schema = json.loads((root / "catalog.schema.json").read_text())
assert schema["$schema"] == "https://json-schema.org/draft/2020-12/schema"
assert isinstance(data, list) and len(data) >= 1
for section in data:
    assert set(section) == {"file", "title", "description", "circuits"}
    assert re.fullmatch(r"[0-9]{2}-[a-z0-9-]+\.md", section["file"])
    assert section["title"] and section["description"] and section["circuits"]
    for circuit in section["circuits"]:
        assert set(circuit) == {"name", "level", "final", "description", "in_game", "note", "schematic", "bom"}
        assert circuit["name"] and circuit["description"] and circuit["in_game"]
        assert circuit["level"] is None or (type(circuit["level"]) is int and 1 <= circuit["level"] <= 5)
        assert type(circuit["final"]) is bool
        assert circuit["note"] is None or type(circuit["note"]) is str
        assert re.fullmatch(r"svg/[0-9]{2}-[a-z0-9-]+/[0-9]+\.svg", circuit["schematic"])
        assert isinstance(circuit["bom"], list)
        for item in circuit["bom"]:
            assert len(item) == 2 and type(item[0]) is str and item[0]
            assert type(item[1]) is int and item[1] >= 1
print("schema-semantic validation: passed")
PY
```

Result: `schema-semantic validation: passed`. This dependency-free command
mirrors the Draft 2020-12 constraints in the repository schema, including
`prefixItems` tuple validation. `fastjsonschema` was also checked but its
installed API treated the Draft 2020 `items: false` tuple tail as a Draft-07
array rule and rejected valid BOM pairs; it was not used as evidence. A
generic AJV attempt was blocked by unavailable registry DNS (`EAI_AGAIN`), not
counted as a pass, and did not modify the repository.

```text
python3 - <<'PY'
import json, re
from pathlib import Path
root = Path("breadboard-circuits/spec")
data = json.loads((root / "catalog.json").read_text())
assert len(data) == 20
assert sum(len(s["circuits"]) for s in data) == 212
for s in data:
    assert (root / s["file"]).is_file()
    md = (root / s["file"]).read_text()
    names = re.findall(r"^###\s+\d+\.\s+(.+?)\s*$", md, re.M)
    images = re.findall(r"!\[[^]]*\]\(([^)]+)\)", md)
    assert names == [c["name"] for c in s["circuits"]]
    assert images == [c["schematic"] for c in s["circuits"]]
    assert all((root / c["schematic"]).is_file() for c in s["circuits"])
print("catalog reconciliation: 20 sections, 212 entries, 212 SVGs; Markdown headings/images match")
PY
```

Result: `catalog reconciliation: 20 sections, 212 entries, 212 SVGs;
Markdown headings/images match`. A second source check compared all Markdown
BOM tables with their JSON arrays; all 20 sections matched.

```text
mkdir -p /tmp/bredboard-c01-svg
for f in breadboard-circuits/spec/svg/01-level-1-first-current/*.svg breadboard-circuits/spec/svg/02-level-2-transistors/*.svg; do rsvg-convert -o /tmp/bredboard-c01-svg/$(basename "$f" .svg).png "$f"; done
```

Result: 13 SVGs rendered and were reviewed together with their extracted
labels. No rendering or reference failure was found. The rendered images are
temporary review evidence, not repository assets.

## Evidence

- The full ledger reports all 212 keys and reconciles its disposition counts:
  `blocked_component=129`, `blocked_design=3`, `physical_scope=68`, and
  `out_of_scope=12`.
- C01 has 13 explicit rows in the detailed audit, including the source
  voltage mismatch, NPN/PNP pin ambiguity, diode gap, touch/water physical
  scope, RC timing finding, and speaker power risk.
- No Cargo command, app run, browser check, real-board assembly, commit, or
  push was performed as part of this documentation-only task.

## Codex Goal

```text
/goal Establish an auditable baseline for all 212 source schemes and prepare
the first implementation batch, C01, for execution. Validate catalog.json
against catalog.schema.json; reconcile all 20 sections and SVG references;
create the 212-entry ledger; triage all sections; audit all 13 C01 schemes;
define the shared manual breadboard protocol; and create the bounded C01 task
card. Do not implement circuits, fixtures, menu entries, solver changes, new
component kinds, sprites, or UI changes. Do not start C01 implementation or
deferred free-assembly, lesson, migration, .cir/ngspice, browser, or Steam
work. Use English for new repository artifacts, preserve unrelated changes,
and do not claim Cargo checks passed.
```

## Completion and fukit handoff

The baseline is ready for review. `ready_for_fukit` means the documentation
acceptance checks above passed; it does not mean the work was committed or
published. C01 remains `pending` and must be requested separately.
