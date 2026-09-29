//! Shared electrical contract for catalog ICs and small modules.
//!
//! The contract deliberately describes a device in terms of named pins and a
//! calculated transfer law.  It is not a fixture result table: the solver
//! evaluates the law from the voltages on the compiled board nodes on every
//! fixed step.

use crate::PinId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const MAX_IC_DEVICE_PINS: usize = 32;
pub const MAX_IC_DEVICE_INPUTS: usize = 8;
pub const IC_DEVICE_MIN_RESISTANCE: f64 = 1.0;
pub const IC_DEVICE_MAX_RESISTANCE: f64 = 1e9;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IcDevicePinRole {
    Supply,
    Ground,
    Input,
    Output,
    Reference,
    Clock,
    Reset,
    Bidirectional,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct IcDeviceSpec {
    /// Pin roles are part of the device contract; physical hole placement
    /// remains in `Component::pins` and is the only connectivity authority.
    pub pin_roles: BTreeMap<PinId, IcDevicePinRole>,
    pub behavior: IcDeviceBehavior,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "behavior", rename_all = "snake_case")]
pub enum IcDeviceBehavior {
    /// A bounded linear transfer with finite input impedance. `offset` and
    /// `min_output`/`max_output` are measured from the reference pin. This
    /// covers educational op-amp, regulator, DAC, and instrumentation blocks
    /// without pretending to model a particular silicon part.
    Linear {
        output: PinId,
        reference: PinId,
        inputs: Vec<IcDeviceLinearInput>,
        offset: f64,
        min_output: f64,
        max_output: f64,
        input_resistance: f64,
        output_resistance: f64,
    },
    /// A voltage comparator with a finite output resistance. The output is
    /// high when `positive - negative` reaches `threshold`.
    Comparator {
        positive: PinId,
        negative: PinId,
        output: PinId,
        reference: PinId,
        threshold: f64,
        high_output: f64,
        low_output: f64,
        input_resistance: f64,
        output_resistance: f64,
    },
    /// A two-input or multi-input digital function. Inputs are sampled from
    /// the calculated node voltages using the supply midpoint as the logic
    /// threshold; the output is still a finite-resistance electrical source.
    Logic {
        inputs: Vec<PinId>,
        output: PinId,
        reference: PinId,
        supply: PinId,
        operation: IcLogicOperation,
        output_resistance: f64,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct IcDeviceLinearInput {
    pub pin: PinId,
    pub gain: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IcLogicOperation {
    And,
    Or,
    Nand,
    Nor,
    Xor,
    Xnor,
    Not,
}

impl IcDeviceBehavior {
    pub fn output_pins(&self) -> Vec<&PinId> {
        match self {
            Self::Linear { output, .. }
            | Self::Comparator { output, .. }
            | Self::Logic { output, .. } => vec![output],
        }
    }
}

impl IcDeviceSpec {
    pub fn output_pins(&self) -> Vec<&PinId> {
        self.behavior.output_pins()
    }

    pub fn referenced_pins(&self) -> Vec<&PinId> {
        match &self.behavior {
            IcDeviceBehavior::Linear {
                output,
                reference,
                inputs,
                ..
            } => {
                let mut pins = vec![output, reference];
                pins.extend(inputs.iter().map(|input| &input.pin));
                pins
            }
            IcDeviceBehavior::Comparator {
                positive,
                negative,
                output,
                reference,
                ..
            } => vec![positive, negative, output, reference],
            IcDeviceBehavior::Logic {
                inputs,
                output,
                reference,
                supply,
                ..
            } => {
                let mut pins = inputs.iter().collect::<Vec<_>>();
                pins.extend([output, reference, supply]);
                pins
            }
        }
    }
}
