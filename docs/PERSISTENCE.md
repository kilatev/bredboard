# Snapshots and action logs

Snapshots use format version 1 and record model/solver versions, the current
project, the reset project, integer step, run state, capacitor voltages,
controls, diagnostics, and last valid readings. They never store compiled
topology. Restore rebuilds topology from the project and validates reading
labels against it before returning new state values.

Create and validate a resumable snapshot with the headless tools:

```sh
cargo run -p bredboard-tools --locked -- snapshot fixtures/projects/rc-charging.json 1000 /tmp/rc-snapshot.json
cargo run -p bredboard-tools --locked -- validate-snapshot /tmp/rc-snapshot.json
```

Generate the schema with `cargo run -p bredboard-tools --locked -- schema
snapshot`. Project, snapshot, and action-log schema types all have English
descriptions. Example files live in `fixtures/snapshots/`.

An action log includes a complete initial snapshot and ordered events. Each
event's `step` is the simulation boundary immediately before applying its
action; `final_step` describes how far to advance after the last event. The
action values use a tagged JSON form, for example `{ "action": "run" }` or
`{ "action": "set_parameter", "component": "R1", "name": "resistance",
"value": 2200.0 }`. The example at
`fixtures/action-logs/rc-charging-10-steps.json` starts a run at step 0 and
advances to step 10. Validate and replay it with:

```sh
cargo run -p bredboard-tools --locked -- schema action-log
cargo run -p bredboard-tools --locked -- replay fixtures/action-logs/rc-charging-10-steps.json
```

Restore and replay APIs return new values or an error. Callers only replace
their active project and simulation after successful validation.
