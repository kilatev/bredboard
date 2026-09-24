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
    pub transistor_collector_currents: BTreeMap<ComponentId, f64>,
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
    Npn {
        id: ComponentId,
        base: usize,
        collector: usize,
        emitter: usize,
        beta: f64,
        saturation: f64,
    },
}

pub const FIXED_STEP_SECONDS: f64 = 100e-6;
pub const MAX_NONLINEAR_ITERATIONS: usize = 80;

/// Solve a DC project using MNA, bounded nonlinear iteration, and deterministic partial pivoting.
/// A floating connected network, ideal-source short, contradictory source loop,
/// or singular ideal-source arrangement is reported as an electrical error.
pub fn solve_dc(
    project: &Project,
    states: &BTreeMap<ComponentId, ControlState>,
) -> Result<SolveResult, ElectricalError> {
    solve_internal(
        project,
        states,
        &BTreeMap::new(),
        None,
        MAX_NONLINEAR_ITERATIONS,
    )
}

/// Solve one 100 microsecond Backward Euler step from the supplied capacitor state.
pub fn solve_transient(
    project: &Project,
    states: &BTreeMap<ComponentId, ControlState>,
    capacitor_voltages: &BTreeMap<ComponentId, f64>,
) -> Result<SolveResult, ElectricalError> {
    solve_transient_with_iteration_limit(
        project,
        states,
        capacitor_voltages,
        MAX_NONLINEAR_ITERATIONS,
    )
}

pub(crate) fn solve_transient_with_iteration_limit(
    project: &Project,
    states: &BTreeMap<ComponentId, ControlState>,
    capacitor_voltages: &BTreeMap<ComponentId, f64>,
    max_iterations: usize,
) -> Result<SolveResult, ElectricalError> {
    solve_internal(
        project,
        states,
        capacitor_voltages,
        Some(FIXED_STEP_SECONDS),
        max_iterations,
    )
}

