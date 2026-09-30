use crate::{
    ComponentId, ComponentKind, ControlState, Diagnostic, ElectricalDiagnostic, ElectricalError,
    FaultRepair, MAX_NONLINEAR_ITERATIONS, Project, SolveResult, compile_topology,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const STEP_SECONDS: f64 = 100e-6;
pub const PASSIVE_PIEZO_HISTORY_STEPS: usize = 64;
const PASSIVE_PIEZO_REQUIRED_VARIATIONS: usize = 2;

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
    /// Sets a variable resistor's continuous control ratio
    /// (0.0..=1.0). Routed through the same ordered action reduction as
    /// `SetControl`, so two ratio changes between steps apply in order and
    /// do not depend on frame rate.
    SetControlRatio {
        component: ComponentId,
        ratio: f64,
    },
    RepairFault {
        fault: String,
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
    /// Stateful digital component values. Counter bits occupy the low ten
    /// bits; shift-register storage uses low and high bytes for shift and
    /// latched outputs. Edge-history bits are implementation state.
    #[serde(default)]
    pub digital_states: BTreeMap<ComponentId, u32>,
    pub controls: BTreeMap<ComponentId, ControlState>,
    /// Continuous 0.0..=1.0 control ratio per variable resistor.
    #[serde(default)]
    pub control_ratios: BTreeMap<ComponentId, f64>,
    /// Recent calculated currents for passive piezos, in fixed-step order.
    /// This is presentation input owned by the core so a passive piezo can
    /// distinguish an oscillating drive from steady DC.
    #[serde(default)]
    pub piezo_current_history: BTreeMap<ComponentId, Vec<f64>>,
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
        let mut digital_states = BTreeMap::new();
        let mut controls = BTreeMap::new();
        let mut control_ratios = BTreeMap::new();
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
                ComponentKind::Potentiometer
                | ComponentKind::Photoresistor
                | ComponentKind::Thermistor
                | ComponentKind::TouchPad
                | ComponentKind::WaterProbe => {
                    control_ratios.insert(
                        component.id.clone(),
                        project
                            .initial_conditions
                            .control_ratios
                            .get(&component.id)
                            .copied()
                            .unwrap_or(0.5),
                    );
                }
                ComponentKind::DFlipFlop
                | ComponentKind::DigitalCounter
                | ComponentKind::ShiftRegister
                | ComponentKind::StepSequencer
                | ComponentKind::Sram => {
                    digital_states.insert(component.id.clone(), 0);
                }
                _ => {}
            }
        }
        Self {
            step: 0,
            running: false,
            capacitor_voltages,
            digital_states,
            controls,
            control_ratios,
            piezo_current_history: BTreeMap::new(),
            last_valid: None,
            stale: false,
            diagnostics: Vec::new(),
            needs_solve: true,
        }
    }
    pub fn time_seconds(&self) -> f64 {
        self.step as f64 * STEP_SECONDS
    }

    /// Whether a passive piezo's recent calculated current contains an
    /// oscillating drive rather than steady DC.
    pub fn passive_piezo_sounding(&self, component: &ComponentId) -> bool {
        self.piezo_current_history
            .get(component)
            .is_some_and(|history| passive_piezo_is_sounding(history))
    }
}

/// Detect an externally oscillating passive-piezo drive from one fixed-step
/// current window. Threshold crossings cover a square wave switching between
/// silence and load current; magnitude and polarity changes cover drives that
/// stay above threshold or reverse direction. A constant current has no
/// variations and therefore never sounds.
pub fn passive_piezo_is_sounding(history: &[f64]) -> bool {
    if history.len() < 3 {
        return false;
    }
    let threshold_variations = history
        .windows(2)
        .filter(|pair| {
            let previous = pair[0];
            let current = pair[1];
            let threshold_crossed = (previous.abs() >= crate::SOUNDING_CURRENT)
                != (current.abs() >= crate::SOUNDING_CURRENT);
            let polarity_changed = previous.signum() != current.signum()
                && previous.abs() >= crate::SOUNDING_CURRENT
                && current.abs() >= crate::SOUNDING_CURRENT;
            threshold_crossed || polarity_changed
        })
        .count();
    let directional_variations = history
        .windows(3)
        .filter(|window| {
            let rising = window[1] - window[0];
            let falling = window[2] - window[1];
            rising * falling < 0.0
                && rising.abs() >= crate::SOUNDING_CURRENT * 0.5
                && falling.abs() >= crate::SOUNDING_CURRENT * 0.5
        })
        .count();
    threshold_variations + directional_variations >= PASSIVE_PIEZO_REQUIRED_VARIATIONS
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
                            state.piezo_current_history.clear();
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
            Action::SetControlRatio { component, ratio } => {
                let kind = project
                    .components
                    .iter()
                    .find(|c| &c.id == component)
                    .map(|c| c.kind);
                let valid = matches!(
                    kind,
                    Some(
                        ComponentKind::Potentiometer
                            | ComponentKind::Photoresistor
                            | ComponentKind::Thermistor
                            | ComponentKind::TouchPad
                            | ComponentKind::WaterProbe,
                    )
                ) && ratio.is_finite()
                    && (0.0..=1.0).contains(ratio);
                if valid {
                    state.control_ratios.insert(component.clone(), *ratio);
                    state.needs_solve = true;
                } else {
                    state.diagnostics = vec![SimulationDiagnostic {
                        code: "invalid_control_ratio".into(),
                        path: format!("components.{}", component.0),
                        message: "control ratio must be finite, in 0.0..=1.0, and match a variable-resistor component".into(),
                    }];
                }
            }
            Action::RepairFault { fault } => {
                let mut candidate = project.clone();
                let Some(spec) = candidate.faults.iter().find(|item| item.id == *fault) else {
                    state.diagnostics = vec![SimulationDiagnostic {
                        code: "unknown_fault".into(),
                        path: format!("faults.{fault}"),
                        message: "fault repair refers to an unknown fault".into(),
                    }];
                    continue;
                };
                let repairs = spec.repairs.clone();
                match apply_fault_repairs(&mut candidate, &repairs) {
                    Ok(()) => match compile_topology(&candidate) {
                        Ok(_) => {
                            candidate.faults.retain(|item| item.id != *fault);
                            *project = candidate;
                            state.last_valid = None;
                            state.piezo_current_history.clear();
                            state.stale = true;
                            state.needs_solve = true;
                            state.diagnostics.clear();
                        }
                        Err(errors) => {
                            state.diagnostics =
                                errors.into_iter().map(structural_diagnostic).collect();
                        }
                    },
                    Err(message) => {
                        state.diagnostics = vec![SimulationDiagnostic {
                            code: "invalid_fault_repair".into(),
                            path: format!("faults.{fault}"),
                            message,
                        }];
                    }
                }
            }
        }
    }
}

