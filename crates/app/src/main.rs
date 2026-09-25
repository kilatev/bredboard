mod sprites;
mod text;

use bevy::camera::ScalingMode;
use bevy::prelude::*;
use bredboard_core::{
    Action, Component, ComponentId, ComponentKind, ControlState, Project, SimulationState,
    advance_steps, apply_actions, compile_topology,
};

const LED_JSON: &str = include_str!("../../../fixtures/projects/led-bench.json");
const RC_JSON: &str = include_str!("../../../fixtures/projects/rc-bench.json");
const TRANSISTOR_JSON: &str = include_str!("../../../fixtures/projects/transistor-bench.json");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Circuit {
    Led,
    Rc,
    Transistor,
}

impl Circuit {
    fn all() -> [Self; 3] {
        [Self::Led, Self::Rc, Self::Transistor]
    }
    fn json(self) -> &'static str {
        match self {
            Self::Led => LED_JSON,
            Self::Rc => RC_JSON,
            Self::Transistor => TRANSISTOR_JSON,
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Led => "LED + RESISTOR",
            Self::Rc => "CAPACITOR CHARGE / DISCHARGE",
            Self::Transistor => "TRANSISTOR SWITCH",
        }
    }
    fn control(self) -> &'static str {
        match self {
            Self::Rc => "S1: CHANGE PATH",
            _ => "B1: PRESS / RELEASE",
        }
    }
}

struct Bench {
    circuit: Circuit,
    project: Project,
    initial: Project,
    simulation: SimulationState,
}

impl Bench {
    fn new(circuit: Circuit) -> Self {
        let project: Project = serde_json::from_str(circuit.json()).expect("embedded project JSON");
        compile_topology(&project).expect("embedded project topology");
        Self {
            circuit,
            initial: project.clone(),
            simulation: SimulationState::new(&project),
            project,
        }
    }
    fn act(&mut self, action: Action) {
        apply_actions(
            &mut self.project,
            &self.initial,
            &mut self.simulation,
            &[action],
        );
    }
    fn toggle_control(&mut self) {
        let (id, state) = if self.circuit == Circuit::Rc {
            let id = ComponentId("S1".into());
            let state = if self.simulation.controls[&id] == ControlState::SwitchNormallyClosed {
                ControlState::SwitchNormallyOpen
            } else {
                ControlState::SwitchNormallyClosed
            };
            (id, state)
        } else {
            let id = ComponentId("B1".into());
            let state = if self.simulation.controls[&id] == ControlState::ButtonReleased {
                ControlState::ButtonPressed
            } else {
                ControlState::ButtonReleased
            };
            (id, state)
        };
        self.act(Action::SetControl {
            component: id,
            state,
        });
    }
}

#[derive(Resource, Default)]
struct Session {
    bench: Option<Bench>,
}

#[derive(Component)]
struct SceneEntity;
#[derive(Component, Clone, Copy)]
struct ClickTarget(Control, Vec2);
#[derive(Clone, Copy)]
enum Control {
    Select(Circuit),
    Back,
    RunPause,
    Reset,
    Circuit,
}
#[derive(Component)]
enum Readout {
    Status,
    Value,
    Control,
    Hover,
}
/// A component drawn with pixel art; `states` holds one image per visual state.
#[derive(Component)]
struct PartVisual {
    id: ComponentId,
    kind: ComponentKind,
    states: Vec<Handle<Image>>,
    shown: usize,
}

fn main() {
    let mut app = App::new();
    app.insert_resource(Time::<Virtual>::from_max_delta(std::time::Duration::MAX))
        .insert_resource(Time::<Fixed>::from_hz(1_000.0))
        .insert_resource(Session::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: text::WINDOW_TITLE.into(),
                resolution: (1200, 760).into(),
                resize_constraints: WindowResizeConstraints {
                    min_width: 1200.0,
                    min_height: 760.0,
                    ..default()
                },
                #[cfg(target_arch = "wasm32")]
                canvas: Some("#bredboard".into()),
                #[cfg(target_arch = "wasm32")]
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, fixed_step)
        .add_systems(Update, (handle_mouse, handle_keyboard, update_view).chain())
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: 1200.0,
                min_height: 760.0,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
    spawn_menu(&mut commands);
}

fn rect(commands: &mut Commands, point: Vec2, size: Vec2, color: Color, z: f32) -> Entity {
    commands
        .spawn((
            Sprite::from_color(color, size),
            Transform::from_xyz(point.x, point.y, z),
            SceneEntity,
        ))
        .id()
}

