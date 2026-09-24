use bredboard_core::{Project, compile_topology};
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
        _ => return Err("usage: bredboard-tools <schema|validate <project.json>>".into()),
    }
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}
