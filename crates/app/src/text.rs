pub const WINDOW_TITLE: &str = "bredboard";

/// Assembly palette, controls and status text.
pub mod assembly {
    pub const TITLE: &str = "FREE ASSEMBLY";
    pub const EDIT_COPY: &str = "EDIT A COPY";
    pub const RESISTOR: &str = "RESISTOR";
    pub const LED: &str = "LED + / -";
    pub const CAPACITOR: &str = "CAPACITOR";
    pub const DIODE: &str = "DIODE + / -";
    pub const NPN: &str = "NPN: B / C / E";
    pub const BUTTON: &str = "BUTTON";
    pub const SWITCH: &str = "SWITCH";
    pub const SELECT: &str = "SELECT";
    pub const WIRE: &str = "WIRE";
    pub const MOVE: &str = "MOVE LEAD";
    pub const REMOVE: &str = "REMOVE";
    pub const PARAMETER: &str = "PARAMETER";
    pub const VALUE: &str = "SET VALUE";
    pub const FLIP: &str = "FLIP + / -";
    pub const OPERATE: &str = "OPERATE";
    pub const RUN: &str = "RUN / PAUSE";
    pub const RESET: &str = "RESET TIME";
    pub const UNDO: &str = "UNDO";
    pub const CLEAR: &str = "CLEAR";
    pub const CIRCUITS: &str = "CIRCUITS";
    pub const HELP: &str = "Esc / RMB: cancel | Ctrl+Z: undo | Space: run";
    pub const WELCOME: &str = "Choose a part, then click its contacts in order.";
    pub const COPY: &str = "Editing a copy. CIRCUITS returns to the unchanged exercises.";
    pub const UPDATED: &str = "Updated; simulation restarted.";
    pub const UNDONE: &str = "Undone; simulation restarted.";
    pub const SELECT_PARAMETER: &str = "Select a component with a numeric parameter first.";
    pub const TWO_TERMINALS: &str =
        "Flip applies to two-terminal parts; move other leads individually.";
    pub const SELECT_CONTROL: &str = "Select a button or switch to operate it.";
    pub const MOVE_DESTINATION: &str = "Choose the new hole for this lead. Esc cancels.";
    pub const DUPLICATE_WIRE: &str = "Duplicate or zero-length wire ignored.";
    pub const CANCELLED: &str = "Gesture cancelled.";
    pub const INVALID_NUMBER: &str = "Enter a finite SI value, e.g. 1000 or 1e-4.";
}
