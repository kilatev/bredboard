//! 8-bit component sprites drawn in code on an 8 px hole pitch.
//!
//! Each supported component kind implements [`PartArt`] in its own module and is
//! registered in [`art_for`]. Leads are not part of a body sprite: [`place`]
//! draws them from each pin's hole to the body, so any valid hole layout works.
//! Design reference: `docs/design/sprites/README.md`.

mod button;
mod buzzer;
mod canvas;
mod capacitor;
mod diode;
#[cfg(test)]
mod future;
mod ic;
mod led;
mod motor;
pub mod palette;
mod photoresistor;
mod piezo_passive;
mod potentiometer;
mod probe;
mod relay;
mod resistor;
mod source;
mod speaker;
mod switch;
mod transistor;

use bevy::math::{IVec2, Vec2};
use bredboard_core::{Component, ComponentKind, ControlState, PinId};

pub use canvas::PixelCanvas;

/// World units per art pixel. One hole pitch is 8 art px = 16 world units.
pub const PIXEL: f32 = 2.0;

/// Presentation inputs a part may react to; values come from core state.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PartContext {
    /// Calculated LED current in amperes; 0 when readings are stale.
    pub led_current: f64,
    /// Calculated current in amperes for a buzzer or speaker; 0 when
    /// readings are stale. Shared by both kinds: they are electrically
    /// identical fixed resistive loads with the same sounding threshold.
    pub buzzer_current: f64,
    /// Calculated signed motor speed in RPM; 0 when readings are stale.
    pub motor_speed: f64,
    /// Core-detected oscillating drive for a passive piezo.
    pub passive_piezo_sounding: bool,
    /// Calculated relay coil state.
    pub relay_energized: bool,
    pub control: Option<ControlState>,
}

/// Pixel art for one component kind.
pub trait PartArt: Sync {
    /// Pins defining the body axis. Unrotated bodies are drawn with the first
    /// pin on the left and the second on the right.
    fn axis_pins(&self) -> [&'static str; 2];

    /// Number of visual states; bodies are pre-rendered for each.
    fn state_count(&self) -> usize {
        1
    }

    /// Visual state index for the current presentation context.
    fn state(&self, _context: &PartContext) -> usize {
        0
    }

    /// Unrotated body sprite. Every state must have the same size.
    fn body(&self, component: &Component, state: usize) -> PixelCanvas;
}

/// Registry of kinds with sprites. Kinds returning `None` use the plain fallback.
pub fn art_for(kind: ComponentKind) -> Option<&'static dyn PartArt> {
    match kind {
        ComponentKind::Resistor => Some(&resistor::Resistor),
        ComponentKind::Led => Some(&led::Led),
        ComponentKind::Diode => Some(&diode::Diode),
        ComponentKind::MomentaryButton => Some(&button::Button),
        ComponentKind::Capacitor => Some(&capacitor::Capacitor),
        ComponentKind::NpnTransistor => Some(&transistor::Transistor),
        ComponentKind::PnpTransistor => Some(&transistor::Transistor),
        ComponentKind::ChangeoverSwitch => Some(&switch::Switch),
        ComponentKind::DcVoltageSource => Some(&source::Source),
        ComponentKind::Potentiometer => Some(&potentiometer::Potentiometer),
        ComponentKind::Photoresistor => Some(&photoresistor::Photoresistor),
        ComponentKind::Thermistor => Some(&photoresistor::Photoresistor),
        ComponentKind::TouchPad => Some(&probe::TouchPad),
        ComponentKind::WaterProbe => Some(&probe::WaterProbe),
        ComponentKind::Buzzer => Some(&buzzer::Buzzer),
        ComponentKind::Speaker => Some(&speaker::Speaker),
        ComponentKind::PiezoPassive => Some(&piezo_passive::PiezoPassive),
        ComponentKind::Motor => Some(&motor::Motor),
        ComponentKind::Relay => Some(&relay::Relay),
        ComponentKind::LogicGate
        | ComponentKind::SchmittInverter
        | ComponentKind::Comparator
        | ComponentKind::Timer555
        | ComponentKind::DFlipFlop
        | ComponentKind::DigitalCounter
        | ComponentKind::ClockSource
        | ComponentKind::ShiftRegister
        | ComponentKind::SevenSegmentDisplay
        | ComponentKind::FourBitAdder
        | ComponentKind::BargraphDisplay
        | ComponentKind::AudioAmplifier
        | ComponentKind::StepSequencer
        | ComponentKind::Sram
        | ComponentKind::Optocoupler
        | ComponentKind::IcDevice
        | ComponentKind::Module
        | ComponentKind::Other => Some(&ic::IntegratedCircuit),
    }
}

