//! Calculated two-terminal diode behavior shared by the project contract and solver.
//!
//! The model is intentionally small and deterministic.  A forward junction is
//! represented by a smooth rectifying characteristic with a finite series
//! slope.  A zener adds a second smooth branch in reverse bias.  Schottky
//! diodes use the same law with a lower declared forward voltage; their
//! distinction is part metadata, not a second electrical implementation.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const DIODE_MIN_FORWARD_VOLTAGE: f64 = 0.0;
pub const DIODE_MAX_FORWARD_VOLTAGE: f64 = 10.0;
pub const DIODE_MIN_SERIES_RESISTANCE: f64 = 1.0;
pub const DIODE_MAX_SERIES_RESISTANCE: f64 = 1e7;
pub const DIODE_MIN_BREAKDOWN_VOLTAGE: f64 = 0.1;
pub const DIODE_MAX_BREAKDOWN_VOLTAGE: f64 = 100.0;
pub const DIODE_MIN_BREAKDOWN_RESISTANCE: f64 = 1.0;
pub const DIODE_MAX_BREAKDOWN_RESISTANCE: f64 = 1e7;

const SMOOTHING_VOLTAGE: f64 = 0.05;
const REVERSE_LEAKAGE_CONDUCTANCE: f64 = 1e-9;

/// The physical family selects the additional reverse-bias contract.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DiodeKind {
    /// A general-purpose rectifier or signal diode with reverse leakage only.
    #[default]
    Standard,
    /// A forward diode with Schottky metadata; use a lower forward voltage.
    Schottky,
    /// A forward diode with a bounded, calculated reverse-breakdown branch.
    Zener,
}