fn label(
    commands: &mut Commands,
    value: impl Into<String>,
    point: Vec2,
    size: f32,
    color: Color,
) -> Entity {
    commands
        .spawn((
            Text2d::new(value),
            TextFont::from_font_size(size),
            TextColor(color),
            Transform::from_xyz(point.x, point.y, 2.0),
            SceneEntity,
        ))
        .id()
}

fn button(commands: &mut Commands, value: &str, point: Vec2, size: Vec2, control: Control) {
    let entity = rect(commands, point, size, Color::srgb(0.13, 0.30, 0.35), 1.0);
    commands.entity(entity).insert(ClickTarget(control, size));
    label(commands, value, point, 21.0, Color::srgb(0.94, 0.97, 0.96));
}

fn spawn_menu(commands: &mut Commands) {
    rect(
        commands,
        Vec2::ZERO,
        Vec2::new(1200.0, 760.0),
        Color::srgb(0.09, 0.13, 0.15),
        -1.0,
    );
    label(
        commands,
        "BREDBOARD",
        Vec2::new(0.0, 285.0),
        44.0,
        Color::srgb(0.42, 0.91, 0.76),
    );
    label(
        commands,
        "Choose a real breadboard circuit",
        Vec2::new(0.0, 220.0),
        23.0,
        Color::srgb(0.77, 0.84, 0.83),
    );
    for (index, circuit) in Circuit::all().into_iter().enumerate() {
        button(
            commands,
            circuit.label(),
            Vec2::new(0.0, 100.0 - index as f32 * 105.0),
            Vec2::new(520.0, 74.0),
            Control::Select(circuit),
        );
    }
    label(
        commands,
        "5 V examples - select a circuit to inspect its exact holes and wires",
        Vec2::new(0.0, -285.0),
        17.0,
        Color::srgb(0.58, 0.68, 0.68),
    );
}

fn line(commands: &mut Commands, from: Vec2, to: Vec2, color: Color, width: f32, z: f32) {
    let delta = to - from;
    let entity = rect(
        commands,
        (from + to) * 0.5,
        Vec2::new(delta.length().max(1.0), width),
        color,
        z,
    );
    commands.entity(entity).insert(Transform {
        translation: ((from + to) * 0.5).extend(z),
        rotation: Quat::from_rotation_z(delta.y.atan2(delta.x)),
        ..default()
    });
}

/// Square 16-unit hole pitch (8 art pixels at 2 units each) keeps sprites pixel-exact.
const PITCH: f32 = 16.0;
const BOARD_CENTER_X: f32 = -385.0;
const TOP_ROW_Y: f32 = 255.0;

/// Horizontal position of a board column (A–J) or rail (TP+/TP-/BP+/BP-).
/// Strips and rails are separated by two-pitch gaps, as on a real breadboard.
fn column_x(column: &str) -> Option<f32> {
    let pitches = match column {
        "TP+" => -8.0,
        "TP-" => -7.0,
        "BP+" => 7.0,
        "BP-" => 8.0,
        _ => {
            let mut chars = column.chars();
            let letter = chars.next()?;
            if chars.next().is_some() {
                return None;
            }
            match letter {
                'A'..='E' => -5.0 + (letter as u8 - b'A') as f32,
                'F'..='J' => 1.0 + (letter as u8 - b'F') as f32,
                _ => return None,
            }
        }
    };
    Some(BOARD_CENTER_X + pitches * PITCH)
}

fn row_y(row: u32) -> f32 {
    TOP_ROW_Y - (row - 1) as f32 * PITCH
}

fn hole_position(id: &str) -> Option<Vec2> {
    let (column, row) = match id.split_once(':') {
        Some((rail, row)) => (rail, row),
        None => id.split_at_checked(1)?,
    };
    let row: u32 = row.parse().ok()?;
    if !(1..=30).contains(&row) || id.contains(':') != (column.len() == 3) {
        return None;
    }
    Some(Vec2::new(column_x(column)?, row_y(row)))
}

fn srgb(color: sprites::palette::Rgb) -> Color {
    Color::srgb_u8(color[0], color[1], color[2])
}

