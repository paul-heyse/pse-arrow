// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Typed retained expectation observations; no scalar substitutes for unavailable samples.
use super::declarations::{column, documented, relation, run_id};
use crate::{RegistryBuilder, model::{FieldContract as T, Namespace as N, SnapshotClass as S, ResultSelectionRole as R}};
use arrow_schema::DataType as D;
pub(super) fn declare(builder: &mut RegistryBuilder) {
    let ordinal = || T::nonnegative(i64::MAX);
    let real = || T::native(D::Float64);
    let flag = || T::native(D::Boolean);
    relation(builder, N::Runtime, "expectation_summaries", S::Derived, &["run_id", "parent_id"], vec![
        run_id().with_result_selection(R::Run), column("parent_id", T::id()).with_identity("study").with_result_selection(R::Owner),
        column("source_revision", T::hash()), column("physical_context", T::hash()),
        column("completed_iterations", ordinal()), column("recorded_iterations", ordinal()), column("issued", ordinal()),
        column("interrupted", flag()), column("limit_reached", flag()),
        column("failure_probability", real()).optional().with_result_selection(R::Value),
        column("failure_standard_error", real()).optional(), column("failure_probability_complete", flag()),
        documented("original_report", T::native(D::Utf8), "Versioned source-owned ExpectationReport observation: original request, admitted law and frozen typed estimates. Stored alongside the registered sample and estimate products; no sampler/evaluator serialization."),
    ], "Original terminal explicit-measure parent observation published through the ordinary run/attempt result manifest. Sampling uncertainty is not a physical enclosure.");
    let input = T::structure(vec![
        column("member", T::id()).with_boundary_tag(1), column("quantity", T::id()).with_identity("quantity_type").with_boundary_tag(2),
        column("canonical", real()).with_boundary_tag(3),
    ]).named("ExpectationPhysicalInput").with_boundary_shape(crate::model::BoundaryShape::native(file!(), 1, pse_contract::BoundaryDirection::Output));
    let output = T::structure(vec![column("member", T::id()).with_boundary_tag(1), column("canonical", real()).optional().with_boundary_tag(2)]).named("ExpectationOriginalOutput").with_boundary_shape(crate::model::BoundaryShape::native(file!(), 1, pse_contract::BoundaryDirection::Output));
    builder.declare_relation(super::declarations::declaration(N::Runtime, "expectation_samples", 1, S::Derived, &["run_id", "parent_id", "iteration", "ordinal"], vec![
        run_id().with_result_selection(R::Run).with_boundary_tag(1),
        column("parent_id", T::id()).with_identity("study").with_boundary_tag(2), column("child_id", T::id()).with_identity("study").with_result_selection(R::Owner).with_boundary_tag(3),
        column("iteration", ordinal()).with_boundary_tag(4), column("ordinal", ordinal()).with_boundary_tag(5), column("point_index", ordinal()).with_boundary_tag(6),
        column("lane", ordinal()).with_boundary_tag(7), column("lane_ordinal", ordinal()).with_boundary_tag(8),
        documented("target_mass", real(), "Product of explicit target category masses; continuous unit-cube target density is one.").with_boundary_tag(9),
        documented("proposal_weight", real(), "Inverse frozen proposal density/probability; applied once in the estimator, not in scientific outputs.").with_result_selection(R::Value).with_boundary_tag(10),
        column("categories", T::list(T::id())).with_boundary_tag(11), column("base_coordinates", T::list(real())).with_boundary_tag(12),
        column("inputs", T::list(input)).with_boundary_tag(13), column("outputs", T::list(output)).with_boundary_tag(14),
        column("scientific_failure", flag()).optional().with_boundary_tag(15), column("operational_incomplete", flag()).with_boundary_tag(16),
    ], "Original finite sample attribution and physical observations from a settled child. Equal values remain distinct occurrences. Missing values do not shrink the expectation denominator.").with_boundary_record(crate::model::BoundaryRecord { source:file!(), name:"RuntimeExpectationSamples", version:1, direction:pse_contract::BoundaryDirection::Output, reserved_tags:&[], reserved_names:&[] }));
    relation(builder, N::Runtime, "expectation_estimates", S::Derived, &["run_id", "parent_id", "output_id"], vec![
        run_id().with_result_selection(R::Run), column("parent_id", T::id()).with_identity("study"), column("output_id", T::id()).with_result_selection(R::Owner),
        column("quantity_id", T::id()).with_identity("quantity_type"), column("unit_id", T::id()).with_identity("unit"),
        column("completed_iterations", ordinal()), column("issued", ordinal()), column("available", ordinal()),
        column("unavailable_ordinals", T::list(ordinal())), column("reference", real()),
        column("estimate", real()).optional().with_result_selection(R::Value),
        documented("standard_error", real(), "Nominal Monte Carlo sampling standard error in the output's interval representation, never a physical enclosure.").optional(),
        column("complete", flag()), column("precision_met", flag()), column("interrupted", flag()), column("limit_reached", flag()),
    ], "Original terminal parent output estimates with physical meaning, actual coverage, precision and stopping status; a completed adaptation subset is not a completed parent expectation.");
}
