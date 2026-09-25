use bredboard_core::Component;

use super::palette::{self, Band};
use super::{PartArt, PartContext, PixelCanvas};

pub struct Resistor;

impl PartArt for Resistor {
    fn axis_pins(&self) -> [&'static str; 2] {
        ["a", "b"]
    }

    fn state(&self, _context: &PartContext) -> usize {
        0
    }

    fn body(&self, component: &Component, _state: usize) -> PixelCanvas {
        let ohms = component
            .parameters
            .get("resistance")
            .copied()
            .unwrap_or(0.0);
        body(color_code(ohms))
    }
}

/// Four-band code: two significant digits, multiplier, gold 5 % tolerance.
/// Values round to two significant digits; supported values are 1 ohm to 10 Mohm.
pub fn color_code(ohms: f64) -> [Band; 4] {
    if !ohms.is_finite() || ohms <= 0.0 {
        return [Band::Black, Band::Black, Band::Black, Band::Gold];
    }
    // Scale to a two-digit mantissa 10..=99 and a power-of-ten exponent.
    let mut exponent = ohms.log10().floor() as i32 - 1;
    let mut mantissa = (ohms / 10f64.powi(exponent)).round() as i32;
    if mantissa >= 100 {
        mantissa /= 10;
        exponent += 1;
    }
    let multiplier = match exponent {
        -2 => Band::Silver,
        -1 => Band::Gold,
        0..=9 => Band::DIGITS[exponent as usize],
        _ if exponent < -2 => Band::Silver,
        _ => Band::White,
    };
    [
        Band::DIGITS[(mantissa / 10) as usize],
        Band::DIGITS[(mantissa % 10) as usize],
        multiplier,
        Band::Gold,
    ]
}

/// Compact 14 x 8 px dog-bone so both leads and their holes stay visible.
/// Bands are 1 px wide at columns 2, 5, 7 and 11.
pub fn body(bands: [Band; 4]) -> PixelCanvas {
    let mut canvas = PixelCanvas::new(14, 8);
    let band_columns = [2, 5, 7, 11];
    for x in 0..14 {
        let (top, bottom) = match x {
            1..=3 | 10..=12 => (0, 7),
            _ => (1, 6),
        };
        let band = band_columns
            .iter()
            .zip(bands)
            .find(|(column, _)| x == **column)
            .map(|(_, band)| band.shades());
        for y in top..=bottom {
            let color = if y == top || y == bottom || x == 0 || x == 13 {
                palette::OUTLINE
            } else if let Some((light, dark)) = band {
                if y >= bottom - 1 { dark } else { light }
            } else if y == top + 1 {
                palette::RESISTOR_HIGHLIGHT
            } else if y >= bottom - 1 {
                palette::RESISTOR_SHADE
            } else {
                palette::RESISTOR_BODY
            };
            canvas.set(x, y, color);
        }
    }
    canvas
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn decode(bands: [Band; 4]) -> f64 {
        let digit = |band: Band| Band::DIGITS.iter().position(|d| *d == band).unwrap() as f64;
        let exponent = match bands[2] {
            Band::Gold => -1,
            Band::Silver => -2,
            band => digit(band) as i32,
        };
        (digit(bands[0]) * 10.0 + digit(bands[1])) * 10f64.powi(exponent)
    }

    #[test]
    fn common_values_use_standard_codes() {
        use Band::*;
        assert_eq!(color_code(330.0), [Orange, Orange, Brown, Gold]);
        assert_eq!(color_code(220.0), [Red, Red, Brown, Gold]);
        assert_eq!(color_code(10_000.0), [Brown, Black, Orange, Gold]);
        assert_eq!(color_code(100_000.0), [Brown, Black, Yellow, Gold]);
        assert_eq!(color_code(1.0), [Brown, Black, Gold, Gold]);
        assert_eq!(color_code(10_000_000.0), [Brown, Black, Blue, Gold]);
        assert_eq!(color_code(99.6), [Brown, Black, Brown, Gold]);
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 256, rng_seed: proptest::test_runner::RngSeed::Fixed(0xB4ED_5B17), .. ProptestConfig::default() })]

        #[test]
        fn bands_decode_to_the_value_within_two_digit_rounding(log_ohms in 0.0f64..=7.0) {
            let ohms = 10f64.powf(log_ohms);
            let bands = color_code(ohms);
            let decoded = decode(bands);
            prop_assert!(Band::DIGITS.contains(&bands[0]) && bands[0] != Band::Black, "{bands:?}");
            prop_assert!((decoded - ohms).abs() / ohms <= 0.05 + 1e-9, "{ohms} -> {bands:?} = {decoded}");
        }
    }
}