fn spawn_board(commands: &mut Commands, images: &mut Assets<Image>, bench: &Bench) {
    use sprites::palette;
    let width = 17.0 * PITCH + 34.0;
    rect(
        commands,
        Vec2::new(BOARD_CENTER_X, 20.0),
        Vec2::new(width + 18.0, 525.0),
        Color::srgb(0.16, 0.31, 0.28),
        0.0,
    );
    rect(
        commands,
        Vec2::new(BOARD_CENTER_X, 20.0),
        Vec2::new(width, 509.0),
        srgb(palette::BOARD),
        0.1,
    );
    rect(
        commands,
        Vec2::new(BOARD_CENTER_X, 20.0),
        Vec2::new(12.0, 480.0),
        srgb(palette::BOARD_GROOVE),
        0.2,
    );
    for rail in ["TP+", "TP-", "BP+", "BP-"] {
        let x = column_x(rail).unwrap();
        let color = srgb(if rail.ends_with('+') {
            palette::RAIL_RED
        } else {
            palette::RAIL_BLUE
        });
        // Marking line on the far side of each rail from its partner rail.
        let side = if rail.ends_with('+') { -1.0 } else { 1.0 };
        rect(
            commands,
            Vec2::new(x + side * 7.0, 20.0),
            Vec2::new(2.0, 480.0),
            color,
            0.22,
        );
        label(
            commands,
            if rail.ends_with('+') { "+" } else { "-" },
            Vec2::new(x, 278.0),
            19.0,
            color,
        );
    }
    for col in 'A'..='J' {
        label(
            commands,
            col.to_string(),
            Vec2::new(column_x(&col.to_string()).unwrap(), 278.0),
            12.0,
            Color::srgb(0.25, 0.26, 0.22),
        );
    }
    for row in 1..=30 {
        if row == 1 || row % 5 == 0 {
            label(
                commands,
                row.to_string(),
                Vec2::new(BOARD_CENTER_X - 1.0, row_y(row)),
                11.0,
                Color::srgb(0.25, 0.26, 0.22),
            );
        }
        let holes = ('A'..='J')
            .map(|col| format!("{col}{row}"))
            .chain(["TP+", "TP-", "BP+", "BP-"].map(|rail| format!("{rail}:{row}")));
        for id in holes {
            rect(
                commands,
                hole_position(&id).unwrap(),
                Vec2::splat(sprites::PIXEL * 2.0),
                srgb(palette::HOLE),
                0.4,
            );
        }
    }
    for wire in &bench.project.wires {
        let a = hole_position(&wire.from.0).unwrap();
        let b = hole_position(&wire.to.0).unwrap();
        line(commands, a, b, srgb(palette::OUTLINE), 8.0, 0.7);
        line(commands, a, b, srgb(palette::WIRE), 4.0, 0.75);
        for point in [a, b] {
            rect(
                commands,
                point,
                Vec2::splat(8.0),
                srgb(palette::OUTLINE),
                0.9,
            );
            rect(
                commands,
                point,
                Vec2::splat(4.0),
                srgb(palette::WIRE_SHADE),
                0.95,
            );
        }
    }
    for (index, component) in bench.project.components.iter().enumerate() {
        match sprites::art_for(component.kind) {
            Some(art) => spawn_part(commands, images, component, art, index),
            None => spawn_component(commands, component),
        }
    }
}

/// Spawns a pixel-art part with one pre-rendered image per visual state.
fn spawn_part(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    component: &Component,
    art: &dyn sprites::PartArt,
    index: usize,
) {
    let pins: Vec<Vec2> = component
        .pins
        .values()
        .map(|hole| hole_position(&hole.0).unwrap())
        .collect();
    let placed: Vec<_> = (0..art.state_count())
        .map(|state| sprites::place(art, component, &pins, state))
        .collect();
    let size = Vec2::new(
        placed[0].canvas.width() as f32,
        placed[0].canvas.height() as f32,
    ) * sprites::PIXEL;
    let center = pins[0] + placed[0].center_offset;
    let states: Vec<_> = placed
        .iter()
        .map(|p| images.add(p.canvas.to_image()))
        .collect();
    // LEDs sit on top so their halo is never hidden; the index keeps order stable.
    let layer = if component.kind == ComponentKind::Led {
        1.4
    } else {
        1.2
    };
    commands.spawn((
        Sprite {
            image: states[0].clone(),
            custom_size: Some(size),
            ..default()
        },
        Transform::from_translation(center.extend(layer + index as f32 * 0.001)),
        SceneEntity,
        PartVisual {
            id: component.id.clone(),
            kind: component.kind,
            states,
            shown: 0,
        },
    ));
}

