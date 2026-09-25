use bredboard_core::{Component, ControlState};

use super::palette;
use super::{PartArt, PartContext, PixelCanvas};

pub struct Button;

impl PartArt for Button {
    fn axis_pins(&self) -> [&'static str; 2] {
        ["a", "b"]
    }

    fn state_count(&self) -> usize {
        2
    }

    fn state(&self, context: &PartContext) -> usize {
        usize::from(context.control == Some(ControlState::ButtonPressed))
    }

    fn body(&self, _component: &Component, state: usize) -> PixelCanvas {
        body(state == 1)
    }
}

/// Compact 10 x 10 px tactile switch so both legs and holes stay visible.
pub fn body(pressed: bool) -> PixelCanvas {
    let mut canvas = PixelCanvas::new(10, 10);
    for y in 0..10 {
        for x in 0..10 {
            let color = if x == 0 || x == 9 || y == 0 || y == 9 {
                palette::OUTLINE
            } else if x == 1 || y == 1 {
                palette::METAL_LIGHT
            } else if x == 8 || y == 8 {
                palette::METAL_DARK
            } else {
                palette::METAL
            };
            canvas.set(x, y, color);
        }
    }
    for (x, y) in [(0, 0), (9, 0), (0, 9), (9, 9)] {
        canvas.clear(x, y);
    }
    let (cx, cy) = (4.5, if pressed { 5.0 } else { 4.5 });
    let radius = if pressed { 3.0 } else { 3.5 };
    let distance = |x: i32, y: i32, ox: f64, oy: f64| (x as f64 - ox).hypot(y as f64 - oy);
    if !pressed {
        for y in 2..8 {
            for x in 2..8 {
                if distance(x, y, cx + 1.0, cy + 1.0) <= radius && distance(x, y, cx, cy) > radius {
                    canvas.set(x, y, palette::METAL_DARK);
                }
            }
        }
    }
    for y in 0..10 {
        for x in 0..10 {
            let d = distance(x, y, cx, cy);
            if d <= radius {
                let edge = d > radius - 0.9;
                canvas.set(x, y, if edge { palette::OUTLINE } else { palette::CAP });
            }
        }
    }
    if !pressed {
        canvas.set(3, 3, palette::CAP_SPECULAR);
        canvas.set(4, 3, palette::CAP_LIGHT);
        canvas.set(3, 4, palette::CAP_LIGHT);
    }
    canvas
}
