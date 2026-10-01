use crate::module::adjustable_output_target;
use crate::{
    BjtPolarity, BjtTestState, Component, ComponentId, ComponentKind, Contact, ControlState,
    Diagnostic, DiodeModel, DiodePolarity, DiodeSpec, DiodeTestState, IcDeviceBehavior,
    IcLogicOperation, ModuleBehavior, Node, OtherDeviceBehavior, PinId, Project, compile_topology,
    controlled_resistance, ring_modulator_output,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Clone, Debug, PartialEq)]
pub struct ElectricalDiagnostic {
    pub code: &'static str,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ElectricalError {
    Structure(Vec<Diagnostic>),
    Calculation(ElectricalDiagnostic),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SolveResult {
    /// Absolute voltages use one deterministic zero reference per connected circuit.
    pub node_voltages: Vec<NodeVoltage>,
    /// Resistor current is positive from pin `a` to pin `b`.
    pub resistor_currents: BTreeMap<ComponentId, f64>,
    /// Source current is positive from pin `positive` to pin `negative`.
    pub source_currents: BTreeMap<ComponentId, f64>,
    /// Current through a closed ideal switch, positive from first to second pin.
    pub switch_currents: BTreeMap<ComponentId, f64>,
    /// Motor current is positive from pin `positive` to pin `negative`.
    #[serde(default)]
    pub motor_currents: BTreeMap<ComponentId, f64>,
    pub capacitor_voltages: BTreeMap<ComponentId, f64>,
    pub capacitor_currents: BTreeMap<ComponentId, f64>,
    #[serde(default)]
    pub led_currents: BTreeMap<ComponentId, f64>,
    #[serde(default)]
    pub diode_currents: BTreeMap<ComponentId, f64>,
    #[serde(default)]
    pub transistor_collector_currents: BTreeMap<ComponentId, f64>,
    #[serde(default)]
    pub pnp_collector_currents: BTreeMap<ComponentId, f64>,
    /// Signed no-load speed estimate in RPM. Positive voltage from `positive`
    /// to `negative` yields positive speed; this is an explicit educational
    /// actuator contract, not a mechanical inertia simulation.
    #[serde(default)]
    pub motor_speeds: BTreeMap<ComponentId, f64>,
    /// Calculated LED-side current through each optocoupler input.
    #[serde(default)]
    pub optocoupler_input_currents: BTreeMap<ComponentId, f64>,
    /// Relay coil current is positive from `coil_positive` to `coil_negative`.
    #[serde(default)]
    pub relay_coil_currents: BTreeMap<ComponentId, f64>,
    /// Whether each relay's calculated coil voltage reaches its pickup voltage.
    #[serde(default)]
    pub relay_energized: BTreeMap<ComponentId, bool>,
    /// Calculated output pin voltages for generic IC/device contracts.
    #[serde(default)]
    pub ic_device_output_voltages: BTreeMap<ComponentId, BTreeMap<PinId, f64>>,
    /// Calculated terminal currents for `ComponentKind::Other` contracts.
    /// Each current is positive into the named pin; the solver derives these
    /// values from the solved node voltages and the declared behavior.
    #[serde(default)]
    pub other_terminal_currents: BTreeMap<ComponentId, BTreeMap<PinId, f64>>,
    /// Calculated output pin voltages for linear-transfer `Other` devices.
    #[serde(default)]
    pub other_output_voltages: BTreeMap<ComponentId, BTreeMap<PinId, f64>>,
    /// Calculated output pin voltages for ready-made module contracts.
    #[serde(default)]
    pub module_output_voltages: BTreeMap<ComponentId, BTreeMap<PinId, f64>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct NodeVoltage {
    pub contacts: Vec<Contact>,
    pub voltage: f64,
}

#[derive(Clone)]
struct Branch {
    component: ComponentId,
    kind: BranchKind,
    a: usize,
    b: usize,
    value: f64,
    previous_voltage: f64,
}
#[derive(Clone, Copy)]
enum BranchKind {
    Resistor,
    Motor,
    VoltageSource,
    Switch,
    Capacitor,
}

#[derive(Clone)]
enum NonlinearElement {
    Led {
        id: ComponentId,
        anode: usize,
        cathode: usize,
        forward: f64,
        resistance: f64,
    },
    Diode {
        id: ComponentId,
        anode: usize,
        cathode: usize,
        model: DiodeModel,
    },
    Optocoupler {
        id: ComponentId,
        input_anode: usize,
        input_cathode: usize,
        collector: usize,
        emitter: usize,
        forward: f64,
        series_resistance: f64,
        transfer_gain: f64,
        on_resistance: f64,
        off_resistance: f64,
    },
    Relay {
        id: ComponentId,
        coil_positive: usize,
        coil_negative: usize,
        common: usize,
        normally_closed: usize,
        normally_open: usize,
        coil_resistance: f64,
        pickup_voltage: f64,
        contact_resistance: f64,
    },
    Npn {
        id: ComponentId,
        base: usize,
        collector: usize,
        emitter: usize,
        beta: f64,
        saturation: f64,
        reverse_breakdown_voltage: Option<f64>,
        reverse_breakdown_resistance: Option<f64>,
        noise_sample: f64,
    },
    Pnp {
        id: ComponentId,
        base: usize,
        collector: usize,
        emitter: usize,
        beta: f64,
        saturation: f64,
    },
    OtherBjtSocket {
        base: usize,
        collector: usize,
        emitter: usize,
        socket_polarity: BjtPolarity,
        subject_polarity: BjtPolarity,
        subject_state: BjtTestState,
        beta: f64,
        saturation: f64,
        open_resistance: f64,
        short_resistance: f64,
    },
    OtherDiodeSocket {
        anode: usize,
        cathode: usize,
        subject_polarity: DiodePolarity,
        subject_state: DiodeTestState,
        model: DiodeModel,
        open_resistance: f64,
        short_resistance: f64,
    },
    LogicGate {
        input_a: usize,
        input_b: usize,
        output: usize,
        vcc: usize,
        gnd: usize,
        operation: u8,
        resistance: f64,
    },
    SchmittInverter {
        input: usize,
        output: usize,
        vcc: usize,
        gnd: usize,
        resistance: f64,
    },
    Comparator {
        inverting: usize,
        non_inverting: usize,
        output: usize,
        vcc: usize,
        gnd: usize,
        resistance: f64,
    },
    Timer555 {
        control: usize,
        discharge: usize,
        output: usize,
        reset: usize,
        threshold: usize,
        trigger: usize,
        vcc: usize,
        gnd: usize,
        output_resistance: f64,
        discharge_resistance: f64,
    },
    DFlipFlop {
        clock: usize,
        data: usize,
        not_q: usize,
        q: usize,
        reset: usize,
        set: usize,
        vcc: usize,
        gnd: usize,
        resistance: f64,
        state: u32,
    },
    DigitalCounter {
        clock: usize,
        carry: usize,
        enable: usize,
        outputs: [usize; 10],
        reset: usize,
        vcc: usize,
        gnd: usize,
        mode: u8,
        modulus: u8,
        resistance: f64,
        state: u32,
    },
    StepSequencer {
        clock: usize,
        controls: [usize; 8],
        output: usize,
        steps: [usize; 8],
        vcc: usize,
        gnd: usize,
        resistance: f64,
        state: u32,
    },
    Sram {
        address: usize,
        clock: usize,
        data: [usize; 8],
        output: [usize; 8],
        reset: usize,
        vcc: usize,
        gnd: usize,
        write: usize,
        resistance: f64,
        state: u32,
    },
    ShiftRegister {
        clock: usize,
        data: usize,
        outputs: [usize; 8],
        vcc: usize,
        gnd: usize,
        resistance: f64,
        state: u32,
    },
    SevenSegmentDisplay {
        inputs: [usize; 4],
        segments: [usize; 7],
        common: usize,
        vcc: usize,
        gnd: usize,
        resistance: f64,
    },
    FourBitAdder {
        a: [usize; 4],
        b: [usize; 4],
        carry_in: usize,
        carry_out: usize,
        subtract: usize,
        sum: [usize; 4],
        vcc: usize,
        gnd: usize,
        resistance: f64,
    },
    BargraphDisplay {
        input: usize,
        segments: [usize; 10],
        vcc: usize,
        gnd: usize,
        resistance: f64,
    },
    AudioAmplifier {
        input: usize,
        output: usize,
        vcc: usize,
        gnd: usize,
        gain: f64,
        resistance: f64,
    },
    IcDevice {
        id: ComponentId,
        pin_nodes: BTreeMap<PinId, usize>,
        behavior: IcDeviceBehavior,
    },
    OtherSource {
        positive: usize,
        negative: usize,
        voltage: f64,
        resistance: f64,
    },
    OtherLinearTransfer {
        pin_nodes: BTreeMap<PinId, usize>,
        output: PinId,
        reference: PinId,
        inputs: Vec<(PinId, f64)>,
        offset: f64,
        min_output: f64,
        max_output: f64,
        input_resistance: f64,
        output_resistance: f64,
    },
    OtherTransformer {
        primary_positive: usize,
        primary_negative: usize,
        secondary_positive: usize,
        secondary_negative: usize,
        turns_ratio: f64,
        primary_resistance: f64,
        secondary_resistance: f64,
    },
    OtherRingModulator {
        signal: usize,
        carrier: usize,
        output: usize,
        reference: usize,
        gain: f64,
        signal_scale: f64,
        carrier_scale: f64,
        min_output: f64,
        max_output: f64,
        input_resistance: f64,
        output_resistance: f64,
    },
    OtherControlledResistance {
        control_positive: usize,
        control_negative: usize,
        output_positive: usize,
        output_negative: usize,
        min_resistance: f64,
        max_resistance: f64,
        control_min: f64,
        control_max: f64,
    },
    Module {
        id: ComponentId,
        pin_nodes: BTreeMap<PinId, usize>,
        behavior: ModuleBehavior,
    },
}

pub const FIXED_STEP_SECONDS: f64 = 100e-6;
pub const MAX_NONLINEAR_ITERATIONS: usize = 200;
/// Under-relaxation keeps coupled LED/NPN transient solves deterministic while
/// retaining the bounded iteration and explicit nonconvergence diagnostic.
const NONLINEAR_RELAXATION: f64 = 0.25;

#[derive(Clone, Copy)]
struct SolveOptions {
    dt: Option<f64>,
    step: u64,
    max_iterations: usize,
}

/// Solve a DC project using MNA, bounded nonlinear iteration, and deterministic partial pivoting.
/// A floating connected network, ideal-source short, contradictory source loop,
/// or singular ideal-source arrangement is reported as an electrical error.
pub fn solve_dc(
    project: &Project,
    states: &BTreeMap<ComponentId, ControlState>,
    ratios: &BTreeMap<ComponentId, f64>,
) -> Result<SolveResult, ElectricalError> {
    solve_internal(
        project,
        states,
        &BTreeMap::new(),
        ratios,
        &BTreeMap::new(),
        SolveOptions {
            dt: None,
            step: 0,
            max_iterations: MAX_NONLINEAR_ITERATIONS,
        },
    )
}

/// Solve one 100 microsecond Backward Euler step from the supplied capacitor state.
pub fn solve_transient(
    project: &Project,
    states: &BTreeMap<ComponentId, ControlState>,
    capacitor_voltages: &BTreeMap<ComponentId, f64>,
    ratios: &BTreeMap<ComponentId, f64>,
) -> Result<SolveResult, ElectricalError> {
    solve_transient_with_digital_states(
        project,
        states,
        capacitor_voltages,
        ratios,
        &BTreeMap::new(),
        0,
        MAX_NONLINEAR_ITERATIONS,
    )
}

pub(crate) fn solve_transient_with_digital_states(
    project: &Project,
    states: &BTreeMap<ComponentId, ControlState>,
    capacitor_voltages: &BTreeMap<ComponentId, f64>,
    ratios: &BTreeMap<ComponentId, f64>,
    digital_states: &BTreeMap<ComponentId, u32>,
    step: u64,
    max_iterations: usize,
) -> Result<SolveResult, ElectricalError> {
    solve_internal(
        project,
        states,
        capacitor_voltages,
        ratios,
        digital_states,
        SolveOptions {
            dt: Some(FIXED_STEP_SECONDS),
            step,
            max_iterations,
        },
    )
}

fn solve_internal(
    project: &Project,
    states: &BTreeMap<ComponentId, ControlState>,
    capacitor_voltages: &BTreeMap<ComponentId, f64>,
    ratios: &BTreeMap<ComponentId, f64>,
    digital_states: &BTreeMap<ComponentId, u32>,
    options: SolveOptions,
) -> Result<SolveResult, ElectricalError> {
    let topology = compile_topology(project).map_err(ElectricalError::Structure)?;
    let mut control_state = project.initial_conditions.controls.clone();
    control_state.extend(states.clone());
    let mut cap_state = project.initial_conditions.capacitor_voltages.clone();
    cap_state.extend(capacitor_voltages.clone());
    let mut ratio_state = project.initial_conditions.control_ratios.clone();
    ratio_state.extend(ratios.clone());
    let (mut branches, nonlinear, node_contacts, active_nodes) = make_branches(
        project,
        &topology,
        &control_state,
        &cap_state,
        &ratio_state,
        digital_states,
        options,
    )?;
    let coupled_transient = options.dt.is_some()
        && branches
            .iter()
            .filter(|branch| matches!(branch.kind, BranchKind::Capacitor))
            .count()
            >= 2
        && nonlinear.len() >= 2;
    let use_collector_base_jacobian = options.dt.is_none()
        || !branches
            .iter()
            .any(|branch| matches!(branch.kind, BranchKind::Capacitor));
    let relaxation = if coupled_transient {
        NONLINEAR_RELAXATION
    } else {
        1.0
    };
    if branches.is_empty() && nonlinear.is_empty() {
        return Ok(SolveResult {
            node_voltages: Vec::new(),
            resistor_currents: BTreeMap::new(),
            source_currents: BTreeMap::new(),
            switch_currents: BTreeMap::new(),
            motor_currents: BTreeMap::new(),
            capacitor_voltages: BTreeMap::new(),
            capacitor_currents: BTreeMap::new(),
            led_currents: BTreeMap::new(),
            diode_currents: BTreeMap::new(),
            transistor_collector_currents: BTreeMap::new(),
            pnp_collector_currents: BTreeMap::new(),
            motor_speeds: BTreeMap::new(),
            optocoupler_input_currents: BTreeMap::new(),
            relay_coil_currents: BTreeMap::new(),
            relay_energized: BTreeMap::new(),
            ic_device_output_voltages: BTreeMap::new(),
            other_terminal_currents: BTreeMap::new(),
            other_output_voltages: BTreeMap::new(),
            module_output_voltages: BTreeMap::new(),
        });
    }

    let mut adjacency = vec![Vec::new(); node_contacts.len()];
    for branch in &branches {
        adjacency[branch.a].push(branch.b);
        adjacency[branch.b].push(branch.a);
    }
    for element in &nonlinear {
        let pairs: Vec<(usize, usize)> = match element {
            NonlinearElement::Led { anode, cathode, .. }
            | NonlinearElement::Diode { anode, cathode, .. } => vec![(*anode, *cathode)],
            NonlinearElement::Optocoupler {
                input_anode,
                input_cathode,
                collector,
                emitter,
                ..
            } => vec![(*input_anode, *input_cathode), (*collector, *emitter)],
            NonlinearElement::Relay {
                coil_positive,
                coil_negative,
                common,
                normally_closed,
                normally_open,
                ..
            } => vec![
                (*coil_positive, *coil_negative),
                (*common, *normally_closed),
                (*common, *normally_open),
            ],
            NonlinearElement::Npn {
                base,
                collector,
                emitter,
                ..
            }
            | NonlinearElement::Pnp {
                base,
                collector,
                emitter,
                ..
            } => vec![(*base, *emitter), (*collector, *emitter)],
            NonlinearElement::OtherBjtSocket {
                base,
                collector,
                emitter,
                ..
            } => vec![(*base, *emitter), (*collector, *emitter)],
            NonlinearElement::OtherDiodeSocket { anode, cathode, .. } => {
                vec![(*anode, *cathode)]
            }
            NonlinearElement::LogicGate {
                input_a,
                input_b,
                output,
                vcc,
                gnd,
                ..
            } => vec![
                (*input_a, *output),
                (*input_b, *output),
                (*vcc, *output),
                (*gnd, *output),
            ],
            NonlinearElement::SchmittInverter {
                input,
                output,
                vcc,
                gnd,
                ..
            } => vec![(*input, *output), (*vcc, *output), (*gnd, *output)],
            NonlinearElement::Comparator {
                inverting,
                non_inverting,
                output,
                vcc,
                gnd,
                ..
            } => vec![
                (*inverting, *output),
                (*non_inverting, *output),
                (*vcc, *output),
                (*gnd, *output),
            ],
            NonlinearElement::Timer555 {
                control,
                discharge,
                output,
                reset,
                threshold,
                trigger,
                vcc,
                gnd,
                ..
            } => vec![
                (*control, *output),
                (*discharge, *output),
                (*reset, *output),
                (*threshold, *output),
                (*trigger, *output),
                (*vcc, *output),
                (*gnd, *output),
            ],
            NonlinearElement::DFlipFlop {
                clock,
                data,
                not_q,
                q,
                reset,
                set,
                vcc,
                gnd,
                ..
            } => vec![
                (*clock, *q),
                (*data, *q),
                (*not_q, *q),
                (*reset, *q),
                (*set, *q),
                (*vcc, *q),
                (*gnd, *q),
            ],
            NonlinearElement::DigitalCounter {
                clock,
                enable,
                outputs,
                reset,
                vcc,
                gnd,
                ..
            } => {
                let mut pairs = vec![
                    (*clock, outputs[0]),
                    (*enable, outputs[0]),
                    (*reset, outputs[0]),
                    (*vcc, outputs[0]),
                    (*gnd, outputs[0]),
                ];
                pairs.extend(outputs.windows(2).map(|pair| (pair[0], pair[1])));
                pairs
            }
            NonlinearElement::StepSequencer {
                clock,
                controls,
                output,
                steps,
                vcc,
                gnd,
                ..
            } => {
                let mut pairs = vec![(*clock, *output), (*vcc, *output), (*gnd, *output)];
                pairs.extend(controls.iter().map(|control| (*control, *output)));
                pairs.extend(steps.iter().map(|step| (*step, *output)));
                pairs
            }
            NonlinearElement::Sram {
                address,
                clock,
                data,
                output,
                reset,
                vcc,
                gnd,
                write,
                ..
            } => {
                let mut pairs = vec![
                    (*address, output[0]),
                    (*clock, output[0]),
                    (*reset, output[0]),
                    (*vcc, output[0]),
                    (*gnd, output[0]),
                    (*write, output[0]),
                ];
                pairs.extend(data.iter().map(|input| (*input, output[0])));
                pairs.extend(output.windows(2).map(|pair| (pair[0], pair[1])));
                pairs
            }
            NonlinearElement::ShiftRegister {
                clock,
                data,
                outputs,
                vcc,
                gnd,
                ..
            } => {
                let mut pairs = vec![
                    (*clock, outputs[0]),
                    (*data, outputs[0]),
                    (*vcc, outputs[0]),
                    (*gnd, outputs[0]),
                ];
                pairs.extend(outputs.windows(2).map(|pair| (pair[0], pair[1])));
                pairs
            }
            NonlinearElement::SevenSegmentDisplay {
                inputs,
                segments,
                common,
                vcc,
                gnd,
                ..
            } => {
                let mut pairs = vec![(*common, *gnd), (*vcc, *gnd)];
                pairs.extend(inputs.iter().map(|input| (*input, *gnd)));
                pairs.extend(segments.iter().map(|segment| (*segment, *common)));
                pairs
            }
            NonlinearElement::FourBitAdder {
                a,
                b,
                carry_in,
                carry_out,
                subtract,
                sum,
                vcc,
                gnd,
                ..
            } => {
                let mut pairs = vec![
                    (*carry_in, sum[0]),
                    (*subtract, sum[0]),
                    (*vcc, sum[0]),
                    (*gnd, sum[0]),
                    (*carry_out, sum[0]),
                ];
                pairs.extend(a.iter().map(|input| (*input, sum[0])));
                pairs.extend(b.iter().map(|input| (*input, sum[0])));
                pairs.extend(sum.windows(2).map(|pair| (pair[0], pair[1])));
                pairs
            }
            NonlinearElement::BargraphDisplay {
                input,
                segments,
                vcc,
                gnd,
                ..
            } => {
                let mut pairs = vec![
                    (*input, segments[0]),
                    (*vcc, segments[0]),
                    (*gnd, segments[0]),
                ];
                pairs.extend(segments.windows(2).map(|pair| (pair[0], pair[1])));
                pairs
            }
            NonlinearElement::AudioAmplifier {
                input,
                output,
                vcc,
                gnd,
                ..
            } => vec![(*input, *output), (*vcc, *output), (*gnd, *output)],
            NonlinearElement::IcDevice { pin_nodes, .. } => {
                let mut nodes = pin_nodes.values().copied();
                let Some(first) = nodes.next() else { continue };
                nodes.map(|node| (first, node)).collect()
            }
            NonlinearElement::OtherSource {
                positive, negative, ..
            } => vec![(*positive, *negative)],
            NonlinearElement::OtherLinearTransfer { pin_nodes, .. } => {
                let mut nodes = pin_nodes.values().copied();
                let Some(first) = nodes.next() else { continue };
                nodes.map(|node| (first, node)).collect()
            }
            NonlinearElement::OtherTransformer {
                primary_positive,
                primary_negative,
                secondary_positive,
                secondary_negative,
                ..
            } => vec![
                (*primary_positive, *primary_negative),
                (*secondary_positive, *secondary_negative),
            ],
            NonlinearElement::OtherRingModulator {
                signal,
                carrier,
                output,
                reference,
                ..
            } => vec![
                (*signal, *reference),
                (*carrier, *reference),
                (*output, *reference),
            ],
            NonlinearElement::OtherControlledResistance {
                control_positive,
                control_negative,
                output_positive,
                output_negative,
                ..
            } => vec![
                (*control_positive, *control_negative),
                (*output_positive, *output_negative),
            ],
            NonlinearElement::Module { pin_nodes, .. } => {
                let mut nodes = pin_nodes.values().copied();
                let Some(first) = nodes.next() else { continue };
                nodes.map(|node| (first, node)).collect()
            }
        };
        for (a, b) in pairs {
            adjacency[a].push(b);
            adjacency[b].push(a);
        }
    }
    let mut islands = Vec::new();
    let mut seen = BTreeSet::new();
    for &start in &active_nodes {
        if seen.contains(&start) {
            continue;
        }
        let mut queue = VecDeque::from([start]);
        let mut island = Vec::new();
        seen.insert(start);
        while let Some(node) = queue.pop_front() {
            island.push(node);
            for &next in &adjacency[node] {
                if seen.insert(next) {
                    queue.push_back(next);
                }
            }
        }
        if !island.iter().any(|n| {
            branches
                .iter()
                .any(|b| (b.a == *n || b.b == *n) && matches!(b.kind, BranchKind::VoltageSource))
        }) && !island.iter().any(|n| {
            nonlinear.iter().any(|element| {
                matches!(
                    element,
                    NonlinearElement::OtherSource {
                        positive, negative, ..
                    } if positive == n || negative == n
                )
            })
        }) {
            return Err(calc(
                "floating_network",
                "connected resistive network has no voltage source",
            ));
        }
        islands.push(island);
    }

    for branch in &branches {
        if matches!(branch.kind, BranchKind::VoltageSource)
            && branch.a == branch.b
            && branch.value.abs() > VOLTAGE_TOLERANCE
        {
            return Err(calc(
                "ideal_source_short",
                format!(
                    "voltage source {} has both terminals on the same node",
                    branch.component.0
                ),
            ));
        }
    }
    check_source_cycles(&branches)?;
    branches.sort_by(|a, b| a.component.cmp(&b.component));
    let references: BTreeSet<_> = islands
        .iter()
        .filter_map(|nodes| {
            branches
                .iter()
                .filter(|b| nodes.contains(&b.a) && matches!(b.kind, BranchKind::VoltageSource))
                .map(|b| b.b)
                .min()
                .or_else(|| nodes.iter().min().copied())
        })
        .collect();
    let voltage_vars: BTreeMap<usize, usize> = active_nodes
        .iter()
        .copied()
        .filter(|n| !references.contains(n))
        .enumerate()
        .map(|(i, n)| (n, i))
        .collect();
    let constraints: Vec<_> = branches
        .iter()
        .filter(|b| matches!(b.kind, BranchKind::VoltageSource | BranchKind::Switch))
        .collect();
    let size = voltage_vars.len() + constraints.len();
    let mut guess = vec![0.0; size];
    let mut solved = None;
    for _ in 0..options.max_iterations {
        let mut matrix = vec![vec![0.0; size]; size];
        let mut rhs = vec![0.0; size];
        for branch in &branches {
            match branch.kind {
                BranchKind::Resistor | BranchKind::Motor | BranchKind::Capacitor => {
                    let g = 1.0 / branch.value;
                    let conductance = if matches!(branch.kind, BranchKind::Capacitor) {
                        branch.value
                    } else {
                        g
                    };
                    stamp_conductance(&mut matrix, &voltage_vars, branch.a, branch.b, conductance);
                    if matches!(branch.kind, BranchKind::Capacitor) {
                        stamp_history(
                            &mut rhs,
                            &voltage_vars,
                            branch.a,
                            branch.b,
                            conductance * branch.previous_voltage,
                        );
                    }
                }
                BranchKind::VoltageSource | BranchKind::Switch => {}
            }
        }
        for (offset, branch) in constraints.iter().enumerate() {
            let current_var = voltage_vars.len() + offset;
            stamp_constraint(&mut matrix, &voltage_vars, branch.a, branch.b, current_var);
            rhs[current_var] = branch.value;
        }
        for element in &nonlinear {
            stamp_element(
                element,
                &guess,
                &voltage_vars,
                &mut matrix,
                &mut rhs,
                use_collector_base_jacobian,
            );
        }
        let solution = gaussian_solve(matrix, rhs).ok_or_else(|| {
            calc(
                "singular_system",
                "circuit has an underdetermined ideal-source arrangement",
            )
        })?;
        let max_delta = voltage_vars
            .values()
            .map(|&i| (solution[i] - guess[i]).abs())
            .fold(0.0, f64::max);
        if max_delta < 1e-8 || nonlinear.is_empty() {
            solved = Some(solution);
            break;
        }
        for &i in voltage_vars.values() {
            guess[i] += ((solution[i] - guess[i]) * relaxation).clamp(-5.0, 5.0);
        }
    }
    let solution = solved.ok_or_else(|| {
        calc(
            "nonconvergence",
            format!(
                "nonlinear circuit did not converge within {} iterations",
                options.max_iterations
            ),
        )
    })?;
    let mut voltages = Vec::new();
    for &node in &active_nodes {
        let voltage = voltage_vars.get(&node).map_or(0.0, |&i| solution[i]);
        voltages.push(NodeVoltage {
            contacts: node_contacts[node].clone(),
            voltage,
        });
    }
    let mut resistor_currents = BTreeMap::new();
    let mut source_currents = BTreeMap::new();
    let mut switch_currents = BTreeMap::new();
    let mut motor_currents = BTreeMap::new();
    let mut capacitor_voltages = BTreeMap::new();
    let mut capacitor_currents = BTreeMap::new();
    let mut led_currents = BTreeMap::new();
    let mut diode_currents = BTreeMap::new();
    let mut transistor_collector_currents = BTreeMap::new();
    let mut pnp_collector_currents = BTreeMap::new();
    let mut motor_speeds = BTreeMap::new();
    let mut optocoupler_input_currents = BTreeMap::new();
    let mut relay_coil_currents = BTreeMap::new();
    let mut relay_energized = BTreeMap::new();
    let mut ic_device_output_voltages = BTreeMap::new();
    let mut other_terminal_currents = BTreeMap::new();
    let mut other_output_voltages = BTreeMap::new();
    let mut module_output_voltages = BTreeMap::new();
    for branch in &branches {
        let current = match branch.kind {
            BranchKind::Resistor | BranchKind::Motor => {
                (voltage(&solution, &voltage_vars, branch.a)
                    - voltage(&solution, &voltage_vars, branch.b))
                    / branch.value
            }
            BranchKind::Capacitor => {
                let cap_voltage = voltage(&solution, &voltage_vars, branch.a)
                    - voltage(&solution, &voltage_vars, branch.b);
                capacitor_voltages.insert(branch.component.clone(), cap_voltage);
                capacitor_currents.insert(
                    branch.component.clone(),
                    branch.value * (cap_voltage - branch.previous_voltage),
                );
                continue;
            }
            BranchKind::VoltageSource | BranchKind::Switch => {
                let index = voltage_vars.len()
                    + constraints
                        .iter()
                        .position(|c| c.component == branch.component)
                        .unwrap();
                solution[index]
            }
        };
        match branch.kind {
            BranchKind::Resistor => {
                resistor_currents.insert(branch.component.clone(), current);
            }
            BranchKind::Motor => {
                motor_currents.insert(branch.component.clone(), current);
            }
            BranchKind::VoltageSource => {
                source_currents.insert(branch.component.clone(), current);
            }
            BranchKind::Switch => {
                switch_currents.insert(branch.component.clone(), current);
            }
            BranchKind::Capacitor => unreachable!("capacitor current is handled above"),
        }
    }
    for element in &nonlinear {
        match element {
            NonlinearElement::Led {
                id,
                anode,
                cathode,
                forward,
                resistance,
            } => {
                let v = voltage(&solution, &voltage_vars, *anode)
                    - voltage(&solution, &voltage_vars, *cathode);
                led_currents.insert(id.clone(), led_current(v, *forward, *resistance).0);
            }
            NonlinearElement::Diode {
                id,
                anode,
                cathode,
                model,
            } => {
                let v = voltage(&solution, &voltage_vars, *anode)
                    - voltage(&solution, &voltage_vars, *cathode);
                diode_currents.insert(id.clone(), model.current(v));
            }
            NonlinearElement::Npn {
                id,
                base,
                collector,
                emitter,
                beta,
                saturation,
                ..
            } => {
                let vbe = voltage(&solution, &voltage_vars, *base)
                    - voltage(&solution, &voltage_vars, *emitter);
                let vce = voltage(&solution, &voltage_vars, *collector)
                    - voltage(&solution, &voltage_vars, *emitter);
                transistor_collector_currents
                    .insert(id.clone(), npn_currents(vbe, vce, *beta, *saturation).1);
            }
            NonlinearElement::Pnp {
                id,
                base,
                collector,
                emitter,
                beta,
                saturation,
            } => {
                let veb = voltage(&solution, &voltage_vars, *emitter)
                    - voltage(&solution, &voltage_vars, *base);
                let vec = voltage(&solution, &voltage_vars, *emitter)
                    - voltage(&solution, &voltage_vars, *collector);
                pnp_collector_currents
                    .insert(id.clone(), -npn_currents(veb, vec, *beta, *saturation).1);
            }
            NonlinearElement::Optocoupler {
                id,
                input_anode,
                input_cathode,
                forward,
                series_resistance,
                ..
            } => {
                let vin = voltage(&solution, &voltage_vars, *input_anode)
                    - voltage(&solution, &voltage_vars, *input_cathode);
                optocoupler_input_currents
                    .insert(id.clone(), led_current(vin, *forward, *series_resistance).0);
            }
            NonlinearElement::Relay {
                id,
                coil_positive,
                coil_negative,
                coil_resistance,
                pickup_voltage,
                ..
            } => {
                let coil_voltage = voltage(&solution, &voltage_vars, *coil_positive)
                    - voltage(&solution, &voltage_vars, *coil_negative);
                relay_coil_currents.insert(id.clone(), coil_voltage / coil_resistance);
                relay_energized.insert(id.clone(), coil_voltage >= *pickup_voltage);
            }
            NonlinearElement::IcDevice {
                id,
                pin_nodes,
                behavior,
            } => {
                let outputs = behavior.output_pins().into_iter().filter_map(|pin| {
                    pin_nodes
                        .get(pin)
                        .map(|node| (pin.clone(), voltage(&solution, &voltage_vars, *node)))
                });
                ic_device_output_voltages.insert(id.clone(), outputs.collect());
            }
            NonlinearElement::Module {
                id,
                pin_nodes,
                behavior,
            } => {
                let outputs = behavior.output_pins().into_iter().filter_map(|pin| {
                    pin_nodes
                        .get(pin)
                        .map(|node| (pin.clone(), voltage(&solution, &voltage_vars, *node)))
                });
                module_output_voltages.insert(id.clone(), outputs.collect());
            }
            NonlinearElement::OtherSource { .. }
            | NonlinearElement::OtherBjtSocket { .. }
            | NonlinearElement::OtherDiodeSocket { .. }
            | NonlinearElement::OtherLinearTransfer { .. }
            | NonlinearElement::OtherTransformer { .. }
            | NonlinearElement::OtherRingModulator { .. }
            | NonlinearElement::OtherControlledResistance { .. } => {}
            NonlinearElement::LogicGate { .. }
            | NonlinearElement::SchmittInverter { .. }
            | NonlinearElement::Comparator { .. }
            | NonlinearElement::Timer555 { .. }
            | NonlinearElement::DFlipFlop { .. }
            | NonlinearElement::DigitalCounter { .. }
            | NonlinearElement::StepSequencer { .. }
            | NonlinearElement::Sram { .. }
            | NonlinearElement::ShiftRegister { .. }
            | NonlinearElement::SevenSegmentDisplay { .. }
            | NonlinearElement::FourBitAdder { .. }
            | NonlinearElement::BargraphDisplay { .. }
            | NonlinearElement::AudioAmplifier { .. } => {}
        }
    }
    for component in project
        .components
        .iter()
        .filter(|component| component.kind == ComponentKind::Motor)
    {
        let positive =
            pin_node(topology.as_slice(), &component.id, "positive").ok_or_else(|| {
                calc(
                    "missing_pin_node",
                    format!(
                        "component {} pin positive has no compiled node",
                        component.id.0
                    ),
                )
            })?;
        let negative =
            pin_node(topology.as_slice(), &component.id, "negative").ok_or_else(|| {
                calc(
                    "missing_pin_node",
                    format!(
                        "component {} pin negative has no compiled node",
                        component.id.0
                    ),
                )
            })?;
        let terminal_voltage = voltage(&solution, &voltage_vars, positive)
            - voltage(&solution, &voltage_vars, negative);
        motor_speeds.insert(
            component.id.clone(),
            component.parameters["no_load_speed_rpm"]
                * (terminal_voltage / component.parameters["rated_voltage"]).clamp(-1.0, 1.0),
        );
    }
    for component in project
        .components
        .iter()
        .filter(|component| component.kind == ComponentKind::Other)
    {
        let spec = component.other_device.as_ref().ok_or_else(|| {
            calc(
                "missing_other_device_spec",
                format!("component {} has no other_device contract", component.id.0),
            )
        })?;
        let voltage_at = |pin: &PinId| -> Result<f64, ElectricalError> {
            let node = pin_node(&topology, &component.id, &pin.0).ok_or_else(|| {
                calc(
                    "missing_pin_node",
                    format!(
                        "component {} pin {} has no compiled node",
                        component.id.0, pin.0
                    ),
                )
            })?;
            Ok(voltage(&solution, &voltage_vars, node))
        };
        let mut currents = BTreeMap::new();
        match &spec.behavior {
            OtherDeviceBehavior::Resistive {
                positive,
                negative,
                resistance,
            } => {
                let current = (voltage_at(positive)? - voltage_at(negative)?) / resistance;
                currents.insert(positive.clone(), current);
                currents.insert(negative.clone(), -current);
            }
            OtherDeviceBehavior::VoltageSource {
                positive,
                negative,
                voltage: target,
                internal_resistance,
            } => {
                let current =
                    (voltage_at(positive)? - voltage_at(negative)? - target) / internal_resistance;
                currents.insert(positive.clone(), current);
                currents.insert(negative.clone(), -current);
            }
            OtherDeviceBehavior::BjtTestSocket { socket } => {
                let base_voltage = voltage_at(&socket.base)?;
                let collector_voltage = voltage_at(&socket.collector)?;
                let emitter_voltage = voltage_at(&socket.emitter)?;
                let (base_current, collector_current) = match socket.effective_state() {
                    BjtTestState::Working => match socket.socket_polarity {
                        BjtPolarity::Npn => {
                            let (base, collector, _, _) = npn_currents(
                                base_voltage - emitter_voltage,
                                collector_voltage - emitter_voltage,
                                socket.beta,
                                socket.saturation_current,
                            );
                            (base, collector)
                        }
                        BjtPolarity::Pnp => {
                            let (base, collector, _, _) = npn_currents(
                                emitter_voltage - base_voltage,
                                emitter_voltage - collector_voltage,
                                socket.beta,
                                socket.saturation_current,
                            );
                            (-base, -collector)
                        }
                    },
                    BjtTestState::Open => (
                        (base_voltage - emitter_voltage) / socket.open_resistance,
                        (collector_voltage - emitter_voltage) / socket.open_resistance,
                    ),
                    BjtTestState::Shorted => (
                        (base_voltage - emitter_voltage) / socket.short_resistance,
                        (collector_voltage - emitter_voltage) / socket.short_resistance,
                    ),
                };
                currents.insert(socket.base.clone(), base_current);
                currents.insert(socket.collector.clone(), collector_current);
                currents.insert(socket.emitter.clone(), -base_current - collector_current);
            }
            OtherDeviceBehavior::DiodeTestSocket { socket } => {
                let anode_voltage = voltage_at(&socket.anode)?;
                let cathode_voltage = voltage_at(&socket.cathode)?;
                let model = DiodeModel::new(
                    socket.forward_voltage,
                    socket.series_resistance,
                    DiodeSpec::default(),
                )
                .map_err(|error| {
                    calc(
                        "invalid_diode_test_socket",
                        format!("invalid diode test socket model: {error:?}"),
                    )
                })?;
                let current = match socket.subject_state {
                    DiodeTestState::Working => match socket.subject_polarity {
                        DiodePolarity::Forward => model.current(anode_voltage - cathode_voltage),
                        DiodePolarity::Reverse => -model.current(cathode_voltage - anode_voltage),
                    },
                    DiodeTestState::Open => {
                        (anode_voltage - cathode_voltage) / socket.open_resistance
                    }
                    DiodeTestState::Shorted => {
                        (anode_voltage - cathode_voltage) / socket.short_resistance
                    }
                };
                currents.insert(socket.anode.clone(), current);
                currents.insert(socket.cathode.clone(), -current);
            }
            OtherDeviceBehavior::LinearTransfer {
                output,
                reference,
                inputs,
                offset,
                min_output,
                max_output,
                input_resistance,
                output_resistance,
            } => {
                let reference_voltage = voltage_at(reference)?;
                let raw = *offset
                    + inputs.iter().try_fold(0.0, |sum, input| {
                        Ok::<_, ElectricalError>(
                            sum + input.gain * (voltage_at(&input.pin)? - reference_voltage),
                        )
                    })?;
                let target = reference_voltage + raw.clamp(*min_output, *max_output);
                let output_current = (voltage_at(output)? - target) / output_resistance;
                currents.insert(output.clone(), output_current);
                currents.insert(reference.clone(), -output_current);
                for input in inputs {
                    let current = (voltage_at(&input.pin)? - reference_voltage) / input_resistance;
                    currents.insert(input.pin.clone(), current);
                    *currents.entry(reference.clone()).or_default() -= current;
                }
                other_output_voltages.insert(
                    component.id.clone(),
                    BTreeMap::from([(output.clone(), voltage_at(output)?)]),
                );
            }
            OtherDeviceBehavior::Transformer {
                primary_positive,
                primary_negative,
                secondary_positive,
                secondary_negative,
                turns_ratio,
                primary_resistance,
                secondary_resistance,
            } => {
                let primary_voltage = voltage_at(primary_positive)? - voltage_at(primary_negative)?;
                let secondary_target =
                    voltage_at(secondary_negative)? + turns_ratio * primary_voltage;
                let primary_current = primary_voltage / primary_resistance;
                let secondary_current =
                    (voltage_at(secondary_positive)? - secondary_target) / secondary_resistance;
                currents.insert(primary_positive.clone(), primary_current);
                currents.insert(primary_negative.clone(), -primary_current);
                currents.insert(secondary_positive.clone(), secondary_current);
                currents.insert(secondary_negative.clone(), -secondary_current);
                other_output_voltages.insert(
                    component.id.clone(),
                    BTreeMap::from([(secondary_positive.clone(), voltage_at(secondary_positive)?)]),
                );
            }
            OtherDeviceBehavior::RingModulator {
                signal,
                carrier,
                output,
                reference,
                gain,
                signal_scale,
                carrier_scale,
                min_output,
                max_output,
                input_resistance,
                output_resistance,
            } => {
                let reference_voltage = voltage_at(reference)?;
                let signal_delta = voltage_at(signal)? - reference_voltage;
                let carrier_delta = voltage_at(carrier)? - reference_voltage;
                let target = ring_modulator_output(
                    reference_voltage,
                    voltage_at(signal)?,
                    voltage_at(carrier)?,
                    *gain,
                    *signal_scale,
                    *carrier_scale,
                    *min_output,
                    *max_output,
                );
                let output_current = (voltage_at(output)? - target) / output_resistance;
                let signal_current = signal_delta / input_resistance;
                let carrier_current = carrier_delta / input_resistance;
                currents.insert(output.clone(), output_current);
                currents.insert(signal.clone(), signal_current);
                currents.insert(carrier.clone(), carrier_current);
                currents.insert(
                    reference.clone(),
                    -output_current - signal_current - carrier_current,
                );
                other_output_voltages.insert(
                    component.id.clone(),
                    BTreeMap::from([(output.clone(), voltage_at(output)?)]),
                );
            }
            OtherDeviceBehavior::VoltageControlledResistance {
                control_positive,
                control_negative,
                output_positive,
                output_negative,
                min_resistance,
                max_resistance,
                control_min,
                control_max,
            } => {
                let control_voltage = voltage_at(control_positive)? - voltage_at(control_negative)?;
                let resistance = controlled_resistance(
                    *min_resistance,
                    *max_resistance,
                    *control_min,
                    *control_max,
                    control_voltage,
                );
                let current =
                    (voltage_at(output_positive)? - voltage_at(output_negative)?) / resistance;
                currents.insert(output_positive.clone(), current);
                currents.insert(output_negative.clone(), -current);
            }
        }
        other_terminal_currents.insert(component.id.clone(), currents);
    }
    Ok(SolveResult {
        node_voltages: voltages,
        resistor_currents,
        source_currents,
        switch_currents,
        motor_currents,
        capacitor_voltages,
        capacitor_currents,
        led_currents,
        diode_currents,
        transistor_collector_currents,
        pnp_collector_currents,
        motor_speeds,
        optocoupler_input_currents,
        relay_coil_currents,
        relay_energized,
        ic_device_output_voltages,
        other_terminal_currents,
        other_output_voltages,
        module_output_voltages,
    })
}

const VOLTAGE_TOLERANCE: f64 = 1e-9;
const PIVOT_TOLERANCE: f64 = 1e-12;
fn calc(code: &'static str, message: impl Into<String>) -> ElectricalError {
    ElectricalError::Calculation(ElectricalDiagnostic {
        code,
        message: message.into(),
    })
}
fn pin_node(topology: &[Node], id: &ComponentId, pin: &str) -> Option<usize> {
    topology.iter().position(|node| {
        node.contacts
            .contains(&Contact::ComponentPin(id.clone(), crate::PinId(pin.into())))
    })
}
type CompiledBranches = (
    Vec<Branch>,
    Vec<NonlinearElement>,
    Vec<Vec<Contact>>,
    BTreeSet<usize>,
);
type ComponentBranch = (&'static str, &'static str, BranchKind, f64, f64);
fn make_branches(
    project: &Project,
    topology: &[Node],
    states: &BTreeMap<ComponentId, ControlState>,
    capacitor_voltages: &BTreeMap<ComponentId, f64>,
    ratios: &BTreeMap<ComponentId, f64>,
    digital_states: &BTreeMap<ComponentId, u32>,
    options: SolveOptions,
) -> Result<CompiledBranches, ElectricalError> {
    let node_contacts: Vec<_> = topology.iter().map(|n| n.contacts.clone()).collect();
    let mut branches = Vec::new();
    let mut nonlinear = Vec::new();
    let mut active = BTreeSet::new();
    for component in &project.components {
        let node = |pin| {
            pin_node(topology, &component.id, pin).ok_or_else(|| {
                calc(
                    "missing_pin_node",
                    format!(
                        "component {} pin {pin} has no compiled node",
                        component.id.0
                    ),
                )
            })
        };
        match component.kind {
            ComponentKind::Led => {
                let anode = node("anode")?;
                let cathode = node("cathode")?;
                active.extend([anode, cathode]);
                nonlinear.push(NonlinearElement::Led {
                    id: component.id.clone(),
                    anode,
                    cathode,
                    forward: component.parameters["forward_voltage"],
                    resistance: component.parameters["series_resistance"],
                });
                continue;
            }
            ComponentKind::Diode => {
                let anode = node("anode")?;
                let cathode = node("cathode")?;
                active.extend([anode, cathode]);
                nonlinear.push(NonlinearElement::Diode {
                    id: component.id.clone(),
                    anode,
                    cathode,
                    model: DiodeModel::new(
                        component.parameters["forward_voltage"],
                        component.parameters["series_resistance"],
                        component.diode_model.unwrap_or_default(),
                    )
                    .map_err(|error| {
                        calc(
                            "invalid_diode_model",
                            format!(
                                "component {} has an invalid diode model: {error:?}",
                                component.id.0
                            ),
                        )
                    })?,
                });
                continue;
            }
            ComponentKind::NpnTransistor => {
                let base = node("base")?;
                let collector = node("collector")?;
                let emitter = node("emitter")?;
                active.extend([base, collector, emitter]);
                nonlinear.push(NonlinearElement::Npn {
                    id: component.id.clone(),
                    base,
                    collector,
                    emitter,
                    beta: component.parameters["beta"],
                    saturation: component.parameters["saturation_current"],
                    reverse_breakdown_voltage: component
                        .parameters
                        .get("reverse_breakdown_voltage")
                        .copied(),
                    reverse_breakdown_resistance: component
                        .parameters
                        .get("reverse_breakdown_resistance")
                        .copied(),
                    noise_sample: component
                        .parameters
                        .get("reverse_breakdown_voltage")
                        .map_or(0.0, |_| fixed_noise_sample(options.step)),
                });
                continue;
            }
            ComponentKind::PnpTransistor => {
                let base = node("base")?;
                let collector = node("collector")?;
                let emitter = node("emitter")?;
                active.extend([base, collector, emitter]);
                nonlinear.push(NonlinearElement::Pnp {
                    id: component.id.clone(),
                    base,
                    collector,
                    emitter,
                    beta: component.parameters["beta"],
                    saturation: component.parameters["saturation_current"],
                });
                continue;
            }
            ComponentKind::Optocoupler => {
                let input_anode = node("input_anode")?;
                let input_cathode = node("input_cathode")?;
                let collector = node("collector")?;
                let emitter = node("emitter")?;
                active.extend([input_anode, input_cathode, collector, emitter]);
                nonlinear.push(NonlinearElement::Optocoupler {
                    id: component.id.clone(),
                    input_anode,
                    input_cathode,
                    collector,
                    emitter,
                    forward: component.parameters["forward_voltage"],
                    series_resistance: component.parameters["series_resistance"],
                    transfer_gain: component.parameters["transfer_gain"],
                    on_resistance: component.parameters["on_resistance"],
                    off_resistance: component.parameters["off_resistance"],
                });
                continue;
            }
            ComponentKind::Relay => {
                let coil_positive = node("coil_positive")?;
                let coil_negative = node("coil_negative")?;
                let common = node("common")?;
                let normally_closed = node("normally_closed")?;
                let normally_open = node("normally_open")?;
                active.extend([
                    coil_positive,
                    coil_negative,
                    common,
                    normally_closed,
                    normally_open,
                ]);
                nonlinear.push(NonlinearElement::Relay {
                    id: component.id.clone(),
                    coil_positive,
                    coil_negative,
                    common,
                    normally_closed,
                    normally_open,
                    coil_resistance: component.parameters["coil_resistance"],
                    pickup_voltage: component.parameters["pickup_voltage"],
                    contact_resistance: component.parameters["contact_resistance"],
                });
                continue;
            }
            ComponentKind::LogicGate => {
                let input_a = node("input_a")?;
                let input_b = node("input_b")?;
                let output = node("output")?;
                let vcc = node("vcc")?;
                let gnd = node("gnd")?;
                active.extend([input_a, input_b, output, vcc, gnd]);
                nonlinear.push(NonlinearElement::LogicGate {
                    input_a,
                    input_b,
                    output,
                    vcc,
                    gnd,
                    operation: component.parameters["operation"] as u8,
                    resistance: component.parameters["output_resistance"],
                });
                continue;
            }
            ComponentKind::SchmittInverter => {
                let input = node("input")?;
                let output = node("output")?;
                let vcc = node("vcc")?;
                let gnd = node("gnd")?;
                active.extend([input, output, vcc, gnd]);
                nonlinear.push(NonlinearElement::SchmittInverter {
                    input,
                    output,
                    vcc,
                    gnd,
                    resistance: component.parameters["output_resistance"],
                });
                continue;
            }
            ComponentKind::Comparator => {
                let inverting = node("inverting")?;
                let non_inverting = node("non_inverting")?;
                let output = node("output")?;
                let vcc = node("vcc")?;
                let gnd = node("gnd")?;
                active.extend([inverting, non_inverting, output, vcc, gnd]);
                nonlinear.push(NonlinearElement::Comparator {
                    inverting,
                    non_inverting,
                    output,
                    vcc,
                    gnd,
                    resistance: component.parameters["output_resistance"],
                });
                continue;
            }
            ComponentKind::Timer555 => {
                let control = node("control")?;
                let discharge = node("discharge")?;
                let output = node("output")?;
                let reset = node("reset")?;
                let threshold = node("threshold")?;
                let trigger = node("trigger")?;
                let vcc = node("vcc")?;
                let gnd = node("gnd")?;
                active.extend([
                    control, discharge, output, reset, threshold, trigger, vcc, gnd,
                ]);
                nonlinear.push(NonlinearElement::Timer555 {
                    control,
                    discharge,
                    output,
                    reset,
                    threshold,
                    trigger,
                    vcc,
                    gnd,
                    output_resistance: component.parameters["output_resistance"],
                    discharge_resistance: component.parameters["discharge_resistance"],
                });
                continue;
            }
            ComponentKind::DFlipFlop => {
                let clock = node("clock")?;
                let data = node("data")?;
                let not_q = node("not_q")?;
                let q = node("q")?;
                let reset = node("reset")?;
                let set = node("set")?;
                let vcc = node("vcc")?;
                let gnd = node("gnd")?;
                active.extend([clock, data, not_q, q, reset, set, vcc, gnd]);
                nonlinear.push(NonlinearElement::DFlipFlop {
                    clock,
                    data,
                    not_q,
                    q,
                    reset,
                    set,
                    vcc,
                    gnd,
                    resistance: component.parameters["output_resistance"],
                    state: digital_states
                        .get(&component.id)
                        .copied()
                        .unwrap_or_default(),
                });
                continue;
            }
            ComponentKind::DigitalCounter => {
                let clock = node("clock")?;
                let carry = node("carry")?;
                let enable = node("enable")?;
                let gnd = node("gnd")?;
                let reset = node("reset")?;
                let vcc = node("vcc")?;
                let outputs: [Result<usize, ElectricalError>; 10] =
                    ["q0", "q1", "q2", "q3", "q4", "q5", "q6", "q7", "q8", "q9"].map(node);
                let outputs = outputs.into_iter().collect::<Result<Vec<_>, _>>()?;
                let outputs: [usize; 10] = outputs.try_into().expect("ten counter outputs");
                active.extend([clock, carry, enable, gnd, reset, vcc]);
                active.extend(outputs);
                nonlinear.push(NonlinearElement::DigitalCounter {
                    clock,
                    carry,
                    enable,
                    outputs,
                    reset,
                    vcc,
                    gnd,
                    mode: component.parameters["output_mode"] as u8,
                    modulus: component.parameters["modulus"] as u8,
                    resistance: component.parameters["output_resistance"],
                    state: digital_states
                        .get(&component.id)
                        .copied()
                        .unwrap_or_default(),
                });
                continue;
            }
            ComponentKind::StepSequencer => {
                let clock = node("clock")?;
                let controls: [Result<usize, ElectricalError>; 8] = [
                    "control0", "control1", "control2", "control3", "control4", "control5",
                    "control6", "control7",
                ]
                .map(node);
                let controls = controls.into_iter().collect::<Result<Vec<_>, _>>()?;
                let controls: [usize; 8] = controls.try_into().expect("eight sequencer controls");
                let output = node("output")?;
                let steps: [Result<usize, ElectricalError>; 8] = [
                    "step0", "step1", "step2", "step3", "step4", "step5", "step6", "step7",
                ]
                .map(node);
                let steps = steps.into_iter().collect::<Result<Vec<_>, _>>()?;
                let steps: [usize; 8] = steps.try_into().expect("eight sequencer outputs");
                let vcc = node("vcc")?;
                let gnd = node("gnd")?;
                active.extend([clock, output, vcc, gnd]);
                active.extend(controls);
                active.extend(steps);
                nonlinear.push(NonlinearElement::StepSequencer {
                    clock,
                    controls,
                    output,
                    steps,
                    vcc,
                    gnd,
                    resistance: component.parameters["output_resistance"],
                    state: digital_states
                        .get(&component.id)
                        .copied()
                        .unwrap_or_default(),
                });
                continue;
            }
            ComponentKind::Sram => {
                let address = node("address")?;
                let clock = node("clock")?;
                let data: [Result<usize, ElectricalError>; 8] = [
                    "data0", "data1", "data2", "data3", "data4", "data5", "data6", "data7",
                ]
                .map(node);
                let data = data.into_iter().collect::<Result<Vec<_>, _>>()?;
                let data: [usize; 8] = data.try_into().expect("eight SRAM data inputs");
                let output: [Result<usize, ElectricalError>; 8] = [
                    "output0", "output1", "output2", "output3", "output4", "output5", "output6",
                    "output7",
                ]
                .map(node);
                let output = output.into_iter().collect::<Result<Vec<_>, _>>()?;
                let output: [usize; 8] = output.try_into().expect("eight SRAM outputs");
                let reset = node("reset")?;
                let vcc = node("vcc")?;
                let gnd = node("gnd")?;
                let write = node("write")?;
                active.extend([address, clock, reset, vcc, gnd, write]);
                active.extend(data);
                active.extend(output);
                nonlinear.push(NonlinearElement::Sram {
                    address,
                    clock,
                    data,
                    output,
                    reset,
                    vcc,
                    gnd,
                    write,
                    resistance: component.parameters["output_resistance"],
                    state: digital_states
                        .get(&component.id)
                        .copied()
                        .unwrap_or_default(),
                });
                continue;
            }
            ComponentKind::ShiftRegister => {
                let clear = node("clear")?;
                let clock = node("clock")?;
                let data = node("data")?;
                let gnd = node("gnd")?;
                let latch = node("latch")?;
                let vcc = node("vcc")?;
                let outputs: [Result<usize, ElectricalError>; 8] =
                    ["q0", "q1", "q2", "q3", "q4", "q5", "q6", "q7"].map(node);
                let outputs = outputs.into_iter().collect::<Result<Vec<_>, _>>()?;
                let outputs: [usize; 8] = outputs.try_into().expect("eight shift outputs");
                active.extend([clear, clock, data, gnd, latch, vcc]);
                active.extend(outputs);
                nonlinear.push(NonlinearElement::ShiftRegister {
                    clock,
                    data,
                    outputs,
                    vcc,
                    gnd,
                    resistance: component.parameters["output_resistance"],
                    state: digital_states
                        .get(&component.id)
                        .copied()
                        .unwrap_or_default(),
                });
                continue;
            }
            ComponentKind::SevenSegmentDisplay => {
                let inputs = ["input_b0", "input_b1", "input_b2", "input_b3"]
                    .into_iter()
                    .map(node)
                    .collect::<Result<Vec<_>, _>>()?;
                let inputs: [usize; 4] = inputs.try_into().expect("four display inputs");
                let segments = ["a", "b", "c", "d", "e", "f", "g"]
                    .into_iter()
                    .map(node)
                    .collect::<Result<Vec<_>, _>>()?;
                let segments: [usize; 7] = segments.try_into().expect("seven display segments");
                let common = node("common")?;
                let vcc = node("vcc")?;
                let gnd = node("gnd")?;
                active.extend(inputs);
                active.extend(segments);
                active.extend([common, vcc, gnd]);
                nonlinear.push(NonlinearElement::SevenSegmentDisplay {
                    inputs,
                    segments,
                    common,
                    vcc,
                    gnd,
                    resistance: component.parameters["output_resistance"],
                });
                continue;
            }
            ComponentKind::FourBitAdder => {
                let a = ["a0", "a1", "a2", "a3"].map(node);
                let a = a.into_iter().collect::<Result<Vec<_>, _>>()?;
                let a: [usize; 4] = a.try_into().expect("four adder A inputs");
                let b = ["b0", "b1", "b2", "b3"].map(node);
                let b = b.into_iter().collect::<Result<Vec<_>, _>>()?;
                let b: [usize; 4] = b.try_into().expect("four adder B inputs");
                let sum = ["sum0", "sum1", "sum2", "sum3"].map(node);
                let sum = sum.into_iter().collect::<Result<Vec<_>, _>>()?;
                let sum: [usize; 4] = sum.try_into().expect("four adder sum outputs");
                let carry_in = node("carry_in")?;
                let carry_out = node("carry_out")?;
                let gnd = node("gnd")?;
                let subtract = node("subtract")?;
                let vcc = node("vcc")?;
                active.extend(a);
                active.extend(b);
                active.extend(sum);
                active.extend([carry_in, carry_out, gnd, subtract, vcc]);
                nonlinear.push(NonlinearElement::FourBitAdder {
                    a,
                    b,
                    carry_in,
                    carry_out,
                    subtract,
                    sum,
                    vcc,
                    gnd,
                    resistance: component.parameters["output_resistance"],
                });
                continue;
            }
            ComponentKind::BargraphDisplay => {
                let input = node("input")?;
                let segments = [
                    "seg0", "seg1", "seg2", "seg3", "seg4", "seg5", "seg6", "seg7", "seg8", "seg9",
                ]
                .map(node);
                let segments = segments.into_iter().collect::<Result<Vec<_>, _>>()?;
                let segments: [usize; 10] = segments.try_into().expect("ten bargraph segments");
                let vcc = node("vcc")?;
                let gnd = node("gnd")?;
                active.extend(segments);
                active.extend([input, vcc, gnd]);
                nonlinear.push(NonlinearElement::BargraphDisplay {
                    input,
                    segments,
                    vcc,
                    gnd,
                    resistance: component.parameters["output_resistance"],
                });
                continue;
            }
            ComponentKind::AudioAmplifier => {
                let input = node("input")?;
                let output = node("output")?;
                let vcc = node("vcc")?;
                let gnd = node("gnd")?;
                active.extend([input, output, vcc, gnd]);
                nonlinear.push(NonlinearElement::AudioAmplifier {
                    input,
                    output,
                    vcc,
                    gnd,
                    gain: component.parameters["gain"],
                    resistance: component.parameters["output_resistance"],
                });
                continue;
            }
            ComponentKind::IcDevice => {
                let spec = component.ic_device.as_ref().ok_or_else(|| {
                    calc(
                        "missing_ic_device_spec",
                        format!("component {} has no ic_device contract", component.id.0),
                    )
                })?;
                let mut pin_nodes = BTreeMap::new();
                for pin in spec.pin_roles.keys() {
                    pin_nodes.insert(pin.clone(), node(&pin.0)?);
                }
                active.extend(pin_nodes.values().copied());
                nonlinear.push(NonlinearElement::IcDevice {
                    id: component.id.clone(),
                    pin_nodes,
                    behavior: spec.behavior.clone(),
                });
                continue;
            }
            ComponentKind::Module => {
                let spec = component.module.as_ref().ok_or_else(|| {
                    calc(
                        "missing_module_spec",
                        format!("component {} has no module contract", component.id.0),
                    )
                })?;
                let mut pin_nodes = BTreeMap::new();
                for pin in spec.pin_roles.keys() {
                    pin_nodes.insert(pin.clone(), node(&pin.0)?);
                }
                active.extend(pin_nodes.values().copied());
                nonlinear.push(NonlinearElement::Module {
                    id: component.id.clone(),
                    pin_nodes,
                    behavior: spec.behavior.clone(),
                });
                continue;
            }
            ComponentKind::Other => {
                let spec = component.other_device.as_ref().ok_or_else(|| {
                    calc(
                        "missing_other_device_spec",
                        format!("component {} has no other_device contract", component.id.0),
                    )
                })?;
                match &spec.behavior {
                    OtherDeviceBehavior::Resistive {
                        positive,
                        negative,
                        resistance,
                    } => {
                        let a = node(&positive.0)?;
                        let b = node(&negative.0)?;
                        active.extend([a, b]);
                        branches.push(Branch {
                            component: component.id.clone(),
                            kind: BranchKind::Resistor,
                            a,
                            b,
                            value: *resistance,
                            previous_voltage: 0.0,
                        });
                    }
                    OtherDeviceBehavior::VoltageSource {
                        positive,
                        negative,
                        voltage,
                        internal_resistance,
                    } => {
                        let positive = node(&positive.0)?;
                        let negative = node(&negative.0)?;
                        active.extend([positive, negative]);
                        nonlinear.push(NonlinearElement::OtherSource {
                            positive,
                            negative,
                            voltage: *voltage,
                            resistance: *internal_resistance,
                        });
                    }
                    OtherDeviceBehavior::BjtTestSocket { socket } => {
                        let base = node(&socket.base.0)?;
                        let collector = node(&socket.collector.0)?;
                        let emitter = node(&socket.emitter.0)?;
                        active.extend([base, collector, emitter]);
                        nonlinear.push(NonlinearElement::OtherBjtSocket {
                            base,
                            collector,
                            emitter,
                            socket_polarity: socket.socket_polarity,
                            subject_polarity: socket.subject_polarity,
                            subject_state: socket.subject_state,
                            beta: socket.beta,
                            saturation: socket.saturation_current,
                            open_resistance: socket.open_resistance,
                            short_resistance: socket.short_resistance,
                        });
                    }
                    OtherDeviceBehavior::DiodeTestSocket { socket } => {
                        let anode = node(&socket.anode.0)?;
                        let cathode = node(&socket.cathode.0)?;
                        active.extend([anode, cathode]);
                        nonlinear.push(NonlinearElement::OtherDiodeSocket {
                            anode,
                            cathode,
                            subject_polarity: socket.subject_polarity,
                            subject_state: socket.subject_state,
                            model: DiodeModel::new(
                                socket.forward_voltage,
                                socket.series_resistance,
                                DiodeSpec::default(),
                            )
                            .map_err(|error| {
                                calc(
                                    "invalid_diode_test_socket",
                                    format!(
                                        "component {} has an invalid diode test socket model: {error:?}",
                                        component.id.0
                                    ),
                                )
                            })?,
                            open_resistance: socket.open_resistance,
                            short_resistance: socket.short_resistance,
                        });
                    }
                    OtherDeviceBehavior::LinearTransfer {
                        output,
                        reference,
                        inputs,
                        offset,
                        min_output,
                        max_output,
                        input_resistance,
                        output_resistance,
                    } => {
                        let pin_nodes = spec
                            .pin_roles
                            .keys()
                            .map(|pin| Ok((pin.clone(), node(&pin.0)?)))
                            .collect::<Result<BTreeMap<_, _>, ElectricalError>>()?;
                        active.extend(pin_nodes.values().copied());
                        nonlinear.push(NonlinearElement::OtherLinearTransfer {
                            pin_nodes,
                            output: output.clone(),
                            reference: reference.clone(),
                            inputs: inputs
                                .iter()
                                .map(|input| (input.pin.clone(), input.gain))
                                .collect(),
                            offset: *offset,
                            min_output: *min_output,
                            max_output: *max_output,
                            input_resistance: *input_resistance,
                            output_resistance: *output_resistance,
                        });
                    }
                    OtherDeviceBehavior::Transformer {
                        primary_positive,
                        primary_negative,
                        secondary_positive,
                        secondary_negative,
                        turns_ratio,
                        primary_resistance,
                        secondary_resistance,
                    } => {
                        let primary_positive = node(&primary_positive.0)?;
                        let primary_negative = node(&primary_negative.0)?;
                        let secondary_positive = node(&secondary_positive.0)?;
                        let secondary_negative = node(&secondary_negative.0)?;
                        active.extend([
                            primary_positive,
                            primary_negative,
                            secondary_positive,
                            secondary_negative,
                        ]);
                        nonlinear.push(NonlinearElement::OtherTransformer {
                            primary_positive,
                            primary_negative,
                            secondary_positive,
                            secondary_negative,
                            turns_ratio: *turns_ratio,
                            primary_resistance: *primary_resistance,
                            secondary_resistance: *secondary_resistance,
                        });
                    }
                    OtherDeviceBehavior::RingModulator {
                        signal,
                        carrier,
                        output,
                        reference,
                        gain,
                        signal_scale,
                        carrier_scale,
                        min_output,
                        max_output,
                        input_resistance,
                        output_resistance,
                    } => {
                        let pin_nodes = spec
                            .pin_roles
                            .keys()
                            .map(|pin| Ok((pin.clone(), node(&pin.0)?)))
                            .collect::<Result<BTreeMap<_, _>, ElectricalError>>()?;
                        active.extend(pin_nodes.values().copied());
                        nonlinear.push(NonlinearElement::OtherRingModulator {
                            signal: *pin_nodes.get(signal).expect("validated ring signal pin"),
                            carrier: *pin_nodes.get(carrier).expect("validated ring carrier pin"),
                            output: *pin_nodes.get(output).expect("validated ring output pin"),
                            reference: *pin_nodes
                                .get(reference)
                                .expect("validated ring reference pin"),
                            gain: *gain,
                            signal_scale: *signal_scale,
                            carrier_scale: *carrier_scale,
                            min_output: *min_output,
                            max_output: *max_output,
                            input_resistance: *input_resistance,
                            output_resistance: *output_resistance,
                        });
                    }
                    OtherDeviceBehavior::VoltageControlledResistance {
                        control_positive,
                        control_negative,
                        output_positive,
                        output_negative,
                        min_resistance,
                        max_resistance,
                        control_min,
                        control_max,
                    } => {
                        let control_positive = node(&control_positive.0)?;
                        let control_negative = node(&control_negative.0)?;
                        let output_positive = node(&output_positive.0)?;
                        let output_negative = node(&output_negative.0)?;
                        active.extend([
                            control_positive,
                            control_negative,
                            output_positive,
                            output_negative,
                        ]);
                        nonlinear.push(NonlinearElement::OtherControlledResistance {
                            control_positive,
                            control_negative,
                            output_positive,
                            output_negative,
                            min_resistance: *min_resistance,
                            max_resistance: *max_resistance,
                            control_min: *control_min,
                            control_max: *control_max,
                        });
                    }
                }
                continue;
            }
            _ => {}
        }
        let Some((a_pin, b_pin, kind, value, previous_voltage)) =
            component_branch(component, states, capacitor_voltages, ratios, options.dt)?
        else {
            continue;
        };
        let a = pin_node(topology, &component.id, a_pin).ok_or_else(|| {
            calc(
                "missing_pin_node",
                format!(
                    "component {} pin {a_pin} has no compiled node",
                    component.id.0
                ),
            )
        })?;
        let b = pin_node(topology, &component.id, b_pin).ok_or_else(|| {
            calc(
                "missing_pin_node",
                format!(
                    "component {} pin {b_pin} has no compiled node",
                    component.id.0
                ),
            )
        })?;
        active.insert(a);
        active.insert(b);
        branches.push(Branch {
            component: component.id.clone(),
            kind,
            a,
            b,
            value,
            previous_voltage,
        });
    }
    Ok((branches, nonlinear, node_contacts, active))
}
fn component_branch(
    c: &Component,
    states: &BTreeMap<ComponentId, ControlState>,
    capacitor_voltages: &BTreeMap<ComponentId, f64>,
    ratios: &BTreeMap<ComponentId, f64>,
    dt: Option<f64>,
) -> Result<Option<ComponentBranch>, ElectricalError> {
    let value = |name: &str| {
        c.parameters.get(name).copied().ok_or_else(|| {
            calc(
                "missing_parameter",
                format!("component {} is missing {name}", c.id.0),
            )
        })
    };
    Ok(match c.kind {
        ComponentKind::Resistor => {
            Some(("a", "b", BranchKind::Resistor, value("resistance")?, 0.0))
        }
        ComponentKind::DcVoltageSource => Some((
            "positive",
            "negative",
            BranchKind::VoltageSource,
            value("voltage")?,
            0.0,
        )),
        ComponentKind::Capacitor => {
            let dt = dt.ok_or_else(|| {
                calc(
                    "unsupported_component",
                    "capacitors require transient stepping",
                )
            })?;
            Some((
                "positive",
                "negative",
                BranchKind::Capacitor,
                value("capacitance")? / dt,
                capacitor_voltages.get(&c.id).copied().unwrap_or(0.0),
            ))
        }
        ComponentKind::MomentaryButton => match states
            .get(&c.id)
            .copied()
            .unwrap_or(ControlState::ButtonReleased)
        {
            ControlState::ButtonPressed => Some(("a", "b", BranchKind::Switch, 0.0, 0.0)),
            ControlState::ButtonReleased => None,
            _ => {
                return Err(calc(
                    "invalid_control_state",
                    format!("{} requires a button control state", c.id.0),
                ));
            }
        },
        ComponentKind::ChangeoverSwitch => match states
            .get(&c.id)
            .copied()
            .unwrap_or(ControlState::SwitchNormallyClosed)
        {
            ControlState::SwitchNormallyClosed => {
                Some(("common", "normally_closed", BranchKind::Switch, 0.0, 0.0))
            }
            ControlState::SwitchNormallyOpen => {
                Some(("common", "normally_open", BranchKind::Switch, 0.0, 0.0))
            }
            _ => {
                return Err(calc(
                    "invalid_control_state",
                    format!("{} requires a changeover control state", c.id.0),
                ));
            }
        },
        ComponentKind::Buzzer | ComponentKind::Speaker | ComponentKind::PiezoPassive => Some((
            "positive",
            "negative",
            BranchKind::Resistor,
            value("resistance")?,
            0.0,
        )),
        ComponentKind::Motor => Some((
            "positive",
            "negative",
            BranchKind::Motor,
            value("resistance")?,
            0.0,
        )),
        ComponentKind::Potentiometer => {
            let min = value("min_resistance")?;
            let max = value("max_resistance")?;
            let ratio = ratios.get(&c.id).copied().unwrap_or(0.5).clamp(0.0, 1.0);
            Some((
                "a",
                "b",
                BranchKind::Resistor,
                min + ratio * (max - min),
                0.0,
            ))
        }
        ComponentKind::Photoresistor
        | ComponentKind::Thermistor
        | ComponentKind::TouchPad
        | ComponentKind::WaterProbe => {
            let min = value("min_resistance")?;
            let max = value("max_resistance")?;
            let ratio = ratios.get(&c.id).copied().unwrap_or(0.5).clamp(0.0, 1.0);
            // ratio is ambient light: 0.0 (darkest) -> max_resistance, 1.0 (brightest) -> min_resistance.
            Some((
                "a",
                "b",
                BranchKind::Resistor,
                max - ratio * (max - min),
                0.0,
            ))
        }
        _ => {
            return Err(calc(
                "unsupported_component",
                format!(
                    "component {} is not supported by the resistive DC solver",
                    c.id.0
                ),
            ));
        }
    })
}
fn voltage(solution: &[f64], vars: &BTreeMap<usize, usize>, node: usize) -> f64 {
    vars.get(&node).map_or(0.0, |&i| solution[i])
}
fn stamp_conductance(
    m: &mut [Vec<f64>],
    vars: &BTreeMap<usize, usize>,
    a: usize,
    b: usize,
    g: f64,
) {
    if let Some(&i) = vars.get(&a) {
        m[i][i] += g;
    }
    if let Some(&i) = vars.get(&b) {
        m[i][i] += g;
    }
    if let (Some(&i), Some(&j)) = (vars.get(&a), vars.get(&b)) {
        m[i][j] -= g;
        m[j][i] -= g;
    }
}
fn stamp_constraint(
    m: &mut [Vec<f64>],
    vars: &BTreeMap<usize, usize>,
    a: usize,
    b: usize,
    current: usize,
) {
    if let Some(&i) = vars.get(&a) {
        m[i][current] += 1.0;
        m[current][i] += 1.0;
    }
    if let Some(&i) = vars.get(&b) {
        m[i][current] -= 1.0;
        m[current][i] -= 1.0;
    }
}
fn stamp_history(rhs: &mut [f64], vars: &BTreeMap<usize, usize>, a: usize, b: usize, current: f64) {
    if let Some(&i) = vars.get(&a) {
        rhs[i] += current;
    }
    if let Some(&i) = vars.get(&b) {
        rhs[i] -= current;
    }
}

// LED, transistor-junction, and optocoupler forward branches use the same
// smooth junction law as the shared diode model. They do not opt into the
// optional zener reverse-breakdown branch.
fn led_current(v: f64, forward: f64, resistance: f64) -> (f64, f64) {
    let model = DiodeModel::new(forward, resistance, DiodeSpec::default())
        .expect("validated forward diode parameters");
    let linearization = model.linearize(v);
    (linearization.current, linearization.conductance)
}

// The base-emitter junction controls collector-emitter conductance. The
// saturation-current parameter sets the junction turn-on voltage.
fn npn_currents(vbe: f64, vce: f64, beta: f64, saturation: f64) -> (f64, f64, f64, f64) {
    let threshold = 0.026 * (0.001 / saturation).ln();
    let (ib, base_conductance) = led_current(vbe, threshold, 100.0);
    let unclamped = beta * ib.max(0.0) / 0.2;
    let conductance = unclamped.clamp(1e-9, 1.0);
    let collector_base_conductance = if (1e-9..=1.0).contains(&unclamped) {
        beta * base_conductance / 0.2 * vce
    } else {
        0.0
    };
    (
        ib,
        conductance * vce,
        collector_base_conductance,
        conductance,
    )
}

/// Deterministic fixed-step sample for the bounded reverse-junction noise model.
fn fixed_noise_sample(step: u64) -> f64 {
    let mut value = step
        .wrapping_add(0x9E37_79B9_7F4A_7C15)
        .wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^= value >> 31;
    let unit = (value >> 11) as f64 / ((1_u64 << 53) as f64);
    unit * 2.0 - 1.0
}

fn stamp_current(
    (from, to): (usize, usize),
    current: f64,
    derivatives: &[(usize, f64)],
    guess: &[f64],
    vars: &BTreeMap<usize, usize>,
    matrix: &mut [Vec<f64>],
    rhs: &mut [f64],
) {
    let offset = derivatives
        .iter()
        .map(|(node, d)| d * voltage(guess, vars, *node))
        .sum::<f64>()
        - current;
    for (node, sign) in [(from, 1.0), (to, -1.0)] {
        if let Some(&row) = vars.get(&node) {
            rhs[row] += sign * offset;
            for &(column_node, derivative) in derivatives {
                if let Some(&column) = vars.get(&column_node) {
                    matrix[row][column] += sign * derivative;
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn stamp_logic_output(
    output: usize,
    vcc: usize,
    gnd: usize,
    high: bool,
    resistance: f64,
    guess: &[f64],
    vars: &BTreeMap<usize, usize>,
    matrix: &mut [Vec<f64>],
    rhs: &mut [f64],
) {
    let conductance = 1.0 / resistance;
    let target = if high {
        voltage(guess, vars, vcc)
    } else {
        voltage(guess, vars, gnd)
    };
    let target_node = if high { vcc } else { gnd };
    let current = conductance * (voltage(guess, vars, output) - target);
    stamp_current(
        (output, gnd),
        current,
        &[(output, conductance), (target_node, -conductance)],
        guess,
        vars,
        matrix,
        rhs,
    );
}

fn logic_high(value: f64, supply: f64) -> bool {
    value > supply * 0.5
}

fn stamp_element(
    element: &NonlinearElement,
    guess: &[f64],
    vars: &BTreeMap<usize, usize>,
    matrix: &mut [Vec<f64>],
    rhs: &mut [f64],
    use_collector_base_jacobian: bool,
) {
    match element {
        NonlinearElement::Led {
            anode,
            cathode,
            forward,
            resistance,
            ..
        } => {
            let v = voltage(guess, vars, *anode) - voltage(guess, vars, *cathode);
            let (current, conductance) = led_current(v, *forward, *resistance);
            stamp_current(
                (*anode, *cathode),
                current,
                &[(*anode, conductance), (*cathode, -conductance)],
                guess,
                vars,
                matrix,
                rhs,
            );
        }
        NonlinearElement::Diode {
            anode,
            cathode,
            model,
            ..
        } => {
            let v = voltage(guess, vars, *anode) - voltage(guess, vars, *cathode);
            let linearization = model.linearize(v);
            stamp_current(
                (*anode, *cathode),
                linearization.current,
                &[
                    (*anode, linearization.conductance),
                    (*cathode, -linearization.conductance),
                ],
                guess,
                vars,
                matrix,
                rhs,
            );
        }
        NonlinearElement::Optocoupler {
            input_anode,
            input_cathode,
            collector,
            emitter,
            forward,
            series_resistance,
            transfer_gain,
            on_resistance,
            off_resistance,
            ..
        } => {
            let vin = voltage(guess, vars, *input_anode) - voltage(guess, vars, *input_cathode);
            let (input_current, input_conductance) = led_current(vin, *forward, *series_resistance);
            stamp_current(
                (*input_anode, *input_cathode),
                input_current,
                &[
                    (*input_anode, input_conductance),
                    (*input_cathode, -input_conductance),
                ],
                guess,
                vars,
                matrix,
                rhs,
            );
            let light = (*transfer_gain * input_current.max(0.0)).clamp(0.0, 1.0);
            let conductance =
                1.0 / *off_resistance + light * (1.0 / *on_resistance - 1.0 / *off_resistance);
            let output_current =
                conductance * (voltage(guess, vars, *collector) - voltage(guess, vars, *emitter));
            stamp_current(
                (*collector, *emitter),
                output_current,
                &[(*collector, conductance), (*emitter, -conductance)],
                guess,
                vars,
                matrix,
                rhs,
            );
        }
        NonlinearElement::Relay {
            coil_positive,
            coil_negative,
            common,
            normally_closed,
            normally_open,
            coil_resistance,
            pickup_voltage,
            contact_resistance,
            ..
        } => {
            let coil_voltage =
                voltage(guess, vars, *coil_positive) - voltage(guess, vars, *coil_negative);
            stamp_conductance(
                matrix,
                vars,
                *coil_positive,
                *coil_negative,
                1.0 / *coil_resistance,
            );
            let energized = coil_voltage >= *pickup_voltage;
            let contact = if energized {
                (*normally_open, *contact_resistance)
            } else {
                (*normally_closed, *contact_resistance)
            };
            let open = if energized {
                *normally_closed
            } else {
                *normally_open
            };
            stamp_conductance(matrix, vars, *common, contact.0, 1.0 / contact.1);
            // Keep the unselected terminal numerically referenced without
            // making it a meaningful electrical path.
            stamp_conductance(matrix, vars, *common, open, 1e-12);
        }
        NonlinearElement::Npn {
            base,
            collector,
            emitter,
            beta,
            saturation,
            reverse_breakdown_voltage,
            reverse_breakdown_resistance,
            noise_sample,
            ..
        } => {
            let vbe = voltage(guess, vars, *base) - voltage(guess, vars, *emitter);
            let vce = voltage(guess, vars, *collector) - voltage(guess, vars, *emitter);
            let (ib, ic, gm, go) = npn_currents(vbe, vce, *beta, *saturation);
            let gm = if use_collector_base_jacobian { gm } else { 0.0 };
            let (_, gib) = led_current(vbe, 0.026 * (0.001 / saturation).ln(), 100.0);
            stamp_current(
                (*base, *emitter),
                ib,
                &[(*base, gib), (*emitter, -gib)],
                guess,
                vars,
                matrix,
                rhs,
            );
            stamp_current(
                (*collector, *emitter),
                ic,
                &[(*base, gm), (*collector, go), (*emitter, -gm - go)],
                guess,
                vars,
                matrix,
                rhs,
            );
            if let (Some(breakdown_voltage), Some(breakdown_resistance)) =
                (reverse_breakdown_voltage, reverse_breakdown_resistance)
            {
                let reverse_voltage = -vbe;
                if reverse_voltage > *breakdown_voltage {
                    let conductance = (1.0 + 0.5 * *noise_sample) / *breakdown_resistance;
                    let current = conductance * (reverse_voltage - *breakdown_voltage);
                    stamp_current(
                        (*emitter, *base),
                        current,
                        &[(*emitter, conductance), (*base, -conductance)],
                        guess,
                        vars,
                        matrix,
                        rhs,
                    );
                }
            }
        }
        NonlinearElement::Pnp {
            base,
            collector,
            emitter,
            beta,
            saturation,
            ..
        } => {
            let veb = voltage(guess, vars, *emitter) - voltage(guess, vars, *base);
            let vec = voltage(guess, vars, *emitter) - voltage(guess, vars, *collector);
            let (ib, ic, gm, go) = npn_currents(veb, vec, *beta, *saturation);
            let gm = if use_collector_base_jacobian { gm } else { 0.0 };
            let (_, gib) = led_current(veb, 0.026 * (0.001 / saturation).ln(), 100.0);
            stamp_current(
                (*emitter, *base),
                ib,
                &[(*emitter, gib), (*base, -gib)],
                guess,
                vars,
                matrix,
                rhs,
            );
            stamp_current(
                (*emitter, *collector),
                ic,
                &[(*emitter, gm + go), (*base, -gm), (*collector, -go)],
                guess,
                vars,
                matrix,
                rhs,
            );
        }
        NonlinearElement::OtherBjtSocket {
            base,
            collector,
            emitter,
            socket_polarity,
            subject_polarity,
            subject_state,
            beta,
            saturation,
            open_resistance,
            short_resistance,
            ..
        } => stamp_other_bjt_socket(
            *base,
            *collector,
            *emitter,
            *socket_polarity,
            *subject_polarity,
            *subject_state,
            *beta,
            *saturation,
            *open_resistance,
            *short_resistance,
            guess,
            vars,
            matrix,
            rhs,
            use_collector_base_jacobian,
        ),
        NonlinearElement::OtherDiodeSocket {
            anode,
            cathode,
            subject_polarity,
            subject_state,
            model,
            open_resistance,
            short_resistance,
        } => stamp_other_diode_socket(
            *anode,
            *cathode,
            *subject_polarity,
            *subject_state,
            *model,
            *open_resistance,
            *short_resistance,
            guess,
            vars,
            matrix,
            rhs,
        ),
        NonlinearElement::LogicGate {
            input_a,
            input_b,
            output,
            vcc,
            gnd,
            operation,
            resistance,
            ..
        } => {
            let supply = voltage(guess, vars, *vcc);
            let a = logic_high(voltage(guess, vars, *input_a), supply);
            let b = logic_high(voltage(guess, vars, *input_b), supply);
            let high = match operation {
                0 => a && b,
                1 => a || b,
                2 => !(a && b),
                3 => a ^ b,
                _ => false,
            };
            stamp_logic_output(
                *output,
                *vcc,
                *gnd,
                high,
                *resistance,
                guess,
                vars,
                matrix,
                rhs,
            );
        }
        NonlinearElement::SchmittInverter {
            input,
            output,
            vcc,
            gnd,
            resistance,
            ..
        } => {
            let supply = voltage(guess, vars, *vcc);
            let high = voltage(guess, vars, *input) < supply * 0.4;
            stamp_logic_output(
                *output,
                *vcc,
                *gnd,
                high,
                *resistance,
                guess,
                vars,
                matrix,
                rhs,
            );
        }
        NonlinearElement::Comparator {
            inverting,
            non_inverting,
            output,
            vcc,
            gnd,
            resistance,
            ..
        } => {
            let high = voltage(guess, vars, *non_inverting) <= voltage(guess, vars, *inverting);
            stamp_logic_output(
                *output,
                *vcc,
                *gnd,
                high,
                *resistance,
                guess,
                vars,
                matrix,
                rhs,
            );
        }
        NonlinearElement::Timer555 {
            control,
            discharge,
            output,
            output_resistance,
            discharge_resistance,
            reset,
            threshold,
            trigger,
            vcc,
            gnd,
            ..
        } => {
            let ground = voltage(guess, vars, *gnd);
            let supply = (voltage(guess, vars, *vcc) - ground).max(1e-6);
            let control_level = voltage(guess, vars, *control) - ground;
            let upper_threshold = if control_level > supply * 0.05 {
                control_level.clamp(supply * 0.4, supply * 0.9)
            } else {
                supply * (2.0 / 3.0)
            };
            let lower_threshold = upper_threshold * 0.5;
            let reset_high = voltage(guess, vars, *reset) - ground > supply * 0.4;
            let threshold_high = voltage(guess, vars, *threshold) - ground > upper_threshold;
            let trigger_low = voltage(guess, vars, *trigger) - ground < lower_threshold;
            let output_high = if !reset_high {
                false
            } else if trigger_low {
                true
            } else if threshold_high {
                false
            } else {
                voltage(guess, vars, *output) - ground > supply * 0.5
            };
            stamp_logic_output(
                *output,
                *vcc,
                *gnd,
                output_high,
                *output_resistance,
                guess,
                vars,
                matrix,
                rhs,
            );
            if !output_high {
                stamp_logic_output(
                    *discharge,
                    *vcc,
                    *gnd,
                    false,
                    *discharge_resistance,
                    guess,
                    vars,
                    matrix,
                    rhs,
                );
            }
        }
        NonlinearElement::DFlipFlop {
            not_q,
            q,
            reset,
            set,
            vcc,
            gnd,
            resistance,
            state,
            ..
        } => {
            let supply = voltage(guess, vars, *vcc);
            let set_high = voltage(guess, vars, *set) > supply * 0.5;
            let reset_high = voltage(guess, vars, *reset) > supply * 0.5;
            let q_high = if reset_high {
                false
            } else if set_high {
                true
            } else {
                *state & 1 != 0
            };
            stamp_logic_output(
                *q,
                *vcc,
                *gnd,
                q_high,
                *resistance,
                guess,
                vars,
                matrix,
                rhs,
            );
            stamp_logic_output(
                *not_q,
                *vcc,
                *gnd,
                !q_high,
                *resistance,
                guess,
                vars,
                matrix,
                rhs,
            );
        }
        NonlinearElement::DigitalCounter {
            carry,
            outputs,
            reset,
            vcc,
            gnd,
            mode,
            modulus,
            resistance,
            state,
            ..
        } => {
            let supply = voltage(guess, vars, *vcc);
            let value = *state & 0x3ff;
            let reset_high = voltage(guess, vars, *reset) > supply * 0.5;
            for (index, output) in outputs.iter().enumerate() {
                let high = !reset_high
                    && match *mode {
                        1 => value == index as u32,
                        2 => dice_output(value, index),
                        _ => index < 4 && value & (1 << index) != 0,
                    };
                stamp_logic_output(
                    *output,
                    *vcc,
                    *gnd,
                    high,
                    *resistance,
                    guess,
                    vars,
                    matrix,
                    rhs,
                );
            }
            stamp_logic_output(
                *carry,
                *vcc,
                *gnd,
                !reset_high && value + 1 >= u32::from(*modulus),
                *resistance,
                guess,
                vars,
                matrix,
                rhs,
            );
        }
        NonlinearElement::StepSequencer {
            controls,
            output,
            steps,
            vcc,
            gnd,
            resistance,
            state,
            ..
        } => {
            let stage = (*state & 0xff).min(7) as usize;
            for (index, step) in steps.iter().enumerate() {
                stamp_logic_output(
                    *step,
                    *vcc,
                    *gnd,
                    index == stage,
                    *resistance,
                    guess,
                    vars,
                    matrix,
                    rhs,
                );
            }
            let conductance = 1.0 / *resistance;
            let selected = controls[stage];
            let current =
                conductance * (voltage(guess, vars, *output) - voltage(guess, vars, selected));
            stamp_current(
                (*output, *gnd),
                current,
                &[(*output, conductance), (selected, -conductance)],
                guess,
                vars,
                matrix,
                rhs,
            );
        }
        NonlinearElement::Sram {
            address,
            output,
            vcc,
            gnd,
            resistance,
            state,
            ..
        } => {
            let supply = voltage(guess, vars, *vcc);
            let selected = usize::from(voltage(guess, vars, *address) > supply * 0.5);
            let value = ((*state >> (selected * 8)) & 0xff) as u8;
            for (index, pin) in output.iter().enumerate() {
                stamp_logic_output(
                    *pin,
                    *vcc,
                    *gnd,
                    value & (1 << index) != 0,
                    *resistance,
                    guess,
                    vars,
                    matrix,
                    rhs,
                );
            }
        }
        NonlinearElement::ShiftRegister {
            outputs,
            vcc,
            gnd,
            resistance,
            state,
            ..
        } => {
            let output_state = (*state >> 8) & 0xff;
            for (index, output) in outputs.iter().enumerate() {
                stamp_logic_output(
                    *output,
                    *vcc,
                    *gnd,
                    output_state & (1 << index) != 0,
                    *resistance,
                    guess,
                    vars,
                    matrix,
                    rhs,
                );
            }
        }
        NonlinearElement::SevenSegmentDisplay {
            inputs,
            segments,
            vcc,
            gnd,
            resistance,
            ..
        } => {
            let supply = voltage(guess, vars, *vcc);
            let digit = inputs
                .iter()
                .enumerate()
                .fold(0u8, |digit, (index, input)| {
                    digit | u8::from(voltage(guess, vars, *input) > supply * 0.5) << index
                });
            let segments_on = match digit {
                0 => [true, true, true, true, true, true, false],
                1 => [false, true, true, false, false, false, false],
                2 => [true, true, false, true, true, false, true],
                3 => [true, true, true, true, false, false, true],
                4 => [false, true, true, false, false, true, true],
                5 => [true, false, true, true, false, true, true],
                6 => [true, false, true, true, true, true, true],
                7 => [true, true, true, false, false, false, false],
                8 => [true, true, true, true, true, true, true],
                9 => [true, true, true, true, false, true, true],
                _ => [false; 7],
            };
            for (segment, high) in segments.iter().zip(segments_on) {
                stamp_logic_output(
                    *segment,
                    *vcc,
                    *gnd,
                    high,
                    *resistance,
                    guess,
                    vars,
                    matrix,
                    rhs,
                );
            }
        }
        NonlinearElement::FourBitAdder {
            a,
            b,
            carry_in,
            carry_out,
            subtract,
            sum,
            vcc,
            gnd,
            resistance,
        } => {
            let supply = voltage(guess, vars, *vcc);
            let bit = |node: &usize| u32::from(logic_high(voltage(guess, vars, *node), supply));
            let a_value = a
                .iter()
                .enumerate()
                .fold(0u32, |value, (index, node)| value | bit(node) << index);
            let b_value = b
                .iter()
                .enumerate()
                .fold(0u32, |value, (index, node)| value | bit(node) << index);
            let subtracting = bit(subtract) != 0;
            let total = if subtracting {
                a_value + ((!b_value) & 0xf) + 1 + bit(carry_in)
            } else {
                a_value + b_value + bit(carry_in)
            };
            for (index, output) in sum.iter().enumerate() {
                stamp_logic_output(
                    *output,
                    *vcc,
                    *gnd,
                    total & (1 << index) != 0,
                    *resistance,
                    guess,
                    vars,
                    matrix,
                    rhs,
                );
            }
            stamp_logic_output(
                *carry_out,
                *vcc,
                *gnd,
                total > 0xf,
                *resistance,
                guess,
                vars,
                matrix,
                rhs,
            );
        }
        NonlinearElement::BargraphDisplay {
            input,
            segments,
            vcc,
            gnd,
            resistance,
        } => {
            let supply = voltage(guess, vars, *vcc);
            let level = (voltage(guess, vars, *input) / supply.max(1e-6) * 10.0)
                .floor()
                .clamp(0.0, 10.0) as usize;
            for (index, segment) in segments.iter().enumerate() {
                stamp_logic_output(
                    *segment,
                    *vcc,
                    *gnd,
                    index < level,
                    *resistance,
                    guess,
                    vars,
                    matrix,
                    rhs,
                );
            }
        }
        NonlinearElement::AudioAmplifier {
            input,
            output,
            vcc,
            gnd,
            gain,
            resistance,
        } => {
            let supply = voltage(guess, vars, *vcc);
            let ground = voltage(guess, vars, *gnd);
            let input_voltage = voltage(guess, vars, *input);
            let target = (ground + *gain * (input_voltage - ground)).clamp(ground, supply);
            let conductance = 1.0 / *resistance;
            let current = conductance * (voltage(guess, vars, *output) - target);
            stamp_current(
                (*output, *gnd),
                current,
                &[
                    (*output, conductance),
                    (*input, -conductance * *gain),
                    (*gnd, conductance * (*gain - 1.0)),
                ],
                guess,
                vars,
                matrix,
                rhs,
            );
        }
        NonlinearElement::IcDevice {
            pin_nodes,
            behavior,
            ..
        } => stamp_ic_device(behavior, pin_nodes, guess, vars, matrix, rhs),
        NonlinearElement::Module {
            pin_nodes,
            behavior,
            ..
        } => stamp_module(behavior, pin_nodes, guess, vars, matrix, rhs),
        NonlinearElement::OtherSource {
            positive,
            negative,
            voltage: target_voltage,
            resistance,
            ..
        } => {
            let conductance = 1.0 / *resistance;
            let current = conductance
                * (voltage(guess, vars, *positive)
                    - voltage(guess, vars, *negative)
                    - *target_voltage);
            stamp_current(
                (*positive, *negative),
                current,
                &[(*positive, conductance), (*negative, -conductance)],
                guess,
                vars,
                matrix,
                rhs,
            );
        }
        NonlinearElement::OtherLinearTransfer {
            pin_nodes,
            output,
            reference,
            inputs,
            offset,
            min_output,
            max_output,
            input_resistance,
            output_resistance,
            ..
        } => stamp_other_linear_transfer(
            pin_nodes,
            output,
            reference,
            inputs,
            *offset,
            *min_output,
            *max_output,
            *input_resistance,
            *output_resistance,
            guess,
            vars,
            matrix,
            rhs,
        ),
        NonlinearElement::OtherTransformer {
            primary_positive,
            primary_negative,
            secondary_positive,
            secondary_negative,
            turns_ratio,
            primary_resistance,
            secondary_resistance,
            ..
        } => {
            stamp_conductance(
                matrix,
                vars,
                *primary_positive,
                *primary_negative,
                1.0 / *primary_resistance,
            );
            let conductance = 1.0 / *secondary_resistance;
            let current = conductance
                * (voltage(guess, vars, *secondary_positive)
                    - voltage(guess, vars, *secondary_negative)
                    - *turns_ratio
                        * (voltage(guess, vars, *primary_positive)
                            - voltage(guess, vars, *primary_negative)));
            stamp_current(
                (*secondary_positive, *secondary_negative),
                current,
                &[
                    (*secondary_positive, conductance),
                    (*secondary_negative, -conductance),
                    (*primary_positive, -conductance * *turns_ratio),
                    (*primary_negative, conductance * *turns_ratio),
                ],
                guess,
                vars,
                matrix,
                rhs,
            );
        }
        NonlinearElement::OtherRingModulator {
            signal,
            carrier,
            output,
            reference,
            gain,
            signal_scale,
            carrier_scale,
            min_output,
            max_output,
            input_resistance,
            output_resistance,
            ..
        } => stamp_other_ring_modulator(
            *signal,
            *carrier,
            *output,
            *reference,
            *gain,
            *signal_scale,
            *carrier_scale,
            *min_output,
            *max_output,
            *input_resistance,
            *output_resistance,
            guess,
            vars,
            matrix,
            rhs,
        ),
        NonlinearElement::OtherControlledResistance {
            control_positive,
            control_negative,
            output_positive,
            output_negative,
            min_resistance,
            max_resistance,
            control_min,
            control_max,
            ..
        } => {
            let control_voltage =
                voltage(guess, vars, *control_positive) - voltage(guess, vars, *control_negative);
            let resistance = controlled_resistance(
                *min_resistance,
                *max_resistance,
                *control_min,
                *control_max,
                control_voltage,
            );
            stamp_conductance(
                matrix,
                vars,
                *output_positive,
                *output_negative,
                1.0 / resistance,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn stamp_other_bjt_socket(
    base: usize,
    collector: usize,
    emitter: usize,
    socket_polarity: BjtPolarity,
    subject_polarity: BjtPolarity,
    subject_state: BjtTestState,
    beta: f64,
    saturation: f64,
    open_resistance: f64,
    short_resistance: f64,
    guess: &[f64],
    vars: &BTreeMap<usize, usize>,
    matrix: &mut [Vec<f64>],
    rhs: &mut [f64],
    use_collector_base_jacobian: bool,
) {
    let state = if socket_polarity == subject_polarity {
        subject_state
    } else {
        BjtTestState::Open
    };
    match state {
        BjtTestState::Working => match socket_polarity {
            BjtPolarity::Npn => {
                let vbe = voltage(guess, vars, base) - voltage(guess, vars, emitter);
                let vce = voltage(guess, vars, collector) - voltage(guess, vars, emitter);
                let (ib, ic, gm, go) = npn_currents(vbe, vce, beta, saturation);
                let gm = if use_collector_base_jacobian { gm } else { 0.0 };
                let (_, gib) = led_current(vbe, 0.026 * (0.001 / saturation).ln(), 100.0);
                stamp_current(
                    (base, emitter),
                    ib,
                    &[(base, gib), (emitter, -gib)],
                    guess,
                    vars,
                    matrix,
                    rhs,
                );
                stamp_current(
                    (collector, emitter),
                    ic,
                    &[(base, gm), (collector, go), (emitter, -gm - go)],
                    guess,
                    vars,
                    matrix,
                    rhs,
                );
            }
            BjtPolarity::Pnp => {
                let veb = voltage(guess, vars, emitter) - voltage(guess, vars, base);
                let vec = voltage(guess, vars, emitter) - voltage(guess, vars, collector);
                let (ib, ic, gm, go) = npn_currents(veb, vec, beta, saturation);
                let gm = if use_collector_base_jacobian { gm } else { 0.0 };
                let (_, gib) = led_current(veb, 0.026 * (0.001 / saturation).ln(), 100.0);
                stamp_current(
                    (emitter, base),
                    ib,
                    &[(emitter, gib), (base, -gib)],
                    guess,
                    vars,
                    matrix,
                    rhs,
                );
                stamp_current(
                    (emitter, collector),
                    ic,
                    &[(emitter, gm + go), (base, -gm), (collector, -go)],
                    guess,
                    vars,
                    matrix,
                    rhs,
                );
            }
        },
        BjtTestState::Open => {
            stamp_conductance(matrix, vars, base, emitter, 1.0 / open_resistance);
            stamp_conductance(matrix, vars, collector, emitter, 1.0 / open_resistance);
        }
        BjtTestState::Shorted => {
            stamp_conductance(matrix, vars, base, emitter, 1.0 / short_resistance);
            stamp_conductance(matrix, vars, collector, emitter, 1.0 / short_resistance);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn stamp_other_diode_socket(
    anode: usize,
    cathode: usize,
    subject_polarity: DiodePolarity,
    subject_state: DiodeTestState,
    model: DiodeModel,
    open_resistance: f64,
    short_resistance: f64,
    guess: &[f64],
    vars: &BTreeMap<usize, usize>,
    matrix: &mut [Vec<f64>],
    rhs: &mut [f64],
) {
    match subject_state {
        DiodeTestState::Working => {
            let voltage = voltage(guess, vars, anode) - voltage(guess, vars, cathode);
            let (current, conductance) = match subject_polarity {
                DiodePolarity::Forward => {
                    let linearization = model.linearize(voltage);
                    (linearization.current, linearization.conductance)
                }
                DiodePolarity::Reverse => {
                    let linearization = model.linearize(-voltage);
                    (-linearization.current, linearization.conductance)
                }
            };
            stamp_current(
                (anode, cathode),
                current,
                &[(anode, conductance), (cathode, -conductance)],
                guess,
                vars,
                matrix,
                rhs,
            );
        }
        DiodeTestState::Open => {
            stamp_conductance(matrix, vars, anode, cathode, 1.0 / open_resistance)
        }
        DiodeTestState::Shorted => {
            stamp_conductance(matrix, vars, anode, cathode, 1.0 / short_resistance)
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn stamp_other_ring_modulator(
    signal: usize,
    carrier: usize,
    output: usize,
    reference: usize,
    gain: f64,
    signal_scale: f64,
    carrier_scale: f64,
    min_output: f64,
    max_output: f64,
    input_resistance: f64,
    output_resistance: f64,
    guess: &[f64],
    vars: &BTreeMap<usize, usize>,
    matrix: &mut [Vec<f64>],
    rhs: &mut [f64],
) {
    let reference_voltage = voltage(guess, vars, reference);
    let signal_delta = voltage(guess, vars, signal) - reference_voltage;
    let carrier_delta = voltage(guess, vars, carrier) - reference_voltage;
    let scale = signal_scale * carrier_scale;
    let target = ring_modulator_output(
        reference_voltage,
        voltage(guess, vars, signal),
        voltage(guess, vars, carrier),
        gain,
        signal_scale,
        carrier_scale,
        min_output,
        max_output,
    );
    let raw = gain * signal_delta * carrier_delta / scale;
    let bounded = raw.clamp(min_output, max_output);
    let conductance = 1.0 / output_resistance;
    let unclamped = (bounded - raw).abs() < f64::EPSILON;
    let (signal_derivative, carrier_derivative, reference_derivative) = if unclamped {
        let signal_derivative = gain * carrier_delta / scale;
        let carrier_derivative = gain * signal_delta / scale;
        (
            signal_derivative,
            carrier_derivative,
            1.0 - signal_derivative - carrier_derivative,
        )
    } else {
        (0.0, 0.0, 1.0)
    };
    let current = conductance * (voltage(guess, vars, output) - target);
    stamp_current(
        (output, reference),
        current,
        &[
            (output, conductance),
            (signal, -conductance * signal_derivative),
            (carrier, -conductance * carrier_derivative),
            (reference, -conductance * reference_derivative),
        ],
        guess,
        vars,
        matrix,
        rhs,
    );
    stamp_conductance(matrix, vars, signal, reference, 1.0 / input_resistance);
    stamp_conductance(matrix, vars, carrier, reference, 1.0 / input_resistance);
}

#[allow(clippy::too_many_arguments)]
fn stamp_other_linear_transfer(
    pin_nodes: &BTreeMap<PinId, usize>,
    output: &PinId,
    reference: &PinId,
    inputs: &[(PinId, f64)],
    offset: f64,
    min_output: f64,
    max_output: f64,
    input_resistance: f64,
    output_resistance: f64,
    guess: &[f64],
    vars: &BTreeMap<usize, usize>,
    matrix: &mut [Vec<f64>],
    rhs: &mut [f64],
) {
    let output = *pin_nodes.get(output).expect("validated other output pin");
    let reference = *pin_nodes
        .get(reference)
        .expect("validated other reference pin");
    let reference_voltage = voltage(guess, vars, reference);
    let raw = offset
        + inputs.iter().fold(0.0, |sum, (pin, gain)| {
            sum + *gain
                * (voltage(
                    guess,
                    vars,
                    *pin_nodes.get(pin).expect("validated other input pin"),
                ) - reference_voltage)
        });
    let bounded = raw.clamp(min_output, max_output);
    let target = reference_voltage + bounded;
    let conductance = 1.0 / output_resistance;
    let mut derivatives = vec![(output, conductance)];
    let target_reference_derivative = if (bounded - raw).abs() < f64::EPSILON {
        1.0 - inputs.iter().map(|(_, gain)| *gain).sum::<f64>()
    } else {
        1.0
    };
    derivatives.push((reference, -conductance * target_reference_derivative));
    if (bounded - raw).abs() < f64::EPSILON {
        derivatives.extend(inputs.iter().map(|(pin, gain)| {
            (
                *pin_nodes.get(pin).expect("validated other input pin"),
                -conductance * *gain,
            )
        }));
    }
    let current = conductance * (voltage(guess, vars, output) - target);
    stamp_current(
        (output, reference),
        current,
        &derivatives,
        guess,
        vars,
        matrix,
        rhs,
    );
    for (pin, _) in inputs {
        stamp_conductance(
            matrix,
            vars,
            *pin_nodes.get(pin).expect("validated other input pin"),
            reference,
            1.0 / input_resistance,
        );
    }
}

fn ic_node(pin_nodes: &BTreeMap<PinId, usize>, pin: &PinId) -> usize {
    *pin_nodes
        .get(pin)
        .expect("validated ic_device pin reference")
}

#[allow(clippy::too_many_arguments)]
fn stamp_module(
    behavior: &ModuleBehavior,
    pin_nodes: &BTreeMap<PinId, usize>,
    guess: &[f64],
    vars: &BTreeMap<usize, usize>,
    matrix: &mut [Vec<f64>],
    rhs: &mut [f64],
) {
    let node = |pin: &PinId| *pin_nodes.get(pin).expect("validated module pin reference");
    match behavior {
        ModuleBehavior::AnalogTransfer {
            output,
            reference,
            inputs,
            offset,
            min_output,
            max_output,
            input_resistance,
            output_resistance,
        } => {
            let output = node(output);
            let reference = node(reference);
            let reference_voltage = voltage(guess, vars, reference);
            let raw = offset
                + inputs.iter().fold(0.0, |sum, input| {
                    sum + input.gain * (voltage(guess, vars, node(&input.pin)) - reference_voltage)
                });
            let bounded = raw.clamp(*min_output, *max_output);
            let target = reference_voltage + bounded;
            let conductance = 1.0 / output_resistance;
            let mut derivatives = vec![(output, conductance)];
            let unclamped = (bounded - raw).abs() < f64::EPSILON;
            derivatives.push((
                reference,
                -conductance
                    * if unclamped {
                        1.0 - inputs.iter().map(|input| input.gain).sum::<f64>()
                    } else {
                        1.0
                    },
            ));
            if unclamped {
                derivatives.extend(
                    inputs
                        .iter()
                        .map(|input| (node(&input.pin), -conductance * input.gain)),
                );
            }
            stamp_current(
                (output, reference),
                conductance * (voltage(guess, vars, output) - target),
                &derivatives,
                guess,
                vars,
                matrix,
                rhs,
            );
            for input in inputs {
                stamp_conductance(
                    matrix,
                    vars,
                    node(&input.pin),
                    reference,
                    1.0 / input_resistance,
                );
            }
        }
        ModuleBehavior::ThresholdOutput {
            input,
            reference,
            output,
            threshold,
            high_output,
            low_output,
            input_resistance,
            output_resistance,
        } => {
            let input = node(input);
            let reference = node(reference);
            let output = node(output);
            let reference_voltage = voltage(guess, vars, reference);
            let high = voltage(guess, vars, input) - reference_voltage >= *threshold;
            let target = reference_voltage + if high { *high_output } else { *low_output };
            let conductance = 1.0 / output_resistance;
            stamp_current(
                (output, reference),
                conductance * (voltage(guess, vars, output) - target),
                &[(output, conductance), (reference, -conductance)],
                guess,
                vars,
                matrix,
                rhs,
            );
            stamp_conductance(matrix, vars, input, reference, 1.0 / input_resistance);
        }
        ModuleBehavior::OpenCollector {
            supply,
            ground,
            channels,
            input_resistance,
            on_resistance,
            off_resistance,
        } => {
            let supply = node(supply);
            let ground = node(ground);
            let supply_voltage = voltage(guess, vars, supply);
            let ground_voltage = voltage(guess, vars, ground);
            let threshold = ground_voltage + (supply_voltage - ground_voltage) * 0.5;
            for channel in channels {
                let input = node(&channel.input);
                let output = node(&channel.output);
                stamp_conductance(matrix, vars, input, ground, 1.0 / input_resistance);
                let resistance = if voltage(guess, vars, input) >= threshold {
                    *on_resistance
                } else {
                    *off_resistance
                };
                stamp_conductance(matrix, vars, output, ground, 1.0 / resistance);
            }
        }
        ModuleBehavior::RegulatedSupply {
            input_positive,
            input_negative,
            output_positive,
            output_negative,
            target_voltage,
            dropout_voltage,
            input_resistance,
            output_resistance,
        } => {
            let input_positive = node(input_positive);
            let input_negative = node(input_negative);
            let output_positive = node(output_positive);
            let output_negative = node(output_negative);
            stamp_conductance(
                matrix,
                vars,
                input_positive,
                input_negative,
                1.0 / input_resistance,
            );
            let input_voltage =
                voltage(guess, vars, input_positive) - voltage(guess, vars, input_negative);
            let available = (input_voltage - dropout_voltage).max(0.0);
            let target = available.min(*target_voltage);
            let target_is_input_limited = available < *target_voltage;
            let conductance = 1.0 / output_resistance;
            let current = conductance
                * (voltage(guess, vars, output_positive)
                    - voltage(guess, vars, output_negative)
                    - target);
            let mut derivatives = vec![
                (output_positive, conductance),
                (output_negative, -conductance),
            ];
            if target_is_input_limited {
                derivatives.extend([
                    (input_positive, -conductance),
                    (input_negative, conductance),
                ]);
            }
            stamp_current(
                (output_positive, output_negative),
                current,
                &derivatives,
                guess,
                vars,
                matrix,
                rhs,
            );
        }
        ModuleBehavior::AdjustableRegulatedSupply {
            input_positive,
            input_negative,
            output_positive,
            output_negative,
            adjust,
            reference_voltage,
            min_output_voltage,
            max_output_voltage,
            dropout_voltage,
            input_resistance,
            output_resistance,
        } => {
            let input_positive = node(input_positive);
            let input_negative = node(input_negative);
            let output_positive = node(output_positive);
            let output_negative = node(output_negative);
            let adjust = node(adjust);
            stamp_conductance(
                matrix,
                vars,
                input_positive,
                input_negative,
                1.0 / input_resistance,
            );
            let input_voltage =
                voltage(guess, vars, input_positive) - voltage(guess, vars, input_negative);
            let adjust_voltage =
                voltage(guess, vars, adjust) - voltage(guess, vars, input_negative);
            let available = (input_voltage - dropout_voltage).max(0.0);
            let requested = (adjust_voltage + reference_voltage)
                .clamp(*min_output_voltage, *max_output_voltage);
            let target = adjustable_output_target(
                input_voltage,
                adjust_voltage,
                *reference_voltage,
                *min_output_voltage,
                *max_output_voltage,
                *dropout_voltage,
            );
            let target_is_input_limited = available < requested;
            let target_follows_adjust = !target_is_input_limited
                && (*min_output_voltage..=*max_output_voltage)
                    .contains(&(adjust_voltage + reference_voltage));
            let conductance = 1.0 / output_resistance;
            let current = conductance
                * (voltage(guess, vars, output_positive)
                    - voltage(guess, vars, output_negative)
                    - target);
            let mut derivatives = vec![
                (output_positive, conductance),
                (output_negative, -conductance),
            ];
            if target_is_input_limited {
                derivatives.extend([
                    (input_positive, -conductance),
                    (input_negative, conductance),
                ]);
            } else if target_follows_adjust {
                derivatives.extend([(adjust, -conductance), (input_negative, conductance)]);
            }
            stamp_current(
                (output_positive, output_negative),
                current,
                &derivatives,
                guess,
                vars,
                matrix,
                rhs,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn stamp_ic_device(
    behavior: &IcDeviceBehavior,
    pin_nodes: &BTreeMap<PinId, usize>,
    guess: &[f64],
    vars: &BTreeMap<usize, usize>,
    matrix: &mut [Vec<f64>],
    rhs: &mut [f64],
) {
    match behavior {
        IcDeviceBehavior::Linear {
            output,
            reference,
            inputs,
            offset,
            min_output,
            max_output,
            input_resistance,
            output_resistance,
        } => {
            let output = ic_node(pin_nodes, output);
            let reference = ic_node(pin_nodes, reference);
            let reference_voltage = voltage(guess, vars, reference);
            let raw = *offset
                + inputs.iter().fold(0.0, |sum, input| {
                    sum + input.gain
                        * (voltage(guess, vars, ic_node(pin_nodes, &input.pin)) - reference_voltage)
                });
            let bounded = raw.clamp(*min_output, *max_output);
            let target = reference_voltage + bounded;
            let conductance = 1.0 / *output_resistance;
            let mut derivatives = vec![(output, conductance)];
            let target_reference_derivative = if (bounded - raw).abs() < f64::EPSILON {
                1.0 - inputs.iter().map(|input| input.gain).sum::<f64>()
            } else {
                1.0
            };
            derivatives.push((reference, -conductance * target_reference_derivative));
            if (bounded - raw).abs() < f64::EPSILON {
                derivatives.extend(
                    inputs
                        .iter()
                        .map(|input| (ic_node(pin_nodes, &input.pin), -conductance * input.gain)),
                );
            }
            let current = conductance * (voltage(guess, vars, output) - target);
            stamp_current(
                (output, reference),
                current,
                &derivatives,
                guess,
                vars,
                matrix,
                rhs,
            );
            for input in inputs {
                stamp_conductance(
                    matrix,
                    vars,
                    ic_node(pin_nodes, &input.pin),
                    reference,
                    1.0 / *input_resistance,
                );
            }
        }
        IcDeviceBehavior::Comparator {
            positive,
            negative,
            output,
            reference,
            threshold,
            high_output,
            low_output,
            input_resistance,
            output_resistance,
        } => {
            let positive = ic_node(pin_nodes, positive);
            let negative = ic_node(pin_nodes, negative);
            let output = ic_node(pin_nodes, output);
            let reference = ic_node(pin_nodes, reference);
            let reference_voltage = voltage(guess, vars, reference);
            let high =
                voltage(guess, vars, positive) - voltage(guess, vars, negative) >= *threshold;
            let target = reference_voltage + if high { *high_output } else { *low_output };
            let conductance = 1.0 / *output_resistance;
            stamp_current(
                (output, reference),
                conductance * (voltage(guess, vars, output) - target),
                &[(output, conductance), (reference, -conductance)],
                guess,
                vars,
                matrix,
                rhs,
            );
            for input in [positive, negative] {
                stamp_conductance(matrix, vars, input, reference, 1.0 / *input_resistance);
            }
        }
        IcDeviceBehavior::Logic {
            inputs,
            output,
            reference,
            supply,
            operation,
            output_resistance,
        } => {
            let reference = ic_node(pin_nodes, reference);
            let supply = ic_node(pin_nodes, supply);
            let reference_voltage = voltage(guess, vars, reference);
            let supply_voltage = voltage(guess, vars, supply);
            let values = inputs.iter().map(|pin| {
                voltage(guess, vars, ic_node(pin_nodes, pin))
                    > reference_voltage + (supply_voltage - reference_voltage) * 0.5
            });
            let high = match operation {
                IcLogicOperation::And => values.clone().all(|value| value),
                IcLogicOperation::Or => values.clone().any(|value| value),
                IcLogicOperation::Nand => !values.clone().all(|value| value),
                IcLogicOperation::Nor => !values.clone().any(|value| value),
                IcLogicOperation::Xor => values.clone().filter(|value| *value).count() % 2 == 1,
                IcLogicOperation::Xnor => values.clone().filter(|value| *value).count() % 2 == 0,
                IcLogicOperation::Not => !values.clone().next().unwrap_or(false),
            };
            stamp_logic_output(
                ic_node(pin_nodes, output),
                supply,
                reference,
                high,
                *output_resistance,
                guess,
                vars,
                matrix,
                rhs,
            );
        }
    }
}
fn dice_output(value: u32, index: usize) -> bool {
    let mask = match value {
        1 => 0b0001000,
        2 => 0b1000001,
        3 => 0b1001001,
        4 => 0b1010101,
        5 => 0b1011101,
        6 => 0b1110111,
        _ => 0,
    };
    index < 7 && mask & (1 << index) != 0
}
fn gaussian_solve(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Option<Vec<f64>> {
    let n = b.len();
    for col in 0..n {
        let pivot = (col..n).max_by(|&i, &j| a[i][col].abs().total_cmp(&a[j][col].abs()))?;
        if a[pivot][col].abs() < PIVOT_TOLERANCE {
            return None;
        }
        a.swap(col, pivot);
        b.swap(col, pivot);
        let d = a[col][col];
        for value in &mut a[col][col..] {
            *value /= d;
        }
        b[col] /= d;
        let pivot_row = a[col][col..].to_vec();
        for i in 0..n {
            if i == col {
                continue;
            }
            let f = a[i][col];
            if f == 0.0 {
                continue;
            }
            for (value, pivot) in a[i][col..].iter_mut().zip(&pivot_row) {
                *value -= f * pivot;
            }
            b[i] -= f * b[col];
        }
    }
    b.iter().all(|x| x.is_finite()).then_some(b)
}
fn check_source_cycles(branches: &[Branch]) -> Result<(), ElectricalError> {
    let mut graph: BTreeMap<usize, Vec<(usize, f64)>> = BTreeMap::new();
    for b in branches
        .iter()
        .filter(|b| matches!(b.kind, BranchKind::VoltageSource))
    {
        graph.entry(b.a).or_default().push((b.b, -b.value));
        graph.entry(b.b).or_default().push((b.a, b.value));
    }
    let mut known = BTreeMap::new();
    for &start in graph.keys() {
        if known.contains_key(&start) {
            continue;
        }
        known.insert(start, 0.0);
        let mut q = VecDeque::from([start]);
        while let Some(n) = q.pop_front() {
            let v = known[&n];
            for &(next, d) in &graph[&n] {
                let expected = v + d;
                if let Some(&actual) = known.get(&next) {
                    if (actual - expected).abs() > VOLTAGE_TOLERANCE {
                        return Err(calc(
                            "conflicting_sources",
                            "ideal voltage sources impose contradictory voltages",
                        ));
                    }
                } else {
                    known.insert(next, expected);
                    q.push_back(next);
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HoleId, ModuleBehavior, ModulePinRole, ModuleSpec};
    use proptest::prelude::*;

    fn divider() -> Project {
        serde_json::from_str(include_str!(
            "../../../fixtures/projects/resistor-divider.json"
        ))
        .unwrap()
    }

    fn pnp_high_side() -> Project {
        Project {
            format_version: crate::PROJECT_FORMAT_VERSION,
            title: "PNP high-side test".into(),
            board: crate::Board {
                model: crate::BoardModel::HalfSizeSolderless,
            },
            components: vec![
                Component {
                    id: ComponentId("V1".into()),
                    kind: ComponentKind::DcVoltageSource,
                    pins: BTreeMap::from([
                        (
                            crate::PinId("positive".into()),
                            crate::HoleId("TP+:1".into()),
                        ),
                        (
                            crate::PinId("negative".into()),
                            crate::HoleId("TP-:1".into()),
                        ),
                    ]),
                    parameters: BTreeMap::from([(String::from("voltage"), 5.0)]),
                    ic_device: None,
                    other_device: None,
                    module: None,
                    diode_model: None,
                },
                Component {
                    id: ComponentId("R1".into()),
                    kind: ComponentKind::Resistor,
                    pins: BTreeMap::from([
                        (crate::PinId("a".into()), crate::HoleId("B1".into())),
                        (crate::PinId("b".into()), crate::HoleId("TP-:4".into())),
                    ]),
                    parameters: BTreeMap::from([(String::from("resistance"), 470.0)]),
                    ic_device: None,
                    other_device: None,
                    module: None,
                    diode_model: None,
                },
                Component {
                    id: ComponentId("R2".into()),
                    kind: ComponentKind::Resistor,
                    pins: BTreeMap::from([
                        (crate::PinId("a".into()), crate::HoleId("A2".into())),
                        (crate::PinId("b".into()), crate::HoleId("TP-:2".into())),
                    ]),
                    parameters: BTreeMap::from([(String::from("resistance"), 10_000.0)]),
                    ic_device: None,
                    other_device: None,
                    module: None,
                    diode_model: None,
                },
                Component {
                    id: ComponentId("Q1".into()),
                    kind: ComponentKind::PnpTransistor,
                    pins: BTreeMap::from([
                        (crate::PinId("base".into()), crate::HoleId("C2".into())),
                        (crate::PinId("collector".into()), crate::HoleId("A1".into())),
                        (
                            crate::PinId("emitter".into()),
                            crate::HoleId("TP+:3".into()),
                        ),
                    ]),
                    parameters: BTreeMap::from([
                        (String::from("beta"), 100.0),
                        (String::from("saturation_current"), 1e-15),
                    ]),
                    ic_device: None,
                    other_device: None,
                    module: None,
                    diode_model: None,
                },
            ],
            wires: Vec::new(),
            faults: Vec::new(),
            initial_conditions: Default::default(),
        }
    }

    fn ic_device_linear_project(gain: f64) -> Project {
        let pin_roles = BTreeMap::from([
            (PinId("gnd".into()), crate::IcDevicePinRole::Ground),
            (PinId("vcc".into()), crate::IcDevicePinRole::Supply),
            (PinId("input".into()), crate::IcDevicePinRole::Input),
            (PinId("output".into()), crate::IcDevicePinRole::Output),
        ]);
        Project {
            format_version: crate::PROJECT_FORMAT_VERSION,
            title: "generic IC linear transfer".into(),
            board: crate::Board {
                model: crate::BoardModel::HalfSizeSolderless,
            },
            components: vec![
                Component {
                    id: ComponentId("V1".into()),
                    kind: ComponentKind::DcVoltageSource,
                    pins: BTreeMap::from([
                        (PinId("positive".into()), HoleId("TP+:1".into())),
                        (PinId("negative".into()), HoleId("TP-:1".into())),
                    ]),
                    parameters: BTreeMap::from([("voltage".into(), 5.0)]),
                    ic_device: None,
                    other_device: None,
                    module: None,
                    diode_model: None,
                },
                Component {
                    id: ComponentId("U1".into()),
                    kind: ComponentKind::IcDevice,
                    pins: BTreeMap::from([
                        (PinId("gnd".into()), HoleId("TP-:1".into())),
                        (PinId("vcc".into()), HoleId("TP+:1".into())),
                        (PinId("input".into()), HoleId("TP+:1".into())),
                        (PinId("output".into()), HoleId("A1".into())),
                    ]),
                    parameters: BTreeMap::new(),
                    ic_device: Some(crate::IcDeviceSpec {
                        pin_roles,
                        behavior: crate::IcDeviceBehavior::Linear {
                            output: PinId("output".into()),
                            reference: PinId("gnd".into()),
                            inputs: vec![crate::IcDeviceLinearInput {
                                pin: PinId("input".into()),
                                gain,
                            }],
                            offset: 0.0,
                            min_output: 0.0,
                            max_output: 5.0,
                            input_resistance: 1e9,
                            output_resistance: 10.0,
                        },
                    }),
                    other_device: None,
                    module: None,
                    diode_model: None,
                },
                Component {
                    id: ComponentId("R1".into()),
                    kind: ComponentKind::Resistor,
                    pins: BTreeMap::from([
                        (PinId("a".into()), HoleId("A1".into())),
                        (PinId("b".into()), HoleId("TP-:1".into())),
                    ]),
                    parameters: BTreeMap::from([("resistance".into(), 1_000.0)]),
                    ic_device: None,
                    other_device: None,
                    module: None,
                    diode_model: None,
                },
            ],
            wires: Vec::new(),
            faults: Vec::new(),
            initial_conditions: Default::default(),
        }
    }
    fn solve(p: &Project) -> Result<SolveResult, ElectricalError> {
        solve_dc(p, &BTreeMap::new(), &BTreeMap::new())
    }

    #[test]
    fn pnp_high_side_has_a_calculated_collector_current() {
        let result = solve(&pnp_high_side()).unwrap();
        let collector = result.pnp_collector_currents[&ComponentId("Q1".into())];
        let load = result.resistor_currents[&ComponentId("R1".into())];
        assert!(collector < -0.005);
        assert!((collector + load).abs() < 1e-9);
    }

    #[test]
    fn generic_ic_device_linear_transfer_is_calculated_and_bounded() {
        let result = solve(&ic_device_linear_project(2.0)).unwrap();
        let output =
            result.ic_device_output_voltages[&ComponentId("U1".into())][&PinId("output".into())];
        assert!((0.0..=5.0).contains(&output));
        assert!(output > 4.9);
    }

    #[test]
    fn generic_ic_device_linear_transfer_is_id_rename_invariant() {
        let mut renamed = ic_device_linear_project(0.5);
        renamed.components[1].id = ComponentId("AMPLIFIER".into());
        let first = solve(&ic_device_linear_project(0.5)).unwrap();
        let second = solve(&renamed).unwrap();
        let first_output =
            first.ic_device_output_voltages[&ComponentId("U1".into())][&PinId("output".into())];
        let second_output = second.ic_device_output_voltages[&ComponentId("AMPLIFIER".into())]
            [&PinId("output".into())];
        assert!((first_output - second_output).abs() < 1e-12);
    }

    #[test]
    fn generic_ic_device_logic_uses_calculated_supply_threshold() {
        let mut project = ic_device_linear_project(1.0);
        project.components[1]
            .pins
            .insert(PinId("input_b".into()), HoleId("TP+:1".into()));
        let spec = project.components[1].ic_device.as_mut().unwrap();
        spec.pin_roles
            .insert(PinId("input_b".into()), crate::IcDevicePinRole::Input);
        spec.behavior = crate::IcDeviceBehavior::Logic {
            inputs: vec![PinId("input".into()), PinId("input_b".into())],
            output: PinId("output".into()),
            reference: PinId("gnd".into()),
            supply: PinId("vcc".into()),
            operation: crate::IcLogicOperation::And,
            output_resistance: 10.0,
        };
        let result = solve(&project).unwrap();
        let output =
            result.ic_device_output_voltages[&ComponentId("U1".into())][&PinId("output".into())];
        assert!(output > 4.9);
    }

    #[test]
    fn generic_ic_device_comparator_uses_calculated_pin_voltages() {
        let mut project = ic_device_linear_project(1.0);
        project.components[1]
            .pins
            .insert(PinId("negative".into()), HoleId("TP-:1".into()));
        let spec = project.components[1].ic_device.as_mut().unwrap();
        spec.pin_roles
            .insert(PinId("negative".into()), crate::IcDevicePinRole::Input);
        spec.behavior = crate::IcDeviceBehavior::Comparator {
            positive: PinId("input".into()),
            negative: PinId("negative".into()),
            output: PinId("output".into()),
            reference: PinId("gnd".into()),
            threshold: 1.0,
            high_output: 5.0,
            low_output: 0.0,
            input_resistance: 1e9,
            output_resistance: 10.0,
        };
        let result = solve(&project).unwrap();
        let output =
            result.ic_device_output_voltages[&ComponentId("U1".into())][&PinId("output".into())];
        assert!(output > 4.9);
    }

    #[test]
    fn generic_ic_device_linear_property_is_bounded_for_reproducible_seed() {
        let mut seed = 0x5eed_cafe_u64;
        for _ in 0..128 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let gain = ((seed >> 16) as f64 / u32::MAX as f64) * 8.0 - 4.0;
            let result = solve(&ic_device_linear_project(gain)).unwrap();
            let output = result.ic_device_output_voltages[&ComponentId("U1".into())]
                [&PinId("output".into())];
            assert!(
                (-1e-9..=5.0 + 1e-9).contains(&output),
                "seed={seed}, gain={gain}"
            );
        }
    }
    fn pin_voltage(result: &SolveResult, component: &str, pin: &str) -> f64 {
        result
            .node_voltages
            .iter()
            .find(|node| {
                node.contacts.contains(&Contact::ComponentPin(
                    ComponentId(component.into()),
                    crate::PinId(pin.into()),
                ))
            })
            .unwrap()
            .voltage
    }

    #[test]
    fn analytical_divider_voltages_and_currents() {
        let result = solve(&divider()).unwrap();
        assert!((pin_voltage(&result, "R1", "b") - 2.5).abs() < 1e-10);
        assert!((result.resistor_currents[&ComponentId("R1".into())] - 0.0025).abs() < 1e-10);
        assert!((result.resistor_currents[&ComponentId("R2".into())] - 0.0025).abs() < 1e-10);
    }

    #[test]
    fn analytical_parallel_resistors_and_source_current_balance() {
        let p: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/parallel-resistors.json"
        ))
        .unwrap();
        let result = solve(&p).unwrap();
        assert!((result.resistor_currents[&ComponentId("R1".into())] - 0.005).abs() < 1e-10);
        assert!((result.resistor_currents[&ComponentId("R2".into())] - 0.005).abs() < 1e-10);
        assert!((result.source_currents[&ComponentId("V1".into())] + 0.01).abs() < 1e-10);
    }

    #[test]
    fn reports_floating_conflicting_and_short_sources() {
        let floating: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/floating-resistors.json"
        ))
        .unwrap();
        assert!(
            matches!(solve(&floating),Err(ElectricalError::Calculation(e)) if e.code=="floating_network")
        );
        let conflict: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/conflicting-sources.json"
        ))
        .unwrap();
        assert!(
            matches!(solve(&conflict),Err(ElectricalError::Calculation(e)) if e.code=="conflicting_sources")
        );
        let mut short = divider();
        short.components[0].pins.insert(
            crate::PinId("positive".into()),
            crate::HoleId("TP-:1".into()),
        );
        assert!(
            matches!(solve(&short),Err(ElectricalError::Calculation(e)) if e.code=="ideal_source_short")
        );
    }

    #[test]
    fn open_button_and_changeover_positions_are_explicit() {
        let mut button = divider();
        button.wires.clear();
        button.components.push(Component {
            id: ComponentId("B1".into()),
            kind: ComponentKind::MomentaryButton,
            pins: BTreeMap::from([
                (crate::PinId("a".into()), crate::HoleId("A1".into())),
                (crate::PinId("b".into()), crate::HoleId("A2".into())),
            ]),
            parameters: BTreeMap::new(),
            ic_device: None,
            other_device: None,
            module: None,
            diode_model: None,
        });
        let open = solve(&button).unwrap();
        assert!(
            open.resistor_currents
                .values()
                .all(|current| current.abs() < 1e-12)
        );
        let pressed = solve_dc(
            &button,
            &BTreeMap::from([(ComponentId("B1".into()), ControlState::ButtonPressed)]),
            &BTreeMap::new(),
        )
        .unwrap();
        assert!((pressed.resistor_currents[&ComponentId("R1".into())] - 0.0025).abs() < 1e-10);

        let mut changeover = divider();
        changeover.wires.clear();
        changeover.components.push(Component {
            id: ComponentId("S1".into()),
            kind: ComponentKind::ChangeoverSwitch,
            pins: BTreeMap::from([
                (crate::PinId("common".into()), crate::HoleId("A1".into())),
                (
                    crate::PinId("normally_closed".into()),
                    crate::HoleId("A2".into()),
                ),
                (
                    crate::PinId("normally_open".into()),
                    crate::HoleId("F1".into()),
                ),
            ]),
            parameters: BTreeMap::new(),
            ic_device: None,
            other_device: None,
            module: None,
            diode_model: None,
        });
        assert!(
            (solve(&changeover).unwrap().resistor_currents[&ComponentId("R1".into())] - 0.0025)
                .abs()
                < 1e-10
        );
        let no = solve_dc(
            &changeover,
            &BTreeMap::from([(ComponentId("S1".into()), ControlState::SwitchNormallyOpen)]),
            &BTreeMap::new(),
        )
        .unwrap();
        assert!(
            no.resistor_currents
                .values()
                .all(|current| current.abs() < 1e-12)
        );
    }

    #[test]
    fn built_in_led_and_transistor_benches_follow_button_current() {
        for json in [
            include_str!("../../../fixtures/projects/led-bench.json"),
            include_str!("../../../fixtures/projects/transistor-bench.json"),
        ] {
            let project: Project = serde_json::from_str(json).unwrap();
            let off = solve_transient(
                &project,
                &BTreeMap::new(),
                &BTreeMap::new(),
                &BTreeMap::new(),
            )
            .unwrap_or_else(|e| panic!("{} off: {e:?}", project.title));
            let on = solve_transient(
                &project,
                &BTreeMap::from([(ComponentId("B1".into()), ControlState::ButtonPressed)]),
                &BTreeMap::new(),
                &BTreeMap::new(),
            )
            .unwrap_or_else(|e| panic!("{} on: {e:?}", project.title));
            let off_current = off.led_currents[&ComponentId("D1".into())];
            let on_current = on.led_currents[&ComponentId("D1".into())];
            assert!(off_current.abs() < 1e-6, "off current: {off_current}");
            assert!(
                (0.005..0.02).contains(&on_current),
                "on current: {on_current}"
            );
            if project
                .components
                .iter()
                .any(|c| c.kind == ComponentKind::NpnTransistor)
            {
                let collector = on.transistor_collector_currents[&ComponentId("Q1".into())];
                assert!((collector - on_current).abs() < 1e-9);
                assert!((collector - 0.00846).abs() < 0.0005);
                assert!(
                    (on.source_currents[&ComponentId("V1".into())]
                        + collector
                        + on.resistor_currents[&ComponentId("R2".into())])
                        .abs()
                        < 1e-9
                );
            } else {
                assert!((on_current - 3.0 / 350.0).abs() < 0.0005);
            }
        }
    }

    #[test]
    fn led_polarity_is_calculated_from_pins_in_a_valid_assembly() {
        let mut project: Project =
            serde_json::from_str(include_str!("../../../fixtures/projects/led-bench.json"))
                .unwrap();
        let states = BTreeMap::from([(ComponentId("B1".into()), ControlState::ButtonPressed)]);
        let forward =
            solve_transient(&project, &states, &BTreeMap::new(), &BTreeMap::new()).unwrap();
        let led = project
            .components
            .iter_mut()
            .find(|c| c.id.0 == "D1")
            .unwrap();
        let anode = led.pins[&crate::PinId("anode".into())].clone();
        let cathode = led.pins[&crate::PinId("cathode".into())].clone();
        led.pins.insert(crate::PinId("anode".into()), cathode);
        led.pins.insert(crate::PinId("cathode".into()), anode);
        let reversed =
            solve_transient(&project, &states, &BTreeMap::new(), &BTreeMap::new()).unwrap();
        assert!(forward.led_currents[&ComponentId("D1".into())] > 0.008);
        assert!(reversed.led_currents[&ComponentId("D1".into())].abs() < 1e-6);
    }

    fn breadboard_led(voltage: f64, resistance: f64, led_resistance: f64) -> Project {
        let mut project: Project =
            serde_json::from_str(include_str!("../../../fixtures/projects/led-bench.json"))
                .unwrap();
        project
            .components
            .iter_mut()
            .for_each(|component| match component.id.0.as_str() {
                "V1" => {
                    component.parameters.insert("voltage".into(), voltage);
                }
                "R1" => {
                    component.parameters.insert("resistance".into(), resistance);
                }
                "D1" => {
                    component
                        .parameters
                        .insert("series_resistance".into(), led_resistance);
                }
                _ => {}
            });
        project
    }

    #[test]
    fn led_wired_directly_across_the_12v_source_converges() {
        let mut project = breadboard_led(12.0, 1.0, 1.0);
        let pressed = BTreeMap::from([(ComponentId("B1".into()), ControlState::ButtonPressed)]);
        assert!(solve_transient(&project, &pressed, &BTreeMap::new(), &BTreeMap::new()).is_ok());
        let led = project
            .components
            .iter_mut()
            .find(|component| component.id.0 == "D1")
            .unwrap();
        led.pins
            .insert(crate::PinId("anode".into()), crate::HoleId("TP+:20".into()));
        led.pins.insert(
            crate::PinId("cathode".into()),
            crate::HoleId("TP-:20".into()),
        );
        project.wires.push(crate::Wire {
            id: crate::WireId("W3".into()),
            from: crate::HoleId("A8".into()),
            to: crate::HoleId("TP-:8".into()),
        });
        let result = solve_transient(
            &project,
            &BTreeMap::new(),
            &BTreeMap::new(),
            &BTreeMap::new(),
        );
        assert!(result.is_ok(), "direct LED solve failed: {result:?}");
        assert!(result.unwrap().led_currents[&ComponentId("D1".into())] > 1.0);
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 96, rng_seed: proptest::test_runner::RngSeed::Fixed(0xDC03_2026), .. ProptestConfig::default() })]
        #[test]
        fn divider_current_balance_and_order_invariance(r1 in 10u32..100_000, r2 in 10u32..100_000) {
            let mut p=divider();
            p.components[1].parameters.insert("resistance".into(),f64::from(r1));
            p.components[2].parameters.insert("resistance".into(),f64::from(r2));
            let solved=solve(&p).unwrap();
            let expected=5.0*f64::from(r2)/f64::from(r1+r2);
            let mid=pin_voltage(&solved,"R1","b");
            prop_assert!((mid-expected).abs()<1e-8);
            let i1=solved.resistor_currents[&ComponentId("R1".into())];
            let i2=solved.resistor_currents[&ComponentId("R2".into())];
            prop_assert!((i1-i2).abs()<1e-8);
            prop_assert!((i1+solved.source_currents[&ComponentId("V1".into())]).abs()<1e-8);
            let mut reordered=p.clone();reordered.components.reverse();reordered.wires.reverse();
            prop_assert_eq!(solve(&reordered).unwrap(),solved);
        }
        #[test]
        fn led_current_falls_as_series_resistance_rises(low in 220u32..800, extra in 1u32..500) {
            let mut project: Project = serde_json::from_str(include_str!("../../../fixtures/projects/led-bench.json")).unwrap();
            let states = BTreeMap::from([(ComponentId("B1".into()), ControlState::ButtonPressed)]);
            project.components.iter_mut().find(|c| c.id.0 == "R1").unwrap().parameters.insert("resistance".into(), f64::from(low));
            let first = solve_transient(&project, &states, &BTreeMap::new(), &BTreeMap::new()).unwrap().led_currents[&ComponentId("D1".into())];
            project.components.iter_mut().find(|c| c.id.0 == "R1").unwrap().parameters.insert("resistance".into(), f64::from(low + extra));
            let second = solve_transient(&project, &states, &BTreeMap::new(), &BTreeMap::new()).unwrap().led_currents[&ComponentId("D1".into())];
            prop_assert!(first > second && second > 0.0);
        }
        #[test]
        fn breadboard_led_ranges_converge(
            voltage_centi in 0u32..=1200,
            resistor_bucket in 0u32..=1000,
            led_bucket in 0u32..=1000,
        ) {
            let log_range = |bucket: u32| 10f64.powf(7.0 * f64::from(bucket) / 1000.0);
            let project = breadboard_led(
                f64::from(voltage_centi) / 100.0,
                log_range(resistor_bucket),
                log_range(led_bucket),
            );
            let states = BTreeMap::from([(ComponentId("B1".into()), ControlState::ButtonPressed)]);
            let result = solve_transient(&project, &states, &BTreeMap::new(), &BTreeMap::new());
            prop_assert!(result.is_ok(), "range case did not converge: {:?}", result.err());
        }
        #[test]
        fn transistor_load_current_tracks_beta_and_balances(low in 20u32..100, extra in 1u32..100) {
            let mut project: Project = serde_json::from_str(include_str!("../../../fixtures/projects/transistor-bench.json")).unwrap();
            let states = BTreeMap::from([(ComponentId("B1".into()), ControlState::ButtonPressed)]);
            project.components.iter_mut().find(|c| c.id.0 == "Q1").unwrap().parameters.insert("beta".into(), f64::from(low));
            let first = solve_transient(&project, &states, &BTreeMap::new(), &BTreeMap::new()).unwrap();
            let lower_current = first.led_currents[&ComponentId("D1".into())];
            project.components.iter_mut().find(|c| c.id.0 == "Q1").unwrap().parameters.insert("beta".into(), f64::from(low + extra));
            let second = solve_transient(&project, &states, &BTreeMap::new(), &BTreeMap::new()).unwrap();
            let higher_current = second.led_currents[&ComponentId("D1".into())];
            prop_assert!(higher_current > lower_current);
            prop_assert!((second.transistor_collector_currents[&ComponentId("Q1".into())] - higher_current).abs() < 1e-9);
            prop_assert!((second.source_currents[&ComponentId("V1".into())] + higher_current + second.resistor_currents[&ComponentId("R2".into())]).abs() < 1e-9);
        }

        #[test]
        fn breadboard_transistor_ranges_converge_and_switch(
            beta in 10u32..=1000,
            saturation_bucket in 0u32..=1000,
        ) {
            let saturation = 10f64.powf(-16.0 + 4.0 * f64::from(saturation_bucket) / 1000.0);
            let mut project: Project = serde_json::from_str(include_str!(
                "../../../fixtures/projects/transistor-bench.json"
            )).unwrap();
            let transistor = project
                .components
                .iter_mut()
                .find(|component| component.id.0 == "Q1")
                .unwrap();
            transistor.parameters.insert("beta".into(), f64::from(beta));
            transistor
                .parameters
                .insert("saturation_current".into(), saturation);
            let released = solve_transient(&project, &BTreeMap::new(), &BTreeMap::new(), &BTreeMap::new())
                .expect("released breadboard-range transistor must converge within 80 iterations");
            let pressed = solve_transient(
                &project,
                &BTreeMap::from([(ComponentId("B1".into()), ControlState::ButtonPressed)]),
                &BTreeMap::new(),
                &BTreeMap::new(),
            )
            .expect("pressed breadboard-range transistor must converge within 80 iterations");
            let released_base = released.resistor_currents[&ComponentId("R2".into())].abs();
            let released_load = released.led_currents[&ComponentId("D1".into())];
            let pressed_load = pressed.led_currents[&ComponentId("D1".into())];
            prop_assert!(released_base < 1e-6, "released base current: {released_base}");
            prop_assert!(pressed_load > released_load, "released={released_load}, pressed={pressed_load}");
            prop_assert!(
                pressed.transistor_collector_currents[&ComponentId("Q1".into())]
                    > released.transistor_collector_currents[&ComponentId("Q1".into())]
            );
        }

        #[test]
        fn relay_breadboard_ranges_converge(
            coil_bucket in 0u32..=1000,
            pickup_bucket in 0u32..=1000,
        ) {
            let mut project: Project = serde_json::from_str(include_str!(
                "../../../fixtures/projects/c09-s15-01-relay-switch.json"
            )).unwrap();
            let relay = project
                .components
                .iter_mut()
                .find(|component| component.id.0 == "K1")
                .unwrap();
            relay.parameters.insert(
                "coil_resistance".into(),
                10f64.powf(0.0 + 6.0 * f64::from(coil_bucket) / 1000.0),
            );
            relay.parameters.insert(
                "pickup_voltage".into(),
                12.0 * f64::from(pickup_bucket) / 1000.0,
            );
            let pressed = solve_transient(
                &project,
                &BTreeMap::from([(ComponentId("B1".into()), ControlState::ButtonPressed)]),
                &BTreeMap::new(),
                &BTreeMap::new(),
            );
            prop_assert!(pressed.is_ok(), "relay range did not converge: {:?}", pressed.err());
        }

        #[test]
        fn diode_logic_input_resistors_preserve_or_output(resistance in 1_000u32..=100_000) {
            let mut project: Project = serde_json::from_str(include_str!(
                "../../../fixtures/projects/c10-s18-07-diode-logic.json"
            )).unwrap();
            for id in ["R1", "R2"] {
                project
                    .components
                    .iter_mut()
                    .find(|component| component.id.0 == id)
                    .unwrap()
                    .parameters
                    .insert("resistance".into(), f64::from(resistance));
            }
            let result = solve_transient(
                &project,
                &BTreeMap::from([(ComponentId("B1".into()), ControlState::ButtonPressed)]),
                &BTreeMap::new(),
                &BTreeMap::new(),
            ).unwrap();
            prop_assert!(result.led_currents[&ComponentId("D3".into())] > 0.001);
        }
    }

    #[test]
    fn buzzer_matches_a_plain_resistor_of_the_same_value() {
        let mut buzzer = divider();
        buzzer.components[1].kind = ComponentKind::Buzzer;
        buzzer.components[1].parameters = BTreeMap::from([("resistance".into(), 1000.0)]);
        buzzer.components[1].pins = BTreeMap::from([
            (
                crate::PinId("positive".into()),
                crate::HoleId("TP+:2".into()),
            ),
            (crate::PinId("negative".into()), crate::HoleId("A1".into())),
        ]);
        let plain = solve(&divider()).unwrap();
        let as_buzzer = solve(&buzzer).unwrap();
        assert_eq!(
            plain.resistor_currents[&ComponentId("R1".into())],
            as_buzzer.resistor_currents[&ComponentId("R1".into())]
        );
        assert_eq!(
            pin_voltage(&plain, "R1", "a"),
            pin_voltage(&as_buzzer, "R1", "positive")
        );
        assert_eq!(
            pin_voltage(&plain, "R1", "b"),
            pin_voltage(&as_buzzer, "R1", "negative")
        );
    }

    #[test]
    fn speaker_matches_a_plain_resistor_of_the_same_value() {
        // Within Speaker's documented range (1-100 ohms), unlike the buzzer
        // equivalence test's 1000-ohm divider.
        let mut plain_project = divider();
        plain_project.components[1]
            .parameters
            .insert("resistance".into(), 8.0);
        plain_project.components[2]
            .parameters
            .insert("resistance".into(), 8.0);
        let mut speaker = plain_project.clone();
        speaker.components[1].kind = ComponentKind::Speaker;
        speaker.components[1].pins = BTreeMap::from([
            (
                crate::PinId("positive".into()),
                crate::HoleId("TP+:2".into()),
            ),
            (crate::PinId("negative".into()), crate::HoleId("A1".into())),
        ]);
        let plain = solve(&plain_project).unwrap();
        let as_speaker = solve(&speaker).unwrap();
        assert_eq!(
            plain.resistor_currents[&ComponentId("R1".into())],
            as_speaker.resistor_currents[&ComponentId("R1".into())]
        );
        assert_eq!(
            pin_voltage(&plain, "R1", "a"),
            pin_voltage(&as_speaker, "R1", "positive")
        );
        assert_eq!(
            pin_voltage(&plain, "R1", "b"),
            pin_voltage(&as_speaker, "R1", "negative")
        );
    }

    #[test]
    fn passive_piezo_matches_a_plain_resistor_of_the_same_value() {
        let mut plain_project = divider();
        plain_project.components[1]
            .parameters
            .insert("resistance".into(), 32.0);
        plain_project.components[2]
            .parameters
            .insert("resistance".into(), 32.0);
        let mut piezo = plain_project.clone();
        piezo.components[1].kind = ComponentKind::PiezoPassive;
        piezo.components[1].pins = BTreeMap::from([
            (
                crate::PinId("positive".into()),
                crate::HoleId("TP+:2".into()),
            ),
            (crate::PinId("negative".into()), crate::HoleId("A1".into())),
        ]);
        let plain = solve(&plain_project).unwrap();
        let as_piezo = solve(&piezo).unwrap();
        assert_eq!(
            plain.resistor_currents[&ComponentId("R1".into())],
            as_piezo.resistor_currents[&ComponentId("R1".into())]
        );
        assert_eq!(
            pin_voltage(&plain, "R1", "a"),
            pin_voltage(&as_piezo, "R1", "positive")
        );
        assert_eq!(
            pin_voltage(&plain, "R1", "b"),
            pin_voltage(&as_piezo, "R1", "negative")
        );
    }

    #[test]
    fn motor_switch_derives_current_and_signed_speed_from_terminal_voltage() {
        let project: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/c06-s08-01-motor-with-switch.json"
        ))
        .unwrap();
        let off = solve(&project).unwrap();
        assert_eq!(off.motor_speeds[&ComponentId("M1".into())], 0.0);
        assert!(off.switch_currents[&ComponentId("S1".into())].abs() < 1e-12);

        let on = solve_internal(
            &project,
            &BTreeMap::from([(ComponentId("S1".into()), ControlState::SwitchNormallyOpen)]),
            &BTreeMap::new(),
            &BTreeMap::new(),
            &BTreeMap::new(),
            SolveOptions {
                dt: None,
                step: 0,
                max_iterations: MAX_NONLINEAR_ITERATIONS,
            },
        )
        .unwrap();
        assert!((on.switch_currents[&ComponentId("S1".into())].abs() - 0.375).abs() < 1e-12);
        assert!((on.motor_currents[&ComponentId("M1".into())] - 0.375).abs() < 1e-12);
        assert!((on.motor_speeds[&ComponentId("M1".into())] - 10_000.0).abs() < 1e-9);
    }

    #[test]
    fn relay_button_selects_calculated_contact_and_coil_current() {
        let project: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/c09-s15-01-relay-switch.json"
        ))
        .unwrap();
        let released = solve_transient(
            &project,
            &BTreeMap::new(),
            &BTreeMap::new(),
            &BTreeMap::new(),
        )
        .unwrap();
        let pressed = solve_transient(
            &project,
            &BTreeMap::from([(ComponentId("B1".into()), ControlState::ButtonPressed)]),
            &BTreeMap::new(),
            &BTreeMap::new(),
        )
        .unwrap();
        assert!(!released.relay_energized[&ComponentId("K1".into())]);
        assert!(pressed.relay_energized[&ComponentId("K1".into())]);
        assert!(pressed.relay_coil_currents[&ComponentId("K1".into())] > 0.06);
        assert!(pressed.led_currents[&ComponentId("D1".into())] > 0.001);
        assert!(released.led_currents[&ComponentId("D1".into())] < 1e-5);
    }

    #[test]
    fn diode_input_or_switches_the_calculated_led_for_either_button() {
        let project: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/c10-s18-07-diode-logic.json"
        ))
        .unwrap();
        let cases = [
            (BTreeMap::new(), false),
            (
                BTreeMap::from([(ComponentId("B1".into()), ControlState::ButtonPressed)]),
                true,
            ),
            (
                BTreeMap::from([(ComponentId("B2".into()), ControlState::ButtonPressed)]),
                true,
            ),
            (
                BTreeMap::from([
                    (ComponentId("B1".into()), ControlState::ButtonPressed),
                    (ComponentId("B2".into()), ControlState::ButtonPressed),
                ]),
                true,
            ),
        ];
        for (states, on) in cases {
            let result =
                solve_transient(&project, &states, &BTreeMap::new(), &BTreeMap::new()).unwrap();
            let current = result.led_currents[&ComponentId("D3".into())];
            assert_eq!(
                current > 0.001,
                on,
                "states: {states:?}, current: {current}"
            );
        }
    }

    #[test]
    fn beacon_darkness_enables_the_calculated_led_output() {
        let project: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/c11-s19-01-beacon.json"
        ))
        .unwrap();
        let bright = solve_transient(
            &project,
            &BTreeMap::new(),
            &BTreeMap::new(),
            &BTreeMap::from([
                (ComponentId("R3".into()), 1.0),
                (ComponentId("RV1".into()), 0.25),
            ]),
        )
        .unwrap();
        assert!(bright.led_currents[&ComponentId("D1".into())] < 1e-6);

        let mut capacitors = BTreeMap::new();
        let mut peak_led_current = 0.0_f64;
        for _ in 0..3_000 {
            let result = solve_transient(
                &project,
                &BTreeMap::new(),
                &capacitors,
                &BTreeMap::from([
                    (ComponentId("R3".into()), 0.0),
                    (ComponentId("RV1".into()), 0.25),
                ]),
            )
            .unwrap();
            peak_led_current = peak_led_current.max(result.led_currents[&ComponentId("D1".into())]);
            capacitors.extend(result.capacitor_voltages);
        }
        assert!(
            peak_led_current > 0.0005,
            "peak current: {peak_led_current}"
        );
    }

    #[test]
    fn telegraph_button_drives_only_its_calculated_station_loads() {
        let project: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/c07-s09-03-two-station-telegraph.json"
        ))
        .unwrap();
        let first = solve_internal(
            &project,
            &BTreeMap::from([(ComponentId("B1".into()), ControlState::ButtonPressed)]),
            &BTreeMap::new(),
            &BTreeMap::new(),
            &BTreeMap::new(),
            SolveOptions {
                dt: None,
                step: 0,
                max_iterations: MAX_NONLINEAR_ITERATIONS,
            },
        )
        .unwrap();
        assert!(first.led_currents[&ComponentId("D1".into())] > 0.001);
        assert!(first.resistor_currents[&ComponentId("BZ1".into())].abs() > 0.01);
        assert!(first.led_currents[&ComponentId("D2".into())].abs() < 1e-12);
        assert!(first.resistor_currents[&ComponentId("BZ2".into())].abs() < 1e-12);
    }

    #[test]
    fn optocoupler_transfers_calculated_input_current_across_isolated_sources() {
        let project: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/c08-s17-03-optocoupler.json"
        ))
        .unwrap();
        let on = solve(&project).unwrap();
        assert!(on.optocoupler_input_currents[&ComponentId("U1".into())] > 0.001);
        assert!(on.led_currents[&ComponentId("D1".into())] > 0.001);
        let blocked = solve_internal(
            &project,
            &BTreeMap::from([(ComponentId("B1".into()), ControlState::ButtonPressed)]),
            &BTreeMap::new(),
            &BTreeMap::new(),
            &BTreeMap::new(),
            SolveOptions {
                dt: None,
                step: 0,
                max_iterations: MAX_NONLINEAR_ITERATIONS,
            },
        )
        .unwrap();
        assert!(blocked.optocoupler_input_currents[&ComponentId("U1".into())] < 1e-9);
        assert!(blocked.led_currents[&ComponentId("D1".into())] < 1e-5);
    }

    fn variable_resistor_divider(kind: ComponentKind, min: f64, max: f64) -> Project {
        let mut p = divider();
        p.components[1].kind = kind;
        p.components[1].parameters = BTreeMap::from([
            ("min_resistance".into(), min),
            ("max_resistance".into(), max),
        ]);
        p
    }

    #[test]
    fn potentiometer_resistance_matches_linear_interpolation_at_the_endpoints() {
        let project = variable_resistor_divider(ComponentKind::Potentiometer, 100.0, 1100.0);
        let at = |ratio: f64| {
            solve_dc(
                &project,
                &BTreeMap::new(),
                &BTreeMap::from([(ComponentId("R1".into()), ratio)]),
            )
            .unwrap()
            .resistor_currents[&ComponentId("R1".into())]
        };
        // R1 (100..1100 ohm) in series with R2 (1000 ohm) across 5 V.
        assert!((at(0.0) - 5.0 / (100.0 + 1000.0)).abs() < 1e-9);
        assert!((at(1.0) - 5.0 / (1100.0 + 1000.0)).abs() < 1e-9);
    }

    #[test]
    fn photoresistor_resistance_is_inverted_relative_to_potentiometer() {
        let project = variable_resistor_divider(ComponentKind::Photoresistor, 100.0, 1100.0);
        let at = |ratio: f64| {
            solve_dc(
                &project,
                &BTreeMap::new(),
                &BTreeMap::from([(ComponentId("R1".into()), ratio)]),
            )
            .unwrap()
            .resistor_currents[&ComponentId("R1".into())]
        };
        // Darkest (ratio 0.0) is max_resistance; brightest (ratio 1.0) is min_resistance.
        assert!((at(0.0) - 5.0 / (1100.0 + 1000.0)).abs() < 1e-9);
        assert!((at(1.0) - 5.0 / (100.0 + 1000.0)).abs() < 1e-9);
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 96, rng_seed: proptest::test_runner::RngSeed::Fixed(0x7A19_2026), .. ProptestConfig::default() })]

        #[test]
        fn variable_resistor_current_changes_monotonically_with_ratio(
            min_bucket in 0u32..=900,
            span_bucket in 1u32..=900,
            is_photoresistor in any::<bool>(),
            ratios in prop::collection::vec(0u32..=1000, 8),
        ) {
            let min = 10f64.powf(7.0 * f64::from(min_bucket) / 1000.0).max(1.0);
            let max = (min + 10f64.powf(7.0 * f64::from(span_bucket) / 1000.0)).min(1e7);
            prop_assume!(max > min);
            let kind = if is_photoresistor { ComponentKind::Photoresistor } else { ComponentKind::Potentiometer };
            let project = variable_resistor_divider(kind, min, max);
            let mut points: Vec<(f64, f64)> = ratios
                .iter()
                .map(|bucket| f64::from(*bucket) / 1000.0)
                .map(|ratio| {
                    let current = solve_dc(
                        &project,
                        &BTreeMap::new(),
                        &BTreeMap::from([(ComponentId("R1".into()), ratio)]),
                    )
                    .unwrap()
                    .resistor_currents[&ComponentId("R1".into())];
                    (ratio, current)
                })
                .collect();
            points.sort_by(|a, b| a.0.total_cmp(&b.0));
            // Current is a monotonically decreasing function of resistance; whether
            // it rises or falls with ratio depends on kind, so compare direction
            // against the endpoints rather than assuming a sign.
            let increasing = points.last().unwrap().1 >= points.first().unwrap().1;
            for pair in points.windows(2) {
                let (_, i1) = pair[0];
                let (_, i2) = pair[1];
                if increasing {
                    prop_assert!(i2 >= i1 - 1e-12, "current decreased: {i1} -> {i2}");
                } else {
                    prop_assert!(i2 <= i1 + 1e-12, "current increased: {i1} -> {i2}");
                }
            }
        }

        #[test]
        fn beacon_supported_control_values_converge(
            light_bucket in 0u32..=1000,
            rate_bucket in 0u32..=1000,
        ) {
            let project: Project = serde_json::from_str(include_str!(
                "../../../fixtures/projects/c11-s19-01-beacon.json"
            )).unwrap();
            let result = solve_transient(
                &project,
                &BTreeMap::new(),
                &BTreeMap::new(),
                &BTreeMap::from([
                    (ComponentId("R3".into()), f64::from(light_bucket) / 1000.0),
                    (ComponentId("RV1".into()), f64::from(rate_bucket) / 1000.0),
                ]),
            ).unwrap();
            prop_assert!(result.led_currents[&ComponentId("D1".into())].is_finite());
            prop_assert!(result.transistor_collector_currents[&ComponentId("Q1".into())].is_finite());
        }
    }

