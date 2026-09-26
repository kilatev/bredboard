use crate::{
    Action, ComponentKind, ControlState, Project, SimulationState, advance_steps, apply_actions,
    compile_topology,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const SNAPSHOT_FORMAT_VERSION: u32 = 1;
pub const ACTION_LOG_FORMAT_VERSION: u32 = 1;
pub const MODEL_VERSION: u32 = 1;
pub const SOLVER_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "A validated, resumable Bredboard simulation state.")]
pub struct Snapshot {
    pub format_version: u32,
    pub model_version: u32,
    pub solver_version: u32,
    pub project: Project,
    pub reset_project: Project,
    pub state: SimulationState,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(
    description = "An ordered list of user actions with the simulation step at each action boundary."
)]
pub struct ActionLog {
    pub format_version: u32,
    pub model_version: u32,
    pub solver_version: u32,
    pub initial_snapshot: Snapshot,
    pub events: Vec<ActionEvent>,
    pub final_step: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ActionEvent {
    pub step: u64,
    pub action: Action,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersistenceError {
    pub code: &'static str,
    pub message: String,
}

impl Snapshot {
    pub fn capture(project: &Project, reset_project: &Project, state: &SimulationState) -> Self {
        Self {
            format_version: SNAPSHOT_FORMAT_VERSION,
            model_version: MODEL_VERSION,
            solver_version: SOLVER_VERSION,
            project: project.clone(),
            reset_project: reset_project.clone(),
            state: state.clone(),
        }
    }
}

impl ActionLog {
    pub fn new(initial_snapshot: Snapshot) -> Self {
        let final_step = initial_snapshot.state.step;
        Self {
            format_version: ACTION_LOG_FORMAT_VERSION,
            model_version: MODEL_VERSION,
            solver_version: SOLVER_VERSION,
            initial_snapshot,
            events: Vec::new(),
            final_step,
        }
    }

    /// Record before applying the action so the step is its exact boundary.
    pub fn record(&mut self, state: &SimulationState, action: Action) {
        self.events.push(ActionEvent {
            step: state.step,
            action,
        });
    }

    pub fn finish(&mut self, state: &SimulationState) {
        self.final_step = state.step;
    }
}

/// Validate and restore into new values. The caller replaces active state only
/// after this returns successfully, so invalid data cannot partially mutate it.
pub fn restore_snapshot(
    snapshot: &Snapshot,
) -> Result<(Project, Project, SimulationState), PersistenceError> {
    check_versions(
        snapshot.format_version,
        snapshot.model_version,
        snapshot.solver_version,
    )?;
    let topology = compile_topology(&snapshot.project).map_err(|e| {
        persistence_error(
            "invalid_project",
            e.into_iter()
                .map(|d| format!("{} at {}: {}", d.code, d.path, d.message))
                .collect::<Vec<_>>()
                .join("; "),
        )
    })?;
    compile_topology(&snapshot.reset_project).map_err(|_| {
        persistence_error(
            "invalid_reset_project",
            "reset project is structurally invalid",
        )
    })?;
    let capacitor_ids: BTreeSet<_> = snapshot
        .project
        .components
        .iter()
        .filter(|c| c.kind == ComponentKind::Capacitor)
        .map(|c| c.id.clone())
        .collect();
    let state_cap_ids: BTreeSet<_> = snapshot.state.capacitor_voltages.keys().cloned().collect();
    if capacitor_ids != state_cap_ids
        || snapshot
            .state
            .capacitor_voltages
            .values()
            .any(|v| !v.is_finite())
    {
        return Err(persistence_error(
            "invalid_capacitor_state",
            "snapshot capacitor state is incomplete or non-finite",
        ));
    }
    let control_ids: BTreeSet<_> = snapshot
        .project
        .components
        .iter()
        .filter(|c| {
            matches!(
                c.kind,
                ComponentKind::MomentaryButton | ComponentKind::ChangeoverSwitch
            )
        })
        .map(|c| c.id.clone())
        .collect();
    if control_ids != snapshot.state.controls.keys().cloned().collect() {
        return Err(persistence_error(
            "invalid_control_state",
            "snapshot control state is incomplete",
        ));
    }
    for (id, control) in &snapshot.state.controls {
        let valid = snapshot.project.components.iter().any(|c| {
            &c.id == id
                && matches!(
                    (c.kind, control),
                    (
                        ComponentKind::MomentaryButton,
                        ControlState::ButtonPressed | ControlState::ButtonReleased
                    ) | (
                        ComponentKind::ChangeoverSwitch,
                        ControlState::SwitchNormallyClosed | ControlState::SwitchNormallyOpen
                    )
                )
        });
        if !valid {
            return Err(persistence_error(
                "invalid_control_state",
                format!(
                    "snapshot control state for {} does not match a project control",
                    id.0
                ),
            ));
        }
    }
    let ratio_ids: BTreeSet<_> = snapshot
        .project
        .components
        .iter()
        .filter(|c| {
            matches!(
                c.kind,
                ComponentKind::Potentiometer | ComponentKind::Photoresistor
            )
        })
        .map(|c| c.id.clone())
        .collect();
    if ratio_ids != snapshot.state.control_ratios.keys().cloned().collect()
        || snapshot
            .state
            .control_ratios
            .values()
            .any(|ratio| !ratio.is_finite() || !(0.0..=1.0).contains(ratio))
    {
        return Err(persistence_error(
            "invalid_control_ratio_state",
            "snapshot control ratio state is incomplete or out of range",
        ));
    }
    if snapshot.state.stale && snapshot.state.running {
        return Err(persistence_error(
            "invalid_run_state",
            "a stale failed simulation cannot still be running",
        ));
    }
    if snapshot.state.step > 0 && !snapshot.state.stale && snapshot.state.last_valid.is_none() {
        return Err(persistence_error(
            "missing_readings",
            "a non-stale advanced snapshot must include its last valid readings",
        ));
    }
    if let Some(readings) = &snapshot.state.last_valid {
        let valid_nodes: BTreeSet<_> = topology.iter().map(|n| n.contacts.clone()).collect();
        let ids_of = |kind| {
            snapshot
                .project
                .components
                .iter()
                .filter(move |c| c.kind == kind)
                .map(|c| c.id.clone())
                .collect::<BTreeSet<_>>()
        };
        let resistor_ids = ids_of(ComponentKind::Resistor);
        let source_ids = ids_of(ComponentKind::DcVoltageSource);
        let capacitor_ids = ids_of(ComponentKind::Capacitor);
        let led_ids = ids_of(ComponentKind::Led);
        let transistor_ids = ids_of(ComponentKind::NpnTransistor);
        let switch_ids = ids_of(ComponentKind::MomentaryButton)
            .union(&ids_of(ComponentKind::ChangeoverSwitch))
            .cloned()
            .collect::<BTreeSet<_>>();
        let actual_switch_ids: BTreeSet<_> = readings.switch_currents.keys().cloned().collect();
        if readings
            .resistor_currents
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>()
            != resistor_ids
            || readings
                .source_currents
                .keys()
                .cloned()
                .collect::<BTreeSet<_>>()
                != source_ids
            || !actual_switch_ids.is_subset(&switch_ids)
            || readings
                .capacitor_currents
                .keys()
                .cloned()
                .collect::<BTreeSet<_>>()
                != capacitor_ids
            || readings
                .capacitor_voltages
                .keys()
                .cloned()
                .collect::<BTreeSet<_>>()
                != capacitor_ids
            || readings
                .led_currents
                .keys()
                .cloned()
                .collect::<BTreeSet<_>>()
                != led_ids
            || readings
                .transistor_collector_currents
                .keys()
                .cloned()
                .collect::<BTreeSet<_>>()
                != transistor_ids
        {
            return Err(persistence_error(
                "invalid_readings",
                "snapshot readings do not match project component IDs",
            ));
        }
        if readings
            .node_voltages
            .iter()
            .any(|n| !valid_nodes.contains(&n.contacts) || !n.voltage.is_finite())
            || readings
                .resistor_currents
                .values()
                .chain(readings.source_currents.values())
                .chain(readings.switch_currents.values())
                .chain(readings.capacitor_currents.values())
                .chain(readings.capacitor_voltages.values())
                .chain(readings.led_currents.values())
                .chain(readings.transistor_collector_currents.values())
                .any(|v| !v.is_finite())
        {
            return Err(persistence_error(
                "invalid_readings",
                "snapshot contains invalid or non-derived node readings",
            ));
        }
    }
    Ok((
        snapshot.project.clone(),
        snapshot.reset_project.clone(),
        snapshot.state.clone(),
    ))
}

pub fn replay_action_log(
    log: &ActionLog,
) -> Result<(Project, Project, SimulationState), PersistenceError> {
    if log.format_version != ACTION_LOG_FORMAT_VERSION {
        return Err(persistence_error(
            "unsupported_action_log_version",
            format!("supported action log format is {ACTION_LOG_FORMAT_VERSION}"),
        ));
    }
    check_model_solver_versions(log.model_version, log.solver_version)?;
    let (mut project, reset_project, mut state) = restore_snapshot(&log.initial_snapshot)?;
    for event in &log.events {
        if event.step < state.step {
            return Err(persistence_error(
                "invalid_action_boundary",
                "action boundaries must be ordered and cannot precede the current step",
            ));
        }
        let steps = event.step - state.step;
        advance_steps(&project, &mut state, steps);
        if state.step != event.step {
            return Err(persistence_error(
                "unreachable_action_boundary",
                format!("simulation could not reach action boundary {}", event.step),
            ));
        }
        apply_actions(
            &mut project,
            &reset_project,
            &mut state,
            std::slice::from_ref(&event.action),
        );
    }
    if log.final_step < state.step {
        return Err(persistence_error(
            "invalid_final_step",
            "final step precedes the replayed actions",
        ));
    }
    let steps = log.final_step - state.step;
    advance_steps(&project, &mut state, steps);
    if state.step != log.final_step {
        return Err(persistence_error(
            "unreachable_final_step",
            format!("simulation could not reach final step {}", log.final_step),
        ));
    }
    Ok((project, reset_project, state))
}

fn check_versions(format: u32, model: u32, solver: u32) -> Result<(), PersistenceError> {
    if format != SNAPSHOT_FORMAT_VERSION {
        return Err(persistence_error(
            "unsupported_snapshot_version",
            format!("supported snapshot format is {SNAPSHOT_FORMAT_VERSION}"),
        ));
    }
    check_model_solver_versions(model, solver)
}
fn check_model_solver_versions(model: u32, solver: u32) -> Result<(), PersistenceError> {
    if model != MODEL_VERSION {
        return Err(persistence_error(
            "unsupported_model_version",
            format!("supported model version is {MODEL_VERSION}"),
        ));
    }
    if solver != SOLVER_VERSION {
        return Err(persistence_error(
            "unsupported_solver_version",
            format!("supported solver version is {SOLVER_VERSION}"),
        ));
    }
    Ok(())
}
fn persistence_error(code: &'static str, message: impl Into<String>) -> PersistenceError {
    PersistenceError {
        code,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Action, ComponentId, STEP_SECONDS, apply_actions};
    use proptest::prelude::*;

    fn project() -> Project {
        serde_json::from_str(include_str!("../../../fixtures/projects/rc-charging.json")).unwrap()
    }

    fn variable_resistor_project() -> Project {
        let mut p = project();
        p.components[1].kind = ComponentKind::Potentiometer;
        p.components[1].parameters = std::collections::BTreeMap::from([
            ("min_resistance".into(), 100.0),
            ("max_resistance".into(), 10_000.0),
        ]);
        p.components.push(crate::Component {
            id: ComponentId("R2".into()),
            kind: ComponentKind::Photoresistor,
            pins: std::collections::BTreeMap::from([
                (crate::PinId("a".into()), crate::HoleId("A5".into())),
                (crate::PinId("b".into()), crate::HoleId("A6".into())),
            ]),
            parameters: std::collections::BTreeMap::from([
                ("min_resistance".into(), 200.0),
                ("max_resistance".into(), 20_000.0),
            ]),
        });
        p.wires.push(crate::Wire {
            id: crate::WireId("W2".into()),
            from: crate::HoleId("A5".into()),
            to: crate::HoleId("A7".into()),
        });
        p
    }

    #[test]
    fn snapshot_round_trip_preserves_control_ratios_for_variable_resistors() {
        let reset = variable_resistor_project();
        let mut current = reset.clone();
        let mut state = SimulationState::new(&current);
        assert_eq!(
            state.control_ratios[&ComponentId("R1".into())],
            0.5,
            "default ratio"
        );
        apply_actions(
            &mut current,
            &reset,
            &mut state,
            &[
                Action::SetControlRatio {
                    component: ComponentId("R1".into()),
                    ratio: 0.25,
                },
                Action::SetControlRatio {
                    component: ComponentId("R2".into()),
                    ratio: 0.75,
                },
            ],
        );
        let encoded = serde_json::to_string(&Snapshot::capture(&current, &reset, &state)).unwrap();
        let decoded: Snapshot = serde_json::from_str(&encoded).unwrap();
        let (restored, restored_reset, resumed) = restore_snapshot(&decoded).unwrap();
        assert_eq!(restored, current);
        assert_eq!(restored_reset, reset);
        assert_eq!(resumed, state);
        assert_eq!(resumed.control_ratios[&ComponentId("R1".into())], 0.25);
        assert_eq!(resumed.control_ratios[&ComponentId("R2".into())], 0.75);
    }

    #[test]
    fn snapshot_round_trip_continues_identically() {
        let reset = project();
        let mut current = reset.clone();
        let mut state = SimulationState::new(&current);
        apply_actions(&mut current, &reset, &mut state, &[Action::Run]);
        advance_steps(&current, &mut state, 317);
        let encoded = serde_json::to_string(&Snapshot::capture(&current, &reset, &state)).unwrap();
        let decoded: Snapshot = serde_json::from_str(&encoded).unwrap();
        let (restored, restored_reset, mut resumed) = restore_snapshot(&decoded).unwrap();
        advance_steps(&current, &mut state, 683);
        advance_steps(&restored, &mut resumed, 683);
        assert_eq!(restored, current);
        assert_eq!(restored_reset, reset);
        assert_eq!(resumed, state);
    }

    #[test]
    fn transistor_snapshot_preserves_calculated_led_and_collector_readings() {
        let reset: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/transistor-bench.json"
        ))
        .unwrap();
        let mut current = reset.clone();
        let mut state = SimulationState::new(&current);
        apply_actions(
            &mut current,
            &reset,
            &mut state,
            &[
                Action::SetControl {
                    component: ComponentId("B1".into()),
                    state: ControlState::ButtonPressed,
                },
                Action::Run,
            ],
        );
        advance_steps(&current, &mut state, 100);
        let encoded = serde_json::to_string(&Snapshot::capture(&current, &reset, &state)).unwrap();
        let decoded: Snapshot = serde_json::from_str(&encoded).unwrap();
        let (restored, _, mut resumed) = restore_snapshot(&decoded).unwrap();
        advance_steps(&current, &mut state, 100);
        advance_steps(&restored, &mut resumed, 100);
        assert_eq!(resumed, state);
        assert!(resumed.last_valid.unwrap().led_currents[&ComponentId("D1".into())] > 0.008);
    }

