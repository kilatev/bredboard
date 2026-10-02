//! Pin-level contract for catalog parts that are not yet specific enough to
//! warrant their own component kind.
//!
//! An `OtherDeviceSpec` is deliberately an electrical contract, not a name for
//! an unmodelled black box. Its named pins are connected through the normal
//! component pin map and every output is calculated from terminal voltages.
//! Fixture authors can later promote a stable behavior to a dedicated model
//! without changing board topology.

use crate::PinId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const MAX_OTHER_DEVICE_PINS: usize = 32;
pub const OTHER_DEVICE_MIN_RESISTANCE: f64 = 1e-3;
pub const OTHER_DEVICE_MAX_RESISTANCE: f64 = 1e12;
pub const COUPLED_WINDING_MIN_INDUCTANCE: f64 = 1e-9;
pub const COUPLED_WINDING_MAX_INDUCTANCE: f64 = 1.0;
pub const COUPLED_WINDING_MAX_ENERGY: f64 = 1.0;

/// Map a control voltage to the bounded resistance used by
/// `VoltageControlledResistance`. Values outside the declared control range
/// saturate rather than creating an invalid or negative conductance.
pub fn controlled_resistance(
    min_resistance: f64,
    max_resistance: f64,
    control_min: f64,
    control_max: f64,
    control_voltage: f64,
) -> f64 {
    let ratio = ((control_voltage - control_min) / (control_max - control_min)).clamp(0.0, 1.0);
    max_resistance - ratio * (max_resistance - min_resistance)
}

/// Calculate the bounded voltage produced by a ring-modulator contract.
/// Inputs are normalized around the declared reference voltage; this helper
/// keeps the solver and property tests on the same deterministic contract.
#[allow(clippy::too_many_arguments)]
pub fn ring_modulator_output(
    reference_voltage: f64,
    signal_voltage: f64,
    carrier_voltage: f64,
    gain: f64,
    signal_scale: f64,
    carrier_scale: f64,
    min_output: f64,
    max_output: f64,
) -> f64 {
    let raw = gain * (signal_voltage - reference_voltage) * (carrier_voltage - reference_voltage)
        / (signal_scale * carrier_scale);
    reference_voltage + raw.clamp(min_output, max_output)
}

/// Select the finite resistance of a reed contact from its ordered control
/// state. Keeping this helper pure makes the open/closed invariant testable
/// independently of the board solver.
pub fn reed_resistance(closed_resistance: f64, open_resistance: f64, closed: bool) -> f64 {
    if closed {
        closed_resistance
    } else {
        open_resistance
    }
}

/// Return the magnetic energy represented by two coupled winding currents.
pub fn coupled_winding_energy(
    primary_inductance: f64,
    secondary_inductance: f64,
    coupling: f64,
    primary_current: f64,
    secondary_current: f64,
) -> f64 {
    let mutual = coupling * (primary_inductance * secondary_inductance).sqrt();
    (0.5 * (primary_inductance * primary_current * primary_current
        + secondary_inductance * secondary_current * secondary_current
        + 2.0 * mutual * primary_current * secondary_current))
        .max(0.0)
}

