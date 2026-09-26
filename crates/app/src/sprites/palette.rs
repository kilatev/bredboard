//! Fixed sprite palette. Every colour has a legend symbol used by the golden
//! ASCII references in `docs/design/sprites/`.

pub type Rgb = [u8; 3];

pub(crate) const fn hex(value: u32) -> Rgb {
    [(value >> 16) as u8, (value >> 8) as u8, value as u8]
}

pub const OUTLINE: Rgb = hex(0x1b1b24);
pub const LEAD: Rgb = hex(0xc9d0d6);
pub const LEAD_SHADE: Rgb = hex(0x747d86);
pub const HOLE: Rgb = hex(0x2f3531);

pub const BOARD: Rgb = hex(0xe3d4a8);
pub const BOARD_GROOVE: Rgb = hex(0xc6b585);
pub const RAIL_RED: Rgb = hex(0xd63c32);
pub const RAIL_BLUE: Rgb = hex(0x3b5ccc);
pub const WIRE: Rgb = hex(0x27a3a8);
pub const WIRE_SHADE: Rgb = hex(0x15666b);

pub const RESISTOR_BODY: Rgb = hex(0xe8c688);
pub const RESISTOR_SHADE: Rgb = hex(0xb88b4c);
pub const RESISTOR_HIGHLIGHT: Rgb = hex(0xfbe9bd);

pub const METAL: Rgb = hex(0xb3bbc2);
pub const METAL_LIGHT: Rgb = hex(0xe4e9ed);
pub const METAL_DARK: Rgb = hex(0x78828b);
pub const CAP: Rgb = hex(0x33343f);
pub const CAP_LIGHT: Rgb = hex(0x5d6073);
pub const CAP_SPECULAR: Rgb = hex(0x8a8ea3);

/// Electrolytic capacitor can: blue sleeve, silver top vent, minus stripe.
pub const ELCAP_BODY: Rgb = hex(0x2e5fa3);
pub const ELCAP_SHADE: Rgb = hex(0x1c3d70);
pub const ELCAP_HIGHLIGHT: Rgb = hex(0x5b8fd6);
pub const ELCAP_TOP: Rgb = hex(0xc7ccd2);
pub const ELCAP_TOP_SHADE: Rgb = hex(0x8f959c);
pub const ELCAP_STRIPE: Rgb = hex(0xdfe4ea);

/// TO-92 transistor body: black plastic, D-shaped, flat face toward the viewer.
pub const TO92_BODY: Rgb = hex(0x24252c);
pub const TO92_SHADE: Rgb = hex(0x15151a);
pub const TO92_HIGHLIGHT: Rgb = hex(0x494a55);

/// Slide/changeover switch: black case, light slider knob.
pub const SWITCH_CASE: Rgb = hex(0x2a2b32);
pub const SWITCH_CASE_SHADE: Rgb = hex(0x18181d);
pub const SWITCH_TRACK: Rgb = hex(0x111114);
pub const SWITCH_SLIDER: Rgb = hex(0xd7dbe0);
pub const SWITCH_SLIDER_SHADE: Rgb = hex(0x9a9ea5);

/// Buzzer sound-wave marks, drawn only in the current-driven "sounding" state.
pub const SOUND_WAVE: Rgb = hex(0xa8e8ef);

/// Off-board 5 V supply block: dark case, red + and blue - terminals.
pub const SOURCE_CASE: Rgb = hex(0x30363a);
pub const SOURCE_CASE_LIGHT: Rgb = hex(0x4b5359);
pub const SOURCE_CASE_SHADE: Rgb = hex(0x1c2124);
pub const SOURCE_LABEL: Rgb = hex(0xd7dee2);

/// Resistor colour code bands: (light, dark) shades.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Band {
    Black,
    Brown,
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Violet,
    Grey,
    White,
    Gold,
    Silver,
}

impl Band {
    pub const DIGITS: [Band; 10] = [
        Band::Black,
        Band::Brown,
        Band::Red,
        Band::Orange,
        Band::Yellow,
        Band::Green,
        Band::Blue,
        Band::Violet,
        Band::Grey,
        Band::White,
    ];

    pub fn shades(self) -> (Rgb, Rgb) {
        let (light, dark) = match self {
            Band::Black => (0x2a2a33, 0x16161c),
            Band::Brown => (0x7e4a24, 0x4f2d15),
            Band::Red => (0xd63c32, 0x93241d),
            Band::Orange => (0xf08a24, 0xa95c14),
            Band::Yellow => (0xf4d03a, 0xb09220),
            Band::Green => (0x3fa34d, 0x276a31),
            Band::Blue => (0x3b5ccc, 0x263d8a),
            Band::Violet => (0x8b52c9, 0x5b3387),
            Band::Grey => (0x8e8e96, 0x5e5e66),
            Band::White => (0xf2f2f2, 0xbdbdc4),
            Band::Gold => (0xe8b83c, 0xa07a1c),
            Band::Silver => (0xc9ccd4, 0x8a8d96),
        };
        (hex(light), hex(dark))
    }
}

