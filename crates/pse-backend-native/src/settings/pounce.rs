// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The POUNCE settings document: the NLP method, the complete native FERAL configuration and
//! the interior-point restart. The FERAL configuration serializes through a remote
//! definition checked against every upstream field, so a FERAL upgrade that adds a field
//! fails to compile instead of leaving identity (F09); its orderings and scalings are
//! registry vocabularies, and its defaults are FERAL's own.
use crate::solve::WarmRestart;
use feral::{scaling::ScalingStrategy, symbolic::OrderingMethod};
use pse_model::generated::enums::{
    FeralOrdering, FeralScaling, PounceFdColoring, PounceFdPattern, PouncePartitionedElements,
    PouncePartitionedUpdate,
};

/// The native infinity threshold, shared by contextual admission and final execution.
pub(crate) fn admit_bound(value: f64) -> Result<(), crate::ProblemError> {
    if value.is_nan() || value.is_finite() && value.abs() >= 1e19 {
        return Err(crate::ProblemError::Unsupported(
            "POUNCE finite bound reaches native infinity threshold".into(),
        ));
    }
    Ok(())
}

/// The NLP method, a registry vocabulary (ADR-0115 Outcome 3). The algorithm is
/// explicit; POUNCE never silently changes the selected problem class. The Thierry–Biegler
/// ℓ1 exact penalty-barrier method (`pounce-l1penalty`, ADR-0109) is explicit only: never
/// selected automatically and never a retry. Every row is relaxed (inequalities through
/// bounded slacks), so an infeasible model returns a least-infeasible point and a feasible
/// one a point the penalty makes exact. Presolve `Auto` resolves to `Off` for it, recorded
/// in the presolve report; explicit passes are refused, since every pass assumes the rows
/// hold.
pub use pse_model::generated::enums::PounceMethod as Method;

/// Complete native FERAL configuration, with thread/FMA policy applied at execution.
pub type LinearSettings = pounce_feral::FeralConfig;

/// The POUNCE adapter's settings type. Its identity derives from serde.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
#[schemars(rename = "PounceSettings")]
pub struct Settings {
    /// NLP method.
    pub method: Method,
    /// Native linear settings.
    #[serde(with = "FeralIdentity")]
    pub linear: LinearSettings,
    /// Interior-point restart of a submitted primal-dual seed (L-N3).
    pub restart: WarmRestart,
    /// Library-owned partitioned curvature settings.
    pub partitioned: PartitionedSettings,
    /// Library-owned finite-difference curvature settings.
    pub finite_difference: FiniteDifferenceSettings,
}
impl Default for Settings {
    /// The interior-point method with FERAL's own defaults and the default restart.
    fn default() -> Self {
        Self {
            method: Method::InteriorPoint,
            linear: LinearSettings::default(),
            restart: WarmRestart::default(),
            partitioned: PartitionedSettings::default(),
            finite_difference: FiniteDifferenceSettings::default(),
        }
    }
}