/// Plain fallback for kinds without pixel art yet.
fn spawn_component(commands: &mut Commands, component: &Component) {
    if component.kind == ComponentKind::DcVoltageSource {
        let center = Vec2::new(
            (column_x("TP+").unwrap() + column_x("TP-").unwrap()) / 2.0,
            322.0,
        );
        for (pin, hole) in &component.pins {
            let start = center + Vec2::new(if pin.0 == "positive" { -10.0 } else { 10.0 }, -14.0);
            line(
                commands,
                start,
                hole_position(&hole.0).unwrap(),
                if pin.0 == "positive" {
                    Color::srgb(0.9, 0.20, 0.16)
                } else {
                    Color::srgb(0.20, 0.35, 0.85)
                },
                3.0,
                1.0,
            );
        }
        rect(
            commands,
            center,
            Vec2::new(70.0, 32.0),
            Color::srgb(0.21, 0.27, 0.30),
            1.1,
        );
        label(commands, "V1  5 V", center, 13.0, Color::WHITE);
        return;
    }
    let points: Vec<Vec2> = component
        .pins
        .values()
        .map(|hole| hole_position(&hole.0).unwrap())
        .collect();
    let center = points.iter().copied().sum::<Vec2>() / points.len() as f32;
    let color = match component.kind {
        ComponentKind::Capacitor => Color::srgb(0.35, 0.46, 0.56),
        ComponentKind::NpnTransistor => Color::srgb(0.11, 0.16, 0.19),
        ComponentKind::ChangeoverSwitch => Color::srgb(0.35, 0.36, 0.40),
        _ => Color::srgb(0.36, 0.42, 0.42),
    };
    for point in &points {
        line(
            commands,
            *point,
            center,
            Color::srgb(0.50, 0.54, 0.51),
            3.0,
            1.0,
        );
        rect(
            commands,
            *point,
            Vec2::splat(9.0),
            Color::srgb(0.85, 0.87, 0.79),
            1.1,
        );
    }
    rect(commands, center, Vec2::splat(27.0), color, 1.2);
    if component.kind == ComponentKind::Capacitor {
        for dy in [-6.0, 6.0] {
            rect(
                commands,
                center + Vec2::new(0.0, dy),
                Vec2::new(27.0, 3.0),
                Color::srgb(0.78, 0.83, 0.83),
                1.3,
            );
        }
    }
}

fn component_summary(component: &Component) -> String {
    let id = &component.id.0;
    match component.kind {
        ComponentKind::DcVoltageSource => format!("{id}  5 V supply  TP+:1 / TP-:1"),
        ComponentKind::Resistor => format!(
            "{id}  {:.0} ohm  {} / {}",
            component.parameters["resistance"],
            component.pins[&bredboard_core::PinId("a".into())].0,
            component.pins[&bredboard_core::PinId("b".into())].0
        ),
        ComponentKind::Led => format!(
            "{id}  LED  A {} / K {}",
            component.pins[&bredboard_core::PinId("anode".into())].0,
            component.pins[&bredboard_core::PinId("cathode".into())].0
        ),
        ComponentKind::Capacitor => format!(
            "{id}  {:.0} uF  + {} / - {}",
            component.parameters["capacitance"] * 1_000_000.0,
            component.pins[&bredboard_core::PinId("positive".into())].0,
            component.pins[&bredboard_core::PinId("negative".into())].0
        ),
        ComponentKind::MomentaryButton => format!(
            "{id}  button  {} / {}",
            component.pins[&bredboard_core::PinId("a".into())].0,
            component.pins[&bredboard_core::PinId("b".into())].0
        ),
        ComponentKind::ChangeoverSwitch => format!(
            "{id}  switch  NC {} C {} NO {}",
            component.pins[&bredboard_core::PinId("normally_closed".into())].0,
            component.pins[&bredboard_core::PinId("common".into())].0,
            component.pins[&bredboard_core::PinId("normally_open".into())].0
        ),
        ComponentKind::NpnTransistor => format!(
            "{id}  NPN  B {} C {} E {}",
            component.pins[&bredboard_core::PinId("base".into())].0,
            component.pins[&bredboard_core::PinId("collector".into())].0,
            component.pins[&bredboard_core::PinId("emitter".into())].0
        ),
    }
}

