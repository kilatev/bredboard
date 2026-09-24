use crate::{
    Component, ComponentId, ComponentKind, Contact, Diagnostic, Node, Project, compile_topology,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Explicit interactive state supplied by the caller; buttons default open and switches NC.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlState {
    ButtonPressed,
    ButtonReleased,
    SwitchNormallyClosed,
    SwitchNormallyOpen,
}

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

#[derive(Clone, Debug, PartialEq)]
pub struct SolveResult {
    /// Absolute voltages use one deterministic zero reference per connected circuit.
    pub node_voltages: BTreeMap<Vec<Contact>, f64>,
    /// Resistor current is positive from pin `a` to pin `b`.
    pub resistor_currents: BTreeMap<ComponentId, f64>,
    /// Source current is positive from pin `positive` to pin `negative`.
    pub source_currents: BTreeMap<ComponentId, f64>,
    /// Current through a closed ideal switch, positive from first to second pin.
    pub switch_currents: BTreeMap<ComponentId, f64>,
}

#[derive(Clone)]
struct Branch {
    component: ComponentId,
    kind: BranchKind,
    a: usize,
    b: usize,
    value: f64,
}
#[derive(Clone, Copy)]
enum BranchKind {
    Resistor,
    VoltageSource,
    Switch,
}

/// Solve a linear DC project using MNA and deterministic partial pivoting.
/// A floating connected network, ideal-source short, contradictory source loop,
/// or singular ideal-source arrangement is reported as an electrical error.
pub fn solve_dc(
    project: &Project,
    states: &BTreeMap<ComponentId, ControlState>,
) -> Result<SolveResult, ElectricalError> {
    let topology = compile_topology(project).map_err(ElectricalError::Structure)?;
    let (mut branches, node_contacts, active_nodes) = make_branches(project, &topology, states)?;
    if branches.is_empty() {
        return Ok(SolveResult {
            node_voltages: BTreeMap::new(),
            resistor_currents: BTreeMap::new(),
            source_currents: BTreeMap::new(),
            switch_currents: BTreeMap::new(),
        });
    }

    let mut adjacency = vec![Vec::new(); node_contacts.len()];
    for branch in &branches {
        adjacency[branch.a].push(branch.b);
        adjacency[branch.b].push(branch.a);
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
    let mut matrix = vec![vec![0.0; size]; size];
    let mut rhs = vec![0.0; size];
    for branch in &branches {
        match branch.kind {
            BranchKind::Resistor => {
                let g = 1.0 / branch.value;
                stamp_conductance(&mut matrix, &voltage_vars, branch.a, branch.b, g);
            }
            BranchKind::VoltageSource | BranchKind::Switch => {}
        }
    }
    for (offset, branch) in constraints.iter().enumerate() {
        let current_var = voltage_vars.len() + offset;
        stamp_constraint(&mut matrix, &voltage_vars, branch.a, branch.b, current_var);
        rhs[current_var] = branch.value;
    }
    let solution = gaussian_solve(matrix, rhs).ok_or_else(|| {
        calc(
            "singular_system",
            "circuit has an underdetermined ideal-source arrangement",
        )
    })?;
    let mut voltages = BTreeMap::new();
    for &node in &active_nodes {
        let voltage = voltage_vars.get(&node).map_or(0.0, |&i| solution[i]);
        voltages.insert(node_contacts[node].clone(), voltage);
    }
    let mut resistor_currents = BTreeMap::new();
    let mut source_currents = BTreeMap::new();
    let mut switch_currents = BTreeMap::new();
    for branch in &branches {
        let current = match branch.kind {
            BranchKind::Resistor => {
                (voltage(&solution, &voltage_vars, branch.a)
                    - voltage(&solution, &voltage_vars, branch.b))
                    / branch.value
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
        }
    }
    Ok(SolveResult {
        node_voltages: voltages,
        resistor_currents,
        source_currents,
        switch_currents,
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
type CompiledBranches = (Vec<Branch>, Vec<Vec<Contact>>, BTreeSet<usize>);
fn make_branches(
    project: &Project,
    topology: &[Node],
    states: &BTreeMap<ComponentId, ControlState>,
) -> Result<CompiledBranches, ElectricalError> {
    let node_contacts: Vec<_> = topology.iter().map(|n| n.contacts.clone()).collect();
    let mut branches = Vec::new();
    let mut active = BTreeSet::new();
    for component in &project.components {
        let Some((a_pin, b_pin, kind, value)) = component_branch(component, states)? else {
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
        });
    }
    Ok((branches, node_contacts, active))
}
fn component_branch(
    c: &Component,
    states: &BTreeMap<ComponentId, ControlState>,
) -> Result<Option<(&'static str, &'static str, BranchKind, f64)>, ElectricalError> {
    let value = |name: &str| {
        c.parameters.get(name).copied().ok_or_else(|| {
            calc(
                "missing_parameter",
                format!("component {} is missing {name}", c.id.0),
            )
        })
    };
    Ok(match c.kind {
        ComponentKind::Resistor => Some(("a", "b", BranchKind::Resistor, value("resistance")?)),
        ComponentKind::DcVoltageSource => Some((
            "positive",
            "negative",
            BranchKind::VoltageSource,
            value("voltage")?,
        )),
        ComponentKind::MomentaryButton => match states
            .get(&c.id)
            .copied()
            .unwrap_or(ControlState::ButtonReleased)
        {
            ControlState::ButtonPressed => Some(("a", "b", BranchKind::Switch, 0.0)),
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
                Some(("common", "normally_closed", BranchKind::Switch, 0.0))
            }
            ControlState::SwitchNormallyOpen => {
                Some(("common", "normally_open", BranchKind::Switch, 0.0))
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
            .find(|(contacts, _)| {
                contacts.contains(&Contact::ComponentPin(
                    ComponentId(component.into()),
                    crate::PinId(pin.into()),
                ))
            })
            .unwrap()
            .1
            .to_owned()
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
    }
}