/// LED dome shades for one brightness state.
pub struct LedShades {
    pub rim: Rgb,
    pub dark: Rgb,
    pub mid: Rgb,
    pub light: Rgb,
    pub specular: Rgb,
    pub glow: Option<(Rgb, Rgb)>,
}

pub const RED_LED_OFF: LedShades = LedShades {
    rim: hex(0x5e1a1f),
    dark: hex(0x6e1e24),
    mid: hex(0x9c2c32),
    light: hex(0xc24a4c),
    specular: hex(0xe6a2a0),
    glow: None,
};
pub const RED_LED_DIM: LedShades = LedShades {
    rim: hex(0x8a1f1f),
    dark: hex(0xa3262a),
    mid: hex(0xd83a3a),
    light: hex(0xf07262),
    specular: hex(0xffd2c4),
    glow: None,
};
pub const RED_LED_ON: LedShades = LedShades {
    rim: hex(0xb8261e),
    dark: hex(0xe0322a),
    mid: hex(0xff5242),
    light: hex(0xffa184),
    specular: hex(0xffffff),
    glow: Some((hex(0xff8a6a), hex(0xffc0a8))),
};

/// Design-only LED colour families for T17 Part B (green, yellow, blue).
/// Not wired to any `ComponentKind`; used only to render the "lit" state
/// reference in `docs/design/sprites/future/` via `sprites::future`. Off and
/// dim states follow the same darkening pattern as the red LED once these
/// colours become a real part.
#[cfg(test)]
pub mod future {
    use super::{LedShades, Rgb, hex};

    const GREEN_RIM: Rgb = hex(0x1e3a24);
    const GREEN_MID: Rgb = hex(0x3ad84e);
    const GREEN_LIGHT: Rgb = hex(0x8af0a0);
    const GREEN_SPECULAR: Rgb = hex(0xf4fff2);
    const GREEN_GLOW: Rgb = hex(0x6ae06a);
    const GREEN_GLOW_LIGHT: Rgb = hex(0xa0ffa0);
    pub const GREEN_LED_LIT: LedShades = LedShades {
        rim: GREEN_RIM,
        dark: GREEN_RIM,
        mid: GREEN_MID,
        light: GREEN_LIGHT,
        specular: GREEN_SPECULAR,
        glow: Some((GREEN_GLOW, GREEN_GLOW_LIGHT)),
    };

    const YELLOW_RIM: Rgb = hex(0x554512);
    const YELLOW_MID: Rgb = hex(0xffe042);
    const YELLOW_LIGHT: Rgb = hex(0xfff0a0);
    const YELLOW_SPECULAR: Rgb = hex(0xfffbe6);
    const YELLOW_GLOW: Rgb = hex(0xffce4a);
    const YELLOW_GLOW_LIGHT: Rgb = hex(0xffe86a);
    pub const YELLOW_LED_LIT: LedShades = LedShades {
        rim: YELLOW_RIM,
        dark: YELLOW_RIM,
        mid: YELLOW_MID,
        light: YELLOW_LIGHT,
        specular: YELLOW_SPECULAR,
        glow: Some((YELLOW_GLOW, YELLOW_GLOW_LIGHT)),
    };

    const BLUE_RIM: Rgb = hex(0x152048);
    const BLUE_MID: Rgb = hex(0x4268ff);
    const BLUE_LIGHT: Rgb = hex(0x9ab0ff);
    const BLUE_SPECULAR: Rgb = hex(0xeef3ff);
    const BLUE_GLOW: Rgb = hex(0x6a8aff);
    const BLUE_GLOW_LIGHT: Rgb = hex(0x8aa0ff);
    pub const BLUE_LED_LIT: LedShades = LedShades {
        rim: BLUE_RIM,
        dark: BLUE_RIM,
        mid: BLUE_MID,
        light: BLUE_LIGHT,
        specular: BLUE_SPECULAR,
        glow: Some((BLUE_GLOW, BLUE_GLOW_LIGHT)),
    };

    /// Legend entries for the colours above; appended to the main legend.
    pub(super) const LEGEND: &[(Rgb, char)] = &[
        (GREEN_RIM, '"'),
        (GREEN_MID, '+'),
        (GREEN_LIGHT, ','),
        (GREEN_SPECULAR, '-'),
        (GREEN_GLOW, '/'),
        (GREEN_GLOW_LIGHT, ':'),
        (YELLOW_RIM, ';'),
        (YELLOW_MID, '<'),
        (YELLOW_LIGHT, '='),
        (YELLOW_SPECULAR, '>'),
        (YELLOW_GLOW, '['),
        (YELLOW_GLOW_LIGHT, ']'),
        (BLUE_RIM, '_'),
        (BLUE_MID, '`'),
        (BLUE_LIGHT, '{'),
        (BLUE_SPECULAR, '|'),
        (BLUE_GLOW, '}'),
        (BLUE_GLOW_LIGHT, '~'),
    ];
}

