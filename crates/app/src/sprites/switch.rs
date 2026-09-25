use bredboard_core::{Component, ControlState};

use super::palette;
use super::{PartArt, PartContext, PixelCanvas};

/// Changeover (SPDT) switch. Slide and toggle switch designs reuse this body
/// (see the design reference, Part B); only this changeover kind is wired to
/// core control state today.
pub struct Switch;

impl PartArt for Switch {
    fn axis_pins(&self) -> [&'static str; 2] {
        ["normally_closed", "normally_open"]
    }

    fn state_count(&self) -> usize {
        2
    }

    fn state(&self, context: &PartContext) -> usize {
        usize::from(context.control == Some(ControlState::SwitchNormallyOpen))
    }

    fn body(&self, _component: &Component, state: usize) -> PixelCanvas {
        body(state == 1)
    }
}

/// 14 x 8 black slide switch; the slider sits over the left (NC) or right
/// (NO) end depending on the control state.
pub fn body(toward_no: bool) -> PixelCanvas {
    let mut canvas = PixelCanvas::new(14, 8);
    for y in 0..8 {
        for x in 0..14 {
            let color = if x == 0 || x == 13 || y == 0 || y == 7 {
                palette::OUTLINE
            } else if y == 1 {
                palette::SWITCH_CASE
            } else if y == 6 {
                palette::SWITCH_CASE_SHADE
            } else if y == 3 || y == 4 {
                palette::SWITCH_TRACK
            } else {
                palette::SWITCH_CASE
            };
            canvas.set(x, y, color);
        }
    }
    for (x, y) in [(0, 0), (13, 0), (0, 7), (13, 7)] {
        canvas.clear(x, y);
    }
    let slider_x = if toward_no { 8 } else { 2 };
    for y in 2..6 {
        for dx in 0..4 {
            let x = slider_x + dx;
            let color = if y == 2 {
                palette::SWITCH_SLIDER
            } else if y == 5 {
                palette::SWITCH_SLIDER_SHADE
            } else {
                palette::SWITCH_SLIDER
            };
            canvas.set(x, y, color);
        }
    }
    for y in 2..6 {
        canvas.set(slider_x, y, palette::OUTLINE);
        canvas.set(slider_x + 3, y, palette::OUTLINE);
    }
    canvas
}
