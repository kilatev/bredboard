# T14 — Verified MVP release artifacts

Status: pending

## Dependencies

[T13 — Automated native and WASM verification](T13-cross-platform-verification.md). Its acceptance criteria must be satisfied before starting this task.

## Outcome and commit boundary

Prepare reproducible Linux and web artifacts, final English user/contributor documentation, and the open-source release checklist.

Suggested commit title: `release: prepare verified Linux and web MVP artifacts`.

The deliverable must stand on its own at this milestone. Unfinished successor tasks do not prevent this task from being accepted. Do not include unrelated refactors or publication steps.

## Scope and interfaces

No public publishing without explicit destination and authorization. Preserve the owner-selected MIT License and review third-party obligations; changing the license requires user direction.

Follow the shared architecture, language, numerical, and persistence contracts in [the plan](../PLAN.md). Repository initialization, remote selection, and publication are not implicit parts of this task.

## Acceptance criteria

- [ ] All three lessons and file workflows pass on Linux, Chromium, and Firefox.
- [ ] Builds reproduce from pinned dependencies with documented steps and known limitations.
- [ ] The owner-selected MIT License is preserved in LICENSE and dependency/asset obligations and attributions are reviewed.
- [ ] All shipped UI, board labels, lessons, diagnostics, schema descriptions, and documentation are English.
- [ ] Required check evidence is recorded; missing platform access or unresolved third-party licensing obligations are reported as blockers.
- [ ] Artifacts are prepared locally; publication is not claimed unless separately authorized and verified.

## Required verification

- Run the complete documented checks and cross-platform runner against the release revision.
- Perform the full three-lesson and file-workflow acceptance checklist on all supported platforms.
- Build release artifacts from a clean checkout once a repository exists; verify packaging contents, documentation links, and license/attribution files.

“Baseline checks” means the actual project commands established in T01 and documented in README, not invented commands or an assumed passing suite. Record commands and evidence below. Required browser/UI checks cannot be replaced by successful compilation. A missing or failing required check blocks readiness.

## Codex Goal

```text
/goal Complete T14 in docs/tasks/T14-mvp-release.md according to docs/PLAN.md
and AGENTS.md. Deliver this outcome: Prepare reproducible Linux and web artifacts, final English user/contributor documentation, and the open-source release checklist.
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
