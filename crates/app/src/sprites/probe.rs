use bredboard_core::Component;

use super::palette::{self, Band};
use super::{PartArt, PartContext, PixelCanvas};

pub struct TouchPad;
pub struct WaterProbe;

impl PartArt for TouchPad {
    fn axis_pins(&self) -> [&'static str; 2] {
        ["a", "b"]
    }

    fn state(&self, _context: &PartContext) -> usize {
        0
    }

    fn body(&self, _component: &Component, _state: usize) -> PixelCanvas {
        body(Band::Yellow.shades().0)
    }
}

impl PartArt for WaterProbe {
    fn axis_pins(&self) -> [&'static str; 2] {
        ["a", "b"]
    }

    fn state(&self, _context: &PartContext) -> usize {
        0
    }

    fn body(&self, _component: &Component, _state: usize) -> PixelCanvas {
        body(Band::Blue.shades().0)
    }
}

fn body(pad: [u8; 3]) -> PixelCanvas {
    let mut canvas = PixelCanvas::new(16, 8);
    for y in 1..7 {
        for x in 1..7 {
            canvas.set(x, y, pad);
        }
        for x in 9..15 {
            canvas.set(x, y, pad);
        }
    }
    for x in 0..16 {
        canvas.set(x, 0, palette::OUTLINE);
        canvas.set(x, 7, palette::OUTLINE);
    }
    canvas.set(7, 3, palette::OUTLINE);
    canvas.set(8, 3, palette::OUTLINE);
    canvas.set(7, 4, palette::OUTLINE);
    canvas.set(8, 4, palette::OUTLINE);
    canvas
}
