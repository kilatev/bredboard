//! Presentation-only editing gestures; Project remains the electrical authority.
use super::*;
use crate::text::assembly as labels;
use bredboard_core::{HoleId, Wire, WireId};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Select,
    Wire,
    Move,
    Remove,
    Place(ComponentKind),
}
#[derive(Clone, Copy)]
pub enum Command {
    Tool(Tool),
    Clear,
    Undo,
    Flip,
    Parameter,
    Value,
    Toggle,
}
#[derive(Clone)]
enum Lead {
    Pin(ComponentId, PinId),
    Wire(WireId, bool),
}
#[derive(Default)]
pub struct Editor {
    pub tool: Option<Tool>,
    pending: Vec<HoleId>,
    moving: Option<Lead>,
    selected: Option<ComponentId>,
    parameter: usize,
    input: Option<String>,
    undo: Vec<Project>,
    pub message: String,
}

pub const PARTS: &[(ComponentKind, &str)] = &[
    (ComponentKind::Resistor, labels::RESISTOR),
    (ComponentKind::Led, labels::LED),
    (ComponentKind::Capacitor, labels::CAPACITOR),
    (ComponentKind::Diode, labels::DIODE),
    (ComponentKind::NpnTransistor, labels::NPN),
    (ComponentKind::MomentaryButton, labels::BUTTON),
    (ComponentKind::ChangeoverSwitch, labels::SWITCH),
];

fn template(kind: ComponentKind) -> Component {
    let json = match kind {
        ComponentKind::Capacitor | ComponentKind::ChangeoverSwitch => RC_JSON,
        ComponentKind::NpnTransistor => TRANSISTOR_JSON,
        _ => LED_JSON,
    };
    let project: Project = serde_json::from_str(json).expect("embedded part catalog");
    if kind == ComponentKind::Diode {
        let mut diode = project
            .components
            .into_iter()
            .find(|c| c.kind == ComponentKind::Led)
            .unwrap();
        diode.kind = kind;
        diode.parameters.insert("forward_voltage".into(), 0.7);
        diode.parameters.insert("series_resistance".into(), 1.0);
        diode
    } else {
        project
            .components
            .into_iter()
            .find(|c| c.kind == kind)
            .expect("catalog part")
    }
}

fn pins(kind: ComponentKind) -> Vec<PinId> {
    let names: &[&str] = match kind {
        ComponentKind::Led | ComponentKind::Diode => &["anode", "cathode"],
        ComponentKind::Capacitor | ComponentKind::DcVoltageSource => &["positive", "negative"],
        ComponentKind::NpnTransistor => &["base", "collector", "emitter"],
        ComponentKind::ChangeoverSwitch => &["common", "normally_closed", "normally_open"],
        _ => &["a", "b"],
    };
    names.iter().map(|p| PinId((*p).into())).collect()
}

pub fn blank() -> Bench {
    let mut bench = Bench::new(Circuit::Led);
    bench.project.title = "Free assembly".into();
    bench
        .project
        .components
        .retain(|c| c.kind == ComponentKind::DcVoltageSource);
    bench.project.wires.clear();
    bench.project.initial_conditions = Default::default();
    bench.initial = bench.project.clone();
    bench.simulation = SimulationState::new(&bench.project);
    bench.editor = Some(Editor {
        tool: Some(Tool::Wire),
        message: labels::WELCOME.into(),
        ..default()
    });
    bench
}

pub fn enable(bench: &mut Bench) {
    bench.project.faults.clear();
    bench.initial = bench.project.clone();
    bench.simulation.running = false;
    bench.editor = Some(Editor {
        tool: Some(Tool::Select),
        message: labels::COPY.into(),
        ..default()
    });
}

fn physical_contacts(project: &Project) -> Result<(), String> {
    let mut used = BTreeSet::new();
    for hole in project
        .components
        .iter()
        .flat_map(|c| c.pins.values())
        .chain(project.wires.iter().flat_map(|w| [&w.from, &w.to]))
    {
        if !used.insert(hole) {
            return Err(format!(
                "{} is occupied; use another hole in the same contact strip.",
                hole.0
            ));
        }
    }
    Ok(())
}

