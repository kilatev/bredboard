use bredboard_core::{Contact, ControlState, Project, compile_topology, solve_dc};
use schemars::{SchemaGenerator, generate::SchemaSettings};
use std::{env, fs, process};

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("schema") => {
            let schema = SchemaGenerator::new(SchemaSettings::draft2020_12())
                .into_root_schema_for::<Project>();
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
            match solve_dc(&project, &states) {
                Ok(result) => {
                    for (node, voltage) in result.node_voltages {
                        let labels = node
                            .iter()
                            .filter_map(|contact| match contact {
                                Contact::ComponentPin(id, pin) => {
                                    Some(format!("{}.{}", id.0, pin.0))
                                }
                                Contact::Hole(_) => None,
                            })
                            .collect::<Vec<_>>()
                            .join(", ");
                        println!("node [{labels}]: {voltage:.9} V");
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
        _ => return Err("usage: bredboard-tools <schema|validate|solve <project.json>>".into()),
    }
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}