    #[test]
    fn rejects_unsupported_and_corrupt_snapshots_without_mutation() {
        let project = project();
        let state = SimulationState::new(&project);
        let original = state.clone();
        let mut snapshot = Snapshot::capture(&project, &project, &state);
        snapshot.format_version += 1;
        assert_eq!(
            restore_snapshot(&snapshot).unwrap_err().code,
            "unsupported_snapshot_version"
        );
        assert_eq!(state, original);
        snapshot.format_version = SNAPSHOT_FORMAT_VERSION;
        snapshot.state.capacitor_voltages.clear();
        assert_eq!(
            restore_snapshot(&snapshot).unwrap_err().code,
            "invalid_capacitor_state"
        );
        assert_eq!(state, original);
        let mut snapshot = Snapshot::capture(&project, &project, &state);
        let mut readings = crate::solve_transient(
            &project,
            &state.controls,
            &state.capacitor_voltages,
            &state.control_ratios,
        )
        .unwrap();
        readings
            .resistor_currents
            .insert(ComponentId("unknown".into()), 0.0);
        snapshot.state.last_valid = Some(readings);
        assert_eq!(
            restore_snapshot(&snapshot).unwrap_err().code,
            "invalid_readings"
        );
    }

    #[test]
    fn action_log_replays_ordered_step_boundaries() {
        let reset = project();
        let mut current = reset.clone();
        let mut state = SimulationState::new(&current);
        let mut log = ActionLog::new(Snapshot::capture(&current, &reset, &state));
        let action = Action::Run;
        log.record(&state, action.clone());
        apply_actions(&mut current, &reset, &mut state, &[action]);
        advance_steps(&current, &mut state, 300);
        let action = Action::SetParameter {
            component: ComponentId("R1".into()),
            name: "resistance".into(),
            value: 2200.0,
        };
        log.record(&state, action.clone());
        apply_actions(&mut current, &reset, &mut state, &[action]);
        advance_steps(&current, &mut state, 500);
        let action = Action::Pause;
        log.record(&state, action.clone());
        apply_actions(&mut current, &reset, &mut state, &[action]);
        log.finish(&state);
        let decoded: ActionLog =
            serde_json::from_str(&serde_json::to_string(&log).unwrap()).unwrap();
        let (replayed, _, replay_state) = replay_action_log(&decoded).unwrap();
        assert_eq!(replayed, current);
        assert_eq!(replay_state, state);
    }

