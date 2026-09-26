//! Design-only pixel generators for T17 Part B: future simple parts that do
//! not yet have an electrical model or a `ComponentKind`. These functions
//! exist only to produce the golden `.txt` references under
//! `docs/design/sprites/future/` for review; nothing here is registered in
//! [`super::art_for`] and no runtime code depends on this module.
#![cfg(test)]

use super::PixelCanvas;
use super::led;
use super::palette::{self, Band, future as future_palette};

/// 14 x 8, the same footprint as the resistor: a black cylinder with a
/// silver cathode band near the right (cathode) end.
pub fn rectifier_diode() -> PixelCanvas {
    diode_body(14, 8, 10)
}

/// 14 x 6 orange glass body with a black cathode band near the right end.
pub fn signal_diode() -> PixelCanvas {
    let mut canvas = PixelCanvas::new(14, 6);
    let (light, dark) = Band::Orange.shades();
    for y in 0..6 {
        for x in 0..14 {
            let color = if x == 0 || x == 13 || y == 0 || y == 5 {
                palette::OUTLINE
            } else if y == 1 {
                light
            } else if y >= 4 {
                dark
            } else {
                light
            };
            canvas.set(x, y, color);
        }
    }
    for x in 10..12 {
        for y in 1..5 {
            canvas.set(x, y, palette::OUTLINE);
        }
    }
    canvas
}

fn diode_body(width: i32, height: i32, band_x: i32) -> PixelCanvas {
    let mut canvas = PixelCanvas::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let color = if x == 0 || x == width - 1 || y == 0 || y == height - 1 {
                palette::OUTLINE
            } else if y == 1 {
                palette::TO92_HIGHLIGHT
            } else if y >= height - 2 {
                palette::TO92_SHADE
            } else {
                palette::TO92_BODY
            };
            canvas.set(x, y, color);
        }
    }
    for x in band_x..(band_x + 2).min(width - 1) {
        for y in 1..height - 1 {
            canvas.set(
                x,
                y,
                if y == 1 {
                    palette::METAL_LIGHT
                } else {
                    palette::METAL
                },
            );
        }
    }
    canvas
}

/// 10 x 10 small ochre disc on two leg stubs.
pub fn ceramic_capacitor() -> PixelCanvas {
    let mut canvas = PixelCanvas::new(10, 10);
    let (light, dark) = Band::Brown.shades();
    let (cx, cy) = (4.5, 4.5);
    for y in 0..10 {
        for x in 0..10 {
            let d = (x as f64 - cx).hypot(y as f64 - cy);
            if d > 4.4 {
                continue;
            }
            let color = if d > 3.7 {
                palette::OUTLINE
            } else if (x as f64 - cx) + (y as f64 - cy) < 0.0 {
                light
            } else {
                dark
            };
            canvas.set(x, y, color);
        }
    }
    for (x, y) in [(2, 9), (7, 9)] {
        canvas.set(x, y, palette::LEAD);
        canvas.set(x, y - 1, palette::LEAD_SHADE);
    }
    canvas
}

/// Representative "lit" state for a colored LED, reusing the real LED dome
/// drawing with a design-only colour family. Off and dim states are not
/// rendered here; they would follow the same three-tier shading the red LED
/// already uses.
pub fn green_led_lit() -> PixelCanvas {
    led::body(&future_palette::GREEN_LED_LIT)
}
pub fn yellow_led_lit() -> PixelCanvas {
    led::body(&future_palette::YELLOW_LED_LIT)
}
pub fn blue_led_lit() -> PixelCanvas {
    led::body(&future_palette::BLUE_LED_LIT)
}