/// A composed placement: leads plus body for one visual state.
#[derive(Debug)]
pub struct Placed {
    pub canvas: PixelCanvas,
    /// World-space offset of the canvas centre from the anchor (first pin's hole).
    pub center_offset: Vec2,
}

/// Composes one state of a component placed on holes at `pins` (world positions
/// of hole centres, in the component's pin map order).
pub fn place(art: &dyn PartArt, component: &Component, pins: &[Vec2], state: usize) -> Placed {
    let anchor = pins[0];
    // Hole centre on the art grid, y down; a pin's 2x2 lead starts one px up-left.
    let to_art = |p: Vec2| {
        let d = (p - anchor) / PIXEL;
        IVec2::new(d.x.round() as i32, -d.y.round() as i32)
    };
    let tops: Vec<IVec2> = pins.iter().map(|p| to_art(*p) - IVec2::ONE).collect();
    let axis = art.axis_pins().map(|name| {
        component
            .pins
            .keys()
            .position(|pin| *pin == PinId(name.into()))
            .map_or(IVec2::ZERO, |i| tops[i])
    });
    let body = art
        .body(component, state)
        .rotated(quarter_turns(axis[1] - axis[0]));

    let sum = tops.iter().copied().sum::<IVec2>();
    let count = tops.len() as i32;
    let target = IVec2::new(sum.x.div_euclid(count), sum.y.div_euclid(count));
    let centre = sum.as_vec2() / count as f32 + Vec2::ONE;
    let body_at = IVec2::new(
        (centre.x - body.width() as f32 / 2.0).round() as i32,
        (centre.y - body.height() as f32 / 2.0).round() as i32,
    );

    let body_end = body_at + IVec2::new(body.width(), body.height());
    let min = tops.iter().fold(body_at, |m, t| m.min(*t)) - IVec2::ONE;
    let max = tops
        .iter()
        .fold(body_end, |m, t| m.max(*t + IVec2::splat(2)))
        + IVec2::ONE;
    let size = max - min;
    let mut canvas = PixelCanvas::new(size.x, size.y);
    for top in &tops {
        canvas.lead(*top - min, target - min);
    }
    canvas.blit(&body, body_at.x - min.x, body_at.y - min.y);

    let half = size.as_vec2() / 2.0;
    let origin = (-min).as_vec2();
    Placed {
        canvas,
        center_offset: Vec2::new(half.x - origin.x, origin.y - half.y) * PIXEL,
    }
}

