mod exercise_catalog;
mod sprites;
mod text;

use bevy::audio::{AudioPlayer, AudioSink, Pitch, PlaybackSettings};
use bevy::camera::ScalingMode;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bredboard_core::{
    Action, Component, ComponentId, ComponentKind, ControlState, Project, SimulationState,
    advance_steps, apply_actions, compile_topology,
};

const LED_JSON: &str = include_str!("../../../fixtures/projects/led-bench.json");
const RC_JSON: &str = include_str!("../../../fixtures/projects/rc-bench.json");
const TRANSISTOR_JSON: &str = include_str!("../../../fixtures/projects/transistor-bench.json");
const E1_JSON: &str = include_str!("../../../fixtures/projects/e1-first-light.json");
const E2_JSON: &str = include_str!("../../../fixtures/projects/e2-push-button-switch.json");
const E3_JSON: &str = include_str!("../../../fixtures/projects/e3-two-leds-in-series.json");
const E4_JSON: &str = include_str!("../../../fixtures/projects/e4-two-leds-in-parallel.json");
const E7_JSON: &str = include_str!("../../../fixtures/projects/e7-buzzer-doorbell.json");
const E8_JSON: &str = include_str!("../../../fixtures/projects/e8-transistor-switch.json");
const E9_JSON: &str = include_str!("../../../fixtures/projects/e9-logical-and.json");
const E10_JSON: &str = include_str!("../../../fixtures/projects/e10-smooth-fade.json");
const E5_JSON: &str = include_str!("../../../fixtures/projects/e5-brightness-dial.json");
const E6_JSON: &str = include_str!("../../../fixtures/projects/e6-light-reactive-led.json");
const E11_JSON: &str = include_str!("../../../fixtures/projects/e11-two-way-switch.json");
const E12_JSON: &str = include_str!("../../../fixtures/projects/e12-mixed-wiring.json");
const E13_JSON: &str = include_str!("../../../fixtures/projects/e13-capacitor-against-bounce.json");
const E14_JSON: &str = include_str!("../../../fixtures/projects/e14-buttons-or.json");
const E15_JSON: &str = include_str!("../../../fixtures/projects/e15-transistor-inverter.json");
const E16_JSON: &str = include_str!("../../../fixtures/projects/e16-automatic-night-light.json");
const E17_JSON: &str = include_str!("../../../fixtures/projects/e17-light-alarm.json");
const E18_JSON: &str = include_str!("../../../fixtures/projects/e18-volume-control.json");
const E19_JSON: &str =
    include_str!("../../../fixtures/projects/e19-three-independent-branches.json");
const E20_JSON: &str = include_str!("../../../fixtures/projects/e20-turn-on-delay.json");
const E21_JSON: &str =
    include_str!("../../../fixtures/projects/e21-adjustable-night-light-threshold.json");
const E22_JSON: &str = include_str!("../../../fixtures/projects/e22-mixed-logic.json");
const E23_JSON: &str = include_str!("../../../fixtures/projects/e23-light-and-sound-together.json");
const E24_JSON: &str =
    include_str!("../../../fixtures/projects/e24-capacitor-charge-and-discharge.json");
const E25_JSON: &str = include_str!("../../../fixtures/projects/e25-transistor-and.json");
const E26_JSON: &str = include_str!("../../../fixtures/projects/e26-transistor-or.json");
const E27_JSON: &str =
    include_str!("../../../fixtures/projects/e27-shared-brightness-control.json");
const E28_JSON: &str = include_str!("../../../fixtures/projects/e28-power-source-selector.json");
const E29_JSON: &str = include_str!("../../../fixtures/projects/e29-sensitivity-detector.json");
const E30_JSON: &str = include_str!("../../../fixtures/projects/e30-two-transistor-flasher.json");
const C01_S01_01_JSON: &str =
    include_str!("../../../fixtures/projects/c01-s01-01-first-safe-light.json");
const C01_S01_02_JSON: &str =
    include_str!("../../../fixtures/projects/c01-s01-02-button-and-switch.json");
const C01_S01_03_JSON: &str =
    include_str!("../../../fixtures/projects/c01-s01-03-series-and-parallel.json");
const C01_S01_04_JSON: &str =
    include_str!("../../../fixtures/projects/c01-s01-04-potentiometer-dimmer.json");
const C01_S01_05_JSON: &str =
    include_str!("../../../fixtures/projects/c01-s01-05-reverse-polarity.json");
const C01_S01_06_JSON: &str =
    include_str!("../../../fixtures/projects/c01-s01-06-smooth-fade.json");
const C01_S02_01_JSON: &str =
    include_str!("../../../fixtures/projects/c01-s02-01-transistor-key.json");
const C01_S02_02_JSON: &str =
    include_str!("../../../fixtures/projects/c01-s02-02-dusk-night-light.json");
const C01_S02_06_JSON: &str =
    include_str!("../../../fixtures/projects/c01-s02-06-transistor-logic.json");
const C02_S03_03_JSON: &str =
    include_str!("../../../fixtures/projects/c02-s03-03-logic-gates.json");
const C02_S03_01_JSON: &str =
    include_str!("../../../fixtures/projects/c02-s03-01-555-flasher.json");
const C02_S03_02_JSON: &str =
    include_str!("../../../fixtures/projects/c02-s03-02-555-monostable.json");
const C02_S03_04_JSON: &str = include_str!("../../../fixtures/projects/c02-s03-04-rs-latch.json");
const C02_S03_05_JSON: &str = include_str!("../../../fixtures/projects/c02-s03-05-debounce.json");
const C02_S03_06_JSON: &str = include_str!("../../../fixtures/projects/c02-s03-06-thermostat.json");
const C02_S03_07_JSON: &str =
    include_str!("../../../fixtures/projects/c02-s03-07-light-theremin.json");
const C03_S04_03_JSON: &str =
    include_str!("../../../fixtures/projects/c03-s04-03-button-counter.json");
const C03_S04_05_JSON: &str =
    include_str!("../../../fixtures/projects/c03-s04-05-shift-register.json");
const C03_S04_04_JSON: &str =
    include_str!("../../../fixtures/projects/c03-s04-04-binary-counter.json");
const C03_S04_01_JSON: &str =
    include_str!("../../../fixtures/projects/c03-s04-01-running-lights.json");
const C03_S04_02_JSON: &str =
    include_str!("../../../fixtures/projects/c03-s04-02-electronic-dice.json");
const C03_S04_09_JSON: &str =
    include_str!("../../../fixtures/projects/c03-s04-09-dual-555-noisemaker.json");
const C03_S05_01_JSON: &str =
    include_str!("../../../fixtures/projects/c03-s05-01-four-bit-adder.json");
const C03_S05_02_JSON: &str =
    include_str!("../../../fixtures/projects/c03-s05-02-four-bit-subtractor.json");
const C03_S05_03_JSON: &str =
    include_str!("../../../fixtures/projects/c03-s05-03-pedestrian-signal.json");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Circuit {
    Led,
    Rc,
    Transistor,
    E1,
    E2,
    E3,
    E4,
    E7,
    E8,
    E9,
    E10,
    E5,
    E6,
    E11,
    E12,
    E13,
    E14,
    E15,
    E16,
    E17,
    E18,
    E19,
    E20,
    E21,
    E22,
    E23,
    E24,
    E25,
    E26,
    E27,
    E28,
    E29,
    E30,
    C01S01_01,
    C01S01_02,
    C01S01_03,
    C01S01_04,
    C01S01_05,
    C01S01_06,
    C01S02_01,
    C01S02_02,
    C01S02_06,
    C02S03_03,
    C02S03_01,
    C02S03_02,
    C02S03_04,
    C02S03_05,
    C02S03_06,
    C02S03_07,
    C03S04_03,
    C03S04_05,
    C03S04_04,
    C03S04_01,
    C03S04_02,
    C03S04_09,
    C03S05_01,
    C03S05_02,
    C03S05_03,
}