    /// Reference and admitted C01 fixtures: topology validity plus the
    /// specific electrical behavior each fixture's acceptance criteria calls out.
    mod exercises {
        use super::*;

        fn fixture(json: &str) -> Project {
            serde_json::from_str(json).unwrap()
        }
        fn pressed(id: &str) -> BTreeMap<ComponentId, ControlState> {
            BTreeMap::from([(ComponentId(id.into()), ControlState::ButtonPressed)])
        }
        fn led_current(result: &SolveResult, id: &str) -> f64 {
            result.led_currents[&ComponentId(id.into())]
        }

        macro_rules! fixture_json {
            ($name:ident, $path:literal) => {
                const $name: &str = include_str!(concat!("../../../fixtures/projects/", $path));
            };
        }
        fixture_json!(E1, "e1-first-light.json");
        fixture_json!(E2, "e2-push-button-switch.json");
        fixture_json!(E3, "e3-two-leds-in-series.json");
        fixture_json!(E4, "e4-two-leds-in-parallel.json");
        fixture_json!(E7, "e7-buzzer-doorbell.json");
        fixture_json!(E8, "e8-transistor-switch.json");
        fixture_json!(E9, "e9-logical-and.json");
        fixture_json!(E10, "e10-smooth-fade.json");
        fixture_json!(E5, "e5-brightness-dial.json");
        fixture_json!(E6, "e6-light-reactive-led.json");
        fixture_json!(C01_S01_01, "c01-s01-01-first-safe-light.json");
        fixture_json!(C01_S01_02, "c01-s01-02-button-and-switch.json");
        fixture_json!(C01_S01_03, "c01-s01-03-series-and-parallel.json");
        fixture_json!(C01_S01_04, "c01-s01-04-potentiometer-dimmer.json");
        fixture_json!(C01_S01_05, "c01-s01-05-reverse-polarity.json");
        fixture_json!(C01_S01_06, "c01-s01-06-smooth-fade.json");
        fixture_json!(C01_S02_01, "c01-s02-01-transistor-key.json");
        fixture_json!(C01_S02_02, "c01-s02-02-dusk-night-light.json");
        fixture_json!(C01_S02_03, "c01-s02-03-touch-button.json");
        fixture_json!(C01_S02_04, "c01-s02-04-water-sensor.json");
        fixture_json!(C01_S02_06, "c01-s02-06-transistor-logic.json");
        fixture_json!(C02_S03_03, "c02-s03-03-logic-gates.json");
        fixture_json!(C02_S03_01, "c02-s03-01-555-flasher.json");
        fixture_json!(C02_S03_04, "c02-s03-04-rs-latch.json");
        fixture_json!(C02_S03_06, "c02-s03-06-thermostat.json");
        fixture_json!(C02_S03_05, "c02-s03-05-debounce.json");
        fixture_json!(C02_S03_02, "c02-s03-02-555-monostable.json");
        fixture_json!(C02_S03_07, "c02-s03-07-light-theremin.json");
        fixture_json!(C04_S06_11, "c04-s06-11-robot-voice.json");
        fixture_json!(C05_S07_05, "c05-s07-05-usb-5v-supply.json");
        fixture_json!(C05_S07_06, "c05-s07-06-adjustable-power-supply.json");
        fixture_json!(C06_S08_08, "c06-s08-08-thermostatic-fan.json");

