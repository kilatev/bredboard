use bredboard_core::Component;

use super::palette;
use super::{PartArt, PartContext, PixelCanvas};

pub struct Motor;

impl PartArt for Motor {
    fn axis_pins(&self) -> [&'static str; 2] {
        ["positive", "negative"]
    }

    fn state_count(&self) -> usize {
        2
    }

    fn state(&self, context: &PartContext) -> usize {
        usize::from(context.motor_speed.abs() > 1.0)
    }

    fn body(&self, _component: &Component, state: usize) -> PixelCanvas {
        body(state == 1)
    }
}

pub fn body(spinning: bool) -> PixelCanvas {
    let mut canvas = PixelCanvas::new(16, 12);
    for y in 2..10 {
        for x in 2..14 {
            let color = if x == 2 || x == 13 || y == 2 || y == 9 {
                palette::OUTLINE
            } else if spinning && (x + y) % 3 == 0 {
                palette::METAL_LIGHT
            } else {
                palette::METAL_DARK
            };
            canvas.set(x, y, color);
        }
    }
    for y in 4..8 {
        canvas.set(4, y, palette::MOTOR_SHAFT);
        canvas.set(11, y, palette::MOTOR_SHAFT);
    }
    for x in 6..10 {
        canvas.set(x, 5, palette::MOTOR_SHAFT);
        canvas.set(x, 6, palette::MOTOR_SHAFT);
    }
    if spinning {
        for (x, y) in [(7, 3), (10, 5), (8, 8), (5, 6)] {
            canvas.set(x, y, palette::SOUND_WAVE);
        }
    }
    canvas
}
