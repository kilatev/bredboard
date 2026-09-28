use bredboard_core::Component;

use super::palette;
use super::{PartArt, PartContext, PixelCanvas};

pub struct Relay;

impl PartArt for Relay {
    fn axis_pins(&self) -> [&'static str; 2] {
        ["coil_positive", "coil_negative"]
    }

    fn state_count(&self) -> usize {
        2
    }

    fn state(&self, context: &PartContext) -> usize {
        usize::from(context.relay_energized)
    }

    fn body(&self, _component: &Component, state: usize) -> PixelCanvas {
        body(state == 1)
    }
}

fn body(energized: bool) -> PixelCanvas {
    let mut canvas = PixelCanvas::new(16, 12);
    for y in 2..10 {
        for x in 2..14 {
            let color = if x == 2 || x == 13 || y == 2 || y == 9 {
                palette::OUTLINE
            } else if energized && (x + y) % 3 == 0 {
                palette::MOTOR_SHAFT
            } else {
                palette::SWITCH_CASE
            };
            canvas.set(x, y, color);
        }
    }
    for y in 4..8 {
        canvas.set(5, y, palette::METAL);
        canvas.set(10, y, palette::METAL);
    }
    canvas
}