fn spawn_bench(commands: &mut Commands, images: &mut Assets<Image>, bench: &Bench) {
    rect(
        commands,
        Vec2::ZERO,
        Vec2::new(1200.0, 760.0),
        Color::srgb(0.10, 0.14, 0.16),
        -1.0,
    );
    spawn_board(commands, images, bench);
    label(
        commands,
        bench.circuit.label(),
        Vec2::new(185.0, 308.0),
        28.0,
        Color::srgb(0.84, 0.94, 0.93),
    );
    label(
        commands,
        "Hover a hole to inspect a wire or pin",
        Vec2::new(185.0, 264.0),
        16.0,
        Color::srgb(0.58, 0.76, 0.76),
    );
    label(
        commands,
        "BUILD LIST",
        Vec2::new(185.0, 232.0),
        17.0,
        Color::srgb(0.42, 0.90, 0.76),
    );
    for (index, component) in bench.project.components.iter().enumerate() {
        label(
            commands,
            component_summary(component),
            Vec2::new(185.0, 205.0 - index as f32 * 24.0),
            14.0,
            Color::srgb(0.76, 0.85, 0.82),
        );
    }
    let value = label(
        commands,
        "",
        Vec2::new(185.0, 28.0),
        25.0,
        Color::srgb(0.84, 0.95, 0.91),
    );
    commands.entity(value).insert(Readout::Value);
    let status = label(
        commands,
        "",
        Vec2::new(185.0, -18.0),
        17.0,
        Color::srgb(0.72, 0.82, 0.83),
    );
    commands.entity(status).insert(Readout::Status);
    let hover = label(
        commands,
        "",
        Vec2::new(185.0, -70.0),
        16.0,
        Color::srgb(0.65, 0.81, 0.85),
    );
    commands.entity(hover).insert(Readout::Hover);
    let control = label(
        commands,
        "",
        Vec2::new(185.0, -110.0),
        17.0,
        Color::srgb(0.75, 0.88, 0.89),
    );
    commands.entity(control).insert(Readout::Control);
    button(
        commands,
        "RUN / PAUSE",
        Vec2::new(35.0, -170.0),
        Vec2::new(180.0, 53.0),
        Control::RunPause,
    );
    button(
        commands,
        "RESET",
        Vec2::new(215.0, -170.0),
        Vec2::new(150.0, 53.0),
        Control::Reset,
    );
    button(
        commands,
        bench.circuit.control(),
        Vec2::new(205.0, -245.0),
        Vec2::new(400.0, 53.0),
        Control::Circuit,
    );
    button(
        commands,
        "CIRCUITS",
        Vec2::new(185.0, -328.0),
        Vec2::new(260.0, 45.0),
        Control::Back,
    );
}

fn clear_scene(commands: &mut Commands, entities: &Query<Entity, With<SceneEntity>>) {
    for entity in entities {
        commands.entity(entity).despawn();
    }
}

fn cursor_world(window: &Window) -> Option<Vec2> {
    let cursor = window.cursor_position()?;
    let width = window.width().max(1.0);
    let height = window.height().max(1.0);
    let world_width = 1200.0_f32.max(760.0 * width / height);
    let world_height = 760.0_f32.max(1200.0 * height / width);
    Some(Vec2::new(
        (cursor.x / width - 0.5) * world_width,
        (0.5 - cursor.y / height) * world_height,
    ))
}

fn handle_mouse(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    targets: Query<(&Transform, &ClickTarget)>,
    scene: Query<Entity, With<SceneEntity>>,
    mut commands: Commands,
    mut session: ResMut<Session>,
    mut images: ResMut<Assets<Image>>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(point) = cursor_world(window) else {
        return;
    };
    let selected = targets.iter().find_map(|(transform, target)| {
        ((point - transform.translation.truncate())
            .abs()
            .cmple(target.1 * 0.5)
            .all())
        .then_some(target.0)
    });
    match selected {
        Some(Control::Select(circuit)) => {
            clear_scene(&mut commands, &scene);
            let bench = Bench::new(circuit);
            spawn_bench(&mut commands, &mut images, &bench);
            session.bench = Some(bench);
        }
        Some(Control::Back) => {
            clear_scene(&mut commands, &scene);
            session.bench = None;
            spawn_menu(&mut commands);
        }
        Some(Control::RunPause) => {
            if let Some(bench) = &mut session.bench {
                bench.act(if bench.simulation.running {
                    Action::Pause
                } else {
                    Action::Run
                });
            }
        }
        Some(Control::Reset) => {
            if let Some(bench) = &mut session.bench {
                bench.act(Action::Reset);
            }
        }
        Some(Control::Circuit) => {
            if let Some(bench) = &mut session.bench {
                bench.toggle_control();
            }
        }
        None => {}
    }
}

fn handle_keyboard(keys: Res<ButtonInput<KeyCode>>, mut session: ResMut<Session>) {
    let Some(bench) = &mut session.bench else {
        return;
    };
    if keys.just_pressed(KeyCode::Space) {
        bench.act(if bench.simulation.running {
            Action::Pause
        } else {
            Action::Run
        });
    }
    if keys.just_pressed(KeyCode::KeyR) {
        bench.act(Action::Reset);
    }
    if keys.just_pressed(KeyCode::KeyC) {
        bench.toggle_control();
    }
}

