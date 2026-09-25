// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Library-owned continuous NLP transformations shared by both native backends.
mod pipeline;
#[cfg(test)]
mod tests;
use crate::{ProblemError, quality::Tolerances};
pub use pipeline::{Pipeline, Transport};
use pounce_presolve::{AuxiliaryCouplingPolicy, LicqAction, PresolveOptions};
use pse_ids::{ContentHash, FramedHasher};
use std::collections::{BTreeMap, BTreeSet};

/// Positive numerical scales in original source order; never physical conversions.
#[derive(Clone, Debug)]
pub struct Scaling {
    /// Native objective scale.
    pub objective: f64,
    /// Native independent variable scales.
    pub variables: Vec<f64>,
    /// Native row scales.
    pub constraints: Vec<f64>,
}
impl Scaling {
    /// Native scaling must preserve feasible sets and objective sense.
    pub fn validate(&self, n: usize, m: usize) -> Result<(), ProblemError> {
        if self.variables.len() != n
            || self.constraints.len() != m
            || self
                .variables
                .iter()
                .chain(&self.constraints)
                .chain(std::iter::once(&self.objective))
                .any(|v| !v.is_finite() || *v < 1e-12)
        {
            return Err(ProblemError::Contract(
                "native scaling dimensions/positive values".into(),
            ));
        }
        Ok(())
    }
    /// Bit-preserving numerical identity.
    pub fn key(&self) -> ContentHash {
        let mut h = FramedHasher::new("pse.native.scaling.v1");
        h.u64(self.objective.to_bits())
            .u64(self.variables.len() as u64)
            .u64(self.constraints.len() as u64);
        for v in self.variables.iter().chain(&self.constraints) {
            h.u64(v.to_bits());
        }
        h.finish_hash()
    }
}
/// Native library passes, independently requested and qualified.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Pass {
    /// Propagation using proved affine rows.
    LinearBounds,
    /// Library redundancy analysis.
    RedundantRows,
    /// Library affine column elimination and recovery.
    AffineElimination,
    /// Native expression-tape interval propagation.
    Fbbt,
    /// Equality-rank diagnostics, without objective-changing remedies.
    RankDiagnostics,
    /// Explicit safe auxiliary nonlinear reduction.
    Auxiliary,
}
/// User policy. Native options are retained in full rather than stringly reimplemented.
#[derive(Clone, Debug, Default)]
pub enum Policy {
    /// Preserve source coordinates through an identity transport.
    Off,
    /// Enable only qualified source-backed passes.
    #[default]
    Auto,
    /// Complete library controls; required ineligible passes fail admission.
    Explicit {
        /// Native presolve controls, with bounded caps.
        options: PresolveOptions,
        /// Passes which may not silently become unavailable.
        required: BTreeSet<Pass>,
    },
}
/// One pass's requested, eligible and effective state.
#[derive(Clone, Debug)]
pub struct PassReport {
    /// The caller requested the pass (including automatic policy).
    pub requested: bool,
    /// The source contract qualifies for the pinned implementation.
    pub eligible: bool,
    /// The qualified pass was installed in the library pipeline.
    pub applied: bool,
    /// Why eligibility was declined, if applicable.
    pub reason: Option<String>,
}
/// Immutable original-to-native transformation receipt, never an executable math IR.
#[derive(Clone, Debug)]
pub struct Report {
    /// Requested policy and all native controls.
    pub requested: Policy,
    /// Effective native controls after qualification.
    pub effective: PresolveOptions,
    /// Independent pass decisions.
    pub passes: BTreeMap<Pass, PassReport>,
    /// Complete source assumptions, including tape identity.
    pub facts: Option<ContentHash>,
    /// Actual projected bounds, rows, columns, scales and source identity.
    pub transformation: ContentHash,
    /// Source and native dimensions.
    pub dimensions: (usize, usize, usize, usize),
    /// Native diagnostics and explicit numerical qualification limits.
    pub diagnostics: BTreeMap<String, String>,
    /// Original indices of retained independent variables.
    pub columns: Vec<usize>,
    /// Original indices of retained constraint rows.
    pub rows: Vec<usize>,
    /// Library-certified infeasibility; raw propagation crossings do not set this.
    pub certified_infeasible: bool,
}
impl Policy {
    /// Complete option identity; no Debug strings or library fingerprint alone.
    pub fn key(&self) -> ContentHash {
        let mut h = FramedHasher::new("pse.presolve.policy.v1");
        let (o, required) = match self {
            Self::Off => {
                h.u64(0);
                return h.finish_hash();
            }
            Self::Auto => {
                h.u64(1);
                return h.finish_hash();
            }
            Self::Explicit { options, required } => {
                h.u64(2);
                (options, required)
            }
        };
        for v in [
            o.enabled,
            o.bound_tightening,
            o.redundant_constraint_removal,
            o.linear_eq_reduction,
            o.licq_check,
            o.warm_z_bounds,
            o.auxiliary,
            o.auxiliary_diagnostics,
            o.fbbt,
        ] {
            h.bool(v);
        }
        for v in [
            o.certify_tol,
            o.bound_mult_init_val,
            o.auxiliary_tol,
            o.auxiliary_wall_time_fraction,
            o.fbbt_tol,
        ] {
            h.u64(v.to_bits());
        }
        for v in [
            o.print_level,
            o.max_passes,
            o.auxiliary_max_block_dim,
            o.fbbt_max_iter,
            o.fbbt_max_constraints,
        ] {
            h.u64(v as u64);
        }
        h.u64(o.licq_action as u64)
            .u64(o.auxiliary_coupling as u64)
            .u64(required.len() as u64);
        for v in required {
            h.u64(*v as u64);
        }
        h.finish_hash()
    }
    pub(super) fn qualify(
        &self,
        oracle: &dyn crate::NlpOracle,
        t: &Tolerances,
    ) -> Result<Report, ProblemError> {
        let n = oracle.contract().variables.len();
        let m = oracle.contract().rows.len();
        t.validate(n, m)?;
        let mut o = match self {
            Self::Off => PresolveOptions::defaults(),
            Self::Auto => PresolveOptions {
                enabled: true,
                linear_eq_reduction: true,
                fbbt: true,
                ..PresolveOptions::defaults()
            },
            Self::Explicit { options, .. } => *options,
        };
        if !o.certify_tol.is_finite()
            || o.certify_tol <= 0.0
            || !o.fbbt_tol.is_finite()
            || o.fbbt_tol <= 0.0
            || !o.auxiliary_tol.is_finite()
            || o.auxiliary_tol <= 0.0
            || !o.bound_mult_init_val.is_finite()
            || o.bound_mult_init_val < 0.0
            || !(1..=1000).contains(&o.max_passes)
            || !(1..=1000).contains(&o.fbbt_max_iter)
            || !(1..=1024).contains(&o.auxiliary_max_block_dim)
            || o.fbbt_max_constraints < 0
            || !(0.0..=1.0).contains(&o.auxiliary_wall_time_fraction)
            || o.licq_action != LicqAction::Warn
            || o.auxiliary_coupling == AuxiliaryCouplingPolicy::Aggressive
        {
            return Err(ProblemError::Contract(
                "unsupported or unbounded native presolve controls".into(),
            ));
        }
        let facts = oracle.presolve_facts();
        let affine = facts.is_some_and(|f| f.affine.iter().any(Option::is_some));
        let has_tape = facts.is_some_and(|f| f.complete.iter().any(|v| *v));
        // The wrapper fixes eq_tol/coeff_tol at 1e-12. Decline rather than turn
        // a narrow interval into an equality or silently discard small support.
        let exact_rows = oracle
            .constraint_bounds()
            .iter()
            .all(|(l, u)| l == u || (u - l).abs() > 1e-12);
        let safe_coefficients = facts.is_some_and(|f| {
            f.affine.iter().flatten().all(|r| {
                let scale = r.entries.values().map(|v| v.abs()).fold(1.0, f64::max);
                r.entries.values().all(|v| v.abs() > 1e-12 * scale)
            })
        });
        let safe_tolerance = t.rows.iter().chain(&t.variables).all(|v| *v >= 1e-12);
        let mut passes = BTreeMap::new();
        for (pass, requested, eligible, reason) in [
            (
                Pass::LinearBounds,
                o.bound_tightening,
                affine && safe_coefficients,
                "affine coefficient support is unavailable or below native threshold",
            ),
            (
                Pass::RedundantRows,
                o.redundant_constraint_removal,
                affine && exact_rows && safe_coefficients,
                "row bounds or coefficients do not qualify",
            ),
            (
                Pass::AffineElimination,
                o.linear_eq_reduction,
                affine && exact_rows && safe_coefficients && safe_tolerance,
                "native 1e-12 equality/coefficient thresholds do not qualify",
            ),
            (
                Pass::Fbbt,
                o.fbbt,
                has_tape,
                "no completely projected expression row",
            ),
            (
                Pass::RankDiagnostics,
                o.licq_check,
                exact_rows,
                "native equality threshold does not qualify",
            ),
            (
                Pass::Auxiliary,
                o.auxiliary,
                facts.is_some() && exact_rows && safe_tolerance,
                "auxiliary source/objective facts or tolerance are unavailable",
            ),
        ] {
            let requested = requested && o.enabled;
            let applied = requested && eligible;
            if let Self::Explicit { required, .. } = self
                && required.contains(&pass)
                && !applied
            {
                return Err(ProblemError::Contract(format!(
                    "required {pass:?} unavailable: {reason}"
                )));
            }
            passes.insert(
                pass,
                PassReport {
                    requested,
                    eligible,
                    applied,
                    reason: (!eligible).then(|| reason.into()),
                },
            );
        }
        o.bound_tightening = passes[&Pass::LinearBounds].applied;
        o.redundant_constraint_removal = passes[&Pass::RedundantRows].applied;
        o.linear_eq_reduction = passes[&Pass::AffineElimination].applied;
        o.fbbt = passes[&Pass::Fbbt].applied;
        o.licq_check = passes[&Pass::RankDiagnostics].applied;
        o.auxiliary = passes[&Pass::Auxiliary].applied;
        // A certificate must exceed every caller's allowed physical violation.
        o.certify_tol = o.certify_tol.max(
            t.rows
                .iter()
                .chain(&t.variables)
                .copied()
                .fold(0.0, f64::max),
        );
        Ok(Report{requested:self.clone(),effective:o,passes,facts:facts.map(|f|f.key),transformation:self.key(),dimensions:(n,m,n,m),
            diagnostics:BTreeMap::from([("native.qualification".into(),"pounce-presolve 0.12.0: equality/coefficient tolerance 1e-12; bound-dual recovery activity tolerance 1e-6; LICQ diagnostics only".into())]),
            columns:(0..n).collect(),rows:(0..m).collect(),certified_infeasible:false})
    }
}

