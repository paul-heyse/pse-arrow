// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Library-owned continuous NLP transformations shared by both native backends.
mod pipeline;
mod records;
#[cfg(test)]
mod tests;
use crate::{ProblemError, quality::Tolerances};
pub use pipeline::{Pipeline, Transport};
use pounce_presolve::{AuxiliaryCouplingPolicy, LicqAction, PresolveOptions};
use pse_ids::{ContentHash, FramedHasher};
use pse_math::index::{OriginalCol, OriginalRow, PresolvedCol, PresolvedRow, TiVec};
use std::collections::{BTreeMap, BTreeSet};

/// The native library passes, independently requested and qualified, and the kind of a
/// [`Policy`]: registry vocabularies whose spellings are their one boundary name (ADR-0115
/// Outcome 3). Native options and required passes belong to an explicit policy only.
pub use pse_model::generated::enums::{PresolvePass as Pass, PresolvePolicyKind as PolicyKind};
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
/// Source and native problem dimensions of one presolve.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Dimensions {
    /// Columns of the source problem.
    pub original_columns: usize,
    /// Rows of the source problem.
    pub original_rows: usize,
    /// Columns of the presolved problem the native solver receives.
    pub presolved_columns: usize,
    /// Rows of the presolved problem the native solver receives.
    pub presolved_rows: usize,
}
impl Dimensions {
    /// The dimensions of a presolve that retains every column and row.
    pub const fn identity(columns: usize, rows: usize) -> Self {
        Self {
            original_columns: columns,
            original_rows: rows,
            presolved_columns: columns,
            presolved_rows: rows,
        }
    }
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
    pub dimensions: Dimensions,
    /// Native diagnostics and explicit numerical qualification limits.
    pub diagnostics: BTreeMap<String, String>,
    /// The original column of each retained independent variable, by presolved column.
    pub columns: TiVec<PresolvedCol, OriginalCol>,
    /// The original row of each retained constraint row, by presolved row.
    pub rows: TiVec<PresolvedRow, OriginalRow>,
    /// Library-certified infeasibility; raw propagation crossings do not set this.
    pub proof: Option<PresolveProof>,
    /// A submitted active-set working set and whether this transformation retained it.
    pub working_set: Option<crate::solve::WorkingSetTransfer>,
    /// Why the policy that ran differs from `requested`; `None` when it is the requested one.
    pub resolution: Option<Resolution>,
}
/// Why a requested presolve policy resolved to another one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Resolution {
    /// `Auto` under a method that relaxes every row (the ℓ1 exact penalty, ADR-0109): every
    /// pass assumes the rows hold, so `Auto` selects none and presolve runs `Off`.
    RelaxedRows,
}
/// Retained library proof plus the original scope and the budget it survived.
#[derive(Clone, Debug)]
pub struct PresolveProof {
    /// Original row, producing instance and output mappings within this proof scope.
    pub contributions: Vec<(pse_ids::SemanticId, pse_ids::SemanticId, usize)>,
    /// Library-owned proof kind, retaining its original row ordinal if supplied.
    pub native: pounce_nlp::tnlp::InfeasibilityProof,
    /// Original row identity for an interval witness; absent for coarse propagation.
    pub witness_row: Option<pse_ids::SemanticId>,
    /// Complete original row scope, not a claimed minimal conflicting set.
    pub rows: Vec<pse_ids::SemanticId>,
    /// Complete original variable scope.
    pub columns: Vec<pse_ids::SemanticId>,
    /// Exact coordinate transformation used for confirmation.
    pub normalization: ContentHash,
    /// Per-bound normalized acceptance budgets used in confirmation.
    pub budgets: Tolerances,
}
impl PresolveProof {
    /// Stable interpretation of the retained library proof, without reconstructing it.
    pub fn kind(&self) -> &'static str {
        match self.native {
            pounce_nlp::tnlp::InfeasibilityProof::IntervalArithmetic { .. } => {
                "interval_arithmetic"
            }
            pounce_nlp::tnlp::InfeasibilityProof::BoundPropagation => "bound_propagation",
        }
    }
}
impl Policy {
    /// Complete option identity; no Debug strings or library fingerprint alone.
    pub fn key(&self) -> ContentHash {
        let mut h = FramedHasher::new(pse_ids::Frame::PresolvePolicyV1);
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
            return Err(ProblemError::Unsupported(
                "unsupported or unbounded native presolve controls".into(),
            ));
        }
        let normalization = oracle
            .normalization()
            .cloned()
            .unwrap_or_else(|| pse_math::normalization::Normalization::identity(n, m));
        normalization.validate(n, m)?;
        let normalized_tolerances = t.normalized(&normalization)?;
        let t = &normalized_tolerances;
        let normalized_facts = oracle
            .presolve_facts()
            .map(|f| normalization.facts(f, 1_000_000))
            .transpose()?;
        let facts = normalized_facts.as_ref();
        let affine = facts.is_some_and(|f| f.affine.iter().any(Option::is_some));
        let has_tape = facts.is_some_and(|f| f.complete.iter().any(|v| *v));
        // The wrapper fixes eq_tol/coeff_tol at 1e-12. Decline rather than turn
        // a narrow interval into an equality or silently discard small support.
        let exact_rows = oracle
            .constraint_bounds()
            .iter()
            .zip(&normalization.rows)
            .all(|((l, u), s)| l == u || ((u - l) / s).abs() > 1e-12);
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
                return Err(ProblemError::Unsupported(format!(
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
        // Native margins are normalized algorithm controls. A terminal certificate
        // separately survives each original bound's own acceptance budget.
        Ok(Report{requested:self.clone(),effective:o,passes,facts:facts.map(|f|f.key),transformation:self.key(),dimensions:Dimensions::identity(n,m),
            diagnostics:BTreeMap::from([("native.qualification".into(),"pounce-presolve 0.12.0: equality/coefficient tolerance 1e-12; bound-dual recovery activity tolerance 1e-6; LICQ diagnostics only".into())]),
            columns:(0..n).map(OriginalCol::new).collect(),rows:(0..m).map(OriginalRow::new).collect(),proof:None,working_set:None,resolution:None})
    }
}

impl Policy {
    /// A policy of `kind`. Native options and required passes are admitted only by an
    /// explicit policy, which resolves them against the library registry.
    ///
    /// # Errors
    /// Options or required passes for a non-explicit kind, or options the library refuses.
    pub fn new(
        kind: PolicyKind,
        supplied: &crate::solve::Options,
        required: BTreeSet<Pass>,
    ) -> Result<Self, ProblemError> {
        match kind {
            PolicyKind::Explicit => Self::from_native_options(supplied, required),
            _ if !supplied.is_empty() || !required.is_empty() => Err(ProblemError::Contract(
                "native presolve options and required passes need an explicit policy".into(),
            )),
            PolicyKind::Off => Ok(Self::Off),
            PolicyKind::Auto => Ok(Self::Auto),
        }
    }
    /// The kind of this policy.
    pub const fn kind(&self) -> PolicyKind {
        match self {
            Self::Off => PolicyKind::Off,
            Self::Auto => PolicyKind::Auto,
            Self::Explicit { .. } => PolicyKind::Explicit,
        }
    }
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