fn fixed_step(mut session: ResMut<Session>) {
    if let Some(bench) = &mut session.bench
        && bench.simulation.running
    {
        advance_steps(&bench.project, &mut bench.simulation, 1);
    }
}

fn update_view(
    session: Res<Session>,
    windows: Query<&Window>,
    mut labels: Query<(&Readout, &mut Text2d, &mut TextColor)>,
    mut parts: Query<(&mut PartVisual, &mut Sprite)>,
) {
    let Some(bench) = &session.bench else {
        return;
    };
    let hover = windows
        .single()
        .ok()
        .and_then(cursor_world)
        .and_then(hole_at)
        .map(|id| {
            let mut contacts = Vec::new();
            for component in &bench.project.components {
                for (pin, hole) in &component.pins {
                    if hole.0 == id {
                        contacts.push(format!("{} {}", component.id.0, pin.0));
                    }
                }
            }
            for wire in &bench.project.wires {
                if wire.from.0 == id {
                    contacts.push(format!("{} to {}", wire.id.0, wire.to.0));
                } else if wire.to.0 == id {
                    contacts.push(format!("{} to {}", wire.id.0, wire.from.0));
                }
            }
            if contacts.is_empty() {
                id
            } else {
                format!("{id} - {}", contacts.join(", "))
            }
        })
        .unwrap_or_default();
    for (kind, mut text, mut color) in &mut labels {
        text.0 = match kind {
            Readout::Status => {
                if bench.simulation.stale {
                    color.0 = Color::srgb(1.0, 0.40, 0.32);
                    format!(
                        "CALCULATION FAILED: {}",
                        bench
                            .simulation
                            .diagnostics
                            .first()
                            .map_or("unknown", |d| d.message.as_str())
                    )
                } else {
                    color.0 = Color::srgb(0.72, 0.82, 0.83);
                    format!(
                        "{} - {:.3} s",
                        if bench.simulation.running {
                            "Running"
                        } else {
                            "Paused"
                        },
                        bench.simulation.time_seconds()
                    )
                }
            }
            Readout::Value => {
                if bench.simulation.stale {
                    "Readings stale".into()
                } else {
                    match bench.circuit {
                        Circuit::Rc => format!(
                            "C1  {:.3} V",
                            bench.simulation.capacitor_voltages[&ComponentId("C1".into())]
                        ),
                        _ => bench.simulation.last_valid.as_ref().map_or(
                            "LED current: run to measure".into(),
                            |result| {
                                format!(
                                    "D1  {:.2} mA",
                                    1000.0 * result.led_currents[&ComponentId("D1".into())]
                                )
                            },
                        ),
                    }
                }
            }
            Readout::Control => {
                let id = ComponentId(
                    if bench.circuit == Circuit::Rc {
                        "S1"
                    } else {
                        "B1"
                    }
                    .into(),
                );
                match bench.simulation.controls[&id] {
                    ControlState::SwitchNormallyClosed => "S1: charging path".into(),
                    ControlState::SwitchNormallyOpen => "S1: discharge path".into(),
                    ControlState::ButtonPressed => "B1: pressed".into(),
                    ControlState::ButtonReleased => "B1: released".into(),
                }
            }
            Readout::Hover => hover.clone(),
        };
    }
    let readings = bench
        .simulation
        .last_valid
        .as_ref()
        .filter(|_| !bench.simulation.stale);
    for (mut part, mut sprite) in &mut parts {
        let Some(art) = sprites::art_for(part.kind) else {
            continue;
        };
        let context = sprites::PartContext {
            led_current: readings
                .and_then(|r| r.led_currents.get(&part.id))
                .copied()
                .unwrap_or(0.0),
            control: bench.simulation.controls.get(&part.id).copied(),
        };
        let state = art.state(&context).min(part.states.len() - 1);
        if state != part.shown {
            sprite.image = part.states[state].clone();
            part.shown = state;
        }
    }
}