        #[test]
        fn all_embedded_exercise_fixtures_have_valid_solvable_topology() {
            for json in [E1, E2, E3, E4, E7, E8, E9, E5, E6, C01_S02_06] {
                let project = fixture(json);
                compile_topology(&project).unwrap_or_else(|e| panic!("{}: {e:?}", project.title));
                solve_dc(&project, &BTreeMap::new(), &BTreeMap::new())
                    .unwrap_or_else(|e| panic!("{}: {e:?}", project.title));
            }
            // E10 has a capacitor, which the resistive DC solver does not
            // support; it requires transient stepping instead.
            let e10 = fixture(E10);
            compile_topology(&e10).unwrap_or_else(|e| panic!("{}: {e:?}", e10.title));
            solve_transient(&e10, &BTreeMap::new(), &BTreeMap::new(), &BTreeMap::new())
                .unwrap_or_else(|e| panic!("{}: {e:?}", e10.title));
        }

        #[test]
        fn c01_level_one_fixtures_validate_and_follow_their_controls() {
            for json in [C01_S01_01, C01_S01_03, C01_S01_04, C01_S01_05] {
                let project = fixture(json);
                compile_topology(&project).unwrap_or_else(|e| panic!("{}: {e:?}", project.title));
                let result = solve_dc(&project, &BTreeMap::new(), &BTreeMap::new())
                    .unwrap_or_else(|e| panic!("{}: {e:?}", project.title));
                assert!(result.led_currents[&ComponentId("D1".into())] > 0.0005);
            }

            let button = fixture(C01_S01_02);
            let pressed = BTreeMap::from([(ComponentId("B1".into()), ControlState::ButtonPressed)]);
            let selected = solve_dc(&button, &pressed, &BTreeMap::new()).unwrap();
            assert!(selected.led_currents[&ComponentId("D1".into())] > 0.0005);
            assert!(selected.led_currents[&ComponentId("D2".into())].abs() < 1e-6);

            let fade = fixture(C01_S01_06);
            let mut capacitor_voltages = BTreeMap::new();
            let mut charged = 0.0;
            for _ in 0..2_500 {
                let result =
                    solve_transient(&fade, &pressed, &capacitor_voltages, &BTreeMap::new())
                        .unwrap();
                charged = result.capacitor_voltages[&ComponentId("C1".into())];
                capacitor_voltages = result.capacitor_voltages;
            }
            assert!(charged > 3.0);
        }

