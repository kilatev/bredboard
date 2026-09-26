use bredboard_core::Component;

use super::palette::{self, LedShades};
use super::{PartArt, PartContext, PixelCanvas};

/// Current thresholds (amperes) between the off, dim and lit sprites.
pub const DIM_CURRENT: f64 = 0.000_5;
pub const LIT_CURRENT: f64 = 0.005;

pub struct Led;

impl PartArt for Led {
    fn axis_pins(&self) -> [&'static str; 2] {
        ["anode", "cathode"]
    }

    fn state_count(&self) -> usize {
        3
    }

    /// Brightness is a presentation mapping of the calculated LED current.
    fn state(&self, context: &PartContext) -> usize {
        match context.led_current {
            current if current >= LIT_CURRENT => 2,
            current if current >= DIM_CURRENT => 1,
            _ => 0,
        }
    }

    fn body(&self, _component: &Component, state: usize) -> PixelCanvas {
        body(match state {
            0 => &palette::RED_LED_OFF,
            1 => &palette::RED_LED_DIM,
            _ => &palette::RED_LED_ON,
        })
    }
}

/// 20 x 20 px: a compact 12 px dome and rim centred with room for the lit halo.
/// Unrotated, the anode is left and the rim's flat marks the cathode on the right.
pub fn body(shades: &LedShades) -> PixelCanvas {
    const DOME: i32 = 12;
    const PAD: i32 = 4;
    let mut dome = PixelCanvas::new(DOME, DOME);
    let (cx, cy) = (5.5, 5.5);
    for y in 0..DOME {
        for x in 0..DOME {
            if (x as f64 - cx).hypot(y as f64 - cy) <= 5.4 && x <= 9 {
                dome.set(x, y, shades.rim);
            }
        }
    }
    let rim = dome.clone();
    for y in 0..DOME {
        for x in 0..DOME {
            let open = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                .iter()
                .any(|(dx, dy)| rim.get(x + dx, y + dy).is_none());
            if rim.get(x, y).is_some() && open {
                dome.set(x, y, palette::OUTLINE);
            }
        }
    }
    for y in 0..DOME {
        for x in 0..DOME {
            let (dx, dy) = (x as f64 - cx, y as f64 - cy);
            let d = dx.hypot(dy);
            if d > 3.9 {
                continue;
            }
            let lit = if d > 0.0 {
                (-dx - dy) / (d * 1.414 + 1e-6)
            } else {
                1.0
            };
            let color = if d > 3.2 {
                if lit < 0.3 { shades.dark } else { shades.mid }
            } else if lit > 0.45 && d > 1.0 {
                shades.light
            } else if lit < -0.35 && d > 1.5 {
                shades.dark
            } else {
                shades.mid
            };
            dome.set(x, y, color);
        }
    }
    for (x, y) in [(4, 3), (3, 4), (4, 4)] {
        dome.set(x, y, shades.specular);
    }
    dome.set(7, 7, shades.light);

    let mut canvas = PixelCanvas::new(DOME + 2 * PAD, DOME + 2 * PAD);
    if let Some((glow, glow_light)) = shades.glow {
        let centre = (DOME / 2 + PAD) as f64;
        let (inner, outer) = (6.4, 9.0);
        for y in 0..canvas.height() {
            for x in 0..canvas.width() {
                let d = (x as f64 + 0.5 - centre).hypot(y as f64 + 0.5 - centre);
                let odd = (x + y) % 2 == 1;
                if (inner..inner + 1.6).contains(&d) {
                    canvas.set(x, y, if odd { glow_light } else { glow });
                } else if (inner + 1.6..outer).contains(&d) && !odd {
                    canvas.set(x, y, glow);
                }
            }
        }
    }
    canvas.blit(&dome, PAD, PAD);
    canvas
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig { cases: 128, rng_seed: proptest::test_runner::RngSeed::Fixed(0x1ED5_7A7E), .. ProptestConfig::default() })]

        #[test]
        fn brightness_state_never_drops_as_current_rises(a in -0.01f64..0.05, b in -0.01f64..0.05) {
            let (low, high) = if a <= b { (a, b) } else { (b, a) };
            let state = |current| {
                Led.state(&PartContext {
                    led_current: current,
                    ..Default::default()
                })
            };
            prop_assert!(state(low) <= state(high));
        }
    }

    #[test]
    fn off_and_lit_are_visibly_distinct() {
        let off = body(&palette::RED_LED_OFF);
        let lit = body(&palette::RED_LED_ON);
        assert_eq!((off.width(), off.height()), (lit.width(), lit.height()));
        assert!(
            off.get(2, 10).is_none() && lit.get(2, 10).is_some(),
            "lit adds a halo"
        );
        assert_ne!(off.get(10, 10), lit.get(10, 10));
    }
}
