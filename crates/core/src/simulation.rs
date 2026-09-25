use crate::{
    ComponentId, ComponentKind, ControlState, Diagnostic, ElectricalDiagnostic, ElectricalError,
    MAX_NONLINEAR_ITERATIONS, Project, SolveResult, compile_topology,
    solver::solve_transient_with_iteration_limit,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const STEP_SECONDS: f64 = 100e-6;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Action {
    Run,
    Pause,
    SingleStep,
    Reset,
    SetParameter {
        component: ComponentId,
        name: String,
        value: f64,
    },
    SetControl {
        component: ComponentId,
        state: ControlState,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SimulationDiagnostic {
    pub code: String,
    pub path: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SimulationState {
    pub step: u64,
    pub running: bool,
    pub capacitor_voltages: BTreeMap<ComponentId, f64>,
    pub controls: BTreeMap<ComponentId, ControlState>,
    pub last_valid: Option<SolveResult>,
    pub stale: bool,
    pub diagnostics: Vec<SimulationDiagnostic>,
    #[serde(default = "solve_required")]
    pub needs_solve: bool,
}

fn solve_required() -> bool {
    true
}

impl SimulationState {
    pub fn new(project: &Project) -> Self {
        let mut capacitor_voltages = BTreeMap::new();
        let mut controls = BTreeMap::new();
        for component in &project.components {
            match component.kind {
                ComponentKind::Capacitor => {
                    capacitor_voltages.insert(
                        component.id.clone(),
                        project
                            .initial_conditions
                            .capacitor_voltages
                            .get(&component.id)
                            .copied()
                            .unwrap_or(0.0),
                    );
                }
                ComponentKind::MomentaryButton => {
                    controls.insert(
                        component.id.clone(),
                        project
                            .initial_conditions
                            .controls
                            .get(&component.id)
                            .copied()
                            .unwrap_or(ControlState::ButtonReleased),
                    );
                }
                ComponentKind::ChangeoverSwitch => {
                    controls.insert(
                        component.id.clone(),
                        project
                            .initial_conditions
                            .controls
                            .get(&component.id)
                            .copied()
                            .unwrap_or(ControlState::SwitchNormallyClosed),
                    );
                }
                _ => {}
            }
        }
        Self {
            step: 0,
            running: false,
            capacitor_voltages,
            controls,
            last_valid: None,
            stale: false,
            diagnostics: Vec::new(),
            needs_solve: true,
        }
    }
    pub fn time_seconds(&self) -> f64 {
        self.step as f64 * STEP_SECONDS
    }
}

/// Reduce actions in order at the current step boundary. Parameter edits are
/// validated on a candidate project before becoming visible.
pub fn apply_actions(
    project: &mut Project,
    initial_project: &Project,
    state: &mut SimulationState,
    actions: &[Action],
) {
    for action in actions {
        match action {
            Action::Run => state.running = true,
            Action::Pause => state.running = false,
            Action::SingleStep => {
                step_once(project, state);
            }
            Action::Reset => {
                *project = initial_project.clone();
                *state = SimulationState::new(initial_project);
            }
            Action::SetParameter {
                component,
                name,
                value,
            } => {
                let mut candidate = project.clone();
                if let Some(c) = candidate.components.iter_mut().find(|c| &c.id == component) {
                    c.parameters.insert(name.clone(), *value);
                    match compile_topology(&candidate) {
                        Ok(_) => {
                            *project = candidate;
                            state.needs_solve = true;
                        }
                        Err(errors) => {
                            state.diagnostics =
                                errors.into_iter().map(structural_diagnostic).collect()
                        }
                    }
                } else {
                    state.diagnostics = vec![SimulationDiagnostic {
                        code: "unknown_component".into(),
                        path: format!("components.{}", component.0),
                        message: "parameter edit refers to an unknown component".into(),
                    }];
                }
            }
            Action::SetControl {
                component,
                state: control,
            } => {
                let kind = project
                    .components
                    .iter()
                    .find(|c| &c.id == component)
                    .map(|c| c.kind);
                let valid = matches!(
                    (kind, control),
                    (
                        Some(ComponentKind::MomentaryButton),
                        ControlState::ButtonPressed | ControlState::ButtonReleased
                    ) | (
                        Some(ComponentKind::ChangeoverSwitch),
                        ControlState::SwitchNormallyClosed | ControlState::SwitchNormallyOpen
                    )
                );
                if valid {
                    state.controls.insert(component.clone(), *control);
                    state.needs_solve = true;
                } else {
                    state.diagnostics = vec![SimulationDiagnostic {
                        code: "invalid_control_state".into(),
                        path: format!("components.{}", component.0),
                        message: "control state does not match a button or switch component".into(),
                    }];
                }
            }
        }
    }
}

/// Advance exactly `count` fixed steps while running; failures stop the run.
pub fn advance_steps(project: &Project, state: &mut SimulationState, count: u64) {
    for _ in 0..count {
        if !state.running || !step_once(project, state) {
            break;
        }
    }
}

fn step_once(project: &Project, state: &mut SimulationState) -> bool {
    step_once_with_iteration_limit(project, state, MAX_NONLINEAR_ITERATIONS)
}

fn step_once_with_iteration_limit(
    project: &Project,
    state: &mut SimulationState,
    max_iterations: usize,
) -> bool {
    if !state.needs_solve && state.last_valid.is_some() && state.capacitor_voltages.is_empty() {
        state.step += 1;
        return true;
    }
    match solve_transient_with_iteration_limit(
        project,
        &state.controls,
        &state.capacitor_voltages,
        max_iterations,
    ) {
        Ok(result) => {
            state
                .capacitor_voltages
                .extend(result.capacitor_voltages.clone());
            state.last_valid = Some(result);
            state.step += 1;
            state.stale = false;
            state.diagnostics.clear();
            state.needs_solve = false;
            true
        }
        Err(error) => {
            state.running = false;
            state.stale = true;
            state.diagnostics = match error {
                ElectricalError::Structure(errors) => {
                    errors.into_iter().map(structural_diagnostic).collect()
                }
                ElectricalError::Calculation(e) => vec![electrical_diagnostic(e)],
            };
            false
        }
    }
}

fn structural_diagnostic(d: Diagnostic) -> SimulationDiagnostic {
    SimulationDiagnostic {
        code: d.code.into(),
        path: d.path,
        message: d.message,
    }
}
fn electrical_diagnostic(d: ElectricalDiagnostic) -> SimulationDiagnostic {
    SimulationDiagnostic {
        code: d.code.into(),
        path: String::new(),
        message: d.message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn rc() -> Project {
        serde_json::from_str(include_str!("../../../fixtures/projects/rc-charging.json")).unwrap()
    }
    fn capacitor_voltage(state: &SimulationState) -> f64 {
        state.capacitor_voltages[&ComponentId("C1".into())]
    }

    #[test]
    fn run_pause_single_step_and_reset_are_discrete() {
        let baseline = rc();
        let mut project = baseline.clone();
        let mut state = SimulationState::new(&project);
        apply_actions(&mut project, &baseline, &mut state, &[Action::Run]);
        advance_steps(&project, &mut state, 7);
        apply_actions(
            &mut project,
            &baseline,
            &mut state,
            &[Action::Pause, Action::SingleStep],
        );
        assert_eq!(state.step, 8);
        assert!(!state.running);
        assert!(capacitor_voltage(&state) > 0.0);
        apply_actions(&mut project, &baseline, &mut state, &[Action::Reset]);
        assert_eq!(state.step, 0);
        assert!(!state.running);
        assert_eq!(capacitor_voltage(&state), 0.0);
        assert!(state.last_valid.is_none());
    }

    #[test]
    fn dc_readings_recalculate_after_control_actions() {
        let initial: Project =
            serde_json::from_str(include_str!("../../../fixtures/projects/led-bench.json"))
                .unwrap();
        let mut project = initial.clone();
        let mut state = SimulationState::new(&project);
        apply_actions(&mut project, &initial, &mut state, &[Action::Run]);
        advance_steps(&project, &mut state, 100);
        assert_eq!(state.step, 100);
        let id = ComponentId("D1".into());
        assert!(state.last_valid.as_ref().unwrap().led_currents[&id] < 1e-6);
        apply_actions(
            &mut project,
            &initial,
            &mut state,
            &[Action::SetControl {
                component: ComponentId("B1".into()),
                state: ControlState::ButtonPressed,
            }],
        );
        advance_steps(&project, &mut state, 1);
        assert!(state.last_valid.as_ref().unwrap().led_currents[&id] > 0.008);
        apply_actions(
            &mut project,
            &initial,
            &mut state,
            &[Action::SetControl {
                component: ComponentId("B1".into()),
                state: ControlState::ButtonReleased,
            }],
        );
        advance_steps(&project, &mut state, 1);
        assert!(state.last_valid.as_ref().unwrap().led_currents[&id] < 1e-6);
    }

    #[test]
    fn nonlinear_failure_stops_and_marks_previous_readings_stale() {
        let initial: Project =
            serde_json::from_str(include_str!("../../../fixtures/projects/led-bench.json"))
                .unwrap();
        let mut project = initial.clone();
        let mut state = SimulationState::new(&project);
        apply_actions(&mut project, &initial, &mut state, &[Action::Run]);
        advance_steps(&project, &mut state, 1);
        let previous = state.last_valid.clone();
        state.running = true;
        state.needs_solve = true;
        assert!(!step_once_with_iteration_limit(&project, &mut state, 0));
        assert_eq!(state.step, 1);
        assert!(!state.running);
        assert!(state.stale);
        assert_eq!(state.last_valid, previous);
        assert_eq!(state.diagnostics[0].code, "nonconvergence");
        assert!(state.diagnostics[0].message.contains("0 iterations"));
    }

    #[test]
    fn parameter_and_control_actions_are_ordered_and_resettable() {
        let baseline = rc();
        let mut project = baseline.clone();
        project.components.push(crate::Component {
            id: ComponentId("B1".into()),
            kind: ComponentKind::MomentaryButton,
            pins: BTreeMap::from([
                (crate::PinId("a".into()), crate::HoleId("A1".into())),
                (crate::PinId("b".into()), crate::HoleId("F1".into())),
            ]),
            parameters: BTreeMap::new(),
        });
        let baseline = project.clone();
        let mut state = SimulationState::new(&project);
        apply_actions(
            &mut project,
            &baseline,
            &mut state,
            &[
                Action::SetParameter {
                    component: ComponentId("R1".into()),
                    name: "resistance".into(),
                    value: 2000.0,
                },
                Action::SetControl {
                    component: ComponentId("B1".into()),
                    state: ControlState::ButtonPressed,
                },
                Action::SingleStep,
            ],
        );
        assert_eq!(project.components[1].parameters["resistance"], 2000.0);
        assert_eq!(
            state.controls[&ComponentId("B1".into())],
            ControlState::ButtonPressed
        );
        assert_eq!(state.step, 1);
        apply_actions(
            &mut project,
            &baseline,
            &mut state,
            &[Action::SetParameter {
                component: ComponentId("R1".into()),
                name: "resistance".into(),
                value: -1.0,
            }],
        );
        assert_eq!(project.components[1].parameters["resistance"], 2000.0);
        assert_eq!(state.diagnostics[0].code, "parameter_out_of_range");
        apply_actions(&mut project, &baseline, &mut state, &[Action::Reset]);
        assert_eq!(project, baseline);
        assert_eq!(state.step, 0);
        assert_eq!(
            state.controls[&ComponentId("B1".into())],
            ControlState::ButtonReleased
        );
    }

    #[test]
    fn rc_charge_and_discharge_samples_stay_within_one_percent_of_source() {
        let mut baseline = rc();
        let mut project = baseline.clone();
        let mut state = SimulationState::new(&project);
        apply_actions(&mut project, &baseline, &mut state, &[Action::Run]);
        advance_steps(&project, &mut state, 1000);
        let exact = 5.0 * (1.0 - (-1.0f64).exp());
        assert!((capacitor_voltage(&state) - exact).abs() < 0.01 * 5.0);

        baseline
            .initial_conditions
            .capacitor_voltages
            .insert(ComponentId("C1".into()), 5.0);
        baseline.components[0]
            .parameters
            .insert("voltage".into(), 0.0);
        project = baseline.clone();
        state = SimulationState::new(&project);
        apply_actions(&mut project, &baseline, &mut state, &[Action::Run]);
        advance_steps(&project, &mut state, 1000);
        let exact = 5.0 * (-1.0f64).exp();
        assert!((capacitor_voltage(&state) - exact).abs() < 0.01 * 5.0);
        apply_actions(&mut project, &baseline, &mut state, &[Action::Reset]);
        assert_eq!(state.step, 0);
        assert_eq!(capacitor_voltage(&state), 5.0);
    }

    #[test]
    fn failed_step_keeps_last_valid_state_and_marks_it_stale() {
        let mut project = rc();
        let baseline = project.clone();
        let mut state = SimulationState::new(&project);
        apply_actions(&mut project, &baseline, &mut state, &[Action::Run]);
        advance_steps(&project, &mut state, 2);
        let old_step = state.step;
        let old_solution = state.last_valid.clone();
        project
            .components
            .retain(|c| c.kind != ComponentKind::DcVoltageSource);
        state.running = true;
        advance_steps(&project, &mut state, 1);
        assert_eq!(state.step, old_step);
        assert_eq!(state.last_valid, old_solution);
        assert!(state.stale);
        assert_eq!(state.diagnostics[0].code, "floating_network");
        assert!(!state.running);
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 48, rng_seed: proptest::test_runner::RngSeed::Fixed(0xA004_2026), .. ProptestConfig::default() })]
        #[test]
        fn step_batching_is_independent_of_call_boundaries(batch in 1u32..500) {
            let mut p1=rc();let base=p1.clone();let mut a=SimulationState::new(&p1);apply_actions(&mut p1,&base,&mut a,&[Action::Run]);advance_steps(&p1,&mut a,1000);
            let mut p2=rc();let base=p2.clone();let mut b=SimulationState::new(&p2);apply_actions(&mut p2,&base,&mut b,&[Action::Run]);
            let mut left=1000u64;while left>0{let n=left.min(u64::from(batch));advance_steps(&p2,&mut b,n);left-=n;}
            prop_assert_eq!(b,a);
        }

        #[test]
        fn generated_action_sequences_reduce_deterministically(ops in prop::collection::vec((0u8..5, 1u16..200), 1..20)) {
            let base=rc();
            let schedule:Vec<_>=ops.iter().map(|(op,value)| match op {
                0=>Action::Run, 1=>Action::Pause, 2=>Action::SingleStep,
                3=>Action::SetParameter { component:ComponentId("R1".into()),name:"resistance".into(),value:100.0+f64::from(*value)*10.0 },
                _=>Action::Reset,
            }).collect();
            let mut p1=base.clone();let mut s1=SimulationState::new(&p1);apply_actions(&mut p1,&base,&mut s1,&schedule);
            let mut p2=base.clone();let mut s2=SimulationState::new(&p2);apply_actions(&mut p2,&base,&mut s2,&schedule);
            prop_assert_eq!(p1,p2);prop_assert_eq!(s1,s2);
        }

        #[test]
        fn breadboard_rc_ranges_solve_each_step(
            voltage_centi in 0u32..=1200,
            initial_centi in 0u32..=1200,
            resistor_bucket in 0u32..=1000,
            capacitor_bucket in 0u32..=1000,
        ) {
            let source_voltage = f64::from(voltage_centi) / 100.0;
            let initial_voltage = f64::from(initial_centi) / 100.0;
            let resistor = 10f64.powf(7.0 * f64::from(resistor_bucket) / 1000.0);
            let capacitance = 10f64.powf(-10.0 + 8.0 * f64::from(capacitor_bucket) / 1000.0);
            let mut project = rc();
            project
                .components
                .iter_mut()
                .for_each(|component| match component.id.0.as_str() {
                    "V1" => {
                        component.parameters.insert("voltage".into(), source_voltage);
                    }
                    "R1" => {
                        component.parameters.insert("resistance".into(), resistor);
                    }
                    "C1" => {
                        component.parameters.insert("capacitance".into(), capacitance);
                    }
                    _ => {}
                });
            project.initial_conditions.capacitor_voltages.insert(
                ComponentId("C1".into()),
                initial_voltage,
            );
            let low = source_voltage.min(initial_voltage);
            let high = source_voltage.max(initial_voltage);
            let baseline = project.clone();
            let mut state = SimulationState::new(&project);
            apply_actions(&mut project, &baseline, &mut state, &[Action::Run]);
            for expected_step in 1..=16 {
                advance_steps(&project, &mut state, 1);
                prop_assert_eq!(state.step, expected_step);
                prop_assert!(!state.stale);
                prop_assert!(state.diagnostics.is_empty());
                let voltage = capacitor_voltage(&state);
                prop_assert!(
                    voltage >= low - 1e-9 && voltage <= high + 1e-9,
                    "capacitor voltage {voltage} escaped {low}..={high}"
                );
            }
        }
    }
}