impl Policy {
    /// Resolve the pinned library's complete presolve option registry. Names, native
    /// types and ranges are library-owned; qualification still checks physical meaning.
    pub fn from_native_options(
        supplied: &crate::solve::Options,
        required: BTreeSet<Pass>,
    ) -> Result<Self, ProblemError> {
        use crate::solve::OptionValue;
        use pounce_common::{options_list::OptionsList, reg_options::RegisteredOptions};
        let registry = RegisteredOptions::default();
        let error =
            |e: pounce_common::exception::SolverException| ProblemError::Contract(e.to_string());
        pounce_presolve::options::register_options(&registry).map_err(error)?;
        let mut options = OptionsList::with_registered(std::rc::Rc::new(registry));
        options
            .set_bool_value("presolve", true, true, false)
            .map_err(error)?;
        for (key, value) in supplied {
            match value {
                OptionValue::Text(v) => options.set_string_value(key, v, true, false),
                OptionValue::Real(v) => options.set_numeric_value(key, *v, true, false),
                OptionValue::Integer(v) => options.set_integer_value(key, *v, true, false),
                OptionValue::Bool(v) => options.set_bool_value(key, *v, true, false),
            }
            .map_err(error)?;
        }
        let mut resolved = PresolveOptions::from_options_list(&options).map_err(error)?;
        // This registry excludes the solver's `tol`; its getter returns zero for
        // an unregistered key. Physical qualification below owns certification.
        resolved.certify_tol = PresolveOptions::defaults().certify_tol;
        Ok(Self::Explicit {
            options: resolved,
            required,
        })
    }
}