fn replace(bench: &mut Bench, mut candidate: Project) -> Result<(), String> {
    let ids: BTreeSet<_> = candidate.components.iter().map(|c| c.id.clone()).collect();
    candidate
        .initial_conditions
        .capacitor_voltages
        .retain(|id, _| ids.contains(id));
    candidate
        .initial_conditions
        .controls
        .retain(|id, _| ids.contains(id));
    candidate
        .initial_conditions
        .control_ratios
        .retain(|id, _| ids.contains(id));
    candidate
        .initial_conditions
        .digital_states
        .retain(|id, _| ids.contains(id));
    candidate
        .initial_conditions
        .coupled_winding_currents
        .retain(|id, _| ids.contains(id));
    compile_topology(&candidate).map_err(|errors| errors[0].message.clone())?;
    physical_contacts(&candidate)?;
    bench.initial = candidate.clone();
    bench.project = candidate;
    bench.simulation = SimulationState::new(&bench.project);
    bench.act(Action::SingleStep);
    Ok(())
}

impl Editor {
    fn cancel(&mut self) {
        self.pending.clear();
        self.moving = None;
        self.input = None;
    }
    fn commit(&mut self, bench: &mut Bench, candidate: Project) {
        if candidate == bench.project {
            return;
        }
        let previous = bench.project.clone();
        match replace(bench, candidate) {
            Ok(()) => {
                if self.undo.len() == 32 {
                    self.undo.remove(0);
                }
                self.undo.push(previous);
                self.message = labels::UPDATED.into();
                if matches!(self.tool, Some(Tool::Place(_) | Tool::Wire))
                    && let Some(tool) = learning::next_tool(bench)
                {
                    self.tool = Some(tool);
                }
                if !bench
                    .project
                    .components
                    .iter()
                    .any(|c| Some(&c.id) == self.selected.as_ref())
                {
                    self.selected = None;
                }
            }
            Err(error) => self.message = error,
        }
    }
    fn selected<'a>(&self, bench: &'a Bench) -> Option<&'a Component> {
        bench
            .project
            .components
            .iter()
            .find(|c| Some(&c.id) == self.selected.as_ref())
    }
    fn parameter_name(&self, bench: &Bench) -> Option<String> {
        let part = self.selected(bench)?;
        part.parameters
            .keys()
            .nth(self.parameter % part.parameters.len().max(1))
            .cloned()
    }
    fn apply(&mut self, bench: &mut Bench, command: Command) {
        match command {
            Command::Tool(tool) => {
                self.cancel();
                self.tool = Some(tool);
                self.message = format!("Tool: {tool:?}");
            }
            Command::Clear => {
                self.cancel();
                let mut candidate = bench.project.clone();
                candidate.components = blank().project.components;
                candidate.wires.clear();
                candidate.initial_conditions = Default::default();
                self.commit(bench, candidate);
            }
            Command::Undo => {
                self.cancel();
                if let Some(project) = self.undo.pop() {
                    // Only previously accepted, structurally valid projects enter history.
                    self.message = replace(bench, project)
                        .err()
                        .unwrap_or_else(|| labels::UNDONE.into());
                    self.selected = None;
                }
            }
            Command::Parameter => {
                self.input = None;
                self.parameter += 1;
            }
            Command::Value => {
                if self.parameter_name(bench).is_some() {
                    self.input = Some(String::new());
                } else {
                    self.message = labels::SELECT_PARAMETER.into();
                }
            }
            Command::Flip => {
                let mut project = bench.project.clone();
                if let Some(part) = project
                    .components
                    .iter_mut()
                    .find(|c| Some(&c.id) == self.selected.as_ref())
                {
                    if part.pins.len() == 2 {
                        let keys: Vec<_> = part.pins.keys().cloned().collect();
                        let a = part.pins[&keys[0]].clone();
                        let b = part.pins[&keys[1]].clone();
                        part.pins.insert(keys[0].clone(), b);
                        part.pins.insert(keys[1].clone(), a);
                        self.commit(bench, project);
                    } else {
                        self.message = labels::TWO_TERMINALS.into();
                    }
                }
            }
            Command::Toggle => {
                if let Some(part) = self.selected(bench) {
                    let id = part.id.clone();
                    let state = match bench.simulation.controls.get(&id) {
                        Some(ControlState::ButtonReleased) => ControlState::ButtonPressed,
                        Some(ControlState::ButtonPressed) => ControlState::ButtonReleased,
                        Some(ControlState::SwitchNormallyClosed) => {
                            ControlState::SwitchNormallyOpen
                        }
                        Some(ControlState::SwitchNormallyOpen) => {
                            ControlState::SwitchNormallyClosed
                        }
                        None => {
                            self.message = labels::SELECT_CONTROL.into();
                            return;
                        }
                    };
                    bench.act(Action::SetControl {
                        component: id,
                        state,
                    });
                }
            }
        }
    }
    fn click(&mut self, bench: &mut Bench, point: Vec2) {
        let hole = hole_at(point).map(HoleId);
        let part = part_at(&bench.project, point);
        match self.tool.unwrap_or(Tool::Select) {
            Tool::Select => {
                self.selected = part;
                self.parameter = 0;
                self.input = None;
            }
            Tool::Remove => {
                let mut project = bench.project.clone();
                if let Some(id) = part {
                    project.components.retain(|c| c.id != id);
                } else if let Some(id) = wire_at(&project, point) {
                    project.wires.retain(|w| w.id != id);
                }
                self.commit(bench, project);
            }
            Tool::Move => {
                let Some(hole) = hole else {
                    return;
                };
                if let Some(lead) = self.moving.take() {
                    let mut project = bench.project.clone();
                    match lead {
                        Lead::Pin(id, pin) => {
                            if let Some(c) = project.components.iter_mut().find(|c| c.id == id) {
                                c.pins.insert(pin, hole);
                            }
                        }
                        Lead::Wire(id, from) => {
                            if let Some(w) = project.wires.iter_mut().find(|w| w.id == id) {
                                if from {
                                    w.from = hole;
                                } else {
                                    w.to = hole;
                                }
                            }
                        }
                    }
                    self.commit(bench, project);
                } else {
                    self.moving = bench
                        .project
                        .components
                        .iter()
                        .find_map(|c| {
                            c.pins
                                .iter()
                                .find(|(_, h)| **h == hole)
                                .map(|(p, _)| Lead::Pin(c.id.clone(), p.clone()))
                        })
                        .or_else(|| {
                            bench
                                .project
                                .wires
                                .iter()
                                .find(|w| w.from == hole || w.to == hole)
                                .map(|w| Lead::Wire(w.id.clone(), w.from == hole))
                        });
                    self.message = labels::MOVE_DESTINATION.into();
                }
            }
            Tool::Wire | Tool::Place(_) => {
                let Some(hole) = hole else {
                    return;
                };
                self.pending.push(hole);
                let needed = if let Some(Tool::Place(kind)) = self.tool {
                    pins(kind).len()
                } else {
                    2
                };
                if self.pending.len() < needed {
                    return;
                }
                let holes = std::mem::take(&mut self.pending);
                let mut project = bench.project.clone();
                if let Some(Tool::Place(kind)) = self.tool {
                    let mut part =
                        learning::reference_template(bench, kind).unwrap_or_else(|| template(kind));
                    let prefix = match kind {
                        ComponentKind::Resistor => "R",
                        ComponentKind::Capacitor => "C",
                        ComponentKind::NpnTransistor => "Q",
                        ComponentKind::MomentaryButton => "B",
                        ComponentKind::ChangeoverSwitch => "S",
                        _ => "D",
                    };
                    part.id = ComponentId(
                        (1..=1024)
                            .map(|n| format!("{prefix}{n}"))
                            .find(|id| !project.components.iter().any(|c| c.id.0 == *id))
                            .unwrap(),
                    );
                    part.pins = pins(kind).into_iter().zip(holes).collect();
                    self.selected = Some(part.id.clone());
                    project.components.push(part);
                } else {
                    if holes[0] == holes[1]
                        || project.wires.iter().any(|w| {
                            (w.from == holes[0] && w.to == holes[1])
                                || (w.to == holes[0] && w.from == holes[1])
                        })
                    {
                        self.message = labels::DUPLICATE_WIRE.into();
                        return;
                    }
                    let id = (1..=1024)
                        .map(|n| format!("W{n}"))
                        .find(|id| !project.wires.iter().any(|w| w.id.0 == *id))
                        .unwrap();
                    project.wires.push(Wire {
                        id: WireId(id),
                        from: holes[0].clone(),
                        to: holes[1].clone(),
                    });
                }
                self.commit(bench, project);
            }
        }
    }
    pub fn description(&self, bench: &Bench) -> String {
        let mut text = format!(
            "{:?}\n{}\n",
            self.tool.unwrap_or(Tool::Select),
            self.message
        );
        if let Some(Tool::Place(kind)) = self.tool {
            let roles = pins(kind);
            text += &format!(
                "Next: {} ({}/{})\n",
                roles[self.pending.len()].0,
                self.pending.len() + 1,
                roles.len()
            );
        } else if let Some(first) = self.pending.first() {
            text += &format!("{} -> choose second hole\n", first.0);
        }
        if let Some(part) = self.selected(bench) {
            text += &format!("{}\n", component_summary(part));
            if let Some(name) = self.parameter_name(bench) {
                text += &format!(
                    "{name}: {} [SI units]\n",
                    self.input.as_ref().map_or_else(
                        || part.parameters[&name].to_string(),
                        |s| format!("{s}_ (Enter to apply)")
                    )
                );
            }
            text += &part
                .pins
                .iter()
                .map(|(p, h)| format!("{}={} ", p.0, h.0))
                .collect::<String>();
            if !bench.simulation.stale
                && let Some(result) = &bench.simulation.last_valid
            {
                let current = result
                    .led_currents
                    .get(&part.id)
                    .or_else(|| result.resistor_currents.get(&part.id))
                    .or_else(|| result.source_currents.get(&part.id));
                let mut measurements = Vec::new();
                if let Some(current) = current {
                    measurements.push(format!("I {:.3} mA", current * 1000.0));
                }
                for (pin, hole) in &part.pins {
                    if let Some(node) = result.node_voltages.iter().find(|n| {
                        n.contacts
                            .contains(&bredboard_core::Contact::Hole(hole.clone()))
                    }) {
                        measurements.push(format!("{} {:.2} V", pin.0, node.voltage));
                    }
                }
                text += &format!("\n{}", measurements.join(" | "));
            }
        }
        text
    }
}

