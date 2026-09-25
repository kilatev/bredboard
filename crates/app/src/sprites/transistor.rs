use bredboard_core::Component;

use super::palette;
use super::{PartArt, PixelCanvas};

/// NPN and PNP transistors share the same TO-92 body; they are told apart by
/// label and hover text, not by shape (see the design reference, Part B).
pub struct Transistor;

impl PartArt for Transistor {
    fn axis_pins(&self) -> [&'static str; 2] {
        ["emitter", "collector"]
    }

    fn body(&self, _component: &Component, _state: usize) -> PixelCanvas {
        body()
    }
}

/// TO-92 seen from above: 14 x 8, a flat face on the left and a rounded back
/// on the right. The middle (base) lead is drawn by the shared placement code.
pub fn body() -> PixelCanvas {
    let mut canvas = PixelCanvas::new(14, 8);
    let (cx, cy, radius) = (4.0, 3.5, 3.6);
    for y in 0..8 {
        for x in 4..14 {
            let d = (x as f64 - cx).hypot(y as f64 - cy);
            if d > radius {
                continue;
            }
            let color = if d > radius - 0.9 {
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
    // Flat face: a straight vertical edge in front of the rounded back.
    for y in 0..8 {
        canvas.set(1, y, palette::OUTLINE);
        for x in 2..4 {
            canvas.set(
                x,
                y,
                if y == 1 {
                    palette::TO92_HIGHLIGHT
                } else if y >= 6 {
                    palette::TO92_SHADE
                } else {
                    palette::TO92_BODY
                },
            );
        }
    }
    canvas
}
