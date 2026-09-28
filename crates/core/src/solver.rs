use crate::{
    Component, ComponentId, ComponentKind, Contact, ControlState, Diagnostic, Node, Project,
    compile_topology,
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
        forward: f64,
        resistance: f64,
    },
    Npn {
        id: ComponentId,
        base: usize,
        collector: usize,
        emitter: usize,
        beta: f64,
        saturation: f64,
    },
    Pnp {
        id: ComponentId,
        base: usize,
        collector: usize,
        emitter: usize,
        beta: f64,
        saturation: f64,
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
    },
}

pub const FIXED_STEP_SECONDS: f64 = 100e-6;
pub const MAX_NONLINEAR_ITERATIONS: usize = 80;
/// Under-relaxation keeps coupled LED/NPN transient solves deterministic while
/// retaining the bounded iteration and explicit nonconvergence diagnostic.
const NONLINEAR_RELAXATION: f64 = 0.25;

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
        None,
        MAX_NONLINEAR_ITERATIONS,
    )
}

/// Solve one 100 microsecond Backward Euler step from the supplied capacitor state.
pub fn solve_transient(
    project: &Project,
    states: &BTreeMap<ComponentId, ControlState>,
    capacitor_voltages: &BTreeMap<ComponentId, f64>,
    ratios: &BTreeMap<ComponentId, f64>,
) -> Result<SolveResult, ElectricalError> {
    solve_transient_with_iteration_limit(
        project,
        states,
        capacitor_voltages,
        ratios,
        MAX_NONLINEAR_ITERATIONS,
    )
}

pub(crate) fn solve_transient_with_iteration_limit(
    project: &Project,
    states: &BTreeMap<ComponentId, ControlState>,
    capacitor_voltages: &BTreeMap<ComponentId, f64>,
    ratios: &BTreeMap<ComponentId, f64>,
    max_iterations: usize,
) -> Result<SolveResult, ElectricalError> {
    solve_internal(
        project,
        states,
        capacitor_voltages,
        ratios,
        Some(FIXED_STEP_SECONDS),
        max_iterations,
    )
}

