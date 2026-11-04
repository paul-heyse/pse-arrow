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
mod projection;
pub use projection::{OrderingProjection, ScalingProjection};
/// Actual FERAL ordering type, publicly nameable by downstream checked bindings.
pub type NativeOrdering = OrderingMethod;
/// Actual FERAL scaling type, publicly nameable by downstream checked bindings.
pub type NativeScaling = ScalingStrategy;

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

pse_contract::projection_struct! {
/// The POUNCE adapter's settings type. Its identity derives from serde.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
#[schemars(rename = "PounceSettings")]
pub struct Settings {
    projection: { name: "PounceSettings", version: 1, direction: pse_contract::boundary_projection::Direction::Both,
        reserved_tags: [], reserved_names: [] };
    /// NLP method.
    pub method: Method => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Native linear settings.
    #[serde(with = "FeralIdentity")]
    pub linear: LinearSettings => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [], adapter: FeralIdentity };
    /// Interior-point restart of a submitted primal-dual seed (L-N3).
    pub restart: WarmRestart => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Library-owned partitioned curvature settings.
    pub partitioned: PartitionedSettings => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Library-owned finite-difference curvature settings.
    pub finite_difference: FiniteDifferenceSettings => { tag: 5, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
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

pse_contract::projection_struct! {
/// Typed partitioned update controls. An absent formula preserves the library's
/// mode-dependent default and its option set-ness.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct PartitionedSettings {
    projection: { name: "PouncePartitionedSettings", version: 1, direction: pse_contract::boundary_projection::Direction::Both,
        reserved_tags: [], reserved_names: [] };
    /// Explicit element update, or the library default.
    pub update_type: Option<PouncePartitionedUpdate> => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Wider elements degrade to the library diagonal approximation.
    pub max_element: usize => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Element construction.
    pub elements: PouncePartitionedElements => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Width of primal blocks.
    pub block_size: usize => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Finite positive curvature multiplier, absent for uncapped native updates.
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub curvature_cap: Option<f64> => { tag: 5, direction: pse_contract::boundary_projection::Direction::Both,
        rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::PositiveFinite)] };
}
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
pse_contract::projection_struct! {
/// Typed finite-difference pattern, coloring and native reuse controls.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct FiniteDifferenceSettings {
    projection: { name: "PounceFiniteDifferenceSettings", version: 1, direction: pse_contract::boundary_projection::Direction::Both,
        reserved_tags: [], reserved_names: [] };
    /// Declared Hessian support, falling back to conservative Jacobian/objective support.
    pub pattern: PounceFdPattern => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Library coloring algorithm.
    pub coloring: PounceFdColoring => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Relative native iterate distance permitting Hessian reuse.
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub reuse_tolerance: f64 => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both,
        rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::NonnegativeFinite)] };
}
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
            || !pse_contract::boundary_projection::Float64Domain::NonnegativeFinite
                .contains(self.finite_difference.reuse_tolerance)
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