/// One bench control button: its label, the component it drives, and
/// whether that component is a changeover switch (vs. a momentary button).
struct ControlSpec {
    label: &'static str,
    component: &'static str,
    is_switch: bool,
}
/// A continuous dial/slider control: its label and the potentiometer or
/// photoresistor component it drives.
struct DialSpec {
    label: &'static str,
    component: &'static str,
}
impl Circuit {
    #[cfg(test)]
    fn all() -> [Self; 58] {
        [
            Self::C01S01_01,
            Self::C01S01_02,
            Self::C01S01_03,
            Self::C01S01_04,
            Self::C01S01_05,
            Self::C01S01_06,
            Self::C01S02_01,
            Self::C01S02_02,
            Self::C01S02_06,
            Self::C02S03_03,
            Self::C02S03_01,
            Self::C02S03_02,
            Self::C02S03_04,
            Self::C02S03_05,
            Self::C02S03_06,
            Self::C02S03_07,
            Self::C03S04_03,
            Self::C03S04_05,
            Self::C03S04_04,
            Self::C03S04_01,
            Self::C03S04_02,
            Self::C03S04_09,
            Self::C03S05_01,
            Self::C03S05_02,
            Self::C03S05_03,
            Self::Led,
            Self::Rc,
            Self::Transistor,
            Self::E1,
            Self::E2,
            Self::E3,
            Self::E4,
            Self::E7,
            Self::E8,
            Self::E9,
            Self::E10,
            Self::E5,
            Self::E6,
            Self::E11,
            Self::E12,
            Self::E13,
            Self::E14,
            Self::E15,
            Self::E16,
            Self::E17,
            Self::E18,
            Self::E19,
            Self::E20,
            Self::E21,
            Self::E22,
            Self::E23,
            Self::E24,
            Self::E25,
            Self::E26,
            Self::E27,
            Self::E28,
            Self::E29,
            Self::E30,
        ]
    }
    fn json(self) -> &'static str {
        match self {
            Self::Led => LED_JSON,
            Self::Rc => RC_JSON,
            Self::Transistor => TRANSISTOR_JSON,
            Self::E1 => E1_JSON,
            Self::E2 => E2_JSON,
            Self::E3 => E3_JSON,
            Self::E4 => E4_JSON,
            Self::E7 => E7_JSON,
            Self::E8 => E8_JSON,
            Self::E9 => E9_JSON,
            Self::E10 => E10_JSON,
            Self::E5 => E5_JSON,
            Self::E6 => E6_JSON,
            Self::E11 => E11_JSON,
            Self::E12 => E12_JSON,
            Self::E13 => E13_JSON,
            Self::E14 => E14_JSON,
            Self::E15 => E15_JSON,
            Self::E16 => E16_JSON,
            Self::E17 => E17_JSON,
            Self::E18 => E18_JSON,
            Self::E19 => E19_JSON,
            Self::E20 => E20_JSON,
            Self::E21 => E21_JSON,
            Self::E22 => E22_JSON,
            Self::E23 => E23_JSON,
            Self::E24 => E24_JSON,
            Self::E25 => E25_JSON,
            Self::E26 => E26_JSON,
            Self::E27 => E27_JSON,
            Self::E28 => E28_JSON,
            Self::E29 => E29_JSON,
            Self::E30 => E30_JSON,
            Self::C01S01_01 => C01_S01_01_JSON,
            Self::C01S01_02 => C01_S01_02_JSON,
            Self::C01S01_03 => C01_S01_03_JSON,
            Self::C01S01_04 => C01_S01_04_JSON,
            Self::C01S01_05 => C01_S01_05_JSON,
            Self::C01S01_06 => C01_S01_06_JSON,
            Self::C01S02_01 => C01_S02_01_JSON,
            Self::C01S02_02 => C01_S02_02_JSON,
            Self::C01S02_06 => C01_S02_06_JSON,
            Self::C02S03_03 => C02_S03_03_JSON,
            Self::C02S03_01 => C02_S03_01_JSON,
            Self::C02S03_02 => C02_S03_02_JSON,
            Self::C02S03_04 => C02_S03_04_JSON,
            Self::C02S03_05 => C02_S03_05_JSON,
            Self::C02S03_06 => C02_S03_06_JSON,
            Self::C02S03_07 => C02_S03_07_JSON,
            Self::C03S04_03 => C03_S04_03_JSON,
            Self::C03S04_05 => C03_S04_05_JSON,
            Self::C03S04_04 => C03_S04_04_JSON,
            Self::C03S04_01 => C03_S04_01_JSON,
            Self::C03S04_02 => C03_S04_02_JSON,
            Self::C03S04_09 => C03_S04_09_JSON,
            Self::C03S05_01 => C03_S05_01_JSON,
            Self::C03S05_02 => C03_S05_02_JSON,
            Self::C03S05_03 => C03_S05_03_JSON,
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Led => "LED + RESISTOR",
            Self::Rc => "CAPACITOR CHARGE / DISCHARGE",
            Self::Transistor => "TRANSISTOR SWITCH",
            Self::E1 => "E1: FIRST LIGHT",
            Self::E2 => "E2: PUSH-BUTTON SWITCH",
            Self::E3 => "E3: TWO LEDS IN SERIES",
            Self::E4 => "E4: TWO LEDS IN PARALLEL",
            Self::E7 => "E7: BUZZER DOORBELL",
            Self::E8 => "E8: TRANSISTOR SWITCH",
            Self::E9 => "E9: LOGICAL AND",
            Self::E10 => "E10: SMOOTH FADE",
            Self::E5 => "E5: BRIGHTNESS DIAL",
            Self::E6 => "E6: LIGHT-REACTIVE LED",
            Self::E11 => "E11: TWO-WAY SWITCH",
            Self::E12 => "E12: MIXED SERIES/PARALLEL WIRING",
            Self::E13 => "E13: CAPACITOR AGAINST BOUNCE",
            Self::E14 => "E14: BUTTONS OR",
            Self::E15 => "E15: TRANSISTOR INVERTER",
            Self::E16 => "E16: AUTOMATIC NIGHT LIGHT",
            Self::E17 => "E17: LIGHT ALARM",
            Self::E18 => "E18: VOLUME CONTROL",
            Self::E19 => "E19: THREE INDEPENDENT BRANCHES",
            Self::E20 => "E20: TURN-ON DELAY",
            Self::E21 => "E21: ADJUSTABLE NIGHT-LIGHT THRESHOLD",
            Self::E22 => "E22: MIXED LOGIC",
            Self::E23 => "E23: LIGHT AND SOUND TOGETHER",
            Self::E24 => "E24: CAPACITOR CHARGE AND DISCHARGE",
            Self::E25 => "E25: TRANSISTOR AND",
            Self::E26 => "E26: TRANSISTOR OR",
            Self::E27 => "E27: SHARED BRIGHTNESS CONTROL",
            Self::E28 => "E28: POWER SOURCE SELECTOR",
            Self::E29 => "E29: SENSITIVITY DETECTOR",
            Self::E30 => "E30: TWO-TRANSISTOR FLASHER",
            Self::C01S01_01 => "C01-S01-01: FIRST SAFE LIGHT",
            Self::C01S01_02 => "C01-S01-02: BUTTON AND CHANGEOVER SWITCH",
            Self::C01S01_03 => "C01-S01-03: SERIES AND PARALLEL LED PATHS",
            Self::C01S01_04 => "C01-S01-04: POTENTIOMETER DIMMER",
            Self::C01S01_05 => "C01-S01-05: REVERSE-POLARITY PROTECTION",
            Self::C01S01_06 => "C01-S01-06: SMOOTH FADE",
            Self::C01S02_01 => "C01-S02-01: BC547 TRANSISTOR KEY",
            Self::C01S02_02 => "C01-S02-02: DUSK NIGHT LIGHT",
            Self::C01S02_06 => "C01-S02-06: TRANSISTOR LOGIC",
            Self::C02S03_03 => "C02-S03-03: 74HC LOGIC GATES",
            Self::C02S03_01 => "C02-S03-01: 555 FLASHER",
            Self::C02S03_02 => "C02-S03-02: 555 MONOSTABLE LIGHT",
            Self::C02S03_04 => "C02-S03-04: 74HC00 RS LATCH",
            Self::C02S03_05 => "C02-S03-05: SCHMITT DEBOUNCE",
            Self::C02S03_06 => "C02-S03-06: COMPARATOR THERMOSTAT",
            Self::C02S03_07 => "C02-S03-07: LIGHT THEREMIN",
            Self::C03S04_03 => "C03-S04-03: BUTTON COUNTER DISPLAY",
            Self::C03S04_05 => "C03-S04-05: MANUAL SHIFT REGISTER",
            Self::C03S04_04 => "C03-S04-04: BINARY COUNTER",
            Self::C03S04_01 => "C03-S04-01: RUNNING LIGHTS",
            Self::C03S04_02 => "C03-S04-02: ELECTRONIC DICE",
            Self::C03S04_09 => "C03-S04-09: DUAL 555 NOISEMAKER",
            Self::C03S05_01 => "C03-S05-01: FOUR-BIT ADDER",
            Self::C03S05_02 => "C03-S05-02: FOUR-BIT SUBTRACTOR",
            Self::C03S05_03 => "C03-S05-03: PEDESTRIAN SIGNAL",
        }
    }
    /// One-paragraph explanation and a short player task, shown together in
    /// the bench's secondary text slot. Empty for the three MVP benches,
    /// which keep their original generic hover hint instead (see
    /// `spawn_bench`); this task does not change their behavior or layout.
    fn explanation_and_task(self) -> (&'static str, &'static str) {
        match self {
            Self::Led | Self::Rc | Self::Transistor => ("", ""),
            Self::C01S01_01 => (
                "A 470 ohm resistor limits the 5 V supply current, so the LED lights safely without a destructive failure outcome.",
                "Task: run the fixture and confirm the calculated LED current stays bounded.",
            ),
            Self::C01S01_02 => (
                "The button enables the circuit and the changeover switch selects exactly one resistor-and-LED branch.",
                "Task: hold the button and toggle the switch; confirm only the selected LED lights.",
            ),
            Self::C01S01_03 => (
                "The fixture places a two-LED series path beside independent resistor paths so current and topology can be compared.",
                "Task: run the board and compare the calculated currents in the series and parallel paths.",
            ),
            Self::C01S01_04 => (
                "The potentiometer is used as a two-terminal rheostat with a 330 ohm safety floor, changing LED current continuously.",
                "Task: drag the dial and confirm the LED current changes monotonically.",
            ),
            Self::C01S01_05 => (
                "A rectifier diode in series with the LED branch conducts in the forward direction and blocks a reversed source.",
                "Task: run the forward-polarity fixture and inspect the diode's anode/cathode labels before reversing the source in a later experiment.",
            ),
            Self::C01S01_06 => (
                "The button charges a 470 uF capacitor through 100 ohms; after release, the capacitor discharges through the 470 ohm LED branch.",
                "Task: hold the button, release it, and observe the calculated capacitor voltage and LED fade.",
            ),
            Self::C01S02_01 => (
                "A fixed BC547 NPN transistor uses a small button current through 10 kΩ to switch the LED branch through its collector and emitter.",
                "Task: press and hold B1; confirm the calculated collector and LED currents rise together.",
            ),
            Self::C01S02_02 => (
                "The photoresistor and two-terminal threshold control bias a BC547 so the LED becomes brighter as the simulated room gets darker.",
                "Task: sweep R3 from bright to dark and adjust RV1; compare the LED current at each threshold.",
            ),
            Self::C01S02_06 => (
                "Two button input rails drive five BC547 stages: two transistors form AND, two form OR, and the fifth is a calculated NOT output.",
                "Task: test all four A/B button combinations and record the calculated AND, OR, and NOT LED states.",
            ),
            Self::C02S03_03 => (
                "Four calculated 74HC-style gates show AND, OR, NAND, and XOR behavior from two button input rails.",
                "Task: test all four A/B combinations and compare the four LED outputs with their truth tables.",
            ),
            Self::C02S03_01 => (
                "An NE555 astable timer charges and discharges a capacitor so the LED flashes at a calculated fixed-step frequency.",
                "Task: run the circuit and drag RV1; compare the calculated capacitor waveform and LED transitions.",
            ),
            Self::C02S03_02 => (
                "A button triggers a calculated NE555 monostable pulse whose duration is set by the RC network.",
                "Task: press and release S1, then observe the calculated LED pulse and capacitor voltage.",
            ),
            Self::C02S03_04 => (
                "Two calculated NAND gates form a capacitor-backed RS latch that retains the set output after release.",
                "Task: press SET, release it, and verify the LED remains on until RESET is pressed.",
            ),
            Self::C02S03_05 => (
                "A calculated Schmitt inverter and D flip-flop turn a button's RC transition into a stable output.",
                "Task: press S1 and compare the filtered capacitor and calculated LED output.",
            ),
            Self::C02S03_06 => (
                "An LM393-style calculated comparator compares an NTC thermistor divider with an adjustable threshold.",
                "Task: drag TH1 from cold to hot and adjust RV1; record where the calculated LED changes state.",
            ),
            Self::C02S03_07 => (
                "An NE555 timer and photoresistor form a calculated light-controlled oscillator driving the speaker load.",
                "Task: drag TH1 from dark to bright and compare the calculated timer transition rate.",
            ),
            Self::C03S04_03 => (
                "A button supplies calculated clock edges to a bounded decimal counter. Four electrical bit outputs drive a common-cathode seven-segment decoder.",
                "Task: press S1 one edge at a time and verify that the calculated display advances from 0 through 9.",
            ),
            Self::C03S04_05 => (
                "A calculated eight-bit shift register samples DATA on each rising CLOCK edge and copies the stored pattern to its outputs on LATCH.",
                "Task: set DATA, press CLOCK once, then press LATCH and inspect the calculated output state.",
            ),
            Self::C03S04_04 => (
                "A calculated bounded counter advances on each button edge and drives four current-limited LEDs as binary bits.",
                "Task: press S1 repeatedly and record the four calculated bit states from 0 through 9.",
            ),
            Self::C03S04_01 => (
                "A calculated one-hot counter advances on each button edge and drives one current-limited LED at a time.",
                "Task: press S1 repeatedly and follow the calculated light as it moves through all ten positions.",
            ),
            Self::C03S04_02 => (
                "A calculated modulo-six counter decodes each state into the seven-dot pattern of a die face.",
                "Task: press S1 repeatedly and inspect the calculated dot pattern for faces one through six.",
            ),
            Self::C03S04_09 => (
                "Two calculated 555 timer stages form a bounded coupled oscillator, with two dials changing its timing and a speaker load reading the output.",
                "Task: drag RV1 and RV2 separately and compare the calculated timer and speaker readings.",
            ),
            Self::C03S05_01 => (
                "Eight electrical changeover inputs feed a calculated four-bit adder. Four sum outputs and carry drive current-limited LEDs.",
                "Task: set A and B with the eight input switches and verify the calculated sum and carry outputs.",
            ),
            Self::C03S05_02 => (
                "The calculated four-bit arithmetic unit uses two's-complement subtraction to show a bounded A minus B result and carry state.",
                "Task: set the eight input switches and verify the calculated difference and carry output.",
            ),
            Self::C03S05_03 => (
                "A calculated five-state counter advances the pedestrian signal phases from fixed-step button edges, with one current-limited phase LED active at a time.",
                "Task: press S1 and record the calculated five-phase cycle.",
            ),
            Self::E1 => (
                "A resistor limits current from the 5 V supply so the LED lights safely and stays lit.",
                "Task: run the circuit and confirm the LED lights immediately, with no switch needed.",
            ),
            Self::E2 => (
                "The same LED and resistor, now gated by a momentary button: current only flows, and the LED lights, while the button is held.",
                "Task: press and hold the button, then release it and confirm the LED goes dark.",
            ),
            Self::E3 => (
                "Two LEDs share one resistor and one current path, so the same current lights both LEDs together.",
                "Task: run the circuit and confirm both LEDs light at once.",
            ),
            Self::E4 => (
                "Two independent resistor-and-LED branches share the same 5 V supply, so each LED gets its own steady current regardless of the other.",
                "Task: run the circuit and confirm both LEDs light independently of each other.",
            ),
            Self::E7 => (
                "A momentary button connects the buzzer directly to the 5 V supply, so it draws current, and its sprite shows sounding, only while the button is held.",
                "Task: press and hold the button and confirm the buzzer's sound-wave marks appear, then disappear on release.",
            ),
            Self::E8 => (
                "A small button current through a base resistor turns the transistor on, which then switches a much larger LED current between its collector and emitter.",
                "Task: press and hold the button and confirm the LED lights, then goes dark on release.",
            ),
            Self::E9 => (
                "Two momentary buttons wired in series both need to be pressed to complete the path to the resistor and LED, the same as a logical AND gate.",
                "Task: confirm the LED lights only when both buttons are held at once, not when either is held alone.",
            ),
            Self::E10 => (
                "Holding the button charges a capacitor through a resistor; the LED, wired to the same charging node, brightens smoothly as the capacitor's voltage rises instead of snapping on.",
                "Task: press and hold the button and watch the LED brighten gradually rather than instantly.",
            ),
            Self::E5 => (
                "A potentiometer's resistance is set by the dial ratio instead of a fixed value, so it shares the current-limiting job with a resistor and changes the LED's brightness continuously.",
                "Task: drag the dial across its full range and confirm the LED's brightness changes continuously and monotonically.",
            ),
            Self::E6 => (
                "A photoresistor's resistance falls as its ambient-light control rises, so more simulated light means less resistance and a brighter LED.",
                "Task: drag the ambient-light slider across its full range and confirm the LED dims continuously toward off at the darkest setting.",
            ),
            Self::E11 => (
                "A changeover switch always connects its common pin to exactly one of two branches, each with its own resistor and LED, so exactly one LED can light at a time.",
                "Task: toggle the switch and confirm D1 lights alone in one position and D2 lights alone in the other, never both together.",
            ),
            Self::E12 => (
                "Two independent branches share the 5 V supply: one branch wires two LEDs in series behind a single resistor, the other wires a single LED behind its own resistor, so the branches' currents do not affect each other.",
                "Task: run the circuit and confirm all three LEDs light steadily, with D1 and D2 in the series branch matching each other's brightness.",
            ),
            Self::E13 => (
                "A capacitor sits across the same node the button feeds, alongside a resistor-and-LED branch; holding the button charges both the capacitor and the LED branch together, and releasing it lets the capacitor's stored charge keep the LED fading out smoothly instead of snapping dark.",
                "Task: press and hold the button, then release it and confirm the LED fades out gradually rather than turning off instantly.",
            ),
            Self::E14 => (
                "Two momentary buttons wired in parallel each independently complete the path to the resistor and LED, the same as a logical OR gate.",
                "Task: confirm the LED lights when either button is held alone, and also when both are held together.",
            ),
            Self::E15 => (
                "A pull-up resistor keeps the transistor's base high by default, switching it on and lighting the LED; pressing the button pulls the base directly to the negative rail, switching the transistor off, the same as a logical NOT gate.",
                "Task: confirm the LED is lit while the button is unpressed, and goes dark while the button is held.",
            ),
            Self::E16 => (
                "The photoresistor senses ambient light through a voltage divider and controls a transistor that switches the LED. Darkness raises the base voltage, so the LED becomes a night light.",
                "Task: move the ambient-light control from bright to dark and confirm the LED responds inversely to E6: it becomes brighter as the room gets darker.",
            ),
            Self::E17 => (
                "The photoresistor and base resistor control a transistor that powers the buzzer. Bright ambient light raises the base voltage enough to sound the alarm.",
                "Task: sweep the ambient-light control from dark to bright and find the threshold where the buzzer changes from silent to sounding.",
            ),
            Self::E18 => (
                "The potentiometer sets the series resistance feeding the buzzer, so turning the dial changes the buzzer current and eventually silences it.",
                "Task: turn the volume dial upward, find where the buzzer cuts off, and record that dial ratio.",
            ),
            Self::E19 => (
                "Three separate resistor-and-LED branches share the supply rails but not each other's components. Each resistor sets its own branch current independently.",
                "Task: run the circuit and compare each LED current with its own resistor value; changing one branch must not change the other two.",
            ),
            Self::E20 => (
                "A resistor slowly charges a capacitor at the transistor base. Only after the capacitor voltage crosses the base threshold does the transistor switch the LED branch on.",
                "Task: run the circuit and measure the LED turn-on delay in simulated seconds, using the step counter rather than wall-clock time.",
            ),
            Self::E21 => (
                "Ambient light and a potentiometer form two adjustable inputs to the transistor base, so the night-light threshold can be tuned without changing the fixed LED branch.",
                "Task: set several potentiometer positions and find how the ambient-light level where the LED turns on shifts between them.",
            ),
            Self::E22 => (
                "Two buttons in series make one AND path while a third button provides a parallel path, so the LED follows (S1 AND S2) OR S3.",
                "Task: test all eight button combinations and write down which combinations light the LED.",
            ),
            Self::E23 => (
                "The transistor's collector branch powers an LED and a buzzer together. A photoresistor controls its base, so both outputs switch at the same ambient-light threshold.",
                "Task: sweep the ambient-light control through the threshold and confirm the LED and buzzer turn on and off together.",
            ),
            Self::E24 => (
                "A changeover switch selects either a low-resistance bright charging path or a higher-resistance slow path into two LED branches that share one capacitor.",
                "Task: compare both switch positions by watching the brief charge flash and the slower discharge flash in the simulated current traces.",
            ),
            Self::E25 => (
                "Two transistor collector-emitter paths are stacked in series, and each base is controlled by its own button. Both transistors must conduct before the LED has a complete path.",
                "Task: test all four button combinations and compare this transistor AND with E9's two-button series wiring.",
            ),
            Self::E26 => (
                "Two transistors share the LED branch, so either button can drive its own base and switch current through the LED. This is a transistor version of OR, unlike E14's parallel button wiring.",
                "Task: test all four button combinations and confirm either button alone, or both together, lights the LED.",
            ),
            Self::E27 => (
                "One potentiometer feeds both transistor bases, so both LED branches receive the same adjustable base drive and change brightness together.",
                "Task: sweep the shared brightness dial across its full range and compare both LED branches at every setting.",
            ),
            Self::E28 => (
                "A changeover switch selects a resistor branch powered by either a 5 V source or a 9 V source. The two negative rails are tied together so both choices share one return.",
                "Task: toggle the source selector, verify the shared negative node, and compare the LED brightness in both positions.",
            ),
            Self::E29 => (
                "A Darlington pair feeds the second transistor's base from the first transistor's emitter, multiplying their current gains. A 1 MΩ button path can trigger this pair even though E8's single transistor used 10 kΩ.",
                "Task: hold the button and confirm the buzzer sounds; release it and confirm the detector is silent.",
            ),
            Self::E30 => (
                "Two cross-coupled transistor-capacitor paths form a free-running flasher. The documented 0.5 V initial voltage on C2 breaks symmetry so the alternating sequence is deterministic.",
                "Task: run the circuit long enough to observe several alternating on/off cycles of both LEDs.",
            ),
        }
    }
    /// Control buttons for this bench, in display order. Empty for exercises
    /// with no live control (for example, E1, E3, E4, E19, and E20).
    fn controls(self) -> &'static [ControlSpec] {
        const RC: &[ControlSpec] = &[ControlSpec {
            label: "S1: CHANGE PATH",
            component: "S1",
            is_switch: true,
        }];
        const B1_BUTTON: &[ControlSpec] = &[ControlSpec {
            label: "B1: PRESS / RELEASE",
            component: "B1",
            is_switch: false,
        }];
        const S1_BUTTON: &[ControlSpec] = &[ControlSpec {
            label: "S1: PRESS / RELEASE",
            component: "S1",
            is_switch: false,
        }];
        const E9_BUTTONS: &[ControlSpec] = &[
            ControlSpec {
                label: "S1: PRESS / RELEASE",
                component: "S1",
                is_switch: false,
            },
            ControlSpec {
                label: "S2: PRESS / RELEASE",
                component: "S2",
                is_switch: false,
            },
        ];
        const E22_BUTTONS: &[ControlSpec] = &[
            ControlSpec {
                label: "S1: PRESS / RELEASE",
                component: "S1",
                is_switch: false,
            },
            ControlSpec {
                label: "S2: PRESS / RELEASE",
                component: "S2",
                is_switch: false,
            },
            ControlSpec {
                label: "S3: PRESS / RELEASE",
                component: "S3",
                is_switch: false,
            },
        ];
        const E25_BUTTONS: &[ControlSpec] = &[
            ControlSpec {
                label: "S1: PRESS / RELEASE",
                component: "S1",
                is_switch: false,
            },
            ControlSpec {
                label: "S2: PRESS / RELEASE",
                component: "S2",
                is_switch: false,
            },
        ];
        const E11_SWITCH: &[ControlSpec] = &[ControlSpec {
            label: "S1: TOGGLE A / B",
            component: "S1",
            is_switch: true,
        }];
        const E24_SWITCH: &[ControlSpec] = &[ControlSpec {
            label: "SW1: TOGGLE A / B",
            component: "SW1",
            is_switch: true,
        }];
        match self {
            Self::Rc => RC,
            Self::C01S01_02 => &[
                ControlSpec {
                    label: "B1: PRESS / RELEASE",
                    component: "B1",
                    is_switch: false,
                },
                ControlSpec {
                    label: "S1: TOGGLE A / B",
                    component: "S1",
                    is_switch: true,
                },
            ],
            Self::C01S01_06 => B1_BUTTON,
            Self::C01S02_01 => B1_BUTTON,
            Self::E11 => E11_SWITCH,
            Self::E24 => E24_SWITCH,
            Self::Led
            | Self::Transistor
            | Self::E2
            | Self::E7
            | Self::E8
            | Self::E10
            | Self::E13
            | Self::E15 => {
                if matches!(self, Self::Led | Self::Transistor) {
                    B1_BUTTON
                } else {
                    S1_BUTTON
                }
            }
            Self::E9 | Self::E14 => E9_BUTTONS,
            Self::E22 => E22_BUTTONS,
            Self::E25 => E25_BUTTONS,
            Self::E26 => E9_BUTTONS,
            Self::C01S02_06 => E9_BUTTONS,
            Self::C02S03_03 => E9_BUTTONS,
            Self::C02S03_02 => S1_BUTTON,
            Self::C02S03_04 => &[
                ControlSpec {
                    label: "S_SET: PRESS / RELEASE",
                    component: "S_SET",
                    is_switch: false,
                },
                ControlSpec {
                    label: "S_RESET: PRESS / RELEASE",
                    component: "S_RESET",
                    is_switch: false,
                },
            ],
            Self::C02S03_05 => S1_BUTTON,
            Self::C03S04_03 => S1_BUTTON,
            Self::C03S04_05 => &[
                ControlSpec {
                    label: "S_DATA: PRESS / RELEASE",
                    component: "S_DATA",
                    is_switch: false,
                },
                ControlSpec {
                    label: "S_CLOCK: PRESS / RELEASE",
                    component: "S_CLOCK",
                    is_switch: false,
                },
                ControlSpec {
                    label: "S_LATCH: PRESS / RELEASE",
                    component: "S_LATCH",
                    is_switch: false,
                },
            ],
            Self::C03S04_04 => S1_BUTTON,
            Self::C03S04_01 => S1_BUTTON,
            Self::C03S04_02 => S1_BUTTON,
            Self::E28 => E24_SWITCH,
            Self::E29 => S1_BUTTON,
            Self::E1
            | Self::E3
            | Self::E4
            | Self::E5
            | Self::E6
            | Self::E12
            | Self::E16
            | Self::E17
            | Self::E18
            | Self::E19
            | Self::E20 => &[],
            Self::E21 | Self::E23 | Self::E27 | Self::E30 => &[],
            Self::C01S01_01
            | Self::C01S01_03
            | Self::C01S01_04
            | Self::C01S01_05
            | Self::C01S02_02 => &[],
            Self::C02S03_01 | Self::C02S03_06 | Self::C02S03_07 | Self::C03S04_09 => &[],
            Self::C03S05_01 => &[
                ControlSpec {
                    label: "A0: LOW / HIGH",
                    component: "SA0",
                    is_switch: true,
                },
                ControlSpec {
                    label: "A1: LOW / HIGH",
                    component: "SA1",
                    is_switch: true,
                },
                ControlSpec {
                    label: "A2: LOW / HIGH",
                    component: "SA2",
                    is_switch: true,
                },
                ControlSpec {
                    label: "A3: LOW / HIGH",
                    component: "SA3",
                    is_switch: true,
                },
                ControlSpec {
                    label: "B0: LOW / HIGH",
                    component: "SB0",
                    is_switch: true,
                },
                ControlSpec {
                    label: "B1: LOW / HIGH",
                    component: "SB1",
                    is_switch: true,
                },
                ControlSpec {
                    label: "B2: LOW / HIGH",
                    component: "SB2",
                    is_switch: true,
                },
                ControlSpec {
                    label: "B3: LOW / HIGH",
                    component: "SB3",
                    is_switch: true,
                },
            ],
            Self::C03S05_02 => &[
                ControlSpec {
                    label: "A0: LOW / HIGH",
                    component: "SA0",
                    is_switch: true,
                },
                ControlSpec {
                    label: "A1: LOW / HIGH",
                    component: "SA1",
                    is_switch: true,
                },
                ControlSpec {
                    label: "A2: LOW / HIGH",
                    component: "SA2",
                    is_switch: true,
                },
                ControlSpec {
                    label: "A3: LOW / HIGH",
                    component: "SA3",
                    is_switch: true,
                },
                ControlSpec {
                    label: "B0: LOW / HIGH",
                    component: "SB0",
                    is_switch: true,
                },
                ControlSpec {
                    label: "B1: LOW / HIGH",
                    component: "SB1",
                    is_switch: true,
                },
                ControlSpec {
                    label: "B2: LOW / HIGH",
                    component: "SB2",
                    is_switch: true,
                },
                ControlSpec {
                    label: "B3: LOW / HIGH",
                    component: "SB3",
                    is_switch: true,
                },
            ],
            Self::C03S05_03 => S1_BUTTON,
        }
    }
    /// The continuous dial/slider control for variable-resistor exercises.
    /// `None` for every other bench.
    fn dials(self) -> &'static [DialSpec] {
        const E21_DIALS: &[DialSpec] = &[
            DialSpec {
                label: "R2: AMBIENT LIGHT - drag left/right",
                component: "R2",
            },
            DialSpec {
                label: "RV1: THRESHOLD - drag left/right",
                component: "RV1",
            },
        ];
        match self {
            Self::C01S01_04 => &[DialSpec {
                label: "RV1: DIMMER - drag left/right",
                component: "RV1",
            }],
            Self::C01S02_02 => &[
                DialSpec {
                    label: "R3: AMBIENT LIGHT - drag left/right",
                    component: "R3",
                },
                DialSpec {
                    label: "RV1: THRESHOLD - drag left/right",
                    component: "RV1",
                },
            ],
            Self::E5 => &[DialSpec {
                label: "RV1: BRIGHTNESS DIAL - drag left/right",
                component: "RV1",
            }],
            Self::E6 => &[DialSpec {
                label: "RV1: AMBIENT LIGHT - drag left/right",
                component: "RV1",
            }],
            Self::E16 => &[DialSpec {
                label: "R3: AMBIENT LIGHT - drag left/right",
                component: "R3",
            }],
            Self::E17 => &[DialSpec {
                label: "R2: AMBIENT LIGHT - drag left/right",
                component: "R2",
            }],
            Self::E18 => &[DialSpec {
                label: "RV1: VOLUME DIAL - drag left/right",
                component: "RV1",
            }],
            Self::E21 => E21_DIALS,
            Self::E23 => &[DialSpec {
                label: "R3: AMBIENT LIGHT - drag left/right",
                component: "R3",
            }],
            Self::C02S03_01 => &[DialSpec {
                label: "RV1: FREQUENCY - drag left/right",
                component: "RV1",
            }],
            Self::C02S03_06 => &[
                DialSpec {
                    label: "TH1: TEMPERATURE - drag left/right",
                    component: "TH1",
                },
                DialSpec {
                    label: "RV1: THRESHOLD - drag left/right",
                    component: "RV1",
                },
            ],
            Self::C02S03_07 => &[DialSpec {
                label: "TH1: LIGHT - drag left/right",
                component: "TH1",
            }],
            Self::C03S04_09 => &[
                DialSpec {
                    label: "RV1: MODULATOR RATE - drag left/right",
                    component: "RV1",
                },
                DialSpec {
                    label: "RV2: OUTPUT TONE - drag left/right",
                    component: "RV2",
                },
            ],
            Self::E27 => &[DialSpec {
                label: "RV1: SHARED BRIGHTNESS - drag left/right",
                component: "RV1",
            }],
            _ => &[],
        }
    }
}