fn apply_fault_repairs(project: &mut Project, repairs: &[FaultRepair]) -> Result<(), String> {
    for repair in repairs {
        match repair {
            FaultRepair::SwapPins {
                component,
                first,
                second,
            } => {
                let c = project
                    .components
                    .iter_mut()
                    .find(|item| item.id == *component)
                    .ok_or_else(|| format!("unknown component {}", component.0))?;
                let first_hole = c
                    .pins
                    .get(first)
                    .cloned()
                    .ok_or_else(|| format!("unknown pin {}", first.0))?;
                let second_hole = c
                    .pins
                    .get(second)
                    .cloned()
                    .ok_or_else(|| format!("unknown pin {}", second.0))?;
                c.pins.insert(first.clone(), second_hole);
                c.pins.insert(second.clone(), first_hole);
            }
            FaultRepair::SetPin {
                component,
                pin,
                hole,
            } => {
                let c = project
                    .components
                    .iter_mut()
                    .find(|item| item.id == *component)
                    .ok_or_else(|| format!("unknown component {}", component.0))?;
                if !c.pins.contains_key(pin) {
                    return Err(format!("unknown pin {}", pin.0));
                }
                c.pins.insert(pin.clone(), hole.clone());
            }
            FaultRepair::SetParameter {
                component,
                name,
                value,
            } => {
                let c = project
                    .components
                    .iter_mut()
                    .find(|item| item.id == *component)
                    .ok_or_else(|| format!("unknown component {}", component.0))?;
                c.parameters.insert(name.clone(), *value);
            }
            FaultRepair::AddWire { wire } => {
                if project.wires.iter().any(|item| item.id == wire.id) {
                    return Err(format!("wire {} already exists", wire.id.0));
                }
                project.wires.push(wire.clone());
            }
            FaultRepair::RemoveWire { wire } => {
                let before = project.wires.len();
                project.wires.retain(|item| item.id != *wire);
                if before == project.wires.len() {
                    return Err(format!("unknown wire {}", wire.0));
                }
            }
        }
    }
    Ok(())
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
        if let Some(result) = state.last_valid.clone() {
            record_piezo_currents(project, state, &result);
        }
        state.step += 1;
        return true;
    }
    match crate::solver::solve_transient_with_digital_states(
        project,
        &state.controls,
        &state.capacitor_voltages,
        &state.control_ratios,
        &state.digital_states,
        max_iterations,
    ) {
        Ok(mut result) => {
            update_digital_states(project, &mut state.digital_states, &result);
            if !state.digital_states.is_empty() {
                result = match crate::solver::solve_transient_with_digital_states(
                    project,
                    &state.controls,
                    &state.capacitor_voltages,
                    &state.control_ratios,
                    &state.digital_states,
                    max_iterations,
                ) {
                    Ok(result) => result,
                    Err(error) => {
                        state.running = false;
                        state.stale = true;
                        state.diagnostics = match error {
                            ElectricalError::Structure(errors) => {
                                errors.into_iter().map(structural_diagnostic).collect()
                            }
                            ElectricalError::Calculation(e) => vec![electrical_diagnostic(e)],
                        };
                        return false;
                    }
                };
            }
            state
                .capacitor_voltages
                .extend(result.capacitor_voltages.clone());
            record_piezo_currents(project, state, &result);
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

fn record_piezo_currents(project: &Project, state: &mut SimulationState, result: &SolveResult) {
    for component in project
        .components
        .iter()
        .filter(|component| component.kind == ComponentKind::PiezoPassive)
    {
        let history = state
            .piezo_current_history
            .entry(component.id.clone())
            .or_default();
        history.push(
            result
                .resistor_currents
                .get(&component.id)
                .copied()
                .unwrap_or(0.0),
        );
        if history.len() > PASSIVE_PIEZO_HISTORY_STEPS {
            history.remove(0);
        }
    }
}

fn update_digital_states(
    project: &Project,
    states: &mut BTreeMap<ComponentId, u32>,
    result: &SolveResult,
) {
    let voltage = |id: &ComponentId, pin: &str| {
        result
            .node_voltages
            .iter()
            .find(|node| {
                node.contacts.contains(&crate::Contact::ComponentPin(
                    id.clone(),
                    crate::PinId(pin.into()),
                ))
            })
            .map_or(0.0, |node| node.voltage)
    };
    for component in &project.components {
        let entry = match component.kind {
            ComponentKind::DFlipFlop
            | ComponentKind::DigitalCounter
            | ComponentKind::ShiftRegister
            | ComponentKind::StepSequencer
            | ComponentKind::Sram => states.entry(component.id.clone()).or_default(),
            _ => continue,
        };
        let supply = voltage(&component.id, "vcc").max(1e-6);
        let clock_high = voltage(&component.id, "clock") > supply * 0.5;
        let old_clock_high = *entry & (1 << 16) != 0;
        let rising = clock_high && !old_clock_high;
        match component.kind {
            ComponentKind::DFlipFlop => {
                let reset_high = voltage(&component.id, "reset") > supply * 0.5;
                let set_high = voltage(&component.id, "set") > supply * 0.5;
                let mut q_high = *entry & 1 != 0;
                if reset_high {
                    q_high = false;
                } else if set_high {
                    q_high = true;
                } else if rising {
                    q_high = voltage(&component.id, "data") > supply * 0.5;
                }
                *entry = u32::from(q_high) | u32::from(clock_high) << 16;
            }
            ComponentKind::DigitalCounter => {
                let reset_high = voltage(&component.id, "reset") > supply * 0.5;
                if reset_high {
                    *entry = u32::from(clock_high) << 16;
                } else if rising && voltage(&component.id, "enable") > supply * 0.5 {
                    let modulus = component.parameters["modulus"] as u32;
                    let value = *entry & 0x3ff;
                    *entry = (if value + 1 >= modulus { 0 } else { value + 1 }) | (1 << 16);
                } else {
                    *entry = (*entry & 0x3ff) | u32::from(clock_high) << 16;
                }
            }
            ComponentKind::ShiftRegister => {
                let old_latch_high = *entry & (1 << 17) != 0;
                let latch_high = voltage(&component.id, "latch") > supply * 0.5;
                let clear_high = voltage(&component.id, "clear") > supply * 0.5;
                let mut value = *entry & 0xffff;
                if !clear_high {
                    value = 0;
                } else if rising {
                    let data = u32::from(voltage(&component.id, "data") > supply * 0.5);
                    value = ((value << 1) | data) & 0xff | (value & 0xff00);
                }
                if latch_high && !old_latch_high {
                    value = (value & 0xff) | ((value & 0xff) << 8);
                }
                value = (value & !(1 << 16 | 1 << 17))
                    | u32::from(clock_high) << 16
                    | u32::from(latch_high) << 17;
                *entry = value;
            }
            ComponentKind::StepSequencer => {
                let stage = *entry & 0xff;
                let next = if rising { (stage + 1) % 8 } else { stage };
                *entry = next | u32::from(clock_high) << 16;
            }
            ComponentKind::Sram => {
                let reset_high = voltage(&component.id, "reset") > supply * 0.5;
                let address = u32::from(voltage(&component.id, "address") > supply * 0.5);
                let mut memory = *entry & 0xffff;
                if reset_high {
                    memory = 0;
                } else if rising && voltage(&component.id, "write") > supply * 0.5 {
                    let value = (0..8).fold(0, |value, bit| {
                        value
                            | u32::from(
                                voltage(&component.id, &format!("data{bit}")) > supply * 0.5,
                            ) << bit
                    });
                    let mask = 0xff << (address * 8);
                    memory = (memory & !mask) | (value << (address * 8));
                }
                *entry = memory | u32::from(clock_high) << 16;
            }
            _ => {}
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

    fn counter_project() -> Project {
        let pins = |pairs: &[(&str, &str)]| {
            pairs
                .iter()
                .map(|(pin, hole)| (crate::PinId((*pin).into()), crate::HoleId((*hole).into())))
                .collect()
        };
        let mut counter_pins = vec![
            ("clock", "A1"),
            ("carry", "B1"),
            ("enable", "TP+:3"),
            ("gnd", "TP-:3"),
            ("reset", "TP-:4"),
            ("vcc", "TP+:4"),
        ];
        counter_pins.extend([
            ("q0", "C1"),
            ("q1", "C2"),
            ("q2", "C3"),
            ("q3", "C4"),
            ("q4", "C5"),
            ("q5", "C6"),
            ("q6", "C7"),
            ("q7", "C8"),
            ("q8", "C9"),
            ("q9", "C10"),
        ]);
        Project {
            format_version: 1,
            title: "counter test".into(),
            board: crate::Board {
                model: crate::BoardModel::HalfSizeSolderless,
            },
            components: vec![
                crate::Component {
                    id: ComponentId("V1".into()),
                    kind: ComponentKind::DcVoltageSource,
                    pins: pins(&[("positive", "TP+:1"), ("negative", "TP-:1")]),
                    parameters: BTreeMap::from([("voltage".into(), 5.0)]),
                    ic_device: None,
                    other_device: None,
                    module: None,
                    diode_model: None,
                },
                crate::Component {
                    id: ComponentId("S1".into()),
                    kind: ComponentKind::MomentaryButton,
                    pins: pins(&[("a", "TP+:2"), ("b", "A1")]),
                    parameters: BTreeMap::new(),
                    ic_device: None,
                    other_device: None,
                    module: None,
                    diode_model: None,
                },
                crate::Component {
                    id: ComponentId("R1".into()),
                    kind: ComponentKind::Resistor,
                    pins: pins(&[("a", "A1"), ("b", "TP-:2")]),
                    parameters: BTreeMap::from([("resistance".into(), 10_000.0)]),
                    ic_device: None,
                    other_device: None,
                    module: None,
                    diode_model: None,
                },
                crate::Component {
                    id: ComponentId("U1".into()),
                    kind: ComponentKind::DigitalCounter,
                    pins: pins(&counter_pins),
                    parameters: BTreeMap::from([
                        ("modulus".into(), 10.0),
                        ("output_mode".into(), 0.0),
                        ("output_resistance".into(), 100.0),
                    ]),
                    ic_device: None,
                    other_device: None,
                    module: None,
                    diode_model: None,
                },
            ],
            wires: Vec::new(),
            faults: Vec::new(),
            initial_conditions: crate::InitialConditions {
                controls: BTreeMap::from([(
                    ComponentId("S1".into()),
                    ControlState::ButtonReleased,
                )]),
                ..crate::InitialConditions::default()
            },
        }
    }

    fn d_flip_flop_project() -> Project {
        let pins = |pairs: &[(&str, &str)]| {
            pairs
                .iter()
                .map(|(pin, hole)| (crate::PinId((*pin).into()), crate::HoleId((*hole).into())))
                .collect()
        };
        Project {
            format_version: 1,
            title: "D flip-flop test".into(),
            board: crate::Board {
                model: crate::BoardModel::HalfSizeSolderless,
            },
            components: vec![
                crate::Component {
                    id: ComponentId("V1".into()),
                    kind: ComponentKind::DcVoltageSource,
                    pins: pins(&[("positive", "TP+:1"), ("negative", "TP-:1")]),
                    parameters: BTreeMap::from([("voltage".into(), 5.0)]),
                    ic_device: None,
                    other_device: None,
                    module: None,
                    diode_model: None,
                },
                crate::Component {
                    id: ComponentId("S_DATA".into()),
                    kind: ComponentKind::MomentaryButton,
                    pins: pins(&[("a", "TP+:2"), ("b", "A1")]),
                    parameters: BTreeMap::new(),
                    ic_device: None,
                    other_device: None,
                    module: None,
                    diode_model: None,
                },
                crate::Component {
                    id: ComponentId("R_DATA".into()),
                    kind: ComponentKind::Resistor,
                    pins: pins(&[("a", "A1"), ("b", "TP-:2")]),
                    parameters: BTreeMap::from([("resistance".into(), 10_000.0)]),
                    ic_device: None,
                    other_device: None,
                    module: None,
                    diode_model: None,
                },
                crate::Component {
                    id: ComponentId("S_CLOCK".into()),
                    kind: ComponentKind::MomentaryButton,
                    pins: pins(&[("a", "TP+:3"), ("b", "A2")]),
                    parameters: BTreeMap::new(),
                    ic_device: None,
                    other_device: None,
                    module: None,
                    diode_model: None,
                },
                crate::Component {
                    id: ComponentId("R_CLOCK".into()),
                    kind: ComponentKind::Resistor,
                    pins: pins(&[("a", "A2"), ("b", "TP-:3")]),
                    parameters: BTreeMap::from([("resistance".into(), 10_000.0)]),
                    ic_device: None,
                    other_device: None,
                    module: None,
                    diode_model: None,
                },
                crate::Component {
                    id: ComponentId("U1".into()),
                    kind: ComponentKind::DFlipFlop,
                    pins: pins(&[
                        ("clock", "A2"),
                        ("data", "A1"),
                        ("gnd", "TP-:4"),
                        ("not_q", "B10"),
                        ("q", "B11"),
                        ("reset", "TP-:4"),
                        ("set", "TP-:5"),
                        ("vcc", "TP+:4"),
                    ]),
                    parameters: BTreeMap::from([("output_resistance".into(), 100.0)]),
                    ic_device: None,
                    other_device: None,
                    module: None,
                    diode_model: None,
                },
            ],
            wires: Vec::new(),
            faults: Vec::new(),
            initial_conditions: crate::InitialConditions::default(),
        }
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
    fn digital_counter_advances_once_per_calculated_rising_edge() {
        let baseline = counter_project();
        let mut project = baseline.clone();
        let mut state = SimulationState::new(&project);
        apply_actions(&mut project, &baseline, &mut state, &[Action::Run]);
        advance_steps(&project, &mut state, 1);
        assert_eq!(state.digital_states[&ComponentId("U1".into())] & 0x3ff, 0);
        apply_actions(
            &mut project,
            &baseline,
            &mut state,
            &[
                Action::SetControl {
                    component: ComponentId("S1".into()),
                    state: ControlState::ButtonPressed,
                },
                Action::SingleStep,
            ],
        );
        let clock = state
            .last_valid
            .as_ref()
            .unwrap()
            .node_voltages
            .iter()
            .find(|node| {
                node.contacts.contains(&crate::Contact::ComponentPin(
                    ComponentId("U1".into()),
                    crate::PinId("clock".into()),
                ))
            })
            .unwrap()
            .voltage;
        assert!(clock > 4.0);
        assert_eq!(state.digital_states[&ComponentId("U1".into())] & 0x3ff, 1);
        let q0 = state
            .last_valid
            .as_ref()
            .unwrap()
            .node_voltages
            .iter()
            .find(|node| {
                node.contacts.contains(&crate::Contact::ComponentPin(
                    ComponentId("U1".into()),
                    crate::PinId("q0".into()),
                ))
            })
            .unwrap()
            .voltage;
        assert!(q0 > 4.0);
        apply_actions(
            &mut project,
            &baseline,
            &mut state,
            &[Action::SetControl {
                component: ComponentId("S1".into()),
                state: ControlState::ButtonReleased,
            }],
        );
        advance_steps(&project, &mut state, 1);
        assert_eq!(state.digital_states[&ComponentId("U1".into())] & 0x3ff, 1);
    }

    #[test]
    fn d_flip_flop_captures_data_only_on_a_calculated_rising_edge() {
        let baseline = d_flip_flop_project();
        let mut project = baseline.clone();
        let mut state = SimulationState::new(&project);
        let control = |component: &str, control| Action::SetControl {
            component: ComponentId(component.into()),
            state: control,
        };

        apply_actions(&mut project, &baseline, &mut state, &[Action::SingleStep]);
        assert_eq!(state.digital_states[&ComponentId("U1".into())] & 1, 0);

        apply_actions(
            &mut project,
            &baseline,
            &mut state,
            &[
                control("S_DATA", ControlState::ButtonPressed),
                Action::SingleStep,
            ],
        );
        assert_eq!(state.digital_states[&ComponentId("U1".into())] & 1, 0);

        apply_actions(
            &mut project,
            &baseline,
            &mut state,
            &[
                control("S_CLOCK", ControlState::ButtonPressed),
                Action::SingleStep,
            ],
        );
        assert_eq!(state.digital_states[&ComponentId("U1".into())] & 1, 1);

        apply_actions(
            &mut project,
            &baseline,
            &mut state,
            &[
                control("S_DATA", ControlState::ButtonReleased),
                Action::SingleStep,
            ],
        );
        assert_eq!(state.digital_states[&ComponentId("U1".into())] & 1, 1);

        apply_actions(
            &mut project,
            &baseline,
            &mut state,
            &[
                control("S_CLOCK", ControlState::ButtonReleased),
                Action::SingleStep,
            ],
        );
        apply_actions(
            &mut project,
            &baseline,
            &mut state,
            &[
                control("S_CLOCK", ControlState::ButtonPressed),
                Action::SingleStep,
            ],
        );
        assert_eq!(state.digital_states[&ComponentId("U1".into())] & 1, 0);
    }

    #[test]
    fn c03_code_lock_requires_ordered_edges_and_calculates_reset_and_unlock_outputs() {
        let baseline: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/c03-s05-04-code-lock.json"
        ))
        .unwrap();
        let mut project = baseline.clone();
        let mut state = SimulationState::new(&project);
        let pulse = |project: &mut Project, state: &mut SimulationState, id: &str| {
            apply_actions(
                project,
                &baseline,
                state,
                &[
                    Action::SetControl {
                        component: ComponentId(id.into()),
                        state: ControlState::ButtonPressed,
                    },
                    Action::SingleStep,
                    Action::SetControl {
                        component: ComponentId(id.into()),
                        state: ControlState::ButtonReleased,
                    },
                    Action::SingleStep,
                ],
            );
        };

        apply_actions(&mut project, &baseline, &mut state, &[Action::SingleStep]);
        pulse(&mut project, &mut state, "C3");
        assert_eq!(state.digital_states[&ComponentId("U1".into())] & 1, 0);
        assert_eq!(state.digital_states[&ComponentId("U2".into())] & 1, 0);
        assert_eq!(state.digital_states[&ComponentId("U3".into())] & 1, 0);
        assert_eq!(state.digital_states[&ComponentId("U4".into())] & 1, 0);

        for id in ["C1", "C2", "C3", "C4"] {
            pulse(&mut project, &mut state, id);
        }
        assert_eq!(state.digital_states[&ComponentId("U1".into())] & 1, 1);
        assert_eq!(state.digital_states[&ComponentId("U2".into())] & 1, 1);
        assert_eq!(state.digital_states[&ComponentId("U3".into())] & 1, 1);
        assert_eq!(state.digital_states[&ComponentId("U4".into())] & 1, 1);
        assert!(
            state.last_valid.as_ref().unwrap().led_currents[&ComponentId("LED_GREEN".into())]
                > 0.001
        );

        apply_actions(
            &mut project,
            &baseline,
            &mut state,
            &[
                Action::SetControl {
                    component: ComponentId("RST1".into()),
                    state: ControlState::ButtonPressed,
                },
                Action::SingleStep,
            ],
        );
        for id in ["U1", "U2", "U3", "U4"] {
            assert_eq!(state.digital_states[&ComponentId(id.into())] & 1, 0);
        }
        assert!(
            state.last_valid.as_ref().unwrap().led_currents[&ComponentId("LED_RED".into())] > 0.001
        );
    }

    #[test]
    fn c03_reaction_game_latches_the_first_player_and_resets_both_outputs() {
        let baseline: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/c03-s04-06-reaction-game.json"
        ))
        .unwrap();
        let mut project = baseline.clone();
        let mut state = SimulationState::new(&project);
        let pulse = |project: &mut Project, state: &mut SimulationState, id: &str| {
            apply_actions(
                project,
                &baseline,
                state,
                &[
                    Action::SetControl {
                        component: ComponentId(id.into()),
                        state: ControlState::ButtonPressed,
                    },
                    Action::SingleStep,
                    Action::SetControl {
                        component: ComponentId(id.into()),
                        state: ControlState::ButtonReleased,
                    },
                    Action::SingleStep,
                ],
            );
        };

        apply_actions(&mut project, &baseline, &mut state, &[Action::SingleStep]);
        pulse(&mut project, &mut state, "P_GREEN");
        pulse(&mut project, &mut state, "P_RED");
        assert_eq!(state.digital_states[&ComponentId("U_RED".into())] & 1, 0);
        assert_eq!(state.digital_states[&ComponentId("U_GREEN".into())] & 1, 1);
        let result = state.last_valid.as_ref().unwrap();
        assert!(result.led_currents[&ComponentId("LED_GREEN".into())] > 0.001);
        assert!(result.led_currents[&ComponentId("LED_RED".into())] < 1e-6);
        assert!(result.resistor_currents[&ComponentId("BZ1".into())] > 0.001);

        pulse(&mut project, &mut state, "RESET");
        assert_eq!(state.digital_states[&ComponentId("U_RED".into())] & 1, 0);
        assert_eq!(state.digital_states[&ComponentId("U_GREEN".into())] & 1, 0);
        let result = state.last_valid.as_ref().unwrap();
        assert!(result.led_currents[&ComponentId("LED_GREEN".into())] < 1e-6);
        assert!(result.resistor_currents[&ComponentId("BZ1".into())] < 1e-6);
    }

    #[test]
    fn c03_level_indicator_calculates_segment_count_from_input_ratio() {
        let baseline: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/c03-s04-07-level-indicator.json"
        ))
        .unwrap();
        let mut project = baseline.clone();
        let mut state = SimulationState::new(&project);
        apply_actions(&mut project, &baseline, &mut state, &[Action::SingleStep]);
        let dark = state.last_valid.as_ref().unwrap();
        assert!(dark.led_currents.values().all(|current| *current < 1e-6));

        apply_actions(
            &mut project,
            &baseline,
            &mut state,
            &[
                Action::SetControlRatio {
                    component: ComponentId("PHOTO1".into()),
                    ratio: 1.0,
                },
                Action::SingleStep,
            ],
        );
        let bright = state.last_valid.as_ref().unwrap();
        assert!(
            (0..9).all(|index| bright.led_currents[&ComponentId(format!("LED{index}"))] > 0.001)
        );
        assert!(bright.led_currents[&ComponentId("LED9".into())] < 1e-6);
    }

    #[test]
    fn c03_audio_amplifier_transfers_calculated_input_level_to_speaker_load() {
        let baseline: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/c03-s04-08-audio-amplifier.json"
        ))
        .unwrap();
        let mut project = baseline.clone();
        let mut state = SimulationState::new(&project);
        apply_actions(&mut project, &baseline, &mut state, &[Action::SingleStep]);
        let loud =
            state.last_valid.as_ref().unwrap().resistor_currents[&ComponentId("SPK1".into())];
        apply_actions(
            &mut project,
            &baseline,
            &mut state,
            &[
                Action::SetControlRatio {
                    component: ComponentId("POT1".into()),
                    ratio: 1.0,
                },
                Action::SingleStep,
            ],
        );
        let quiet =
            state.last_valid.as_ref().unwrap().resistor_currents[&ComponentId("SPK1".into())];
        assert!(loud > quiet);
        assert!(quiet > 0.0);
    }

    #[test]
    fn c03_counter_fixture_drives_a_calculated_display_digit() {
        let baseline: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/c03-s04-03-button-counter.json"
        ))
        .unwrap();
        let mut project = baseline.clone();
        let mut state = SimulationState::new(&project);
        apply_actions(&mut project, &baseline, &mut state, &[Action::Run]);
        advance_steps(&project, &mut state, 1);
        apply_actions(
            &mut project,
            &baseline,
            &mut state,
            &[
                Action::SetControl {
                    component: ComponentId("S1".into()),
                    state: ControlState::ButtonPressed,
                },
                Action::SingleStep,
            ],
        );
        let clock = state
            .last_valid
            .as_ref()
            .unwrap()
            .node_voltages
            .iter()
            .find(|node| {
                node.contacts.contains(&crate::Contact::ComponentPin(
                    ComponentId("U1".into()),
                    crate::PinId("clock".into()),
                ))
            })
            .unwrap()
            .voltage;
        assert!(clock > 4.0);
        assert_eq!(state.digital_states[&ComponentId("U1".into())] & 0x3ff, 1);
        let segment_a = state
            .last_valid
            .as_ref()
            .unwrap()
            .node_voltages
            .iter()
            .find(|node| {
                node.contacts.contains(&crate::Contact::ComponentPin(
                    ComponentId("DISP1".into()),
                    crate::PinId("a".into()),
                ))
            })
            .unwrap()
            .voltage;
        let segment_f = state
            .last_valid
            .as_ref()
            .unwrap()
            .node_voltages
            .iter()
            .find(|node| {
                node.contacts.contains(&crate::Contact::ComponentPin(
                    ComponentId("DISP1".into()),
                    crate::PinId("f".into()),
                ))
            })
            .unwrap()
            .voltage;
        assert!(segment_a < 1.0);
        assert!(segment_f < 1.0);
    }

    #[test]
    fn c03_dice_counter_calculates_the_one_dot_pattern() {
        let baseline: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/c03-s04-02-electronic-dice.json"
        ))
        .unwrap();
        let mut project = baseline.clone();
        let mut state = SimulationState::new(&project);
        apply_actions(&mut project, &baseline, &mut state, &[Action::Run]);
        advance_steps(&project, &mut state, 1);
        apply_actions(
            &mut project,
            &baseline,
            &mut state,
            &[
                Action::SetControl {
                    component: ComponentId("S1".into()),
                    state: ControlState::ButtonPressed,
                },
                Action::SingleStep,
            ],
        );
        assert_eq!(state.digital_states[&ComponentId("U1".into())] & 0x3ff, 1);
        let solved = state.last_valid.as_ref().unwrap();
        let pin_voltage = |pin: &str| {
            solved
                .node_voltages
                .iter()
                .find(|node| {
                    node.contacts.contains(&crate::Contact::ComponentPin(
                        ComponentId("U1".into()),
                        crate::PinId(pin.into()),
                    ))
                })
                .unwrap()
                .voltage
        };
        assert!(pin_voltage("q3") > 3.0);
        assert!(pin_voltage("q0") < 1.0);
        assert!(pin_voltage("q6") < 1.0);
    }

    #[test]
    fn c03_four_bit_adder_calculates_five_plus_nine() {
        let baseline: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/c03-s05-01-four-bit-adder.json"
        ))
        .unwrap();
        let mut project = baseline.clone();
        let mut state = SimulationState::new(&project);
        let pressed = ["SA0", "SA2", "SB0", "SB3"];
        let mut actions = pressed
            .iter()
            .map(|id| Action::SetControl {
                component: ComponentId((*id).into()),
                state: ControlState::SwitchNormallyOpen,
            })
            .collect::<Vec<_>>();
        actions.push(Action::SingleStep);
        apply_actions(&mut project, &baseline, &mut state, &actions);
        let solved = state.last_valid.as_ref().unwrap();
        let pin_voltage = |pin: &str| {
            solved
                .node_voltages
                .iter()
                .find(|node| {
                    node.contacts.contains(&crate::Contact::ComponentPin(
                        ComponentId("U1".into()),
                        crate::PinId(pin.into()),
                    ))
                })
                .unwrap()
                .voltage
        };
        assert!(pin_voltage("sum0") < 1.0);
        assert!(pin_voltage("sum1") > 3.0);
        assert!(pin_voltage("sum2") > 3.0);
        assert!(pin_voltage("sum3") > 3.0);
        assert!(pin_voltage("carry_out") < 1.0);
    }

    #[test]
    fn c03_four_bit_subtractor_calculates_nine_minus_five() {
        let baseline: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/c03-s05-02-four-bit-subtractor.json"
        ))
        .unwrap();
        let mut project = baseline.clone();
        let mut state = SimulationState::new(&project);
        apply_actions(&mut project, &baseline, &mut state, &[Action::SingleStep]);
        let solved = state.last_valid.as_ref().unwrap();
        let pin_voltage = |pin: &str| {
            solved
                .node_voltages
                .iter()
                .find(|node| {
                    node.contacts.contains(&crate::Contact::ComponentPin(
                        ComponentId("U1".into()),
                        crate::PinId(pin.into()),
                    ))
                })
                .unwrap()
                .voltage
        };
        assert!(pin_voltage("sum0") < 1.0);
        assert!(pin_voltage("sum1") < 1.0);
        assert!(pin_voltage("sum2") > 3.0);
        assert!(pin_voltage("sum3") < 1.0);
        assert!(pin_voltage("carry_out") > 3.0);
    }

    #[test]
    fn c03_pedestrian_signal_advances_calculated_phases() {
        let baseline: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/c03-s05-03-pedestrian-signal.json"
        ))
        .unwrap();
        let mut project = baseline.clone();
        let mut state = SimulationState::new(&project);
        advance_steps(&project, &mut state, 1);
        for _ in 0..4 {
            apply_actions(
                &mut project,
                &baseline,
                &mut state,
                &[
                    Action::SetControl {
                        component: ComponentId("S1".into()),
                        state: ControlState::ButtonPressed,
                    },
                    Action::SingleStep,
                    Action::SetControl {
                        component: ComponentId("S1".into()),
                        state: ControlState::ButtonReleased,
                    },
                    Action::SingleStep,
                ],
            );
        }
        assert_eq!(state.digital_states[&ComponentId("U1".into())] & 0x3ff, 4);
    }

    #[test]
    fn c03_stopwatch_cascades_clock_carry_and_reset() {
        let baseline: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/c03-s05-05-digital-stopwatch.json"
        ))
        .unwrap();
        let mut project = baseline.clone();
        let mut state = SimulationState::new(&project);
        apply_actions(
            &mut project,
            &baseline,
            &mut state,
            &[
                Action::SetControl {
                    component: ComponentId("RUN".into()),
                    state: ControlState::SwitchNormallyOpen,
                },
                Action::Run,
            ],
        );
        for _ in 0..12 {
            apply_actions(
                &mut project,
                &baseline,
                &mut state,
                &[
                    Action::SetControl {
                        component: ComponentId("STEP".into()),
                        state: ControlState::ButtonPressed,
                    },
                    Action::SingleStep,
                    Action::SetControl {
                        component: ComponentId("STEP".into()),
                        state: ControlState::ButtonReleased,
                    },
                    Action::SingleStep,
                ],
            );
        }

        let first = state.digital_states[&ComponentId("U1".into())] & 0x3ff;
        let second = state.digital_states[&ComponentId("U2".into())] & 0x3ff;
        assert!(
            state.last_valid.is_some(),
            "stopwatch solve failed: states={:?} diagnostics={:?}",
            state.digital_states,
            state.diagnostics
        );
        let u1_voltage = |pin: &str| {
            state
                .last_valid
                .as_ref()
                .unwrap()
                .node_voltages
                .iter()
                .find(|node| {
                    node.contacts.contains(&crate::Contact::ComponentPin(
                        ComponentId("U1".into()),
                        crate::PinId(pin.into()),
                    ))
                })
                .map(|node| node.voltage)
                .unwrap_or(-1.0)
        };
        assert!(
            first > 0,
            "step input must advance the first decimal stage: states={:?} controls={:?} diagnostics={:?} clock={} enable={} reset={}",
            state.digital_states,
            state.controls,
            state.diagnostics,
            u1_voltage("clock"),
            u1_voltage("enable"),
            u1_voltage("reset")
        );
        assert!(second > 0, "carry must advance the second decimal stage");

        apply_actions(&mut project, &baseline, &mut state, &[Action::Reset]);
        assert_eq!(state.digital_states[&ComponentId("U1".into())] & 0x3ff, 0);
        assert_eq!(state.digital_states[&ComponentId("U2".into())] & 0x3ff, 0);
        assert_eq!(state.digital_states[&ComponentId("U3".into())] & 0x3ff, 0);
        assert_eq!(state.digital_states[&ComponentId("U4".into())] & 0x3ff, 0);
    }

    #[test]
    fn c03_clock_core_cascades_six_calculated_stages() {
        let baseline: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/c03-s05-06-digital-clock.json"
        ))
        .unwrap();
        let mut project = baseline.clone();
        let mut state = SimulationState::new(&project);
        apply_actions(
            &mut project,
            &baseline,
            &mut state,
            &[
                Action::SetControl {
                    component: ComponentId("RUN".into()),
                    state: ControlState::SwitchNormallyOpen,
                },
                Action::Run,
            ],
        );
        for _ in 0..12 {
            apply_actions(
                &mut project,
                &baseline,
                &mut state,
                &[
                    Action::SetControl {
                        component: ComponentId("STEP".into()),
                        state: ControlState::ButtonPressed,
                    },
                    Action::SingleStep,
                    Action::SetControl {
                        component: ComponentId("STEP".into()),
                        state: ControlState::ButtonReleased,
                    },
                    Action::SingleStep,
                ],
            );
        }
        assert_eq!(state.digital_states[&ComponentId("U1".into())] & 0x3ff, 2);
        assert_eq!(state.digital_states[&ComponentId("U2".into())] & 0x3ff, 1);
        assert_eq!(state.digital_states[&ComponentId("U3".into())] & 0x3ff, 0);
        assert_eq!(state.digital_states[&ComponentId("U4".into())] & 0x3ff, 0);
        assert_eq!(state.digital_states[&ComponentId("U5".into())] & 0x3ff, 0);
        assert_eq!(state.digital_states[&ComponentId("U6".into())] & 0x3ff, 0);
        apply_actions(&mut project, &baseline, &mut state, &[Action::Reset]);
        assert!(
            state
                .digital_states
                .values()
                .all(|value| value & 0x3ff == 0)
        );
    }

    #[test]
    fn c03_step_sequencer_advances_led_and_control_selection() {
        let baseline: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/c03-s05-07-step-sequencer.json"
        ))
        .unwrap();
        let mut project = baseline.clone();
        let mut state = SimulationState::new(&project);
        apply_actions(
            &mut project,
            &baseline,
            &mut state,
            &[
                Action::SetControl {
                    component: ComponentId("STEP".into()),
                    state: ControlState::ButtonPressed,
                },
                Action::SingleStep,
            ],
        );
        assert_eq!(
            state.digital_states[&ComponentId("SEQ1".into())] & 0xff,
            1,
            "states={:?} diagnostics={:?} clock_voltage={:?}",
            state.digital_states,
            state.diagnostics,
            state.last_valid.as_ref().and_then(|result| {
                result.node_voltages.iter().find_map(|node| {
                    node.contacts
                        .contains(&crate::Contact::ComponentPin(
                            ComponentId("SEQ1".into()),
                            crate::PinId("clock".into()),
                        ))
                        .then_some(node.voltage)
                })
            })
        );
        let solved = state.last_valid.as_ref().unwrap();
        assert!(solved.led_currents[&ComponentId("LED2".into())] > 0.001);
        assert!(solved.led_currents[&ComponentId("LED1".into())] < 1e-6);

        for _ in 0..7 {
            apply_actions(
                &mut project,
                &baseline,
                &mut state,
                &[
                    Action::SetControl {
                        component: ComponentId("STEP".into()),
                        state: ControlState::ButtonReleased,
                    },
                    Action::SingleStep,
                    Action::SetControl {
                        component: ComponentId("STEP".into()),
                        state: ControlState::ButtonPressed,
                    },
                    Action::SingleStep,
                ],
            );
        }
        assert_eq!(state.digital_states[&ComponentId("SEQ1".into())] & 0xff, 0);
    }

    #[test]
    fn c03_bounded_sram_writes_and_reads_a_calculated_byte() {
        let baseline: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/c03-s05-08-bounded-sram.json"
        ))
        .unwrap();
        let mut project = baseline.clone();
        let mut state = SimulationState::new(&project);
        apply_actions(
            &mut project,
            &baseline,
            &mut state,
            &[
                Action::SetControl {
                    component: ComponentId("D0".into()),
                    state: ControlState::SwitchNormallyOpen,
                },
                Action::SetControl {
                    component: ComponentId("D2".into()),
                    state: ControlState::SwitchNormallyOpen,
                },
                Action::SetControl {
                    component: ComponentId("WRITE".into()),
                    state: ControlState::ButtonPressed,
                },
                Action::SetControl {
                    component: ComponentId("CLOCK".into()),
                    state: ControlState::ButtonPressed,
                },
                Action::SingleStep,
            ],
        );
        assert_eq!(
            state.digital_states[&ComponentId("U1".into())] & 0xffff,
            0b0000_0101,
            "states={:?} diagnostics={:?} readings={:?}",
            state.digital_states,
            state.diagnostics,
            state.last_valid
        );
        let solved = state.last_valid.as_ref().unwrap();
        assert!(solved.led_currents[&ComponentId("LED0".into())] > 0.001);
        assert!(solved.led_currents[&ComponentId("LED2".into())] > 0.001);
        assert!(solved.led_currents[&ComponentId("LED1".into())] < 1e-6);
    }

    #[test]
    fn c04_metronome_calculates_led_and_speaker_pulses() {
        let baseline: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/c04-s06-04-metronome.json"
        ))
        .unwrap();
        let mut project = baseline.clone();
        let mut state = SimulationState::new(&project);
        apply_actions(&mut project, &baseline, &mut state, &[Action::Run]);
        let mut max_led = 0.0_f64;
        let mut max_speaker = 0.0_f64;
        for _ in 0..4_000 {
            advance_steps(&project, &mut state, 1);
            let solved = state.last_valid.as_ref().unwrap();
            max_led = max_led.max(solved.led_currents[&ComponentId("LED1".into())]);
            max_speaker = max_speaker.max(
                solved
                    .resistor_currents
                    .get(&ComponentId("SP1".into()))
                    .copied()
                    .unwrap_or(0.0)
                    .abs(),
            );
        }
        assert!(!state.stale, "diagnostics={:?}", state.diagnostics);
        assert!(max_led > 0.001);
        assert!(max_speaker > 0.004);
    }

    #[test]
    fn c04_cricket_composes_two_calculated_timer_stages() {
        let baseline: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/c04-s06-02-cricket.json"
        ))
        .unwrap();
        let mut project = baseline.clone();
        let mut state = SimulationState::new(&project);
        apply_actions(&mut project, &baseline, &mut state, &[Action::Run]);
        let mut max_speaker = 0.0_f64;
        for _ in 0..4_000 {
            advance_steps(&project, &mut state, 1);
            let solved = state.last_valid.as_ref().unwrap();
            max_speaker = max_speaker.max(
                solved
                    .resistor_currents
                    .get(&ComponentId("SP1".into()))
                    .copied()
                    .unwrap_or(0.0)
                    .abs(),
            );
        }
        assert!(!state.stale, "diagnostics={:?}", state.diagnostics);
        assert!(max_speaker > 0.004);
    }

    #[test]
    fn passive_piezo_steady_dc_stays_silent_even_above_threshold() {
        let baseline: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/e7-buzzer-doorbell.json"
        ))
        .unwrap();
        let mut project = baseline.clone();
        project
            .components
            .iter_mut()
            .find(|component| component.id == ComponentId("BZ1".into()))
            .unwrap()
            .kind = ComponentKind::PiezoPassive;
        let mut state = SimulationState::new(&project);
        apply_actions(
            &mut project,
            &baseline,
            &mut state,
            &[
                Action::SetControl {
                    component: ComponentId("S1".into()),
                    state: ControlState::ButtonPressed,
                },
                Action::Run,
            ],
        );
        advance_steps(&project, &mut state, PASSIVE_PIEZO_HISTORY_STEPS as u64 + 4);
        assert!(
            state.last_valid.as_ref().unwrap().resistor_currents[&ComponentId("BZ1".into())].abs()
                > crate::SOUNDING_CURRENT
        );
        assert!(!state.passive_piezo_sounding(&ComponentId("BZ1".into())));
    }

    #[test]
    fn passive_piezo_detects_the_expected_fixed_step_square_wave() {
        assert!(!passive_piezo_is_sounding(&[0.0, 0.0, 0.005]));
        assert!(passive_piezo_is_sounding(&[
            0.0, 0.0, 0.005, 0.005, 0.0, 0.0, 0.005,
        ]));
        assert!(passive_piezo_is_sounding(&[0.005, -0.005, 0.005]));
        assert!(!passive_piezo_is_sounding(
            &[0.02; PASSIVE_PIEZO_HISTORY_STEPS]
        ));
    }

    #[test]
    fn passive_piezo_timer_555_is_sounding_from_calculated_drive() {
        let baseline: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/c02-s03-01-555-flasher.json"
        ))
        .unwrap();
        let mut project = baseline.clone();
        project
            .components
            .iter_mut()
            .find(|component| component.id == ComponentId("R3".into()))
            .unwrap()
            .clone_from(&crate::Component {
                id: ComponentId("R3".into()),
                kind: ComponentKind::PiezoPassive,
                pins: BTreeMap::from([
                    (crate::PinId("positive".into()), crate::HoleId("G5".into())),
                    (crate::PinId("negative".into()), crate::HoleId("G8".into())),
                ]),
                parameters: BTreeMap::from([("resistance".into(), 470.0)]),
                ic_device: None,
                other_device: None,
                module: None,
                diode_model: None,
            });
        project
            .components
            .iter_mut()
            .find(|component| component.id == ComponentId("R2".into()))
            .unwrap()
            .parameters
            .insert("resistance".into(), 1_000.0);
        project
            .components
            .iter_mut()
            .find(|component| component.id == ComponentId("C1".into()))
            .unwrap()
            .parameters
            .insert("capacitance".into(), 1e-6);
        project
            .components
            .retain(|component| component.id != ComponentId("D1".into()));
        project
            .components
            .iter_mut()
            .find(|component| component.id == ComponentId("RV1".into()))
            .unwrap()
            .parameters
            .insert("max_resistance".into(), 1_000.0);
        let mut state = SimulationState::new(&project);
        apply_actions(&mut project, &baseline, &mut state, &[Action::Run]);
        let mut sounding_steps = Vec::new();
        for step in 0..4_000 {
            advance_steps(&project, &mut state, 1);
            if state.passive_piezo_sounding(&ComponentId("R3".into())) {
                sounding_steps.push(step);
            }
        }
        assert!(!state.stale, "diagnostics={:?}", state.diagnostics);
        assert!(
            !sounding_steps.is_empty(),
            "no oscillating-drive steps observed"
        );
    }

    #[test]
    fn c04_doorbell_calculates_button_pulse_and_transistor_branches() {
        let baseline: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/c04-s06-05-doorbell.json"
        ))
        .unwrap();
        let mut project = baseline.clone();
        let mut state = SimulationState::new(&project);
        apply_actions(
            &mut project,
            &baseline,
            &mut state,
            &[
                Action::SetControl {
                    component: ComponentId("S1".into()),
                    state: ControlState::ButtonPressed,
                },
                Action::Run,
            ],
        );
        let mut max_speaker = 0.0_f64;
        let mut max_npn = 0.0_f64;
        let mut max_pnp = 0.0_f64;
        for _ in 0..4_000 {
            advance_steps(&project, &mut state, 1);
            let solved = state
                .last_valid
                .as_ref()
                .unwrap_or_else(|| panic!("diagnostics={:?}", state.diagnostics));
            max_speaker = max_speaker.max(
                solved
                    .resistor_currents
                    .get(&ComponentId("SP1".into()))
                    .copied()
                    .unwrap_or(0.0)
                    .abs(),
            );
            max_npn = max_npn.max(
                solved
                    .transistor_collector_currents
                    .get(&ComponentId("Q1".into()))
                    .copied()
                    .unwrap_or(0.0)
                    .abs(),
            );
            max_pnp = max_pnp.max(
                solved
                    .pnp_collector_currents
                    .get(&ComponentId("Q2".into()))
                    .copied()
                    .unwrap_or(0.0)
                    .abs(),
            );
        }
        assert!(!state.stale, "diagnostics={:?}", state.diagnostics);
        assert!(
            max_speaker > 0.004,
            "speaker={max_speaker} npn={max_npn} pnp={max_pnp}"
        );
        assert!(max_npn > 0.0001);
        assert!(max_pnp > 0.0001);
    }

    #[test]
    fn c04_siren_composes_dual_timers_and_switchable_mode_load() {
        let baseline: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/c04-s06-08-siren.json"
        ))
        .unwrap();
        let mut project = baseline.clone();
        let mut state = SimulationState::new(&project);
        apply_actions(&mut project, &baseline, &mut state, &[Action::Run]);
        let mut max_speaker = 0.0_f64;
        for _ in 0..4_000 {
            advance_steps(&project, &mut state, 1);
            let solved = state.last_valid.as_ref().unwrap();
            max_speaker = max_speaker.max(
                solved
                    .resistor_currents
                    .get(&ComponentId("SP1".into()))
                    .copied()
                    .unwrap_or(0.0)
                    .abs(),
            );
        }
        let normally_closed =
            state.last_valid.as_ref().unwrap().resistor_currents[&ComponentId("R8".into())];
        apply_actions(
            &mut project,
            &baseline,
            &mut state,
            &[
                Action::SetControl {
                    component: ComponentId("S1".into()),
                    state: ControlState::SwitchNormallyOpen,
                },
                Action::SingleStep,
            ],
        );
        let normally_open =
            state.last_valid.as_ref().unwrap().resistor_currents[&ComponentId("R7".into())];
        assert!(!state.stale, "diagnostics={:?}", state.diagnostics);
        assert!(max_speaker > 0.004);
        assert!(normally_closed.abs() > 0.00001);
        assert!(normally_open.abs() > 0.00001);
        assert_ne!(normally_closed, normally_open);
    }

    #[test]
    fn c04_piano_exposes_eight_calculated_key_branches() {
        let baseline: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/c04-s06-03-piano.json"
        ))
        .unwrap();
        let mut project = baseline.clone();
        let mut state = SimulationState::new(&project);
        apply_actions(
            &mut project,
            &baseline,
            &mut state,
            &[
                Action::SetControl {
                    component: ComponentId("B1".into()),
                    state: ControlState::ButtonPressed,
                },
                Action::Run,
            ],
        );
        let mut max_speaker = 0.0_f64;
        for _ in 0..4_000 {
            advance_steps(&project, &mut state, 1);
            let solved = state.last_valid.as_ref().unwrap();
            max_speaker = max_speaker.max(
                solved
                    .resistor_currents
                    .get(&ComponentId("SP1".into()))
                    .copied()
                    .unwrap_or(0.0)
                    .abs(),
            );
        }
        let key_one =
            state.last_valid.as_ref().unwrap().resistor_currents[&ComponentId("RV1".into())].abs();
        let key_two =
            state.last_valid.as_ref().unwrap().resistor_currents[&ComponentId("RV2".into())].abs();
        assert!(!state.stale, "diagnostics={:?}", state.diagnostics);
        assert!(max_speaker > 0.004);
        assert!(key_one > 0.0001);
        assert!(key_two < 1e-6);
    }

    #[test]
    fn c04_drum_machine_maps_counter_masks_and_calculates_speaker_path() {
        let baseline: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/c04-s06-12-drum-machine.json"
        ))
        .unwrap();
        let mut project = baseline.clone();
        let mut state = SimulationState::new(&project);
        apply_actions(&mut project, &baseline, &mut state, &[Action::Run]);
        let mut max_speaker = 0.0_f64;
        let mut max_kick_mask = 0.0_f64;
        let mut max_hat_mask = 0.0_f64;
        for _ in 0..50 {
            advance_steps(&project, &mut state, 1);
            let solved = state
                .last_valid
                .as_ref()
                .unwrap_or_else(|| panic!("diagnostics={:?}", state.diagnostics));
            max_speaker = max_speaker.max(
                solved
                    .resistor_currents
                    .get(&ComponentId("SP1".into()))
                    .copied()
                    .unwrap_or_default()
                    .abs(),
            );
            max_kick_mask = max_kick_mask.max(
                solved
                    .other_output_voltages
                    .get(&ComponentId("DIP_KICK".into()))
                    .and_then(|pins| pins.get(&crate::PinId("out".into())))
                    .copied()
                    .unwrap_or_default(),
            );
            max_hat_mask = max_hat_mask.max(
                solved
                    .other_output_voltages
                    .get(&ComponentId("DIP_HAT".into()))
                    .and_then(|pins| pins.get(&crate::PinId("out".into())))
                    .copied()
                    .unwrap_or_default(),
            );
        }
        assert!(!state.stale, "diagnostics={:?}", state.diagnostics);
        assert!(max_speaker > 0.001, "speaker current={max_speaker}");
        assert!(
            max_kick_mask > 0.1,
            "kick mask={max_kick_mask} states={:?} outputs={:?}",
            state.digital_states,
            state.last_valid.as_ref().and_then(|result| result
                .other_output_voltages
                .get(&ComponentId("DIP_KICK".into())))
        );
        assert!(
            max_hat_mask > 0.1,
            "hat mask={max_hat_mask} states={:?} outputs={:?}",
            state.digital_states,
            state.last_valid.as_ref().and_then(|result| result
                .other_output_voltages
                .get(&ComponentId("DIP_HAT".into())))
        );
    }

    #[test]
    fn c05_two_minute_timer_calculates_button_led_buzzer_and_pnp_state() {
        let baseline: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/c05-s07-10-two-minute-timer.json"
        ))
        .unwrap();
        let mut project = baseline.clone();
        let mut state = SimulationState::new(&project);
        apply_actions(
            &mut project,
            &baseline,
            &mut state,
            &[
                Action::SetControl {
                    component: ComponentId("S1".into()),
                    state: ControlState::ButtonPressed,
                },
                Action::Run,
            ],
        );
        let mut max_led = 0.0_f64;
        let mut max_buzzer = 0.0_f64;
        let mut max_pnp = 0.0_f64;
        for _ in 0..4_000 {
            advance_steps(&project, &mut state, 1);
            let solved = state.last_valid.as_ref().unwrap();
            max_led = max_led.max(solved.led_currents[&ComponentId("LED1".into())]);
            max_buzzer = max_buzzer.max(
                solved
                    .resistor_currents
                    .get(&ComponentId("BZ1".into()))
                    .copied()
                    .unwrap_or(0.0)
                    .abs(),
            );
            max_pnp = max_pnp.max(
                solved
                    .pnp_collector_currents
                    .get(&ComponentId("Q1".into()))
                    .copied()
                    .unwrap_or(0.0)
                    .abs(),
            );
        }
        assert!(!state.stale, "diagnostics={:?}", state.diagnostics);
        assert!(max_led > 0.001);
        assert!(max_buzzer > 0.001);
        assert!(max_pnp > 0.0001);
    }

    #[test]
    fn c05_pulse_generator_calculates_led_pulses_and_dial_effect() {
        let baseline: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/c05-s07-08-pulse-generator.json"
        ))
        .unwrap();
        let mut project = baseline.clone();
        let mut state = SimulationState::new(&project);
        apply_actions(&mut project, &baseline, &mut state, &[Action::Run]);
        let mut max_led = 0.0_f64;
        for _ in 0..4_000 {
            advance_steps(&project, &mut state, 1);
            max_led = max_led
                .max(state.last_valid.as_ref().unwrap().led_currents[&ComponentId("LED1".into())]);
        }
        let middle =
            state.last_valid.as_ref().unwrap().capacitor_voltages[&ComponentId("C1".into())];
        apply_actions(
            &mut project,
            &baseline,
            &mut state,
            &[
                Action::Reset,
                Action::SetControlRatio {
                    component: ComponentId("RV1".into()),
                    ratio: 0.9,
                },
                Action::Run,
            ],
        );
        for _ in 0..4_000 {
            advance_steps(&project, &mut state, 1);
        }
        let high = state.last_valid.as_ref().unwrap().capacitor_voltages[&ComponentId("C1".into())];
        assert!(!state.stale, "diagnostics={:?}", state.diagnostics);
        assert!(max_led > 0.001);
        assert_ne!(middle, high);
    }

    #[test]
    fn c03_shift_register_calculates_shift_then_latch() {
        let baseline: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/c03-s04-05-shift-register.json"
        ))
        .unwrap();
        let mut project = baseline.clone();
        let mut state = SimulationState::new(&project);
        apply_actions(&mut project, &baseline, &mut state, &[Action::Run]);
        advance_steps(&project, &mut state, 1);
        apply_actions(
            &mut project,
            &baseline,
            &mut state,
            &[
                Action::SetControl {
                    component: ComponentId("S_DATA".into()),
                    state: ControlState::ButtonPressed,
                },
                Action::SetControl {
                    component: ComponentId("S_CLOCK".into()),
                    state: ControlState::ButtonPressed,
                },
                Action::SingleStep,
            ],
        );
        let digital = state.digital_states[&ComponentId("U1".into())];
        assert_eq!(digital & 0xff, 1);
        assert_eq!((digital >> 8) & 0xff, 0);
        apply_actions(
            &mut project,
            &baseline,
            &mut state,
            &[
                Action::SetControl {
                    component: ComponentId("S_LATCH".into()),
                    state: ControlState::ButtonPressed,
                },
                Action::SingleStep,
            ],
        );
        assert_eq!(
            (state.digital_states[&ComponentId("U1".into())] >> 8) & 0xff,
            1
        );
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
            ic_device: None,
            other_device: None,
            module: None,
            diode_model: None,
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

    fn potentiometer_project() -> Project {
        let mut project = rc();
        project.components[1].kind = ComponentKind::Potentiometer;
        project.components[1].parameters = BTreeMap::from([
            ("min_resistance".into(), 100.0),
            ("max_resistance".into(), 10_000.0),
        ]);
        project
    }

    #[test]
    fn control_ratio_actions_apply_in_order_between_steps_regardless_of_batching() {
        let baseline = potentiometer_project();
        let mut project = baseline.clone();
        let mut state = SimulationState::new(&project);
        let id = ComponentId("R1".into());
        assert_eq!(state.control_ratios[&id], 0.5);
        // Two ratio changes applied in one ordered batch between steps: only
        // the last one is visible, matching SetParameter's overwrite semantics.
        apply_actions(
            &mut project,
            &baseline,
            &mut state,
            &[
                Action::SetControlRatio {
                    component: id.clone(),
                    ratio: 0.2,
                },
                Action::SetControlRatio {
                    component: id.clone(),
                    ratio: 0.8,
                },
                Action::SingleStep,
            ],
        );
        assert_eq!(state.control_ratios[&id], 0.8);
        assert_eq!(state.step, 1);
        assert!(!state.stale);

        // Rejected: out of range and unknown component leave state untouched.
        apply_actions(
            &mut project,
            &baseline,
            &mut state,
            &[Action::SetControlRatio {
                component: id.clone(),
                ratio: 1.5,
            }],
        );
        assert_eq!(state.control_ratios[&id], 0.8);
        assert_eq!(state.diagnostics[0].code, "invalid_control_ratio");

        apply_actions(&mut project, &baseline, &mut state, &[Action::Reset]);
        assert_eq!(state.control_ratios[&id], 0.5);
        assert_eq!(state.step, 0);
    }

    #[test]
    fn control_ratio_change_is_visible_in_recalculated_readings() {
        let baseline = potentiometer_project();
        let mut project = baseline.clone();
        let mut state = SimulationState::new(&project);
        let id = ComponentId("R1".into());
        apply_actions(&mut project, &baseline, &mut state, &[Action::Run]);
        advance_steps(&project, &mut state, 1);
        let low_ratio_current = state.last_valid.as_ref().unwrap().resistor_currents[&id];
        apply_actions(
            &mut project,
            &baseline,
            &mut state,
            &[Action::SetControlRatio {
                component: id.clone(),
                ratio: 1.0,
            }],
        );
        advance_steps(&project, &mut state, 1);
        let high_ratio_current = state.last_valid.as_ref().unwrap().resistor_currents[&id];
        // Higher ratio -> higher resistance -> lower current for a potentiometer.
        assert!(high_ratio_current < low_ratio_current);
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

    #[test]
    fn repair_fault_action_changes_the_authoritative_project_and_recalculates() {
        let mut project: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/c12-s20-01-reversed-led.json"
        ))
        .unwrap();
        let baseline = project.clone();
        let mut state = SimulationState::new(&project);
        apply_actions(&mut project, &baseline, &mut state, &[Action::SingleStep]);
        assert!(state.last_valid.as_ref().unwrap().led_currents[&ComponentId("D1".into())] < 1e-6);

        apply_actions(
            &mut project,
            &baseline,
            &mut state,
            &[Action::RepairFault { fault: "F1".into() }],
        );
        assert!(state.stale);
        apply_actions(&mut project, &baseline, &mut state, &[Action::SingleStep]);
        assert!(!state.stale);
        let repaired_current =
            state.last_valid.as_ref().unwrap().led_currents[&ComponentId("D1".into())];
        assert!(
            repaired_current > 0.001,
            "repaired current: {repaired_current}"
        );
        assert!(project.faults.is_empty());
        assert_ne!(project, baseline);
    }

    #[test]
    fn c12_fault_fixtures_repair_calculated_outputs() {
        for (json, led) in [
            (
                include_str!("../../../fixtures/projects/c12-s20-02-broken-rail.json"),
                "D4",
            ),
            (
                include_str!("../../../fixtures/projects/c12-s20-03-wrong-row.json"),
                "D1",
            ),
            (
                include_str!("../../../fixtures/projects/c12-s20-04-wrong-resistor.json"),
                "D1",
            ),
        ] {
            let mut project: Project = serde_json::from_str(json).unwrap();
            let baseline = project.clone();
            let fault = project.faults[0].id.clone();
            let mut state = SimulationState::new(&project);
            apply_actions(&mut project, &baseline, &mut state, &[Action::SingleStep]);
            let before = state.last_valid.as_ref().unwrap().led_currents[&ComponentId(led.into())];
            apply_actions(
                &mut project,
                &baseline,
                &mut state,
                &[Action::RepairFault { fault }],
            );
            apply_actions(&mut project, &baseline, &mut state, &[Action::SingleStep]);
            let after = state.last_valid.as_ref().unwrap().led_currents[&ComponentId(led.into())];
            assert!(after > 0.001, "{led}: before={before}, after={after}");
            assert!(
                after > before + 0.001,
                "{led}: before={before}, after={after}"
            );
        }
    }

    #[test]
    fn all_c12_fault_pairs_repair_through_explicit_actions() {
        for (index, json) in [
            include_str!("../../../fixtures/projects/c12-s20-05-wrong-transistor-pin.json"),
            include_str!("../../../fixtures/projects/c12-s20-06-reversed-555.json"),
            include_str!("../../../fixtures/projects/c12-s20-07-floating-cmos-input.json"),
            include_str!("../../../fixtures/projects/c12-s20-08-missing-flyback.json"),
            include_str!("../../../fixtures/projects/c12-s20-09-missing-decoupling.json"),
            include_str!("../../../fixtures/projects/c12-s20-10-button-bounce.json"),
            include_str!("../../../fixtures/projects/c12-s20-11-missing-ground.json"),
            include_str!("../../../fixtures/projects/c12-s20-12-broken-project.json"),
        ]
        .into_iter()
        .enumerate()
        {
            let mut project: Project = serde_json::from_str(json).unwrap();
            let baseline = project.clone();
            let mut state = SimulationState::new(&project);
            while let Some(fault) = project.faults.first().map(|fault| fault.id.clone()) {
                apply_actions(
                    &mut project,
                    &baseline,
                    &mut state,
                    &[Action::RepairFault { fault }],
                );
                apply_actions(&mut project, &baseline, &mut state, &[Action::SingleStep]);
                assert!(
                    !state.stale,
                    "fixture {index}: diagnostics={:?}",
                    state.diagnostics
                );
            }
            assert!(state.last_valid.is_some());
        }
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
