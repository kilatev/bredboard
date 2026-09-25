use bredboard_core::{
    Action, ActionLog, ComponentId, ControlState, ElectricalError, Project, SimulationState,
    Snapshot, advance_steps, apply_actions, compile_topology, replay_action_log, solve_transient,
};
use std::collections::BTreeMap;
use std::process::Command;

const RC_SOURCE_VOLTAGE: f64 = 5.0;
const RC_TRACE_TOLERANCE: f64 = 0.01 * RC_SOURCE_VOLTAGE;
const RUNNER_SEED: &str = "0xA013_2026";
const RUNNER_CASES: usize = 32;

struct Transcript {
    project: Project,
    reset_project: Project,
    state: SimulationState,
    log: ActionLog,
    trace: Vec<String>,
}

pub fn run() -> Result<(), String> {
    println!("native verification");
    println!(
        "target: {}-{}",
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    println!("rustc: {}", rustc_version());
    println!("core model/solver: 1/1");
    println!("property seed/cases: {RUNNER_SEED}/{RUNNER_CASES}");

    for (name, json) in [
        (
            "led",
            include_str!("../../../fixtures/projects/led-bench.json"),
        ),
        (
            "rc",
            include_str!("../../../fixtures/projects/rc-bench.json"),
        ),
        (
            "transistor",
            include_str!("../../../fixtures/projects/transistor-bench.json"),
        ),
    ] {
        let first = transcript(json)?;
        let second = transcript(json)?;
        if first.project != second.project
            || first.reset_project != second.reset_project
            || first.state != second.state
            || first.trace != second.trace
        {
            return Err(format!("{name}: repeated native transcript diverged"));
        }
        verify_trace(name, &first)?;
        let (_, _, replayed) = replay_action_log(&first.log)
            .map_err(|error| format!("{name}: replay failed: {}", error.message))?;
        if replayed != first.state {
            return Err(format!(
                "{name}: replayed state differs from direct execution"
            ));
        }
        println!(
            "{name}: step {}, {} trace samples, replay matched",
            first.state.step,
            first.trace.len()
        );
    }

    verify_diagnostics()?;
    println!("native verification passed");
    Ok(())
}

fn transcript(json: &str) -> Result<Transcript, String> {
    let reset_project: Project = serde_json::from_str(json).map_err(|error| error.to_string())?;
    compile_topology(&reset_project).map_err(|errors| format_diagnostics(&errors))?;
    let mut project = reset_project.clone();
    let mut state = SimulationState::new(&project);
    let mut log = ActionLog::new(Snapshot::capture(&project, &reset_project, &state));
    let mut trace = Vec::new();

    record(
        &mut log,
        &mut project,
        &reset_project,
        &mut state,
        Action::Run,
    );
    advance_steps(&project, &mut state, 1000);
    record_trace(&mut trace, &state);

    let control = if reset_project
        .components
        .iter()
        .any(|component| component.id.0 == "S1")
    {
        Action::SetControl {
            component: ComponentId("S1".into()),
            state: ControlState::SwitchNormallyOpen,
        }
    } else {
        Action::SetControl {
            component: ComponentId("B1".into()),
            state: ControlState::ButtonPressed,
        }
    };
    record(&mut log, &mut project, &reset_project, &mut state, control);
    advance_steps(&project, &mut state, 1);
    record_trace(&mut trace, &state);
    log.finish(&state);

    if state.stale {
        return Err(format!(
            "{}: stale readings: {:?}",
            project.title, state.diagnostics
        ));
    }
    Ok(Transcript {
        project,
        reset_project,
        state,
        log,
        trace,
    })
}

fn record(
    log: &mut ActionLog,
    project: &mut Project,
    reset_project: &Project,
    state: &mut SimulationState,
    action: Action,
) {
    log.record(state, action.clone());
    apply_actions(project, reset_project, state, &[action]);
}

fn record_trace(trace: &mut Vec<String>, state: &SimulationState) {
    let reading = state.last_valid.as_ref();
    trace.push(format!(
        "step={} capacitor={:?} led={:?} collector={:?}",
        state.step,
        reading.map(|value| &value.capacitor_voltages),
        reading.map(|value| &value.led_currents),
        reading.map(|value| &value.transistor_collector_currents),
    ));
}

fn verify_trace(name: &str, transcript: &Transcript) -> Result<(), String> {
    let readings = transcript
        .state
        .last_valid
        .as_ref()
        .ok_or_else(|| format!("{name}: transcript has no readings"))?;
    match name {
        "led" => {
            let current = readings.led_currents[&ComponentId("D1".into())];
            expect_range(name, "LED current", current, 0.005, 0.02)?;
        }
        "rc" => {
            let voltage = readings.capacitor_voltages[&ComponentId("C1".into())];
            let expected = RC_SOURCE_VOLTAGE * (1.0 - (-1.0f64).exp());
            expect_close(
                name,
                "RC charge voltage",
                voltage,
                expected,
                RC_TRACE_TOLERANCE,
            )?;
        }
        "transistor" => {
            let led = readings.led_currents[&ComponentId("D1".into())];
            let collector = readings.transistor_collector_currents[&ComponentId("Q1".into())];
            expect_range(name, "transistor LED current", led, 0.005, 0.02)?;
            expect_close(name, "collector/LED current", collector, led, 1e-9)?;
        }
        _ => return Err(format!("unknown transcript {name}")),
    }
    Ok(())
}

fn verify_diagnostics() -> Result<(), String> {
    let floating: Project = serde_json::from_str(include_str!(
        "../../../fixtures/projects/floating-resistors.json"
    ))
    .map_err(|error| error.to_string())?;
    expect_diagnostic(
        "floating",
        solve_transient(&floating, &BTreeMap::new(), &BTreeMap::new()),
        "floating_network",
    )?;

    let conflicting: Project = serde_json::from_str(include_str!(
        "../../../fixtures/projects/conflicting-sources.json"
    ))
    .map_err(|error| error.to_string())?;
    expect_diagnostic(
        "contradictory sources",
        solve_transient(&conflicting, &BTreeMap::new(), &BTreeMap::new()),
        "conflicting_sources",
    )?;

    let malformed: Project = serde_json::from_str(include_str!(
        "../../../fixtures/projects/invalid-reference.json"
    ))
    .map_err(|error| error.to_string())?;
    if compile_topology(&malformed).is_ok() {
        return Err("malformed: invalid reference was accepted".into());
    }
    println!("diagnostics: malformed, floating, and contradictory cases rejected");
    Ok(())
}

fn expect_diagnostic(
    name: &str,
    result: Result<bredboard_core::SolveResult, ElectricalError>,
    expected: &str,
) -> Result<(), String> {
    match result {
        Err(ElectricalError::Calculation(diagnostic)) if diagnostic.code == expected => Ok(()),
        Err(ElectricalError::Calculation(diagnostic)) => Err(format!(
            "{name}: expected {expected}, got {}",
            diagnostic.code
        )),
        Err(ElectricalError::Structure(errors)) => {
            Err(format!("{name}: expected {expected}, got {errors:?}"))
        }
        Ok(_) => Err(format!("{name}: expected {expected}, but solve succeeded")),
    }
}

fn expect_close(
    name: &str,
    label: &str,
    actual: f64,
    expected: f64,
    tolerance: f64,
) -> Result<(), String> {
    if (actual - expected).abs() <= tolerance {
        Ok(())
    } else {
        Err(format!(
            "{name}: {label} {actual} differs from {expected} by more than {tolerance}"
        ))
    }
}

fn expect_range(name: &str, label: &str, actual: f64, low: f64, high: f64) -> Result<(), String> {
    if (low..=high).contains(&actual) {
        Ok(())
    } else {
        Err(format!(
            "{name}: {label} {actual} is outside {low}..={high}"
        ))
    }
}

fn format_diagnostics(errors: &[bredboard_core::Diagnostic]) -> String {
    errors
        .iter()
        .map(|error| format!("{} at {}: {}", error.code, error.path, error.message))
        .collect::<Vec<_>>()
        .join("; ")
}

fn rustc_version() -> String {
    Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|version| version.trim().to_owned())
        .unwrap_or_else(|| "unavailable".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intentionally_mismatched_fixture_is_detected() {
        let error = expect_close("test", "fixture", 1.0, 1.1, 0.01).unwrap_err();
        assert!(error.contains("fixture"));
    }

    #[test]
    fn native_runner_transcripts_pass() {
        run().unwrap();
    }
}
