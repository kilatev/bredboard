mod sprites;
mod text;

use bevy::camera::ScalingMode;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bredboard_core::{
    Action, Component, ComponentId, ComponentKind, ControlState, Project, SimulationState,
    advance_steps, apply_actions, compile_topology,
};

const LED_JSON: &str = include_str!("../../../fixtures/projects/led-bench.json");
const RC_JSON: &str = include_str!("../../../fixtures/projects/rc-bench.json");
const TRANSISTOR_JSON: &str = include_str!("../../../fixtures/projects/transistor-bench.json");
const E1_JSON: &str = include_str!("../../../fixtures/projects/e1-first-light.json");
const E2_JSON: &str = include_str!("../../../fixtures/projects/e2-push-button-switch.json");
const E3_JSON: &str = include_str!("../../../fixtures/projects/e3-two-leds-in-series.json");
const E4_JSON: &str = include_str!("../../../fixtures/projects/e4-two-leds-in-parallel.json");
const E7_JSON: &str = include_str!("../../../fixtures/projects/e7-buzzer-doorbell.json");
const E8_JSON: &str = include_str!("../../../fixtures/projects/e8-transistor-switch.json");
const E9_JSON: &str = include_str!("../../../fixtures/projects/e9-logical-and.json");
const E10_JSON: &str = include_str!("../../../fixtures/projects/e10-smooth-fade.json");
const E5_JSON: &str = include_str!("../../../fixtures/projects/e5-brightness-dial.json");
const E6_JSON: &str = include_str!("../../../fixtures/projects/e6-light-reactive-led.json");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Circuit {
    Led,
    Rc,
    Transistor,
    E1,
    E2,
    E3,
    E4,
    E7,
    E8,
    E9,
    E10,
    E5,
    E6,
}

