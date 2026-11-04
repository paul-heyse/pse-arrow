// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
// Actual source declarations; original native methods and admission remain beside the owner.

pub(crate) const RELATIVE_PUSH_DOMAIN: pse_model::boundary_projection::Float64Domain =
    pse_model::boundary_projection::Float64Domain::Interval(
        pse_model::boundary_projection::Float64Interval::new(0.0, false, 0.5, true),
    );

pse_model::projection_enum! {
    /// Primitive native option values, preserving the original untagged document.
    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
    #[serde(untagged)]
    pub enum OptionValue {
        projection: { name: "NativeOptionValue", rust_path: crate::solve::OptionValue, version: 1,
            direction: pse_model::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
        /// Native Boolean.
        Bool(bool) => 1;
        /// Native integer.
        Integer(i32) => 2;
        /// Finite native real; nonfinite observations use the separate evidence owner.
        Real(#[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")] f64 =>
            { rules: [pse_model::boundary_projection::LocalRule::Float64Domain(pse_model::boundary_projection::Float64Domain::Finite)] }) => 3;
        /// Native text/enumeration.
        Text(String) => 4;
    }
}

pse_model::projection_struct! {
/// Finite shared attempt controls chosen by the caller; accuracy stays with resolved policy.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
#[schemars(rename = "SolveControls")]
pub struct Controls {
    projection: { name: "SolveControls", rust_path: crate::solve::Controls, version: 1,
        direction: pse_model::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Positive wall-clock allowance, including callbacks, in seconds.
    #[serde(with = "seconds")]
    #[schemars(with = "Tolerance")]
    pub time_limit: Duration => { tag: 1, direction: pse_model::boundary_projection::Direction::Both, rules: [] };
    /// Positive native iteration limit.
    pub iterations: u32 => { tag: 2, direction: pse_model::boundary_projection::Direction::Both, rules: [pse_model::boundary_projection::LocalRule::UnsignedRange { min: 1, max: i32::MAX as u64 }] };
    /// Explicit admitted native thread count.
    pub threads: usize => { tag: 3, direction: pse_model::boundary_projection::Direction::Both, rules: [pse_model::boundary_projection::LocalRule::UnsignedRange { min: 1, max: i32::MAX as u64 }] };
    /// Bounded retained progress and failure events.
    pub history: usize => { tag: 4, direction: pse_model::boundary_projection::Direction::Both, rules: [pse_model::boundary_projection::LocalRule::UnsignedRange { min: 0, max: 1_000_000 }] };
    /// Hessian representation for compatible NLP methods.
    pub hessian: HessianMode => { tag: 5, direction: pse_model::boundary_projection::Direction::Both, rules: [] };
    /// Native allocation/data reuse policy.
    pub reuse: ReusePolicy => { tag: 6, direction: pse_model::boundary_projection::Direction::Both, rules: [] };
    /// Numerical start policy, independent of native allocation reuse.
    pub start: StartPolicy => { tag: 7, direction: pse_model::boundary_projection::Direction::Both, rules: [] };
    /// Additional native options, admitted by the selected adapter.
    pub options: Options => { tag: 8, direction: pse_model::boundary_projection::Direction::Both, rules: [pse_model::boundary_projection::LocalRule::Length { min: 0, max: 4096 }] };
    /// Positive foreign-library allowance of this solve, in bytes: its reservation charges
    /// it, and a library that enforces its own memory limit receives it. Absent, the
    /// deployment's allowance applies (`MathPolicy.foreign_bytes`); an absent allowance is
    /// not encoded, so controls without one keep their identity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub foreign_bytes: Option<usize> => { tag: 9, direction: pse_model::boundary_projection::Direction::Both, rules: [pse_model::boundary_projection::LocalRule::UnsignedRange { min: 1, max: usize::MAX as u64 }] };
}
}
