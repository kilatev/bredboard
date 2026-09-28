use super::{PartArt, PartContext, PixelCanvas};
use crate::sprites::palette;
use bredboard_core::Component;

pub struct IntegratedCircuit;

impl PartArt for IntegratedCircuit {
    fn axis_pins(&self) -> [&'static str; 2] {
        ["input_a", "output"]
    }

    fn body(&self, _component: &Component, _state: usize) -> PixelCanvas {
        let mut canvas = PixelCanvas::new(18, 12);
        for y in 0..12 {
            for x in 0..18 {
                let edge = x == 0 || x == 17 || y == 0 || y == 11;
                canvas.set(
                    x,
                    y,
                    if edge {
                        palette::OUTLINE
                    } else if y < 2 {
                        palette::METAL_LIGHT
                    } else {
                        palette::TO92_BODY
                    },
                );
            }
        }
        canvas.set(2, 2, palette::METAL);
        canvas.set(2, 3, palette::METAL);
        canvas
    }

    fn state(&self, _context: &PartContext) -> usize {
        0
    }
}
