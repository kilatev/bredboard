use bredboard_core::Component;

use super::palette;
use super::{PartArt, PartContext, PixelCanvas};

/// Current threshold (amperes) between the silent and sounding sprites.
/// Same order of magnitude as the LED's dim threshold: a buzzer this small
/// draws a few milliamps to sound.
pub const SOUNDING_CURRENT: f64 = 0.001;

pub struct Buzzer;

impl PartArt for Buzzer {
    fn axis_pins(&self) -> [&'static str; 2] {
        ["positive", "negative"]
    }

    fn state_count(&self) -> usize {
        2
    }

    /// "Sounding" is a presentation mapping of the calculated buzzer current;
    /// the buzzer has no audio output, only a visual state (per the sprite
    /// design reference), the same pattern as the LED's current thresholds.
    fn state(&self, context: &PartContext) -> usize {
        usize::from(context.buzzer_current.abs() >= SOUNDING_CURRENT)
    }

    fn body(&self, _component: &Component, state: usize) -> PixelCanvas {
        body(state == 1)
    }
}

/// 16 x 12 top-down disc (case reused from `changeover_switch`'s dark
/// plastic, grille from `capacitor`'s metal top): a round black case with a
/// silver grille, a polarity stripe on the positive (left) side, and two
/// sound-wave arcs above the case that are only drawn while sounding.
pub fn body(sounding: bool) -> PixelCanvas {
    const DISC: i32 = 12;
    const WAVE_PAD: i32 = 4;
    let mut canvas = PixelCanvas::new(DISC, DISC + WAVE_PAD);
    let (cx, cy) = (5.5, (WAVE_PAD + DISC / 2) as f64 - 0.5);
    for y in 0..DISC {
        for x in 0..DISC {
            let d = (x as f64 - cx).hypot((y + WAVE_PAD) as f64 - cy);
            if d > 5.9 {
                continue;
            }
            let color = if d > 5.1 {
                palette::OUTLINE
            } else if d > 3.4 {
                palette::SWITCH_CASE
            } else {
                palette::SWITCH_CASE_SHADE
            };
            canvas.set(x, y + WAVE_PAD, color);
        }
    }
    // Grille: a small ring of metal dots, reused from the capacitor's vent colours.
    for (dx, dy) in [
        (0, -3),
        (0, 3),
        (-3, 0),
        (3, 0),
        (-2, -2),
        (2, 2),
        (-2, 2),
        (2, -2),
    ] {
        let (x, y) = (
            (cx + dx as f64).round() as i32,
            (cy + dy as f64).round() as i32,
        );
        canvas.set(x, y, palette::METAL);
    }
    canvas.set(cx.round() as i32, cy.round() as i32, palette::METAL_LIGHT);
    // Polarity stripe on the positive (left) side, reused from the
    // electrolytic capacitor's stripe colour.
    for y in 3..9 {
        canvas.set(1, y + WAVE_PAD, palette::ELCAP_STRIPE);
    }
    if sounding {
        for (x, y) in [(4, 0), (5, 1), (8, 0), (7, 1), (11, 1), (2, 2), (9, 2)] {
            canvas.set(x, y, palette::SOUND_WAVE);
        }
    }
    canvas
}