/// One bench control button: its label, the component it drives, and
/// whether that component is a changeover switch (vs. a momentary button).
struct ControlSpec {
    label: &'static str,
    component: &'static str,
    is_switch: bool,
}
/// A continuous dial/slider control: its label and the potentiometer or
/// photoresistor component it drives.
struct DialSpec {
    label: &'static str,
    component: &'static str,
}
impl Circuit {
    fn all() -> [Self; 13] {
        [
            Self::Led,
            Self::Rc,
            Self::Transistor,
            Self::E1,
            Self::E2,
            Self::E3,
            Self::E4,
            Self::E7,
            Self::E8,
            Self::E9,
            Self::E10,
            Self::E5,
            Self::E6,
        ]
    }
    fn json(self) -> &'static str {
        match self {
            Self::Led => LED_JSON,
            Self::Rc => RC_JSON,
            Self::Transistor => TRANSISTOR_JSON,
            Self::E1 => E1_JSON,
            Self::E2 => E2_JSON,
            Self::E3 => E3_JSON,
            Self::E4 => E4_JSON,
            Self::E7 => E7_JSON,
            Self::E8 => E8_JSON,
            Self::E9 => E9_JSON,
            Self::E10 => E10_JSON,
            Self::E5 => E5_JSON,
            Self::E6 => E6_JSON,
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Led => "LED + RESISTOR",
            Self::Rc => "CAPACITOR CHARGE / DISCHARGE",
            Self::Transistor => "TRANSISTOR SWITCH",
            Self::E1 => "E1: FIRST LIGHT",
            Self::E2 => "E2: PUSH-BUTTON SWITCH",
            Self::E3 => "E3: TWO LEDS IN SERIES",
            Self::E4 => "E4: TWO LEDS IN PARALLEL",
            Self::E7 => "E7: BUZZER DOORBELL",
            Self::E8 => "E8: TRANSISTOR SWITCH",
            Self::E9 => "E9: LOGICAL AND",
            Self::E10 => "E10: SMOOTH FADE",
            Self::E5 => "E5: BRIGHTNESS DIAL",
            Self::E6 => "E6: LIGHT-REACTIVE LED",
        }
    }
    /// One-paragraph explanation and a short player task, shown together in
    /// the bench's secondary text slot. Empty for the three MVP benches,
    /// which keep their original generic hover hint instead (see
    /// `spawn_bench`); this task does not change their behavior or layout.
    fn explanation_and_task(self) -> (&'static str, &'static str) {
        match self {
            Self::Led | Self::Rc | Self::Transistor => ("", ""),
            Self::E1 => (
                "A resistor limits current from the 5 V supply so the LED lights safely and stays lit.",
                "Task: run the circuit and confirm the LED lights immediately, with no switch needed.",
            ),
            Self::E2 => (
                "The same LED and resistor, now gated by a momentary button: current only flows, and the LED lights, while the button is held.",
                "Task: press and hold the button, then release it and confirm the LED goes dark.",
            ),
            Self::E3 => (
                "Two LEDs share one resistor and one current path, so the same current lights both LEDs together.",
                "Task: run the circuit and confirm both LEDs light at once.",
            ),
            Self::E4 => (
                "Two independent resistor-and-LED branches share the same 5 V supply, so each LED gets its own steady current regardless of the other.",
                "Task: run the circuit and confirm both LEDs light independently of each other.",
            ),
            Self::E7 => (
                "A momentary button connects the buzzer directly to the 5 V supply, so it draws current, and its sprite shows sounding, only while the button is held.",
                "Task: press and hold the button and confirm the buzzer's sound-wave marks appear, then disappear on release.",
            ),
            Self::E8 => (
                "A small button current through a base resistor turns the transistor on, which then switches a much larger LED current between its collector and emitter.",
                "Task: press and hold the button and confirm the LED lights, then goes dark on release.",
            ),
            Self::E9 => (
                "Two momentary buttons wired in series both need to be pressed to complete the path to the resistor and LED, the same as a logical AND gate.",
                "Task: confirm the LED lights only when both buttons are held at once, not when either is held alone.",
            ),
            Self::E10 => (
                "Holding the button charges a capacitor through a resistor; the LED, wired to the same charging node, brightens smoothly as the capacitor's voltage rises instead of snapping on.",
                "Task: press and hold the button and watch the LED brighten gradually rather than instantly.",
            ),
            Self::E5 => (
                "A potentiometer's resistance is set by the dial ratio instead of a fixed value, so it shares the current-limiting job with a resistor and changes the LED's brightness continuously.",
                "Task: drag the dial across its full range and confirm the LED's brightness changes continuously and monotonically.",
            ),
            Self::E6 => (
                "A photoresistor's resistance falls as its ambient-light control rises, so more simulated light means less resistance and a brighter LED.",
                "Task: drag the ambient-light slider across its full range and confirm the LED dims continuously toward off at the darkest setting.",
            ),
        }
    }
    /// Control buttons for this bench, in display order. Empty for exercises
    /// with no live control (E1, E3, E4: always-on circuits).
    fn controls(self) -> &'static [ControlSpec] {
        const RC: &[ControlSpec] = &[ControlSpec {
            label: "S1: CHANGE PATH",
            component: "S1",
            is_switch: true,
        }];
        const B1_BUTTON: &[ControlSpec] = &[ControlSpec {
            label: "B1: PRESS / RELEASE",
            component: "B1",
            is_switch: false,
        }];
        const S1_BUTTON: &[ControlSpec] = &[ControlSpec {
            label: "S1: PRESS / RELEASE",
            component: "S1",
            is_switch: false,
        }];
        const E9_BUTTONS: &[ControlSpec] = &[
            ControlSpec {
                label: "S1: PRESS / RELEASE",
                component: "S1",
                is_switch: false,
            },
            ControlSpec {
                label: "S2: PRESS / RELEASE",
                component: "S2",
                is_switch: false,
            },
        ];
        match self {
            Self::Rc => RC,
            Self::Led | Self::Transistor | Self::E2 | Self::E7 | Self::E8 | Self::E10 => {
                if matches!(self, Self::Led | Self::Transistor) {
                    B1_BUTTON
                } else {
                    S1_BUTTON
                }
            }
            Self::E9 => E9_BUTTONS,
            Self::E1 | Self::E3 | Self::E4 | Self::E5 | Self::E6 => &[],
        }
    }
    /// The continuous dial/slider control for E5/E6's potentiometer or
    /// photoresistor, shown instead of a button row. `None` for every other
    /// bench.
    fn dial(self) -> Option<DialSpec> {
        match self {
            Self::E5 => Some(DialSpec {
                label: "RV1: BRIGHTNESS DIAL - drag left/right",
                component: "RV1",
            }),
            Self::E6 => Some(DialSpec {
                label: "RV1: AMBIENT LIGHT - drag left/right",
                component: "RV1",
            }),
            _ => None,
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
    /// Toggles the `index`-th control button for this bench (see
    /// `Circuit::controls`); out-of-range indices are ignored.
    fn toggle(&mut self, index: usize) {
        let Some(spec) = self.circuit.controls().get(index) else {
            return;
        };
        let id = ComponentId(spec.component.into());
        let state = if spec.is_switch {
            if self.simulation.controls[&id] == ControlState::SwitchNormallyClosed {
                ControlState::SwitchNormallyOpen
            } else {
                ControlState::SwitchNormallyClosed
            }
        } else if self.simulation.controls[&id] == ControlState::ButtonReleased {
            ControlState::ButtonPressed
        } else {
            ControlState::ButtonReleased
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

/// Vertical scroll offset for the menu list, in world units. Positive values
/// reveal later entries by shifting the list up. Presentation-only; never
/// read by `bredboard-core` or routed through an `Action`.
#[derive(Resource, Default)]
struct MenuScroll {
    offset: f32,
}

/// Top/bottom of the menu's scrollable viewport in world space, and the
/// vertical spacing between entries. The header (title/subtitle) sits above
/// `MENU_TOP`; the footer hint sits below `MENU_BOTTOM`.
const MENU_TOP: f32 = 155.0;
const MENU_BOTTOM: f32 = -220.0;
const MENU_ENTRY_HEIGHT: f32 = 105.0;
const MENU_ENTRY_SIZE: Vec2 = Vec2::new(520.0, 74.0);

/// World-space Y of a menu entry at rest (scroll offset zero). Index 0 is the
/// first entry; matches the original fixed 3-entry layout exactly.
fn menu_entry_base_y(index: usize) -> f32 {
    100.0 - index as f32 * MENU_ENTRY_HEIGHT
}

/// Largest scroll offset that still keeps the last entry's bottom edge at or
/// above `MENU_BOTTOM`; zero when every entry already fits in the viewport.
fn menu_max_scroll(entry_count: usize) -> f32 {
    if entry_count == 0 {
        return 0.0;
    }
    let content_top = menu_entry_base_y(0) + MENU_ENTRY_SIZE.y * 0.5;
    let content_bottom = menu_entry_base_y(entry_count - 1) - MENU_ENTRY_SIZE.y * 0.5;
    let content_height = content_top - content_bottom;
    (content_height - (MENU_TOP - MENU_BOTTOM)).max(0.0)
}

#[derive(Component)]
struct SceneEntity;
/// Marks an entity as one of the N scrollable menu entries, keyed by its
/// index in `Circuit::all()`. A single index is shared by an entry's button
/// rect and its label so both move and hide together.
#[derive(Component, Clone, Copy)]
struct MenuEntry(usize);
#[derive(Component, Clone, Copy)]
struct ClickTarget(Control, Vec2);
/// The draggable track for E5/E6's continuous dial/slider control.
#[derive(Component)]
struct DialTrack {
    component: ComponentId,
}
/// The handle sprite that shows a dial's current ratio; repositioned every
/// frame from `SimulationState.control_ratios`.
#[derive(Component)]
struct DialHandle {
    component: ComponentId,
}
const DIAL_TRACK_CENTER: Vec2 = Vec2::new(205.0, -245.0);
const DIAL_TRACK_SIZE: Vec2 = Vec2::new(400.0, 53.0);
const DIAL_HANDLE_SIZE: Vec2 = Vec2::new(14.0, 53.0);
#[derive(Clone, Copy)]
enum Control {
    Select(Circuit),
    Back,
    RunPause,
    Reset,
    Toggle(usize),
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
        .insert_resource(MenuScroll::default())
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, fixed_step)
        .add_systems(
            Update,
            (
                handle_scroll,
                scroll_menu,
                handle_mouse,
                handle_keyboard,
                handle_dial,
                update_dial_handle,
                update_view,
            )
                .chain(),
        )
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

/// A scrollable menu entry: a selectable button tagged with `MenuEntry(index)`
/// on both its rect and label so `scroll_menu` can move and hide them together.
fn menu_entry_button(commands: &mut Commands, value: &str, index: usize, control: Control) {
    let point = Vec2::new(0.0, menu_entry_base_y(index));
    let rect_entity = rect(
        commands,
        point,
        MENU_ENTRY_SIZE,
        Color::srgb(0.13, 0.30, 0.35),
        1.0,
    );
    commands
        .entity(rect_entity)
        .insert((ClickTarget(control, MENU_ENTRY_SIZE), MenuEntry(index)));
    let label_entity = label(commands, value, point, 21.0, Color::srgb(0.94, 0.97, 0.96));
    commands.entity(label_entity).insert(MenuEntry(index));
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
        menu_entry_button(commands, circuit.label(), index, Control::Select(circuit));
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

/// Draws a wire the same way everywhere: a dark outline, a colored core, and
/// rounded end caps where it plugs into a hole.
fn draw_wire(commands: &mut Commands, from: Vec2, to: Vec2, color: Color, shade: Color) {
    let outline = srgb(sprites::palette::OUTLINE);
    line(commands, from, to, outline, 8.0, 0.7);
    line(commands, from, to, color, 4.0, 0.75);
    for point in [from, to] {
        rect(commands, point, Vec2::splat(8.0), outline, 0.9);
        rect(commands, point, Vec2::splat(4.0), shade, 0.95);
    }
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
        draw_wire(
            commands,
            a,
            b,
            srgb(palette::WIRE),
            srgb(palette::WIRE_SHADE),
        );
    }
    for (index, component) in bench.project.components.iter().enumerate() {
        if component.kind == ComponentKind::DcVoltageSource {
            spawn_source(commands, images, component);
            continue;
        }
        let art = sprites::art_for(component.kind).expect("every component kind has sprite art");
        spawn_part(commands, images, component, art, index);
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

/// Off-board 5 V supply: fixed position above the board, colored supply
/// wires to each pin's hole, and a pixel-art body sprite (see `sprites::source`).
fn spawn_source(commands: &mut Commands, images: &mut Assets<Image>, component: &Component) {
    let center = Vec2::new(
        (column_x("TP+").unwrap() + column_x("TP-").unwrap()) / 2.0,
        322.0,
    );
    for (pin, hole) in &component.pins {
        let start = center + Vec2::new(if pin.0 == "positive" { -10.0 } else { 10.0 }, -14.0);
        let (color, shade) = if pin.0 == "positive" {
            (
                srgb(sprites::palette::RAIL_RED),
                srgb(sprites::palette::WIRE_RED_SHADE),
            )
        } else {
            (
                srgb(sprites::palette::RAIL_BLUE),
                srgb(sprites::palette::WIRE_BLUE_SHADE),
            )
        };
        draw_wire(commands, start, hole_position(&hole.0).unwrap(), color, shade);
    }
    let art = sprites::art_for(component.kind).expect("dc_voltage_source has sprite art");
    let body = art.body(component, 0);
    let size = Vec2::new(body.width() as f32, body.height() as f32) * sprites::PIXEL;
    commands.spawn((
        Sprite {
            image: images.add(body.to_image()),
            custom_size: Some(size),
            ..default()
        },
        Transform::from_translation(center.extend(1.1)),
        SceneEntity,
    ));
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
        ComponentKind::Potentiometer => format!(
            "{id}  potentiometer {:.0}-{:.0} ohm  {} / {}",
            component.parameters["min_resistance"],
            component.parameters["max_resistance"],
            component.pins[&bredboard_core::PinId("a".into())].0,
            component.pins[&bredboard_core::PinId("b".into())].0
        ),
        ComponentKind::Photoresistor => format!(
            "{id}  photoresistor {:.0}-{:.0} ohm  {} / {}",
            component.parameters["min_resistance"],
            component.parameters["max_resistance"],
            component.pins[&bredboard_core::PinId("a".into())].0,
            component.pins[&bredboard_core::PinId("b".into())].0
        ),
        ComponentKind::Buzzer => format!(
            "{id}  buzzer {:.0} ohm  + {} / - {}",
            component.parameters["resistance"],
            component.pins[&bredboard_core::PinId("positive".into())].0,
            component.pins[&bredboard_core::PinId("negative".into())].0
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
    let (explanation, task) = bench.circuit.explanation_and_task();
    let secondary = if explanation.is_empty() {
        "Hover a hole to inspect a wire or pin".to_string()
    } else {
        format!("{explanation}\n{task}")
    };
    label(
        commands,
        secondary,
        Vec2::new(185.0, 264.0),
        14.0,
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
    // Control buttons share one row so the window layout never depends on
    // how many controls a bench has (at most 2, for E9's two buttons); RESET
    // and CIRCUITS always stay at their original fixed positions.
    let controls = bench.circuit.controls();
    if controls.len() == 1 {
        button(
            commands,
            controls[0].label,
            Vec2::new(205.0, -245.0),
            Vec2::new(400.0, 53.0),
            Control::Toggle(0),
        );
    } else {
        let width = 400.0 / controls.len().max(1) as f32 - 10.0;
        for (index, spec) in controls.iter().enumerate() {
            let x = 5.0 + (width + 10.0) * (index as f32 + 0.5);
            button(
                commands,
                spec.label,
                Vec2::new(x, -245.0),
                Vec2::new(width, 53.0),
                Control::Toggle(index),
            );
        }
    }
    if let Some(spec) = bench.circuit.dial() {
        let component = ComponentId(spec.component.into());
        let track = rect(
            commands,
            DIAL_TRACK_CENTER,
            DIAL_TRACK_SIZE,
            Color::srgb(0.10, 0.22, 0.25),
            1.0,
        );
        commands.entity(track).insert(DialTrack {
            component: component.clone(),
        });
        label(
            commands,
            spec.label,
            DIAL_TRACK_CENTER + Vec2::new(0.0, 20.0),
            13.0,
            Color::srgb(0.75, 0.88, 0.89),
        );
        let ratio = bench
            .simulation
            .control_ratios
            .get(&component)
            .copied()
            .unwrap_or(0.5);
        let handle_x =
            DIAL_TRACK_CENTER.x - DIAL_TRACK_SIZE.x / 2.0 + ratio as f32 * DIAL_TRACK_SIZE.x;
        let handle = rect(
            commands,
            Vec2::new(handle_x, DIAL_TRACK_CENTER.y),
            DIAL_HANDLE_SIZE,
            Color::srgb(0.42, 0.90, 0.76),
            1.1,
        );
        commands.entity(handle).insert(DialHandle { component });
    }
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

/// Mouse-wheel-driven vertical scroll of the menu list, clamped so it cannot
/// scroll past the first or last entry. Presentation input only.
fn handle_scroll(
    mut wheel: MessageReader<MouseWheel>,
    mut scroll: ResMut<MenuScroll>,
    entries: Query<&MenuEntry>,
) {
    let entry_count = entries.iter().map(|entry| entry.0 + 1).max().unwrap_or(0);
    if entry_count == 0 {
        wheel.clear();
        return;
    }
    let max_scroll = menu_max_scroll(entry_count);
    for event in wheel.read() {
        let delta = match event.unit {
            MouseScrollUnit::Line => event.y * 40.0,
            MouseScrollUnit::Pixel => event.y,
        };
        scroll.offset = (scroll.offset - delta).clamp(0.0, max_scroll);
    }
}

/// Moves each menu entry to its scrolled position and hides entries that
/// fall outside the menu's visible viewport so they cannot overlap the
/// header or footer, and are not clickable while off-screen.
fn scroll_menu(
    scroll: Res<MenuScroll>,
    mut entries: Query<(&MenuEntry, &mut Transform, &mut Visibility)>,
) {
    let half_height = MENU_ENTRY_SIZE.y * 0.5;
    for (entry, mut transform, mut visibility) in &mut entries {
        let y = menu_entry_base_y(entry.0) + scroll.offset;
        transform.translation.y = y;
        *visibility = if y + half_height < MENU_BOTTOM || y - half_height > MENU_TOP {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
}

/// Bundles the two menu/bench presentation resources so `handle_mouse` stays
/// under Clippy's argument-count limit.
#[derive(bevy::ecs::system::SystemParam)]
struct MenuState<'w> {
    session: ResMut<'w, Session>,
    scroll: ResMut<'w, MenuScroll>,
}

/// Drag-driven continuous control: while the left mouse button is held over
/// a dial's track, sets the driven component's control ratio from the
/// cursor's horizontal position. Presentation input, routed through the same
/// `Action::SetControlRatio` path as any other caller.
fn handle_dial(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    tracks: Query<&DialTrack>,
    mut session: ResMut<Session>,
) {
    if !mouse.pressed(MouseButton::Left) {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(point) = cursor_world(window) else {
        return;
    };
    let Some(track) = tracks.iter().next() else {
        return;
    };
    let half = DIAL_TRACK_SIZE * 0.5;
    if (point - DIAL_TRACK_CENTER).abs().cmpgt(half).any() {
        return;
    }
    let ratio = ((point.x - (DIAL_TRACK_CENTER.x - half.x)) / DIAL_TRACK_SIZE.x) as f64;
    if let Some(bench) = &mut session.bench {
        bench.act(Action::SetControlRatio {
            component: track.component.clone(),
            ratio: ratio.clamp(0.0, 1.0),
        });
    }
}

/// Repositions a dial's handle sprite from the live control ratio each frame.
fn update_dial_handle(session: Res<Session>, mut handles: Query<(&DialHandle, &mut Transform)>) {
    let Some(bench) = &session.bench else {
        return;
    };
    for (handle, mut transform) in &mut handles {
        let ratio = bench
            .simulation
            .control_ratios
            .get(&handle.component)
            .copied()
            .unwrap_or(0.5);
        transform.translation.x =
            DIAL_TRACK_CENTER.x - DIAL_TRACK_SIZE.x / 2.0 + ratio as f32 * DIAL_TRACK_SIZE.x;
    }
}

fn handle_mouse(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    targets: Query<(&Transform, &ClickTarget, Option<&Visibility>)>,
    scene: Query<Entity, With<SceneEntity>>,
    mut commands: Commands,
    mut state: MenuState,
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
    let selected = targets.iter().find_map(|(transform, target, visibility)| {
        if visibility == Some(&Visibility::Hidden) {
            return None;
        }
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
            state.session.bench = Some(bench);
        }
        Some(Control::Back) => {
            clear_scene(&mut commands, &scene);
            state.session.bench = None;
            state.scroll.offset = 0.0;
            spawn_menu(&mut commands);
        }
        Some(Control::RunPause) => {
            if let Some(bench) = &mut state.session.bench {
                bench.act(if bench.simulation.running {
                    Action::Pause
                } else {
                    Action::Run
                });
            }
        }
        Some(Control::Reset) => {
            if let Some(bench) = &mut state.session.bench {
                bench.act(Action::Reset);
            }
        }
        Some(Control::Toggle(index)) => {
            if let Some(bench) = &mut state.session.bench {
                bench.toggle(index);
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
        bench.toggle(0);
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
                } else if bench.circuit == Circuit::Rc {
                    format!(
                        "C1  {:.3} V",
                        bench.simulation.capacitor_voltages[&ComponentId("C1".into())]
                    )
                } else if bench.circuit == Circuit::E7 {
                    bench.simulation.last_valid.as_ref().map_or(
                        "Buzzer current: run to measure".into(),
                        |result| {
                            format!(
                                "BZ1  {:.2} mA",
                                1000.0
                                    * result
                                        .resistor_currents
                                        .get(&ComponentId("BZ1".into()))
                                        .copied()
                                        .unwrap_or(0.0)
                            )
                        },
                    )
                } else {
                    bench.simulation.last_valid.as_ref().map_or(
                        "LED current: run to measure".into(),
                        |result| {
                            let extra = if bench.circuit == Circuit::E10 {
                                format!(
                                    "  C1 {:.3} V",
                                    bench.simulation.capacitor_voltages[&ComponentId("C1".into())]
                                )
                            } else {
                                String::new()
                            };
                            format!(
                                "D1  {:.2} mA{extra}",
                                1000.0
                                    * result
                                        .led_currents
                                        .get(&ComponentId("D1".into()))
                                        .copied()
                                        .unwrap_or(0.0)
                            )
                        },
                    )
                }
            }
            Readout::Control => bench
                .circuit
                .controls()
                .iter()
                .map(|spec| {
                    let id = ComponentId(spec.component.into());
                    let state = bench.simulation.controls[&id];
                    let text = match state {
                        ControlState::SwitchNormallyClosed => "charging path",
                        ControlState::SwitchNormallyOpen => "discharge path",
                        ControlState::ButtonPressed => "pressed",
                        ControlState::ButtonReleased => "released",
                    };
                    format!("{}: {text}", spec.component)
                })
                .collect::<Vec<_>>()
                .join("   "),
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
            buzzer_current: readings
                .and_then(|r| r.resistor_currents.get(&part.id))
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
            bench.toggle(0);
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
        bench.toggle(0);
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
    fn all_circuits_are_selectable_via_the_scrolled_menu() {
        let mut app = App::new();
        app.insert_resource(Session::default())
            .insert_resource(MenuScroll::default())
            .insert_resource(ButtonInput::<MouseButton>::default())
            .init_resource::<Assets<Image>>()
            .add_systems(Startup, setup)
            .add_systems(Update, (scroll_menu, handle_mouse).chain());
        let window = app
            .world_mut()
            .spawn(Window {
                resolution: (1200, 760).into(),
                ..default()
            })
            .id();
        app.update();
        let entries = Circuit::all();
        let max_scroll = menu_max_scroll(entries.len());
        for (index, circuit) in entries.into_iter().enumerate() {
            let offset = (menu_entry_base_y(0) - menu_entry_base_y(index)).clamp(0.0, max_scroll);
            app.world_mut().resource_mut::<MenuScroll>().offset = offset;
            let y = menu_entry_base_y(index) + offset;
            click(&mut app, window, Vec2::new(0.0, y));
            assert_eq!(
                app.world()
                    .resource::<Session>()
                    .bench
                    .as_ref()
                    .unwrap()
                    .circuit,
                circuit,
                "index {index}"
            );
            // Return to the menu for the next iteration; the Back button
            // moves down when a bench has more than one control button.
            // Return to the menu for the next iteration.
            click(&mut app, window, Vec2::new(185.0, -328.0));
        }
    }

    #[test]
    fn dragging_the_dial_sets_the_control_ratio_and_moves_its_handle() {
        for circuit in [Circuit::E5, Circuit::E6] {
            let mut app = App::new();
            app.insert_resource(Session {
                bench: Some(Bench::new(circuit)),
            })
            .insert_resource(ButtonInput::<MouseButton>::default())
            .init_resource::<Assets<Image>>()
            .add_systems(Update, (handle_dial, update_dial_handle).chain());
            let window = app
                .world_mut()
                .spawn(Window {
                    resolution: (1200, 760).into(),
                    ..default()
                })
                .id();
            app.world_mut()
                .resource_scope(|world, mut images: Mut<Assets<Image>>| {
                    let session = world.resource::<Session>();
                    let bench = session.bench.as_ref().unwrap();
                    let mut queue = bevy::ecs::world::CommandQueue::default();
                    let mut commands = Commands::new(&mut queue, world);
                    spawn_bench(&mut commands, &mut images, bench);
                    queue.apply(world);
                });

            let drag = |window: Entity, point: Vec2, app: &mut App| {
                let mut w = app.world_mut().get_mut::<Window>(window).unwrap();
                let width = w.width();
                let height = w.height();
                let world_width = 1200.0_f32.max(760.0 * width / height);
                let world_height = 760.0_f32.max(1200.0 * height / width);
                w.set_cursor_position(Some(Vec2::new(
                    width * (0.5 + point.x / world_width),
                    height * (0.5 - point.y / world_height),
                )));
                let mut mouse = ButtonInput::<MouseButton>::default();
                mouse.press(MouseButton::Left);
                app.world_mut().insert_resource(mouse);
                app.update();
            };

            let id = ComponentId("RV1".into());
            let left_x = DIAL_TRACK_CENTER.x - DIAL_TRACK_SIZE.x / 2.0 + 2.0;
            drag(window, Vec2::new(left_x, DIAL_TRACK_CENTER.y), &mut app);
            let low_ratio = app
                .world()
                .resource::<Session>()
                .bench
                .as_ref()
                .unwrap()
                .simulation
                .control_ratios[&id];
            assert!(
                low_ratio < 0.05,
                "{circuit:?} ratio should be near 0.0: {low_ratio}"
            );

            let right_x = DIAL_TRACK_CENTER.x + DIAL_TRACK_SIZE.x / 2.0 - 2.0;
            drag(window, Vec2::new(right_x, DIAL_TRACK_CENTER.y), &mut app);
            let high_ratio = app
                .world()
                .resource::<Session>()
                .bench
                .as_ref()
                .unwrap()
                .simulation
                .control_ratios[&id];
            assert!(
                high_ratio > 0.95,
                "{circuit:?} ratio should be near 1.0: {high_ratio}"
            );

            let mut handles = app.world_mut().query::<(&DialHandle, &Transform)>();
            let (_, transform) = handles.iter(app.world()).next().unwrap();
            let expected_x = DIAL_TRACK_CENTER.x - DIAL_TRACK_SIZE.x / 2.0
                + high_ratio as f32 * DIAL_TRACK_SIZE.x;
            assert!(
                (transform.translation.x - expected_x).abs() < 1.0,
                "{circuit:?} handle should track the ratio"
            );
        }
    }

    #[test]
    fn new_exercises_show_title_explanation_and_task_text() {
        for circuit in [
            Circuit::E1,
            Circuit::E2,
            Circuit::E3,
            Circuit::E4,
            Circuit::E7,
            Circuit::E8,
            Circuit::E9,
            Circuit::E10,
            Circuit::E5,
            Circuit::E6,
        ] {
            let mut app = App::new();
            app.init_resource::<Assets<Image>>();
            let bench = Bench::new(circuit);
            app.world_mut()
                .resource_scope(|world, mut images: Mut<Assets<Image>>| {
                    let mut queue = bevy::ecs::world::CommandQueue::default();
                    let mut commands = Commands::new(&mut queue, world);
                    spawn_bench(&mut commands, &mut images, &bench);
                    queue.apply(world);
                });
            let mut texts = app.world_mut().query::<&Text2d>();
            let all_text: String = texts
                .iter(app.world())
                .map(|t| t.0.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            assert!(all_text.contains(circuit.label()), "{circuit:?} title");
            let (explanation, task) = circuit.explanation_and_task();
            assert!(!explanation.is_empty(), "{circuit:?} explanation");
            assert!(
                all_text.contains(explanation),
                "{circuit:?} explanation shown"
            );
            assert!(all_text.contains(task), "{circuit:?} task shown");
            for component in &bench.project.components {
                assert!(
                    all_text.contains(&component.id.0),
                    "{circuit:?} parts list missing {}",
                    component.id.0
                );
            }
        }
    }

    #[test]
    fn shared_menu_and_visible_buttons_dispatch_core_actions() {
        let mut app = App::new();
        app.insert_resource(Session::default())
            .insert_resource(MenuScroll::default())
            .insert_resource(ButtonInput::<MouseButton>::default())
            .init_resource::<Assets<Image>>()
            .add_systems(Startup, setup)
            .add_systems(Update, (scroll_menu, handle_mouse).chain());
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
            bench.toggle(0);
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

    /// Debug-only stand-in for the eventual 13-entry list (T22/T23): spawns
    /// `count` menu entries so scrolling can be exercised without depending
    /// on real fixed circuits.
    fn spawn_stub_menu(commands: &mut Commands, count: usize) {
        for index in 0..count {
            menu_entry_button(
                commands,
                &format!("STUB {index}"),
                index,
                Control::Select(Circuit::Led),
            );
        }
    }

    fn visible_entries(app: &mut App) -> BTreeSet<usize> {
        let mut query = app
            .world_mut()
            .query_filtered::<(&MenuEntry, &Visibility), With<ClickTarget>>();
        query
            .iter(app.world())
            .filter(|(_, visibility)| **visibility != Visibility::Hidden)
            .map(|(entry, _)| entry.0)
            .collect()
    }

    #[test]
    fn thirteen_stub_entries_are_all_reachable_by_scrolling_without_overlap() {
        let mut app = App::new();
        app.insert_resource(MenuScroll::default())
            .init_resource::<Assets<Image>>()
            .add_systems(Update, scroll_menu);
        app.world_mut()
            .resource_scope(|world, mut _images: Mut<Assets<Image>>| {
                let mut queue = bevy::ecs::world::CommandQueue::default();
                let mut commands = Commands::new(&mut queue, world);
                spawn_stub_menu(&mut commands, 13);
                queue.apply(world);
            });
        app.update();
        let mut seen = BTreeSet::new();
        seen.extend(visible_entries(&mut app));
        let max_scroll = menu_max_scroll(13);
        assert!(max_scroll > 0.0, "13 entries must overflow the viewport");
        let steps = 20;
        for step in 0..=steps {
            app.world_mut().resource_mut::<MenuScroll>().offset =
                max_scroll * step as f32 / steps as f32;
            app.update();
            seen.extend(visible_entries(&mut app));
        }
        assert_eq!(seen, (0..13).collect::<BTreeSet<_>>());

        // No two visible entries may overlap: their base Y spacing already
        // exceeds their height, so overlap-freedom reduces to checking every
        // visible entry sits within the declared viewport band.
        let half_height = MENU_ENTRY_SIZE.y * 0.5;
        let mut transforms = app
            .world_mut()
            .query::<(&MenuEntry, &Transform, &Visibility)>();
        for (_, transform, visibility) in transforms.iter(app.world()) {
            if *visibility == Visibility::Hidden {
                continue;
            }
            let y = transform.translation.y;
            assert!(y + half_height >= MENU_BOTTOM && y - half_height <= MENU_TOP);
        }
    }

    #[test]
    fn scroll_offset_is_clamped_to_first_and_last_entry() {
        let mut app = App::new();
        app.insert_resource(MenuScroll::default())
            .init_resource::<Assets<Image>>()
            .add_message::<MouseWheel>()
            .add_systems(Update, handle_scroll);
        app.world_mut()
            .resource_scope(|world, mut _images: Mut<Assets<Image>>| {
                let mut queue = bevy::ecs::world::CommandQueue::default();
                let mut commands = Commands::new(&mut queue, world);
                spawn_stub_menu(&mut commands, 13);
                queue.apply(world);
            });
        let max_scroll = menu_max_scroll(13);
        // A huge downward scroll must clamp at the last entry, not overshoot.
        app.world_mut().write_message(MouseWheel {
            unit: MouseScrollUnit::Pixel,
            x: 0.0,
            y: -10_000.0,
            window: Entity::PLACEHOLDER,
            phase: bevy::input::touch::TouchPhase::Moved,
        });
        app.update();
        assert_eq!(app.world().resource::<MenuScroll>().offset, max_scroll);
        // A huge upward scroll must clamp back at the first entry.
        app.world_mut().write_message(MouseWheel {
            unit: MouseScrollUnit::Pixel,
            x: 0.0,
            y: 10_000.0,
            window: Entity::PLACEHOLDER,
            phase: bevy::input::touch::TouchPhase::Moved,
        });
        app.update();
        assert_eq!(app.world().resource::<MenuScroll>().offset, 0.0);
    }

    #[test]
    fn existing_three_entry_menu_layout_is_unchanged() {
        assert_eq!(menu_max_scroll(3), 0.0);
        assert_eq!(menu_entry_base_y(0), 100.0);
        assert_eq!(menu_entry_base_y(1), -5.0);
        assert_eq!(menu_entry_base_y(2), -110.0);
    }
}
