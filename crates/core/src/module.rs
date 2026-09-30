//! Shared pin-level contract for ready-made catalog modules.
//!
//! A module is still an electrical component, not a scripted lesson result.
//! Its named pins are mapped to board holes by `Component::pins`, and the
//! solver evaluates the selected bounded behavior from the voltages on those
//! compiled nodes on every solve.

use crate::PinId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const MAX_MODULE_PINS: usize = 32;
pub const MAX_MODULE_INPUTS: usize = 8;
pub const MAX_MODULE_CHANNELS: usize = 8;
pub const MODULE_MIN_RESISTANCE: f64 = 1e-3;
pub const MODULE_MAX_RESISTANCE: f64 = 1e12;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ModulePinRole {
    Supply,
    Ground,
    Input,
    Output,
    Reference,
    PowerOutput,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ModuleSpec {
    /// Pin roles describe the reusable module contract. Physical placement
    /// and connectivity remain exclusively in `Component::pins` and wires.
    pub pin_roles: BTreeMap<PinId, ModulePinRole>,
    pub behavior: ModuleBehavior,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "behavior", rename_all = "snake_case")]
pub enum ModuleBehavior {
    /// A finite-input-impedance, bounded analog transfer. This is suitable
    /// for regulator/sensor/amplifier-style modules whose contract is a
    /// voltage relationship rather than an internal firmware implementation.
    AnalogTransfer {
        output: PinId,
        reference: PinId,
        inputs: Vec<ModuleInput>,
        offset: f64,
        min_output: f64,
        max_output: f64,
        input_resistance: f64,
        output_resistance: f64,
    },
    /// A finite-resistance digital or thresholded sensor output. The input
    /// is sampled from the solved voltage; the output is a bounded electrical
    /// source relative to the declared reference pin.
    ThresholdOutput {
        input: PinId,
        reference: PinId,
        output: PinId,
        threshold: f64,
        high_output: f64,
        low_output: f64,
        input_resistance: f64,
        output_resistance: f64,
    },
    /// One or more open-collector channels. A high input turns its output on
    /// toward ground; a high off resistance preserves a finite, diagnosable
    /// circuit when a channel is idle.
    OpenCollector {
        supply: PinId,
        ground: PinId,
        channels: Vec<ModuleChannel>,
        input_resistance: f64,
        on_resistance: f64,
        off_resistance: f64,
    },
    /// A bounded low-voltage regulator contract. The output target follows
    /// the input voltage after dropout and is limited by the declared target;
    /// finite input/output resistance keeps source conflicts diagnosable.
    RegulatedSupply {
        input_positive: PinId,
        input_negative: PinId,
        output_positive: PinId,
        output_negative: PinId,
        target_voltage: f64,
        dropout_voltage: f64,
        input_resistance: f64,
        output_resistance: f64,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ModuleInput {
    pub pin: PinId,
    pub gain: f64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ModuleChannel {
    pub input: PinId,
    pub output: PinId,
}

impl ModuleBehavior {
    pub fn output_pins(&self) -> Vec<&PinId> {
        match self {
            Self::AnalogTransfer { output, .. } | Self::ThresholdOutput { output, .. } => {
                vec![output]
            }
            Self::OpenCollector { channels, .. } => {
                channels.iter().map(|channel| &channel.output).collect()
            }
            Self::RegulatedSupply {
                output_positive, ..
            } => vec![output_positive],
        }
    }

    pub fn referenced_pins(&self) -> Vec<&PinId> {
        match self {
            Self::AnalogTransfer {
                output,
                reference,
                inputs,
                ..
            } => {
                let mut pins = vec![output, reference];
                pins.extend(inputs.iter().map(|input| &input.pin));
                pins
            }
            Self::ThresholdOutput {
                input,
                reference,
                output,
                ..
            } => vec![input, reference, output],
            Self::OpenCollector {
                supply,
                ground,
                channels,
                ..
            } => {
                let mut pins = vec![supply, ground];
                for channel in channels {
                    pins.extend([&channel.input, &channel.output]);
                }
                pins
            }
            Self::RegulatedSupply {
                input_positive,
                input_negative,
                output_positive,
                output_negative,
                ..
            } => vec![
                input_positive,
                input_negative,
                output_positive,
                output_negative,
            ],
        }
    }
}

impl ModuleSpec {
    pub fn output_pins(&self) -> Vec<&PinId> {
        self.behavior.output_pins()
    }

    pub fn referenced_pins(&self) -> Vec<&PinId> {
        self.behavior.referenced_pins()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 128,
            rng_seed: proptest::test_runner::RngSeed::Fixed(0x4D4F_DA1E),
            ..ProptestConfig::default()
        })]
        #[test]
        fn regulated_target_is_bounded(
            target in 0.1f64..12.0,
            dropout in 0.0f64..2.0,
            input in 0.0f64..14.0,
        ) {
            let available = (input - dropout).max(0.0);
            let output = available.min(target);
            prop_assert!(output.is_finite());
            prop_assert!((0.0..=target).contains(&output));
        }

        #[test]
        fn channel_contract_preserves_one_output_per_channel(count in 1usize..=MAX_MODULE_CHANNELS) {
            let channels = (0..count).map(|index| ModuleChannel {
                input: PinId(format!("in{index}")),
                output: PinId(format!("out{index}")),
            }).collect::<Vec<_>>();
            let behavior = ModuleBehavior::OpenCollector {
                supply: PinId("vcc".into()),
                ground: PinId("gnd".into()),
                channels,
                input_resistance: 100_000.0,
                on_resistance: 20.0,
                off_resistance: 1e9,
            };
            prop_assert_eq!(behavior.output_pins().len(), count);
            prop_assert_eq!(behavior.referenced_pins().len(), 2 * count + 2);
        }
    }
}