fn part_at(project: &Project, point: Vec2) -> Option<ComponentId> {
    project
        .components
        .iter()
        .rev()
        .find(|c| {
            let points: Vec<_> = c
                .pins
                .values()
                .filter_map(|h| hole_position(&h.0))
                .collect();
            let center = if c.kind == ComponentKind::DcVoltageSource {
                Vec2::new(
                    (column_x("TP+").unwrap() + column_x("TP-").unwrap()) / 2.0,
                    322.0,
                )
            } else {
                points.iter().copied().sum::<Vec2>() / points.len().max(1) as f32
            };
            point.distance(center) < 16.0 || points.iter().any(|p| p.distance(point) < 7.0)
        })
        .map(|c| c.id.clone())
}
fn wire_at(project: &Project, point: Vec2) -> Option<WireId> {
    project
        .wires
        .iter()
        .rev()
        .find(|w| {
            let a = hole_position(&w.from.0).unwrap();
            let b = hole_position(&w.to.0).unwrap();
            let d = b - a;
            let t = ((point - a).dot(d) / d.length_squared().max(0.001)).clamp(0.0, 1.0);
            point.distance(a + t * d) <= 5.0
        })
        .map(|w| w.id.clone())
}

pub fn assisted_edit(bench: &mut Bench, candidate: Project) {
    if let Some(mut editor) = bench.editor.take() {
        editor.cancel();
        let previous = bench.project.clone();
        editor.commit(bench, candidate);
        if bench.project != previous
            && let Some(tool) = learning::next_tool(bench)
        {
            editor.tool = Some(tool);
        }
        bench.editor = Some(editor);
    }
}