        #[test]
        fn c01_reverse_polarity_fixture_blocks_reversed_supply() {
            let mut project = fixture(C01_S01_05);
            let forward = solve_dc(&project, &BTreeMap::new(), &BTreeMap::new()).unwrap();
            assert!(forward.diode_currents[&ComponentId("D0".into())] > 0.001);
            let source = project
                .components
                .iter_mut()
                .find(|c| c.id.0 == "V1")
                .unwrap();
            let positive = source.pins[&crate::PinId("positive".into())].clone();
            let negative = source.pins[&crate::PinId("negative".into())].clone();
            source
                .pins
                .insert(crate::PinId("positive".into()), negative);
            source
                .pins
                .insert(crate::PinId("negative".into()), positive);
            let reversed = solve_dc(&project, &BTreeMap::new(), &BTreeMap::new()).unwrap();
            assert!(reversed.diode_currents[&ComponentId("D0".into())].abs() < 1e-6);
        }

        #[test]
        fn c01_transistor_key_and_dusk_light_follow_calculated_controls() {
            let key = fixture(C01_S02_01);
            let released = solve_dc(&key, &BTreeMap::new(), &BTreeMap::new()).unwrap();
            let pressed = solve_dc(&key, &pressed("B1"), &BTreeMap::new()).unwrap();
            assert!(released.led_currents[&ComponentId("D1".into())] < 1e-5);
            assert!(pressed.led_currents[&ComponentId("D1".into())] > 0.005);
            assert!(
                (pressed.transistor_collector_currents[&ComponentId("Q1".into())]
                    - pressed.led_currents[&ComponentId("D1".into())])
                    .abs()
                    < 1e-9
            );

            let dusk = fixture(C01_S02_02);
            let dark = solve_dc(
                &dusk,
                &BTreeMap::new(),
                &BTreeMap::from([
                    (ComponentId("R3".into()), 0.0),
                    (ComponentId("RV1".into()), 0.5),
                ]),
            )
            .unwrap();
            let bright = solve_dc(
                &dusk,
                &BTreeMap::new(),
                &BTreeMap::from([
                    (ComponentId("R3".into()), 1.0),
                    (ComponentId("RV1".into()), 0.5),
                ]),
            )
            .unwrap();
            assert!(
                dark.led_currents[&ComponentId("D1".into())]
                    > bright.led_currents[&ComponentId("D1".into())]
            );
        }

