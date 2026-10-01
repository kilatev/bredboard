//! Platform-independent project model and derived breadboard connectivity.
mod diode;
mod ic_device;
mod module;
mod other_device;
mod persistence;
mod simulation;
mod solver;
pub use diode::{
    DIODE_MAX_BREAKDOWN_RESISTANCE, DIODE_MAX_BREAKDOWN_VOLTAGE, DIODE_MAX_FORWARD_VOLTAGE,
    DIODE_MAX_SERIES_RESISTANCE, DIODE_MIN_BREAKDOWN_RESISTANCE, DIODE_MIN_BREAKDOWN_VOLTAGE,
    DIODE_MIN_FORWARD_VOLTAGE, DIODE_MIN_SERIES_RESISTANCE, DiodeKind, DiodeLinearization,
    DiodeModel, DiodeModelError, DiodeSpec,
};
pub use ic_device::{
    IC_DEVICE_MAX_RESISTANCE, IC_DEVICE_MIN_RESISTANCE, IcDeviceBehavior, IcDeviceLinearInput,
    IcDevicePinRole, IcDeviceSpec, IcLogicOperation, MAX_IC_DEVICE_INPUTS, MAX_IC_DEVICE_PINS,
};
pub use module::{
    MAX_MODULE_CHANNELS, MAX_MODULE_INPUTS, MAX_MODULE_PINS, MODULE_MAX_RESISTANCE,
    MODULE_MIN_RESISTANCE, ModuleBehavior, ModuleChannel, ModuleInput, ModulePinRole, ModuleSpec,
};
pub use other_device::{
    BjtPolarity, BjtTestSocket, BjtTestState, MAX_OTHER_DEVICE_PINS, OTHER_DEVICE_MAX_RESISTANCE,
    OTHER_DEVICE_MIN_RESISTANCE, OtherDeviceBehavior, OtherDeviceLinearInput, OtherDevicePinRole,
    OtherDeviceSpec, controlled_resistance, ring_modulator_output,
};
pub use persistence::{
    ACTION_LOG_FORMAT_VERSION, ActionEvent, ActionLog, MODEL_VERSION, PersistenceError,
    SNAPSHOT_FORMAT_VERSION, SOLVER_VERSION, Snapshot, replay_action_log, restore_snapshot,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
pub use simulation::{
    Action, PASSIVE_PIEZO_HISTORY_STEPS, STEP_SECONDS, SimulationDiagnostic, SimulationState,
    advance_steps, apply_actions, passive_piezo_is_sounding,
};
pub use solver::{
    ElectricalDiagnostic, ElectricalError, MAX_NONLINEAR_ITERATIONS, NodeVoltage, SolveResult,
    solve_dc, solve_transient,
};
use std::collections::{BTreeMap, BTreeSet};

/// Current threshold shared by current-driven sounding presentations.
pub const SOUNDING_CURRENT: f64 = 0.001;

/// Version of the core crate used by applications and workspace tools.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PROJECT_FORMAT_VERSION: u32 = 1;
pub const MAX_COMPONENTS: usize = 64;
pub const MAX_NODES: usize = 128;
pub const MAX_WIRES: usize = 256;
pub const MAX_BOARDS: usize = 8;
pub const DEFAULT_BOARD_ID: &str = "main";

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
string_id!(BoardId);

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
    /// Optional learner-facing fault metadata. Repairs are structural edits
    /// applied through `Action::RepairFault`; the project remains the single
    /// electrical source of truth.
    #[serde(default)]
    pub faults: Vec<FaultSpec>,
    #[serde(default)]
    pub initial_conditions: InitialConditions,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct FaultSpec {
    pub id: String,
    pub description: String,
    pub repairs: Vec<FaultRepair>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "repair", rename_all = "snake_case")]
