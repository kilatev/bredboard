# T02 — Project JSON and breadboard connectivity

Status: pending

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

Not run yet. Record exact commands or manual procedures, results, environment/browser versions when relevant, and limitations here before marking the task ready.