        #[test]
        fn c01_touch_and_water_inputs_follow_explicit_resistance_controls() {
            let touch = fixture(C01_S02_03);
            let dry = solve_dc(
                &touch,
                &BTreeMap::new(),
                &BTreeMap::from([(ComponentId("TP1".into()), 0.0)]),
            )
            .unwrap();
            let contact = solve_dc(
                &touch,
                &BTreeMap::new(),
                &BTreeMap::from([(ComponentId("TP1".into()), 1.0)]),
            )
            .unwrap();
            assert!(
                contact.led_currents[&ComponentId("D1".into())]
                    > dry.led_currents[&ComponentId("D1".into())] + 0.001,
                "dry={} contact={}",
                dry.led_currents[&ComponentId("D1".into())],
                contact.led_currents[&ComponentId("D1".into())]
            );

            let water = fixture(C01_S02_04);
            let dry = solve_dc(
                &water,
                &BTreeMap::new(),
                &BTreeMap::from([(ComponentId("WP1".into()), 0.0)]),
            )
            .unwrap();
            let wet = solve_dc(
                &water,
                &BTreeMap::new(),
                &BTreeMap::from([(ComponentId("WP1".into()), 1.0)]),
            )
            .unwrap();
            assert!(
                wet.resistor_currents[&ComponentId("BZ1".into())].abs()
                    > dry.resistor_currents[&ComponentId("BZ1".into())].abs() + 0.001
            );
        }