/// Typed partitioned update controls. An absent formula preserves the library's
/// mode-dependent default and its option set-ness.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct PartitionedSettings {
    /// Explicit element update, or the library default.
    pub update_type: Option<PouncePartitionedUpdate>,
    /// Wider elements degrade to the library diagonal approximation.
    pub max_element: usize,
    /// Element construction.
    pub elements: PouncePartitionedElements,
    /// Width of primal blocks.
    pub block_size: usize,
    /// Finite positive curvature multiplier, absent for uncapped native updates.
    pub curvature_cap: Option<f64>,
}
impl Default for PartitionedSettings {
    fn default() -> Self {
        Self {
            update_type: None,
            max_element: 64,
            elements: PouncePartitionedElements::PerConstraint,
            block_size: 64,
            curvature_cap: None,
        }
    }
}
/// Typed finite-difference pattern, coloring and native reuse controls.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct FiniteDifferenceSettings {
    /// Declared Hessian support, falling back to conservative Jacobian/objective support.
    pub pattern: PounceFdPattern,
    /// Library coloring algorithm.
    pub coloring: PounceFdColoring,
    /// Relative native iterate distance permitting Hessian reuse.
    pub reuse_tolerance: f64,
}
impl Default for FiniteDifferenceSettings {
    fn default() -> Self {
        Self {
            pattern: PounceFdPattern::Declared,
            coloring: PounceFdColoring::Cpr,
            reuse_tolerance: 0.0,
        }
    }
}
impl Settings {
    /// One typed lowering for the curvature controls; callers cannot override these
    /// assignments with raw competing options.
    pub(crate) fn curvature_options(
        &self,
        mode: crate::solve::HessianMode,
    ) -> Result<crate::solve::Options, crate::ProblemError> {
        use crate::{
            ProblemError,
            solve::{HessianMode, OptionValue, Options},
        };
        if self.partitioned.max_element == 0
            || self.partitioned.max_element > i32::MAX as usize
            || self.partitioned.block_size == 0
            || self.partitioned.block_size > i32::MAX as usize
            || self
                .partitioned
                .curvature_cap
                .is_some_and(|v| !v.is_finite() || v <= 0.0)
            || !self.finite_difference.reuse_tolerance.is_finite()
            || self.finite_difference.reuse_tolerance < 0.0
        {
            return Err(ProblemError::Contract(
                "invalid POUNCE curvature settings".into(),
            ));
        }
        if self.method == Method::ActiveSetSqp
            && matches!(
                mode,
                HessianMode::Partitioned | HessianMode::FiniteDifference
            )
        {
            return Err(ProblemError::Unsupported("POUNCE active-set SQP does not consume partitioned or finite-difference IPM curvature".into()));
        }
        if let Some(maximum) = self.linear.bounded_storage_max_dimension
            && (maximum == 0
                || !matches!(self.linear.ordering, OrderingMethod::Amd)
                || self.linear.parallel == Some(true)
                || !matches!(
                    self.linear.scaling,
                    ScalingStrategy::Auto
                        | ScalingStrategy::InfNorm
                        | ScalingStrategy::Identity
                        | ScalingStrategy::Mc64Symmetric
                )
                || self.method != Method::InteriorPoint)
        {
            return Err(ProblemError::Unsupported("bounded POUNCE linear storage requires IPM, assembled curvature, serial AMD, supported scaling and a positive geometry maximum".into()));
        }
        let mut options = Options::new();
        match mode {
            HessianMode::Auto => {
                return Err(ProblemError::Contract(
                    "automatic curvature must resolve before POUNCE admission".into(),
                ));
            }
            HessianMode::Partitioned => {
                if let Some(update) = self.partitioned.update_type {
                    options.insert(
                        "partitioned_update_type".into(),
                        OptionValue::Text(
                            match update {
                                PouncePartitionedUpdate::Bfgs => "bfgs",
                                PouncePartitionedUpdate::Sr1 => "sr1",
                            }
                            .into(),
                        ),
                    );
                }
                options.insert(
                    "partitioned_max_element".into(),
                    OptionValue::Integer(self.partitioned.max_element as i32),
                );
                options.insert(
                    "partitioned_elements".into(),
                    OptionValue::Text(
                        match self.partitioned.elements {
                            PouncePartitionedElements::PerConstraint => "per-constraint",
                            PouncePartitionedElements::PrimalBlock => "blocks",
                        }
                        .into(),
                    ),
                );
                options.insert(
                    "partitioned_block_size".into(),
                    OptionValue::Integer(self.partitioned.block_size as i32),
                );
                if let Some(cap) = self.partitioned.curvature_cap {
                    options.insert("partitioned_curvature_cap".into(), OptionValue::Real(cap));
                }
            }
            HessianMode::FiniteDifference => {
                options.insert(
                    "fd_hessian_pattern".into(),
                    OptionValue::Text(
                        match self.finite_difference.pattern {
                            PounceFdPattern::Declared => "declared",
                            PounceFdPattern::Jacobian => "jacobian",
                        }
                        .into(),
                    ),
                );
                options.insert(
                    "fd_hessian_coloring".into(),
                    OptionValue::Text(
                        match self.finite_difference.coloring {
                            PounceFdColoring::Cpr => "cpr",
                            PounceFdColoring::Star => "star",
                        }
                        .into(),
                    ),
                );
                options.insert(
                    "fd_hessian_reuse_tol".into(),
                    OptionValue::Real(self.finite_difference.reuse_tolerance),
                );
            }
            HessianMode::Exact | HessianMode::GaussNewton | HessianMode::LimitedMemory => {}
        }
        Ok(options)
    }
}

#[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(
    remote = "pounce_feral::FeralConfig",
    default = "pounce_feral::FeralConfig::default",
    deny_unknown_fields
)]
#[schemars(rename = "FeralSettings")]
pub(super) struct FeralIdentity {
    bounded_storage_max_dimension: Option<usize>,
    cascade_break: Option<bool>,
    fma: bool,
    refine: bool,
    increase_quality: bool,
    refine_max_steps: usize,
    refine_target: f64,
    singular_pivot_floor: f64,
    inertia_pivot_floor: Option<f64>,
    pivtol: f64,
    #[serde(with = "ordering")]
    #[schemars(with = "FeralOrdering")]
    ordering: OrderingMethod,
    #[serde(with = "scaling")]
    #[schemars(with = "FeralScaling")]
    scaling: ScalingStrategy,
    parallel: Option<bool>,
    min_par_flops: Option<u64>,
    static_pivoting: Option<bool>,
}

