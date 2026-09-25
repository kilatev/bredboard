use bredboard_core::Component;

use super::palette;
use super::{PartArt, PixelCanvas};

pub struct Capacitor;

impl PartArt for Capacitor {
    fn axis_pins(&self) -> [&'static str; 2] {
        ["positive", "negative"]
    }

    fn body(&self, _component: &Component, _state: usize) -> PixelCanvas {
        body()
    }
}

/// Top-down 12 x 12 electrolytic can: blue sleeve, silver vented top, and a
/// minus stripe on the negative side (the right edge, unrotated).
pub fn body() -> PixelCanvas {
    let mut canvas = PixelCanvas::new(12, 12);
    let (cx, cy) = (5.5, 5.5);
    let distance = |x: i32, y: i32| (x as f64 - cx).hypot(y as f64 - cy);
    for y in 0..12 {
        for x in 0..12 {
            let d = distance(x, y);
            if d > 5.9 {
                continue;
            }
            let diag = (x as f64 - cx) + (y as f64 - cy);
            let color = if d > 5.1 {
                palette::OUTLINE
            } else if d > 3.6 {
                if diag < -2.0 {
                    palette::ELCAP_HIGHLIGHT
                } else if diag > 2.0 {
                    palette::ELCAP_SHADE
                } else {
                    palette::ELCAP_BODY
                }
            } else if d > 3.0 {
                palette::OUTLINE
            } else if diag < 0.0 {
                palette::ELCAP_TOP
            } else {
                palette::ELCAP_TOP_SHADE
            };
            canvas.set(x, y, color);
        }
    }
    // Vent cross scored into the silver top.
    for i in 3..=8 {
        canvas.set(i, 5, palette::ELCAP_TOP_SHADE);
        canvas.set(i, 6, palette::ELCAP_TOP_SHADE);
        canvas.set(5, i, palette::ELCAP_TOP_SHADE);
        canvas.set(6, i, palette::ELCAP_TOP_SHADE);
    }
    canvas.set(5, 5, palette::OUTLINE);
    canvas.set(6, 6, palette::OUTLINE);
    // Minus stripe and mark on the negative (right) side of the sleeve.
    for y in 2..10 {
        canvas.set(10, y, palette::ELCAP_STRIPE);
    }
    canvas.set(10, 5, palette::OUTLINE);
    canvas.set(10, 6, palette::OUTLINE);
    canvas
}