struct Bench {
    circuit: Circuit,
    project: Project,
    initial: Project,
    simulation: SimulationState,
}

impl Bench {
    fn new(circuit: Circuit) -> Self {
        let project: Project = serde_json::from_str(circuit.json()).expect("embedded project JSON");
        compile_topology(&project).expect("embedded project topology");
        Self {
            circuit,
            initial: project.clone(),
            simulation: SimulationState::new(&project),
            project,
        }
    }
    fn act(&mut self, action: Action) {
        apply_actions(
            &mut self.project,
            &self.initial,
            &mut self.simulation,
            &[action],
        );
    }
    /// Toggles the `index`-th control button for this bench (see
    /// `Circuit::controls`); out-of-range indices are ignored.
    fn toggle(&mut self, index: usize) {
        let Some(spec) = self.circuit.controls().get(index) else {
            return;
        };
        let id = ComponentId(spec.component.into());
        let state = if spec.is_switch {
            if self.simulation.controls[&id] == ControlState::SwitchNormallyClosed {
                ControlState::SwitchNormallyOpen
            } else {
                ControlState::SwitchNormallyClosed
            }
        } else if self.simulation.controls[&id] == ControlState::ButtonReleased {
            ControlState::ButtonPressed
        } else {
            ControlState::ButtonReleased
        };
        self.act(Action::SetControl {
            component: id,
            state,
        });
    }
}

#[derive(Resource, Default)]
struct Session {
    bench: Option<Bench>,
}

/// Vertical scroll offset for the menu list, in world units. Positive values
/// reveal later entries by shifting the list up. Presentation-only; never
/// read by `bredboard-core` or routed through an `Action`.
#[derive(Resource, Default)]
struct MenuScroll {
    offset: f32,
}

#[derive(Resource, Default)]
struct MenuFilter {
    query: String,
}

#[derive(Resource)]
struct MenuCollapsed {
    sections: [bool; 4],
}

impl Default for MenuCollapsed {
    fn default() -> Self {
        // Keep the short MVP section open; the longer practice groups start
        // collapsed so the menu is useful without scrolling immediately.
        Self {
            sections: [false, true, true, true],
        }
    }
}

impl MenuCollapsed {
    #[cfg(test)]
    fn all_expanded() -> Self {
        Self {
            sections: [false; 4],
        }
    }

    fn index(section: exercise_catalog::Section) -> usize {
        match section {
            exercise_catalog::Section::Mvp => 0,
            exercise_catalog::Section::Foundations => 1,
            exercise_catalog::Section::SwitchingAndTiming => 2,
            exercise_catalog::Section::Advanced => 3,
        }
    }

    fn is_collapsed(&self, section: exercise_catalog::Section) -> bool {
        self.sections[Self::index(section)]
    }

    fn toggle(&mut self, section: exercise_catalog::Section) {
        let index = Self::index(section);
        self.sections[index] = !self.sections[index];
    }
}

/// Top/bottom of the menu's scrollable viewport in world space, and the
/// vertical spacing between entries. The header (title/subtitle) sits above
/// `MENU_TOP`; the footer hint sits below `MENU_BOTTOM`.
const MENU_TOP: f32 = 155.0;
const MENU_BOTTOM: f32 = -220.0;
const MENU_ENTRY_HEIGHT: f32 = 52.0;
const MENU_ENTRY_SIZE: Vec2 = Vec2::new(520.0, 42.0);

/// World-space Y of a menu entry at rest (scroll offset zero). Index 0 is the
/// first entry; matches the original fixed 3-entry layout exactly.
fn menu_entry_base_y(index: usize) -> f32 {
    125.0 - index as f32 * MENU_ENTRY_HEIGHT
}

/// Largest scroll offset that still keeps the last entry's bottom edge at or
/// above `MENU_BOTTOM`; zero when every entry already fits in the viewport.
fn menu_max_scroll(entry_count: usize) -> f32 {
    if entry_count == 0 {
        return 0.0;
    }
    let content_top = menu_entry_base_y(0) + MENU_ENTRY_SIZE.y * 0.5;
    let content_bottom = menu_entry_base_y(entry_count - 1) - MENU_ENTRY_SIZE.y * 0.5;
    let content_height = content_top - content_bottom;
    (content_height - (MENU_TOP - MENU_BOTTOM)).max(0.0)
}

/// Returns the catalog indices that occupy rows right now. Collapsed groups
/// still reveal a matching exercise while the search field is active.
fn menu_layout_indices(filter: &MenuFilter, collapsed: &MenuCollapsed) -> Vec<usize> {
    let query = filter.query.to_ascii_lowercase();
    let mut section = exercise_catalog::Section::Mvp;
    let mut indices = Vec::new();
    for (index, item) in exercise_catalog::items().into_iter().enumerate() {
        match item {
            exercise_catalog::Item::Section(value) => {
                section = value;
                indices.push(index);
            }
            exercise_catalog::Item::Circuit(circuit) => {
                let matches =
                    query.is_empty() || circuit.label().to_ascii_lowercase().contains(&query);
                if matches
                    && (query.is_empty() && !collapsed.is_collapsed(section) || !query.is_empty())
                {
                    indices.push(index);
                }
            }
        }
    }
    indices
}

#[derive(Component)]
struct SceneEntity;
/// Marks an entity as one of the catalog rows, keyed by its index in
/// `exercise_catalog::items()`. A single index is shared by an entry's button
/// rect and its label so both move and hide together.
#[derive(Component, Clone, Copy)]
struct MenuEntry(usize);
#[derive(Component)]
struct SearchLabel;
#[derive(Component, Clone, Copy)]
struct MenuSectionLabel(exercise_catalog::Section);
#[derive(Component, Clone, Copy)]
struct ClickTarget(Control, Vec2);
/// A draggable track for a continuous dial/slider control.
#[derive(Component)]
struct DialTrack {
    component: ComponentId,
    center: Vec2,
    size: Vec2,
}
/// The handle sprite that shows a dial's current ratio; repositioned every
/// frame from `SimulationState.control_ratios`.
#[derive(Component)]
struct DialHandle {
    component: ComponentId,
    center: Vec2,
    size: Vec2,
}
const DIAL_TRACK_CENTER: Vec2 = Vec2::new(205.0, -245.0);
const DIAL_TRACK_SIZE: Vec2 = Vec2::new(400.0, 53.0);
const DIAL_HANDLE_SIZE: Vec2 = Vec2::new(14.0, 53.0);

fn dial_layout(index: usize, count: usize) -> (Vec2, Vec2) {
    if count <= 1 {
        (DIAL_TRACK_CENTER, DIAL_TRACK_SIZE)
    } else {
        (
            Vec2::new(105.0 + index as f32 * 200.0, DIAL_TRACK_CENTER.y),
            Vec2::new(190.0, DIAL_TRACK_SIZE.y),
        )
    }
}
#[derive(Clone, Copy)]
enum Control {
    Select(Circuit),
    ToggleSection(exercise_catalog::Section),
    Back,
    RunPause,
    Reset,
    Toggle(usize),
}
#[derive(Component)]
enum Readout {
    Status,
    Value,
    Control,
    Hover,
}
/// A component drawn with pixel art; `states` holds one image per visual state.
#[derive(Component)]
struct PartVisual {
    id: ComponentId,
    kind: ComponentKind,
    states: Vec<Handle<Image>>,
    shown: usize,
    /// Last-applied audio state for a buzzer/speaker; drives `AudioPlayer`
    /// insertion and removal in `update_view` so the tone starts and stops
    /// exactly when the sprite's sounding state changes, without re-checking
    /// current every frame for parts that never sound.
    sounding: bool,
}

/// Sine tones played while a buzzer or speaker is calculated as sounding.
/// Built once at startup from `bevy_audio`'s `Pitch` asset (a synthesized
/// sine wave; no audio file is shipped). Present only when `AudioPlugin` has
/// registered `Assets<Pitch>` (the real app, not every test harness build),
/// so `update_view` treats it as optional.
#[derive(Resource)]
struct SoundTones {
    /// Piezo buzzer: a high, thin beep.
    buzzer: Handle<Pitch>,
    /// Dynamic speaker: a lower, fuller tone.
    speaker: Handle<Pitch>,
}

fn setup_audio(mut commands: Commands, mut pitches: ResMut<Assets<Pitch>>) {
    let long = std::time::Duration::from_secs(3600);
    commands.insert_resource(SoundTones {
        buzzer: pitches.add(Pitch::new(2800.0, long)),
        speaker: pitches.add(Pitch::new(440.0, long)),
    });
}

