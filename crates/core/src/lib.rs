//! Platform-independent project model and derived breadboard connectivity.
mod persistence;
mod simulation;
mod solver;
pub use persistence::{
    ACTION_LOG_FORMAT_VERSION, ActionEvent, ActionLog, MODEL_VERSION, PersistenceError,
    SNAPSHOT_FORMAT_VERSION, SOLVER_VERSION, Snapshot, replay_action_log, restore_snapshot,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
pub use simulation::{
    Action, STEP_SECONDS, SimulationDiagnostic, SimulationState, advance_steps, apply_actions,
};
pub use solver::{
    ElectricalDiagnostic, ElectricalError, NodeVoltage, SolveResult, solve_dc, solve_transient,
};
use std::collections::{BTreeMap, BTreeSet};

/// Version of the core crate used by applications and workspace tools.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PROJECT_FORMAT_VERSION: u32 = 1;
pub const MAX_COMPONENTS: usize = 64;
pub const MAX_NODES: usize = 128;
pub const MAX_WIRES: usize = 256;

macro_rules! string_id {
    ($name:ident) => {
        #[derive(
            Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
        )]
        #[serde(transparent)]
        pub struct $name(pub String);
    };
}
string_id!(ComponentId);
string_id!(PinId);
string_id!(HoleId);
string_id!(WireId);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(
    description = "A versioned breadboard project with components, placements, wires, and initial conditions."
)]
pub struct Project {
    pub format_version: u32,
    pub title: String,
    pub board: Board,
    pub components: Vec<Component>,
    pub wires: Vec<Wire>,
    #[serde(default)]
    pub initial_conditions: InitialConditions,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct InitialConditions {
    #[serde(default)]
    pub capacitor_voltages: BTreeMap<ComponentId, f64>,
    #[serde(default)]
    pub controls: BTreeMap<ComponentId, ControlState>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ControlState {
    ButtonPressed,
    ButtonReleased,
    SwitchNormallyClosed,
    SwitchNormallyOpen,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Board {
    pub model: BoardModel,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BoardModel {
    HalfSizeSolderless,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Component {
    pub id: ComponentId,
    pub kind: ComponentKind,
    pub pins: BTreeMap<PinId, HoleId>,
    pub parameters: BTreeMap<String, f64>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ComponentKind {
    DcVoltageSource,
    Resistor,
    Led,
    Capacitor,
    NpnTransistor,
    MomentaryButton,
    ChangeoverSwitch,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Wire {
    pub id: WireId,
    pub from: HoleId,
    pub to: HoleId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub path: String,
    pub message: String,
}
impl Diagnostic {
    fn new(code: &'static str, path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code,
            path: path.into(),
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
pub enum Contact {
    Hole(HoleId),
    ComponentPin(ComponentId, PinId),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    pub contacts: Vec<Contact>,
}

/// Validate project references and derive nodes. Rows 1–30 have isolated A–E and F–J
/// five-hole strips; rows do not connect vertically. Rails TP+/TP- and BP+/BP- are
/// continuous from rows 1–30. Only wire endpoints connect; geometric crossings do not.
pub fn compile_topology(project: &Project) -> Result<Vec<Node>, Vec<Diagnostic>> {
    let mut errors = Vec::new();
    if project.format_version != PROJECT_FORMAT_VERSION {
        errors.push(Diagnostic::new(
            "unsupported_version",
            "format_version",
            format!("supported project format is {PROJECT_FORMAT_VERSION}"),
        ));
    }
    if project.title.trim().is_empty() {
        errors.push(Diagnostic::new(
            "invalid_title",
            "title",
            "title must not be empty",
        ));
    }
    if project.components.len() > MAX_COMPONENTS {
        errors.push(Diagnostic::new(
            "component_limit",
            "components",
            format!("at most {MAX_COMPONENTS} components are allowed"),
        ));
    }
    if project.wires.len() > MAX_WIRES {
        errors.push(Diagnostic::new(
            "wire_limit",
            "wires",
            format!("at most {MAX_WIRES} wires are allowed"),
        ));
    }

    let mut component_ids = BTreeSet::new();
    let mut wire_ids = BTreeSet::new();
    let mut parent: BTreeMap<HoleId, HoleId> = BTreeMap::new();
    fn root(parent: &mut BTreeMap<HoleId, HoleId>, x: &HoleId) -> HoleId {
        let p = parent.entry(x.clone()).or_insert_with(|| x.clone()).clone();
        if p == *x {
            p
        } else {
            let r = root(parent, &p);
            parent.insert(x.clone(), r.clone());
            r
        }
    }
    fn union(parent: &mut BTreeMap<HoleId, HoleId>, a: &HoleId, b: &HoleId) {
        let ra = root(parent, a);
        let rb = root(parent, b);
        if ra != rb {
            let (lo, hi) = if ra < rb { (ra, rb) } else { (rb, ra) };
            parent.insert(hi, lo);
        }
    }
    let valid_hole = |h: &HoleId| hole_group(&h.0).is_some();
    for wire in &project.wires {
        if !wire_ids.insert(wire.id.clone()) {
            errors.push(Diagnostic::new(
                "duplicate_wire_id",
                "wires",
                format!("duplicate wire id {}", wire.id.0),
            ));
        }
        for (field, hole) in [("from", &wire.from), ("to", &wire.to)] {
            if !valid_hole(hole) {
                errors.push(Diagnostic::new(
                    "invalid_hole",
                    format!("wires.{}.{}", wire.id.0, field),
                    format!("unknown board hole {}", hole.0),
                ));
            }
        }
        if valid_hole(&wire.from) && valid_hole(&wire.to) {
            union(&mut parent, &wire.from, &wire.to);
        }
    }
    let mut pin_contacts: BTreeMap<HoleId, Vec<Contact>> = BTreeMap::new();
    for c in &project.components {
        if !component_ids.insert(c.id.clone()) {
            errors.push(Diagnostic::new(
                "duplicate_component_id",
                "components",
                format!("duplicate component id {}", c.id.0),
            ));
        }
        if c.id.0.trim().is_empty() || c.id.0.len() > 64 {
            errors.push(Diagnostic::new(
                "invalid_component_id",
                format!("components.{}.id", c.id.0),
                "id must contain 1–64 characters",
            ));
        }
        let expected = pins_for(c.kind);
        if c.pins.len() != expected.len()
            || expected
                .iter()
                .any(|p| !c.pins.contains_key(&PinId((*p).into())))
        {
            errors.push(Diagnostic::new(
                "invalid_pins",
                format!("components.{}.pins", c.id.0),
                format!("expected pins: {}", expected.join(", ")),
            ));
        }
        for (pin, hole) in &c.pins {
            if !expected.contains(&pin.0.as_str()) {
                errors.push(Diagnostic::new(
                    "unknown_pin",
                    format!("components.{}.pins.{}", c.id.0, pin.0),
                    "pin is not defined for this component",
                ));
            }
            if !valid_hole(hole) {
                errors.push(Diagnostic::new(
                    "invalid_hole",
                    format!("components.{}.pins.{}", c.id.0, pin.0),
                    format!("unknown board hole {}", hole.0),
                ));
            } else {
                pin_contacts
                    .entry(hole.clone())
                    .or_default()
                    .push(Contact::ComponentPin(c.id.clone(), pin.clone()));
                parent.entry(hole.clone()).or_insert_with(|| hole.clone());
            }
        }
        for (key, value) in &c.parameters {
            if !value.is_finite() {
                errors.push(Diagnostic::new(
                    "non_finite_parameter",
                    format!("components.{}.parameters.{key}", c.id.0),
                    "parameter must be finite",
                ));
            }
            let range = parameter_range(c.kind, key);
            match range {
                None => errors.push(Diagnostic::new(
                    "unknown_parameter",
                    format!("components.{}.parameters.{key}", c.id.0),
                    "parameter is not defined for this component",
                )),
                Some((min, max)) if !value.is_finite() || *value < min || *value > max => errors
                    .push(Diagnostic::new(
                        "parameter_out_of_range",
                        format!("components.{}.parameters.{key}", c.id.0),
                        format!("parameter must be in {min}..={max}"),
                    )),
                _ => {}
            }
        }
        for key in required_parameters(c.kind) {
            if !c.parameters.contains_key(*key) {
                errors.push(Diagnostic::new(
                    "missing_parameter",
                    format!("components.{}.parameters.{key}", c.id.0),
                    "required parameter is missing",
                ));
            }
        }
    }
    for (id, voltage) in &project.initial_conditions.capacitor_voltages {
        match project.components.iter().find(|c| &c.id == id) {
            Some(c) if c.kind == ComponentKind::Capacitor && voltage.is_finite() => {}
            Some(c) if c.kind == ComponentKind::Capacitor => errors.push(Diagnostic::new(
                "non_finite_initial_condition",
                format!("initial_conditions.capacitor_voltages.{}", id.0),
                "initial capacitor voltage must be finite",
            )),
            _ => errors.push(Diagnostic::new(
                "invalid_initial_condition_reference",
                format!("initial_conditions.capacitor_voltages.{}", id.0),
                "initial voltage must refer to a capacitor component",
            )),
        }
    }
    for (id, state) in &project.initial_conditions.controls {
        let valid = project.components.iter().any(|c| match (c.kind, state) {
            (
                ComponentKind::MomentaryButton,
                ControlState::ButtonPressed | ControlState::ButtonReleased,
            )
            | (
                ComponentKind::ChangeoverSwitch,
                ControlState::SwitchNormallyClosed | ControlState::SwitchNormallyOpen,
            ) => &c.id == id,
            _ => false,
        });
        if !valid {
            errors.push(Diagnostic::new(
                "invalid_initial_control",
                format!("initial_conditions.controls.{}", id.0),
                "control state must match a button or switch component",
            ));
        }
    }
    // All holes in a contact group share a node, including the four continuous rails.
    let holes: Vec<_> = parent.keys().cloned().collect();
    for i in 0..holes.len() {
        for j in (i + 1)..holes.len() {
            if hole_group(&holes[i].0).is_some()
                && hole_group(&holes[i].0) == hole_group(&holes[j].0)
            {
                union(&mut parent, &holes[i], &holes[j]);
            }
        }
    }
    if !errors.is_empty() {
        errors.sort_by(|a, b| (&a.path, a.code).cmp(&(&b.path, b.code)));
        return Err(errors);
    }
    let mut groups: BTreeMap<HoleId, Vec<Contact>> = BTreeMap::new();
    for h in parent.keys().cloned().collect::<Vec<_>>() {
        let r = root(&mut parent, &h);
        let contacts = groups.entry(r).or_default();
        contacts.push(Contact::Hole(h.clone()));
        let group = hole_group(&h.0).expect("validated hole");
        if group.starts_with("TP") || group.starts_with("BP") {
            for row in 1..=30 {
                contacts.push(Contact::Hole(HoleId(format!("{group}:{row}"))));
            }
        } else if let Some((strip, row)) = group.split_once(':')
            && strip == "strip"
        {
            let start = if row.starts_with("left") { 'A' } else { 'F' };
            let row = row.rsplit(':').next().unwrap();
            for col in start..=if start == 'A' { 'E' } else { 'J' } {
                contacts.push(Contact::Hole(HoleId(format!("{col}{row}"))));
            }
        }
    }
    for (h, contacts) in pin_contacts {
        let r = root(&mut parent, &h);
        groups.entry(r).or_default().extend(contacts);
    }
    let mut nodes: Vec<Node> = groups
        .into_values()
        .map(|mut contacts| {
            contacts.sort();
            contacts.dedup();
            Node { contacts }
        })
        .collect();
    nodes.sort_by(|a, b| a.contacts.cmp(&b.contacts));
    if nodes.len() > MAX_NODES {
        return Err(vec![Diagnostic::new(
            "node_limit",
            "board",
            format!(
                "compiled topology has {} nodes; at most {MAX_NODES} are allowed",
                nodes.len()
            ),
        )]);
    }
    Ok(nodes)
}

fn pins_for(k: ComponentKind) -> &'static [&'static str] {
    match k {
        ComponentKind::DcVoltageSource => &["negative", "positive"],
        ComponentKind::Resistor => &["a", "b"],
        ComponentKind::Led => &["anode", "cathode"],
        ComponentKind::Capacitor => &["negative", "positive"],
        ComponentKind::NpnTransistor => &["base", "collector", "emitter"],
        ComponentKind::MomentaryButton => &["a", "b"],
        ComponentKind::ChangeoverSwitch => &["common", "normally_closed", "normally_open"],
    }
}
fn required_parameters(k: ComponentKind) -> &'static [&'static str] {
    match k {
        ComponentKind::DcVoltageSource => &["voltage"],
        ComponentKind::Resistor => &["resistance"],
        ComponentKind::Led => &["forward_voltage", "series_resistance"],
        ComponentKind::Capacitor => &["capacitance"],
        ComponentKind::NpnTransistor => &["beta", "saturation_current"],
        ComponentKind::MomentaryButton | ComponentKind::ChangeoverSwitch => &[],
    }
}
fn parameter_range(k: ComponentKind, p: &str) -> Option<(f64, f64)> {
    match (k, p) {
        (ComponentKind::DcVoltageSource, "voltage") => Some((-1000.0, 1000.0)),
        (ComponentKind::Resistor, "resistance") => Some((1e-3, 1e9)),
        (ComponentKind::Led, "forward_voltage") => Some((0.0, 10.0)),
        (ComponentKind::Led, "series_resistance") => Some((1e-3, 1e9)),
        (ComponentKind::Capacitor, "capacitance") => Some((1e-12, 1e3)),
        (ComponentKind::NpnTransistor, "beta") => Some((1.0, 1000.0)),
        (ComponentKind::NpnTransistor, "saturation_current") => Some((1e-18, 1.0)),
        _ => None,
    }
}
fn hole_group(s: &str) -> Option<String> {
    if let Some((rail, row)) = s.split_once(':') {
        if ["TP+", "TP-", "BP+", "BP-"].contains(&rail) {
            let r: u8 = row.parse().ok()?;
            if (1..=30).contains(&r) {
                return Some(rail.into());
            }
        }
        return None;
    }
    let mut chars = s.chars();
    let c = chars.next()?;
    let row: String = chars.collect();
    let r: u8 = row.parse().ok()?;
    if !(1..=30).contains(&r) || !(('A'..='J').contains(&c)) {
        return None;
    }
    Some(format!(
        "strip:{}:{r}",
        if c <= 'E' { "left" } else { "right" }
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    fn project() -> Project {
        Project {
            format_version: 1,
            title: "Example".into(),
            board: Board {
                model: BoardModel::HalfSizeSolderless,
            },
            components: vec![Component {
                id: ComponentId("R1".into()),
                kind: ComponentKind::Resistor,
                pins: BTreeMap::from([
                    (PinId("a".into()), HoleId("A1".into())),
                    (PinId("b".into()), HoleId("F1".into())),
                ]),
                parameters: BTreeMap::from([("resistance".into(), 1000.0)]),
            }],
            initial_conditions: InitialConditions::default(),
            wires: vec![Wire {
                id: WireId("W1".into()),
                from: HoleId("A2".into()),
                to: HoleId("J30".into()),
            }],
        }
    }
    proptest! {
        #![proptest_config(ProptestConfig { cases: 64, rng_seed: proptest::test_runner::RngSeed::Fixed(0xB8ED_B04D), .. ProptestConfig::default() })]
        #[test] fn serialization_round_trip(title in ".{1,40}") { let mut p=project(); p.title=title; let json=serde_json::to_string(&p).unwrap(); let decoded:Project=serde_json::from_str(&json).unwrap(); prop_assert_eq!(decoded,p); }
        #[test] fn id_rename_does_not_change_hole_partition(id in "[a-zA-Z0-9]{1,12}") { let mut p=project(); let expected=compile_topology(&p).unwrap(); p.components[0].id=ComponentId(id); let actual=compile_topology(&p).unwrap(); let holes=|nodes:&[Node]| nodes.iter().map(|n|n.contacts.iter().filter_map(|c|if let Contact::Hole(h)=c{Some(h.0.clone())}else{None}).collect::<Vec<_>>()).collect::<Vec<_>>(); prop_assert_eq!(holes(&expected),holes(&actual)); }
    }
    #[test]
    fn schema_accepts_example_and_rejects_wrong_shape() {
        let schema =
            schemars::SchemaGenerator::new(schemars::generate::SchemaSettings::draft2020_12())
                .into_root_schema_for::<Project>();
        let schema = serde_json::to_value(schema).unwrap();
        assert!(
            schema["description"]
                .as_str()
                .unwrap()
                .contains("versioned breadboard project")
        );
        let validator = jsonschema::validator_for(&schema).unwrap();
        let accepted: serde_json::Value = serde_json::from_str(include_str!(
            "../../../fixtures/projects/valid-resistor.json"
        ))
        .unwrap();
        assert!(validator.is_valid(&accepted));
        assert!(!validator.is_valid(&serde_json::json!({"format_version": 1})));
    }
    #[test]
    fn array_order_does_not_change_topology() {
        let mut p = project();
        p.components.push(Component {
            id: ComponentId("R2".into()),
            kind: ComponentKind::Resistor,
            pins: BTreeMap::from([
                (PinId("a".into()), HoleId("A3".into())),
                (PinId("b".into()), HoleId("F3".into())),
            ]),
            parameters: BTreeMap::from([("resistance".into(), 220.0)]),
        });
        let before = compile_topology(&p).unwrap();
        p.components.reverse();
        p.wires.reverse();
        assert_eq!(compile_topology(&p).unwrap(), before);
    }
    #[test]
    fn connected_strips_and_endpoint_only_wires() {
        let nodes = compile_topology(&project()).unwrap();
        let row = nodes
            .iter()
            .find(|n| n.contacts.contains(&Contact::Hole(HoleId("A1".into()))))
            .unwrap();
        assert!(row.contacts.contains(&Contact::Hole(HoleId("E1".into()))));
        assert!(!row.contacts.contains(&Contact::Hole(HoleId("F1".into()))));
        let wire = nodes
            .iter()
            .find(|n| n.contacts.contains(&Contact::Hole(HoleId("A2".into()))))
            .unwrap();
        assert!(wire.contacts.contains(&Contact::Hole(HoleId("J30".into()))));
        let mut rails = project();
        rails.wires = vec![Wire {
            id: WireId("rail".into()),
            from: HoleId("TP+:1".into()),
            to: HoleId("TP+:30".into()),
        }];
        let rail = compile_topology(&rails)
            .unwrap()
            .into_iter()
            .find(|n| n.contacts.contains(&Contact::Hole(HoleId("TP+:1".into()))))
            .unwrap();
        assert!(
            rail.contacts
                .contains(&Contact::Hole(HoleId("TP+:30".into())))
        );
    }
    #[test]
    fn rejects_bad_reference_version_and_parameters() {
        let mut p = project();
        p.format_version = 9;
        p.components[0]
            .pins
            .insert(PinId("x".into()), HoleId("Z99".into()));
        p.components[0]
            .parameters
            .insert("resistance".into(), f64::NAN);
        let e = compile_topology(&p).unwrap_err();
        assert!(e.iter().any(|d| d.code == "unsupported_version"));
        assert!(e.iter().any(|d| d.code == "invalid_hole"));
        assert!(e.iter().any(|d| d.code == "non_finite_parameter"));
    }
    #[test]
    fn rejects_duplicate_ids_and_scope_overflow() {
        let mut p = project();
        p.components.push(p.components[0].clone());
        assert!(
            compile_topology(&p)
                .unwrap_err()
                .iter()
                .any(|d| d.code == "duplicate_component_id")
        );
        p.components = (0..=MAX_COMPONENTS)
            .map(|i| Component {
                id: ComponentId(format!("R{i}")),
                kind: ComponentKind::Resistor,
                pins: BTreeMap::from([
                    (PinId("a".into()), HoleId("A1".into())),
                    (PinId("b".into()), HoleId("F1".into())),
                ]),
                parameters: BTreeMap::from([("resistance".into(), 1000.0)]),
            })
            .collect();
        assert!(
            compile_topology(&p)
                .unwrap_err()
                .iter()
                .any(|d| d.code == "component_limit")
        );
    }
    #[test]
    fn rejects_non_ascii_holes_without_panicking() {
        let mut p = project();
        p.components[0]
            .pins
            .insert(PinId("a".into()), HoleId("Å1".into()));
        assert!(
            compile_topology(&p)
                .unwrap_err()
                .iter()
                .any(|d| d.code == "invalid_hole")
        );
    }
}