pse_contract::projection_remote! {
/// Complete original FERAL configuration binding, shared by Serde and typed projection.
#[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(
    remote = "pounce_feral::FeralConfig",
    default = "pounce_feral::FeralConfig::default",
    deny_unknown_fields
)]
#[schemars(rename = "FeralSettings")]
pub struct FeralIdentity for pounce_feral::FeralConfig {
    projection: { name: "FeralSettings", rust_path: crate::settings::pounce::FeralIdentity,
        target_path: crate::settings::pounce::LinearSettings, version: 1,
        direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    bounded_storage_max_dimension: Option<usize> => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    cascade_break: Option<bool> => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    fma: bool => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    refine: bool => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    increase_quality: bool => { tag: 5, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    refine_max_steps: usize => { tag: 6, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    refine_target: f64 => { tag: 7, direction: pse_contract::boundary_projection::Direction::Both,
        rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::Finite)] };
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    singular_pivot_floor: f64 => { tag: 8, direction: pse_contract::boundary_projection::Direction::Both,
        rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::Finite)] };
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    inertia_pivot_floor: Option<f64> => { tag: 9, direction: pse_contract::boundary_projection::Direction::Both,
        rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::Finite)] };
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pivtol: f64 => { tag: 10, direction: pse_contract::boundary_projection::Direction::Both,
        rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::Finite)] };
    #[serde(with = "ordering")]
    #[schemars(with = "FeralOrdering")]
    ordering: OrderingMethod => { tag: 11, direction: pse_contract::boundary_projection::Direction::Both, rules: [], adapter: OrderingProjection };
    #[serde(with = "scaling")]
    #[schemars(with = "FeralScaling")]
    scaling: ScalingStrategy => { tag: 12, direction: pse_contract::boundary_projection::Direction::Both, rules: [], adapter: ScalingProjection };
    parallel: Option<bool> => { tag: 13, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    min_par_flops: Option<u64> => { tag: 14, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    static_pivoting: Option<bool> => { tag: 15, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}

/// FERAL's named orderings as their registry vocabulary. A caller-supplied permutation is
/// problem data of one KKT dimension, not a setting, so it has no settings encoding.
mod ordering {
    use super::{FeralOrdering, OrderingMethod};
    use serde::{Deserialize, Serialize, de, ser};

    pub(super) fn named(value: &OrderingMethod) -> Result<FeralOrdering, &'static str> {
        Ok(match value {
            OrderingMethod::Amd => FeralOrdering::Amd,
            OrderingMethod::Amf => FeralOrdering::Amf,
            OrderingMethod::MetisND => FeralOrdering::MetisNd,
            OrderingMethod::ScotchND => FeralOrdering::ScotchNd,
            OrderingMethod::KahipND => FeralOrdering::KahipNd,
            OrderingMethod::Auto => FeralOrdering::Auto,
            OrderingMethod::AutoRace => FeralOrdering::AutoRace,
            OrderingMethod::External(_) => {
                return Err("an external FERAL permutation is problem data, not a setting");
            }
        })
    }
    pub(super) fn from_named(value: FeralOrdering) -> OrderingMethod {
        match value {
            FeralOrdering::Amd => OrderingMethod::Amd,
            FeralOrdering::Amf => OrderingMethod::Amf,
            FeralOrdering::MetisNd => OrderingMethod::MetisND,
            FeralOrdering::ScotchNd => OrderingMethod::ScotchND,
            FeralOrdering::KahipNd => OrderingMethod::KahipND,
            FeralOrdering::Auto => OrderingMethod::Auto,
            FeralOrdering::AutoRace => OrderingMethod::AutoRace,
        }
    }
    pub(super) fn serialize<S: ser::Serializer>(
        value: &OrderingMethod,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        named(value)
            .map_err(ser::Error::custom)?
            .serialize(serializer)
    }
    pub(super) fn deserialize<'de, D: de::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<OrderingMethod, D::Error> {
        FeralOrdering::deserialize(deserializer).map(from_named)
    }
}

/// FERAL's named scalings as their registry vocabulary; a caller-supplied scaling vector
/// is problem data, not a setting.
mod scaling {
    use super::{FeralScaling, ScalingStrategy};
    use serde::{Deserialize, Serialize, de, ser};

    pub(super) fn named(value: &ScalingStrategy) -> Result<FeralScaling, &'static str> {
        Ok(match value {
            ScalingStrategy::InfNorm => FeralScaling::InfNorm,
            ScalingStrategy::Mc64Symmetric => FeralScaling::Mc64Symmetric,
            ScalingStrategy::Identity => FeralScaling::Identity,
            ScalingStrategy::Auto => FeralScaling::Auto,
            ScalingStrategy::External(_) | ScalingStrategy::ExternalShared(_) => {
                return Err("an external FERAL scaling vector is problem data, not a setting");
            }
        })
    }
    pub(super) fn from_named(value: FeralScaling) -> ScalingStrategy {
        match value {
            FeralScaling::InfNorm => ScalingStrategy::InfNorm,
            FeralScaling::Mc64Symmetric => ScalingStrategy::Mc64Symmetric,
            FeralScaling::Identity => ScalingStrategy::Identity,
            FeralScaling::Auto => ScalingStrategy::Auto,
        }
    }
    pub(super) fn serialize<S: ser::Serializer>(
        value: &ScalingStrategy,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        named(value)
            .map_err(ser::Error::custom)?
            .serialize(serializer)
    }
    pub(super) fn deserialize<'de, D: de::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<ScalingStrategy, D::Error> {
        FeralScaling::deserialize(deserializer).map(from_named)
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