pub fn command(bench: &mut Bench, value: Command) {
    if let Some(mut editor) = bench.editor.take() {
        editor.apply(bench, value);
        bench.editor = Some(editor);
    }
}
pub fn click(bench: &mut Bench, point: Vec2) {
    if let Some(mut editor) = bench.editor.take() {
        editor.click(bench, point);
        bench.editor = Some(editor);
    }
}

pub fn draw(commands: &mut Commands, images: &mut Assets<Image>, bench: &Bench) {
    rect(
        commands,
        Vec2::ZERO,
        Vec2::new(1200.0, 760.0),
        Color::srgb(0.10, 0.14, 0.16),
        -1.0,
    );
    spawn_board(commands, images, bench);
    if bench.learning.is_none() {
        label(
            commands,
            labels::TITLE,
            Vec2::new(185.0, 330.0),
            28.0,
            Color::srgb(0.42, 0.90, 0.76),
        );
    }
    for (index, (kind, name)) in PARTS.iter().enumerate() {
        button(
            commands,
            name,
            Vec2::new(
                75.0 + (index % 2) as f32 * 220.0,
                278.0 - (index / 2) as f32 * 41.0,
            ),
            Vec2::new(210.0, 35.0),
            Control::Assembly(Command::Tool(Tool::Place(*kind))),
        );
    }
    for (index, (name, tool)) in [
        (labels::SELECT, Tool::Select),
        (labels::WIRE, Tool::Wire),
        (labels::MOVE, Tool::Move),
        (labels::REMOVE, Tool::Remove),
    ]
    .into_iter()
    .enumerate()
    {
        button(
            commands,
            name,
            Vec2::new(20.0 + index as f32 * 112.0, 105.0),
            Vec2::new(105.0, 35.0),
            Control::Assembly(Command::Tool(tool)),
        );
    }
    let info = wrapped_label(
        commands,
        "",
        Vec2::new(-25.0, 77.0),
        14.0,
        Color::srgb(0.76, 0.85, 0.82),
    );
    commands
        .entity(info)
        .insert((Readout::Editor, TextBounds::new(TEXT_WRAP_WIDTH, 145.0)));
    let status = wrapped_label(
        commands,
        "",
        Vec2::new(-25.0, -72.0),
        14.0,
        Color::srgb(0.72, 0.82, 0.83),
    );
    commands
        .entity(status)
        .insert((Readout::Status, TextBounds::new(TEXT_WRAP_WIDTH, 65.0)));
    let hover = label(
        commands,
        "",
        Vec2::new(185.0, -133.0),
        13.0,
        Color::srgb(0.65, 0.81, 0.85),
    );
    commands.entity(hover).insert(Readout::Hover);
    for (i, (name, cmd)) in [
        (labels::PARAMETER, Command::Parameter),
        (labels::VALUE, Command::Value),
        (labels::FLIP, Command::Flip),
        (labels::OPERATE, Command::Toggle),
    ]
    .into_iter()
    .enumerate()
    {
        button(
            commands,
            name,
            Vec2::new(
                70.0 + (i % 2) as f32 * 220.0,
                -180.0 - (i / 2) as f32 * 43.0,
            ),
            Vec2::new(210.0, 36.0),
            Control::Assembly(cmd),
        );
    }
    button(
        commands,
        labels::RUN,
        Vec2::new(55.0, -270.0),
        Vec2::new(175.0, 36.0),
        Control::RunPause,
    );
    button(
        commands,
        labels::RESET,
        Vec2::new(255.0, -270.0),
        Vec2::new(175.0, 36.0),
        Control::Reset,
    );
    button(
        commands,
        labels::UNDO,
        Vec2::new(35.0, -312.0),
        Vec2::new(130.0, 35.0),
        Control::Assembly(Command::Undo),
    );
    button(
        commands,
        labels::CLEAR,
        Vec2::new(180.0, -312.0),
        Vec2::new(130.0, 35.0),
        Control::Assembly(Command::Clear),
    );
    button(
        commands,
        labels::CIRCUITS,
        Vec2::new(325.0, -312.0),
        Vec2::new(130.0, 35.0),
        Control::Back,
    );
    label(
        commands,
        labels::HELP,
        Vec2::new(185.0, -353.0),
        13.0,
        Color::srgb(0.58, 0.76, 0.76),
    );
    if let Some(editor) = &bench.editor {
        for h in &editor.pending {
            if let Some(point) = hole_position(&h.0) {
                rect(
                    commands,
                    point,
                    Vec2::splat(9.0),
                    Color::srgb(1.0, 0.65, 0.25),
                    1.9,
                );
            }
        }
    }
}

