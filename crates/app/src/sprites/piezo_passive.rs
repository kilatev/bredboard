use bredboard_core::Component;

use super::palette;
use super::{PartArt, PartContext, PixelCanvas};

/// Passive piezo transducer art. Its state is supplied by the core's
/// fixed-step oscillation detector rather than by this module.
pub struct PiezoPassive;

impl PartArt for PiezoPassive {
    fn axis_pins(&self) -> [&'static str; 2] {
        ["positive", "negative"]
    }

    fn state_count(&self) -> usize {
        2
    }

    fn state(&self, context: &PartContext) -> usize {
        usize::from(context.passive_piezo_sounding)
    }

    fn body(&self, _component: &Component, state: usize) -> PixelCanvas {
        body(state == 1)
    }
}

/// 14 x 14 flat rectangular transducer: a dark ceramic case, a metal centre
/// plate, and a positive-side stripe. This reads as a passive piezo element,
/// distinct from the buzzer grille and the speaker cone.
pub fn body(sounding: bool) -> PixelCanvas {
    const WIDTH: i32 = 14;
    const HEIGHT: i32 = 10;
    const WAVE_PAD: i32 = 4;
    let mut canvas = PixelCanvas::new(WIDTH, HEIGHT + WAVE_PAD);
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let edge = x == 0 || x == WIDTH - 1 || y == 0 || y == HEIGHT - 1;
            let color = if edge {
                palette::OUTLINE
            } else if y <= 2 && (2..WIDTH - 2).contains(&x) {
                palette::CAP_SPECULAR
            } else if y >= HEIGHT - 3 {
                palette::CAP
            } else {
                palette::CAP_LIGHT
            };
            canvas.set(x, y + WAVE_PAD, color);
        }
    }
    for y in 3..HEIGHT - 3 {
        canvas.set(1, y + WAVE_PAD, palette::ELCAP_STRIPE);
    }
    for (x, y) in [(4, 0), (5, 1), (8, 0), (7, 1), (11, 1), (2, 2), (9, 2)] {
        if sounding {
            canvas.set(x, y, palette::SOUND_WAVE);
        }
    }
    canvas
}
