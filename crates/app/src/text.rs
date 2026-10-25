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

pub mod learning {
    pub const BOARD_TIP: &str = "Same row: A-E are joined, F-J are joined. The middle gap separates them. + and - rails run vertically.";
    pub const STILL_BUILDING: &str = "STILL BUILDING";
    pub const START: &str = "START HERE: BUILD & LEARN";
    pub const LIGHT: &str = "1. First light - copy it, then try your own layout";
    pub const CAPACITOR: &str = "2. Store a little energy - charge and discharge";
    pub const TRANSISTOR: &str = "3. A small control switches an LED";
    pub const SCHEMATIC: &str = "SCHEMATIC";
    pub const MY_BOARD: &str = "MY BOARD";
    pub const READY: &str = "READY BOARD";
    pub const READ_ONLY: &str = "READY EXAMPLE - your own board stays unchanged";
    pub const BUILD: &str = "BUILD ON MY BOARD";
    pub const LIBRARY: &str = "CIRCUIT LIBRARY";
    pub const GUIDE_ON: &str = "GUIDE: ON";
    pub const GUIDE_OFF: &str = "GUIDE: OFF";
    pub const HINT: &str = "SHOW HOLES";
    pub const PLACE_NEXT: &str = "PLACE NEXT";
    pub const NEXT_LESSON: &str = "NEXT LESSON";
    pub const LED_DONE: &str = "Select B1 > OPERATE, then RUN. The LED lights. SELECT D1 to read current. Try moving a wire; Undo restores it.";
    pub const RC_DONE: &str = "Select S1 > OPERATE to switch charge/discharge. RUN, then SELECT C1 to see its pin voltage. Undo is always available.";
    pub const OWN_LAYOUT: &str = "Your layout can be different. SELECT a part to inspect it. READY BOARD is always available. Turn GUIDE on if you want help.";
    pub const LED_PIN: &str = "Long leg: anode. Flat case edge: cathode.";
    pub const CAP_PIN: &str = "The striped side is negative.";
    pub const NPN_PIN: &str = "Check B/C/E on your actual part's datasheet.";
    pub const BUTTON_PIN: &str = "Use opposite contacts; paired legs are already joined.";
    pub const RESISTOR_PIN: &str = "Either direction works.";
    pub const HARDWARE: &str = "Physical build: disconnect power while wiring. Use a current-limited 5 V DC supply. Check LED/capacitor polarity and your transistor's actual pinout. Hole names describe this 30-row board.";
}
