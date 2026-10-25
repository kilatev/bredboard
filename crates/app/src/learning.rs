//! A small authored beginner path over the existing editor and core fixtures.
use super::*;
use crate::text::learning as copy;
use bevy::asset::RenderAssetUsages;
use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    Schematic,
    Board,
    Ready,
}
#[derive(Clone, Copy)]
pub enum Command {
    Open(Circuit),
    View(View),
    Guide,
    Hint,
    PlaceNext,
}
pub struct Lesson {
    pub reference: Project,
    pub view: View,
    pub guided: bool,
}
enum Step<'a> {
    Part(&'a Component),
    Wire(&'a bredboard_core::Wire),
}
impl Step<'_> {
    fn holes(&self) -> Vec<&bredboard_core::HoleId> {
        match self {
            Self::Part(p) => p.pins.values().collect(),
            Self::Wire(w) => vec![&w.from, &w.to],
        }
    }
    fn instruction(&self) -> String {
        match self {
            Self::Part(p) => format!(
                "{}: {}. {}",
                component_summary(p),
                p.pins
                    .iter()
                    .map(|(p, h)| format!("{} -> {}", p.0, h.0))
                    .collect::<Vec<_>>()
                    .join("; "),
                match p.kind {
                    ComponentKind::Led => copy::LED_PIN,
                    ComponentKind::Capacitor => copy::CAP_PIN,
                    ComponentKind::NpnTransistor => copy::NPN_PIN,
                    ComponentKind::MomentaryButton => copy::BUTTON_PIN,
                    ComponentKind::Resistor => copy::RESISTOR_PIN,
                    _ => "",
                }
            ),
            Self::Wire(w) => format!("Wire {} -> {}", w.from.0, w.to.0),
        }
    }
}
fn next_step<'a>(lesson: &'a Lesson, project: &Project) -> Option<Step<'a>> {
    for p in &lesson.reference.components {
        if p.kind != ComponentKind::DcVoltageSource
            && !project
                .components
                .iter()
                .any(|c| c.kind == p.kind && c.pins == p.pins && c.parameters == p.parameters)
        {
            return Some(Step::Part(p));
        }
    }
    lesson
        .reference
        .wires
        .iter()
        .find(|w| {
            !project
                .wires
                .iter()
                .any(|c| (c.from == w.from && c.to == w.to) || (c.from == w.to && c.to == w.from))
        })
        .map(Step::Wire)
}
pub fn next_tool(bench: &Bench) -> Option<assembly::Tool> {
    let l = bench.learning.as_ref().filter(|l| l.guided)?;
    Some(match next_step(l, &bench.project) {
        Some(Step::Part(p)) => assembly::Tool::Place(p.kind),
        Some(Step::Wire(_)) => assembly::Tool::Wire,
        None => assembly::Tool::Select,
    })
}
pub fn open(circuit: Circuit) -> Bench {
    assert!(matches!(
        circuit,
        Circuit::Led | Circuit::Rc | Circuit::Transistor
    ));
    let mut bench = assembly::blank();
    bench.circuit = circuit;
    bench.learning = Some(Lesson {
        reference: Bench::new(circuit).project,
        view: if circuit == Circuit::Led {
            View::Ready
        } else {
            View::Schematic
        },
        guided: circuit == Circuit::Led,
    });
    if let Some(tool) = next_tool(&bench) {
        assembly::command(&mut bench, assembly::Command::Tool(tool));
    }
    bench
}
pub fn is_board(bench: &Bench) -> bool {
    bench
        .learning
        .as_ref()
        .is_none_or(|l| l.view == View::Board)
}
pub fn is_building(bench: &Bench) -> bool {
    bench
        .learning
        .as_ref()
        .is_some_and(|l| l.guided && next_step(l, &bench.project).is_some())
}
pub fn reference_template(bench: &Bench, kind: ComponentKind) -> Option<Component> {
    let l = bench.learning.as_ref().filter(|l| l.guided)?;
    match next_step(l, &bench.project)? {
        Step::Part(p) if p.kind == kind => Some(p.clone()),
        _ => None,
    }
}
pub fn command(bench: &mut Bench, command: Command) {
    match command {
        Command::Open(circuit) => *bench = open(circuit),
        Command::View(view) => {
            if let Some(l) = &mut bench.learning {
                l.view = view;
            }
        }
        Command::Guide => {
            if let Some(l) = &mut bench.learning {
                l.guided = !l.guided;
            }
            if let Some(tool) = next_tool(bench) {
                assembly::command(bench, assembly::Command::Tool(tool));
            }
        }
        Command::Hint | Command::PlaceNext => {
            let Some(l) = bench.learning.as_mut() else {
                return;
            };
            l.guided = true;
            let Some(step) = next_step(l, &bench.project) else {
                return;
            };
            if matches!(command, Command::PlaceNext) {
                let mut candidate = bench.project.clone();
                match step {
                    Step::Part(part) => candidate.components.push(part.clone()),
                    Step::Wire(wire) => candidate.wires.push(wire.clone()),
                }
                assembly::assisted_edit(bench, candidate);
            } else {
                let tool = match step {
                    Step::Part(part) => assembly::Tool::Place(part.kind),
                    Step::Wire(_) => assembly::Tool::Wire,
                };
                assembly::command(bench, assembly::Command::Tool(tool));
            }
        }
    }
}
fn paragraph(
    commands: &mut Commands,
    text: impl Into<String>,
    point: Vec2,
    width: f32,
    height: f32,
    size: f32,
) {
    let e = wrapped_label(commands, text, point, size, Color::srgb(0.79, 0.88, 0.85));
    commands.entity(e).insert(TextBounds::new(width, height));
}
fn schematic(circuit: Circuit) -> &'static [u8] {
    match circuit {
        Circuit::Led => include_bytes!("../assets/learning/led.png"),
        Circuit::Rc => include_bytes!("../assets/learning/rc.png"),
        Circuit::Transistor => include_bytes!("../assets/learning/transistor.png"),
        _ => unreachable!("only introductory lessons have authored diagrams"),
    }
}
fn title(circuit: Circuit) -> &'static str {
    match circuit {
        Circuit::Led => copy::LIGHT,
        Circuit::Rc => copy::CAPACITOR,
        _ => copy::TRANSISTOR,
    }
}
pub fn draw(commands: &mut Commands, images: &mut Assets<Image>, bench: &Bench) {
    let l = bench.learning.as_ref().unwrap();
    if l.view == View::Board {
        assembly::draw(commands, images, bench);
        if l.guided {
            let next = next_step(l, &bench.project);
            let message = next.as_ref().map_or_else(
                || {
                    if bench.circuit == Circuit::Rc {
                        copy::RC_DONE.into()
                    } else {
                        copy::LED_DONE.into()
                    }
                },
                Step::instruction,
            );
            paragraph(
                commands,
                message,
                Vec2::new(-550.0, -260.0),
                325.0,
                65.0,
                14.0,
            );
            if let Some(step) = next {
                for h in step.holes() {
                    if let Some(p) = hole_position(&h.0) {
                        rect(
                            commands,
                            p,
                            Vec2::splat(13.0),
                            Color::srgb(1.0, 0.78, 0.23),
                            1.8,
                        );
                        rect(
                            commands,
                            p,
                            Vec2::splat(5.0),
                            Color::srgb(0.12, 0.16, 0.15),
                            1.9,
                        );
                    }
                }
            }
            for (x, label, cmd) in [
                (-475.0, copy::HINT, Command::Hint),
                (-300.0, copy::PLACE_NEXT, Command::PlaceNext),
            ] {
                button(
                    commands,
                    label,
                    Vec2::new(x, -350.0),
                    Vec2::new(160.0, 34.0),
                    Control::Learning(cmd),
                );
            }
        } else {
            paragraph(
                commands,
                copy::OWN_LAYOUT,
                Vec2::new(-550.0, -265.0),
                325.0,
                70.0,
                14.0,
            );
        }
        button(
            commands,
            if l.guided {
                copy::GUIDE_ON
            } else {
                copy::GUIDE_OFF
            },
            Vec2::new(-260.0, 350.0),
            Vec2::new(230.0, 34.0),
            Control::Learning(Command::Guide),
        );
    } else {
        rect(
            commands,
            Vec2::ZERO,
            Vec2::new(1200.0, 760.0),
            Color::srgb(0.10, 0.14, 0.16),
            -1.0,
        );
        if l.view == View::Schematic {
            let image = Image::from_buffer(
                schematic(bench.circuit),
                ImageType::Extension("png"),
                CompressedImageFormats::NONE,
                true,
                ImageSampler::linear(),
                RenderAssetUsages::default(),
            )
            .expect("embedded introductory schematic PNG");
            commands.spawn((
                Sprite {
                    image: images.add(image),
                    custom_size: Some(Vec2::new(880.0, 550.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 0.0),
                SceneEntity,
            ));
        } else {
            let mut reference = Bench::new(bench.circuit);
            reference.act(Action::SingleStep);
            spawn_board(commands, images, &reference);
            let parts = l
                .reference
                .components
                .iter()
                .map(|p| {
                    format!(
                        "{}\n{}",
                        component_summary(p),
                        p.pins
                            .iter()
                            .map(|(p, h)| format!("{}: {}", p.0, h.0))
                            .collect::<Vec<_>>()
                            .join("  ")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n\n");
            let wires = l
                .reference
                .wires
                .iter()
                .map(|w| format!("{}: {} -> {}", w.id.0, w.from.0, w.to.0))
                .collect::<Vec<_>>()
                .join("\n");
            paragraph(
                commands,
                format!("{}\n\n{}\n\n{}", copy::READ_ONLY, parts, wires),
                Vec2::new(-25.0, 235.0),
                600.0,
                510.0,
                14.0,
            );
        }
        if l.view == View::Ready {
            paragraph(
                commands,
                copy::BOARD_TIP,
                Vec2::new(-550.0, -265.0),
                325.0,
                65.0,
                14.0,
            );
            paragraph(
                commands,
                copy::HARDWARE,
                Vec2::new(-25.0, -270.0),
                600.0,
                60.0,
                13.0,
            );
        } else {
            paragraph(
                commands,
                copy::HARDWARE,
                Vec2::new(-550.0, -285.0),
                1100.0,
                55.0,
                15.0,
            );
        }
        button(
            commands,
            copy::BUILD,
            Vec2::new(0.0, -350.0),
            Vec2::new(300.0, 38.0),
            Control::Learning(Command::View(View::Board)),
        );
        button(
            commands,
            copy::LIBRARY,
            Vec2::new(-440.0, -350.0),
            Vec2::new(230.0, 38.0),
            Control::Back,
        );
    }
    // Tabs only change presentation; the learner's Project/state/history stay put.
    for (x, label, view) in [
        (15.0, copy::SCHEMATIC, View::Schematic),
        (235.0, copy::MY_BOARD, View::Board),
        (455.0, copy::READY, View::Ready),
    ] {
        button(
            commands,
            label,
            Vec2::new(x, 330.0),
            Vec2::new(210.0, 36.0),
            Control::Learning(Command::View(view)),
        );
    }
    if l.view != View::Board {
        label(
            commands,
            title(bench.circuit),
            Vec2::new(if l.view == View::Ready { 185.0 } else { 0.0 }, 285.0),
            if l.view == View::Ready { 18.0 } else { 21.0 },
            Color::srgb(0.42, 0.90, 0.76),
        );
    }
    if l.view != View::Board {
        let next = match bench.circuit {
            Circuit::Led => Circuit::Rc,
            Circuit::Rc => Circuit::Transistor,
            _ => Circuit::Led,
        };
        button(
            commands,
            copy::NEXT_LESSON,
            Vec2::new(-260.0, 350.0),
            Vec2::new(230.0, 34.0),
            Control::Learning(Command::Open(next)),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    fn assemble(bench: &mut Bench) {
        for _ in 0..32 {
            if next_step(bench.learning.as_ref().unwrap(), &bench.project).is_none() {
                return;
            }
            command(bench, Command::PlaceNext);
        }
        panic!("introductory guide did not finish within its bounded step count");
    }
    #[test]
    fn all_three_guides_build_real_valid_circuits_and_assets_decode() {
        for circuit in [Circuit::Led, Circuit::Rc, Circuit::Transistor] {
            let mut bench = open(circuit);
            assemble(&mut bench);
            let reference = &bench.learning.as_ref().unwrap().reference;
            assert_eq!(bench.project.components, reference.components);
            assert_eq!(bench.project.wires, reference.wires);
            compile_topology(&bench.project).unwrap();
            if circuit != Circuit::Rc {
                bench.act(Action::SetControl {
                    component: ComponentId("B1".into()),
                    state: ControlState::ButtonPressed,
                });
            }
            bench.act(Action::Run);
            advance_steps(&bench.project, &mut bench.simulation, 100);
            assert!(
                !bench.simulation.stale,
                "{circuit:?}: {:?}",
                bench.simulation.diagnostics
            );
            let result = bench.simulation.last_valid.as_ref().unwrap();
            if circuit == Circuit::Rc {
                assert!(result.capacitor_voltages[&ComponentId("C1".into())] > 0.0);
            } else {
                assert!(result.led_currents[&ComponentId("D1".into())] > 0.001);
            }
            let image = Image::from_buffer(
                schematic(circuit),
                ImageType::Extension("png"),
                CompressedImageFormats::NONE,
                true,
                ImageSampler::linear(),
                RenderAssetUsages::default(),
            )
            .unwrap();
            assert_eq!(image.size(), UVec2::new(800, 500));
        }
    }
    #[test]
    fn guide_advances_tools_so_highlighted_contacts_always_place_the_right_part() {
        let mut bench = open(Circuit::Led);
        for h in [
            "A5", "A8", "B8", "A10", "E10", "F10", "TP+:5", "B5", "G10", "TP-:10",
        ] {
            assembly::click(&mut bench, hole_position(h).unwrap());
        }
        assert_eq!(
            bench.project.components,
            bench.learning.as_ref().unwrap().reference.components
        );
        assert_eq!(
            bench.project.wires,
            bench.learning.as_ref().unwrap().reference.wires
        );
        assert_eq!(
            bench.editor.as_ref().unwrap().tool,
            Some(assembly::Tool::Select)
        );
    }

    #[test]
    fn guided_manual_placement_uses_the_lessons_actual_nominal() {
        let mut bench = open(Circuit::Rc);
        command(&mut bench, Command::Hint);
        for hole in ["A5", "A8"] {
            assembly::click(&mut bench, hole_position(hole).unwrap());
        }
        let r = bench
            .project
            .components
            .iter()
            .find(|p| p.id.0 == "R1")
            .unwrap();
        assert_eq!(r.parameters["resistance"], 1000.0);
        assert_eq!(bench.project.components.len(), 2);
        let before = bench.project.clone();
        command(&mut bench, Command::View(View::Ready));
        command(&mut bench, Command::View(View::Schematic));
        command(&mut bench, Command::View(View::Board));
        assert_eq!(bench.project, before);
    }

    #[test]
    fn a_different_working_layout_is_allowed_and_help_does_not_overwrite_it() {
        let mut bench = open(Circuit::Led);
        assemble(&mut bench);
        let mut candidate = bench.project.clone();
        for c in &mut candidate.components {
            if c.kind != ComponentKind::DcVoltageSource {
                for hole in c.pins.values_mut() {
                    let p = hole.0.find(|ch: char| ch.is_ascii_digit()).unwrap();
                    hole.0 = format!(
                        "{}{}",
                        &hole.0[..p],
                        hole.0[p..].parse::<u32>().unwrap() + 10
                    );
                }
            }
        }
        for wire in &mut candidate.wires {
            for hole in [&mut wire.from, &mut wire.to] {
                let p = hole.0.find(|ch: char| ch.is_ascii_digit()).unwrap();
                hole.0 = format!(
                    "{}{}",
                    &hole.0[..p],
                    hole.0[p..].parse::<u32>().unwrap() + 10
                );
            }
        }
        assembly::assisted_edit(&mut bench, candidate.clone());
        command(&mut bench, Command::Guide); // independent layout, guide off
        bench.act(Action::SetControl {
            component: ComponentId("B1".into()),
            state: ControlState::ButtonPressed,
        });
        bench.act(Action::SingleStep);
        assert!(
            bench.simulation.last_valid.as_ref().unwrap().led_currents[&ComponentId("D1".into())]
                > 0.001
        );
        let project = bench.project.clone();
        let simulation = bench.simulation.clone();
        command(&mut bench, Command::PlaceNext); // duplicate ID or occupied hole is rejected atomically
        assert_eq!(bench.project, project);
        assert_eq!(bench.simulation, simulation);
    }
    proptest! {
        #![proptest_config(ProptestConfig { cases:32,rng_seed:proptest::test_runner::RngSeed::Fixed(20261025),..ProptestConfig::default() })]
        #[test]
        fn preview_preserves_project_simulation_and_undo(views in prop::collection::vec(0u8..3,1..40)) {
            let mut bench=open(Circuit::Led);
            let empty=bench.project.clone();
            command(&mut bench,Command::PlaceNext);
            let project=bench.project.clone(); let state=bench.simulation.clone();
            for view in views {
                command(&mut bench,Command::View(match view {0=>View::Board,1=>View::Ready,_=>View::Schematic}));
                prop_assert_eq!(&bench.project,&project);
                prop_assert_eq!(&bench.simulation,&state);
            }
            assembly::command(&mut bench,assembly::Command::Undo);
            prop_assert_eq!(bench.project,empty);
        }
        #[test]
        fn assisted_edits_remain_valid_and_undoable(steps in 1usize..14) {
            let mut bench=open(Circuit::Transistor);
            let empty=bench.project.clone(); let mut history=Vec::new();
            for _ in 0..steps {
                if next_step(bench.learning.as_ref().unwrap(),&bench.project).is_none() {break;}
                history.push(bench.project.clone());
                command(&mut bench,Command::PlaceNext);
                prop_assert!(compile_topology(&bench.project).is_ok());
            }
            for expected in history.into_iter().rev() {
                assembly::command(&mut bench,assembly::Command::Undo);
                prop_assert_eq!(&bench.project,&expected);
            }
            prop_assert_eq!(bench.project,empty);
        }
    }
}
