# Agent instructions

## Scope and language

- Read `docs/PLAN.md`, `docs/TASKS.md`, and the active task card before changing the project.
- Work on the explicitly requested task. Do not automatically start its successor or pursue the whole roadmap.
- Use English for project artifacts, code comments, UI text, lessons, diagnostics, schema descriptions, examples, and commit descriptions. Match the user's language in conversation.
- Preserve unrelated work. Changes must form a coherent task-scoped commit.
- Keep contributor identity repository-local in both Git and Jujutsu. Never modify global user.name or user.email for this project; global settings may belong to corporate work.

## Architecture

- The Rust core must not depend on Bevy, windowing, rendering, wall-clock time, or filesystem access.
- Core state is the authority for electrical behavior and lesson progress. ECS entities reference stable domain IDs; they do not own duplicate electrical state.
- Route changes through explicit actions. Advance simulation with fixed steps and stable ordering; never derive electrical results from frame rate or animation.
- Derive connectivity from board contacts, component pins, and wire endpoints. Do not maintain a second editable netlist.
- Preserve JSON and model version contracts. Validate imports atomically and reject unsupported versions clearly.
- Do not substitute scripted experiment outcomes for calculated electrical behavior.

## Verification

- This is currently documentation-only. Do not claim Cargo checks passed before a workspace exists.
- T01 establishes and documents exact commands for formatting checks, Clippy, workspace tests, Linux builds, and WASM builds.
- Later tasks use those commands and add the specialized checks in their cards. Record exact commands and results before marking a task ready.
- Add meaningful property tests with each feature; use reproducible seeds and retain minimized failing cases as regressions.
- Check changed UI behavior on supported platforms. A build is not evidence that a browser interaction works.
- An unavailable required check is a blocker, not a pass. Do not weaken criteria to obtain a green result.

## Goals and completion

- Create a Codex Goal only when explicitly requested. The task cards are reusable Goal text, not active goals.
- Record implementation state and evidence in the active card. `ready_for_fukit` means acceptance checks passed, not that commit or publication succeeded.
- Do not implement future tasks to satisfy the active task's completion criteria.
- Use the user's `fukit` skill when invoked. It only describes, commits, and pushes; it does not review or run checks.
- Skill reference on the owner's machine: `/home/vetalik/.codex/skills/fukit/SKILL.md`. This personal integration is optional for other contributors; the contributor guide describes review expectations.
- Do not initialize jj, select a remote/bookmark, commit, or push merely because these documents mention that workflow. Establish those prerequisites separately under user direction.
- Use jj history for committed status and actual push results for publication status; never pre-record a successful push in the commit it would publish.

## Open-source release

- Preserve the MIT License in `LICENSE`, selected by the owner in the initial GitHub commit. Do not change the license without user direction.
- Record third-party source, license, and required attribution when introducing dependencies or assets.
- Public release requires a selected project license and explicit publication destination/authorization.
