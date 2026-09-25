# Task index and Codex Goal workflow

Each task below is one coherent, independently verifiable change. Execute them in order. Their dependencies refer to accepted behavior; `fukit` handles the separate commit/push workflow when explicitly invoked.

No Codex Goal is currently active merely because these files exist. Copy a card's Goal only when requesting that task. Do not use a single unbounded Goal to implement the entire roadmap.

## MVP closure

**MVP complete.** The MVP release boundary is T14. Its acceptance criteria and all accepted prerequisite cards are complete. T15–T17 are already accepted follow-up refinements; T18 and every later card are post-MVP work. Do not revisit MVP planning or add MVP requirements while executing post-MVP tasks unless the owner explicitly requests a bug fix.

## Task history through the MVP release

| Task | Outcome | Depends on |
| --- | --- | --- |
| [T00](tasks/T00-project-docs.md) | English project documentation and task cards | — |
| [T01](tasks/T01-workspace.md) | Runnable Rust and Bevy workspace | T00 |
| [T02](tasks/T02-project-topology.md) | Project JSON and breadboard connectivity | T01 |
| [T03](tasks/T03-dc-solver.md) | Resistive DC circuit simulation | T02 |
| [T04](tasks/T04-rc-actions.md) | RC simulation and deterministic actions | T03 |
| [T05](tasks/T05-snapshots-replay.md) | Simulation snapshots and action replay | T04 |
| [T06](tasks/T06-rc-bench.md) | Buildable RC bench and shared app controls | T05 |
| [T07](tasks/T07-led-model.md) | Calculated LED circuit and current-driven light | T06 |
| [T08](tasks/T08-npn-model.md) | Calculated transistor switch circuit | T07 |
| [T09](tasks/T09-board-instruments.md) | Three-circuit menu and hole-accurate board | T08 |
| [T15](tasks/T15-capacitor-npn-ranges.md) | Breadboard-scale capacitor and transistor ranges | T09 |
| [T16](tasks/T16-component-sprites.md) | 8-bit resistor, LED, and button sprites | T15 |
| [T17](tasks/T17-more-component-sprites.md) | Sprites for remaining kinds and designs for future simple parts | T16 |
| [T13](tasks/T13-cross-platform-verification.md) | Integrated Linux and core verification | T15 |
| [T14](tasks/T14-mvp-release.md) | Verified Linux MVP release artifact | T13 |

## Post-MVP presentation tasks

| Task | Outcome | Depends on |
| --- | --- | --- |
| [T18](tasks/T18-readable-layout.md) | Deterministic automatic breadboard layout optimized for readability | T14, T17 |

The former [T10 file workflow](tasks/T10-file-workflow.md), [T11 LED lesson](tasks/T11-led-lesson.md), and [T12 remaining lessons](tasks/T12-remaining-lessons.md) are deferred to the [post-MVP roadmap](roadmap/POST-MVP.md). Their IDs remain reserved to keep historical references unambiguous.

## Status and evidence

Task cards are the source of implementation status: `pending`, `in_progress`, or `ready_for_fukit`. Keep evidence in the card. Record blockers there without treating missing checks as passes. Task status is a documentation convention, not a request to change a Codex Goal's lifecycle automatically.

Commit and push status are verified separately from jj history and actual publication results. Do not claim `ready_for_fukit` means a task is committed or pushed. Before the first finish workflow, establish a jj repository and explicitly choose its remote/bookmark in a separate authorized action.

## Goal contract

Every card bounds the work to its own outcome and verification surface. Fix failures within that boundary; report external prerequisites concretely. Do not silently expand scope, weaken checks, or start the next card. No token budget is assumed.

The [Codex Goals guide](https://developers.openai.com/cookbook/examples/codex/using_goals_in_codex) describes outcome-based persistent objectives. These task cards use that approach without requiring a custom Goal file format.

## fukit compatibility

After a card is ready, the owner can invoke `fukit`. It only describes, commits, and pushes; review and checks happen before it.

The owner's skill is located at `/home/vetalik/.codex/skills/fukit/SKILL.md`; read its current instructions when invoking it. Other contributors can follow [the contributor guide](../CONTRIBUTING.md) without installing that personal skill.

## Checkpoints and future work

T06 proves the RC vertical slice in the shared app, with Linux interaction acceptance. T07 and T08 add the calculated LED and transistor examples. T09 completes the menu and board presentation. T15 narrows capacitor and transistor ranges to breadboard scale. T13 integrates the checks, and T14 prepares the Linux release artifact. All three circuits' UI systems are shared across Linux and WASM; browser interaction is not tested for MVP acceptance. T14 does not itself authorize public publication.

[Post-MVP phases](roadmap/POST-MVP.md) are intentionally not executable MVP cards. Specify and decompose each phase before implementation.