fn main() {
    let mut app = App::new();
    app.insert_resource(Time::<Virtual>::from_max_delta(std::time::Duration::MAX))
        .insert_resource(Time::<Fixed>::from_hz(1_000.0))
        .insert_resource(Session::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: text::WINDOW_TITLE.into(),
                resolution: (1200, 760).into(),
                resize_constraints: WindowResizeConstraints {
                    min_width: 1200.0,
                    min_height: 760.0,
                    ..default()
                },
                #[cfg(target_arch = "wasm32")]
                canvas: Some("#bredboard".into()),
                #[cfg(target_arch = "wasm32")]
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(MenuScroll::default())
        .insert_resource(MenuFilter::default())
        .insert_resource(MenuCollapsed::default())
        .add_systems(Startup, (setup, setup_audio))
        .add_systems(FixedUpdate, fixed_step)
        .add_systems(
            Update,
            (
                handle_scroll,
                handle_menu_search,
                scroll_menu,
                handle_mouse,
                update_section_labels,
                handle_keyboard,
                update_search_label,
                handle_dial,
                update_dial_handle,
                update_view,
            )
                .chain(),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: 1200.0,
                min_height: 760.0,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
    spawn_menu(&mut commands);
}

fn rect(commands: &mut Commands, point: Vec2, size: Vec2, color: Color, z: f32) -> Entity {
    commands
        .spawn((
            Sprite::from_color(color, size),
            Transform::from_xyz(point.x, point.y, z),
            SceneEntity,
        ))
        .id()
}

fn label(
    commands: &mut Commands,
    value: impl Into<String>,
    point: Vec2,
    size: f32,
    color: Color,
) -> Entity {
    commands
        .spawn((
            Text2d::new(value),
            TextFont::from_font_size(size),
            TextColor(color),
            Transform::from_xyz(point.x, point.y, 2.0),
            SceneEntity,
        ))
        .id()
}

fn button(commands: &mut Commands, value: &str, point: Vec2, size: Vec2, control: Control) {
    let entity = rect(commands, point, size, Color::srgb(0.13, 0.30, 0.35), 1.0);
    commands.entity(entity).insert(ClickTarget(control, size));
    label(commands, value, point, 21.0, Color::srgb(0.94, 0.97, 0.96));
}

/// A fixed-size menu row tagged with `MenuEntry(index)` on both its rect and
/// label so `scroll_menu` can move and hide them together.
fn menu_entry_button(commands: &mut Commands, value: &str, index: usize, control: Control) {
    let point = Vec2::new(0.0, menu_entry_base_y(index));
    let rect_entity = rect(
        commands,
        point,
        MENU_ENTRY_SIZE,
        Color::srgb(0.13, 0.30, 0.35),
        1.0,
    );
    commands
        .entity(rect_entity)
        .insert((ClickTarget(control, MENU_ENTRY_SIZE), MenuEntry(index)));
    let label_entity = label(commands, value, point, 21.0, Color::srgb(0.94, 0.97, 0.96));
    commands.entity(label_entity).insert(MenuEntry(index));
}

fn menu_section_label(commands: &mut Commands, section: exercise_catalog::Section, index: usize) {
    let point = Vec2::new(0.0, menu_entry_base_y(index));
    let rect_entity = rect(
        commands,
        point,
        MENU_ENTRY_SIZE,
        Color::srgb(0.10, 0.24, 0.27),
        1.0,
    );
    commands.entity(rect_entity).insert((
        ClickTarget(Control::ToggleSection(section), MENU_ENTRY_SIZE),
        MenuEntry(index),
    ));
    let entity = label(
        commands,
        section.label(),
        point,
        18.0,
        Color::srgb(0.42, 0.91, 0.76),
    );
    commands
        .entity(entity)
        .insert((MenuEntry(index), MenuSectionLabel(section)));
}

fn spawn_menu(commands: &mut Commands) {
    rect(
        commands,
        Vec2::ZERO,
        Vec2::new(1200.0, 760.0),
        Color::srgb(0.09, 0.13, 0.15),
        -1.0,
    );
    label(
        commands,
        "BREDBOARD",
        Vec2::new(0.0, 285.0),
        44.0,
        Color::srgb(0.42, 0.91, 0.76),
    );
    label(
        commands,
        "Choose a real breadboard circuit",
        Vec2::new(0.0, 220.0),
        23.0,
        Color::srgb(0.77, 0.84, 0.83),
    );
    let search = label(
        commands,
        "FILTER: type an exercise name or number",
        Vec2::new(0.0, 180.0),
        17.0,
        Color::srgb(0.58, 0.68, 0.68),
    );
    commands.entity(search).insert(SearchLabel);
    for (index, item) in exercise_catalog::items().into_iter().enumerate() {
        match item {
            exercise_catalog::Item::Section(section) => {
                menu_section_label(commands, section, index)
            }
            exercise_catalog::Item::Circuit(circuit) => {
                menu_entry_button(commands, circuit.label(), index, Control::Select(circuit))
            }
        }
    }
    label(
        commands,
        "5 V examples - select a circuit to inspect its exact holes and wires",
        Vec2::new(0.0, -285.0),
        17.0,
        Color::srgb(0.58, 0.68, 0.68),
    );
}

fn line(commands: &mut Commands, from: Vec2, to: Vec2, color: Color, width: f32, z: f32) {
    let delta = to - from;
    let entity = rect(
        commands,
        (from + to) * 0.5,
        Vec2::new(delta.length().max(1.0), width),
        color,
        z,
    );
    commands.entity(entity).insert(Transform {
        translation: ((from + to) * 0.5).extend(z),
        rotation: Quat::from_rotation_z(delta.y.atan2(delta.x)),
        ..default()
    });
}

/// Draws a wire the same way everywhere: a dark outline, a colored core, and
/// rounded end caps where it plugs into a hole.
fn draw_wire(commands: &mut Commands, from: Vec2, to: Vec2, color: Color, shade: Color) {
    let outline = srgb(sprites::palette::OUTLINE);
    line(commands, from, to, outline, 8.0, 0.7);
    line(commands, from, to, color, 4.0, 0.75);
    for point in [from, to] {
        rect(commands, point, Vec2::splat(8.0), outline, 0.9);
        rect(commands, point, Vec2::splat(4.0), shade, 0.95);
    }
}

/// Square 16-unit hole pitch (8 art pixels at 2 units each) keeps sprites pixel-exact.
const PITCH: f32 = 16.0;
const BOARD_CENTER_X: f32 = -385.0;
const TOP_ROW_Y: f32 = 255.0;

/// Horizontal position of a board column (A–J) or rail (TP+/TP-/BP+/BP-).
/// Strips and rails are separated by two-pitch gaps, as on a real breadboard.
fn column_x(column: &str) -> Option<f32> {
    let pitches = match column {
        "TP+" => -8.0,
        "TP-" => -7.0,
        "BP+" => 7.0,
        "BP-" => 8.0,
        _ => {
            let mut chars = column.chars();
            let letter = chars.next()?;
            if chars.next().is_some() {
                return None;
            }
            match letter {
                'A'..='E' => -5.0 + (letter as u8 - b'A') as f32,
                'F'..='J' => 1.0 + (letter as u8 - b'F') as f32,
                _ => return None,
            }
        }
    };
    Some(BOARD_CENTER_X + pitches * PITCH)
}

fn row_y(row: u32) -> f32 {
    TOP_ROW_Y - (row - 1) as f32 * PITCH
}

fn hole_position(id: &str) -> Option<Vec2> {
    let (column, row) = match id.split_once(':') {
        Some((rail, row)) => (rail, row),
        None => id.split_at_checked(1)?,
    };
    let row: u32 = row.parse().ok()?;
    if !(1..=30).contains(&row) || id.contains(':') != (column.len() == 3) {
        return None;
    }
    Some(Vec2::new(column_x(column)?, row_y(row)))
}

fn srgb(color: sprites::palette::Rgb) -> Color {
    Color::srgb_u8(color[0], color[1], color[2])
}

fn spawn_board(commands: &mut Commands, images: &mut Assets<Image>, bench: &Bench) {
    use sprites::palette;
    let width = 17.0 * PITCH + 34.0;
    rect(
        commands,
        Vec2::new(BOARD_CENTER_X, 20.0),
        Vec2::new(width + 18.0, 525.0),
        Color::srgb(0.16, 0.31, 0.28),
        0.0,
    );
    rect(
        commands,
        Vec2::new(BOARD_CENTER_X, 20.0),
        Vec2::new(width, 509.0),
        srgb(palette::BOARD),
        0.1,
    );
    rect(
        commands,
        Vec2::new(BOARD_CENTER_X, 20.0),
        Vec2::new(12.0, 480.0),
        srgb(palette::BOARD_GROOVE),
        0.2,
    );
    for rail in ["TP+", "TP-", "BP+", "BP-"] {
        let x = column_x(rail).unwrap();
        let color = srgb(if rail.ends_with('+') {
            palette::RAIL_RED
        } else {
            palette::RAIL_BLUE
        });
        // Marking line on the far side of each rail from its partner rail.
        let side = if rail.ends_with('+') { -1.0 } else { 1.0 };
        rect(
            commands,
            Vec2::new(x + side * 7.0, 20.0),
            Vec2::new(2.0, 480.0),
            color,
            0.22,
        );
        label(
            commands,
            if rail.ends_with('+') { "+" } else { "-" },
            Vec2::new(x, 278.0),
            19.0,
            color,
        );
    }
    for col in 'A'..='J' {
        label(
            commands,
            col.to_string(),
            Vec2::new(column_x(&col.to_string()).unwrap(), 278.0),
            12.0,
            Color::srgb(0.25, 0.26, 0.22),
        );
    }
    for row in 1..=30 {
        if row == 1 || row % 5 == 0 {
            label(
                commands,
                row.to_string(),
                Vec2::new(BOARD_CENTER_X - 1.0, row_y(row)),
                11.0,
                Color::srgb(0.25, 0.26, 0.22),
            );
        }
        let holes = ('A'..='J')
            .map(|col| format!("{col}{row}"))
            .chain(["TP+", "TP-", "BP+", "BP-"].map(|rail| format!("{rail}:{row}")));
        for id in holes {
            rect(
                commands,
                hole_position(&id).unwrap(),
                Vec2::splat(sprites::PIXEL * 2.0),
                srgb(palette::HOLE),
                0.4,
            );
        }
    }
    for wire in &bench.project.wires {
        let a = hole_position(&wire.from.0).unwrap();
        let b = hole_position(&wire.to.0).unwrap();
        draw_wire(
            commands,
            a,
            b,
            srgb(palette::WIRE),
            srgb(palette::WIRE_SHADE),
        );
    }
    for (index, component) in bench.project.components.iter().enumerate() {
        if component.kind == ComponentKind::DcVoltageSource {
            spawn_source(commands, images, component);
            continue;
        }
        let art = sprites::art_for(component.kind).expect("every component kind has sprite art");
        spawn_part(commands, images, component, art, index);
    }
}

/// Spawns a pixel-art part with one pre-rendered image per visual state.
fn spawn_part(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    component: &Component,
    art: &dyn sprites::PartArt,
    index: usize,
) {
    let pins: Vec<Vec2> = component
        .pins
        .values()
        .map(|hole| hole_position(&hole.0).unwrap())
        .collect();
    let placed: Vec<_> = (0..art.state_count())
        .map(|state| sprites::place(art, component, &pins, state))
        .collect();
    let size = Vec2::new(
        placed[0].canvas.width() as f32,
        placed[0].canvas.height() as f32,
    ) * sprites::PIXEL;
    let center = pins[0] + placed[0].center_offset;
    let states: Vec<_> = placed
        .iter()
        .map(|p| images.add(p.canvas.to_image()))
        .collect();
    // LEDs sit on top so their halo is never hidden; the index keeps order stable.
    let layer = if component.kind == ComponentKind::Led {
        1.4
    } else {
        1.2
    };
    commands.spawn((
        Sprite {
            image: states[0].clone(),
            custom_size: Some(size),
            ..default()
        },
        Transform::from_translation(center.extend(layer + index as f32 * 0.001)),
        SceneEntity,
        PartVisual {
            id: component.id.clone(),
            kind: component.kind,
            states,
            shown: 0,
            sounding: false,
        },
    ));
}

/// Off-board 5 V supply: fixed position above the board, colored supply
/// wires to each pin's hole, and a pixel-art body sprite (see `sprites::source`).
fn spawn_source(commands: &mut Commands, images: &mut Assets<Image>, component: &Component) {
    let center = Vec2::new(
        (column_x("TP+").unwrap() + column_x("TP-").unwrap()) / 2.0,
        322.0,
    );
    for (pin, hole) in &component.pins {
        let start = center + Vec2::new(if pin.0 == "positive" { -10.0 } else { 10.0 }, -14.0);
        let (color, shade) = if pin.0 == "positive" {
            (
                srgb(sprites::palette::RAIL_RED),
                srgb(sprites::palette::WIRE_RED_SHADE),
            )
        } else {
            (
                srgb(sprites::palette::RAIL_BLUE),
                srgb(sprites::palette::WIRE_BLUE_SHADE),
            )
        };
        draw_wire(
            commands,
            start,
            hole_position(&hole.0).unwrap(),
            color,
            shade,
        );
    }
    let art = sprites::art_for(component.kind).expect("dc_voltage_source has sprite art");
    let body = art.body(component, 0);
    let size = Vec2::new(body.width() as f32, body.height() as f32) * sprites::PIXEL;
    commands.spawn((
        Sprite {
            image: images.add(body.to_image()),
            custom_size: Some(size),
            ..default()
        },
        Transform::from_translation(center.extend(1.1)),
        SceneEntity,
    ));
}

fn component_summary(component: &Component) -> String {
    let id = &component.id.0;
    match component.kind {
        ComponentKind::DcVoltageSource => format!("{id}  5 V supply  TP+:1 / TP-:1"),
        ComponentKind::Resistor => format!(
            "{id}  {:.0} ohm  {} / {}",
            component.parameters["resistance"],
            component.pins[&bredboard_core::PinId("a".into())].0,
            component.pins[&bredboard_core::PinId("b".into())].0
        ),
        ComponentKind::Led => format!(
            "{id}  LED  A {} / K {}",
            component.pins[&bredboard_core::PinId("anode".into())].0,
            component.pins[&bredboard_core::PinId("cathode".into())].0
        ),
        ComponentKind::Diode => format!(
            "{id}  diode  A {} / K {}",
            component.pins[&bredboard_core::PinId("anode".into())].0,
            component.pins[&bredboard_core::PinId("cathode".into())].0
        ),
        ComponentKind::Capacitor => format!(
            "{id}  {:.0} uF  + {} / - {}",
            component.parameters["capacitance"] * 1_000_000.0,
            component.pins[&bredboard_core::PinId("positive".into())].0,
            component.pins[&bredboard_core::PinId("negative".into())].0
        ),
        ComponentKind::MomentaryButton => format!(
            "{id}  button  {} / {}",
            component.pins[&bredboard_core::PinId("a".into())].0,
            component.pins[&bredboard_core::PinId("b".into())].0
        ),
        ComponentKind::ChangeoverSwitch => format!(
            "{id}  switch  NC {} C {} NO {}",
            component.pins[&bredboard_core::PinId("normally_closed".into())].0,
            component.pins[&bredboard_core::PinId("common".into())].0,
            component.pins[&bredboard_core::PinId("normally_open".into())].0
        ),
        ComponentKind::NpnTransistor => format!(
            "{id}  NPN  B {} C {} E {}",
            component.pins[&bredboard_core::PinId("base".into())].0,
            component.pins[&bredboard_core::PinId("collector".into())].0,
            component.pins[&bredboard_core::PinId("emitter".into())].0
        ),
        ComponentKind::PnpTransistor => format!(
            "{id}  PNP  B {} C {} E {}",
            component.pins[&bredboard_core::PinId("base".into())].0,
            component.pins[&bredboard_core::PinId("collector".into())].0,
            component.pins[&bredboard_core::PinId("emitter".into())].0
        ),
        ComponentKind::Potentiometer => format!(
            "{id}  potentiometer {:.0}-{:.0} ohm  {} / {}",
            component.parameters["min_resistance"],
            component.parameters["max_resistance"],
            component.pins[&bredboard_core::PinId("a".into())].0,
            component.pins[&bredboard_core::PinId("b".into())].0
        ),
        ComponentKind::Photoresistor => format!(
            "{id}  photoresistor {:.0}-{:.0} ohm  {} / {}",
            component.parameters["min_resistance"],
            component.parameters["max_resistance"],
            component.pins[&bredboard_core::PinId("a".into())].0,
            component.pins[&bredboard_core::PinId("b".into())].0
        ),
        ComponentKind::Thermistor => format!(
            "{id}  NTC thermistor {:.0}-{:.0} ohm  {} / {}",
            component.parameters["min_resistance"],
            component.parameters["max_resistance"],
            component.pins[&bredboard_core::PinId("a".into())].0,
            component.pins[&bredboard_core::PinId("b".into())].0
        ),
        ComponentKind::Buzzer => format!(
            "{id}  buzzer {:.0} ohm  + {} / - {}",
            component.parameters["resistance"],
            component.pins[&bredboard_core::PinId("positive".into())].0,
            component.pins[&bredboard_core::PinId("negative".into())].0
        ),
        ComponentKind::Speaker => format!(
            "{id}  speaker {:.0} ohm  + {} / - {}",
            component.parameters["resistance"],
            component.pins[&bredboard_core::PinId("positive".into())].0,
            component.pins[&bredboard_core::PinId("negative".into())].0
        ),
        ComponentKind::LogicGate => format!(
            "{id}  logic gate {}  OUT {}",
            component.parameters["operation"],
            component.pins[&bredboard_core::PinId("output".into())].0
        ),
        ComponentKind::SchmittInverter => format!(
            "{id}  Schmitt inverter  OUT {}",
            component.pins[&bredboard_core::PinId("output".into())].0
        ),
        ComponentKind::Comparator => format!(
            "{id}  comparator  OUT {}",
            component.pins[&bredboard_core::PinId("output".into())].0
        ),
        ComponentKind::Timer555 => format!(
            "{id}  NE555  OUT {}",
            component.pins[&bredboard_core::PinId("output".into())].0
        ),
        ComponentKind::DFlipFlop => format!(
            "{id}  D flip-flop  Q {}",
            component.pins[&bredboard_core::PinId("q".into())].0
        ),
        ComponentKind::DigitalCounter => format!("{id}  digital counter  Q0–Q9/CARRY"),
        ComponentKind::ShiftRegister => format!("{id}  8-bit shift register  Q0–Q7"),
        ComponentKind::SevenSegmentDisplay => {
            format!("{id}  seven-segment display  A–G/common cathode")
        }
        ComponentKind::FourBitAdder => format!("{id}  four-bit adder  A/B → SUM/CARRY"),
    }
}

fn spawn_bench(commands: &mut Commands, images: &mut Assets<Image>, bench: &Bench) {
    rect(
        commands,
        Vec2::ZERO,
        Vec2::new(1200.0, 760.0),
        Color::srgb(0.10, 0.14, 0.16),
        -1.0,
    );
    spawn_board(commands, images, bench);
    label(
        commands,
        bench.circuit.label(),
        Vec2::new(185.0, 308.0),
        28.0,
        Color::srgb(0.84, 0.94, 0.93),
    );
    let (explanation, task) = bench.circuit.explanation_and_task();
    let secondary = if explanation.is_empty() {
        "Hover a hole to inspect a wire or pin".to_string()
    } else {
        format!("{explanation}\n{task}")
    };
    label(
        commands,
        secondary,
        Vec2::new(185.0, 264.0),
        14.0,
        Color::srgb(0.58, 0.76, 0.76),
    );
    label(
        commands,
        "BUILD LIST",
        Vec2::new(185.0, 232.0),
        17.0,
        Color::srgb(0.42, 0.90, 0.76),
    );
    for (index, component) in bench.project.components.iter().enumerate() {
        label(
            commands,
            component_summary(component),
            Vec2::new(185.0, 205.0 - index as f32 * 24.0),
            14.0,
            Color::srgb(0.76, 0.85, 0.82),
        );
    }
    let value = label(
        commands,
        "",
        Vec2::new(185.0, 28.0),
        25.0,
        Color::srgb(0.84, 0.95, 0.91),
    );
    commands.entity(value).insert(Readout::Value);
    let status = label(
        commands,
        "",
        Vec2::new(185.0, -18.0),
        17.0,
        Color::srgb(0.72, 0.82, 0.83),
    );
    commands.entity(status).insert(Readout::Status);
    let hover = label(
        commands,
        "",
        Vec2::new(185.0, -70.0),
        16.0,
        Color::srgb(0.65, 0.81, 0.85),
    );
    commands.entity(hover).insert(Readout::Hover);
    let control = label(
        commands,
        "",
        Vec2::new(185.0, -110.0),
        17.0,
        Color::srgb(0.75, 0.88, 0.89),
    );
    commands.entity(control).insert(Readout::Control);
    button(
        commands,
        "RUN / PAUSE",
        Vec2::new(35.0, -170.0),
        Vec2::new(180.0, 53.0),
        Control::RunPause,
    );
    button(
        commands,
        "RESET",
        Vec2::new(215.0, -170.0),
        Vec2::new(150.0, 53.0),
        Control::Reset,
    );
    // Control buttons share one row so the window layout never depends on
    // how many controls a bench has (at most 2, for E9's two buttons); RESET
    // and CIRCUITS always stay at their original fixed positions.
    let controls = bench.circuit.controls();
    if controls.len() == 1 {
        button(
            commands,
            controls[0].label,
            Vec2::new(205.0, -245.0),
            Vec2::new(400.0, 53.0),
            Control::Toggle(0),
        );
    } else {
        let width = 400.0 / controls.len().max(1) as f32 - 10.0;
        for (index, spec) in controls.iter().enumerate() {
            let x = 5.0 + (width + 10.0) * (index as f32 + 0.5);
            button(
                commands,
                spec.label,
                Vec2::new(x, -245.0),
                Vec2::new(width, 53.0),
                Control::Toggle(index),
            );
        }
    }
    let dials = bench.circuit.dials();
    for (index, spec) in dials.iter().enumerate() {
        let component = ComponentId(spec.component.into());
        let (center, size) = dial_layout(index, dials.len());
        let track = rect(commands, center, size, Color::srgb(0.10, 0.22, 0.25), 1.0);
        commands.entity(track).insert(DialTrack {
            component: component.clone(),
            center,
            size,
        });
        label(
            commands,
            spec.label,
            center + Vec2::new(0.0, 20.0),
            13.0,
            Color::srgb(0.75, 0.88, 0.89),
        );
        let ratio = bench
            .simulation
            .control_ratios
            .get(&component)
            .copied()
            .unwrap_or(0.5);
        let handle_x = center.x - size.x / 2.0 + ratio as f32 * size.x;
        let handle = rect(
            commands,
            Vec2::new(handle_x, center.y),
            DIAL_HANDLE_SIZE,
            Color::srgb(0.42, 0.90, 0.76),
            1.1,
        );
        commands.entity(handle).insert(DialHandle {
            component,
            center,
            size,
        });
    }
    button(
        commands,
        "CIRCUITS",
        Vec2::new(185.0, -328.0),
        Vec2::new(260.0, 45.0),
        Control::Back,
    );
}

fn clear_scene(commands: &mut Commands, entities: &Query<Entity, With<SceneEntity>>) {
    for entity in entities {
        commands.entity(entity).despawn();
    }
}

fn cursor_world(window: &Window) -> Option<Vec2> {
    let cursor = window.cursor_position()?;
    let width = window.width().max(1.0);
    let height = window.height().max(1.0);
    let world_width = 1200.0_f32.max(760.0 * width / height);
    let world_height = 760.0_f32.max(1200.0 * height / width);
    Some(Vec2::new(
        (cursor.x / width - 0.5) * world_width,
        (0.5 - cursor.y / height) * world_height,
    ))
}

/// Mouse-wheel-driven vertical scroll of the menu list, clamped so it cannot
/// scroll past the first or last entry. Presentation input only.
fn handle_scroll(
    mut wheel: MessageReader<MouseWheel>,
    mut scroll: ResMut<MenuScroll>,
    filter: Res<MenuFilter>,
    collapsed: Res<MenuCollapsed>,
) {
    let entry_count = menu_layout_indices(&filter, &collapsed).len();
    if entry_count == 0 {
        wheel.clear();
        return;
    }
    let max_scroll = menu_max_scroll(entry_count);
    for event in wheel.read() {
        let delta = match event.unit {
            MouseScrollUnit::Line => event.y * 40.0,
            MouseScrollUnit::Pixel => event.y,
        };
        scroll.offset = (scroll.offset - delta).clamp(0.0, max_scroll);
    }
}

fn handle_menu_search(
    keys: Res<ButtonInput<KeyCode>>,
    session: Res<Session>,
    mut filter: ResMut<MenuFilter>,
    mut scroll: ResMut<MenuScroll>,
) {
    if session.bench.is_some() {
        return;
    }
    let old_query = filter.query.clone();
    for key in keys.get_just_pressed() {
        match key {
            KeyCode::Backspace => {
                filter.query.pop();
            }
            KeyCode::Escape => filter.query.clear(),
            KeyCode::Space => filter.query.push(' '),
            _ => {
                let name = format!("{key:?}");
                if let Some(letter) = name.strip_prefix("Key").and_then(|s| {
                    (s.len() == 1).then(|| s.chars().next().unwrap().to_ascii_lowercase())
                }) {
                    filter.query.push(letter);
                } else if let Some(digit) = name.strip_prefix("Digit")
                    && digit.len() == 1
                {
                    filter.query.push(digit.chars().next().unwrap());
                }
            }
        }
    }
    if filter.query != old_query {
        scroll.offset = 0.0;
    }
}

/// Moves each menu entry to its scrolled position and hides entries that
/// fall outside the menu's visible viewport so they cannot overlap the
/// header or footer, and are not clickable while off-screen.
fn scroll_menu(
    mut scroll: ResMut<MenuScroll>,
    filter: Res<MenuFilter>,
    collapsed: Res<MenuCollapsed>,
    mut entries: Query<(&MenuEntry, &mut Transform, &mut Visibility)>,
) {
    let layout = menu_layout_indices(&filter, &collapsed);
    scroll.offset = scroll.offset.clamp(0.0, menu_max_scroll(layout.len()));
    let half_height = MENU_ENTRY_SIZE.y * 0.5;
    for (entry, mut transform, mut visibility) in &mut entries {
        let Some(position) = layout.iter().position(|index| *index == entry.0) else {
            *visibility = Visibility::Hidden;
            continue;
        };
        let y = menu_entry_base_y(position) + scroll.offset;
        transform.translation.y = y;
        *visibility = if y + half_height < MENU_BOTTOM || y - half_height > MENU_TOP {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
}

fn update_section_labels(
    collapsed: Res<MenuCollapsed>,
    mut labels: Query<(&MenuSectionLabel, &mut Text2d)>,
) {
    for (label, mut text) in &mut labels {
        let marker = if collapsed.is_collapsed(label.0) {
            "▸"
        } else {
            "▾"
        };
        text.0 = format!("{marker} {}", label.0.label());
    }
}

fn update_search_label(filter: Res<MenuFilter>, mut labels: Query<&mut Text2d, With<SearchLabel>>) {
    let value = if filter.query.is_empty() {
        "FILTER: type an exercise name or number".to_owned()
    } else {
        format!(
            "FILTER: {}  (Backspace removes, Escape clears)",
            filter.query
        )
    };
    for mut label in &mut labels {
        label.0.clone_from(&value);
    }
}

/// Bundles the two menu/bench presentation resources so `handle_mouse` stays
/// under Clippy's argument-count limit.
#[derive(bevy::ecs::system::SystemParam)]
struct MenuState<'w> {
    session: ResMut<'w, Session>,
    scroll: ResMut<'w, MenuScroll>,
    collapsed: ResMut<'w, MenuCollapsed>,
}

/// Drag-driven continuous control: while the left mouse button is held over
/// a dial's track, sets the driven component's control ratio from the
/// cursor's horizontal position. Presentation input, routed through the same
/// `Action::SetControlRatio` path as any other caller.
fn handle_dial(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    tracks: Query<&DialTrack>,
    mut session: ResMut<Session>,
) {
    if !mouse.pressed(MouseButton::Left) {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(point) = cursor_world(window) else {
        return;
    };
    for track in &tracks {
        let half = track.size * 0.5;
        if (point - track.center).abs().cmpgt(half).any() {
            continue;
        }
        let ratio = ((point.x - (track.center.x - half.x)) / track.size.x) as f64;
        if let Some(bench) = &mut session.bench {
            bench.act(Action::SetControlRatio {
                component: track.component.clone(),
                ratio: ratio.clamp(0.0, 1.0),
            });
        }
        return;
    }
}

/// Repositions a dial's handle sprite from the live control ratio each frame.
fn update_dial_handle(session: Res<Session>, mut handles: Query<(&DialHandle, &mut Transform)>) {
    let Some(bench) = &session.bench else {
        return;
    };
    for (handle, mut transform) in &mut handles {
        let ratio = bench
            .simulation
            .control_ratios
            .get(&handle.component)
            .copied()
            .unwrap_or(0.5);
        transform.translation.x =
            handle.center.x - handle.size.x / 2.0 + ratio as f32 * handle.size.x;
    }
}

fn handle_mouse(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    targets: Query<(&Transform, &ClickTarget, Option<&Visibility>)>,
    scene: Query<Entity, With<SceneEntity>>,
    mut commands: Commands,
    mut state: MenuState,
    mut images: ResMut<Assets<Image>>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(point) = cursor_world(window) else {
        return;
    };
    let selected = targets.iter().find_map(|(transform, target, visibility)| {
        if visibility == Some(&Visibility::Hidden) {
            return None;
        }
        ((point - transform.translation.truncate())
            .abs()
            .cmple(target.1 * 0.5)
            .all())
        .then_some(target.0)
    });
    match selected {
        Some(Control::Select(circuit)) => {
            clear_scene(&mut commands, &scene);
            let bench = Bench::new(circuit);
            spawn_bench(&mut commands, &mut images, &bench);
            state.session.bench = Some(bench);
        }
        Some(Control::ToggleSection(section)) => {
            state.collapsed.toggle(section);
            state.scroll.offset = 0.0;
        }
        Some(Control::Back) => {
            clear_scene(&mut commands, &scene);
            state.session.bench = None;
            state.scroll.offset = 0.0;
            spawn_menu(&mut commands);
        }
        Some(Control::RunPause) => {
            if let Some(bench) = &mut state.session.bench {
                bench.act(if bench.simulation.running {
                    Action::Pause
                } else {
                    Action::Run
                });
            }
        }
        Some(Control::Reset) => {
            if let Some(bench) = &mut state.session.bench {
                bench.act(Action::Reset);
            }
        }
        Some(Control::Toggle(index)) => {
            if let Some(bench) = &mut state.session.bench {
                bench.toggle(index);
            }
        }
        None => {}
    }
}

fn handle_keyboard(keys: Res<ButtonInput<KeyCode>>, mut session: ResMut<Session>) {
    let Some(bench) = &mut session.bench else {
        return;
    };
    if keys.just_pressed(KeyCode::Space) {
        bench.act(if bench.simulation.running {
            Action::Pause
        } else {
            Action::Run
        });
    }
    if keys.just_pressed(KeyCode::KeyR) {
        bench.act(Action::Reset);
    }
    if keys.just_pressed(KeyCode::KeyC) {
        bench.toggle(0);
    }
}

fn fixed_step(mut session: ResMut<Session>) {
    if let Some(bench) = &mut session.bench
        && bench.simulation.running
    {
        advance_steps(&bench.project, &mut bench.simulation, 1);
    }
}

fn update_view(
    mut commands: Commands,
    session: Res<Session>,
    windows: Query<&Window>,
    sound_tones: Option<Res<SoundTones>>,
    mut labels: Query<(&Readout, &mut Text2d, &mut TextColor)>,
    mut parts: Query<(Entity, &mut PartVisual, &mut Sprite)>,
) {
    let Some(bench) = &session.bench else {
        return;
    };
    let hover = windows
        .single()
        .ok()
        .and_then(cursor_world)
        .and_then(hole_at)
        .map(|id| {
            let mut contacts = Vec::new();
            for component in &bench.project.components {
                for (pin, hole) in &component.pins {
                    if hole.0 == id {
                        contacts.push(format!("{} {}", component.id.0, pin.0));
                    }
                }
            }
            for wire in &bench.project.wires {
                if wire.from.0 == id {
                    contacts.push(format!("{} to {}", wire.id.0, wire.to.0));
                } else if wire.to.0 == id {
                    contacts.push(format!("{} to {}", wire.id.0, wire.from.0));
                }
            }
            if contacts.is_empty() {
                id
            } else {
                format!("{id} - {}", contacts.join(", "))
            }
        })
        .unwrap_or_default();
    for (kind, mut text, mut color) in &mut labels {
        text.0 = match kind {
            Readout::Status => {
                if bench.simulation.stale {
                    color.0 = Color::srgb(1.0, 0.40, 0.32);
                    format!(
                        "CALCULATION FAILED: {}",
                        bench
                            .simulation
                            .diagnostics
                            .first()
                            .map_or("unknown", |d| d.message.as_str())
                    )
                } else {
                    color.0 = Color::srgb(0.72, 0.82, 0.83);
                    format!(
                        "{} - {:.3} s",
                        if bench.simulation.running {
                            "Running"
                        } else {
                            "Paused"
                        },
                        bench.simulation.time_seconds()
                    )
                }
            }
            Readout::Value => {
                if bench.simulation.stale {
                    "Readings stale".into()
                } else if bench.circuit == Circuit::Rc {
                    format!(
                        "C1  {:.3} V",
                        bench.simulation.capacitor_voltages[&ComponentId("C1".into())]
                    )
                } else if bench.circuit == Circuit::E7 {
                    bench.simulation.last_valid.as_ref().map_or(
                        "Buzzer current: run to measure".into(),
                        |result| {
                            format!(
                                "BZ1  {:.2} mA",
                                1000.0
                                    * result
                                        .resistor_currents
                                        .get(&ComponentId("BZ1".into()))
                                        .copied()
                                        .unwrap_or(0.0)
                            )
                        },
                    )
                } else {
                    bench.simulation.last_valid.as_ref().map_or(
                        "LED current: run to measure".into(),
                        |result| {
                            let extra =
                                if matches!(bench.circuit, Circuit::E10 | Circuit::C01S01_06) {
                                    format!(
                                        "  C1 {:.3} V",
                                        bench.simulation.capacitor_voltages
                                            [&ComponentId("C1".into())]
                                    )
                                } else {
                                    String::new()
                                };
                            format!(
                                "D1  {:.2} mA{extra}",
                                1000.0
                                    * result
                                        .led_currents
                                        .get(&ComponentId("D1".into()))
                                        .copied()
                                        .unwrap_or(0.0)
                            )
                        },
                    )
                }
            }
            Readout::Control => bench
                .circuit
                .controls()
                .iter()
                .map(|spec| {
                    let id = ComponentId(spec.component.into());
                    let state = bench.simulation.controls[&id];
                    let text = match state {
                        ControlState::SwitchNormallyClosed => "charging path",
                        ControlState::SwitchNormallyOpen => "discharge path",
                        ControlState::ButtonPressed => "pressed",
                        ControlState::ButtonReleased => "released",
                    };
                    format!("{}: {text}", spec.component)
                })
                .collect::<Vec<_>>()
                .join("   "),
            Readout::Hover => hover.clone(),
        };
    }
    let readings = bench
        .simulation
        .last_valid
        .as_ref()
        .filter(|_| !bench.simulation.stale);
    for (entity, mut part, mut sprite) in &mut parts {
        let Some(art) = sprites::art_for(part.kind) else {
            continue;
        };
        let context = sprites::PartContext {
            led_current: readings
                .and_then(|r| r.led_currents.get(&part.id))
                .copied()
                .unwrap_or(0.0),
            buzzer_current: readings
                .and_then(|r| r.resistor_currents.get(&part.id))
                .copied()
                .unwrap_or(0.0),
            control: bench.simulation.controls.get(&part.id).copied(),
        };
        let state = art.state(&context).min(part.states.len() - 1);
        if state != part.shown {
            sprite.image = part.states[state].clone();
            part.shown = state;
        }
        // A buzzer/speaker's sprite state 1 is its current-driven "sounding"
        // state (see `sprites::buzzer`/`sprites::speaker`); play or stop its
        // tone exactly when that state changes.
        let sounding =
            state == 1 && matches!(part.kind, ComponentKind::Buzzer | ComponentKind::Speaker);
        if let Some(tones) = &sound_tones
            && sounding != part.sounding
        {
            part.sounding = sounding;
            if sounding {
                let tone = match part.kind {
                    ComponentKind::Speaker => tones.speaker.clone(),
                    _ => tones.buzzer.clone(),
                };
                commands
                    .entity(entity)
                    .insert((AudioPlayer(tone), PlaybackSettings::LOOP));
            } else {
                commands
                    .entity(entity)
                    .remove::<(AudioPlayer<Pitch>, AudioSink)>();
            }
        }
    }
}

fn hole_at(point: Vec2) -> Option<String> {
    for row in 1..=30 {
        for col in 'A'..='J' {
            let id = format!("{col}{row}");
            if point.distance(hole_position(&id)?) <= 6.0 {
                return Some(id);
            }
        }
        for rail in ["TP+", "TP-", "BP+", "BP-"] {
            let id = format!("{rail}:{row}");
            if point.distance(hole_position(&id)?) <= 6.0 {
                return Some(id);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::{TimePlugin, TimeUpdateStrategy};
    use std::collections::{BTreeMap, BTreeSet};
    use std::time::Duration;

    fn click(app: &mut App, window: Entity, point: Vec2) {
        let mut window = app.world_mut().get_mut::<Window>(window).unwrap();
        let width = window.width();
        let height = window.height();
        let world_width = 1200.0_f32.max(760.0 * width / height);
        let world_height = 760.0_f32.max(1200.0 * height / width);
        window.set_cursor_position(Some(Vec2::new(
            width * (0.5 + point.x / world_width),
            height * (0.5 - point.y / world_height),
        )));
        let mut mouse = ButtonInput::<MouseButton>::default();
        mouse.press(MouseButton::Left);
        app.world_mut().insert_resource(mouse);
        app.update();
    }

    #[test]
    fn all_embedded_boards_have_unique_lead_and_wire_holes() {
        for circuit in Circuit::all() {
            let bench = Bench::new(circuit);
            let pins = bench
                .project
                .components
                .iter()
                .flat_map(|c| c.pins.values().map(|h| h.0.clone()));
            let endpoints = bench
                .project
                .wires
                .iter()
                .flat_map(|w| [&w.from.0, &w.to.0].map(Clone::clone));
            let all: Vec<_> = pins.chain(endpoints).collect();
            assert_eq!(
                all.len(),
                all.iter().collect::<BTreeSet<_>>().len(),
                "{circuit:?}"
            );
            assert!(all.iter().all(|hole| hole_position(hole).is_some()));
        }
    }

    #[test]
    fn each_circuit_uses_core_controls_and_reset() {
        for circuit in Circuit::all() {
            let mut bench = Bench::new(circuit);
            bench.toggle(0);
            bench.act(Action::Run);
            advance_steps(&bench.project, &mut bench.simulation, 100);
            assert_eq!(bench.simulation.step, 100, "{circuit:?}");
            assert!(!bench.simulation.stale);
            bench.act(Action::Reset);
            assert_eq!(bench.simulation.step, 0);
            assert!(!bench.simulation.running);
        }
    }

    #[test]
    fn rc_switch_charges_and_discharges_from_the_embedded_board() {
        let mut bench = Bench::new(Circuit::Rc);
        bench.act(Action::Run);
        advance_steps(&bench.project, &mut bench.simulation, 1_000);
        let charged = bench.simulation.capacitor_voltages[&ComponentId("C1".into())];
        assert!((charged - 5.0 * (1.0 - (-1.0_f64).exp())).abs() < 0.05);
        bench.toggle(0);
        advance_steps(&bench.project, &mut bench.simulation, 1_000);
        let discharged = bench.simulation.capacitor_voltages[&ComponentId("C1".into())];
        assert!((discharged - charged * (-1.0_f64).exp()).abs() < 0.05);
        bench.act(Action::Reset);
        assert_eq!(
            bench.simulation.capacitor_voltages[&ComponentId("C1".into())],
            0.0
        );
        assert_eq!(
            bench.simulation.controls[&ComponentId("S1".into())],
            ControlState::SwitchNormallyClosed
        );
    }

    #[test]
    fn all_circuits_are_selectable_via_the_scrolled_menu() {
        let mut app = App::new();
        app.insert_resource(Session::default())
            .insert_resource(MenuScroll::default())
            .insert_resource(MenuFilter::default())
            .insert_resource(MenuCollapsed::all_expanded())
            .insert_resource(ButtonInput::<MouseButton>::default())
            .init_resource::<Assets<Image>>()
            .add_systems(Startup, setup)
            .add_systems(Update, (scroll_menu, handle_mouse).chain());
        let window = app
            .world_mut()
            .spawn(Window {
                resolution: (1200, 760).into(),
                ..default()
            })
            .id();
        app.update();
        let entries = exercise_catalog::items();
        let max_scroll = menu_max_scroll(entries.len());
        for (index, item) in entries.into_iter().enumerate() {
            let exercise_catalog::Item::Circuit(circuit) = item else {
                continue;
            };
            let offset = (menu_entry_base_y(0) - menu_entry_base_y(index)).clamp(0.0, max_scroll);
            app.world_mut().resource_mut::<MenuScroll>().offset = offset;
            let y = menu_entry_base_y(index) + offset;
            click(&mut app, window, Vec2::new(0.0, y));
            assert_eq!(
                app.world()
                    .resource::<Session>()
                    .bench
                    .as_ref()
                    .unwrap()
                    .circuit,
                circuit,
                "index {index}"
            );
            // Return to the menu for the next iteration; the Back button
            // moves down when a bench has more than one control button.
            // Return to the menu for the next iteration.
            click(&mut app, window, Vec2::new(185.0, -328.0));
        }
    }

    #[test]
    fn exercise_catalog_keeps_search_groups_and_circuits_separate() {
        let items = exercise_catalog::items();
        assert_eq!(
            items
                .iter()
                .filter(|item| matches!(item, exercise_catalog::Item::Section(_)))
                .count(),
            4
        );
        assert_eq!(
            items
                .iter()
                .filter(|item| matches!(item, exercise_catalog::Item::Circuit(_)))
                .count(),
            58
        );
        assert!(matches!(
            items[0],
            exercise_catalog::Item::Section(exercise_catalog::Section::Mvp)
        ));
        assert!(matches!(
            items[4],
            exercise_catalog::Item::Section(exercise_catalog::Section::Foundations)
        ));
    }

    #[test]
    fn menu_filter_keeps_only_matching_exercises_clickable() {
        let filter = MenuFilter {
            query: "e27".into(),
        };
        let visible: Vec<_> = menu_layout_indices(&filter, &MenuCollapsed::all_expanded())
            .into_iter()
            .filter_map(|index| match exercise_catalog::items()[index] {
                exercise_catalog::Item::Circuit(circuit) => {
                    Some(circuit.label().to_ascii_lowercase())
                }
                exercise_catalog::Item::Section(_) => None,
            })
            .collect();
        assert_eq!(visible, vec!["e27: shared brightness control"]);
    }

    #[test]
    fn keyboard_search_builds_an_exercise_query_and_escape_clears_it() {
        let mut app = App::new();
        app.insert_resource(MenuFilter::default())
            .insert_resource(MenuScroll::default())
            .insert_resource(Session::default())
            .insert_resource(ButtonInput::<KeyCode>::default())
            .add_systems(Update, handle_menu_search);

        for key in [KeyCode::KeyE, KeyCode::Digit2, KeyCode::Digit7] {
            let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            input.clear();
            input.press(key);
            app.update();
        }
        assert_eq!(app.world().resource::<MenuFilter>().query, "e27");

        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.clear();
        input.press(KeyCode::Escape);
        app.update();
        assert!(app.world().resource::<MenuFilter>().query.is_empty());
    }

    #[test]
    fn dragging_the_dial_sets_the_control_ratio_and_moves_its_handle() {
        for (circuit, id) in [
            (Circuit::E5, "RV1"),
            (Circuit::E6, "RV1"),
            (Circuit::E16, "R3"),
            (Circuit::E17, "R2"),
            (Circuit::E18, "RV1"),
        ] {
            let mut app = App::new();
            app.insert_resource(Session {
                bench: Some(Bench::new(circuit)),
            })
            .insert_resource(ButtonInput::<MouseButton>::default())
            .init_resource::<Assets<Image>>()
            .add_systems(Update, (handle_dial, update_dial_handle).chain());
            let window = app
                .world_mut()
                .spawn(Window {
                    resolution: (1200, 760).into(),
                    ..default()
                })
                .id();
            app.world_mut()
                .resource_scope(|world, mut images: Mut<Assets<Image>>| {
                    let session = world.resource::<Session>();
                    let bench = session.bench.as_ref().unwrap();
                    let mut queue = bevy::ecs::world::CommandQueue::default();
                    let mut commands = Commands::new(&mut queue, world);
                    spawn_bench(&mut commands, &mut images, bench);
                    queue.apply(world);
                });

            let drag = |window: Entity, point: Vec2, app: &mut App| {
                let mut w = app.world_mut().get_mut::<Window>(window).unwrap();
                let width = w.width();
                let height = w.height();
                let world_width = 1200.0_f32.max(760.0 * width / height);
                let world_height = 760.0_f32.max(1200.0 * height / width);
                w.set_cursor_position(Some(Vec2::new(
                    width * (0.5 + point.x / world_width),
                    height * (0.5 - point.y / world_height),
                )));
                let mut mouse = ButtonInput::<MouseButton>::default();
                mouse.press(MouseButton::Left);
                app.world_mut().insert_resource(mouse);
                app.update();
            };

            let id = ComponentId(id.into());
            let left_x = DIAL_TRACK_CENTER.x - DIAL_TRACK_SIZE.x / 2.0 + 2.0;
            drag(window, Vec2::new(left_x, DIAL_TRACK_CENTER.y), &mut app);
            let low_ratio = app
                .world()
                .resource::<Session>()
                .bench
                .as_ref()
                .unwrap()
                .simulation
                .control_ratios[&id];
            assert!(
                low_ratio < 0.05,
                "{circuit:?} ratio should be near 0.0: {low_ratio}"
            );

            let right_x = DIAL_TRACK_CENTER.x + DIAL_TRACK_SIZE.x / 2.0 - 2.0;
            drag(window, Vec2::new(right_x, DIAL_TRACK_CENTER.y), &mut app);
            let high_ratio = app
                .world()
                .resource::<Session>()
                .bench
                .as_ref()
                .unwrap()
                .simulation
                .control_ratios[&id];
            assert!(
                high_ratio > 0.95,
                "{circuit:?} ratio should be near 1.0: {high_ratio}"
            );

            let mut handles = app.world_mut().query::<(&DialHandle, &Transform)>();
            let (_, transform) = handles.iter(app.world()).next().unwrap();
            let expected_x = DIAL_TRACK_CENTER.x - DIAL_TRACK_SIZE.x / 2.0
                + high_ratio as f32 * DIAL_TRACK_SIZE.x;
            assert!(
                (transform.translation.x - expected_x).abs() < 1.0,
                "{circuit:?} handle should track the ratio"
            );
        }
    }

    #[test]
    fn new_exercises_show_title_explanation_and_task_text() {
        for circuit in [
            Circuit::E1,
            Circuit::E2,
            Circuit::E3,
            Circuit::E4,
            Circuit::E7,
            Circuit::E8,
            Circuit::E9,
            Circuit::E10,
            Circuit::E5,
            Circuit::E6,
            Circuit::E11,
            Circuit::E12,
            Circuit::E13,
            Circuit::E14,
            Circuit::E15,
            Circuit::E16,
            Circuit::E17,
            Circuit::E18,
            Circuit::E19,
            Circuit::E20,
            Circuit::E21,
            Circuit::E22,
            Circuit::E23,
            Circuit::E24,
            Circuit::E25,
            Circuit::E26,
            Circuit::E27,
            Circuit::E28,
            Circuit::E29,
            Circuit::E30,
        ] {
            let mut app = App::new();
            app.init_resource::<Assets<Image>>();
            let bench = Bench::new(circuit);
            app.world_mut()
                .resource_scope(|world, mut images: Mut<Assets<Image>>| {
                    let mut queue = bevy::ecs::world::CommandQueue::default();
                    let mut commands = Commands::new(&mut queue, world);
                    spawn_bench(&mut commands, &mut images, &bench);
                    queue.apply(world);
                });
            let mut texts = app.world_mut().query::<&Text2d>();
            let all_text: String = texts
                .iter(app.world())
                .map(|t| t.0.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            assert!(all_text.contains(circuit.label()), "{circuit:?} title");
            let (explanation, task) = circuit.explanation_and_task();
            assert!(!explanation.is_empty(), "{circuit:?} explanation");
            assert!(
                all_text.contains(explanation),
                "{circuit:?} explanation shown"
            );
            assert!(all_text.contains(task), "{circuit:?} task shown");
            for component in &bench.project.components {
                assert!(
                    all_text.contains(&component.id.0),
                    "{circuit:?} parts list missing {}",
                    component.id.0
                );
            }
        }
    }

    #[test]
    fn shared_menu_and_visible_buttons_dispatch_core_actions() {
        let mut app = App::new();
        app.insert_resource(Session::default())
            .insert_resource(MenuScroll::default())
            .insert_resource(MenuFilter::default())
            .insert_resource(MenuCollapsed::all_expanded())
            .insert_resource(ButtonInput::<MouseButton>::default())
            .init_resource::<Assets<Image>>()
            .add_systems(Startup, setup)
            .add_systems(Update, (scroll_menu, handle_mouse).chain());
        let window = app
            .world_mut()
            .spawn(Window {
                resolution: (1200, 760).into(),
                ..default()
            })
            .id();
        app.update();
        // The first menu slot is the catalog header; the first circuit is
        // immediately below it.
        click(&mut app, window, Vec2::new(0.0, menu_entry_base_y(1)));
        assert_eq!(
            app.world()
                .resource::<Session>()
                .bench
                .as_ref()
                .unwrap()
                .circuit,
            Circuit::Led
        );
        click(&mut app, window, Vec2::new(205.0, -245.0));
        assert_eq!(
            app.world()
                .resource::<Session>()
                .bench
                .as_ref()
                .unwrap()
                .simulation
                .controls[&ComponentId("B1".into())],
            ControlState::ButtonPressed
        );
        click(&mut app, window, Vec2::new(35.0, -170.0));
        assert!(
            app.world()
                .resource::<Session>()
                .bench
                .as_ref()
                .unwrap()
                .simulation
                .running
        );
        click(&mut app, window, Vec2::new(215.0, -170.0));
        assert!(
            !app.world()
                .resource::<Session>()
                .bench
                .as_ref()
                .unwrap()
                .simulation
                .running
        );
        click(&mut app, window, Vec2::new(185.0, -328.0));
        assert!(app.world().resource::<Session>().bench.is_none());
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .resolution
            .set(828.0, 1054.0);
        click(&mut app, window, Vec2::new(0.0, menu_entry_base_y(2)));
        assert_eq!(
            app.world()
                .resource::<Session>()
                .bench
                .as_ref()
                .unwrap()
                .circuit,
            Circuit::Rc
        );
    }

    #[test]
    fn failure_state_marks_visible_readings_stale() {
        let mut app = App::new();
        app.insert_resource(Session {
            bench: Some(Bench::new(Circuit::Led)),
        })
        .add_systems(Update, update_view);
        app.world_mut().spawn(Window::default());
        app.world_mut()
            .spawn((Readout::Status, Text2d::default(), TextColor::default()));
        app.world_mut()
            .spawn((Readout::Value, Text2d::default(), TextColor::default()));
        {
            let mut bench = app.world_mut().resource_mut::<Session>();
            let state = &mut bench.bench.as_mut().unwrap().simulation;
            state.stale = true;
            state
                .diagnostics
                .push(bredboard_core::SimulationDiagnostic {
                    code: "nonconvergence".into(),
                    path: String::new(),
                    message: "nonlinear circuit did not converge within 80 iterations".into(),
                });
        }
        app.update();
        let mut readouts = app.world_mut().query::<(&Readout, &Text2d)>();
        let values: Vec<_> = readouts
            .iter(app.world())
            .map(|(kind, value)| (std::mem::discriminant(kind), value.0.clone()))
            .collect();
        assert!(values.iter().any(|(_, value)| value == "Readings stale"));
        assert!(
            values
                .iter()
                .any(|(_, value)| value.contains("CALCULATION FAILED"))
        );
    }

    #[test]
    fn part_sprites_follow_core_led_current_and_button_state() {
        let mut app = App::new();
        app.insert_resource(Session {
            bench: Some(Bench::new(Circuit::Led)),
        })
        .init_resource::<Assets<Image>>()
        .add_systems(Update, update_view);
        app.world_mut().spawn(Window::default());
        app.world_mut()
            .resource_scope(|world, mut images: Mut<Assets<Image>>| {
                let session = world.resource::<Session>();
                let bench = session.bench.as_ref().unwrap();
                let mut queue = bevy::ecs::world::CommandQueue::default();
                let mut commands = Commands::new(&mut queue, world);
                spawn_bench(&mut commands, &mut images, bench);
                queue.apply(world);
            });
        let shown = |app: &mut App, id: &str| {
            let mut parts = app.world_mut().query::<&PartVisual>();
            parts
                .iter(app.world())
                .find(|p| p.id.0 == id)
                .map(|p| (p.shown, p.states.len()))
                .unwrap()
        };
        app.update();
        assert_eq!(shown(&mut app, "D1"), (0, 3));
        assert_eq!(shown(&mut app, "B1"), (0, 2));
        assert_eq!(shown(&mut app, "R1"), (0, 1));
        {
            let mut session = app.world_mut().resource_mut::<Session>();
            let bench = session.bench.as_mut().unwrap();
            bench.toggle(0);
            bench.act(Action::Run);
            advance_steps(&bench.project, &mut bench.simulation, 10);
        }
        app.update();
        assert_eq!(shown(&mut app, "D1").0, 2, "8.6 mA lights the LED");
        assert_eq!(shown(&mut app, "B1").0, 1);
        app.world_mut()
            .resource_mut::<Session>()
            .bench
            .as_mut()
            .unwrap()
            .simulation
            .stale = true;
        app.update();
        assert_eq!(
            shown(&mut app, "D1").0,
            0,
            "stale readings do not light the LED"
        );
    }

    #[test]
    fn different_frame_pacing_keeps_fixed_rc_results() {
        fn run_frames(frames: &[u64]) -> SimulationState {
            let mut bench = Bench::new(Circuit::Rc);
            bench.act(Action::Run);
            let mut app = App::new();
            app.add_plugins(TimePlugin)
                .insert_resource(Time::<Virtual>::from_max_delta(Duration::MAX))
                .insert_resource(Time::<Fixed>::from_hz(1_000.0))
                .insert_resource(Session { bench: Some(bench) })
                .add_systems(FixedUpdate, fixed_step);
            app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
            app.update();
            for milliseconds in frames {
                app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
                    *milliseconds,
                )));
                app.update();
            }
            app.world()
                .resource::<Session>()
                .bench
                .as_ref()
                .unwrap()
                .simulation
                .clone()
        }
        let single = run_frames(&[1_000]);
        let split = run_frames(&[100, 400, 500]);
        assert_eq!(single.step, 1_000);
        assert_eq!(single, split);
    }

    /// Debug-only stand-in for the eventual 13-entry list (T22/T23): spawns
    /// `count` menu entries so scrolling can be exercised without depending
    /// on real fixed circuits.
    fn spawn_stub_menu(commands: &mut Commands, count: usize) {
        for index in 0..count {
            menu_entry_button(
                commands,
                &format!("STUB {index}"),
                index,
                Control::Select(Circuit::Led),
            );
        }
    }

    fn visible_entries(app: &mut App) -> BTreeSet<usize> {
        let mut query = app
            .world_mut()
            .query_filtered::<(&MenuEntry, &Visibility), With<ClickTarget>>();
        query
            .iter(app.world())
            .filter(|(_, visibility)| **visibility != Visibility::Hidden)
            .map(|(entry, _)| entry.0)
            .collect()
    }

    #[test]
    fn thirteen_stub_entries_are_all_reachable_by_scrolling_without_overlap() {
        let mut app = App::new();
        app.insert_resource(MenuScroll::default())
            .insert_resource(MenuFilter::default())
            .insert_resource(MenuCollapsed::all_expanded())
            .init_resource::<Assets<Image>>()
            .add_systems(Update, scroll_menu);
        app.world_mut()
            .resource_scope(|world, mut _images: Mut<Assets<Image>>| {
                let mut queue = bevy::ecs::world::CommandQueue::default();
                let mut commands = Commands::new(&mut queue, world);
                spawn_stub_menu(&mut commands, 13);
                queue.apply(world);
            });
        app.update();
        let mut seen = BTreeSet::new();
        seen.extend(visible_entries(&mut app));
        let max_scroll = menu_max_scroll(
            menu_layout_indices(&MenuFilter::default(), &MenuCollapsed::all_expanded()).len(),
        );
        assert!(max_scroll > 0.0, "stub entries must overflow the viewport");
        let steps = 20;
        for step in 0..=steps {
            app.world_mut().resource_mut::<MenuScroll>().offset =
                max_scroll * step as f32 / steps as f32;
            app.update();
            seen.extend(visible_entries(&mut app));
        }
        assert_eq!(seen, (0..13).collect::<BTreeSet<_>>());

        // No two visible entries may overlap: their base Y spacing already
        // exceeds their height, so overlap-freedom reduces to checking every
        // visible entry sits within the declared viewport band.
        let half_height = MENU_ENTRY_SIZE.y * 0.5;
        let mut transforms = app
            .world_mut()
            .query::<(&MenuEntry, &Transform, &Visibility)>();
        for (_, transform, visibility) in transforms.iter(app.world()) {
            if *visibility == Visibility::Hidden {
                continue;
            }
            let y = transform.translation.y;
            assert!(y + half_height >= MENU_BOTTOM && y - half_height <= MENU_TOP);
        }
    }

    #[test]
    fn scroll_offset_is_clamped_to_first_and_last_entry() {
        let mut app = App::new();
        app.insert_resource(MenuScroll::default())
            .insert_resource(MenuFilter::default())
            .insert_resource(MenuCollapsed::all_expanded())
            .init_resource::<Assets<Image>>()
            .add_message::<MouseWheel>()
            .add_systems(Update, handle_scroll);
        app.world_mut()
            .resource_scope(|world, mut _images: Mut<Assets<Image>>| {
                let mut queue = bevy::ecs::world::CommandQueue::default();
                let mut commands = Commands::new(&mut queue, world);
                spawn_stub_menu(&mut commands, 13);
                queue.apply(world);
            });
        let max_scroll = menu_max_scroll(
            menu_layout_indices(&MenuFilter::default(), &MenuCollapsed::all_expanded()).len(),
        );
        // A huge downward scroll must clamp at the last entry, not overshoot.
        app.world_mut().write_message(MouseWheel {
            unit: MouseScrollUnit::Pixel,
            x: 0.0,
            y: -10_000.0,
            window: Entity::PLACEHOLDER,
            phase: bevy::input::touch::TouchPhase::Moved,
        });
        app.update();
        assert_eq!(app.world().resource::<MenuScroll>().offset, max_scroll);
        // A huge upward scroll must clamp back at the first entry.
        app.world_mut().write_message(MouseWheel {
            unit: MouseScrollUnit::Pixel,
            x: 0.0,
            y: 10_000.0,
            window: Entity::PLACEHOLDER,
            phase: bevy::input::touch::TouchPhase::Moved,
        });
        app.update();
        assert_eq!(app.world().resource::<MenuScroll>().offset, 0.0);
    }

    #[test]
    fn existing_three_entry_menu_layout_is_unchanged() {
        assert_eq!(
            menu_entry_base_y(0) - menu_entry_base_y(1),
            MENU_ENTRY_HEIGHT
        );
        assert_eq!(
            menu_entry_base_y(1) - menu_entry_base_y(2),
            MENU_ENTRY_HEIGHT
        );
        assert_eq!(menu_max_scroll(3), 0.0);
    }

    #[test]
    fn menu_groups_collapse_and_search_reveals_matching_exercises() {
        let filter = MenuFilter::default();
        let collapsed = MenuCollapsed::default();
        assert_eq!(menu_layout_indices(&filter, &collapsed).len(), 7);

        let mut expanded = MenuCollapsed::default();
        expanded.toggle(exercise_catalog::Section::Advanced);
        assert!(menu_layout_indices(&filter, &expanded).len() > 7);

        let filter = MenuFilter {
            query: "e27".into(),
        };
        assert_eq!(menu_layout_indices(&filter, &collapsed).len(), 5);
    }

    #[test]
    fn clicking_a_group_header_toggles_its_exercises() {
        let mut app = App::new();
        app.insert_resource(Session::default())
            .insert_resource(MenuScroll::default())
            .insert_resource(MenuFilter::default())
            .insert_resource(MenuCollapsed::default())
            .insert_resource(ButtonInput::<MouseButton>::default())
            .init_resource::<Assets<Image>>()
            .add_systems(Startup, setup)
            .add_systems(Update, (scroll_menu, handle_mouse).chain());
        let window = app
            .world_mut()
            .spawn(Window {
                resolution: (1200, 760).into(),
                ..default()
            })
            .id();
        app.update();

        let foundation = exercise_catalog::Section::Foundations;
        let header_index = exercise_catalog::items()
            .iter()
            .position(|item| matches!(item, exercise_catalog::Item::Section(value) if *value == foundation))
            .unwrap();
        let row = menu_layout_indices(&MenuFilter::default(), &MenuCollapsed::default())
            .iter()
            .position(|index| *index == header_index)
            .unwrap();
        click(&mut app, window, Vec2::new(0.0, menu_entry_base_y(row)));
        assert!(
            !app.world()
                .resource::<MenuCollapsed>()
                .is_collapsed(foundation)
        );
        click(&mut app, window, Vec2::new(0.0, menu_entry_base_y(row)));
        assert!(
            app.world()
                .resource::<MenuCollapsed>()
                .is_collapsed(foundation)
        );
    }

    #[test]
    fn e11_switch_selects_exactly_one_led() {
        let mut bench = Bench::new(Circuit::E11);
        bench.act(Action::Run);
        advance_steps(&bench.project, &mut bench.simulation, 10);
        let result = bench.simulation.last_valid.as_ref().unwrap();
        let d1 = result.led_currents[&ComponentId("D1".into())];
        let d2 = result.led_currents[&ComponentId("D2".into())];
        assert!(d1 > 0.001 && d2.abs() < 1e-6, "default position: {d1} {d2}");

        bench.toggle(0);
        advance_steps(&bench.project, &mut bench.simulation, 10);
        let result = bench.simulation.last_valid.as_ref().unwrap();
        let d1 = result.led_currents[&ComponentId("D1".into())];
        let d2 = result.led_currents[&ComponentId("D2".into())];
        assert!(d1.abs() < 1e-6 && d2 > 0.001, "toggled position: {d1} {d2}");
    }

    #[test]
    fn e12_branches_are_independently_solvable() {
        let bench = Bench::new(Circuit::E12);
        let empty_ratios = BTreeMap::new();
        let combined =
            bredboard_core::solve_dc(&bench.project, &bench.simulation.controls, &empty_ratios)
                .unwrap();

        let mut series_only = bench.project.clone();
        series_only
            .components
            .retain(|c| !["R2", "D3"].contains(&c.id.0.as_str()));
        series_only
            .wires
            .retain(|w| !["W3", "W4"].contains(&w.id.0.as_str()));
        let series_result =
            bredboard_core::solve_dc(&series_only, &bench.simulation.controls, &empty_ratios)
                .unwrap();

        let mut parallel_only = bench.project.clone();
        parallel_only
            .components
            .retain(|c| !["R1", "D1", "D2"].contains(&c.id.0.as_str()));
        parallel_only
            .wires
            .retain(|w| !["W1", "W2"].contains(&w.id.0.as_str()));
        let parallel_result =
            bredboard_core::solve_dc(&parallel_only, &bench.simulation.controls, &empty_ratios)
                .unwrap();

        for id in ["D1", "D2"] {
            let component = ComponentId(id.into());
            assert!(
                (combined.led_currents[&component] - series_result.led_currents[&component]).abs()
                    < 1e-9,
                "{id} current should not depend on the parallel branch"
            );
        }
        let component = ComponentId("D3".into());
        assert!(
            (combined.led_currents[&component] - parallel_result.led_currents[&component]).abs()
                < 1e-9,
            "D3 current should not depend on the series branch"
        );
    }

    #[test]
    fn e14_or_truth_table_across_all_combinations() {
        for (s1, s2) in [
            (ControlState::ButtonReleased, ControlState::ButtonReleased),
            (ControlState::ButtonPressed, ControlState::ButtonReleased),
            (ControlState::ButtonReleased, ControlState::ButtonPressed),
            (ControlState::ButtonPressed, ControlState::ButtonPressed),
        ] {
            let mut bench = Bench::new(Circuit::E14);
            bench.act(Action::SetControl {
                component: ComponentId("S1".into()),
                state: s1,
            });
            bench.act(Action::SetControl {
                component: ComponentId("S2".into()),
                state: s2,
            });
            bench.act(Action::Run);
            advance_steps(&bench.project, &mut bench.simulation, 10);
            let current = bench.simulation.last_valid.as_ref().unwrap().led_currents
                [&ComponentId("D1".into())];
            let expected_on =
                s1 == ControlState::ButtonPressed || s2 == ControlState::ButtonPressed;
            assert_eq!(current > 0.001, expected_on, "S1={s1:?} S2={s2:?}");
        }
    }

    #[test]
    fn e15_inverter_lights_when_unpressed_and_darkens_when_pressed() {
        let mut bench = Bench::new(Circuit::E15);
        bench.act(Action::Run);
        advance_steps(&bench.project, &mut bench.simulation, 10);
        let unpressed =
            bench.simulation.last_valid.as_ref().unwrap().led_currents[&ComponentId("D1".into())];
        assert!(
            unpressed > 0.001,
            "LED should be lit when unpressed: {unpressed}"
        );

        bench.toggle(0);
        advance_steps(&bench.project, &mut bench.simulation, 10);
        let pressed =
            bench.simulation.last_valid.as_ref().unwrap().led_currents[&ComponentId("D1".into())];
        assert!(
            pressed.abs() < 1e-6,
            "LED should be dark when pressed: {pressed}"
        );
    }

    #[test]
    fn e16_night_light_is_brighter_in_darkness_than_in_bright_light() {
        let mut bench = Bench::new(Circuit::E16);
        bench.act(Action::SetControlRatio {
            component: ComponentId("R3".into()),
            ratio: 0.0,
        });
        bench.act(Action::Run);
        advance_steps(&bench.project, &mut bench.simulation, 10);
        let dark =
            bench.simulation.last_valid.as_ref().unwrap().led_currents[&ComponentId("D1".into())];

        bench.act(Action::SetControlRatio {
            component: ComponentId("R3".into()),
            ratio: 1.0,
        });
        advance_steps(&bench.project, &mut bench.simulation, 10);
        let bright =
            bench.simulation.last_valid.as_ref().unwrap().led_currents[&ComponentId("D1".into())];
        assert!(
            dark > 0.001 && bright < 0.000_5,
            "dark={dark}, bright={bright}"
        );
    }

    #[test]
    fn e17_light_alarm_crosses_from_silent_to_sounding() {
        let mut bench = Bench::new(Circuit::E17);
        bench.act(Action::SetControlRatio {
            component: ComponentId("R2".into()),
            ratio: 0.0,
        });
        bench.act(Action::Run);
        advance_steps(&bench.project, &mut bench.simulation, 10);
        let dark = bench
            .simulation
            .last_valid
            .as_ref()
            .unwrap()
            .resistor_currents[&ComponentId("BZ1".into())];

        bench.act(Action::SetControlRatio {
            component: ComponentId("R2".into()),
            ratio: 1.0,
        });
        advance_steps(&bench.project, &mut bench.simulation, 10);
        let bright = bench
            .simulation
            .last_valid
            .as_ref()
            .unwrap()
            .resistor_currents[&ComponentId("BZ1".into())];
        assert!(
            dark < 0.001 && bright >= 0.001,
            "dark={dark}, bright={bright}"
        );
    }

    #[test]
    fn e18_volume_control_buzzer_current_decreases_to_silence() {
        let mut bench = Bench::new(Circuit::E18);
        bench.act(Action::Run);
        let mut previous = f64::INFINITY;
        let mut silent = false;
        for ratio in [0.0, 0.25, 0.5, 0.75, 1.0] {
            bench.act(Action::SetControlRatio {
                component: ComponentId("RV1".into()),
                ratio,
            });
            advance_steps(&bench.project, &mut bench.simulation, 1);
            let current = bench
                .simulation
                .last_valid
                .as_ref()
                .unwrap()
                .resistor_currents[&ComponentId("BZ1".into())];
            assert!(
                current <= previous + 1e-12,
                "ratio={ratio}, current={current}"
            );
            silent |= current < 0.001;
            previous = current;
        }
        assert!(silent, "the high-resistance end must be silent");
    }

    #[test]
    fn e19_three_led_currents_match_their_independent_resistors() {
        let mut bench = Bench::new(Circuit::E19);
        bench.act(Action::Run);
        advance_steps(&bench.project, &mut bench.simulation, 10);
        let result = bench.simulation.last_valid.as_ref().unwrap();
        for id in ["R1", "R2", "R3"] {
            let resistor = ComponentId(id.into());
            let led = ComponentId(format!("D{}", &id[1..]));
            assert!(
                (result.resistor_currents[&resistor] - result.led_currents[&led]).abs() < 1e-12
            );
            assert!(result.led_currents[&led] > 0.001, "{id}");
        }
    }

    #[test]
    fn e20_turn_on_delay_is_measured_in_fixed_simulation_steps() {
        let mut bench = Bench::new(Circuit::E20);
        bench.act(Action::Run);
        advance_steps(&bench.project, &mut bench.simulation, 1);
        let before =
            bench.simulation.last_valid.as_ref().unwrap().led_currents[&ComponentId("D1".into())];
        assert!(before < 0.001, "LED should start off: {before}");

        advance_steps(&bench.project, &mut bench.simulation, 20_000);
        let after =
            bench.simulation.last_valid.as_ref().unwrap().led_currents[&ComponentId("D1".into())];
        assert!(after > 0.001, "LED should turn on after charging: {after}");
        assert_eq!(bench.simulation.step, 20_001);
    }

    #[test]
    fn e21_potentiometer_shifts_the_ambient_light_threshold_monotonically() {
        let threshold = |pot_ratio: f64| {
            let mut first_on = None;
            for (index, light_ratio) in (0..=20).map(|i| i as f64 / 20.0).enumerate() {
                let mut bench = Bench::new(Circuit::E21);
                bench.act(Action::SetControlRatio {
                    component: ComponentId("RV1".into()),
                    ratio: pot_ratio,
                });
                bench.act(Action::SetControlRatio {
                    component: ComponentId("R2".into()),
                    ratio: light_ratio,
                });
                bench.act(Action::Run);
                advance_steps(&bench.project, &mut bench.simulation, 2);
                let current = bench.simulation.last_valid.as_ref().unwrap().led_currents
                    [&ComponentId("D1".into())];
                if current > 0.001 {
                    first_on = Some(index);
                    break;
                }
            }
            first_on.expect("each E21 potentiometer setting should have a threshold")
        };
        let thresholds = [0.0, 0.5, 1.0].map(threshold);
        assert!(thresholds[0] >= thresholds[1] && thresholds[1] >= thresholds[2]);
    }

    #[test]
    fn e22_mixed_logic_matches_all_eight_button_combinations() {
        for mask in 0..8 {
            let mut bench = Bench::new(Circuit::E22);
            for (index, id) in ["S1", "S2", "S3"].into_iter().enumerate() {
                if mask & (1 << index) != 0 {
                    bench.toggle(index);
                }
                assert!(
                    bench
                        .simulation
                        .controls
                        .contains_key(&ComponentId(id.into()))
                );
            }
            bench.act(Action::Run);
            advance_steps(&bench.project, &mut bench.simulation, 2);
            assert!(
                bench.simulation.last_valid.is_some(),
                "mask={mask:03b}: {:?}",
                bench.simulation.diagnostics
            );
            let current = bench.simulation.last_valid.as_ref().unwrap().led_currents
                [&ComponentId("D1".into())];
            let s1 = mask & 1 != 0;
            let s2 = mask & 2 != 0;
            let s3 = mask & 4 != 0;
            assert_eq!(current > 0.001, (s1 && s2) || s3, "mask={mask:03b}");
        }
    }

    #[test]
    fn e23_led_and_buzzer_switch_together_with_ambient_light() {
        let mut dark = Bench::new(Circuit::E23);
        dark.act(Action::SetControlRatio {
            component: ComponentId("R3".into()),
            ratio: 0.0,
        });
        dark.act(Action::Run);
        advance_steps(&dark.project, &mut dark.simulation, 2);
        let dark_result = dark.simulation.last_valid.as_ref().unwrap();
        assert!(dark_result.led_currents[&ComponentId("D1".into())] > 0.001);
        assert!(dark_result.resistor_currents[&ComponentId("BZ1".into())] >= 0.001);

        let mut bright = Bench::new(Circuit::E23);
        bright.act(Action::SetControlRatio {
            component: ComponentId("R3".into()),
            ratio: 1.0,
        });
        bright.act(Action::Run);
        advance_steps(&bright.project, &mut bright.simulation, 2);
        let bright_result = bright.simulation.last_valid.as_ref().unwrap();
        assert!(bright_result.led_currents[&ComponentId("D1".into())] < 0.000_5);
        assert!(bright_result.resistor_currents[&ComponentId("BZ1".into())] < 0.001);
    }

    #[test]
    fn e24_switch_paths_have_distinct_charge_current_traces() {
        let mut fast = Bench::new(Circuit::E24);
        fast.act(Action::Run);
        advance_steps(&fast.project, &mut fast.simulation, 1);
        let fast_initial =
            fast.simulation.last_valid.as_ref().unwrap().led_currents[&ComponentId("D1".into())];
        advance_steps(&fast.project, &mut fast.simulation, 100);
        let fast_later =
            fast.simulation.last_valid.as_ref().unwrap().led_currents[&ComponentId("D1".into())];

        let mut slow = Bench::new(Circuit::E24);
        slow.toggle(0);
        slow.act(Action::Run);
        advance_steps(&slow.project, &mut slow.simulation, 1);
        let slow_initial =
            slow.simulation.last_valid.as_ref().unwrap().led_currents[&ComponentId("D2".into())];
        advance_steps(&slow.project, &mut slow.simulation, 100);
        let slow_later =
            slow.simulation.last_valid.as_ref().unwrap().led_currents[&ComponentId("D2".into())];

        assert!(
            fast_initial > slow_initial,
            "fast={fast_initial}, slow={slow_initial}"
        );
        assert!(fast_later < fast_initial && slow_later < slow_initial);
        assert!(
            fast_later / fast_initial < slow_later / slow_initial,
            "fast_initial={fast_initial}, fast={fast_later}, slow_initial={slow_initial}, slow={slow_later}"
        );
    }

    #[test]
    fn e25_transistor_and_lights_only_when_both_buttons_are_held() {
        for mask in 0..4 {
            let mut bench = Bench::new(Circuit::E25);
            if mask & 1 != 0 {
                bench.toggle(0);
            }
            if mask & 2 != 0 {
                bench.toggle(1);
            }
            bench.act(Action::Run);
            advance_steps(&bench.project, &mut bench.simulation, 4);
            let current = bench.simulation.last_valid.as_ref().unwrap().led_currents
                [&ComponentId("D1".into())];
            assert_eq!(
                current > 0.001,
                mask == 3,
                "mask={mask:02b}, current={current}"
            );
        }
    }

    #[test]
    fn e21_renders_and_updates_two_independent_dials() {
        let mut app = App::new();
        app.insert_resource(Session {
            bench: Some(Bench::new(Circuit::E21)),
        })
        .insert_resource(ButtonInput::<MouseButton>::default())
        .init_resource::<Assets<Image>>()
        .add_systems(Update, (handle_dial, update_dial_handle).chain());
        let window = app
            .world_mut()
            .spawn(Window {
                resolution: (1200, 760).into(),
                ..default()
            })
            .id();
        app.world_mut()
            .resource_scope(|world, mut images: Mut<Assets<Image>>| {
                let session = world.resource::<Session>();
                let bench = session.bench.as_ref().unwrap();
                let mut queue = bevy::ecs::world::CommandQueue::default();
                let mut commands = Commands::new(&mut queue, world);
                spawn_bench(&mut commands, &mut images, bench);
                queue.apply(world);
            });
        let mut tracks = app.world_mut().query::<&DialTrack>();
        assert_eq!(tracks.iter(app.world()).count(), 2);
        let set_ratio = |app: &mut App, point: Vec2| {
            let mut window = app.world_mut().get_mut::<Window>(window).unwrap();
            let width = window.width();
            let height = window.height();
            let world_width = 1200.0_f32.max(760.0 * width / height);
            let world_height = 760.0_f32.max(1200.0 * height / width);
            window.set_cursor_position(Some(Vec2::new(
                width * (0.5 + point.x / world_width),
                height * (0.5 - point.y / world_height),
            )));
            let mut mouse = ButtonInput::<MouseButton>::default();
            mouse.press(MouseButton::Left);
            app.world_mut().insert_resource(mouse);
            app.update();
        };
        set_ratio(&mut app, Vec2::new(20.0, DIAL_TRACK_CENTER.y));
        set_ratio(&mut app, Vec2::new(390.0, DIAL_TRACK_CENTER.y));
        let bench = app.world().resource::<Session>().bench.as_ref().unwrap();
        assert!(
            bench.simulation.control_ratios[&ComponentId("R2".into())] < 0.1,
            "ratios={:?}",
            bench.simulation.control_ratios
        );
        assert!(
            bench.simulation.control_ratios[&ComponentId("RV1".into())] > 0.9,
            "ratios={:?}",
            bench.simulation.control_ratios
        );
    }

    #[test]
    fn e26_transistor_or_truth_table_has_four_combinations() {
        for mask in 0..4 {
            let mut bench = Bench::new(Circuit::E26);
            if mask & 1 != 0 {
                bench.toggle(0);
            }
            if mask & 2 != 0 {
                bench.toggle(1);
            }
            bench.act(Action::Run);
            advance_steps(&bench.project, &mut bench.simulation, 4);
            let current = bench.simulation.last_valid.as_ref().unwrap().led_currents
                [&ComponentId("D1".into())];
            assert_eq!(current > 0.001, mask != 0, "mask={mask:02b}");
        }
    }

    #[test]
    fn e27_shared_dial_keeps_brightness_equal_and_monotonic() {
        let mut bench = Bench::new(Circuit::E27);
        bench.act(Action::Run);
        let mut previous = f64::INFINITY;
        for ratio in [0.0, 0.25, 0.5, 0.75, 1.0] {
            bench.act(Action::SetControlRatio {
                component: ComponentId("RV1".into()),
                ratio,
            });
            advance_steps(&bench.project, &mut bench.simulation, 1);
            let result = bench.simulation.last_valid.as_ref().unwrap();
            let d1 = result.led_currents[&ComponentId("D1".into())];
            let d2 = result.led_currents[&ComponentId("D2".into())];
            assert!((d1 - d2).abs() < 1e-9, "ratio={ratio}, D1={d1}, D2={d2}");
            assert!(d1 <= previous + 1e-12, "ratio={ratio}, current={d1}");
            previous = d1;
        }
    }

    #[test]
    fn e28_selects_both_sources_with_one_shared_negative_node() {
        let mut bench = Bench::new(Circuit::E28);
        bench.act(Action::Run);
        advance_steps(&bench.project, &mut bench.simulation, 2);
        let five = bench.simulation.last_valid.as_ref().unwrap();
        let five_current = five.led_currents[&ComponentId("D1".into())];
        assert!(five_current > 0.005);
        bench.toggle(0);
        advance_steps(&bench.project, &mut bench.simulation, 2);
        let nine = bench.simulation.last_valid.as_ref().unwrap();
        let nine_current = nine.led_currents[&ComponentId("D1".into())];
        assert!(nine_current > 0.005);
        assert!((nine_current / five_current).abs() < 2.0);
        assert!(nine.source_currents[&ComponentId("B1".into())].abs() < 1e-9);
        assert!(nine.source_currents[&ComponentId("B2".into())].abs() > 0.005);
    }

    #[test]
    fn e29_darlington_buzzer_sounds_only_while_button_is_held() {
        let mut bench = Bench::new(Circuit::E29);
        bench.act(Action::Run);
        advance_steps(&bench.project, &mut bench.simulation, 2);
        let released = bench
            .simulation
            .last_valid
            .as_ref()
            .unwrap()
            .resistor_currents[&ComponentId("BZ1".into())];
        assert!(released < 0.001);
        bench.toggle(0);
        advance_steps(&bench.project, &mut bench.simulation, 2);
        let pressed = bench
            .simulation
            .last_valid
            .as_ref()
            .unwrap()
            .resistor_currents[&ComponentId("BZ1".into())];
        assert!(pressed >= 0.001, "pressed current={pressed}");
    }

    #[test]
    fn e30_asymmetric_initial_state_sustains_multiple_alternating_led_cycles() {
        let mut bench = Bench::new(Circuit::E30);
        assert_eq!(
            bench.project.initial_conditions.capacitor_voltages[&ComponentId("C2".into())],
            0.5
        );
        bench.act(Action::Run);
        let mut d1_states = Vec::new();
        let mut d2_states = Vec::new();
        for _ in 0..60_000 {
            advance_steps(&bench.project, &mut bench.simulation, 1);
            assert!(
                bench.simulation.last_valid.is_some(),
                "E30 stopped: {:?}",
                bench.simulation.diagnostics
            );
            let result = bench.simulation.last_valid.as_ref().unwrap();
            d1_states.push(result.led_currents[&ComponentId("D1".into())] > 0.001);
            d2_states.push(result.led_currents[&ComponentId("D2".into())] > 0.001);
        }
        let transitions =
            |states: &[bool]| states.windows(2).filter(|pair| pair[0] != pair[1]).count();
        assert!(
            transitions(&d1_states) >= 4,
            "D1 transitions: {}, on samples: {}, first/last: {:?}/{:?}",
            transitions(&d1_states),
            d1_states.iter().filter(|state| **state).count(),
            d1_states.first(),
            d1_states.last()
        );
        assert!(
            transitions(&d2_states) >= 4,
            "D2 transitions: {}, on samples: {}, first/last: {:?}/{:?}",
            transitions(&d2_states),
            d2_states.iter().filter(|state| **state).count(),
            d2_states.first(),
            d2_states.last()
        );
    }
}
