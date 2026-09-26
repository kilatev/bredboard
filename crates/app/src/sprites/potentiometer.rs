use bredboard_core::Component;

use super::palette::{self, Band};
use super::{PartArt, PartContext, PixelCanvas};

pub struct Potentiometer;

impl PartArt for Potentiometer {
    fn axis_pins(&self) -> [&'static str; 2] {
        ["a", "b"]
    }

    fn state(&self, _context: &PartContext) -> usize {
        0
    }

    fn body(&self, _component: &Component, _state: usize) -> PixelCanvas {
        body()
    }
}

/// 12 x 12 blue square body with a white cross-slot knob. The control ratio
/// is a presentation-invisible electrical signal, not a visual state (per
/// the sprite design reference), so this is the part's only body.
pub fn body() -> PixelCanvas {
    let mut canvas = PixelCanvas::new(12, 12);
    let (blue_light, blue_dark) = Band::Blue.shades();
    let (white_light, white_dark) = Band::White.shades();
    for y in 0..12 {
        for x in 0..12 {
            let color = if x == 0 || x == 11 || y == 0 || y == 11 {
                palette::OUTLINE
            } else if y == 1 || x == 1 {
                blue_light
            } else {
                blue_dark
            };
            canvas.set(x, y, color);
        }
    }
    let (cx, cy, radius) = (5.5, 5.5, 3.2);
    for y in 0..12 {
        for x in 0..12 {
            let d = (x as f64 - cx).hypot(y as f64 - cy);
            if d > radius {
                continue;
            }
            canvas.set(
                x,
                y,
                if d > radius - 0.9 {
                    palette::OUTLINE
                } else if (x + y) % 2 == 0 {
                    white_light
                } else {
                    white_dark
                },
            );
        }
    }
    for i in 3..8 {
        canvas.set(i, 5, palette::OUTLINE);
        canvas.set(i, 6, palette::OUTLINE);
        canvas.set(5, i, palette::OUTLINE);
        canvas.set(6, i, palette::OUTLINE);
    }
    canvas
}