fn solve_internal(
    project: &Project,
    states: &BTreeMap<ComponentId, ControlState>,
    capacitor_voltages: &BTreeMap<ComponentId, f64>,
    ratios: &BTreeMap<ComponentId, f64>,
    dt: Option<f64>,
    max_iterations: usize,
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
        dt,
    )?;
    let coupled_transient = dt.is_some()
        && branches
            .iter()
            .filter(|branch| matches!(branch.kind, BranchKind::Capacitor))
            .count()
            >= 2
        && nonlinear.len() >= 2;
    let use_collector_base_jacobian = dt.is_none()
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
            capacitor_voltages: BTreeMap::new(),
            capacitor_currents: BTreeMap::new(),
            led_currents: BTreeMap::new(),
            diode_currents: BTreeMap::new(),
            transistor_collector_currents: BTreeMap::new(),
            pnp_collector_currents: BTreeMap::new(),
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
    for _ in 0..max_iterations {
        let mut matrix = vec![vec![0.0; size]; size];
        let mut rhs = vec![0.0; size];
        for branch in &branches {
            match branch.kind {
                BranchKind::Resistor | BranchKind::Capacitor => {
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
            format!("nonlinear circuit did not converge within {max_iterations} iterations"),
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
    let mut capacitor_voltages = BTreeMap::new();
    let mut capacitor_currents = BTreeMap::new();
    let mut led_currents = BTreeMap::new();
    let mut diode_currents = BTreeMap::new();
    let mut transistor_collector_currents = BTreeMap::new();
    let mut pnp_collector_currents = BTreeMap::new();
    for branch in &branches {
        let current = match branch.kind {
            BranchKind::Resistor => {
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
                forward,
                resistance,
            } => {
                let v = voltage(&solution, &voltage_vars, *anode)
                    - voltage(&solution, &voltage_vars, *cathode);
                diode_currents.insert(id.clone(), led_current(v, *forward, *resistance).0);
            }
            NonlinearElement::Npn {
                id,
                base,
                collector,
                emitter,
                beta,
                saturation,
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
            NonlinearElement::LogicGate { .. }
            | NonlinearElement::SchmittInverter { .. }
            | NonlinearElement::Comparator { .. }
            | NonlinearElement::Timer555 { .. }
            | NonlinearElement::DFlipFlop { .. } => {}
        }
    }
    Ok(SolveResult {
        node_voltages: voltages,
        resistor_currents,
        source_currents,
        switch_currents,
        capacitor_voltages,
        capacitor_currents,
        led_currents,
        diode_currents,
        transistor_collector_currents,
        pnp_collector_currents,
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
    dt: Option<f64>,
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
                    forward: component.parameters["forward_voltage"],
                    resistance: component.parameters["series_resistance"],
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
                });
                continue;
            }
            _ => {}
        }
        let Some((a_pin, b_pin, kind, value, previous_voltage)) =
            component_branch(component, states, capacitor_voltages, ratios, dt)?
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
        ComponentKind::Buzzer | ComponentKind::Speaker => Some((
            "positive",
            "negative",
            BranchKind::Resistor,
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
        ComponentKind::Photoresistor | ComponentKind::Thermistor => {
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

// A smooth forward diode with a finite series slope. The current is zero to
// numerical precision in reverse bias and approaches (V - Vf) / Rs forward.
fn led_current(v: f64, forward: f64, resistance: f64) -> (f64, f64) {
    let x = ((v - forward) / 0.05).clamp(-80.0, 80.0);
    let softplus = if x > 30.0 { x } else { (1.0 + x.exp()).ln() };
    let sigmoid = if x >= 0.0 {
        1.0 / (1.0 + (-x).exp())
    } else {
        x.exp() / (1.0 + x.exp())
    };
    (
        0.05 * softplus / resistance + v * 1e-9,
        sigmoid / resistance + 1e-9,
    )
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
        NonlinearElement::Npn {
            base,
            collector,
            emitter,
            beta,
            saturation,
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
            control: _,
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
            let supply = voltage(guess, vars, *vcc).max(1e-6);
            let reset_high = voltage(guess, vars, *reset) > supply * 0.4;
            let threshold_high = voltage(guess, vars, *threshold) > supply * (2.0 / 3.0);
            let trigger_low = voltage(guess, vars, *trigger) < supply / 3.0;
            let output_high = if !reset_high {
                false
            } else if trigger_low {
                true
            } else if threshold_high {
                false
            } else {
                voltage(guess, vars, *output) > supply * 0.5
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
            clock,
            data,
            not_q,
            q,
            reset,
            set,
            vcc,
            gnd,
            resistance,
            ..
        } => {
            let supply = voltage(guess, vars, *vcc);
            let clock_high = voltage(guess, vars, *clock) > supply * 0.5;
            let set_high = voltage(guess, vars, *set) > supply * 0.5;
            let reset_high = voltage(guess, vars, *reset) > supply * 0.5;
            if !clock_high && !set_high && !reset_high {
                return;
            }
            let q_high = if reset_high {
                false
            } else if set_high {
                true
            } else {
                logic_high(voltage(guess, vars, *data), supply)
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
    }
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
                },
                Component {
                    id: ComponentId("R1".into()),
                    kind: ComponentKind::Resistor,
                    pins: BTreeMap::from([
                        (crate::PinId("a".into()), crate::HoleId("B1".into())),
                        (crate::PinId("b".into()), crate::HoleId("TP-:4".into())),
                    ]),
                    parameters: BTreeMap::from([(String::from("resistance"), 470.0)]),
                },
                Component {
                    id: ComponentId("R2".into()),
                    kind: ComponentKind::Resistor,
                    pins: BTreeMap::from([
                        (crate::PinId("a".into()), crate::HoleId("A2".into())),
                        (crate::PinId("b".into()), crate::HoleId("TP-:2".into())),
                    ]),
                    parameters: BTreeMap::from([(String::from("resistance"), 10_000.0)]),
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
                },
            ],
            wires: Vec::new(),
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
        fixture_json!(C01_S02_06, "c01-s02-06-transistor-logic.json");
        fixture_json!(C02_S03_03, "c02-s03-03-logic-gates.json");
        fixture_json!(C02_S03_01, "c02-s03-01-555-flasher.json");
        fixture_json!(C02_S03_04, "c02-s03-04-rs-latch.json");
        fixture_json!(C02_S03_06, "c02-s03-06-thermostat.json");
        fixture_json!(C02_S03_05, "c02-s03-05-debounce.json");
        fixture_json!(C02_S03_02, "c02-s03-02-555-monostable.json");
        fixture_json!(C02_S03_07, "c02-s03-07-light-theremin.json");

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
    }
}