fn hole_at(point: Vec2) -> Option<String> {
    for row in 1..=30 {
        for col in 'A'..='J' {
            let id = format!("{col}{row}");
            if point.distance(hole_position(&id)?) <= 6.0 {
                return Some(id);
            }
        }
        for rail in ["TP+", "TP-", "BP+", "BP-"] {
            let id = format!("{rail}:{row}");
            if point.distance(hole_position(&id)?) <= 6.0 {
                return Some(id);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::{TimePlugin, TimeUpdateStrategy};
    use std::collections::BTreeSet;
    use std::time::Duration;

    fn click(app: &mut App, window: Entity, point: Vec2) {
        let mut window = app.world_mut().get_mut::<Window>(window).unwrap();
        let width = window.width();
        let height = window.height();
        let world_width = 1200.0_f32.max(760.0 * width / height);
        let world_height = 760.0_f32.max(1200.0 * height / width);
        window.set_cursor_position(Some(Vec2::new(
            width * (0.5 + point.x / world_width),
            height * (0.5 - point.y / world_height),
        )));
        let mut mouse = ButtonInput::<MouseButton>::default();
        mouse.press(MouseButton::Left);
        app.world_mut().insert_resource(mouse);
        app.update();
    }

    #[test]
    fn all_embedded_boards_have_unique_lead_and_wire_holes() {
        for circuit in Circuit::all() {
            let bench = Bench::new(circuit);
            let pins = bench
                .project
                .components
                .iter()
                .flat_map(|c| c.pins.values().map(|h| h.0.clone()));
            let endpoints = bench
                .project
                .wires
                .iter()
                .flat_map(|w| [&w.from.0, &w.to.0].map(Clone::clone));
            let all: Vec<_> = pins.chain(endpoints).collect();
            assert_eq!(
                all.len(),
                all.iter().collect::<BTreeSet<_>>().len(),
                "{circuit:?}"
            );
            assert!(all.iter().all(|hole| hole_position(hole).is_some()));
        }
    }

    #[test]
    fn each_circuit_uses_core_controls_and_reset() {
        for circuit in Circuit::all() {
            let mut bench = Bench::new(circuit);
            bench.toggle_control();
            bench.act(Action::Run);
            advance_steps(&bench.project, &mut bench.simulation, 100);
            assert_eq!(bench.simulation.step, 100, "{circuit:?}");
            assert!(!bench.simulation.stale);
            bench.act(Action::Reset);
            assert_eq!(bench.simulation.step, 0);
            assert!(!bench.simulation.running);
        }
    }

    #[test]
    fn rc_switch_charges_and_discharges_from_the_embedded_board() {
        let mut bench = Bench::new(Circuit::Rc);
        bench.act(Action::Run);
        advance_steps(&bench.project, &mut bench.simulation, 1_000);
        let charged = bench.simulation.capacitor_voltages[&ComponentId("C1".into())];
        assert!((charged - 5.0 * (1.0 - (-1.0_f64).exp())).abs() < 0.05);
        bench.toggle_control();
        advance_steps(&bench.project, &mut bench.simulation, 1_000);
        let discharged = bench.simulation.capacitor_voltages[&ComponentId("C1".into())];
        assert!((discharged - charged * (-1.0_f64).exp()).abs() < 0.05);
        bench.act(Action::Reset);
        assert_eq!(
            bench.simulation.capacitor_voltages[&ComponentId("C1".into())],
            0.0
        );
        assert_eq!(
            bench.simulation.controls[&ComponentId("S1".into())],
            ControlState::SwitchNormallyClosed
        );
    }

    #[test]
    fn shared_menu_and_visible_buttons_dispatch_core_actions() {
        let mut app = App::new();
        app.insert_resource(Session::default())
            .insert_resource(ButtonInput::<MouseButton>::default())
            .init_resource::<Assets<Image>>()
            .add_systems(Startup, setup)
            .add_systems(Update, handle_mouse);
        let window = app
            .world_mut()
            .spawn(Window {
                resolution: (1200, 760).into(),
                ..default()
            })
            .id();
        app.update();
        click(&mut app, window, Vec2::new(0.0, 100.0));
        assert_eq!(
            app.world()
                .resource::<Session>()
                .bench
                .as_ref()
                .unwrap()
                .circuit,
            Circuit::Led
        );
        click(&mut app, window, Vec2::new(205.0, -245.0));
        assert_eq!(
            app.world()
                .resource::<Session>()
                .bench
                .as_ref()
                .unwrap()
                .simulation
                .controls[&ComponentId("B1".into())],
            ControlState::ButtonPressed
        );
        click(&mut app, window, Vec2::new(35.0, -170.0));
        assert!(
            app.world()
                .resource::<Session>()
                .bench
                .as_ref()
                .unwrap()
                .simulation
                .running
        );
        click(&mut app, window, Vec2::new(215.0, -170.0));
        assert!(
            !app.world()
                .resource::<Session>()
                .bench
                .as_ref()
                .unwrap()
                .simulation
                .running
        );
        click(&mut app, window, Vec2::new(185.0, -328.0));
        assert!(app.world().resource::<Session>().bench.is_none());
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .resolution
            .set(828.0, 1054.0);
        click(&mut app, window, Vec2::new(0.0, -5.0));
        assert_eq!(
            app.world()
                .resource::<Session>()
                .bench
                .as_ref()
                .unwrap()
                .circuit,
            Circuit::Rc
        );
    }

    #[test]
    fn failure_state_marks_visible_readings_stale() {
        let mut app = App::new();
        app.insert_resource(Session {
            bench: Some(Bench::new(Circuit::Led)),
        })
        .add_systems(Update, update_view);
        app.world_mut().spawn(Window::default());
        app.world_mut()
            .spawn((Readout::Status, Text2d::default(), TextColor::default()));
        app.world_mut()
            .spawn((Readout::Value, Text2d::default(), TextColor::default()));
        {
            let mut bench = app.world_mut().resource_mut::<Session>();
            let state = &mut bench.bench.as_mut().unwrap().simulation;
            state.stale = true;
            state
                .diagnostics
                .push(bredboard_core::SimulationDiagnostic {
                    code: "nonconvergence".into(),
                    path: String::new(),
                    message: "nonlinear circuit did not converge within 80 iterations".into(),
                });
        }
        app.update();
        let mut readouts = app.world_mut().query::<(&Readout, &Text2d)>();
        let values: Vec<_> = readouts
            .iter(app.world())
            .map(|(kind, value)| (std::mem::discriminant(kind), value.0.clone()))
            .collect();
        assert!(values.iter().any(|(_, value)| value == "Readings stale"));
        assert!(
            values
                .iter()
                .any(|(_, value)| value.contains("CALCULATION FAILED"))
        );
    }

    #[test]
    fn part_sprites_follow_core_led_current_and_button_state() {
        let mut app = App::new();
        app.insert_resource(Session {
            bench: Some(Bench::new(Circuit::Led)),
        })
        .init_resource::<Assets<Image>>()
        .add_systems(Update, update_view);
        app.world_mut().spawn(Window::default());
        app.world_mut()
            .resource_scope(|world, mut images: Mut<Assets<Image>>| {
                let session = world.resource::<Session>();
                let bench = session.bench.as_ref().unwrap();
                let mut queue = bevy::ecs::world::CommandQueue::default();
                let mut commands = Commands::new(&mut queue, world);
                spawn_bench(&mut commands, &mut images, bench);
                queue.apply(world);
            });
        let shown = |app: &mut App, id: &str| {
            let mut parts = app.world_mut().query::<&PartVisual>();
            parts
                .iter(app.world())
                .find(|p| p.id.0 == id)
                .map(|p| (p.shown, p.states.len()))
                .unwrap()
        };
        app.update();
        assert_eq!(shown(&mut app, "D1"), (0, 3));
        assert_eq!(shown(&mut app, "B1"), (0, 2));
        assert_eq!(shown(&mut app, "R1"), (0, 1));
        {
            let mut session = app.world_mut().resource_mut::<Session>();
            let bench = session.bench.as_mut().unwrap();
            bench.toggle_control();
            bench.act(Action::Run);
            advance_steps(&bench.project, &mut bench.simulation, 10);
        }
        app.update();
        assert_eq!(shown(&mut app, "D1").0, 2, "8.6 mA lights the LED");
        assert_eq!(shown(&mut app, "B1").0, 1);
        app.world_mut()
            .resource_mut::<Session>()
            .bench
            .as_mut()
            .unwrap()
            .simulation
            .stale = true;
        app.update();
        assert_eq!(
            shown(&mut app, "D1").0,
            0,
            "stale readings do not light the LED"
        );
    }

    #[test]
    fn different_frame_pacing_keeps_fixed_rc_results() {
        fn run_frames(frames: &[u64]) -> SimulationState {
            let mut bench = Bench::new(Circuit::Rc);
            bench.act(Action::Run);
            let mut app = App::new();
            app.add_plugins(TimePlugin)
                .insert_resource(Time::<Virtual>::from_max_delta(Duration::MAX))
                .insert_resource(Time::<Fixed>::from_hz(1_000.0))
                .insert_resource(Session { bench: Some(bench) })
                .add_systems(FixedUpdate, fixed_step);
            app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
            app.update();
            for milliseconds in frames {
                app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
                    *milliseconds,
                )));
                app.update();
            }
            app.world()
                .resource::<Session>()
                .bench
                .as_ref()
                .unwrap()
                .simulation
                .clone()
        }
        let single = run_frames(&[1_000]);
        let split = run_frames(&[100, 400, 500]);
        assert_eq!(single.step, 1_000);
        assert_eq!(single, split);
    }
}
