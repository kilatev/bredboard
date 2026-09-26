use bredboard_core::Component;

use super::palette::{self, Band};
use super::{PartArt, PartContext, PixelCanvas};

pub struct Photoresistor;

impl PartArt for Photoresistor {
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

/// 12 x 12 round orange-red disc with a zigzag conductive track. The ambient
/// light control ratio is an electrical signal, not a visual state, so this
/// is the part's only body.
pub fn body() -> PixelCanvas {
    let mut canvas = PixelCanvas::new(12, 12);
    let (orange, red) = (Band::Orange.shades().0, Band::Red.shades().0);
    let (cx, cy, radius) = (5.5, 5.5, 5.4);
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
                } else if d > radius - 2.4 {
                    orange
                } else {
                    red
                },
            );
        }
    }
    for (x, y) in [
        (2, 6),
        (3, 4),
        (4, 6),
        (5, 4),
        (6, 6),
        (7, 4),
        (8, 6),
        (9, 4),
    ] {
        canvas.set(x, y, palette::OUTLINE);
    }
    canvas
}
