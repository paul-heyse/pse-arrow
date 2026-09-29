// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The advanced step (Plan 22 Y5c2; ADR-0118 item 12; I16): a prediction of the solution at
//! changed parameter values by one parametric backsolve against a retained factor.
//!
//! A sensitivity request that keeps its factor leaves an [`Advance`] behind a certified
//! step: the pinned parametric factor with the candidate it was assembled at. A later step
//! at values `p + Δp` predicts `w + Δw` with `Δw = K⁻¹·(−e_pin)·Δp`, the step S1's
//! sensitivities are columns of, without evaluating or factoring anything.
//!
//! **Validity.** The prediction is the first-order solution of the perturbed problem while
//! the active set holds. It is refused, and the caller solves in full, when the step
//! changes the active set: an active constraint whose multiplier changes sign leaves it,
//! and an inactive row or bound that the step drives beyond its tolerance enters it. An
//! inactive row is tested on its linearization `g + J·Δw` through the parameter columns.
//! On a problem whose KKT conditions are linear in the parameters (a quadratic program
//! with linear constraints), an admitted prediction is the solution itself.
use super::{KktFactor, Side};
use crate::{quality::Tolerances, solve::SolveReport};
use pounce_sens_core::{IndexSchurData, SensApplication, SensOptions};
use pse_ids::SemanticId;

/// A certified step's pinned parametric factor and what a prediction from it needs, kept in
/// the worker's retained state and charged to the job's allowance.
#[derive(Clone, Debug)]
pub struct Advance {
    factor: KktFactor,
    /// The KKT rows of the pins, in parameter order.
    pins: Vec<i32>,
    /// The parameters and the values the factor was assembled at.
    parameters: Vec<(SemanticId, f64)>,
    /// The solve's variables, in report order.
    variables: Vec<SemanticId>,
    /// The candidate's primal values.
    primal: Vec<f64>,
    /// Its row multipliers, in the minimization convention.
    row_dual: Vec<f64>,
    /// Its lower and upper bound multipliers.
    lower: Vec<f64>,
    upper: Vec<f64>,
    /// Its row values and their bounds.
    values: Vec<f64>,
    row_bounds: Vec<(f64, f64)>,
    /// The bounds of the solve's columns.
    bounds: Vec<(f64, f64)>,
    /// The parametric Jacobian at the candidate, `(row, column, value)` over the solve's
    /// columns followed by the parameters.
    jacobian: Vec<(usize, usize, f64)>,
    tolerances: Tolerances,
}

/// A predicted solution at changed parameter values.
#[derive(Clone, Debug, PartialEq)]
pub struct Prediction {
    /// The solve's variables, in report order.
    pub variables: Vec<SemanticId>,
    /// Their predicted values, in original units.
    pub primal: Vec<f64>,
    /// The predicted row multipliers, in the minimization convention of the candidate's.
    pub row_dual: Vec<f64>,
    /// The parameter step `Δp` from the values the factor was assembled at, in parameter
    /// order.
    pub step: Vec<f64>,
}
impl Prediction {
    /// The predicted value of `variable`.
    pub fn value(&self, variable: &SemanticId) -> Option<f64> {
        self.variables
            .iter()
            .position(|v| v == variable)
            .map(|j| self.primal[j])
    }
}

/// Why no prediction was made; the caller solves in full and records it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fallback {
    /// No factor is retained: none was requested, the step's sensitivities were withheld,
    /// or its charge exceeded the job's allowance.
    NotRetained,
    /// The prediction names other parameters than the retained factor, or a nonfinite
    /// value.
    Parameters,
    /// The parametric backsolve failed.
    Backsolve,
    /// The step changes the active set.
    ActiveSet {
        /// Active constraints whose multiplier changes sign.
        leaving: usize,
        /// Inactive rows and bounds driven beyond their tolerance.
        entering: usize,
    },
}
impl Fallback {
    /// Stable snake-case spelling of the reason.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotRetained => "not_retained",
            Self::Parameters => "parameters",
            Self::Backsolve => "backsolve",
            Self::ActiveSet { .. } => "active_set",
        }
    }
}
impl std::fmt::Display for Fallback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotRetained => f.write_str("no parametric factor is retained"),
            Self::Parameters => {
                f.write_str("the prediction's parameters are not the retained factor's")
            }
            Self::Backsolve => f.write_str("the parametric backsolve failed"),
            Self::ActiveSet { leaving, entering } => write!(
                f,
                "the step changes the active set ({leaving} leaving, {entering} entering)"
            ),
        }
    }
}

