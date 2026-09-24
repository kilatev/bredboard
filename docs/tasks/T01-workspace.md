# T01 — Runnable Rust and Bevy workspace

Status: pending

## Dependencies

[T00 — English project documentation and task cards](T00-project-docs.md). Its acceptance criteria must be satisfied before starting this task.

## Outcome and commit boundary

Create core, app, and tools crates with pinned compatible toolchain/dependencies and a minimal Bevy application that opens on Linux and in browsers.

Suggested commit title: `build: add a Linux and web Bevy workspace`.

The deliverable must stand on its own at this milestone. Unfinished successor tasks do not prevent this task from being accepted. Do not include unrelated refactors or publication steps.

## Scope and interfaces

Establish crate boundaries, a small core-to-app interface, Cargo.lock, and documented build/check commands. No circuit models or lesson behavior.

Follow the shared architecture, language, numerical, and persistence contracts in [the plan](../PLAN.md). Repository initialization, remote selection, and publication are not implicit parts of this task.

## Acceptance criteria

- [ ] Core can be built and tested without Bevy/windowing dependencies.
- [ ] The minimal app launches on Linux and in Chromium and Firefox through WASM.
- [ ] README records exact commands for formatting checks, Clippy, workspace tests, Linux builds, WASM builds, and serving the web artifact.
- [ ] Dependency origins/licenses are recorded; user-visible text is separate from application logic.

## Required verification

- Run the newly documented formatting, Clippy, test, Linux-build, and WASM-build commands.
- Launch the application on Linux and in Chromium and Firefox and record versions and observed result.
- Inspect the core dependency graph for forbidden presentation dependencies.

“Baseline checks” means the actual project commands established in T01 and documented in README, not invented commands or an assumed passing suite. Record commands and evidence below. Required browser/UI checks cannot be replaced by successful compilation. A missing or failing required check blocks readiness.

## Codex Goal

```text
/goal Complete T01 in docs/tasks/T01-workspace.md according to docs/PLAN.md
and AGENTS.md. Deliver this outcome: Create core, app, and tools crates with pinned compatible toolchain/dependencies and a minimal Bevy application that opens on Linux and in browsers.
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