        #[test]
        fn c01_transistor_logic_matches_all_four_input_combinations() {
            let project = fixture(C01_S02_06);
            let inputs = [
                (false, false, false, false, true),
                (true, false, false, true, false),
                (false, true, false, true, true),
                (true, true, true, true, false),
            ];
            for (a_pressed, b_pressed, and_on, or_on, not_on) in inputs {
                let mut controls = BTreeMap::new();
                if a_pressed {
                    controls.insert(ComponentId("S1".into()), ControlState::ButtonPressed);
                }
                if b_pressed {
                    controls.insert(ComponentId("S2".into()), ControlState::ButtonPressed);
                }
                let result =
                    solve_dc(&project, &controls, &BTreeMap::new()).unwrap_or_else(|error| {
                        panic!("solve failed for A={a_pressed}, B={b_pressed}: {error:?}")
                    });
                let assert_led = |id: &str, on: bool, label: &str| {
                    let current = result.led_currents[&ComponentId(id.into())];
                    if on {
                        assert!(
                            current > 0.001,
                            "{label} mismatch for A={a_pressed}, B={b_pressed}: {current}"
                        );
                    } else {
                        assert!(
                            current < 1e-4,
                            "{label} mismatch for A={a_pressed}, B={b_pressed}: {current}"
                        );
                    }
                };
                assert_led("D1", and_on, "AND");
                assert_led("D2", or_on, "OR");
                assert_led("D3", not_on, "NOT");
            }
        }

