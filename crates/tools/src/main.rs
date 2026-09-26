use bredboard_core::{
    Action, ActionLog, Contact, ControlState, Project, SimulationState, Snapshot, advance_steps,
    apply_actions, compile_topology, replay_action_log, restore_snapshot, solve_dc,
};
use schemars::{JsonSchema, SchemaGenerator, generate::SchemaSettings};
use std::{env, fs, process};

mod native_verify;

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("schema") => {
            let schema = match args.next().as_deref().unwrap_or("project") {
                "project" => schema_for::<Project>(),
                "snapshot" => schema_for::<Snapshot>(),
                "action-log" => schema_for::<ActionLog>(),
                _ => return Err("schema type must be project, snapshot, or action-log".into()),
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&schema).map_err(|e| e.to_string())?
            );
        }
        Some("validate") => {
            let path = args
                .next()
                .ok_or("usage: bredboard-tools validate <project.json>")?;
            let text = fs::read_to_string(&path).map_err(|e| format!("{path}: {e}"))?;
            let value: serde_json::Value =
                serde_json::from_str(&text).map_err(|e| format!("invalid JSON: {e}"))?;
            let schema = SchemaGenerator::new(SchemaSettings::draft2020_12())
                .into_root_schema_for::<Project>();
            let schema_value = serde_json::to_value(schema).map_err(|e| e.to_string())?;
            let compiled = jsonschema::validator_for(&schema_value)
                .map_err(|e| format!("cannot construct project schema: {e}"))?;
            let errors = compiled.iter_errors(&value).collect::<Vec<_>>();
            if !errors.is_empty() {
                for error in errors {
                    eprintln!("schema: {}", error);
                }
                return Err("project does not match the schema".into());
            }
            let project: Project =
                serde_json::from_value(value).map_err(|e| format!("invalid project: {e}"))?;
            match compile_topology(&project) {
                Ok(nodes) => println!(
                    "valid project: {} components, {} wires, {} derived nodes",
                    project.components.len(),
                    project.wires.len(),
                    nodes.len()
                ),
                Err(errors) => {
                    for e in errors {
                        eprintln!("{} at {}: {}", e.code, e.path, e.message);
                    }
                    return Err("project validation failed".into());
                }
            }
        }
        Some("solve") => {
            let path = args
                .next()
                .ok_or("usage: bredboard-tools solve <project.json>")?;
            let text = fs::read_to_string(&path).map_err(|e| format!("{path}: {e}"))?;
            let project: Project =
                serde_json::from_str(&text).map_err(|e| format!("invalid project JSON: {e}"))?;
            let states =
                std::collections::BTreeMap::<bredboard_core::ComponentId, ControlState>::new();
            let ratios = std::collections::BTreeMap::<bredboard_core::ComponentId, f64>::new();
            match solve_dc(&project, &states, &ratios) {
                Ok(result) => {
                    for node in result.node_voltages {
                        let labels = node
                            .contacts
                            .iter()
                            .filter_map(|contact| match contact {
                                Contact::ComponentPin(id, pin) => {
                                    Some(format!("{}.{}", id.0, pin.0))
                                }
                                Contact::Hole(_) => None,
                            })
                            .collect::<Vec<_>>()
                            .join(", ");
                        println!("node [{labels}]: {:.9} V", node.voltage);
                    }
                    for (id, current) in result.resistor_currents {
                        println!("resistor {}: {current:.9} A", id.0);
                    }
                    for (id, current) in result.source_currents {
                        println!("source {}: {current:.9} A", id.0);
                    }
                    for (id, current) in result.switch_currents {
                        println!("switch {}: {current:.9} A", id.0);
                    }
                    for (id, current) in result.led_currents {
                        println!("LED {}: {current:.9} A", id.0);
                    }
                    for (id, current) in result.transistor_collector_currents {
                        println!("transistor {} collector: {current:.9} A", id.0);
                    }
                }
                Err(bredboard_core::ElectricalError::Structure(errors)) => {
                    for e in errors {
                        eprintln!("{} at {}: {}", e.code, e.path, e.message);
                    }
                    return Err("project validation failed".into());
                }
                Err(bredboard_core::ElectricalError::Calculation(e)) => {
                    eprintln!("{}: {}", e.code, e.message);
                    return Err("DC solve failed".into());
                }
            }
        }
        Some("simulate") => {
            let path = args
                .next()
                .ok_or("usage: bredboard-tools simulate <project.json> <steps>")?;
            let steps: u64 = args
                .next()
                .ok_or("usage: bredboard-tools simulate <project.json> <steps>")?
                .parse()
                .map_err(|_| "steps must be a nonnegative integer")?;
            let text = fs::read_to_string(&path).map_err(|e| format!("{path}: {e}"))?;
            let mut project: Project =
                serde_json::from_str(&text).map_err(|e| format!("invalid project JSON: {e}"))?;
            let initial = project.clone();
            let mut state = SimulationState::new(&project);
            apply_actions(&mut project, &initial, &mut state, &[Action::Run]);
            advance_steps(&project, &mut state, steps);
            println!("step {} at {:.6} s", state.step, state.time_seconds());
            for (id, voltage) in &state.capacitor_voltages {
                println!("capacitor {}: {voltage:.9} V", id.0);
            }
            if state.stale {
                for d in state.diagnostics {
                    eprintln!("{} at {}: {}", d.code, d.path, d.message);
                }
                return Err("simulation stopped on calculation failure".into());
            }
        }
        Some("snapshot") => {
            let project_path = args
                .next()
                .ok_or("usage: bredboard-tools snapshot <project.json> <steps> <output.json>")?;
            let steps: u64 = args
                .next()
                .ok_or("usage: bredboard-tools snapshot <project.json> <steps> <output.json>")?
                .parse()
                .map_err(|_| "steps must be a nonnegative integer")?;
            let output_path = args
                .next()
                .ok_or("usage: bredboard-tools snapshot <project.json> <steps> <output.json>")?;
            let text =
                fs::read_to_string(&project_path).map_err(|e| format!("{project_path}: {e}"))?;
            let mut project: Project =
                serde_json::from_str(&text).map_err(|e| format!("invalid project JSON: {e}"))?;
            let reset = project.clone();
            let mut state = SimulationState::new(&project);
            apply_actions(&mut project, &reset, &mut state, &[Action::Run]);
            advance_steps(&project, &mut state, steps);
            if state.stale {
                return Err(state
                    .diagnostics
                    .iter()
                    .map(|d| format!("{}: {}", d.code, d.message))
                    .collect::<Vec<_>>()
                    .join("; "));
            }
            let snapshot = Snapshot::capture(&project, &reset, &state);
            fs::write(
                &output_path,
                serde_json::to_vec_pretty(&snapshot).map_err(|e| e.to_string())?,
            )
            .map_err(|e| format!("{output_path}: {e}"))?;
            println!("saved snapshot at step {} to {output_path}", state.step);
        }
        Some("validate-snapshot") => {
            let path = args
                .next()
                .ok_or("usage: bredboard-tools validate-snapshot <snapshot.json>")?;
            let value = read_value(&path)?;
            validate_schema::<Snapshot>(&value)?;
            let snapshot: Snapshot =
                serde_json::from_value(value).map_err(|e| format!("invalid snapshot: {e}"))?;
            match restore_snapshot(&snapshot) {
                Ok((_, _, state)) => println!("valid snapshot at step {}", state.step),
                Err(e) => return Err(format!("{}: {}", e.code, e.message)),
            }
        }
        Some("replay") => {
            let path = args
                .next()
                .ok_or("usage: bredboard-tools replay <action-log.json>")?;
            let value = read_value(&path)?;
            validate_schema::<ActionLog>(&value)?;
            let log: ActionLog =
                serde_json::from_value(value).map_err(|e| format!("invalid action log: {e}"))?;
            match replay_action_log(&log) {
                Ok((_, _, state)) => println!(
                    "replayed through step {} at {:.6} s",
                    state.step,
                    state.time_seconds()
                ),
                Err(e) => return Err(format!("{}: {}", e.code, e.message)),
            }
        }
        Some("verify-native") => native_verify::run()?,
        _ => {
            return Err(
                "usage: bredboard-tools <schema|validate|solve|simulate|snapshot|validate-snapshot|replay|verify-native ...>".into(),
            );
        }
    }
    Ok(())
}
fn schema_for<T: JsonSchema>() -> serde_json::Value {
    let schema = SchemaGenerator::new(SchemaSettings::draft2020_12()).into_root_schema_for::<T>();
    serde_json::to_value(schema).expect("generated schema serializes")
}
fn read_value(path: &str) -> Result<serde_json::Value, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    serde_json::from_str(&text).map_err(|e| format!("invalid JSON: {e}"))
}
fn validate_schema<T: JsonSchema>(value: &serde_json::Value) -> Result<(), String> {
    let schema = schema_for::<T>();
    let validator =
        jsonschema::validator_for(&schema).map_err(|e| format!("cannot construct schema: {e}"))?;
    let errors = validator
        .iter_errors(value)
        .map(|e| e.to_string())
        .collect::<Vec<_>>();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!("schema validation failed: {}", errors.join("; ")))
    }
}
fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}
