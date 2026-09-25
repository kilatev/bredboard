# T14 — Verified MVP release artifacts

Status: ready_for_fukit

## Dependencies

[T13 — Integrated Linux and core verification](T13-cross-platform-verification.md). Its acceptance criteria must be satisfied before starting this task.

## Outcome and commit boundary

Prepare a reproducible Linux artifact, final English user/contributor documentation, and the open-source release checklist for the three fixed circuits.

Suggested commit title: `release: prepare verified Linux MVP artifact`.

The deliverable must stand on its own at this milestone. Unfinished successor tasks do not prevent this task from being accepted. Do not include unrelated refactors or publication steps.

## Scope and interfaces

No public publishing without explicit destination and authorization. Preserve the owner-selected MIT License and review third-party obligations; changing the license requires user direction.

Follow the shared architecture, language, numerical, and persistence contracts in [the plan](../PLAN.md). Repository initialization, remote selection, and publication are not implicit parts of this task.

## Acceptance criteria

- [ ] All three circuits, their controls, and hole-hover identification pass in the Linux executable.
- [ ] Builds reproduce from pinned dependencies with documented steps and known limitations.
- [ ] The owner-selected MIT License is preserved in LICENSE and dependency/asset obligations and attributions are reviewed.
- [ ] All shipped UI, board labels, diagnostics, schema descriptions, and documentation are English.
- [ ] Required check evidence is recorded; missing platform access or unresolved third-party licensing obligations are reported as blockers.
- [ ] Artifacts are prepared locally; publication is not claimed unless separately authorized and verified.

## Required verification

- Run the complete documented checks and native replay runner against the release revision.
- Perform the three-circuit menu, board, control, and readout acceptance checklist on Linux. Do not run browser tests.
- Build release artifacts from a clean checkout once a repository exists; verify packaging contents, documentation links, and license/attribution files.

“Baseline checks” means the actual project commands established in T01 and documented in README, not invented commands or an assumed passing suite. Record commands and evidence below. Required Linux UI checks cannot be replaced by successful compilation. A missing or failing required check blocks readiness.

## Codex Goal

```text
/goal Complete T14 in docs/tasks/T14-mvp-release.md according to docs/PLAN.md
and AGENTS.md. Deliver a reproducible Linux artifact for the three-circuit MVP, with a compiling WASM target and no browser interaction testing.
Satisfy every acceptance criterion and run every required check in the card.
Fix task-scoped findings without weakening tests or acceptance criteria.
Do not implement successor tasks. Prepare one coherent change for review;
do not commit or push. Record exact verification evidence in this card.
If a required check is unavailable, report the exact blocker and the input
or environment change needed; do not count it as passed.
```

## Completion and fukit handoff

When all criteria pass, record evidence and set `Status: ready_for_fukit`. Stop without starting the next task. The user may then invoke `fukit` to describe, commit, and push.

An existing jj repository and an unambiguous authorized remote/bookmark are required for that workflow. Do not initialize or guess them. Commit completion is evidenced by jj history; publication is evidenced by the actual push result, not a checkbox pre-written in this change.

## Evidence

Owner-reported verification on 2026-09-25: the T14 release and Linux acceptance checks were tested manually and accepted. Detailed command, artifact, and environment evidence was not recorded in this card.
