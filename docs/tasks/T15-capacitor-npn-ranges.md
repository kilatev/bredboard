# T15 — Breadboard-scale capacitor and transistor ranges

Status: pending

## Dependencies

[T09 — Three-circuit menu and hole-accurate board](T09-board-instruments.md), including the [T07 range amendment](T07-led-model.md#amendment-2026-09-25--breadboard-scale-parameter-ranges). Their acceptance criteria must be satisfied before starting this task.

## Outcome and commit boundary

Narrow the supported capacitor and NPN parameter ranges to parts found in hobby breadboard kits, and guarantee that valid circuits built from them solve within the documented bounds.

Suggested commit title: `fix: limit capacitor and transistor parameters to breadboard scale`.

The deliverable must stand on its own at this milestone. Unfinished successor tasks do not prevent this task from being accepted. Do not include unrelated refactors or publication steps.

## Scope and interfaces

The current ranges admit values with no breadboard meaning. A 1000 F capacitor stamps a 1e7 S companion conductance at the 100 µs step. A `saturation_current` of 1 A gives a negative base-emitter threshold (`0.026*ln(0.001/Is)` ≈ -0.18 V), so the transistor conducts with no base drive. Change only `parameter_range` in `crates/core/src/lib.rs`, the range table in `docs/PROJECT-FORMAT.md`, and the tests that cover them. Do not change the model equations, the 80-iteration bound, or the ranges settled by the T07 amendment.

Follow the shared architecture, language, numerical, and persistence contracts in [the plan](../PLAN.md). Repository initialization, remote selection, and publication are not implicit parts of this task.

## Acceptance criteria

- [ ] Supported ranges are: `capacitor.capacitance` 1e-10 to 1e-2 F (100 pF to 10 mF); `npn_transistor.beta` 10 to 1000; `npn_transistor.saturation_current` 1e-16 to 1e-12 A (base-emitter threshold of about 0.78 V down to 0.54 V). `docs/PROJECT-FORMAT.md` documents each range and the resulting NPN threshold span.
- [ ] Out-of-range values are rejected by validation with the existing clear parameter diagnostic, before solving. All built-in fixtures and committed snapshots remain valid without edits.
- [ ] A fixed-seed property test samples RC assemblies across the full capacitance range with T07-amendment resistor and voltage ranges. Every step solves, and the capacitor voltage stays between the initial voltage and the source voltage.
- [ ] A fixed-seed property test samples transistor-switch assemblies across the full beta and saturation-current ranges. Every case converges within 80 iterations, base current is under 1 µA with the button released, and the load current is higher with the button pressed than released.
- [ ] No project format version bump: the format is unpublished and all committed fixtures stay valid. Record this decision in `docs/PROJECT-FORMAT.md`.

## Required verification

- Run baseline checks and the new range-validation and property tests.
- Validate all three built-in fixtures with the tools `validate` command.
- Confirm in the Linux executable that the three circuits still behave as before. Do not test browser interaction.

“Baseline checks” means the actual project commands established in T01 and documented in README, not invented commands or an assumed passing suite. Record commands and evidence below. Required Linux UI checks cannot be replaced by successful compilation. A missing or failing required check blocks readiness.

## Codex Goal

```text
/goal Complete T15 in docs/tasks/T15-capacitor-npn-ranges.md according to docs/PLAN.md
and AGENTS.md. Narrow capacitor and transistor parameter ranges to breadboard scale.
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

None yet.
