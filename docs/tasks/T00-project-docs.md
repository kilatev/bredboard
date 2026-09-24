# T00 — English project documentation and task cards

Status: ready_for_fukit

## Dependencies

None.

## Outcome and commit boundary

Create the README, contributor and agent instructions, full technical plan, task index, fifteen task cards, and post-MVP roadmap in English.

Suggested commit title: `docs: define the bredboard MVP and verifiable task sequence`.

The deliverable must stand on its own at this milestone. Unfinished successor tasks do not prevent this task from being accepted. Do not include unrelated refactors or publication steps.

## Scope and interfaces

Markdown documents and relative links only; no application, repository initialization, remote setup, license selection, or publication.

Follow the shared architecture, language, numerical, and persistence contracts in [the plan](../PLAN.md). Repository initialization, remote selection, and publication are not implicit parts of this task.

## Acceptance criteria

- [x] The agreed scope, numerical baseline, language policy, and verification tolerances are preserved.
- [x] Every task has dependencies, boundaries, acceptance criteria, checks, and a reusable Goal.
- [x] All relative file links resolve; the index lists exactly T00 through T14.
- [x] The current license status is represented accurately; commit/push prerequisites are explicit.

## Required verification

- Inspect all documents against the agreed plan.
- Check local Markdown file links, task IDs, dependency order, and required card sections.
- Confirm all project prose is English and this documentation change introduces no application implementation or tracked repository metadata. Separately authorized local repository setup is outside T00.

“Baseline checks” means the actual project commands established in T01 and documented in README, not invented commands or an assumed passing suite. Record commands and evidence below. Required browser/UI checks cannot be replaced by successful compilation. A missing or failing required check blocks readiness.

## Codex Goal

```text
/goal Complete T00 in docs/tasks/T00-project-docs.md according to docs/PLAN.md
and AGENTS.md. Deliver this outcome: Create the README, contributor and agent instructions, full technical plan, task index, fifteen task cards, and post-MVP roadmap in English.
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

Verified on 2026-09-24 during documentation preparation.

- Ran an inline Python 3 documentation audit from the workspace: enumerated all Markdown files; resolved each relative Markdown file link; checked T00–T14 IDs, predecessor dependencies, task index entries, per-card Goal paths, all eight required sections, balanced code fences, final newlines, and absence of Cyrillic text. Result: 21 documents, 15 task cards, 55 valid local links, no failures.
- Reviewed the plan against the agreed product decisions: Rust/Bevy, independent core, Linux/web, three lessons, arbitrary supported JSON assemblies, numerical baseline/tolerances, English artifacts, and post-MVP phases are preserved.
- Reviewed every task boundary for a coherent milestone and explicit verification. No card requires its successor to be implemented. Checked the workflow against the owner's fukit skill; repository initialization and destination selection remain separate prerequisites.
- Checked the project root: no Cargo.toml, .git, or .jj was created. This delivery contains documentation only.
- No application tests or builds were run because no application or workspace exists yet. No commit or push was attempted. At that preparation stage, the license, jj repository, and remote/bookmark were unset; this status indicates documentation acceptance, not a completed fukit workflow.

### fukit review — 2026-09-24

- The owner subsequently authorized repository setup and supplied origin. Fetched its main branch and preserved the initial MIT License commit as the parent of this documentation change. No published history was rewritten.
- Reviewed all 21 Markdown documents against T00 and the subsequent setup request. Updated stale license and repository statements to match the existing MIT License and origin. T01–T14 remain pending.
- Rechecked relative links, task IDs/dependencies, Goal paths, required sections, English prose, whitespace, and absence of implementation files. No application checks apply to this documentation-only milestone.
- Verified repository-local Git and Jujutsu identity; corporate global Git settings remain unchanged. The intended publication target is main at origin.
- Commit and publication outcomes must be verified from history and push results after this review; they are not pre-recorded here.