/// Clockwise quarter turns that map the unrotated left-to-right axis onto `axis` (y down).
fn quarter_turns(axis: IVec2) -> u8 {
    if axis.x.abs() >= axis.y.abs() {
        if axis.x >= 0 { 0 } else { 2 }
    } else if axis.y > 0 {
        1
    } else {
        3
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bredboard_core::{HoleId, Project};
    use proptest::prelude::*;
    use std::collections::BTreeMap;

    const GOLDEN_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/design/sprites");

    fn golden(name: &str, canvas: &PixelCanvas) {
        let path = format!("{GOLDEN_DIR}/{name}.txt");
        let actual = canvas.to_ascii();
        if std::env::var_os("BREDBOARD_BLESS_SPRITES").is_some() {
            std::fs::write(&path, &actual).unwrap();
        }
        let expected = std::fs::read_to_string(&path).unwrap_or_default();
        assert_eq!(
            actual, expected,
            "{name}: run with BREDBOARD_BLESS_SPRITES=1 to accept"
        );
    }

    fn component(
        kind: ComponentKind,
        pins: &[(&str, &str)],
        parameters: &[(&str, f64)],
    ) -> Component {
        Component {
            id: bredboard_core::ComponentId("X1".into()),
            kind,
            pins: pins
                .iter()
                .map(|(p, h)| (PinId((*p).into()), HoleId((*h).into())))
                .collect(),
            parameters: parameters
                .iter()
                .map(|(k, v)| ((*k).into(), *v))
                .collect::<BTreeMap<_, _>>(),
            ic_device: None,
            other_device: None,
            module: None,
            diode_model: None,
        }
    }

    #[test]
    fn body_sprites_match_design_references() {
        let resistor = component(
            ComponentKind::Resistor,
            &[("a", "A1"), ("b", "A4")],
            &[("resistance", 330.0)],
        );
        let led = component(
            ComponentKind::Led,
            &[("anode", "A1"), ("cathode", "A2")],
            &[],
        );
        let button = component(
            ComponentKind::MomentaryButton,
            &[("a", "E1"), ("b", "F1")],
            &[],
        );
        let capacitor = component(
            ComponentKind::Capacitor,
            &[("positive", "A1"), ("negative", "A3")],
            &[("capacitance", 1e-4)],
        );
        let transistor = component(
            ComponentKind::NpnTransistor,
            &[("base", "F12"), ("collector", "F13"), ("emitter", "F14")],
            &[],
        );
        let switch = component(
            ComponentKind::ChangeoverSwitch,
            &[
                ("normally_closed", "F8"),
                ("common", "F9"),
                ("normally_open", "F10"),
            ],
            &[],
        );
        let source = component(
            ComponentKind::DcVoltageSource,
            &[("positive", "TP+:1"), ("negative", "TP-:1")],
            &[("voltage", 5.0)],
        );
        golden("resistor-330", &resistor::Resistor.body(&resistor, 0));
        for (state, name) in ["off", "dim", "on"].iter().enumerate() {
            golden(&format!("led-red-{name}"), &led::Led.body(&led, state));
        }
        golden("button-released", &button::Button.body(&button, 0));
        golden("button-pressed", &button::Button.body(&button, 1));
        golden("capacitor", &capacitor::Capacitor.body(&capacitor, 0));
        golden(
            "npn-transistor",
            &transistor::Transistor.body(&transistor, 0),
        );
        golden("changeover-switch-nc", &switch::Switch.body(&switch, 0));
        golden("changeover-switch-no", &switch::Switch.body(&switch, 1));
        golden("dc-voltage-source", &source::Source.body(&source, 0));
        golden("buzzer-silent", &buzzer::body(false));
        golden("buzzer-sounding", &buzzer::body(true));
        golden("speaker-silent", &speaker::body(false));
        golden("speaker-sounding", &speaker::body(true));
        golden("piezo-passive-silent", &piezo_passive::body(false));
        golden("piezo-passive-sounding", &piezo_passive::body(true));
        // T19 promotes these two designs from the T17 Part B future references;
        // the golden files are unchanged and now describe a real component.
        golden("future-trimmer-potentiometer", &potentiometer::body());
        golden("future-photoresistor", &photoresistor::body());
        golden(
            "touch-pad",
            &probe::TouchPad.body(
                &component(
                    ComponentKind::TouchPad,
                    &[("a", "A1"), ("b", "A4")],
                    &[("min_resistance", 1_000.0), ("max_resistance", 1e9)],
                ),
                0,
            ),
        );
        golden(
            "water-probe",
            &probe::WaterProbe.body(
                &component(
                    ComponentKind::WaterProbe,
                    &[("a", "A1"), ("b", "A4")],
                    &[("min_resistance", 1_000.0), ("max_resistance", 1e9)],
                ),
                0,
            ),
        );
    }

    #[test]
    fn passive_piezo_state_uses_core_drive_flag_and_has_distinct_art() {
        let piezo = piezo_passive::PiezoPassive;
        assert_eq!(piezo.state(&PartContext::default()), 0);
        assert_eq!(
            piezo.state(&PartContext {
                passive_piezo_sounding: true,
                ..Default::default()
            }),
            1
        );
        assert_ne!(piezo_passive::body(false), buzzer::body(false));
        assert_ne!(piezo_passive::body(false), speaker::body(false));
    }

    /// T17 Part B: design-only references for future parts. None of these is
    /// a `PartArt` or a `ComponentKind`; `future::*` are plain generator
    /// functions read for review, not simulated.
    #[test]
    fn future_part_designs_match_references() {
        golden("future-rectifier-diode", &future::rectifier_diode());
        golden("future-signal-diode", &future::signal_diode());
        golden("future-ceramic-capacitor", &future::ceramic_capacitor());
        golden("future-led-green-lit", &future::green_led_lit());
        golden("future-led-yellow-lit", &future::yellow_led_lit());
        golden("future-led-blue-lit", &future::blue_led_lit());
    }

    #[test]
    fn every_state_of_a_part_has_the_same_size() {
        let parts: [(&dyn PartArt, Component); 13] = [
            (
                &resistor::Resistor,
                component(
                    ComponentKind::Resistor,
                    &[("a", "A1"), ("b", "A4")],
                    &[("resistance", 1.0)],
                ),
            ),
            (
                &led::Led,
                component(
                    ComponentKind::Led,
                    &[("anode", "A1"), ("cathode", "A2")],
                    &[],
                ),
            ),
            (
                &diode::Diode,
                component(
                    ComponentKind::Diode,
                    &[("anode", "A1"), ("cathode", "A4")],
                    &[("forward_voltage", 0.7), ("series_resistance", 10.0)],
                ),
            ),
            (
                &button::Button,
                component(
                    ComponentKind::MomentaryButton,
                    &[("a", "E1"), ("b", "F1")],
                    &[],
                ),
            ),
            (
                &capacitor::Capacitor,
                component(
                    ComponentKind::Capacitor,
                    &[("positive", "A1"), ("negative", "A3")],
                    &[],
                ),
            ),
            (
                &transistor::Transistor,
                component(
                    ComponentKind::NpnTransistor,
                    &[("base", "F12"), ("collector", "F13"), ("emitter", "F14")],
                    &[],
                ),
            ),
            (
                &switch::Switch,
                component(
                    ComponentKind::ChangeoverSwitch,
                    &[
                        ("normally_closed", "F8"),
                        ("common", "F9"),
                        ("normally_open", "F10"),
                    ],
                    &[],
                ),
            ),
            (
                &potentiometer::Potentiometer,
                component(
                    ComponentKind::Potentiometer,
                    &[("a", "A1"), ("b", "A4")],
                    &[("min_resistance", 1.0), ("max_resistance", 10_000.0)],
                ),
            ),
            (
                &photoresistor::Photoresistor,
                component(
                    ComponentKind::Photoresistor,
                    &[("a", "A1"), ("b", "A4")],
                    &[("min_resistance", 100.0), ("max_resistance", 1_000_000.0)],
                ),
            ),
            (
                &buzzer::Buzzer,
                component(
                    ComponentKind::Buzzer,
                    &[("positive", "A1"), ("negative", "A4")],
                    &[("resistance", 32.0)],
                ),
            ),
            (
                &speaker::Speaker,
                component(
                    ComponentKind::Speaker,
                    &[("positive", "A1"), ("negative", "A4")],
                    &[("resistance", 8.0)],
                ),
            ),
            (
                &piezo_passive::PiezoPassive,
                component(
                    ComponentKind::PiezoPassive,
                    &[("positive", "A1"), ("negative", "A4")],
                    &[("resistance", 32.0)],
                ),
            ),
            (
                &motor::Motor,
                component(
                    ComponentKind::Motor,
                    &[("positive", "A1"), ("negative", "A4")],
                    &[
                        ("resistance", 8.0),
                        ("rated_voltage", 3.0),
                        ("no_load_speed_rpm", 10_000.0),
                    ],
                ),
            ),
        ];
        for (art, c) in parts {
            let sizes: Vec<_> = (0..art.state_count())
                .map(|s| {
                    let body = art.body(&c, s);
                    (body.width(), body.height())
                })
                .collect();
            assert!(sizes.windows(2).all(|w| w[0] == w[1]), "{:?}", c.kind);
        }
    }

    #[test]
    fn embedded_fixture_parts_compose() {
        for json in [
            include_str!("../../../../fixtures/projects/led-bench.json"),
            include_str!("../../../../fixtures/projects/transistor-bench.json"),
            include_str!("../../../../fixtures/projects/rc-bench.json"),
        ] {
            let project: Project = serde_json::from_str(json).unwrap();
            for c in &project.components {
                let Some(art) = art_for(c.kind) else { continue };
                let pins: Vec<_> = c
                    .pins
                    .values()
                    .enumerate()
                    .map(|(i, _)| Vec2::new(0.0, -16.0 * i as f32 * 3.0))
                    .collect();
                for state in 0..art.state_count() {
                    assert!(place(art, c, &pins, state).canvas.width() > 0);
                }
            }
        }
    }

    fn assert_leads_reach_holes(
        art: &dyn PartArt,
        c: &Component,
        pins: &[Vec2],
    ) -> Result<(), TestCaseError> {
        let placed = place(art, c, pins, art.state_count() - 1);
        let canvas = &placed.canvas;
        let centre_world = pins[0] + placed.center_offset;
        for pin in pins {
            // Hole centre in canvas pixels; its top-left lead pixel must be drawn.
            let local = (*pin - centre_world) / PIXEL;
            let x = (local.x + canvas.width() as f32 / 2.0).round() as i32;
            let y = (canvas.height() as f32 / 2.0 - local.y).round() as i32;
            prop_assert!(
                canvas.get(x - 1, y - 1).is_some(),
                "no pixel at pin {pin} ({x},{y})"
            );
        }
        Ok(())
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 96, rng_seed: proptest::test_runner::RngSeed::Fixed(0x5B17_E500), .. ProptestConfig::default() })]

        #[test]
        fn placements_cover_every_pin_hole(
            ax in -8i32..8, ay in -8i32..8, dx in -6i32..=6, dy in -6i32..=6, kind in 0usize..9,
        ) {
            prop_assume!((dx, dy) != (0, 0));
            let a = Vec2::new(ax as f32, ay as f32) * 16.0;
            let b = a + Vec2::new(dx as f32, dy as f32) * 16.0;
            let (art, c): (&dyn PartArt, _) = match kind {
                0 => (&resistor::Resistor, component(ComponentKind::Resistor, &[("a", "A1"), ("b", "A4")], &[("resistance", 4700.0)])),
                1 => (&led::Led, component(ComponentKind::Led, &[("anode", "A1"), ("cathode", "A2")], &[])),
                2 => (&button::Button, component(ComponentKind::MomentaryButton, &[("a", "E1"), ("b", "F1")], &[])),
                3 => (&potentiometer::Potentiometer, component(ComponentKind::Potentiometer, &[("a", "A1"), ("b", "A4")], &[("min_resistance", 1.0), ("max_resistance", 10_000.0)])),
                4 => (&photoresistor::Photoresistor, component(ComponentKind::Photoresistor, &[("a", "A1"), ("b", "A4")], &[("min_resistance", 100.0), ("max_resistance", 1_000_000.0)])),
                5 => (&buzzer::Buzzer, component(ComponentKind::Buzzer, &[("positive", "A1"), ("negative", "A4")], &[("resistance", 32.0)])),
                6 => (&speaker::Speaker, component(ComponentKind::Speaker, &[("positive", "A1"), ("negative", "A4")], &[("resistance", 8.0)])),
                7 => (&piezo_passive::PiezoPassive, component(ComponentKind::PiezoPassive, &[("positive", "A1"), ("negative", "A4")], &[("resistance", 32.0)])),
                _ => (&motor::Motor, component(ComponentKind::Motor, &[("positive", "A1"), ("negative", "A4")], &[("resistance", 8.0), ("rated_voltage", 3.0), ("no_load_speed_rpm", 10_000.0)])),
            };
            assert_leads_reach_holes(art, &c, &[a, b])?;
        }

        #[test]
        fn three_pin_placements_cover_every_pin_hole(
            ax in -8i32..8, ay in -8i32..8, dx in -6i32..=6, dy in -6i32..=6,
            mx in -4i32..=4, my in -4i32..=4, kind in 0usize..2,
        ) {
            prop_assume!((dx, dy) != (0, 0));
            let a = Vec2::new(ax as f32, ay as f32) * 16.0;
            let b = a + Vec2::new(dx as f32, dy as f32) * 16.0;
            let m = a + Vec2::new(mx as f32, my as f32) * 16.0;
            prop_assume!(m != a && m != b);
            let (art, c, pins): (&dyn PartArt, _, [Vec2; 3]) = match kind {
                0 => (
                    &transistor::Transistor,
                    component(
                        ComponentKind::NpnTransistor,
                        &[("base", "F12"), ("collector", "F13"), ("emitter", "F14")],
                        &[],
                    ),
                    // BTreeMap key order: base, collector, emitter.
                    [m, b, a],
                ),
                _ => (
                    &switch::Switch,
                    component(
                        ComponentKind::ChangeoverSwitch,
                        &[
                            ("normally_closed", "F8"),
                            ("common", "F9"),
                            ("normally_open", "F10"),
                        ],
                        &[],
                    ),
                    // BTreeMap key order: common, normally_closed, normally_open.
                    [m, a, b],
                ),
            };
            assert_leads_reach_holes(art, &c, &pins)?;
        }
    }
}
