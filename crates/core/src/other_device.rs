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
    /// A bounded Thevenin source. A non-zero internal resistance keeps source
    /// conflicts diagnosable without introducing an unbounded ideal source.
    VoltageSource {
        positive: PinId,
        negative: PinId,
        voltage: f64,
        internal_resistance: f64,
    },
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
            OtherDeviceBehavior::Resistive { .. }
            | OtherDeviceBehavior::VoltageSource { .. }
            | OtherDeviceBehavior::VoltageControlledResistance { .. } => Vec::new(),
        }
    }

    pub fn referenced_pins(&self) -> Vec<&PinId> {
        match &self.behavior {
            OtherDeviceBehavior::Resistive {
                positive, negative, ..
            }
            | OtherDeviceBehavior::VoltageSource {
                positive, negative, ..
            } => vec![positive, negative],
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
    use super::controlled_resistance;
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
    }
}