fn solve_internal(
    project: &Project,
    states: &BTreeMap<ComponentId, ControlState>,
    capacitor_voltages: &BTreeMap<ComponentId, f64>,
    dt: Option<f64>,
    max_iterations: usize,
) -> Result<SolveResult, ElectricalError> {
    let topology = compile_topology(project).map_err(ElectricalError::Structure)?;
    let mut control_state = project.initial_conditions.controls.clone();
    control_state.extend(states.clone());
    let mut cap_state = project.initial_conditions.capacitor_voltages.clone();
    cap_state.extend(capacitor_voltages.clone());
    let (mut branches, nonlinear, node_contacts, active_nodes) =
        make_branches(project, &topology, &control_state, &cap_state, dt)?;
    if branches.is_empty() && nonlinear.is_empty() {
        return Ok(SolveResult {
            node_voltages: Vec::new(),
            resistor_currents: BTreeMap::new(),
            source_currents: BTreeMap::new(),
            switch_currents: BTreeMap::new(),
            capacitor_voltages: BTreeMap::new(),
            capacitor_currents: BTreeMap::new(),
            led_currents: BTreeMap::new(),
            transistor_collector_currents: BTreeMap::new(),
        });
    }

    let mut adjacency = vec![Vec::new(); node_contacts.len()];
    for branch in &branches {
        adjacency[branch.a].push(branch.b);
        adjacency[branch.b].push(branch.a);
    }
    for element in &nonlinear {
        let (a, b, c) = match element {
            NonlinearElement::Led { anode, cathode, .. } => (*anode, *cathode, None),
            NonlinearElement::Npn {
                base,
                collector,
                emitter,
                ..
            } => (*base, *emitter, Some(*collector)),
        };
        adjacency[a].push(b);
        adjacency[b].push(a);
        if let Some(c) = c {
            adjacency[c].push(b);
            adjacency[b].push(c);
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
            stamp_element(element, &guess, &voltage_vars, &mut matrix, &mut rhs);
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
            guess[i] += (solution[i] - guess[i]).clamp(-5.0, 5.0);
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
    let mut transistor_collector_currents = BTreeMap::new();
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
        transistor_collector_currents,
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
            _ => {}
        }
        let Some((a_pin, b_pin, kind, value, previous_voltage)) =
            component_branch(component, states, capacitor_voltages, dt)?
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
    let (ib, _) = led_current(vbe, threshold, 100.0);
    let conductance = (beta * ib.max(0.0) / 0.2).clamp(1e-9, 1.0);
    (ib, conductance * vce, 0.0, conductance)
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

fn stamp_element(
    element: &NonlinearElement,
    guess: &[f64],
    vars: &BTreeMap<usize, usize>,
    matrix: &mut [Vec<f64>],
    rhs: &mut [f64],
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
    fn solve(p: &Project) -> Result<SolveResult, ElectricalError> {
        solve_dc(p, &BTreeMap::new())
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
            let off = solve_transient(&project, &BTreeMap::new(), &BTreeMap::new())
                .unwrap_or_else(|e| panic!("{} off: {e:?}", project.title));
            let on = solve_transient(
                &project,
                &BTreeMap::from([(ComponentId("B1".into()), ControlState::ButtonPressed)]),
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
        let forward = solve_transient(&project, &states, &BTreeMap::new()).unwrap();
        let led = project
            .components
            .iter_mut()
            .find(|c| c.id.0 == "D1")
            .unwrap();
        let anode = led.pins[&crate::PinId("anode".into())].clone();
        let cathode = led.pins[&crate::PinId("cathode".into())].clone();
        led.pins.insert(crate::PinId("anode".into()), cathode);
        led.pins.insert(crate::PinId("cathode".into()), anode);
        let reversed = solve_transient(&project, &states, &BTreeMap::new()).unwrap();
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
        assert!(solve_transient(&project, &pressed, &BTreeMap::new()).is_ok());
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
        let result = solve_transient(&project, &BTreeMap::new(), &BTreeMap::new());
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
            let first = solve_transient(&project, &states, &BTreeMap::new()).unwrap().led_currents[&ComponentId("D1".into())];
            project.components.iter_mut().find(|c| c.id.0 == "R1").unwrap().parameters.insert("resistance".into(), f64::from(low + extra));
            let second = solve_transient(&project, &states, &BTreeMap::new()).unwrap().led_currents[&ComponentId("D1".into())];
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
            let result = solve_transient(&project, &states, &BTreeMap::new());
            prop_assert!(result.is_ok(), "range case did not converge: {:?}", result.err());
        }
        #[test]
        fn transistor_load_current_tracks_beta_and_balances(low in 20u32..100, extra in 1u32..100) {
            let mut project: Project = serde_json::from_str(include_str!("../../../fixtures/projects/transistor-bench.json")).unwrap();
            let states = BTreeMap::from([(ComponentId("B1".into()), ControlState::ButtonPressed)]);
            project.components.iter_mut().find(|c| c.id.0 == "Q1").unwrap().parameters.insert("beta".into(), f64::from(low));
            let first = solve_transient(&project, &states, &BTreeMap::new()).unwrap();
            let lower_current = first.led_currents[&ComponentId("D1".into())];
            project.components.iter_mut().find(|c| c.id.0 == "Q1").unwrap().parameters.insert("beta".into(), f64::from(low + extra));
            let second = solve_transient(&project, &states, &BTreeMap::new()).unwrap();
            let higher_current = second.led_currents[&ComponentId("D1".into())];
            prop_assert!(higher_current > lower_current);
            prop_assert!((second.transistor_collector_currents[&ComponentId("Q1".into())] - higher_current).abs() < 1e-9);
            prop_assert!((second.source_currents[&ComponentId("V1".into())] + higher_current + second.resistor_currents[&ComponentId("R2".into())]).abs() < 1e-9);
        }
    }
}