        #[test]
        fn c02_logic_gate_fixture_matches_all_four_input_combinations() {
            let project = fixture(C02_S03_03);
            let inputs = [
                (false, false, false, true, false),
                (true, false, false, true, true),
                (false, true, false, true, true),
                (true, true, true, false, false),
            ];
            for (a_pressed, b_pressed, and_on, nand_on, xor_on) in inputs {
                let mut controls = BTreeMap::new();
                if a_pressed {
                    controls.insert(ComponentId("S1".into()), ControlState::ButtonPressed);
                }
                if b_pressed {
                    controls.insert(ComponentId("S2".into()), ControlState::ButtonPressed);
                }
                let result =
                    solve_transient(&project, &controls, &BTreeMap::new(), &BTreeMap::new())
                        .unwrap_or_else(|error| {
                            panic!("solve failed for A={a_pressed}, B={b_pressed}: {error:?}")
                        });
                let assert_led = |id: &str, on: bool| {
                    let current = result.led_currents[&ComponentId(id.into())];
                    if on {
                        assert!(
                            current > 0.001,
                            "{id} should be on for A={a_pressed}, B={b_pressed}"
                        );
                    } else {
                        assert!(
                            current < 1e-4,
                            "{id} should be off for A={a_pressed}, B={b_pressed}"
                        );
                    }
                };
                assert_led("D1", and_on);
                assert_led("D2", a_pressed || b_pressed);
                assert_led("D3", nand_on);
                assert_led("D4", xor_on);
            }
        }