impl Advance {
    /// What a prediction needs from a certified step: its factor, pin rows and candidate.
    /// `None` when the report lacks a multiplier or an observation, or a pin is not a row
    /// of the factor.
    pub(super) fn new(
        report: &SolveReport,
        analysed: &super::sensitivity::Analysed,
        parameters: Vec<(SemanticId, f64)>,
        tolerances: &Tolerances,
    ) -> Option<Self> {
        let candidate = report.candidate.as_ref()?;
        let observation = report.observation.as_ref()?;
        let (n, np) = (report.variables.len(), parameters.len());
        let pins = super::sensitivity::pin_rows(&analysed.factor, n, np)?;
        let (lower, upper) = candidate.bound_dual.clone()?;
        Some(Self {
            factor: analysed.factor.clone(),
            pins,
            parameters,
            variables: report.variables.clone(),
            primal: candidate.primal.clone(),
            row_dual: candidate.row_dual.clone()?,
            lower,
            upper,
            values: observation.values.clone(),
            row_bounds: observation.bounds.clone(),
            bounds: analysed.bounds.clone(),
            jacobian: analysed.jacobian.clone(),
            tolerances: tolerances.clone(),
        })
    }
    /// Bytes held: the factor and the candidate data a prediction reads.
    pub fn bytes(&self) -> usize {
        let f = size_of::<f64>();
        let vectors = (self.primal.len() + self.lower.len() + self.upper.len()) * f
            + (self.row_dual.len() + self.values.len()) * f
            + (self.row_bounds.len() + self.bounds.len()) * 2 * f
            + (self.tolerances.variables.len() + self.tolerances.rows.len()) * f
            + self.jacobian.len() * size_of::<(usize, usize, f64)>()
            + self.variables.len() * size_of::<SemanticId>()
            + self.parameters.len() * size_of::<(SemanticId, f64)>()
            + self.pins.len() * size_of::<i32>();
        self.factor.bytes().saturating_add(vectors)
    }
    /// The parameters and the values the factor was assembled at.
    pub fn parameters(&self) -> &[(SemanticId, f64)] {
        &self.parameters
    }
}

/// Predict the solution at `parameters`, the retained factor's parameters in its order with
/// new values, or say why the caller must solve in full.
///
/// # Errors
/// The [`Fallback`] reason.
pub fn predict(
    advance: &Advance,
    parameters: &[(SemanticId, f64)],
) -> Result<Prediction, Fallback> {
    let np = advance.parameters.len();
    if parameters.len() != np
        || parameters
            .iter()
            .zip(&advance.parameters)
            .any(|((id, value), (held, _))| id != held || !value.is_finite())
    {
        return Err(Fallback::Parameters);
    }
    let delta: Vec<f64> = parameters
        .iter()
        .zip(&advance.parameters)
        .map(|((_, value), (_, held))| value - held)
        .collect();
    let selector = IndexSchurData::from_parts(advance.pins.clone(), vec![-1; np])
        .map_err(|_| Fallback::Backsolve)?;
    let steps = SensApplication::new(
        selector,
        advance.factor.clone(),
        SensOptions {
            run_sens: true,
            ..SensOptions::default()
        },
    );
    let layout = advance.factor.layout();
    let mut step = vec![0.0; layout.dim()];
    if !steps.parametric_step(&delta, &mut step) || step.iter().any(|v| !v.is_finite()) {
        return Err(Fallback::Backsolve);
    }
    let (n, m) = (advance.primal.len(), advance.row_dual.len());
    let primal: Vec<f64> = advance
        .primal
        .iter()
        .zip(&step[..n])
        .map(|(x, dx)| x + dx)
        .collect();
    let mut row_dual = advance.row_dual.clone();
    let mut active_rows = vec![false; m];
    let mut leaving = 0;
    for (k, (r, side)) in layout.rows.iter().enumerate() {
        let r = r.get();
        active_rows[r] = true;
        row_dual[r] += step[layout.variables + k];
        // A row active at its upper limit has λ ≥ 0, at its lower limit λ ≤ 0.
        leaving += usize::from(match side {
            Side::Upper => row_dual[r] < 0.0,
            Side::Lower => row_dual[r] > 0.0,
            Side::Equal => false,
        });
    }
    let base = layout.variables + layout.rows.len();
    let mut active_bounds = vec![false; n];
    for (k, (j, side)) in layout.bounds.iter().enumerate() {
        let j = j.get();
        if j >= n {
            // A pin.
            continue;
        }
        active_bounds[j] = true;
        // The row carries z_L for a lower bound and z_U for an upper one.
        let dz = step[base + k];
        leaving += usize::from(match side {
            Side::Lower => advance.lower[j] + dz < 0.0,
            Side::Upper => advance.upper[j] + dz < 0.0,
            Side::Equal => false,
        });
    }
    let outside = |value: f64, (lower, upper): (f64, f64), tolerance: f64| {
        value < lower - tolerance || value > upper + tolerance
    };
    let mut entering = (0..n)
        .filter(|j| {
            !active_bounds[*j]
                && outside(primal[*j], advance.bounds[*j], advance.tolerances.variables[*j])
        })
        .count();
    // Inactive rows on their linearization through the variable and parameter steps.
    let mut values = advance.values.clone();
    for (r, col, value) in &advance.jacobian {
        if active_rows[*r] {
            continue;
        }
        let d = if *col < n {
            step[*col]
        } else {
            delta[*col - n]
        };
        values[*r] += value * d;
    }
    entering += (0..m)
        .filter(|r| {
            !active_rows[*r] && outside(values[*r], advance.row_bounds[*r], advance.tolerances.rows[*r])
        })
        .count();
    if leaving + entering > 0 {
        return Err(Fallback::ActiveSet { leaving, entering });
    }
    Ok(Prediction {
        variables: advance.variables.clone(),
        primal,
        row_dual,
        step: delta,
    })
}

