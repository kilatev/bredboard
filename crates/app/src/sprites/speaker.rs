use bredboard_core::Component;

use super::buzzer::SOUNDING_CURRENT;
use super::palette;
use super::{PartArt, PartContext, PixelCanvas};

pub struct Speaker;

impl PartArt for Speaker {
    fn axis_pins(&self) -> [&'static str; 2] {
        ["positive", "negative"]
    }

    fn state_count(&self) -> usize {
        2
    }

    /// Same current-driven "sounding" presentation pattern as the buzzer
    /// (see `buzzer::SOUNDING_CURRENT`); both kinds are electrically
    /// identical fixed resistive loads and share the threshold.
    fn state(&self, context: &PartContext) -> usize {
        usize::from(context.buzzer_current.abs() >= SOUNDING_CURRENT)
    }

    fn body(&self, _component: &Component, state: usize) -> PixelCanvas {
        body(state == 1)
    }
}

/// 16 x 12 top-down dynamic speaker, same footprint as the buzzer but reads
/// as a cone rather than a flat disc: a dark case ring, a lighter cone slope
/// ring, and a dark centre dust cap, with the same headroom above the case
/// for sound-wave marks drawn only while sounding.
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
            } else if d > 4.0 {
                // Case rim.
                palette::SWITCH_CASE
            } else if d > 2.2 {
                // Cone slope, lighter toward the top-left for the shared
                // top-left light source.
                if x as f64 - cx < 0.0 && (y + WAVE_PAD) as f64 - cy < 0.0 {
                    palette::METAL_LIGHT
                } else {
                    palette::METAL
                }
            } else {
                // Dust cap.
                palette::METAL_DARK
            };
            canvas.set(x, y + WAVE_PAD, color);
        }
    }
    // Polarity stripe on the positive (left) side, reused from the
    // electrolytic capacitor's stripe colour, same as the buzzer.
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