pub enum FaultRepair {
    SwapPins {
        component: ComponentId,
        first: PinId,
        second: PinId,
    },
    SetPin {
        component: ComponentId,
        pin: PinId,
        hole: HoleId,
    },
    SetParameter {
        component: ComponentId,
        name: String,
        value: f64,
    },
    AddWire {
        wire: Wire,
    },
    RemoveWire {
        wire: WireId,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct InitialConditions {
    #[serde(default)]
    pub capacitor_voltages: BTreeMap<ComponentId, f64>,
    #[serde(default)]
    pub controls: BTreeMap<ComponentId, ControlState>,
    /// Continuous 0.0..=1.0 control ratio for a variable resistor, including
    /// external touch pads and water probes. Analogous to `controls`, but a
    /// continuous ratio instead of a discrete `ControlState`.
    #[serde(default)]
    pub control_ratios: BTreeMap<ComponentId, f64>,
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
    /// A named collection of independent solderless boards. Holes in this
    /// model must use `board_id/local_hole` references; only explicit wire
    /// endpoints may connect two boards.
    MultiBoard {
        boards: Vec<BoardSpec>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BoardSpec {
    pub id: BoardId,
    pub model: BoardSurfaceModel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BoardSurfaceModel {
    HalfSizeSolderless,
}

impl BoardModel {
    /// Return the stable board identities exposed by this project model.
    pub fn board_ids(&self) -> Vec<BoardId> {
        match self {
            Self::HalfSizeSolderless => vec![BoardId(DEFAULT_BOARD_ID.into())],
            Self::MultiBoard { boards } => boards.iter().map(|board| board.id.clone()).collect(),
        }
    }

    pub fn board_count(&self) -> usize {
        self.board_ids().len()
    }

    /// Qualify a local hole for this model. The single-board legacy contract
    /// intentionally keeps its original unqualified JSON spelling.
    pub fn qualified_hole(&self, board_id: &BoardId, local: &HoleId) -> Option<HoleId> {
        hole_group(&local.0)?;
        match self {
            Self::HalfSizeSolderless if board_id.0 == DEFAULT_BOARD_ID => Some(local.clone()),
            Self::HalfSizeSolderless => None,
            Self::MultiBoard { boards } if boards.iter().any(|board| board.id == *board_id) => {
                Some(HoleId(format!("{}/{}", board_id.0, local.0)))
            }
            Self::MultiBoard { .. } => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Component {
    pub id: ComponentId,
    pub kind: ComponentKind,
    pub pins: BTreeMap<PinId, HoleId>,
    pub parameters: BTreeMap<String, f64>,
    /// Required only for `ComponentKind::IcDevice`. The component pin map
    /// remains the sole source of physical connectivity.
    #[serde(default)]
    pub ic_device: Option<IcDeviceSpec>,
    /// Required only for `ComponentKind::Other`. The named-pin contract is
    /// the sole source of the model's electrical behavior.
    #[serde(default)]
    pub other_device: Option<OtherDeviceSpec>,
    /// Required only for `ComponentKind::Module`. The named-pin contract is
    /// the sole source of the module's calculated electrical behavior.
    #[serde(default)]
    pub module: Option<ModuleSpec>,
    /// Optional diode family and reverse-breakdown contract. Omitting it on a
    /// diode preserves the original standard forward-diode JSON contract.
    #[serde(default)]
    pub diode_model: Option<DiodeSpec>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ComponentKind {
    DcVoltageSource,
    Resistor,
    Led,
    /// Two-terminal rectifier diode with a smooth forward characteristic.
    Diode,
    Capacitor,
    NpnTransistor,
    PnpTransistor,
    MomentaryButton,
    ChangeoverSwitch,
    /// Two-terminal variable resistor (rheostat wiring only; the wiper
    /// terminal is not modeled). Resistance is driven by a continuous
    /// control ratio (see `InitialConditions::control_ratios`), not by a
    /// project parameter: `resistance = min_resistance + ratio *
    /// (max_resistance - min_resistance)`.
    Potentiometer,
    /// Two-terminal variable resistor whose resistance is driven by a
    /// continuous "ambient light" control ratio instead of a project
    /// parameter: `resistance = max_resistance - ratio * (max_resistance -
    /// min_resistance)`, so ratio 0.0 (darkest) is `max_resistance` and
    /// ratio 1.0 (brightest) is `min_resistance`.
    Photoresistor,
    /// Two-terminal NTC thermistor whose resistance falls as the temperature
    /// control ratio rises.
    Thermistor,
    /// Two-terminal external touch-pad resistance. Ratio 0 is an open, dry
    /// pad and ratio 1 is the configured contact resistance of a finger.
    TouchPad,
    /// Two-terminal external water-probe resistance. Ratio 0 is dry and ratio
    /// 1 is the configured conductivity of the probe medium.
    WaterProbe,
    /// Fixed-resistance two-terminal load with a current-derived "sounding"
    /// presentation state. The app plays a fixed tone while sounding; the
    /// pitch is a presentation choice, not a calculated electrical result.
    Buzzer,
    /// Fixed-resistance two-terminal load, electrically identical to
    /// `Buzzer` (see its documentation): a low-impedance dynamic speaker
    /// rather than a piezo buzzer, with its own sprite and a lower, fuller
    /// tone while sounding.
    Speaker,
    /// Fixed-resistance two-terminal passive piezo element. Unlike `Buzzer`,
    /// it has no internal oscillator: the app presents it as sounding only
    /// when its calculated fixed-step current is actually varying.
    PiezoPassive,
    /// Two-terminal DC motor. The electrical branch is resistive; the solver
    /// also derives signed shaft speed from terminal voltage using the
    /// component's rated voltage and no-load speed parameters. Inertia,
    /// torque load, and physical props are outside this core contract.
    Motor,
    /// Four-terminal optocoupler with a calculated LED input and
    /// light-controlled collector-emitter conductance.
    Optocoupler,
    /// Five-terminal relay with a resistive coil and threshold-selected
    /// common-to-NC/NO contact pair. Mechanical bounce and inductance are
    /// outside this educational contract.
    Relay,
    /// Two-input digital gate with a voltage-derived output.
    LogicGate,
    /// Voltage-threshold inverting buffer with hysteresis-free educational behavior.
    SchmittInverter,
    /// Open-loop comparator represented as a bounded voltage-output device.
    Comparator,
    /// NE555-compatible timer primitive whose output and discharge pins are
    /// driven from its threshold, trigger, reset, and supply pins.
    #[serde(rename = "timer_555")]
    Timer555,
    /// D-type flip-flop primitive. Its stateful edge contract is completed by
    /// the simulation layer; the pin contract is validated here.
    DFlipFlop,
    /// Bounded binary or one-hot digital counter advanced by rising clock edges.
    DigitalCounter,
    /// Eight-step rising-edge sequencer with calculated analog control selection.
    StepSequencer,
    /// Bounded two-word, eight-bit SRAM with calculated read and write behavior.
    Sram,
    /// Eight-bit serial-in, parallel-out shift register with a separate latch.
    ShiftRegister,
    /// Common-cathode seven-segment display represented as a calculated load.
    SevenSegmentDisplay,
    /// Four-bit combinational adder/subtractor with calculated carry output.
    FourBitAdder,
    /// Ten-segment voltage-level display with calculated threshold outputs.
    BargraphDisplay,
    /// Bounded voltage amplifier used for the LM386-style educational fixture.
    AudioAmplifier,
    /// Configurable pin-level IC/module contract. Its electrical behavior is
    /// calculated from node voltages by the common solver.
    IcDevice,
    /// Ready-made catalog module with a bounded, calculated pin-level contract.
    Module,
    /// Explicit pin-level contract for a catalog part not yet promoted to a
    /// dedicated component kind.
    Other,
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

#[derive(Clone, Debug)]
struct ResolvedHole {
    canonical: HoleId,
    local: String,
}

#[derive(Clone, Debug)]
struct BoardTopology {
    board_ids: BTreeSet<BoardId>,
    multi_board: bool,
}

impl BoardTopology {
    fn new(model: &BoardModel, errors: &mut Vec<Diagnostic>) -> Self {
        let mut board_ids = BTreeSet::new();
        match model {
            BoardModel::HalfSizeSolderless => {
                board_ids.insert(BoardId(DEFAULT_BOARD_ID.into()));
                Self {
                    board_ids,
                    multi_board: false,
                }
            }
            BoardModel::MultiBoard { boards } => {
                if boards.is_empty() || boards.len() > MAX_BOARDS {
                    errors.push(Diagnostic::new(
                        "board_limit",
                        "board.model.boards",
                        format!("a multi_board model must contain 1..={MAX_BOARDS} boards"),
                    ));
                }
                for (index, board) in boards.iter().enumerate() {
                    let path = format!("board.model.boards[{index}]");
                    if board.id.0.is_empty()
                        || board.id.0.len() > 64
                        || !board
                            .id
                            .0
                            .chars()
                            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
                    {
                        errors.push(Diagnostic::new(
                            "invalid_board_id",
                            format!("{path}.id"),
                            "board id must contain 1–64 ASCII letters, digits, '-' or '_'",
                        ));
                    }
                    if !board_ids.insert(board.id.clone()) {
                        errors.push(Diagnostic::new(
                            "duplicate_board_id",
                            format!("{path}.id"),
                            format!("duplicate board id {}", board.id.0),
                        ));
                    }
                    if !matches!(board.model, BoardSurfaceModel::HalfSizeSolderless) {
                        errors.push(Diagnostic::new(
                            "unsupported_board_model",
                            format!("{path}.model"),
                            "only half_size_solderless boards are supported",
                        ));
                    }
                }
                Self {
                    board_ids,
                    multi_board: true,
                }
            }
        }
    }

    fn resolve(&self, hole: &HoleId) -> Option<ResolvedHole> {
        if self.multi_board {
            let (board_id, local) = hole.0.split_once('/')?;
            let board_id = BoardId(board_id.to_owned());
            if !self.board_ids.contains(&board_id) || hole_group(local).is_none() {
                return None;
            }
            Some(ResolvedHole {
                canonical: hole.clone(),
                local: local.to_owned(),
            })
        } else if hole_group(&hole.0).is_some() {
            Some(ResolvedHole {
                canonical: hole.clone(),
                local: hole.0.clone(),
            })
        } else {
            None
        }
    }

    fn qualify(&self, board_id: &BoardId, local: &str) -> HoleId {
        if self.multi_board {
            HoleId(format!("{}/{}", board_id.0, local))
        } else {
            HoleId(local.to_owned())
        }
    }
}

/// Validate project references and derive nodes. Rows 1–30 have isolated A–E and F–J
/// five-hole strips; rows do not connect vertically. Rails TP+/TP- and BP+/BP- are
/// continuous from rows 1–30. In a multi-board model, the same rules apply
/// independently to each named board. Only wire endpoints connect; geometric
/// crossings do not.
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

    let board_topology = BoardTopology::new(&project.board.model, &mut errors);

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
    let valid_hole = |h: &HoleId| board_topology.resolve(h).is_some();
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
                    if board_topology.multi_board {
                        format!(
                            "unknown or unqualified board hole {}; use board_id/local_hole",
                            hole.0
                        )
                    } else {
                        format!("unknown board hole {}", hole.0)
                    },
                ));
            }
        }
        if let (Some(from), Some(to)) = (
            board_topology.resolve(&wire.from),
            board_topology.resolve(&wire.to),
        ) {
            union(&mut parent, &from.canonical, &to.canonical);
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
        if c.kind != ComponentKind::Diode && c.diode_model.is_some() {
            errors.push(Diagnostic::new(
                "unexpected_diode_model",
                format!("components.{}.diode_model", c.id.0),
                "only diode components may carry a diode model contract",
            ));
        }
        if c.kind == ComponentKind::Diode
            && let (Some(forward_voltage), Some(series_resistance)) = (
                c.parameters.get("forward_voltage"),
                c.parameters.get("series_resistance"),
            )
            && let Err(error) = DiodeModel::new(
                *forward_voltage,
                *series_resistance,
                c.diode_model.unwrap_or_default(),
            )
        {
            errors.push(Diagnostic::new(
                "invalid_diode_model",
                format!("components.{}.diode_model", c.id.0),
                format!("diode model contract is invalid: {error:?}"),
            ));
        }
        let expected: Vec<String> = if c.kind == ComponentKind::IcDevice {
            match &c.ic_device {
                Some(spec) => {
                    validate_ic_device(c, spec, &mut errors);
                    spec.pin_roles.keys().map(|pin| pin.0.clone()).collect()
                }
                None => {
                    errors.push(Diagnostic::new(
                        "missing_ic_device_spec",
                        format!("components.{}.ic_device", c.id.0),
                        "ic_device components require a pin-role and behavior contract",
                    ));
                    Vec::new()
                }
            }
        } else if c.kind == ComponentKind::Module {
            match &c.module {
                Some(spec) => {
                    validate_module(c, spec, &mut errors);
                    spec.pin_roles.keys().map(|pin| pin.0.clone()).collect()
                }
                None => {
                    errors.push(Diagnostic::new(
                        "missing_module_spec",
                        format!("components.{}.module", c.id.0),
                        "module components require a named-pin electrical contract",
                    ));
                    Vec::new()
                }
            }
        } else if c.kind == ComponentKind::Other {
            match &c.other_device {
                Some(spec) => {
                    validate_other_device(c, spec, &mut errors);
                    spec.pin_roles.keys().map(|pin| pin.0.clone()).collect()
                }
                None => {
                    errors.push(Diagnostic::new(
                        "missing_other_device_spec",
                        format!("components.{}.other_device", c.id.0),
                        "other components require a named-pin electrical contract",
                    ));
                    Vec::new()
                }
            }
        } else {
            if c.ic_device.is_some() {
                errors.push(Diagnostic::new(
                    "unexpected_ic_device_spec",
                    format!("components.{}.ic_device", c.id.0),
                    "only ic_device components may carry an ic_device contract",
                ));
            }
            if c.other_device.is_some() {
                errors.push(Diagnostic::new(
                    "unexpected_other_device_spec",
                    format!("components.{}.other_device", c.id.0),
                    "only other components may carry an other_device contract",
                ));
            }
            pins_for(c.kind)
                .iter()
                .map(|pin| (*pin).to_owned())
                .collect()
        };
        if c.pins.len() != expected.len()
            || expected
                .iter()
                .any(|p| !c.pins.contains_key(&PinId(p.clone())))
        {
            errors.push(Diagnostic::new(
                "invalid_pins",
                format!("components.{}.pins", c.id.0),
                format!("expected pins: {}", expected.join(", ")),
            ));
        }
        if c.kind == ComponentKind::IcDevice && c.other_device.is_some() {
            errors.push(Diagnostic::new(
                "unexpected_other_device_spec",
                format!("components.{}.other_device", c.id.0),
                "ic_device and other_device contracts cannot be combined",
            ));
        }
        if c.kind == ComponentKind::Module {
            if c.ic_device.is_some() || c.other_device.is_some() {
                errors.push(Diagnostic::new(
                    "unexpected_device_spec",
                    format!("components.{}.module", c.id.0),
                    "module components cannot combine module, ic_device, or other_device contracts",
                ));
            }
        } else if c.module.is_some() {
            errors.push(Diagnostic::new(
                "unexpected_module_spec",
                format!("components.{}.module", c.id.0),
                "only module components may carry a module contract",
            ));
        }
        for (pin, hole) in &c.pins {
            if !expected.iter().any(|expected_pin| expected_pin == &pin.0) {
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
                    if board_topology.multi_board {
                        format!(
                            "unknown or unqualified board hole {}; use board_id/local_hole",
                            hole.0
                        )
                    } else {
                        format!("unknown board hole {}", hole.0)
                    },
                ));
            } else {
                let canonical = board_topology
                    .resolve(hole)
                    .expect("valid hole was resolved")
                    .canonical;
                pin_contacts
                    .entry(canonical.clone())
                    .or_default()
                    .push(Contact::ComponentPin(c.id.clone(), pin.clone()));
                parent
                    .entry(canonical.clone())
                    .or_insert_with(|| canonical.clone());
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
        if c.kind == ComponentKind::NpnTransistor
            && c.parameters.contains_key("reverse_breakdown_voltage")
                != c.parameters.contains_key("reverse_breakdown_resistance")
        {
            errors.push(Diagnostic::new(
                "incomplete_reverse_breakdown_contract",
                format!("components.{}.parameters", c.id.0),
                "reverse_breakdown_voltage and reverse_breakdown_resistance must be declared together",
            ));
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
        if matches!(
            c.kind,
            ComponentKind::Potentiometer
                | ComponentKind::Photoresistor
                | ComponentKind::Thermistor
                | ComponentKind::TouchPad
                | ComponentKind::WaterProbe
        ) && let (Some(min), Some(max)) = (
            c.parameters.get("min_resistance"),
            c.parameters.get("max_resistance"),
        ) && min.is_finite()
            && max.is_finite()
            && *min >= *max
        {
            errors.push(Diagnostic::new(
                "invalid_parameter_range",
                format!("components.{}.parameters.max_resistance", c.id.0),
                "max_resistance must be greater than min_resistance",
            ));
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
    for (id, ratio) in &project.initial_conditions.control_ratios {
        match project.components.iter().find(|c| &c.id == id) {
            Some(c)
                if matches!(
                    c.kind,
                    ComponentKind::Potentiometer
                        | ComponentKind::Photoresistor
                        | ComponentKind::Thermistor
                        | ComponentKind::TouchPad
                        | ComponentKind::WaterProbe
                ) =>
            {
                if !ratio.is_finite() || !(0.0..=1.0).contains(ratio) {
                    errors.push(Diagnostic::new(
                        "control_ratio_out_of_range",
                        format!("initial_conditions.control_ratios.{}", id.0),
                        "control ratio must be finite and in 0.0..=1.0",
                    ));
                }
            }
            _ => errors.push(Diagnostic::new(
                "invalid_initial_control_ratio",
                format!("initial_conditions.control_ratios.{}", id.0),
                "control ratio must refer to a variable-resistor component",
            )),
        }
    }
    // All holes in a contact group share a node, including the four continuous rails.
    let holes: Vec<_> = parent.keys().cloned().collect();
    for i in 0..holes.len() {
        for j in (i + 1)..holes.len() {
            let same_contact_group = match (
                board_topology.resolve(&holes[i]),
                board_topology.resolve(&holes[j]),
            ) {
                (Some(first), Some(second)) => {
                    hole_group(&first.local).is_some()
                        && hole_group(&first.local) == hole_group(&second.local)
                        && holes[i]
                            .0
                            .split_once('/')
                            .map_or(DEFAULT_BOARD_ID, |(board, _)| board)
                            == holes[j]
                                .0
                                .split_once('/')
                                .map_or(DEFAULT_BOARD_ID, |(board, _)| board)
                }
                _ => false,
            };
            if same_contact_group {
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
        let resolved = board_topology.resolve(&h).expect("validated hole");
        let group = hole_group(&resolved.local).expect("validated hole");
        let board_id = BoardId(h.0.split('/').next().unwrap_or(DEFAULT_BOARD_ID).into());
        if group.starts_with("TP") || group.starts_with("BP") {
            for row in 1..=30 {
                contacts.push(Contact::Hole(
                    board_topology.qualify(&board_id, &format!("{group}:{row}")),
                ));
            }
        } else if let Some((strip, row)) = group.split_once(':')
            && strip == "strip"
        {
            let start = if row.starts_with("left") { 'A' } else { 'F' };
            let row = row.rsplit(':').next().unwrap();
            for col in start..=if start == 'A' { 'E' } else { 'J' } {
                contacts.push(Contact::Hole(
                    board_topology.qualify(&board_id, &format!("{col}{row}")),
                ));
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

fn validate_ic_device(c: &Component, spec: &IcDeviceSpec, errors: &mut Vec<Diagnostic>) {
    use IcDeviceBehavior::{Comparator, Linear, Logic};
    use IcDevicePinRole::{Ground, Input, Output, Reference, Supply};

    if spec.pin_roles.is_empty() || spec.pin_roles.len() > MAX_IC_DEVICE_PINS {
        errors.push(Diagnostic::new(
            "ic_device_pin_limit",
            format!("components.{}.ic_device.pin_roles", c.id.0),
            format!("an ic_device must declare 1..={MAX_IC_DEVICE_PINS} named pins"),
        ));
    }
    let supply_count = spec
        .pin_roles
        .values()
        .filter(|role| matches!(role, Supply))
        .count();
    let ground_count = spec
        .pin_roles
        .values()
        .filter(|role| matches!(role, Ground))
        .count();
    if supply_count != 1 || ground_count != 1 {
        errors.push(Diagnostic::new(
            "ic_device_supply_contract",
            format!("components.{}.ic_device.pin_roles", c.id.0),
            "an ic_device must declare exactly one supply and one ground pin",
        ));
    }
    for pin in spec.referenced_pins() {
        if !spec.pin_roles.contains_key(pin) || !c.pins.contains_key(pin) {
            errors.push(Diagnostic::new(
                "ic_device_unknown_pin",
                format!("components.{}.ic_device.behavior", c.id.0),
                format!("behavior references undeclared pin {}", pin.0),
            ));
        }
    }
    let role_is = |pin: &PinId, expected: fn(&IcDevicePinRole) -> bool| {
        spec.pin_roles.get(pin).is_some_and(expected)
    };
    match &spec.behavior {
        Linear {
            output,
            reference,
            inputs,
            offset,
            min_output,
            max_output,
            input_resistance,
            output_resistance,
        } => {
            if !role_is(output, |role| matches!(role, Output)) {
                errors.push(Diagnostic::new(
                    "ic_device_output_pin_role",
                    format!("components.{}.ic_device.behavior.output", c.id.0),
                    "linear output must have the output pin role",
                ));
            }
            if !role_is(reference, |role| matches!(role, Ground | Reference)) {
                errors.push(Diagnostic::new(
                    "ic_device_reference_pin_role",
                    format!("components.{}.ic_device.behavior.reference", c.id.0),
                    "reference must have the ground or reference pin role",
                ));
            }
            if inputs.is_empty() || inputs.len() > MAX_IC_DEVICE_INPUTS {
                errors.push(Diagnostic::new(
                    "ic_device_input_limit",
                    format!("components.{}.ic_device.behavior.inputs", c.id.0),
                    format!("a linear device must declare 1..={MAX_IC_DEVICE_INPUTS} inputs"),
                ));
            }
            for (index, input) in inputs.iter().enumerate() {
                if !role_is(&input.pin, |role| matches!(role, Input | Reference)) {
                    errors.push(Diagnostic::new(
                        "ic_device_input_pin_role",
                        format!(
                            "components.{}.ic_device.behavior.inputs[{index}].pin",
                            c.id.0
                        ),
                        "linear inputs must have the input or reference pin role",
                    ));
                }
                validate_ic_device_finite(c, errors, &format!("inputs[{index}].gain"), input.gain);
            }
            validate_ic_device_finite(c, errors, "offset", *offset);
            validate_ic_device_finite(c, errors, "min_output", *min_output);
            validate_ic_device_finite(c, errors, "max_output", *max_output);
            if min_output > max_output {
                errors.push(Diagnostic::new(
                    "ic_device_output_range",
                    format!("components.{}.ic_device.behavior", c.id.0),
                    "min_output must not exceed max_output",
                ));
            }
            validate_ic_device_resistance(c, errors, "input_resistance", *input_resistance);
            validate_ic_device_resistance(c, errors, "output_resistance", *output_resistance);
        }
        Comparator {
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
            for (name, pin) in [("positive", positive), ("negative", negative)] {
                if !role_is(pin, |role| matches!(role, Input | Reference)) {
                    errors.push(Diagnostic::new(
                        "ic_device_input_pin_role",
                        format!("components.{}.ic_device.behavior.{name}", c.id.0),
                        "comparator inputs must have the input or reference pin role",
                    ));
                }
            }
            if !role_is(output, |role| matches!(role, Output))
                || !role_is(reference, |role| matches!(role, Ground | Reference))
            {
                errors.push(Diagnostic::new(
                    "ic_device_pin_role",
                    format!("components.{}.ic_device.behavior", c.id.0),
                    "comparator output/reference roles do not match the contract",
                ));
            }
            validate_ic_device_finite(c, errors, "threshold", *threshold);
            validate_ic_device_finite(c, errors, "high_output", *high_output);
            validate_ic_device_finite(c, errors, "low_output", *low_output);
            validate_ic_device_resistance(c, errors, "input_resistance", *input_resistance);
            validate_ic_device_resistance(c, errors, "output_resistance", *output_resistance);
        }
        Logic {
            inputs,
            output,
            reference,
            supply,
            output_resistance,
            ..
        } => {
            if inputs.is_empty() || inputs.len() > MAX_IC_DEVICE_INPUTS {
                errors.push(Diagnostic::new(
                    "ic_device_input_limit",
                    format!("components.{}.ic_device.behavior.inputs", c.id.0),
                    format!("a logic device must declare 1..={MAX_IC_DEVICE_INPUTS} inputs"),
                ));
            }
            for pin in inputs {
                if !role_is(pin, |role| matches!(role, Input | Reference)) {
                    errors.push(Diagnostic::new(
                        "ic_device_input_pin_role",
                        format!("components.{}.ic_device.behavior.inputs", c.id.0),
                        "logic inputs must have the input or reference pin role",
                    ));
                }
            }
            if !role_is(output, |role| matches!(role, Output))
                || !role_is(reference, |role| matches!(role, Ground | Reference))
                || !role_is(supply, |role| matches!(role, Supply))
            {
                errors.push(Diagnostic::new(
                    "ic_device_pin_role",
                    format!("components.{}.ic_device.behavior", c.id.0),
                    "logic output, reference, or supply role does not match the contract",
                ));
            }
            validate_ic_device_resistance(c, errors, "output_resistance", *output_resistance);
        }
    }
}

fn validate_ic_device_finite(c: &Component, errors: &mut Vec<Diagnostic>, path: &str, value: f64) {
    if !value.is_finite() {
        errors.push(Diagnostic::new(
            "ic_device_non_finite_parameter",
            format!("components.{}.ic_device.behavior.{path}", c.id.0),
            "device behavior values must be finite",
        ));
    }
}

fn validate_ic_device_resistance(
    c: &Component,
    errors: &mut Vec<Diagnostic>,
    path: &str,
    value: f64,
) {
    if !value.is_finite() || !(IC_DEVICE_MIN_RESISTANCE..=IC_DEVICE_MAX_RESISTANCE).contains(&value)
    {
        errors.push(Diagnostic::new(
            "ic_device_resistance_out_of_range",
            format!("components.{}.ic_device.behavior.{path}", c.id.0),
            format!(
                "resistance must be finite and in {IC_DEVICE_MIN_RESISTANCE}..={IC_DEVICE_MAX_RESISTANCE} ohms"
            ),
        ));
    }
}

fn validate_module(c: &Component, spec: &ModuleSpec, errors: &mut Vec<Diagnostic>) {
    use ModuleBehavior::{
        AdjustableRegulatedSupply, AnalogTransfer, OpenCollector, RegulatedSupply, ThresholdOutput,
    };
    use ModulePinRole::{Ground, Input, Output, PowerOutput, Reference, Supply};

    if spec.pin_roles.len() < 2 || spec.pin_roles.len() > MAX_MODULE_PINS {
        errors.push(Diagnostic::new(
            "module_pin_limit",
            format!("components.{}.module.pin_roles", c.id.0),
            format!("a module must declare 2..={MAX_MODULE_PINS} named pins"),
        ));
    }
    let supply_count = spec
        .pin_roles
        .values()
        .filter(|role| matches!(role, Supply))
        .count();
    let ground_count = spec
        .pin_roles
        .values()
        .filter(|role| matches!(role, Ground))
        .count();
    if supply_count != 1 || ground_count != 1 {
        errors.push(Diagnostic::new(
            "module_supply_contract",
            format!("components.{}.module.pin_roles", c.id.0),
            "a module must declare exactly one supply and one ground pin",
        ));
    }
    for pin in spec.referenced_pins() {
        if !spec.pin_roles.contains_key(pin) || !c.pins.contains_key(pin) {
            errors.push(Diagnostic::new(
                "module_unknown_pin",
                format!("components.{}.module.behavior", c.id.0),
                format!("behavior references undeclared pin {}", pin.0),
            ));
        }
    }
    let role_is = |pin: &PinId, expected: fn(&ModulePinRole) -> bool| {
        spec.pin_roles.get(pin).is_some_and(expected)
    };
    match &spec.behavior {
        AnalogTransfer {
            output,
            reference,
            inputs,
            offset,
            min_output,
            max_output,
            input_resistance,
            output_resistance,
        } => {
            if !role_is(output, |role| matches!(role, Output | PowerOutput))
                || !role_is(reference, |role| matches!(role, Reference | Ground))
            {
                errors.push(Diagnostic::new(
                    "module_transfer_roles",
                    format!("components.{}.module.behavior", c.id.0),
                    "analog transfer output/reference roles do not match the contract",
                ));
            }
            if inputs.is_empty() || inputs.len() > MAX_MODULE_INPUTS {
                errors.push(Diagnostic::new(
                    "module_input_limit",
                    format!("components.{}.module.behavior.inputs", c.id.0),
                    format!("a module transfer must declare 1..={MAX_MODULE_INPUTS} inputs"),
                ));
            }
            for (index, input) in inputs.iter().enumerate() {
                if !role_is(&input.pin, |role| matches!(role, Input | Reference)) {
                    errors.push(Diagnostic::new(
                        "module_input_role",
                        format!("components.{}.module.behavior.inputs[{index}]", c.id.0),
                        "module inputs must have input or reference roles",
                    ));
                }
                validate_module_finite(c, errors, &format!("inputs[{index}].gain"), input.gain);
            }
            validate_module_finite(c, errors, "offset", *offset);
            validate_module_finite(c, errors, "min_output", *min_output);
            validate_module_finite(c, errors, "max_output", *max_output);
            if min_output > max_output {
                errors.push(Diagnostic::new(
                    "module_output_range",
                    format!("components.{}.module.behavior", c.id.0),
                    "min_output must not exceed max_output",
                ));
            }
            validate_module_resistance(c, errors, "input_resistance", *input_resistance);
            validate_module_resistance(c, errors, "output_resistance", *output_resistance);
        }
        ThresholdOutput {
            input,
            reference,
            output,
            threshold,
            high_output,
            low_output,
            input_resistance,
            output_resistance,
        } => {
            if !role_is(input, |role| matches!(role, Input))
                || !role_is(reference, |role| matches!(role, Reference | Ground))
                || !role_is(output, |role| matches!(role, Output))
            {
                errors.push(Diagnostic::new(
                    "module_threshold_roles",
                    format!("components.{}.module.behavior", c.id.0),
                    "threshold input/reference/output roles do not match the contract",
                ));
            }
            for (name, value) in [
                ("threshold", *threshold),
                ("high_output", *high_output),
                ("low_output", *low_output),
            ] {
                validate_module_finite(c, errors, name, value);
            }
            validate_module_resistance(c, errors, "input_resistance", *input_resistance);
            validate_module_resistance(c, errors, "output_resistance", *output_resistance);
        }
        OpenCollector {
            supply,
            ground,
            channels,
            input_resistance,
            on_resistance,
            off_resistance,
        } => {
            if !role_is(supply, |role| matches!(role, Supply))
                || !role_is(ground, |role| matches!(role, Ground))
            {
                errors.push(Diagnostic::new(
                    "module_driver_supply_roles",
                    format!("components.{}.module.behavior", c.id.0),
                    "open-collector supply/ground roles do not match the contract",
                ));
            }
            if channels.is_empty() || channels.len() > MAX_MODULE_CHANNELS {
                errors.push(Diagnostic::new(
                    "module_channel_limit",
                    format!("components.{}.module.behavior.channels", c.id.0),
                    format!("a module driver must declare 1..={MAX_MODULE_CHANNELS} channels"),
                ));
            }
            for (index, channel) in channels.iter().enumerate() {
                if !role_is(&channel.input, |role| matches!(role, Input))
                    || !role_is(&channel.output, |role| matches!(role, Output))
                {
                    errors.push(Diagnostic::new(
                        "module_channel_role",
                        format!("components.{}.module.behavior.channels[{index}]", c.id.0),
                        "driver channel input/output roles do not match the contract",
                    ));
                }
            }
            validate_module_resistance(c, errors, "input_resistance", *input_resistance);
            validate_module_resistance(c, errors, "on_resistance", *on_resistance);
            validate_module_resistance(c, errors, "off_resistance", *off_resistance);
        }
        RegulatedSupply {
            input_positive,
            input_negative,
            output_positive,
            output_negative,
            target_voltage,
            dropout_voltage,
            input_resistance,
            output_resistance,
        } => {
            if !role_is(input_positive, |role| matches!(role, Supply))
                || !role_is(input_negative, |role| matches!(role, Ground | Reference))
                || !role_is(output_positive, |role| matches!(role, Output | PowerOutput))
                || !role_is(output_negative, |role| matches!(role, Ground | Reference))
            {
                errors.push(Diagnostic::new(
                    "module_regulator_roles",
                    format!("components.{}.module.behavior", c.id.0),
                    "regulated-supply pin roles do not match the contract",
                ));
            }
            validate_module_finite(c, errors, "target_voltage", *target_voltage);
            validate_module_finite(c, errors, "dropout_voltage", *dropout_voltage);
            if *target_voltage < 0.0 || *dropout_voltage < 0.0 {
                errors.push(Diagnostic::new(
                    "module_regulator_range",
                    format!("components.{}.module.behavior", c.id.0),
                    "target_voltage and dropout_voltage must be non-negative",
                ));
            }
            validate_module_resistance(c, errors, "input_resistance", *input_resistance);
            validate_module_resistance(c, errors, "output_resistance", *output_resistance);
        }
        AdjustableRegulatedSupply {
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
            if !role_is(input_positive, |role| matches!(role, Supply))
                || !role_is(input_negative, |role| matches!(role, Ground | Reference))
                || !role_is(output_positive, |role| matches!(role, Output | PowerOutput))
                || !role_is(output_negative, |role| matches!(role, Ground | Reference))
                || !role_is(adjust, |role| matches!(role, Input | Reference))
            {
                errors.push(Diagnostic::new(
                    "module_adjustable_regulator_roles",
                    format!("components.{}.module.behavior", c.id.0),
                    "adjustable-regulator pin roles do not match the contract",
                ));
            }
            for (name, value) in [
                ("reference_voltage", *reference_voltage),
                ("min_output_voltage", *min_output_voltage),
                ("max_output_voltage", *max_output_voltage),
                ("dropout_voltage", *dropout_voltage),
            ] {
                validate_module_finite(c, errors, name, value);
            }
            if *reference_voltage < 0.0
                || *min_output_voltage < 0.0
                || *max_output_voltage < 0.0
                || *dropout_voltage < 0.0
            {
                errors.push(Diagnostic::new(
                    "module_adjustable_regulator_range",
                    format!("components.{}.module.behavior", c.id.0),
                    "reference, output, and dropout voltages must be non-negative",
                ));
            }
            if min_output_voltage > max_output_voltage {
                errors.push(Diagnostic::new(
                    "module_adjustable_regulator_output_range",
                    format!("components.{}.module.behavior", c.id.0),
                    "min_output_voltage must not exceed max_output_voltage",
                ));
            }
            validate_module_resistance(c, errors, "input_resistance", *input_resistance);
            validate_module_resistance(c, errors, "output_resistance", *output_resistance);
        }
    }
}

fn validate_module_finite(c: &Component, errors: &mut Vec<Diagnostic>, path: &str, value: f64) {
    if !value.is_finite() {
        errors.push(Diagnostic::new(
            "module_non_finite_value",
            format!("components.{}.module.behavior.{path}", c.id.0),
            "module behavior values must be finite",
        ));
    }
}

fn validate_module_resistance(c: &Component, errors: &mut Vec<Diagnostic>, path: &str, value: f64) {
    validate_module_finite(c, errors, path, value);
    if !value.is_finite() || !(MODULE_MIN_RESISTANCE..=MODULE_MAX_RESISTANCE).contains(&value) {
        errors.push(Diagnostic::new(
            "module_resistance_range",
            format!("components.{}.module.behavior.{path}", c.id.0),
            format!(
                "resistance must be finite and in {MODULE_MIN_RESISTANCE}..={MODULE_MAX_RESISTANCE} ohms"
            ),
        ));
    }
}

fn validate_other_device(c: &Component, spec: &OtherDeviceSpec, errors: &mut Vec<Diagnostic>) {
    use OtherDeviceBehavior::{
        BjtTestSocket as BjtTestSocketBehavior, LinearTransfer, Resistive, RingModulator,
        Transformer, VoltageControlledResistance, VoltageSource,
    };
    use OtherDevicePinRole::{Control, Ground, Input, Output, Reference, Supply, Terminal};

    if spec.pin_roles.len() < 2 || spec.pin_roles.len() > MAX_OTHER_DEVICE_PINS {
        errors.push(Diagnostic::new(
            "other_device_pin_limit",
            format!("components.{}.other_device.pin_roles", c.id.0),
            format!("an other device must declare 2..={MAX_OTHER_DEVICE_PINS} named pins"),
        ));
    }
    for pin in spec.pin_roles.keys() {
        if pin.0.trim().is_empty() || pin.0.len() > 32 {
            errors.push(Diagnostic::new(
                "other_device_pin_name",
                format!("components.{}.other_device.pin_roles", c.id.0),
                "other device pin names must contain 1–32 characters",
            ));
        }
    }
    for pin in spec.referenced_pins() {
        if !spec.pin_roles.contains_key(pin) || !c.pins.contains_key(pin) {
            errors.push(Diagnostic::new(
                "other_device_unknown_pin",
                format!("components.{}.other_device.behavior", c.id.0),
                format!("behavior references undeclared pin {}", pin.0),
            ));
        }
    }
    let role_is = |pin: &PinId, expected: fn(&OtherDevicePinRole) -> bool| {
        spec.pin_roles.get(pin).is_some_and(expected)
    };
    match &spec.behavior {
        Resistive {
            positive,
            negative,
            resistance: value,
        } => {
            if !role_is(positive, |role| matches!(role, Terminal | Output))
                || !role_is(negative, |role| {
                    matches!(role, Terminal | Reference | Ground)
                })
            {
                errors.push(Diagnostic::new(
                    "other_device_terminal_roles",
                    format!("components.{}.other_device.behavior", c.id.0),
                    "resistive terminals must have terminal/output and terminal/reference/ground roles",
                ));
            }
            validate_other_resistance(c, errors, "resistance", *value);
        }
        VoltageSource {
            positive,
            negative,
            voltage,
            internal_resistance,
        } => {
            if !role_is(positive, |role| matches!(role, Terminal | Output | Supply))
                || !role_is(negative, |role| {
                    matches!(role, Terminal | Reference | Ground)
                })
            {
                errors.push(Diagnostic::new(
                    "other_device_source_roles",
                    format!("components.{}.other_device.behavior", c.id.0),
                    "source terminals must have positive/output/supply and negative/reference/ground roles",
                ));
            }
            validate_other_finite(c, errors, "voltage", *voltage);
            validate_other_resistance(c, errors, "internal_resistance", *internal_resistance);
        }
        BjtTestSocketBehavior { socket } => {
            if !role_is(&socket.base, |role| matches!(role, Input | Terminal))
                || !role_is(&socket.collector, |role| matches!(role, Output | Terminal))
                || !role_is(&socket.emitter, |role| {
                    matches!(role, Terminal | Reference | Ground | Supply)
                })
            {
                errors.push(Diagnostic::new(
                    "other_device_bjt_socket_roles",
                    format!("components.{}.other_device.behavior", c.id.0),
                    "BJT test sockets require base input, collector output, and emitter reference/ground roles",
                ));
            }
            for (path, value) in [
                ("beta", socket.beta),
                ("saturation_current", socket.saturation_current),
                ("open_resistance", socket.open_resistance),
                ("short_resistance", socket.short_resistance),
            ] {
                validate_other_finite(c, errors, path, value);
            }
            if !socket.beta.is_finite() || !(10.0..=1000.0).contains(&socket.beta) {
                errors.push(Diagnostic::new(
                    "other_device_bjt_beta_range",
                    format!("components.{}.other_device.behavior.beta", c.id.0),
                    "BJT beta must be in 10..=1000",
                ));
            }
            if !socket.saturation_current.is_finite()
                || !(1e-16..=1e-12).contains(&socket.saturation_current)
            {
                errors.push(Diagnostic::new(
                    "other_device_bjt_saturation_range",
                    format!(
                        "components.{}.other_device.behavior.saturation_current",
                        c.id.0
                    ),
                    "BJT saturation_current must be in 1e-16..=1e-12 A",
                ));
            }
            validate_other_resistance(c, errors, "open_resistance", socket.open_resistance);
            validate_other_resistance(c, errors, "short_resistance", socket.short_resistance);
            if socket.short_resistance >= socket.open_resistance {
                errors.push(Diagnostic::new(
                    "other_device_bjt_failure_range",
                    format!("components.{}.other_device.behavior", c.id.0),
                    "short_resistance must be below open_resistance",
                ));
            }
        }
        LinearTransfer {
            output,
            reference,
            inputs,
            offset,
            min_output,
            max_output,
            input_resistance,
            output_resistance,
        } => {
            if !role_is(output, |role| matches!(role, Output))
                || !role_is(reference, |role| matches!(role, Reference | Ground))
            {
                errors.push(Diagnostic::new(
                    "other_device_transfer_roles",
                    format!("components.{}.other_device.behavior", c.id.0),
                    "linear transfer output/reference roles do not match the contract",
                ));
            }
            if inputs.is_empty() || inputs.len() > MAX_OTHER_DEVICE_PINS {
                errors.push(Diagnostic::new(
                    "other_device_input_limit",
                    format!("components.{}.other_device.behavior.inputs", c.id.0),
                    format!("a linear transfer must declare 1..={MAX_OTHER_DEVICE_PINS} inputs"),
                ));
            }
            for (index, input) in inputs.iter().enumerate() {
                if !role_is(&input.pin, |role| {
                    matches!(role, Input | Control | Reference)
                }) {
                    errors.push(Diagnostic::new(
                        "other_device_input_role",
                        format!(
                            "components.{}.other_device.behavior.inputs[{index}]",
                            c.id.0
                        ),
                        "linear transfer inputs must have input/control/reference roles",
                    ));
                }
                validate_other_finite(c, errors, &format!("inputs[{index}].gain"), input.gain);
            }
            validate_other_finite(c, errors, "offset", *offset);
            validate_other_finite(c, errors, "min_output", *min_output);
            validate_other_finite(c, errors, "max_output", *max_output);
            if min_output > max_output {
                errors.push(Diagnostic::new(
                    "other_device_output_range",
                    format!("components.{}.other_device.behavior", c.id.0),
                    "min_output must not exceed max_output",
                ));
            }
            validate_other_resistance(c, errors, "input_resistance", *input_resistance);
            validate_other_resistance(c, errors, "output_resistance", *output_resistance);
        }
        Transformer {
            primary_positive,
            primary_negative,
            secondary_positive,
            secondary_negative,
            turns_ratio,
            primary_resistance,
            secondary_resistance,
        } => {
            if !role_is(primary_positive, |role| matches!(role, Input | Terminal))
                || !role_is(primary_negative, |role| {
                    matches!(role, Terminal | Reference | Ground)
                })
                || !role_is(secondary_positive, |role| matches!(role, Output | Terminal))
                || !role_is(secondary_negative, |role| {
                    matches!(role, Terminal | Reference | Ground)
                })
            {
                errors.push(Diagnostic::new(
                    "other_device_transformer_roles",
                    format!("components.{}.other_device.behavior", c.id.0),
                    "transformer pins must declare primary input and secondary output roles",
                ));
            }
            validate_other_finite(c, errors, "turns_ratio", *turns_ratio);
            if !turns_ratio.is_finite() || !(0.1..=10.0).contains(turns_ratio) {
                errors.push(Diagnostic::new(
                    "other_device_transformer_ratio",
                    format!("components.{}.other_device.behavior.turns_ratio", c.id.0),
                    "transformer turns_ratio must be in 0.1..=10",
                ));
            }
            validate_other_resistance(c, errors, "primary_resistance", *primary_resistance);
            validate_other_resistance(c, errors, "secondary_resistance", *secondary_resistance);
        }
        RingModulator {
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
            if !role_is(signal, |role| matches!(role, Input))
                || !role_is(carrier, |role| matches!(role, Input))
                || !role_is(output, |role| matches!(role, Output))
                || !role_is(reference, |role| matches!(role, Reference | Ground))
            {
                errors.push(Diagnostic::new(
                    "other_device_ring_modulator_roles",
                    format!("components.{}.other_device.behavior", c.id.0),
                    "ring modulator requires signal/carrier inputs, an output, and a reference",
                ));
            }
            for (path, value) in [
                ("gain", *gain),
                ("signal_scale", *signal_scale),
                ("carrier_scale", *carrier_scale),
                ("min_output", *min_output),
                ("max_output", *max_output),
            ] {
                validate_other_finite(c, errors, path, value);
            }
            if !signal_scale.is_finite() || *signal_scale <= 0.0 {
                errors.push(Diagnostic::new(
                    "other_device_ring_modulator_scale",
                    format!("components.{}.other_device.behavior.signal_scale", c.id.0),
                    "signal_scale must be positive",
                ));
            }
            if !carrier_scale.is_finite() || *carrier_scale <= 0.0 {
                errors.push(Diagnostic::new(
                    "other_device_ring_modulator_scale",
                    format!("components.{}.other_device.behavior.carrier_scale", c.id.0),
                    "carrier_scale must be positive",
                ));
            }
            if min_output > max_output {
                errors.push(Diagnostic::new(
                    "other_device_output_range",
                    format!("components.{}.other_device.behavior", c.id.0),
                    "min_output must not exceed max_output",
                ));
            }
            validate_other_resistance(c, errors, "input_resistance", *input_resistance);
            validate_other_resistance(c, errors, "output_resistance", *output_resistance);
        }
        VoltageControlledResistance {
            control_positive,
            control_negative,
            output_positive,
            output_negative,
            min_resistance,
            max_resistance,
            control_min,
            control_max,
        } => {
            if !role_is(control_positive, |role| matches!(role, Control | Input))
                || !role_is(control_negative, |role| {
                    matches!(role, Control | Reference | Ground)
                })
                || !role_is(output_positive, |role| matches!(role, Output | Terminal))
                || !role_is(output_negative, |role| {
                    matches!(role, Output | Terminal | Reference | Ground)
                })
            {
                errors.push(Diagnostic::new(
                    "other_device_controlled_resistance_roles",
                    format!("components.{}.other_device.behavior", c.id.0),
                    "controlled-resistance pin roles do not match the contract",
                ));
            }
            validate_other_resistance(c, errors, "min_resistance", *min_resistance);
            validate_other_resistance(c, errors, "max_resistance", *max_resistance);
            validate_other_finite(c, errors, "control_min", *control_min);
            validate_other_finite(c, errors, "control_max", *control_max);
            if min_resistance > max_resistance || control_min >= control_max {
                errors.push(Diagnostic::new(
                    "other_device_control_range",
                    format!("components.{}.other_device.behavior", c.id.0),
                    "min_resistance must not exceed max_resistance and control_min must be below control_max",
                ));
            }
        }
    }
}

fn validate_other_finite(c: &Component, errors: &mut Vec<Diagnostic>, path: &str, value: f64) {
    if !value.is_finite() {
        errors.push(Diagnostic::new(
            "other_device_non_finite_value",
            format!("components.{}.other_device.behavior.{path}", c.id.0),
            "other device behavior values must be finite",
        ));
    }
}

fn validate_other_resistance(c: &Component, errors: &mut Vec<Diagnostic>, path: &str, value: f64) {
    validate_other_finite(c, errors, path, value);
    if !value.is_finite()
        || !(OTHER_DEVICE_MIN_RESISTANCE..=OTHER_DEVICE_MAX_RESISTANCE).contains(&value)
    {
        errors.push(Diagnostic::new(
            "other_device_resistance_range",
            format!("components.{}.other_device.behavior.{path}", c.id.0),
            format!("resistance must be in {OTHER_DEVICE_MIN_RESISTANCE}..={OTHER_DEVICE_MAX_RESISTANCE}"),
        ));
    }
}

fn pins_for(k: ComponentKind) -> &'static [&'static str] {
    match k {
        ComponentKind::DcVoltageSource => &["negative", "positive"],
        ComponentKind::Resistor => &["a", "b"],
        ComponentKind::Led | ComponentKind::Diode => &["anode", "cathode"],
        ComponentKind::Capacitor => &["negative", "positive"],
        ComponentKind::NpnTransistor | ComponentKind::PnpTransistor => {
            &["base", "collector", "emitter"]
        }
        ComponentKind::MomentaryButton => &["a", "b"],
        ComponentKind::ChangeoverSwitch => &["common", "normally_closed", "normally_open"],
        ComponentKind::Potentiometer
        | ComponentKind::Photoresistor
        | ComponentKind::Thermistor
        | ComponentKind::TouchPad
        | ComponentKind::WaterProbe => &["a", "b"],
        ComponentKind::Buzzer
        | ComponentKind::Speaker
        | ComponentKind::PiezoPassive
        | ComponentKind::Motor => &["positive", "negative"],
        ComponentKind::Optocoupler => &["input_anode", "input_cathode", "collector", "emitter"],
        ComponentKind::Relay => &[
            "coil_positive",
            "coil_negative",
            "common",
            "normally_closed",
            "normally_open",
        ],
        ComponentKind::LogicGate => &["gnd", "input_a", "input_b", "output", "vcc"],
        ComponentKind::SchmittInverter => &["gnd", "input", "output", "vcc"],
        ComponentKind::Comparator => &["gnd", "inverting", "non_inverting", "output", "vcc"],
        ComponentKind::Timer555 => &[
            "control",
            "discharge",
            "gnd",
            "output",
            "reset",
            "threshold",
            "trigger",
            "vcc",
        ],
        ComponentKind::DFlipFlop => &["clock", "data", "gnd", "not_q", "q", "reset", "set", "vcc"],
        ComponentKind::DigitalCounter => &[
            "clock", "carry", "enable", "gnd", "q0", "q1", "q2", "q3", "q4", "q5", "q6", "q7",
            "q8", "q9", "reset", "vcc",
        ],
        ComponentKind::StepSequencer => &[
            "clock", "control0", "control1", "control2", "control3", "control4", "control5",
            "control6", "control7", "gnd", "output", "step0", "step1", "step2", "step3", "step4",
            "step5", "step6", "step7", "vcc",
        ],
        ComponentKind::Sram => &[
            "address", "clock", "data0", "data1", "data2", "data3", "data4", "data5", "data6",
            "data7", "gnd", "output0", "output1", "output2", "output3", "output4", "output5",
            "output6", "output7", "reset", "vcc", "write",
        ],
        ComponentKind::ShiftRegister => &[
            "clear", "clock", "data", "gnd", "latch", "q0", "q1", "q2", "q3", "q4", "q5", "q6",
            "q7", "vcc",
        ],
        ComponentKind::SevenSegmentDisplay => &[
            "a", "b", "c", "common", "d", "e", "f", "g", "gnd", "input_b0", "input_b1", "input_b2",
            "input_b3", "vcc",
        ],
        ComponentKind::FourBitAdder => &[
            "a0",
            "a1",
            "a2",
            "a3",
            "b0",
            "b1",
            "b2",
            "b3",
            "carry_in",
            "carry_out",
            "gnd",
            "subtract",
            "sum0",
            "sum1",
            "sum2",
            "sum3",
            "vcc",
        ],
        ComponentKind::BargraphDisplay => &[
            "gnd", "input", "seg0", "seg1", "seg2", "seg3", "seg4", "seg5", "seg6", "seg7", "seg8",
            "seg9", "vcc",
        ],
        ComponentKind::AudioAmplifier => &["gnd", "input", "output", "vcc"],
        ComponentKind::IcDevice => &[],
        ComponentKind::Module => &[],
        ComponentKind::Other => &[],
    }
}
fn required_parameters(k: ComponentKind) -> &'static [&'static str] {
    match k {
        ComponentKind::DcVoltageSource => &["voltage"],
        ComponentKind::Resistor => &["resistance"],
        ComponentKind::Led | ComponentKind::Diode => &["forward_voltage", "series_resistance"],
        ComponentKind::Capacitor => &["capacitance"],
        ComponentKind::NpnTransistor | ComponentKind::PnpTransistor => {
            &["beta", "saturation_current"]
        }
        ComponentKind::MomentaryButton | ComponentKind::ChangeoverSwitch => &[],
        ComponentKind::Potentiometer
        | ComponentKind::Photoresistor
        | ComponentKind::Thermistor
        | ComponentKind::TouchPad
        | ComponentKind::WaterProbe => &["min_resistance", "max_resistance"],
        ComponentKind::Buzzer | ComponentKind::Speaker | ComponentKind::PiezoPassive => {
            &["resistance"]
        }
        ComponentKind::Motor => &["resistance", "rated_voltage", "no_load_speed_rpm"],
        ComponentKind::Optocoupler => &[
            "forward_voltage",
            "series_resistance",
            "transfer_gain",
            "on_resistance",
            "off_resistance",
        ],
        ComponentKind::Relay => &["coil_resistance", "pickup_voltage", "contact_resistance"],
        ComponentKind::LogicGate => &["operation", "output_resistance"],
        ComponentKind::SchmittInverter => &["output_resistance"],
        ComponentKind::Comparator => &["output_resistance"],
        ComponentKind::Timer555 => &["output_resistance", "discharge_resistance"],
        ComponentKind::DFlipFlop => &["output_resistance"],
        ComponentKind::DigitalCounter => &["modulus", "output_mode", "output_resistance"],
        ComponentKind::StepSequencer => &["output_resistance"],
        ComponentKind::Sram => &["output_resistance"],
        ComponentKind::ShiftRegister => &["output_resistance"],
        ComponentKind::SevenSegmentDisplay => &["output_resistance"],
        ComponentKind::FourBitAdder => &["output_resistance"],
        ComponentKind::BargraphDisplay => &["output_resistance"],
        ComponentKind::AudioAmplifier => &["gain", "output_resistance"],
        ComponentKind::IcDevice => &[],
        ComponentKind::Module => &[],
        ComponentKind::Other => &[],
    }
}
fn parameter_range(k: ComponentKind, p: &str) -> Option<(f64, f64)> {
    match (k, p) {
        (ComponentKind::DcVoltageSource, "voltage") => Some((0.0, 12.0)),
        (ComponentKind::Resistor, "resistance") => Some((1.0, 1e7)),
        (ComponentKind::Led | ComponentKind::Diode, "forward_voltage") => {
            Some((DIODE_MIN_FORWARD_VOLTAGE, DIODE_MAX_FORWARD_VOLTAGE))
        }
        (ComponentKind::Led | ComponentKind::Diode, "series_resistance") => {
            Some((DIODE_MIN_SERIES_RESISTANCE, DIODE_MAX_SERIES_RESISTANCE))
        }
        (ComponentKind::Capacitor, "capacitance") => Some((1e-10, 1e-2)),
        (ComponentKind::NpnTransistor | ComponentKind::PnpTransistor, "beta") => {
            Some((10.0, 1000.0))
        }
        (ComponentKind::NpnTransistor | ComponentKind::PnpTransistor, "saturation_current") => {
            Some((1e-16, 1e-12))
        }
        (ComponentKind::NpnTransistor, "reverse_breakdown_voltage") => Some((0.1, 12.0)),
        (ComponentKind::NpnTransistor, "reverse_breakdown_resistance") => Some((1.0, 1e7)),
        (
            ComponentKind::Potentiometer | ComponentKind::Photoresistor | ComponentKind::Thermistor,
            "min_resistance" | "max_resistance",
        ) => Some((1.0, 1e7)),
        (
            ComponentKind::TouchPad | ComponentKind::WaterProbe,
            "min_resistance" | "max_resistance",
        ) => Some((1.0, 1e12)),
        (ComponentKind::Buzzer, "resistance") => Some((1.0, 1e7)),
        // Dynamic speakers are low-impedance voice coils (typically 4-32
        // ohms), unlike the piezo buzzer's much wider practical range.
        (ComponentKind::Speaker, "resistance") => Some((1.0, 100.0)),
        (ComponentKind::PiezoPassive, "resistance") => Some((1.0, 1e7)),
        (ComponentKind::Motor, "resistance") => Some((1.0, 1000.0)),
        (ComponentKind::Motor, "rated_voltage") => Some((0.1, 12.0)),
        (ComponentKind::Motor, "no_load_speed_rpm") => Some((1.0, 50_000.0)),
        (ComponentKind::Optocoupler, "forward_voltage") => Some((0.0, 10.0)),
        (ComponentKind::Optocoupler, "series_resistance") => Some((1.0, 1e7)),
        (ComponentKind::Optocoupler, "transfer_gain") => Some((0.0, 1000.0)),
        (ComponentKind::Optocoupler, "on_resistance") => Some((1.0, 1e6)),
        (ComponentKind::Optocoupler, "off_resistance") => Some((1.0, 1e12)),
        (ComponentKind::Relay, "coil_resistance") => Some((1.0, 1e7)),
        (ComponentKind::Relay, "pickup_voltage") => Some((0.0, 12.0)),
        (ComponentKind::Relay, "contact_resistance") => Some((0.01, 1e6)),
        (ComponentKind::LogicGate, "operation") => Some((0.0, 3.0)),
        (
            ComponentKind::LogicGate
            | ComponentKind::SchmittInverter
            | ComponentKind::Comparator
            | ComponentKind::Timer555
            | ComponentKind::DFlipFlop,
            "output_resistance",
        ) => Some((1.0, 1e7)),
        (ComponentKind::Timer555, "discharge_resistance") => Some((1.0, 1e7)),
        (ComponentKind::DigitalCounter, "modulus") => Some((2.0, 10.0)),
        (ComponentKind::DigitalCounter, "output_mode") => Some((0.0, 2.0)),
        (ComponentKind::DigitalCounter, "output_resistance") => Some((1.0, 1e7)),
        (ComponentKind::StepSequencer, "output_resistance") => Some((1.0, 1e7)),
        (ComponentKind::Sram, "output_resistance") => Some((1.0, 1e7)),
        (ComponentKind::ShiftRegister, "output_resistance") => Some((1.0, 1e7)),
        (ComponentKind::SevenSegmentDisplay, "output_resistance") => Some((1.0, 1e7)),
        (ComponentKind::FourBitAdder, "output_resistance") => Some((1.0, 1e7)),
        (ComponentKind::BargraphDisplay, "output_resistance") => Some((1.0, 1e7)),
        (ComponentKind::AudioAmplifier, "gain") => Some((1.0, 100.0)),
        (ComponentKind::AudioAmplifier, "output_resistance") => Some((1.0, 1e7)),
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
                ic_device: None,
                other_device: None,
                module: None,
                diode_model: None,
            }],
            initial_conditions: InitialConditions::default(),
            wires: vec![Wire {
                id: WireId("W1".into()),
                from: HoleId("A2".into()),
                to: HoleId("J30".into()),
            }],
            faults: Vec::new(),
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
            ic_device: None,
            other_device: None,
            module: None,
            diode_model: None,
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

    fn multi_board_model(ids: &[&str]) -> BoardModel {
        BoardModel::MultiBoard {
            boards: ids
                .iter()
                .map(|id| BoardSpec {
                    id: BoardId((*id).into()),
                    model: BoardSurfaceModel::HalfSizeSolderless,
                })
                .collect(),
        }
    }

    #[test]
    fn multi_board_contract_requires_qualified_holes_and_keeps_strips_local() {
        let model = multi_board_model(&["logic", "display"]);
        assert_eq!(model.board_count(), 2);
        assert_eq!(
            model.qualified_hole(&BoardId("logic".into()), &HoleId("A1".into())),
            Some(HoleId("logic/A1".into()))
        );
        assert_eq!(
            model.qualified_hole(&BoardId("missing".into()), &HoleId("A1".into())),
            None
        );

        let mut p = project();
        p.board.model = model;
        p.components.clear();
        p.wires = vec![
            Wire {
                id: WireId("same-board".into()),
                from: HoleId("logic/A1".into()),
                to: HoleId("logic/A2".into()),
            },
            Wire {
                id: WireId("cross-board".into()),
                from: HoleId("logic/F1".into()),
                to: HoleId("display/F1".into()),
            },
        ];
        let json = serde_json::to_string(&p).unwrap();
        assert_eq!(serde_json::from_str::<Project>(&json).unwrap(), p);
        let nodes = compile_topology(&p).unwrap();
        let logic_left = nodes
            .iter()
            .find(|node| {
                node.contacts
                    .contains(&Contact::Hole(HoleId("logic/A1".into())))
            })
            .unwrap();
        assert!(
            logic_left
                .contacts
                .contains(&Contact::Hole(HoleId("logic/E1".into())))
        );
        assert!(
            !logic_left
                .contacts
                .contains(&Contact::Hole(HoleId("display/E1".into())))
        );
        let cross_board = nodes
            .iter()
            .find(|node| {
                node.contacts
                    .contains(&Contact::Hole(HoleId("logic/F1".into())))
            })
            .unwrap();
        assert!(
            cross_board
                .contacts
                .contains(&Contact::Hole(HoleId("display/F1".into())))
        );
    }

    #[test]
    fn multi_board_electrical_path_is_calculated_only_when_wired() {
        let mut p = project();
        p.board.model = multi_board_model(&["source", "load"]);
        p.components = vec![
            Component {
                id: ComponentId("V1".into()),
                kind: ComponentKind::DcVoltageSource,
                pins: BTreeMap::from([
                    (PinId("positive".into()), HoleId("source/TP+:1".into())),
                    (PinId("negative".into()), HoleId("source/TP-:1".into())),
                ]),
                parameters: BTreeMap::from([("voltage".into(), 5.0)]),
                ic_device: None,
                other_device: None,
                module: None,
                diode_model: None,
            },
            Component {
                id: ComponentId("R1".into()),
                kind: ComponentKind::Resistor,
                pins: BTreeMap::from([
                    (PinId("a".into()), HoleId("load/A1".into())),
                    (PinId("b".into()), HoleId("load/F1".into())),
                ]),
                parameters: BTreeMap::from([("resistance".into(), 100.0)]),
                ic_device: None,
                other_device: None,
                module: None,
                diode_model: None,
            },
        ];
        p.wires = vec![
            Wire {
                id: WireId("positive-link".into()),
                from: HoleId("source/TP+:2".into()),
                to: HoleId("load/A1".into()),
            },
            Wire {
                id: WireId("negative-link".into()),
                from: HoleId("load/F1".into()),
                to: HoleId("source/TP-:2".into()),
            },
        ];
        let result = solve_dc(&p, &BTreeMap::new(), &BTreeMap::new()).unwrap();
        assert!((result.resistor_currents[&ComponentId("R1".into())] - 0.05).abs() < 1e-10);
    }

    #[test]
    fn multi_board_rejects_duplicate_ids_and_unqualified_references() {
        let mut p = project();
        p.board.model = multi_board_model(&["logic", "logic"]);
        p.wires = vec![Wire {
            id: WireId("bad".into()),
            from: HoleId("A1".into()),
            to: HoleId("logic/A2".into()),
        }];
        let errors = compile_topology(&p).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error.code == "duplicate_board_id")
        );
        assert!(errors.iter().any(|error| error.code == "invalid_hole"));
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 64,
            rng_seed: proptest::test_runner::RngSeed::Fixed(0xB04D_2026),
            ..ProptestConfig::default()
        })]
        #[test]
        fn multi_board_topology_is_invariant_under_wire_order(
            endpoints in proptest::collection::vec((0usize..4, 0usize..4, 0usize..4, 0usize..4), 1..24)
        ) {
            let ids = ["logic", "display", "controls", "power"];
            let local = ["A1", "E1", "F1", "J1"];
            let mut p = project();
            p.board.model = multi_board_model(&ids);
            p.components.clear();
            p.wires = endpoints
                .iter()
                .enumerate()
                .map(|(index, &(from_board, from_hole, to_board, to_hole))| Wire {
                    id: WireId(format!("W{index}")),
                    from: HoleId(format!("{}/{}", ids[from_board], local[from_hole])),
                    to: HoleId(format!("{}/{}", ids[to_board], local[to_hole])),
                })
                .collect();
            let expected = compile_topology(&p).unwrap();
            p.wires.reverse();
            prop_assert_eq!(compile_topology(&p).unwrap(), expected);
        }
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
    fn enforces_breadboard_scale_voltage_and_resistance_ranges() {
        let mut p: Project =
            serde_json::from_str(include_str!("../../../fixtures/projects/led-bench.json"))
                .unwrap();
        for (id, name, value) in [
            ("V1", "voltage", -0.001),
            ("V1", "voltage", 12.001),
            ("R1", "resistance", 0.999),
            ("R1", "resistance", 10_000_001.0),
            ("D1", "series_resistance", 0.999),
            ("D1", "series_resistance", 10_000_001.0),
        ] {
            p.components
                .iter_mut()
                .find(|component| component.id.0 == id)
                .unwrap()
                .parameters
                .insert(name.into(), value);
            let diagnostics = compile_topology(&p).unwrap_err();
            assert!(diagnostics.iter().any(|d| {
                d.code == "parameter_out_of_range"
                    && d.path == format!("components.{id}.parameters.{name}")
            }));
            p = serde_json::from_str(include_str!("../../../fixtures/projects/led-bench.json"))
                .unwrap();
        }
    }

    #[test]
    fn enforces_breadboard_scale_capacitor_and_npn_ranges() {
        let mut rc: Project =
            serde_json::from_str(include_str!("../../../fixtures/projects/rc-charging.json"))
                .unwrap();
        for value in [1e-10, 1e-2] {
            rc.components
                .iter_mut()
                .find(|component| component.id.0 == "C1")
                .unwrap()
                .parameters
                .insert("capacitance".into(), value);
            assert!(compile_topology(&rc).is_ok());
        }
        for value in [0.999e-10, 1.001e-2] {
            rc.components
                .iter_mut()
                .find(|component| component.id.0 == "C1")
                .unwrap()
                .parameters
                .insert("capacitance".into(), value);
            let diagnostics = compile_topology(&rc).unwrap_err();
            assert!(diagnostics.iter().any(|diagnostic| {
                diagnostic.code == "parameter_out_of_range"
                    && diagnostic.path == "components.C1.parameters.capacitance"
            }));
        }

        let mut transistor: Project = serde_json::from_str(include_str!(
            "../../../fixtures/projects/transistor-bench.json"
        ))
        .unwrap();
        for (name, values) in [
            ("beta", [10.0, 1000.0]),
            ("saturation_current", [1e-16, 1e-12]),
        ] {
            for value in values {
                transistor
                    .components
                    .iter_mut()
                    .find(|component| component.id.0 == "Q1")
                    .unwrap()
                    .parameters
                    .insert(name.into(), value);
                assert!(compile_topology(&transistor).is_ok());
            }
        }
        for (name, values) in [
            ("beta", [9.999, 1000.001]),
            ("saturation_current", [0.999e-16, 1.001e-12]),
        ] {
            for value in values {
                transistor
                    .components
                    .iter_mut()
                    .find(|component| component.id.0 == "Q1")
                    .unwrap()
                    .parameters
                    .insert(name.into(), value);
                let diagnostics = compile_topology(&transistor).unwrap_err();
                assert!(diagnostics.iter().any(|diagnostic| {
                    diagnostic.code == "parameter_out_of_range"
                        && diagnostic.path == format!("components.Q1.parameters.{name}")
                }));
            }
        }
        transistor
            .components
            .iter_mut()
            .find(|component| component.id.0 == "Q1")
            .unwrap()
            .parameters
            .extend([("beta".into(), 100.0), ("saturation_current".into(), 1e-12)]);
        transistor
            .components
            .iter_mut()
            .find(|component| component.id.0 == "Q1")
            .unwrap()
            .parameters
            .extend([
                ("reverse_breakdown_voltage".into(), 8.0),
                ("reverse_breakdown_resistance".into(), 10_000.0),
            ]);
        assert!(compile_topology(&transistor).is_ok());
        transistor
            .components
            .iter_mut()
            .find(|component| component.id.0 == "Q1")
            .unwrap()
            .parameters
            .remove("reverse_breakdown_resistance");
        let diagnostics = compile_topology(&transistor).unwrap_err();
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "incomplete_reverse_breakdown_contract")
        );
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
                ic_device: None,
                other_device: None,
                module: None,
                diode_model: None,
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

    fn variable_resistor(kind: ComponentKind) -> Component {
        Component {
            id: ComponentId("RV1".into()),
            kind,
            pins: BTreeMap::from([
                (PinId("a".into()), HoleId("A1".into())),
                (PinId("b".into()), HoleId("A2".into())),
            ]),
            parameters: BTreeMap::from([
                ("min_resistance".into(), 100.0),
                ("max_resistance".into(), 10_000.0),
            ]),
            ic_device: None,
            other_device: None,
            module: None,
            diode_model: None,
        }
    }

    #[test]
    fn variable_resistor_kinds_have_documented_ranges() {
        for kind in [
            ComponentKind::Potentiometer,
            ComponentKind::Photoresistor,
            ComponentKind::Thermistor,
            ComponentKind::TouchPad,
            ComponentKind::WaterProbe,
        ] {
            let mut p = project();
            p.components = vec![variable_resistor(kind)];
            p.wires.clear();
            assert!(
                compile_topology(&p).is_ok(),
                "{kind:?} with valid range should compile"
            );
            let schema =
                schemars::SchemaGenerator::new(schemars::generate::SchemaSettings::draft2020_12())
                    .into_root_schema_for::<Project>();
            let schema = serde_json::to_value(schema).unwrap();
            let text = serde_json::to_string(&schema).unwrap();
            let expected = match kind {
                ComponentKind::Potentiometer => "\"potentiometer\"",
                ComponentKind::Photoresistor => "\"photoresistor\"",
                ComponentKind::Thermistor => "\"thermistor\"",
                ComponentKind::TouchPad => "\"touch_pad\"",
                ComponentKind::WaterProbe => "\"water_probe\"",
                _ => unreachable!(),
            };
            assert!(text.contains(expected), "schema missing {expected}");
        }
    }

    #[test]
    fn variable_resistor_range_and_reference_diagnostics() {
        let mut p = project();
        p.components = vec![variable_resistor(ComponentKind::Potentiometer)];
        p.wires.clear();
        for (name, value) in [
            ("min_resistance", 0.999),
            ("min_resistance", 10_000_001.0),
            ("max_resistance", 0.999),
            ("max_resistance", 10_000_001.0),
        ] {
            let mut bad = p.clone();
            bad.components[0].parameters.insert(name.into(), value);
            assert!(
                compile_topology(&bad)
                    .unwrap_err()
                    .iter()
                    .any(|d| d.code == "parameter_out_of_range"
                        && d.path == format!("components.RV1.parameters.{name}"))
            );
        }
        let mut inverted = p.clone();
        inverted.components[0]
            .parameters
            .insert("min_resistance".into(), 5_000.0);
        inverted.components[0]
            .parameters
            .insert("max_resistance".into(), 100.0);
        assert!(
            compile_topology(&inverted)
                .unwrap_err()
                .iter()
                .any(|d| d.code == "invalid_parameter_range")
        );

        let mut good_ratio = p.clone();
        good_ratio
            .initial_conditions
            .control_ratios
            .insert(ComponentId("RV1".into()), 0.3);
        assert!(compile_topology(&good_ratio).is_ok());

        let mut bad_ratio = p.clone();
        bad_ratio
            .initial_conditions
            .control_ratios
            .insert(ComponentId("RV1".into()), 1.5);
        assert!(
            compile_topology(&bad_ratio)
                .unwrap_err()
                .iter()
                .any(|d| d.code == "control_ratio_out_of_range")
        );

        let mut wrong_ref = p.clone();
        wrong_ref
            .initial_conditions
            .control_ratios
            .insert(ComponentId("does-not-exist".into()), 0.5);
        assert!(
            compile_topology(&wrong_ref)
                .unwrap_err()
                .iter()
                .any(|d| d.code == "invalid_initial_control_ratio")
        );
    }

    #[test]
    fn variable_resistors_round_trip_including_control_ratio() {
        let mut p = project();
        p.components = vec![
            variable_resistor(ComponentKind::Potentiometer),
            variable_resistor(ComponentKind::Photoresistor),
        ];
        p.components[1].id = ComponentId("RV2".into());
        p.components[1].pins = BTreeMap::from([
            (PinId("a".into()), HoleId("A3".into())),
            (PinId("b".into()), HoleId("A4".into())),
        ]);
        p.wires.clear();
        p.initial_conditions
            .control_ratios
            .insert(ComponentId("RV1".into()), 0.1);
        p.initial_conditions
            .control_ratios
            .insert(ComponentId("RV2".into()), 0.9);
        let json = serde_json::to_string(&p).unwrap();
        let decoded: Project = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, p);
        assert!(compile_topology(&p).is_ok());
    }

    #[test]
    fn buzzer_is_a_valid_kind_with_documented_range_and_round_trips() {
        let mut p = project();
        p.components = vec![Component {
            id: ComponentId("BZ1".into()),
            kind: ComponentKind::Buzzer,
            pins: BTreeMap::from([
                (PinId("positive".into()), HoleId("A1".into())),
                (PinId("negative".into()), HoleId("A2".into())),
            ]),
            parameters: BTreeMap::from([("resistance".into(), 32.0)]),
            ic_device: None,
            other_device: None,
            module: None,
            diode_model: None,
        }];
        p.wires.clear();
        assert!(compile_topology(&p).is_ok());
        let schema =
            schemars::SchemaGenerator::new(schemars::generate::SchemaSettings::draft2020_12())
                .into_root_schema_for::<Project>();
        let text = serde_json::to_string(&serde_json::to_value(schema).unwrap()).unwrap();
        assert!(text.contains("\"buzzer\""));
        for value in [0.999, 10_000_001.0] {
            let mut bad = p.clone();
            bad.components[0]
                .parameters
                .insert("resistance".into(), value);
            assert!(
                compile_topology(&bad)
                    .unwrap_err()
                    .iter()
                    .any(|d| d.code == "parameter_out_of_range")
            );
        }
        let json = serde_json::to_string(&p).unwrap();
        let decoded: Project = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, p);
    }

    #[test]
    fn speaker_is_a_valid_kind_with_documented_range_and_round_trips() {
        let mut p = project();
        p.components = vec![Component {
            id: ComponentId("SPK1".into()),
            kind: ComponentKind::Speaker,
            pins: BTreeMap::from([
                (PinId("positive".into()), HoleId("A1".into())),
                (PinId("negative".into()), HoleId("A2".into())),
            ]),
            parameters: BTreeMap::from([("resistance".into(), 8.0)]),
            ic_device: None,
            other_device: None,
            module: None,
            diode_model: None,
        }];
        p.wires.clear();
        assert!(compile_topology(&p).is_ok());
        let schema =
            schemars::SchemaGenerator::new(schemars::generate::SchemaSettings::draft2020_12())
                .into_root_schema_for::<Project>();
        let text = serde_json::to_string(&serde_json::to_value(schema).unwrap()).unwrap();
        assert!(text.contains("\"speaker\""));
        for value in [0.999, 100.001] {
            let mut bad = p.clone();
            bad.components[0]
                .parameters
                .insert("resistance".into(), value);
            assert!(
                compile_topology(&bad)
                    .unwrap_err()
                    .iter()
                    .any(|d| d.code == "parameter_out_of_range")
            );
        }
        let json = serde_json::to_string(&p).unwrap();
        let decoded: Project = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, p);
    }

    #[test]
    fn passive_piezo_is_a_valid_kind_with_documented_range_and_round_trips() {
        let mut p = project();
        p.components = vec![Component {
            id: ComponentId("PZ1".into()),
            kind: ComponentKind::PiezoPassive,
            pins: BTreeMap::from([
                (PinId("positive".into()), HoleId("A1".into())),
                (PinId("negative".into()), HoleId("A2".into())),
            ]),
            parameters: BTreeMap::from([("resistance".into(), 32.0)]),
            ic_device: None,
            other_device: None,
            module: None,
            diode_model: None,
        }];
        p.wires.clear();
        assert!(compile_topology(&p).is_ok());
        let schema =
            schemars::SchemaGenerator::new(schemars::generate::SchemaSettings::draft2020_12())
                .into_root_schema_for::<Project>();
        let text = serde_json::to_string(&serde_json::to_value(schema).unwrap()).unwrap();
        assert!(text.contains("\"piezo_passive\""));
        for value in [0.999, 10_000_001.0] {
            let mut bad = p.clone();
            bad.components[0]
                .parameters
                .insert("resistance".into(), value);
            assert!(
                compile_topology(&bad)
                    .unwrap_err()
                    .iter()
                    .any(|d| d.code == "parameter_out_of_range")
            );
        }
        let json = serde_json::to_string(&p).unwrap();
        let decoded: Project = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, p);
    }

    #[test]
    fn other_device_contract_derives_topology_and_current() {
        let mut p = project();
        p.components = vec![
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
                id: ComponentId("X1".into()),
                kind: ComponentKind::Other,
                pins: BTreeMap::from([
                    (PinId("positive".into()), HoleId("A1".into())),
                    (PinId("negative".into()), HoleId("F1".into())),
                ]),
                parameters: BTreeMap::new(),
                ic_device: None,
                other_device: Some(OtherDeviceSpec {
                    pin_roles: BTreeMap::from([
                        (PinId("positive".into()), OtherDevicePinRole::Terminal),
                        (PinId("negative".into()), OtherDevicePinRole::Reference),
                    ]),
                    behavior: OtherDeviceBehavior::Resistive {
                        positive: PinId("positive".into()),
                        negative: PinId("negative".into()),
                        resistance: 100.0,
                    },
                }),
                module: None,
                diode_model: None,
            },
        ];
        p.wires = vec![
            Wire {
                id: WireId("V+".into()),
                from: HoleId("TP+:1".into()),
                to: HoleId("A1".into()),
            },
            Wire {
                id: WireId("V-".into()),
                from: HoleId("TP-:1".into()),
                to: HoleId("F1".into()),
            },
        ];
        assert!(compile_topology(&p).is_ok());
        let result = solve_dc(&p, &BTreeMap::new(), &BTreeMap::new()).unwrap();
        let current =
            result.other_terminal_currents[&ComponentId("X1".into())][&PinId("positive".into())];
        assert!((current - 0.05).abs() < 1e-10);
        let json = serde_json::to_string(&p).unwrap();
        assert_eq!(serde_json::from_str::<Project>(&json).unwrap(), p);
    }
}