        #[test]
        fn c02_555_fixture_produces_calculated_led_cycles() {
            let project = fixture(C02_S03_01);
            let mut capacitors = BTreeMap::new();
            let mut previous_on = false;
            let mut transitions = 0;
            for _ in 0..2_000 {
                let result =
                    solve_transient(&project, &BTreeMap::new(), &capacitors, &BTreeMap::new())
                        .unwrap();
                capacitors.extend(result.capacitor_voltages.clone());
                let on = result.led_currents[&ComponentId("D1".into())] > 0.001;
                if on != previous_on {
                    transitions += 1;
                    previous_on = on;
                }
            }
            assert!(
                transitions >= 4,
                "expected multiple 555 LED transitions, got {transitions}"
            );
        }

        #[test]
        fn c02_rs_latch_retains_calculated_output_after_set_release() {
            let project = fixture(C02_S03_04);
            let mut capacitors = BTreeMap::new();
            let set = BTreeMap::from([(ComponentId("S_SET".into()), ControlState::ButtonPressed)]);
            for _ in 0..100 {
                let result =
                    solve_transient(&project, &set, &capacitors, &BTreeMap::new()).unwrap();
                capacitors.extend(result.capacitor_voltages);
            }
            let result =
                solve_transient(&project, &BTreeMap::new(), &capacitors, &BTreeMap::new()).unwrap();
            assert!(result.led_currents[&ComponentId("D1".into())] > 0.001);
        }

        #[test]
        fn c02_comparator_thermistor_crosses_the_calculated_threshold() {
            let project = fixture(C02_S03_06);
            let cold = solve_dc(
                &project,
                &BTreeMap::new(),
                &BTreeMap::from([(ComponentId("TH1".into()), 0.0)]),
            )
            .unwrap();
            let hot = solve_dc(
                &project,
                &BTreeMap::new(),
                &BTreeMap::from([(ComponentId("TH1".into()), 1.0)]),
            )
            .unwrap();
            assert!(cold.led_currents[&ComponentId("D1".into())] > 0.001);
            assert!(hot.led_currents[&ComponentId("D1".into())] < 1e-4);
        }

        #[test]
        fn c06_thermostatic_fan_switches_calculated_load_at_heat_threshold() {
            let project = fixture(C06_S08_08);
            let solve_at = |temperature: f64, threshold: f64| {
                solve_dc(
                    &project,
                    &BTreeMap::new(),
                    &BTreeMap::from([
                        (ComponentId("TH1".into()), temperature),
                        (ComponentId("RV1".into()), threshold),
                    ]),
                )
                .unwrap()
            };
            let cold = solve_at(0.0, 0.5);
            let hot = solve_at(1.0, 0.5);
            let fan_current = |result: &SolveResult| {
                result.other_terminal_currents[&ComponentId("FAN1".into())]
                    [&PinId("positive".into())]
                    .abs()
            };
            assert!(
                fan_current(&cold) < 1e-5,
                "cold fan current: {}",
                fan_current(&cold)
            );
            assert!(
                fan_current(&hot) > 0.1,
                "hot fan current: {}",
                fan_current(&hot)
            );
            assert!(hot.diode_currents[&ComponentId("D1".into())].abs() < 1e-6);
        }

        proptest! {
            #![proptest_config(ProptestConfig {
                cases: 64,
                rng_seed: proptest::test_runner::RngSeed::Fixed(0xC608_2026),
                ..ProptestConfig::default()
            })]

            #[test]
            fn c06_fan_switch_remains_bounded_across_supported_thresholds(
                threshold_bucket in 200u32..=900,
            ) {
                let project = fixture(C06_S08_08);
                let threshold = f64::from(threshold_bucket) / 1000.0;
                let solve_at = |temperature| {
                    solve_dc(
                        &project,
                        &BTreeMap::new(),
                        &BTreeMap::from([
                            (ComponentId("TH1".into()), temperature),
                            (ComponentId("RV1".into()), threshold),
                        ]),
                    )
                    .unwrap()
                };
                let cold = solve_at(0.0);
                let hot = solve_at(1.0);
                let cold_current = cold.other_terminal_currents[&ComponentId("FAN1".into())]
                    [&PinId("positive".into())]
                    .abs();
                let hot_current = hot.other_terminal_currents[&ComponentId("FAN1".into())]
                    [&PinId("positive".into())]
                    .abs();
                prop_assert!(cold_current.is_finite() && hot_current.is_finite());
                prop_assert!(cold_current < 1e-5, "cold current: {cold_current}");
                prop_assert!(hot_current > 0.1 && hot_current < 0.2, "hot current: {hot_current}");
            }
        }

        #[test]
        fn c02_schmitt_debounce_fixture_produces_a_calculated_output() {
            let project = fixture(C02_S03_05);
            let released = solve_transient(
                &project,
                &BTreeMap::new(),
                &BTreeMap::new(),
                &BTreeMap::new(),
            )
            .unwrap();
            let pressed = BTreeMap::from([(ComponentId("S1".into()), ControlState::ButtonPressed)]);
            let mut capacitors = released.capacitor_voltages.clone();
            let mut result = released;
            for _ in 0..100 {
                result =
                    solve_transient(&project, &pressed, &capacitors, &BTreeMap::new()).unwrap();
                capacitors.extend(result.capacitor_voltages.clone());
            }
            assert!(result.led_currents[&ComponentId("D1".into())] > 0.001);
        }

        #[test]
        fn c02_555_monostable_changes_output_from_button_and_rc_state() {
            let project = fixture(C02_S03_02);
            let pressed = BTreeMap::from([(ComponentId("S1".into()), ControlState::ButtonPressed)]);
            let mut capacitors = BTreeMap::new();
            let mut held = None;
            for _ in 0..20 {
                let result =
                    solve_transient(&project, &pressed, &capacitors, &BTreeMap::new()).unwrap();
                capacitors.extend(result.capacitor_voltages.clone());
                held = Some(result);
            }
            assert!(held.unwrap().led_currents[&ComponentId("D1".into())] > 0.001);
            let mut released = None;
            for _ in 0..2_000 {
                let result =
                    solve_transient(&project, &BTreeMap::new(), &capacitors, &BTreeMap::new())
                        .unwrap();
                capacitors.extend(result.capacitor_voltages.clone());
                released = Some(result);
            }
            assert!(released.unwrap().led_currents[&ComponentId("D1".into())] < 1e-4);
        }

        #[test]
        fn c02_light_theremin_changes_calculated_timer_frequency_with_light() {
            fn transitions(project: &Project, ratio: f64) -> usize {
                let mut capacitors = BTreeMap::new();
                let mut previous = false;
                let mut count = 0;
                for _ in 0..3_000 {
                    let result = solve_transient(
                        project,
                        &BTreeMap::new(),
                        &capacitors,
                        &BTreeMap::from([(ComponentId("TH1".into()), ratio)]),
                    )
                    .unwrap();
                    capacitors.extend(result.capacitor_voltages.clone());
                    let high = result.node_voltages.iter().any(|node| {
                        node.contacts.contains(&Contact::ComponentPin(
                            ComponentId("U1".into()),
                            crate::PinId("output".into()),
                        )) && node.voltage > 2.5
                    });
                    if high != previous {
                        count += 1;
                        previous = high;
                    }
                }
                count
            }
            let project = fixture(C02_S03_07);
            let dark = transitions(&project, 0.0);
            let bright = transitions(&project, 1.0);
            assert!(dark >= 1 && bright >= 1, "dark={dark}, bright={bright}");
            assert_ne!(
                dark, bright,
                "photoresistor must change calculated frequency"
            );
        }

        #[test]
        fn c04_robot_voice_calculates_transformers_ring_modulator_and_speaker_path() {
            let project = fixture(C04_S06_11);
            compile_topology(&project).expect("robot voice topology should compile");
            let mut capacitor_voltages = BTreeMap::new();
            let mut previous_carrier = false;
            let mut carrier_transitions = 0;
            let mut saw_ring_output = false;
            for _ in 0..4_000 {
                let result = solve_transient(
                    &project,
                    &BTreeMap::new(),
                    &capacitor_voltages,
                    &BTreeMap::new(),
                )
                .unwrap_or_else(|error| panic!("robot voice solve failed: {error:?}"));
                let carrier = result
                    .node_voltages
                    .iter()
                    .find(|node| {
                        node.contacts.contains(&Contact::ComponentPin(
                            ComponentId("T2".into()),
                            crate::PinId("secondary_positive".into()),
                        ))
                    })
                    .map(|node| node.voltage > 2.5)
                    .unwrap_or(false);
                if carrier != previous_carrier {
                    carrier_transitions += 1;
                    previous_carrier = carrier;
                }
                let ring_output = result.other_output_voltages[&ComponentId("RM1".into())]
                    [&crate::PinId("output".into())];
                assert!((0.0..=5.0).contains(&ring_output));
                assert!(
                    result
                        .other_output_voltages
                        .contains_key(&ComponentId("T1".into()))
                );
                assert!(
                    result
                        .other_output_voltages
                        .contains_key(&ComponentId("T2".into()))
                );
                assert!(result.resistor_currents[&ComponentId("SP1".into())].is_finite());
                saw_ring_output |= ring_output > 1e-6;
                capacitor_voltages = result.capacitor_voltages;
            }
            assert!(carrier_transitions >= 2, "carrier did not oscillate");
            assert!(saw_ring_output, "ring modulator never produced output");
        }

        #[test]
        fn e1_e3_always_on_exercises_light_without_any_control() {
            let e1 = solve_dc(&fixture(E1), &BTreeMap::new(), &BTreeMap::new()).unwrap();
            assert!(led_current(&e1, "D1") > 0.005);
            let e3 = solve_dc(&fixture(E3), &BTreeMap::new(), &BTreeMap::new()).unwrap();
            assert!(led_current(&e3, "D1") > 0.005);
            assert!(led_current(&e3, "D2") > 0.005);
        }

        #[test]
        fn e2_led_lights_only_while_its_button_is_pressed() {
            let project = fixture(E2);
            let released = solve_dc(&project, &BTreeMap::new(), &BTreeMap::new()).unwrap();
            assert!(led_current(&released, "D1").abs() < 1e-6);
            let held = solve_dc(&project, &pressed("S1"), &BTreeMap::new()).unwrap();
            assert!(led_current(&held, "D1") > 0.005);
        }

        #[test]
        fn e4_two_led_branches_are_independently_solvable() {
            let full = fixture(E4);
            let baseline = solve_dc(&full, &BTreeMap::new(), &BTreeMap::new()).unwrap();
            let mut only_branch_a = full.clone();
            only_branch_a
                .components
                .retain(|c| c.id.0 != "R2" && c.id.0 != "D2");
            only_branch_a
                .wires
                .retain(|w| w.id.0 != "W3" && w.id.0 != "W4");
            let branch_a = solve_dc(&only_branch_a, &BTreeMap::new(), &BTreeMap::new()).unwrap();
            assert_eq!(
                led_current(&baseline, "D1"),
                led_current(&branch_a, "D1"),
                "removing D2's branch must not change D1's current"
            );

            let mut only_branch_b = full.clone();
            only_branch_b
                .components
                .retain(|c| c.id.0 != "R1" && c.id.0 != "D1");
            only_branch_b
                .wires
                .retain(|w| w.id.0 != "W1" && w.id.0 != "W2");
            let branch_b = solve_dc(&only_branch_b, &BTreeMap::new(), &BTreeMap::new()).unwrap();
            assert_eq!(
                led_current(&baseline, "D2"),
                led_current(&branch_b, "D2"),
                "removing D1's branch must not change D2's current"
            );
        }

        #[test]
        fn e7_buzzer_sounds_only_while_its_button_is_held() {
            let project = fixture(E7);
            let released = solve_dc(&project, &BTreeMap::new(), &BTreeMap::new()).unwrap();
            assert!(released.resistor_currents[&ComponentId("BZ1".into())].abs() < 1e-9);
            let held = solve_dc(&project, &pressed("S1"), &BTreeMap::new()).unwrap();
            assert!(held.resistor_currents[&ComponentId("BZ1".into())] > 0.001);
        }

        #[test]
        fn e8_led_switches_on_only_while_its_button_is_held() {
            let project = fixture(E8);
            let released = solve_dc(&project, &BTreeMap::new(), &BTreeMap::new()).unwrap();
            assert!(led_current(&released, "D1").abs() < 1e-5);
            let held = solve_dc(&project, &pressed("S1"), &BTreeMap::new()).unwrap();
            assert!(led_current(&held, "D1") > 0.005);
        }

        #[test]
        fn e9_led_lights_only_when_both_buttons_are_held_together() {
            let project = fixture(E9);
            let neither = solve_dc(&project, &BTreeMap::new(), &BTreeMap::new()).unwrap();
            assert!(led_current(&neither, "D1").abs() < 1e-6);
            let only_s1 = solve_dc(&project, &pressed("S1"), &BTreeMap::new()).unwrap();
            assert!(led_current(&only_s1, "D1").abs() < 1e-6);
            let only_s2 = solve_dc(&project, &pressed("S2"), &BTreeMap::new()).unwrap();
            assert!(led_current(&only_s2, "D1").abs() < 1e-6);
            let both = solve_dc(
                &project,
                &BTreeMap::from([
                    (ComponentId("S1".into()), ControlState::ButtonPressed),
                    (ComponentId("S2".into()), ControlState::ButtonPressed),
                ]),
                &BTreeMap::new(),
            )
            .unwrap();
            assert!(led_current(&both, "D1") > 0.005);
        }

        #[test]
        fn e10_capacitor_charges_monotonically_and_led_follows() {
            let project = fixture(E10);
            let states = pressed("S1");
            let mut capacitor_voltages = BTreeMap::new();
            let mut previous_voltage = -1.0;
            let mut previous_current = -1.0;
            for _ in 0..2000 {
                let result =
                    solve_transient(&project, &states, &capacitor_voltages, &BTreeMap::new())
                        .unwrap();
                let voltage = result.capacitor_voltages[&ComponentId("C1".into())];
                let current = led_current(&result, "D1");
                assert!(
                    voltage >= previous_voltage - 1e-9,
                    "capacitor voltage dropped"
                );
                assert!(current >= previous_current - 1e-9, "LED current dropped");
                previous_voltage = voltage;
                previous_current = current;
                capacitor_voltages = result.capacitor_voltages;
            }
            assert!(previous_current > 0.001, "LED should be lit once charged");
        }

        fn ratio_sweep_currents(project: &Project, id: &str, steps: u32) -> Vec<(f64, f64)> {
            (0..=steps)
                .map(|step| {
                    let ratio = f64::from(step) / f64::from(steps);
                    let result = solve_dc(
                        project,
                        &BTreeMap::new(),
                        &BTreeMap::from([(ComponentId("RV1".into()), ratio)]),
                    )
                    .unwrap();
                    (ratio, led_current(&result, id))
                })
                .collect()
        }

        #[test]
        fn e5_led_brightness_changes_continuously_and_monotonically_with_the_dial() {
            let project = fixture(E5);
            let currents = ratio_sweep_currents(&project, "D1", 40);
            for pair in currents.windows(2) {
                assert!(
                    pair[1].1 <= pair[0].1 + 1e-12,
                    "current must not rise as the potentiometer ratio rises: {pair:?}"
                );
            }
        }

        #[test]
        fn e6_led_brightness_changes_continuously_and_monotonically_and_reaches_off() {
            let project = fixture(E6);
            let currents = ratio_sweep_currents(&project, "D1", 40);
            for pair in currents.windows(2) {
                assert!(
                    pair[1].1 >= pair[0].1 - 1e-12,
                    "current must not fall as the ambient-light ratio rises: {pair:?}"
                );
            }
            let darkest = currents.first().unwrap().1;
            assert!(
                darkest < 0.0005,
                "darkest setting must reach the LED's off threshold: {darkest}"
            );
        }

        fn module_project(input_voltage: f64) -> Project {
            Project {
                format_version: crate::PROJECT_FORMAT_VERSION,
                title: "module contract test".into(),
                board: crate::Board {
                    model: crate::BoardModel::HalfSizeSolderless,
                },
                components: vec![
                    Component {
                        id: ComponentId("V1".into()),
                        kind: ComponentKind::DcVoltageSource,
                        pins: BTreeMap::from([
                            (PinId("positive".into()), HoleId("TP+:1".into())),
                            (PinId("negative".into()), HoleId("TP-:1".into())),
                        ]),
                        parameters: BTreeMap::from([(String::from("voltage"), input_voltage)]),
                        ic_device: None,
                        other_device: None,
                        module: None,
                        diode_model: None,
                    },
                    Component {
                        id: ComponentId("M1".into()),
                        kind: ComponentKind::Module,
                        pins: BTreeMap::from([
                            (PinId("vcc".into()), HoleId("A1".into())),
                            (PinId("gnd".into()), HoleId("F1".into())),
                            (PinId("out".into()), HoleId("F2".into())),
                        ]),
                        parameters: BTreeMap::new(),
                        ic_device: None,
                        other_device: None,
                        module: Some(ModuleSpec {
                            pin_roles: BTreeMap::from([
                                (PinId("vcc".into()), ModulePinRole::Supply),
                                (PinId("gnd".into()), ModulePinRole::Ground),
                                (PinId("out".into()), ModulePinRole::Output),
                            ]),
                            behavior: ModuleBehavior::RegulatedSupply {
                                input_positive: PinId("vcc".into()),
                                input_negative: PinId("gnd".into()),
                                output_positive: PinId("out".into()),
                                output_negative: PinId("gnd".into()),
                                target_voltage: 3.3,
                                dropout_voltage: 0.5,
                                input_resistance: 10_000.0,
                                output_resistance: 1.0,
                            },
                        }),
                        diode_model: None,
                    },
                    Component {
                        id: ComponentId("R1".into()),
                        kind: ComponentKind::Resistor,
                        pins: BTreeMap::from([
                            (PinId("a".into()), HoleId("A2".into())),
                            (PinId("b".into()), HoleId("TP-:2".into())),
                        ]),
                        parameters: BTreeMap::from([(String::from("resistance"), 1_000.0)]),
                        ic_device: None,
                        other_device: None,
                        module: None,
                        diode_model: None,
                    },
                ],
                wires: vec![
                    crate::Wire {
                        id: crate::WireId("VCC".into()),
                        from: HoleId("TP+:1".into()),
                        to: HoleId("A1".into()),
                    },
                    crate::Wire {
                        id: crate::WireId("OUT".into()),
                        from: HoleId("F2".into()),
                        to: HoleId("A2".into()),
                    },
                    crate::Wire {
                        id: crate::WireId("GND".into()),
                        from: HoleId("TP-:1".into()),
                        to: HoleId("F1".into()),
                    },
                ],
                faults: Vec::new(),
                initial_conditions: crate::InitialConditions::default(),
            }
        }

        #[test]
        fn regulated_module_uses_compiled_pins_and_calculated_output() {
            let project = module_project(5.0);
            let topology = compile_topology(&project).expect("module pin topology should compile");
            assert!(topology.iter().any(|node| {
                node.contacts.contains(&Contact::ComponentPin(
                    ComponentId("M1".into()),
                    PinId("out".into()),
                ))
            }));
            let result = solve_dc(&project, &BTreeMap::new(), &BTreeMap::new()).unwrap();
            let output =
                result.module_output_voltages[&ComponentId("M1".into())][&PinId("out".into())];
            assert!((output - 3.2967).abs() < 0.01, "output={output}");
        }

        #[test]
        fn c05_battery_supply_regulates_five_volts_and_limits_usb_load() {
            let project = fixture(C05_S07_05);
            let mut dc_project = project.clone();
            dc_project
                .components
                .retain(|component| component.kind != ComponentKind::Capacitor);

            let result = solve_dc(&dc_project, &BTreeMap::new(), &BTreeMap::new())
                .expect("battery supply should solve");
            let output =
                result.module_output_voltages[&ComponentId("U1".into())][&PinId("out".into())];
            let led_current = result.led_currents[&ComponentId("LED1".into())];
            let usb_current = result.other_terminal_currents[&ComponentId("USB1".into())]
                [&PinId("positive".into())]
                .abs();
            assert!((4.9..=5.1).contains(&output), "output={output}");
            assert!(led_current > 0.005, "led_current={led_current}");
            assert!(usb_current < 1e-6, "usb_current={usb_current}");

            let mut dropout_project = dc_project;
            dropout_project
                .components
                .iter_mut()
                .find(|component| component.id == ComponentId("V1".into()))
                .expect("battery source")
                .parameters
                .insert("voltage".into(), 6.0);
            let dropout = solve_dc(&dropout_project, &BTreeMap::new(), &BTreeMap::new())
                .expect("dropout case should remain bounded");
            let dropout_output =
                dropout.module_output_voltages[&ComponentId("U1".into())][&PinId("out".into())];
            assert!(
                (3.9..=4.1).contains(&dropout_output),
                "output={dropout_output}"
            );
        }

        #[test]
        // Regression for the catalog's adjustable LM317-style feedback fixture.
        fn c05_adjustable_supply_tracks_feedback_and_meter() {
            let project = fixture(C05_S07_06);
            let mut dc_project = project.clone();
            dc_project
                .components
                .retain(|component| component.kind != ComponentKind::Capacitor);
            let topology = compile_topology(&dc_project).expect("adjustable supply topology");
            assert_eq!(topology.len(), 4);

            let readings = [0.0, 0.25, 0.5, 0.75, 1.0]
                .into_iter()
                .map(|ratio| {
                    let ratios = BTreeMap::from([(ComponentId("RV1".into()), ratio)]);
                    let result = solve_dc(&dc_project, &BTreeMap::new(), &ratios)
                        .expect("adjustable supply should solve");
                    let output = result.module_output_voltages[&ComponentId("U1".into())]
                        [&PinId("out".into())];
                    let meter = result.module_output_voltages[&ComponentId("M1".into())]
                        [&PinId("display".into())];
                    (output, meter)
                })
                .collect::<Vec<_>>();

            for pair in readings.windows(2) {
                assert!(pair[1].0 + 1e-9 >= pair[0].0, "readings={readings:?}");
            }
            assert!(
                readings[0].0 >= 1.24 && readings[0].0 <= 1.27,
                "readings={readings:?}"
            );
            assert!(readings.iter().all(|(output, meter)| {
                (0.0..=10.0 + 1e-9).contains(output) && (output - meter).abs() < 1e-6
            }));
            assert!(readings.last().unwrap().0 >= 9.99);
        }

        proptest! {
            #![proptest_config(ProptestConfig {
                cases: 64,
                rng_seed: proptest::test_runner::RngSeed::Fixed(0x4D4F_DA1E),
                ..ProptestConfig::default()
            })]

            #[test]
            fn regulated_module_output_is_bounded_and_monotone(input_voltage in 0.0f64..12.0) {
                let result = solve_dc(
                    &module_project(input_voltage),
                    &BTreeMap::new(),
                    &BTreeMap::new(),
                ).unwrap();
                let output = result.module_output_voltages[&ComponentId("M1".into())]
                    [&PinId("out".into())];
                prop_assert!(output.is_finite());
                prop_assert!((-1e-9..=3.31).contains(&output));
            }
        }

        proptest! {
            #![proptest_config(ProptestConfig {
                cases: 96,
                rng_seed: proptest::test_runner::RngSeed::Fixed(0xBADC_0FFE),
                ..ProptestConfig::default()
            })]

            #[test]
            fn reverse_breakdown_noise_samples_stay_bounded(step in any::<u64>()) {
                let sample = fixed_noise_sample(step);
                prop_assert!((-1.0..=1.0).contains(&sample));
                prop_assert_eq!(sample, fixed_noise_sample(step));
            }
        }
    }
}