/// Legend used by golden ASCII files. Symbols must stay unique.
#[cfg(test)]
const LEGEND: &[(Rgb, char)] = &[
    (OUTLINE, 'K'),
    (LEAD, 'L'),
    (LEAD_SHADE, 'l'),
    (HOLE, 'o'),
    (RESISTOR_BODY, 'B'),
    (RESISTOR_SHADE, 'b'),
    (RESISTOR_HIGHLIGHT, 'h'),
    (METAL, 'm'),
    (METAL_LIGHT, 'M'),
    (METAL_DARK, 'n'),
    (CAP, 'c'),
    (CAP_LIGHT, 'C'),
    (CAP_SPECULAR, 's'),
    (hex(0x2a2a33), '0'),
    (hex(0x16161c), ')'),
    (hex(0x7e4a24), '1'),
    (hex(0x4f2d15), '!'),
    (hex(0xd63c32), '2'),
    (hex(0x93241d), '@'),
    (hex(0xf08a24), '3'),
    (hex(0xa95c14), '#'),
    (hex(0xf4d03a), '4'),
    (hex(0xb09220), '$'),
    (hex(0x3fa34d), '5'),
    (hex(0x276a31), '%'),
    (hex(0x3b5ccc), '6'),
    (hex(0x263d8a), '^'),
    (hex(0x8b52c9), '7'),
    (hex(0x5b3387), '&'),
    (hex(0x8e8e96), '8'),
    (hex(0x5e5e66), '*'),
    (hex(0xf2f2f2), '9'),
    (hex(0xbdbdc4), '('),
    (hex(0xe8b83c), 'G'),
    (hex(0xa07a1c), 'g'),
    (hex(0xc9ccd4), 'S'),
    (hex(0x8a8d96), 'z'),
    (hex(0x5e1a1f), 'r'),
    (hex(0x6e1e24), 'd'),
    (hex(0x9c2c32), 'e'),
    (hex(0xc24a4c), 'f'),
    (hex(0xe6a2a0), 'p'),
    (hex(0x8a1f1f), 'R'),
    (hex(0xa3262a), 'D'),
    (hex(0xd83a3a), 'E'),
    (hex(0xf07262), 'F'),
    (hex(0xffd2c4), 'P'),
    (hex(0xb8261e), 'u'),
    (hex(0xe0322a), 'v'),
    (hex(0xff5242), 'w'),
    (hex(0xffa184), 'x'),
    (hex(0xffffff), 'W'),
    (hex(0xff8a6a), 'y'),
    (hex(0xffc0a8), 'Y'),
    (ELCAP_BODY, 'A'),
    (ELCAP_SHADE, 'a'),
    (ELCAP_HIGHLIGHT, 'H'),
    (ELCAP_TOP, 'I'),
    (ELCAP_TOP_SHADE, 'i'),
    (ELCAP_STRIPE, 'J'),
    (TO92_BODY, 'N'),
    (TO92_SHADE, 'O'),
    (TO92_HIGHLIGHT, 'j'),
    (SWITCH_CASE, 'X'),
    (SWITCH_CASE_SHADE, 'k'),
    (SWITCH_TRACK, 'Z'),
    (SWITCH_SLIDER, 'Q'),
    (SWITCH_SLIDER_SHADE, 'q'),
    (SOURCE_CASE, 'U'),
    (SOURCE_CASE_LIGHT, 't'),
    (SOURCE_CASE_SHADE, 'V'),
    (SOURCE_LABEL, 'T'),
    (SOUND_WAVE, '\''),
];

/// Full legend: the parts above plus the design-only Part B colours.
#[cfg(test)]
pub fn legend() -> Vec<(Rgb, char)> {
    LEGEND
        .iter()
        .copied()
        .chain(future::LEGEND.iter().copied())
        .collect()
}

#[cfg(test)]
pub fn symbol(color: Rgb) -> char {
    legend()
        .iter()
        .find(|(c, _)| *c == color)
        .map_or('?', |(_, s)| *s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn legend_symbols_and_colours_are_unique() {
        let legend = legend();
        let symbols: BTreeSet<_> = legend.iter().map(|(_, s)| *s).collect();
        let colours: BTreeSet<_> = legend.iter().map(|(c, _)| *c).collect();
        assert_eq!(symbols.len(), legend.len());
        assert_eq!(colours.len(), legend.len());
        assert!(!symbols.contains(&'.') && !symbols.contains(&'?'));
    }
}
