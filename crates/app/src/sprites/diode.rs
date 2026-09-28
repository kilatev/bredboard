use bredboard_core::Component;

use super::palette;
use super::{PartArt, PartContext, PixelCanvas};

pub struct Diode;

impl PartArt for Diode {
    fn axis_pins(&self) -> [&'static str; 2] {
        ["anode", "cathode"]
    }

    fn body(&self, _component: &Component, _state: usize) -> PixelCanvas {
        body()
    }

    fn state(&self, _context: &PartContext) -> usize {
        0
    }
}

/// A resistor-sized rectifier body with a visible silver cathode band.
pub fn body() -> PixelCanvas {
    let mut canvas = PixelCanvas::new(14, 8);
    for y in 0..8 {
        for x in 0..14 {
            let color = if x == 0 || x == 13 || y == 0 || y == 7 {
                palette::OUTLINE
            } else if y == 1 {
                palette::TO92_HIGHLIGHT
            } else if y >= 6 {
                palette::TO92_SHADE
            } else {
                palette::TO92_BODY
            };
            canvas.set(x, y, color);
        }
    }
    for y in 1..7 {
        canvas.set(
            10,
            y,
            if y == 1 {
                palette::METAL_LIGHT
            } else {
                palette::METAL
            },
        );
        canvas.set(11, y, palette::METAL);
    }
    canvas
}