/// Keep persisted winding currents within the declared energy envelope.
pub fn bound_coupled_winding_currents(
    primary_inductance: f64,
    secondary_inductance: f64,
    coupling: f64,
    max_energy: f64,
    currents: [f64; 2],
) -> [f64; 2] {
    let energy = coupled_winding_energy(
        primary_inductance,
        secondary_inductance,
        coupling,
        currents[0],
        currents[1],
    );
    if energy <= max_energy || energy == 0.0 {
        currents
    } else {
        let scale = (max_energy / energy).sqrt();
        [currents[0] * scale, currents[1] * scale]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OtherDevicePinRole {
    Terminal,
    Input,
    Output,
    Control,
    Reference,
    Supply,
    Ground,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BjtPolarity {
    Npn,
    Pnp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BjtTestState {
    Working,
    Open,
    Shorted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DiodeSubjectKind {
    Led,
    Diode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DiodePolarity {
    Forward,
    Reverse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DiodeTestState {
    Working,
    Open,
    Shorted,
}

/// The calculated electrical contract for a three-pin transistor test socket.
/// The fixture selects the subject polarity and test state explicitly; the
/// solver still derives LED results from the BJT equations and board topology.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BjtTestSocket {
    pub base: PinId,
    pub collector: PinId,
    pub emitter: PinId,
    pub socket_polarity: BjtPolarity,
    pub subject_polarity: BjtPolarity,
    pub subject_state: BjtTestState,
    pub beta: f64,
    pub saturation_current: f64,
    pub open_resistance: f64,
    pub short_resistance: f64,
}

/// The calculated electrical contract for a two-terminal LED/diode test
/// socket. The fixture selects the subject family, polarity, and fault state;
/// the solver still derives current from the diode equation and board nodes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DiodeTestSocket {
    pub anode: PinId,
    pub cathode: PinId,
    pub subject_kind: DiodeSubjectKind,
    pub subject_polarity: DiodePolarity,
    pub subject_state: DiodeTestState,
    pub forward_voltage: f64,
    pub series_resistance: f64,
    pub open_resistance: f64,
    pub short_resistance: f64,
}

impl BjtTestSocket {
    /// A subject with the wrong polarity does not conduct in this socket.
    pub fn effective_state(&self) -> BjtTestState {
        if self.socket_polarity != self.subject_polarity {
            BjtTestState::Open
        } else {
            self.subject_state
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct OtherDeviceSpec {
    /// Pin roles describe the part contract. Physical hole placement remains
    /// in `Component::pins` and is the only connectivity authority.
    pub pin_roles: BTreeMap<PinId, OtherDevicePinRole>,
    pub behavior: OtherDeviceBehavior,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "behavior", rename_all = "snake_case")]
pub enum OtherDeviceBehavior {
    /// A calculated passive branch. This covers a connector, coil, load, or
    /// other two-terminal part when its usable educational contract is an
    /// impedance rather than an internal controller.
    Resistive {
        positive: PinId,
        negative: PinId,
        resistance: f64,
    },
    /// A magnetically actuated two-terminal contact. The simulation exposes
    /// its open/closed state through the same ordered control actions as a
    /// momentary button; the magnet and mechanical hysteresis remain outside
    /// the electrical contract.
    ReedSwitch {
        a: PinId,
        b: PinId,
        closed_resistance: f64,
        open_resistance: f64,
    },
    /// A five-wire unipolar stepper load. The common pin feeds four equal
    /// phase coils; each phase current is calculated from the solved terminal
    /// voltage. This is an electrical load contract, not a mechanical
    /// inertia or shaft-position simulation.
    StepperLoad {
        common: PinId,
        phases: [PinId; 4],
        coil_resistance: f64,
        rated_voltage: f64,
        steps_per_revolution: f64,
    },
    /// A bounded Thevenin source. A non-zero internal resistance keeps source
    /// conflicts diagnosable without introducing an unbounded ideal source.
    VoltageSource {
        positive: PinId,
        negative: PinId,
        voltage: f64,
        internal_resistance: f64,
    },
    /// A three-pin BJT test socket. It reuses the educational NPN/PNP model
    /// with an explicit fixture-selected subject polarity and failure state.
    BjtTestSocket { socket: BjtTestSocket },
    /// A two-terminal LED/diode test socket. It reuses the shared diode model
    /// with an explicit fixture-selected subject polarity and failure state.
    DiodeTestSocket { socket: DiodeTestSocket },
    /// A finite-resistance transfer from one or more sensed pins to an output
    /// relative to a reference pin. It is suitable for an explicitly
    /// parameterized sensor, regulator, amplifier, or module interface.
    LinearTransfer {
        output: PinId,
        reference: PinId,
        inputs: Vec<OtherDeviceLinearInput>,
        offset: f64,
        min_output: f64,
        max_output: f64,
        input_resistance: f64,
        output_resistance: f64,
    },
    /// A bounded four-terminal transformer approximation. The primary draws
    /// a finite resistive input current and the secondary is driven by the
    /// calculated primary voltage through the declared turns ratio. This is
    /// an educational transfer contract, not a magnetic transient model.
    Transformer {
        primary_positive: PinId,
        primary_negative: PinId,
        secondary_positive: PinId,
        secondary_negative: PinId,
        turns_ratio: f64,
        primary_resistance: f64,
        secondary_resistance: f64,
    },
    /// A two-winding backward-Euler magnetic contract. Unlike `Transformer`,
    /// this preserves bounded winding current and magnetic stored energy
    /// between fixed simulation steps.
    CoupledWinding {
        primary_positive: PinId,
        primary_negative: PinId,
        secondary_positive: PinId,
        secondary_negative: PinId,
        turns_ratio: f64,
        primary_inductance: f64,
        coupling: f64,
        primary_resistance: f64,
        secondary_resistance: f64,
        boost_voltage_limit: f64,
        max_stored_energy_joules: f64,
    },
    /// A bounded multiplicative signal transfer used by the four-diode ring
    /// modulator fixture. The carrier and signal are normalized around the
    /// reference pin; finite input/output resistances keep the transfer part
    /// of the common electrical solve.
    RingModulator {
        signal: PinId,
        carrier: PinId,
        output: PinId,
        reference: PinId,
        gain: f64,
        signal_scale: f64,
        carrier_scale: f64,
        min_output: f64,
        max_output: f64,
        input_resistance: f64,
        output_resistance: f64,
    },
    /// A two-terminal output whose resistance is controlled by a sensed
    /// voltage. The control terminals are electrically isolated from the
    /// output terminals in this contract, as required for optical or sensor
    /// couplers. Resistance interpolates from max to min as control voltage
    /// moves from `control_min` to `control_max`.
    VoltageControlledResistance {
        control_positive: PinId,
        control_negative: PinId,
        output_positive: PinId,
        output_negative: PinId,
        min_resistance: f64,
        max_resistance: f64,
        control_min: f64,
        control_max: f64,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct OtherDeviceLinearInput {
    pub pin: PinId,
    pub gain: f64,
}

impl OtherDeviceSpec {
    pub fn output_pins(&self) -> Vec<&PinId> {
        match &self.behavior {
            OtherDeviceBehavior::LinearTransfer { output, .. } => vec![output],
            OtherDeviceBehavior::Transformer {
                secondary_positive, ..
            } => vec![secondary_positive],
            OtherDeviceBehavior::CoupledWinding {
                secondary_positive, ..
            } => vec![secondary_positive],
            OtherDeviceBehavior::RingModulator { output, .. } => vec![output],
            OtherDeviceBehavior::Resistive { .. }
            | OtherDeviceBehavior::ReedSwitch { .. }
            | OtherDeviceBehavior::StepperLoad { .. }
            | OtherDeviceBehavior::VoltageSource { .. }
            | OtherDeviceBehavior::BjtTestSocket { .. }
            | OtherDeviceBehavior::DiodeTestSocket { .. }
            | OtherDeviceBehavior::VoltageControlledResistance { .. } => Vec::new(),
        }
    }

    pub fn referenced_pins(&self) -> Vec<&PinId> {
        match &self.behavior {
            OtherDeviceBehavior::Resistive {
                positive, negative, ..
            }
            | OtherDeviceBehavior::ReedSwitch {
                a: positive,
                b: negative,
                ..
            }
            | OtherDeviceBehavior::VoltageSource {
                positive, negative, ..
            } => vec![positive, negative],
            OtherDeviceBehavior::StepperLoad { common, phases, .. } => {
                let mut pins = vec![common];
                pins.extend(phases.iter());
                pins
            }
            OtherDeviceBehavior::BjtTestSocket { socket } => {
                vec![&socket.base, &socket.collector, &socket.emitter]
            }
            OtherDeviceBehavior::DiodeTestSocket { socket } => {
                vec![&socket.anode, &socket.cathode]
            }
            OtherDeviceBehavior::LinearTransfer {
                output,
                reference,
                inputs,
                ..
            } => {
                let mut pins = vec![output, reference];
                pins.extend(inputs.iter().map(|input| &input.pin));
                pins
            }
            OtherDeviceBehavior::Transformer {
                primary_positive,
                primary_negative,
                secondary_positive,
                secondary_negative,
                ..
            } => vec![
                primary_positive,
                primary_negative,
                secondary_positive,
                secondary_negative,
            ],
            OtherDeviceBehavior::CoupledWinding {
                primary_positive,
                primary_negative,
                secondary_positive,
                secondary_negative,
                ..
            } => vec![
                primary_positive,
                primary_negative,
                secondary_positive,
                secondary_negative,
            ],
            OtherDeviceBehavior::RingModulator {
                signal,
                carrier,
                output,
                reference,
                ..
            } => vec![signal, carrier, output, reference],
            OtherDeviceBehavior::VoltageControlledResistance {
                control_positive,
                control_negative,
                output_positive,
                output_negative,
                ..
            } => vec![
                control_positive,
                control_negative,
                output_positive,
                output_negative,
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BjtPolarity, BjtTestSocket, BjtTestState, controlled_resistance, reed_resistance,
        ring_modulator_output,
    };
    use crate::PinId;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 96,
            rng_seed: proptest::test_runner::RngSeed::Fixed(0x0A7E_2026),
            ..ProptestConfig::default()
        })]
        #[test]
        fn controlled_resistance_is_bounded_and_monotone(
            min in 1u32..10_000,
            span in 1u32..100_000,
            control_min in -1_000i32..0,
            control_span in 1u32..2_000,
            first_offset in 0u32..2_000,
            second_offset in 0u32..2_000,
        ) {
            let min = f64::from(min);
            let max = min + f64::from(span);
            let control_min = f64::from(control_min);
            let control_max = control_min + f64::from(control_span);
            let first = control_min + f64::from(first_offset);
            let second = control_min + f64::from(first_offset.max(second_offset));
            let first_resistance = controlled_resistance(min, max, control_min, control_max, first);
            let second_resistance = controlled_resistance(min, max, control_min, control_max, second);
            prop_assert!((min..=max).contains(&first_resistance));
            prop_assert!((min..=max).contains(&second_resistance));
            prop_assert!(first_resistance + f64::EPSILON >= second_resistance);
        }

        #[test]
        fn ring_modulator_output_is_bounded_and_reference_centered(
            reference in -10_000i32..10_000,
            signal in -10_000i32..10_000,
            carrier in -10_000i32..10_000,
            gain in -10_000i32..10_000,
            signal_scale in 1u32..10_000,
            carrier_scale in 1u32..10_000,
            minimum in -10_000i32..0,
            span in 1u32..10_000,
        ) {
            let reference = f64::from(reference);
            let signal = f64::from(signal);
            let carrier = f64::from(carrier);
            let gain = f64::from(gain) / 1_000.0;
            let signal_scale = f64::from(signal_scale) / 100.0;
            let carrier_scale = f64::from(carrier_scale) / 100.0;
            let minimum = f64::from(minimum) / 100.0;
            let maximum = minimum + f64::from(span) / 100.0;
            let output = ring_modulator_output(
                reference,
                signal,
                carrier,
                gain,
                signal_scale,
                carrier_scale,
                minimum,
                maximum,
            );
            prop_assert!((reference + minimum..=reference + maximum).contains(&output));
            let centered = ring_modulator_output(
                reference,
                reference,
                carrier,
                gain,
                signal_scale,
                carrier_scale,
                minimum,
                maximum,
            );
            if minimum <= 0.0 && maximum >= 0.0 {
                prop_assert_eq!(centered, reference);
            }
        }

        #[test]
        fn bjt_socket_mapping_rejects_mismatched_subjects(
            socket_is_npn in any::<bool>(),
            subject_is_npn in any::<bool>(),
            subject_is_shorted in any::<bool>(),
        ) {
            let socket_polarity = if socket_is_npn { BjtPolarity::Npn } else { BjtPolarity::Pnp };
            let subject_polarity = if subject_is_npn { BjtPolarity::Npn } else { BjtPolarity::Pnp };
            let subject_state = if subject_is_shorted {
                BjtTestState::Shorted
            } else {
                BjtTestState::Working
            };
            let socket = BjtTestSocket {
                base: PinId("base".into()),
                collector: PinId("collector".into()),
                emitter: PinId("emitter".into()),
                socket_polarity,
                subject_polarity,
                subject_state,
                beta: 100.0,
                saturation_current: 1e-15,
                open_resistance: 1e9,
                short_resistance: 1.0,
            };
            let expected = if socket_is_npn == subject_is_npn {
                subject_state
            } else {
                BjtTestState::Open
            };
            prop_assert_eq!(socket.effective_state(), expected);
        }

        #[test]
        fn reed_resistance_selects_a_finite_bounded_contact(
            closed in 0.01f64..1000.0,
            gap in 0.01f64..1_000_000.0,
            is_closed in any::<bool>(),
        ) {
            let open = closed + gap;
            let resistance = reed_resistance(closed, open, is_closed);
            prop_assert!(resistance.is_finite());
            prop_assert!(resistance >= closed);
            prop_assert!(resistance <= open);
            prop_assert_eq!(resistance, if is_closed { closed } else { open });
        }
    }
}
