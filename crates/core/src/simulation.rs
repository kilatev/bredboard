use crate::{
    ComponentId, ComponentKind, ControlState, Diagnostic, ElectricalDiagnostic, ElectricalError,
    Project, SolveResult, compile_topology, solve_transient,
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
                        Ok(_) => *project = candidate,
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
    match solve_transient(project, &state.controls, &state.capacitor_voltages) {
        Ok(result) => {
            state
                .capacitor_voltages
                .extend(result.capacitor_voltages.clone());
            state.last_valid = Some(result);
            state.step += 1;
            state.stale = false;
            state.diagnostics.clear();
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
    }
}