pub fn keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    mut session: ResMut<Session>,
    scene: Query<Entity, With<SceneEntity>>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut events: MessageReader<bevy::input::keyboard::KeyboardInput>,
) {
    let input: Vec<_> = events.read().cloned().collect();
    let Some(bench) = session.bench.as_mut() else {
        return;
    };
    if !learning::is_board(bench) {
        return;
    }
    let Some(mut editor) = bench.editor.take() else {
        return;
    };
    let mut changed = false;
    if mouse.just_pressed(MouseButton::Right) || keys.just_pressed(KeyCode::Escape) {
        editor.cancel();
        editor.message = labels::CANCELLED.into();
        changed = true;
    } else if editor.input.is_some() {
        for event in input
            .iter()
            .filter(|e| e.state == bevy::input::ButtonState::Pressed)
        {
            if editor.input.is_none() {
                break;
            }
            match event.key_code {
                KeyCode::Backspace => {
                    editor.input.as_mut().unwrap().pop();
                }
                KeyCode::Enter | KeyCode::NumpadEnter => {
                    if let (Some(value), Some(name), Some(id)) = (
                        editor
                            .input
                            .as_ref()
                            .and_then(|s| s.parse::<f64>().ok())
                            .filter(|v| v.is_finite()),
                        editor.parameter_name(bench),
                        editor.selected.clone(),
                    ) {
                        let mut project = bench.project.clone();
                        if let Some(part) = project.components.iter_mut().find(|c| c.id == id) {
                            part.parameters.insert(name, value);
                        }
                        editor.commit(bench, project);
                        editor.input = None;
                    } else {
                        editor.message = labels::INVALID_NUMBER.into();
                    }
                }
                _ => {
                    if let Some(text) = &event.text {
                        let buffer = editor.input.as_mut().unwrap();
                        for ch in text.chars().filter(|ch| {
                            ch.is_ascii_digit() || matches!(ch, '.' | '-' | '+' | 'e' | 'E')
                        }) {
                            if buffer.len() < 32 {
                                buffer.push(ch);
                            }
                        }
                    }
                }
            }
            changed = true;
        }
    } else if keys.just_pressed(KeyCode::KeyZ)
        && (keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight))
    {
        editor.apply(bench, Command::Undo);
        changed = true;
    } else if keys.just_pressed(KeyCode::Space) {
        bench.act(if bench.simulation.running {
            Action::Pause
        } else {
            Action::Run
        });
    }
    bench.editor = Some(editor);
    if changed {
        clear_scene(&mut commands, &scene);
        spawn_bench(&mut commands, &mut images, bench);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn place(bench: &mut Bench, tool: Tool, contacts: &[&str]) {
        command(bench, Command::Tool(tool));
        for hole in contacts {
            click(bench, hole_position(hole).unwrap());
        }
    }
    fn led_bench() -> Bench {
        let mut bench = blank();
        place(
            &mut bench,
            Tool::Place(ComponentKind::Resistor),
            &["A5", "A8"],
        );
        place(&mut bench, Tool::Place(ComponentKind::Led), &["B8", "A11"]);
        place(&mut bench, Tool::Wire, &["TP+:5", "B5"]);
        place(&mut bench, Tool::Wire, &["B11", "TP-:11"]);
        bench
    }
    fn current(bench: &Bench) -> f64 {
        bench
            .simulation
            .last_valid
            .as_ref()
            .filter(|_| !bench.simulation.stale)
            .and_then(|r| r.led_currents.get(&ComponentId("D1".into())))
            .copied()
            .unwrap_or(0.0)
    }
    #[test]
    fn assembly_drives_the_real_solver_and_reversing_led_blocks_current() {
        let mut bench = led_bench();
        assert!(current(&bench) > 0.005);
        command(&mut bench, Command::Tool(Tool::Select));
        click(&mut bench, hole_position("B8").unwrap());
        command(&mut bench, Command::Flip);
        assert!(current(&bench).abs() < 1e-6);
        command(&mut bench, Command::Undo);
        assert!(current(&bench) > 0.005);
    }
    #[test]
    fn moving_pins_and_wire_ends_preserves_hole_based_connectivity() {
        let mut bench = led_bench();
        place(&mut bench, Tool::Move, &["A11", "A12"]);
        assert!(current(&bench) < 1e-6);
        place(&mut bench, Tool::Move, &["A12", "E11"]);
        assert!(current(&bench) > 0.005);
        place(&mut bench, Tool::Move, &["B11", "C12"]);
        assert!(current(&bench) < 1e-6);
        command(&mut bench, Command::Undo);
        assert!(current(&bench) > 0.005);
    }
    #[test]
    fn every_palette_part_is_accepted_and_cancellation_has_no_domain_effect() {
        for (kind, _) in PARTS {
            let mut bench = blank();
            let before = bench.project.clone();
            place(&mut bench, Tool::Place(*kind), &["A5"]);
            assert_eq!(bench.project, before);
            bench.editor.as_mut().unwrap().cancel();
            let roles = pins(*kind);
            let holes = ["A5", "A8", "A11"];
            place(&mut bench, Tool::Place(*kind), &holes[..roles.len()]);
            assert_eq!(bench.project.components.len(), 2, "{kind:?}");
            compile_topology(&bench.project).unwrap();
        }
    }
    #[test]
    fn completed_edit_benchmark() {
        let mut bench = led_bench();
        let started = std::time::Instant::now();
        for _ in 0..100 {
            place(&mut bench, Tool::Move, &["A11", "E11"]);
            place(&mut bench, Tool::Move, &["E11", "A11"]);
        }
        eprintln!(
            "ASSEMBLY_BENCH: 200 validated, calculated lead edits in {:?}",
            started.elapsed()
        );
        assert!(current(&bench) > 0.005);
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 48, rng_seed: proptest::test_runner::RngSeed::Fixed(20261025), .. ProptestConfig::default() })]
        #[test]
        fn invalid_edits_are_atomic(row in 0u32..1000) {
            let mut bench=led_bench();
            let project=bench.project.clone(); let state=bench.simulation.clone();
            let mut candidate=project.clone();
            candidate.components[0].pins.insert(PinId("positive".into()),HoleId(format!("BAD{row}")));
            prop_assert!(replace(&mut bench,candidate).is_err());
            prop_assert_eq!(&bench.project,&project);
            prop_assert_eq!(&bench.simulation,&state);
        }
        #[test]
        fn arbitrary_wire_edit_sequences_preserve_placement_and_ids(operations in prop::collection::vec((0u8..5,1u32..31,1u32..31),1..64)) {
            let mut bench=blank();
            for (op,a,b) in operations {
                let first=format!("A{a}");let second=format!("F{b}");
                let previous=bench.project.clone();
                match op {
                    0|1=>place(&mut bench,Tool::Wire,&[&first,&second]),
                    2=>command(&mut bench,Command::Undo),
                    3=>place(&mut bench,Tool::Remove,&[&first]),
                    _=>place(&mut bench,Tool::Place(ComponentKind::Resistor),&[&first,&second]),
                }
                prop_assert!(compile_topology(&bench.project).is_ok());
                prop_assert!(physical_contacts(&bench.project).is_ok());
                prop_assert!(bench.editor.as_ref().unwrap().undo.len()<=32);
                if bench.project!=previous && op!=2 {
                    prop_assert_eq!(bench.editor.as_ref().unwrap().undo.last(),Some(&previous));
                }
            }
        }
    }
}
