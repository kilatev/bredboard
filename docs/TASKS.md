# Task index and Codex Goal workflow

Each task below is one coherent, independently verifiable change. Execute them in order. Their dependencies refer to accepted behavior; `fukit` handles the separate commit/push workflow when explicitly invoked.

No Codex Goal is currently active merely because these files exist. Copy a card's Goal only when requesting that task. Do not use a single unbounded Goal to implement the entire roadmap.

## MVP tasks

| Task | Outcome | Depends on |
| --- | --- | --- |
| [T00](tasks/T00-project-docs.md) | English project documentation and task cards | — |
| [T01](tasks/T01-workspace.md) | Runnable Rust and Bevy workspace | T00 |
| [T02](tasks/T02-project-topology.md) | Project JSON and breadboard connectivity | T01 |
| [T03](tasks/T03-dc-solver.md) | Resistive DC circuit simulation | T02 |
| [T04](tasks/T04-rc-actions.md) | RC simulation and deterministic actions | T03 |
| [T05](tasks/T05-snapshots-replay.md) | Simulation snapshots and action replay | T04 |
| [T06](tasks/T06-rc-bench.md) | End-to-end RC bench on Linux and web | T05 |
| [T07](tasks/T07-led-model.md) | LED simulation and overload warnings | T06 |
| [T08](tasks/T08-npn-model.md) | NPN simulation and transistor bench | T07 |
| [T09](tasks/T09-board-instruments.md) | Educational board presentation and instruments | T08 |
| [T10](tasks/T10-file-workflow.md) | Complete project import and export experience | T09 |
| [T11](tasks/T11-led-lesson.md) | Lesson engine and guided LED experiment | T10 |
| [T12](tasks/T12-remaining-lessons.md) | Guided RC and transistor experiments | T11 |
| [T13](tasks/T13-cross-platform-verification.md) | Automated native and WASM verification | T12 |
| [T14](tasks/T14-mvp-release.md) | Verified MVP release artifacts | T13 |

## Status and evidence

Task cards are the source of implementation status: `pending`, `in_progress`, or `ready_for_fukit`. Keep evidence in the card. Record blockers there without treating missing checks as passes. Task status is a documentation convention, not a request to change a Codex Goal's lifecycle automatically.

Commit and push status are verified separately from jj history and actual publication results. Do not claim `ready_for_fukit` means a task is committed or pushed. Before the first finish workflow, establish a jj repository and explicitly choose its remote/bookmark in a separate authorized action.

## Goal contract

Every card bounds the work to its own outcome and verification surface. Fix failures within that boundary; report external prerequisites concretely. Do not silently expand scope, weaken checks, or start the next card. No token budget is assumed.

The [Codex Goals guide](https://developers.openai.com/cookbook/examples/codex/using_goals_in_codex) describes outcome-based persistent objectives. These task cards use that approach without requiring a custom Goal file format.

## fukit compatibility

After a card is ready, the owner can invoke `fukit`. Review only that milestone's requirements, run the commands applicable at that point, fix scoped findings, describe the final behavior change in English, and commit/push only after checks pass and the destination is established.

The owner's skill is located at `/home/vetalik/.codex/skills/fukit/SKILL.md`; read its current instructions when invoking it. Other contributors can follow [the contributor guide](../CONTRIBUTING.md) without installing that personal skill.

## Checkpoints and future work

T06 proves the RC vertical slice on Linux and web. Resolve feasibility failures before adding nonlinear models. Property tests are introduced with each feature; T13 integrates and expands them. T14 prepares release artifacts and verifies the existing MIT License and third-party attribution, but does not itself authorize public publication.

[Post-MVP phases](roadmap/POST-MVP.md) are intentionally not executable MVP cards. Specify and decompose each phase before implementation.
