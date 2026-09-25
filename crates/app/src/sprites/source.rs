use bredboard_core::Component;

use super::palette;
use super::{PartArt, PixelCanvas};

/// 3 x 5 pixel glyphs for the "5" and "V" marking. `1` lights a pixel.
const FIVE: [[u8; 3]; 5] = [[1, 1, 1], [1, 0, 0], [1, 1, 1], [0, 0, 1], [1, 1, 1]];
const LETTER_V: [[u8; 3]; 5] = [[1, 0, 1], [1, 0, 1], [1, 0, 1], [1, 0, 1], [0, 1, 0]];

/// Off-board 5 V supply. Placed off the board grid, so leads are drawn by the
/// caller rather than the shared placement code; see `main.rs`.
pub struct Source;

impl PartArt for Source {
    fn axis_pins(&self) -> [&'static str; 2] {
        ["positive", "negative"]
    }

    fn body(&self, _component: &Component, _state: usize) -> PixelCanvas {
        body()
    }
}

/// 22 x 14 dark case with a red + and blue - terminal and a "5V" marking.
pub fn body() -> PixelCanvas {
    let mut canvas = PixelCanvas::new(22, 14);
    for y in 0..14 {
        for x in 0..22 {
            let color = if x == 0 || x == 21 || y == 0 || y == 13 {
                palette::OUTLINE
            } else if y == 1 {
                palette::SOURCE_CASE_LIGHT
            } else if y >= 11 {
                palette::SOURCE_CASE_SHADE
            } else {
                palette::SOURCE_CASE
            };
            canvas.set(x, y, color);
        }
    }
    for (x, y) in [(0, 0), (21, 0), (0, 13), (21, 13)] {
        canvas.clear(x, y);
    }
    // Terminal marks: red plus on the left (positive), blue minus on the right.
    for (x, y) in [(3, 3), (4, 2), (4, 3), (4, 4), (5, 3)] {
        canvas.set(x, y, palette::RAIL_RED);
    }
    for (x, y) in [(16, 3), (17, 3), (18, 3)] {
        canvas.set(x, y, palette::RAIL_BLUE);
    }
    glyph(&mut canvas, &FIVE, 7, 6);
    glyph(&mut canvas, &LETTER_V, 12, 6);
    canvas
}

fn glyph(canvas: &mut PixelCanvas, rows: &[[u8; 3]; 5], ox: i32, oy: i32) {
    for (y, row) in rows.iter().enumerate() {
        for (x, lit) in row.iter().enumerate() {
            if *lit == 1 {
                canvas.set(ox + x as i32, oy + y as i32, palette::SOURCE_LABEL);
            }
        }
    }
}