/// Optional model metadata for `ComponentKind::Diode`.
///
/// Omitting this field preserves the original two-parameter diode JSON
/// contract and selects `Standard`.  Zeners must declare both reverse-bias
/// parameters; other diode kinds must omit them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DiodeSpec {
    #[serde(default)]
    pub kind: DiodeKind,
    #[serde(default)]
    pub breakdown_voltage: Option<f64>,
    #[serde(default)]
    pub breakdown_resistance: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DiodeModel {
    forward_voltage: f64,
    series_resistance: f64,
    kind: DiodeKind,
    breakdown_voltage: Option<f64>,
    breakdown_resistance: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DiodeLinearization {
    /// Current from anode to cathode at the operating voltage.
    pub current: f64,
    /// Positive dI/dV used to stamp the Newton linearization.
    pub conductance: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiodeModelError {
    NonFiniteForwardVoltage,
    ForwardVoltageOutOfRange,
    NonFiniteSeriesResistance,
    SeriesResistanceOutOfRange,
    MissingBreakdownVoltage,
    MissingBreakdownResistance,
    UnexpectedBreakdownParameters,
    NonFiniteBreakdownVoltage,
    BreakdownVoltageOutOfRange,
    NonFiniteBreakdownResistance,
    BreakdownResistanceOutOfRange,
}

impl DiodeModel {
    pub fn new(
        forward_voltage: f64,
        series_resistance: f64,
        spec: DiodeSpec,
    ) -> Result<Self, DiodeModelError> {
        if !forward_voltage.is_finite() {
            return Err(DiodeModelError::NonFiniteForwardVoltage);
        }
        if !(DIODE_MIN_FORWARD_VOLTAGE..=DIODE_MAX_FORWARD_VOLTAGE).contains(&forward_voltage) {
            return Err(DiodeModelError::ForwardVoltageOutOfRange);
        }
        if !series_resistance.is_finite() {
            return Err(DiodeModelError::NonFiniteSeriesResistance);
        }
        if !(DIODE_MIN_SERIES_RESISTANCE..=DIODE_MAX_SERIES_RESISTANCE).contains(&series_resistance)
        {
            return Err(DiodeModelError::SeriesResistanceOutOfRange);
        }

        match spec.kind {
            DiodeKind::Zener => {
                let breakdown_voltage = spec
                    .breakdown_voltage
                    .ok_or(DiodeModelError::MissingBreakdownVoltage)?;
                let breakdown_resistance = spec
                    .breakdown_resistance
                    .ok_or(DiodeModelError::MissingBreakdownResistance)?;
                if !breakdown_voltage.is_finite() {
                    return Err(DiodeModelError::NonFiniteBreakdownVoltage);
                }
                if !(DIODE_MIN_BREAKDOWN_VOLTAGE..=DIODE_MAX_BREAKDOWN_VOLTAGE)
                    .contains(&breakdown_voltage)
                {
                    return Err(DiodeModelError::BreakdownVoltageOutOfRange);
                }
                if !breakdown_resistance.is_finite() {
                    return Err(DiodeModelError::NonFiniteBreakdownResistance);
                }
                if !(DIODE_MIN_BREAKDOWN_RESISTANCE..=DIODE_MAX_BREAKDOWN_RESISTANCE)
                    .contains(&breakdown_resistance)
                {
                    return Err(DiodeModelError::BreakdownResistanceOutOfRange);
                }
            }
            DiodeKind::Standard | DiodeKind::Schottky => {
                if spec.breakdown_voltage.is_some() || spec.breakdown_resistance.is_some() {
                    return Err(DiodeModelError::UnexpectedBreakdownParameters);
                }
            }
        }

        Ok(Self {
            forward_voltage,
            series_resistance,
            kind: spec.kind,
            breakdown_voltage: spec.breakdown_voltage,
            breakdown_resistance: spec.breakdown_resistance,
        })
    }

    /// Evaluate the current and positive-slope conductance at a finite voltage.
    pub fn linearize(self, voltage: f64) -> DiodeLinearization {
        let voltage = if voltage.is_finite() { voltage } else { 0.0 };
        let forward = smooth_forward(voltage, self.forward_voltage, self.series_resistance);
        let reverse = match (self.kind, self.breakdown_voltage, self.breakdown_resistance) {
            (DiodeKind::Zener, Some(breakdown_voltage), Some(breakdown_resistance)) => {
                smooth_reverse_breakdown(voltage, breakdown_voltage, breakdown_resistance)
            }
            _ => DiodeLinearization {
                current: 0.0,
                conductance: 0.0,
            },
        };
        DiodeLinearization {
            current: forward.current + reverse.current,
            conductance: forward.conductance + reverse.conductance,
        }
    }

    pub fn current(self, voltage: f64) -> f64 {
        self.linearize(voltage).current
    }

    pub fn forward_voltage(self) -> f64 {
        self.forward_voltage
    }

    pub fn series_resistance(self) -> f64 {
        self.series_resistance
    }
}

fn smooth_forward(
    voltage: f64,
    forward_voltage: f64,
    series_resistance: f64,
) -> DiodeLinearization {
    let x = ((voltage - forward_voltage) / SMOOTHING_VOLTAGE).clamp(-80.0, 80.0);
    let softplus = if x > 30.0 { x } else { (1.0 + x.exp()).ln() };
    let sigmoid = sigmoid(x);
    DiodeLinearization {
        current: SMOOTHING_VOLTAGE * softplus / series_resistance
            + voltage * REVERSE_LEAKAGE_CONDUCTANCE,
        conductance: sigmoid / series_resistance + REVERSE_LEAKAGE_CONDUCTANCE,
    }
}

fn smooth_reverse_breakdown(
    voltage: f64,
    breakdown_voltage: f64,
    breakdown_resistance: f64,
) -> DiodeLinearization {
    let x = ((-voltage - breakdown_voltage) / SMOOTHING_VOLTAGE).clamp(-80.0, 80.0);
    let softplus = if x > 30.0 { x } else { (1.0 + x.exp()).ln() };
    DiodeLinearization {
        current: -SMOOTHING_VOLTAGE * softplus / breakdown_resistance,
        conductance: sigmoid(x) / breakdown_resistance,
    }
}

fn sigmoid(x: f64) -> f64 {
    if x >= 0.0 {
        1.0 / (1.0 + (-x).exp())
    } else {
        x.exp() / (1.0 + x.exp())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn standard() -> DiodeModel {
        DiodeModel::new(
            0.7,
            10.0,
            DiodeSpec {
                kind: DiodeKind::Standard,
                ..DiodeSpec::default()
            },
        )
        .unwrap()
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 128,
            rng_seed: proptest::test_runner::RngSeed::Fixed(0xD10D_E2026),
            ..ProptestConfig::default()
        })]

        #[test]
        fn finite_forward_sweeps_are_bounded_and_monotone(
            forward in 0.1f64..=2.0,
            resistance in 1.0f64..=100_000.0,
            start in -12.0f64..=12.0,
            span in 0.0f64..=12.0,
        ) {
            let model = DiodeModel::new(forward, resistance, DiodeSpec::default()).unwrap();
            let first = model.linearize(start);
            let second = model.linearize(start + span);
            prop_assert!(first.current.is_finite());
            prop_assert!(second.current.is_finite());
            prop_assert!(first.conductance.is_finite() && first.conductance > 0.0);
            prop_assert!(second.conductance.is_finite() && second.conductance > 0.0);
            prop_assert!(second.current + 1e-12 >= first.current);
        }

        #[test]
        fn zener_reverse_branch_is_bounded_and_monotone(
            breakdown in 1.0f64..=12.0,
            resistance in 1.0f64..=100_000.0,
            voltage in -24.0f64..=0.0,
        ) {
            let model = DiodeModel::new(
                0.7,
                10.0,
                DiodeSpec {
                    kind: DiodeKind::Zener,
                    breakdown_voltage: Some(breakdown),
                    breakdown_resistance: Some(resistance),
                },
            ).unwrap();
            let at_voltage = model.linearize(voltage);
            let one_step_more_reverse = model.linearize(voltage - 0.001);
            prop_assert!(at_voltage.current.is_finite());
            prop_assert!(at_voltage.conductance.is_finite() && at_voltage.conductance > 0.0);
            prop_assert!(one_step_more_reverse.current <= at_voltage.current + 1e-12);
        }
    }

    #[test]
    fn standard_diode_blocks_reverse_bias_but_conducts_forward_bias() {
        let model = standard();
        assert!(model.current(-5.0).abs() < 1e-7);
        assert!(model.current(0.8) > 0.001);
        assert!(model.linearize(0.8).conductance > 0.0);
    }

    #[test]
    fn zener_breakdown_is_a_calculated_reverse_current() {
        let model = DiodeModel::new(
            0.7,
            10.0,
            DiodeSpec {
                kind: DiodeKind::Zener,
                breakdown_voltage: Some(6.8),
                breakdown_resistance: Some(10.0),
            },
        )
        .unwrap();
        assert!(model.current(-5.0).abs() < 1e-7);
        assert!(model.current(-7.5) < -0.01);
    }
}
