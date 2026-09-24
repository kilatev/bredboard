# T02 — Project JSON and breadboard connectivity

Status: ready_for_fukit

## Dependencies

[T01 — Runnable Rust and Bevy workspace](T01-workspace.md). Its acceptance criteria must be satisfied before starting this task.

## Outcome and commit boundary

Implement versioned Project data, a documented board contact model, the initial catalog metadata, JSON Schema generation, and a CLI project validator.

Suggested commit title: `feat: validate board projects and derive electrical connectivity`.

The deliverable must stand on its own at this milestone. Unfinished successor tasks do not prevent this task from being accepted. Do not include unrelated refactors or publication steps.

## Scope and interfaces

Define stable component/pin/hole IDs, placement and wire endpoints, structural diagnostics, and topology compilation. Represent the planned catalog without claiming unsupported models can simulate.

Follow the shared architecture, language, numerical, and persistence contracts in [the plan](../PLAN.md). Repository initialization, remote selection, and publication are not implicit parts of this task.

## Acceptance criteria

- [ ] Board contacts and power-rail continuity are documented and covered by fixtures.
- [ ] Wire crossings do not create connections; pin placement and endpoints do.
- [ ] Malformed references, duplicate IDs, unsupported versions, nonfinite/invalid parameters, and scope-limit violations are rejected clearly.
- [ ] Schema and English authoring examples agree with Rust serialization.
- [ ] Compiled connectivity is unchanged by JSON-array permutation or domain-ID renaming, modulo renamed labels.

## Required verification

- Run baseline project checks and focused structural/topology tests.
- Validate representative accepted and rejected JSON fixtures with both the schema and CLI.
- Run Proptest connectivity, serialization round-trip, renaming, and ordering properties; preserve any minimized failures.

“Baseline checks” means the actual project commands established in T01 and documented in README, not invented commands or an assumed passing suite. Record commands and evidence below. Required browser/UI checks cannot be replaced by successful compilation. A missing or failing required check blocks readiness.

## Codex Goal

```text
/goal Complete T02 in docs/tasks/T02-project-topology.md according to docs/PLAN.md
and AGENTS.md. Deliver this outcome: Implement versioned Project data, a documented board contact model, the initial catalog metadata, JSON Schema generation, and a CLI project validator.
Satisfy every acceptance criterion and run every required check in the card.
Fix task-scoped findings without weakening tests or acceptance criteria.
Do not implement successor tasks. Prepare one coherent change for review;
do not commit or push. Record exact verification evidence in this card.
If a required check is unavailable, report the exact blocker and the input
or environment change needed; do not count it as passed.
```

## Completion and fukit handoff

When all criteria pass, record evidence and set `Status: ready_for_fukit`. Stop without starting the next task. The user may then invoke `fukit` for task-scoped compliance review, code review, required checks, fixes, commit, and targeted push.

An existing jj repository and an unambiguous authorized remote/bookmark are required for that workflow. Do not initialize or guess them. Commit completion is evidenced by jj history; publication is evidenced by the actual push result, not a checkbox pre-written in this change.

## Evidence

Environment: Rust 1.95.0; no browser/UI behavior is in scope for this core/CLI task.

- `cargo fmt --all --check` — passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — passed.
- `cargo test --workspace --locked` — passed; eight core tests including deterministic Proptest cases, schema fixture validation, ordering, ID-renaming, round-trip, contacts, diagnostics, malformed Unicode IDs, limits, and rail continuity.
- `cargo build -p bredboard-app --target x86_64-unknown-linux-gnu --locked` — passed.
- `cargo build -p bredboard-app --target wasm32-unknown-unknown --locked` — passed.
- `cargo run -p bredboard-tools --locked -- schema` — passed; generated Draft 2020-12 schema (redirected to `/tmp/bredboard-project-schema.json`).
- `cargo run -p bredboard-tools --locked -- validate fixtures/projects/valid-resistor.json` — accepted, 1 component, 2 wires, 2 derived nodes.
- `cargo run -p bredboard-tools --locked -- validate fixtures/projects/invalid-version.json` — rejected with `unsupported_version`.
- `cargo run -p bredboard-tools --locked -- validate fixtures/projects/invalid-reference.json` — rejected with `invalid_hole`.
- Core unit tests additionally check schema rejection of a malformed object, duplicate IDs, component scope overflow, nonfinite parameters, strip separation, wire endpoint connectivity, and JSON-array order invariance. Proptest uses fixed seed `0xB8ED_B04D` and 64 cases; no minimized failures were produced.
- `cargo metadata --locked --format-version 1` — passed; regenerated `docs/dependency-licenses.csv` (576 registry packages, all with license expressions).