/// FERAL's named orderings as their registry vocabulary. A caller-supplied permutation is
/// problem data of one KKT dimension, not a setting, so it has no settings encoding.
mod ordering {
    use super::{FeralOrdering, OrderingMethod};
    use serde::{Deserialize, Serialize, de, ser};

    pub(super) fn serialize<S: ser::Serializer>(
        value: &OrderingMethod,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let named = match value {
            OrderingMethod::Amd => FeralOrdering::Amd,
            OrderingMethod::Amf => FeralOrdering::Amf,
            OrderingMethod::MetisND => FeralOrdering::MetisNd,
            OrderingMethod::ScotchND => FeralOrdering::ScotchNd,
            OrderingMethod::KahipND => FeralOrdering::KahipNd,
            OrderingMethod::Auto => FeralOrdering::Auto,
            OrderingMethod::AutoRace => FeralOrdering::AutoRace,
            OrderingMethod::External(_) => {
                return Err(ser::Error::custom(
                    "an external FERAL permutation is problem data, not a setting",
                ));
            }
        };
        named.serialize(serializer)
    }
    pub(super) fn deserialize<'de, D: de::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<OrderingMethod, D::Error> {
        Ok(match FeralOrdering::deserialize(deserializer)? {
            FeralOrdering::Amd => OrderingMethod::Amd,
            FeralOrdering::Amf => OrderingMethod::Amf,
            FeralOrdering::MetisNd => OrderingMethod::MetisND,
            FeralOrdering::ScotchNd => OrderingMethod::ScotchND,
            FeralOrdering::KahipNd => OrderingMethod::KahipND,
            FeralOrdering::Auto => OrderingMethod::Auto,
            FeralOrdering::AutoRace => OrderingMethod::AutoRace,
        })
    }
}

/// FERAL's named scalings as their registry vocabulary; a caller-supplied scaling vector
/// is problem data, not a setting.
mod scaling {
    use super::{FeralScaling, ScalingStrategy};
    use serde::{Deserialize, Serialize, de, ser};

    pub(super) fn serialize<S: ser::Serializer>(
        value: &ScalingStrategy,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let named = match value {
            ScalingStrategy::InfNorm => FeralScaling::InfNorm,
            ScalingStrategy::Mc64Symmetric => FeralScaling::Mc64Symmetric,
            ScalingStrategy::Identity => FeralScaling::Identity,
            ScalingStrategy::Auto => FeralScaling::Auto,
            ScalingStrategy::External(_) => {
                return Err(ser::Error::custom(
                    "an external FERAL scaling vector is problem data, not a setting",
                ));
            }
        };
        named.serialize(serializer)
    }
    pub(super) fn deserialize<'de, D: de::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<ScalingStrategy, D::Error> {
        Ok(match FeralScaling::deserialize(deserializer)? {
            FeralScaling::InfNorm => ScalingStrategy::InfNorm,
            FeralScaling::Mc64Symmetric => ScalingStrategy::Mc64Symmetric,
            FeralScaling::Identity => ScalingStrategy::Identity,
            FeralScaling::Auto => ScalingStrategy::Auto,
        })
    }
}

/// The record of an effective FERAL configuration: its settings encoding (F30).
///
/// # Errors
/// A caller-supplied permutation or scaling vector, which has no settings encoding.
#[cfg_attr(
    not(feature = "pounce"),
    expect(
        dead_code,
        reason = "only the linked POUNCE adapter records its FERAL configuration"
    )
)]
pub(crate) fn record(config: &LinearSettings) -> Result<serde_json::Value, serde_json::Error> {
    FeralIdentity::serialize(config, serde_json::value::Serializer)
}

/// Upgrade check: an exhaustive destructuring fails to compile when FERAL adds a field
/// that the serde remote above does not frame.
const _: fn(&LinearSettings) = |c| {
    let pounce_feral::FeralConfig {
        bounded_storage_max_dimension: _,
        cascade_break: _,
        fma: _,
        refine: _,
        increase_quality: _,
        refine_max_steps: _,
        refine_target: _,
        singular_pivot_floor: _,
        inertia_pivot_floor: _,
        pivtol: _,
        ordering: _,
        scaling: _,
        parallel: _,
        min_par_flops: _,
        static_pivoting: _,
    } = c;
};