#[cfg(test)]
mod document_tests {
    use super::*;

    #[test]
    fn finite_setting_guards_preserve_original_field_schemas_and_valid_feral_bytes() {
        fn strip_annotations(value: &mut serde_json::Value) {
            match value {
                serde_json::Value::Object(object) => {
                    for name in ["$schema", "title", "description", "default"] {
                        object.remove(name);
                    }
                    for value in object.values_mut() {
                        strip_annotations(value);
                    }
                }
                serde_json::Value::Array(values) => {
                    for value in values {
                        strip_annotations(value);
                    }
                }
                _ => {}
            }
        }
        fn native_schema<T: schemars::JsonSchema>(_: &T) -> serde_json::Value {
            let mut schema = serde_json::to_value(schemars::schema_for!(T)).unwrap();
            strip_annotations(&mut schema);
            schema
        }
        let native = LinearSettings {
            bounded_storage_max_dimension: Some(7),
            cascade_break: Some(false),
            fma: false,
            refine: true,
            increase_quality: true,
            refine_max_steps: 3,
            refine_target: -0.0,
            singular_pivot_floor: 1.0,
            inertia_pivot_floor: None,
            pivtol: 0.125,
            ordering: OrderingMethod::Amd,
            scaling: ScalingStrategy::Identity,
            parallel: Some(false),
            min_par_flops: Some(9),
            static_pivoting: Some(true),
        };
        let mut bytes = Vec::new();
        FeralIdentity::serialize(&native, &mut serde_json::Serializer::new(&mut bytes)).unwrap();
        assert_eq!(bytes.as_slice(), br#"{"bounded_storage_max_dimension":7,"cascade_break":false,"fma":false,"refine":true,"increase_quality":true,"refine_max_steps":3,"refine_target":-0.0,"singular_pivot_floor":1.0,"inertia_pivot_floor":null,"pivtol":0.125,"ordering":"amd","scaling":"identity","parallel":false,"min_par_flops":9,"static_pivoting":true}"#);
        let mut feral = serde_json::to_value(schemars::schema_for!(FeralIdentity)).unwrap();
        strip_annotations(&mut feral);
        for (field, expected) in [
            ("refine_target", native_schema(&native.refine_target)),
            (
                "singular_pivot_floor",
                native_schema(&native.singular_pivot_floor),
            ),
            (
                "inertia_pivot_floor",
                native_schema(&native.inertia_pivot_floor),
            ),
            ("pivtol", native_schema(&native.pivtol)),
        ] {
            assert_eq!(
                feral["properties"][field], expected,
                "original native schema for {field}"
            );
        }
        let mut partitioned =
            serde_json::to_value(schemars::schema_for!(PartitionedSettings)).unwrap();
        strip_annotations(&mut partitioned);
        assert_eq!(
            partitioned["properties"]["curvature_cap"],
            native_schema(&PartitionedSettings::default().curvature_cap)
        );
        let mut difference =
            serde_json::to_value(schemars::schema_for!(FiniteDifferenceSettings)).unwrap();
        strip_annotations(&mut difference);
        assert_eq!(
            difference["properties"]["reuse_tolerance"],
            native_schema(&FiniteDifferenceSettings::default().reuse_tolerance)
        );
        let mut invalid = native;
        invalid.refine_target = f64::INFINITY;
        assert!(
            FeralIdentity::serialize(&invalid, &mut serde_json::Serializer::new(Vec::new()))
                .is_err()
        );
    }
}