    #[test]
    fn example_files_match_described_snapshot_and_action_log_schemas() {
        let snapshot_schema =
            schemars::SchemaGenerator::new(schemars::generate::SchemaSettings::draft2020_12())
                .into_root_schema_for::<Snapshot>();
        let snapshot_schema = serde_json::to_value(snapshot_schema).unwrap();
        assert!(
            snapshot_schema["description"]
                .as_str()
                .unwrap()
                .contains("simulation state")
        );
        let snapshot_validator = jsonschema::validator_for(&snapshot_schema).unwrap();
        let snapshot: serde_json::Value = serde_json::from_str(include_str!(
            "../../../fixtures/snapshots/rc-charging-1000.json"
        ))
        .unwrap();
        assert!(snapshot_validator.is_valid(&snapshot));

        let log_schema =
            schemars::SchemaGenerator::new(schemars::generate::SchemaSettings::draft2020_12())
                .into_root_schema_for::<ActionLog>();
        let log_schema = serde_json::to_value(log_schema).unwrap();
        assert!(
            log_schema["description"]
                .as_str()
                .unwrap()
                .contains("ordered list")
        );
        let log_validator = jsonschema::validator_for(&log_schema).unwrap();
        let log: serde_json::Value = serde_json::from_str(include_str!(
            "../../../fixtures/action-logs/rc-charging-10-steps.json"
        ))
        .unwrap();
        assert!(log_validator.is_valid(&log));
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 24, rng_seed: proptest::test_runner::RngSeed::Fixed(0xA005_2026), .. ProptestConfig::default() })]
        #[test]
        fn random_snapshot_boundaries_resume_exactly(boundary in 0u64..1000, resistance in 100u32..100_000) {
            let mut uninterrupted=project(); uninterrupted.components[1].parameters.insert("resistance".into(),f64::from(resistance));
            let reset=uninterrupted.clone(); let mut original=SimulationState::new(&uninterrupted);
            apply_actions(&mut uninterrupted,&reset,&mut original,&[Action::Run]); advance_steps(&uninterrupted,&mut original,boundary);
            let (resumed_project,resumed_reset,mut resumed)=restore_snapshot(&Snapshot::capture(&uninterrupted,&reset,&original)).unwrap();
            let remaining=1000-boundary; advance_steps(&uninterrupted,&mut original,remaining); advance_steps(&resumed_project,&mut resumed,remaining);
            prop_assert_eq!(&resumed_project,&uninterrupted); prop_assert_eq!(&resumed_reset,&reset); prop_assert_eq!(&resumed,&original);
            prop_assert!((original.time_seconds()-1000.0*STEP_SECONDS).abs()<f64::EPSILON);
        }

    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 32, rng_seed: proptest::test_runner::RngSeed::Fixed(0xA013_2026), .. ProptestConfig::default() })]
        #[test]
        fn generated_action_logs_replay_exactly(
            operations in prop::collection::vec((0u8..5, 0u16..40), 1..24)
        ) {
            let reset = project();
            let mut current = reset.clone();
            let mut state = SimulationState::new(&current);
            let mut log = ActionLog::new(Snapshot::capture(&current, &reset, &state));
            for (operation, gap) in operations {
                advance_steps(&current, &mut state, u64::from(gap));
                let action = match operation {
                    0 => Action::Run,
                    1 => Action::Pause,
                    2 => Action::SingleStep,
                    3 => Action::Reset,
                    _ => Action::SetParameter {
                        component: ComponentId("R1".into()),
                        name: "resistance".into(),
                        value: 100.0 + f64::from(gap) * 100.0,
                    },
                };
                log.record(&state, action.clone());
                apply_actions(&mut current, &reset, &mut state, &[action]);
            }
            log.finish(&state);
            let encoded = serde_json::to_string(&log).unwrap();
            let decoded: ActionLog = serde_json::from_str(&encoded).unwrap();
            let (replayed, replayed_reset, replayed_state) = replay_action_log(&decoded).unwrap();
            prop_assert_eq!(replayed, current);
            prop_assert_eq!(replayed_reset, reset);
            prop_assert_eq!(replayed_state, state);
        }
    }
}
