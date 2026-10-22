// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Storage-independent scientific product completeness.
use super::declarations::{column, relation};
use crate::{
    RegistryBuilder,
    model::{FieldContract as T, Namespace as N, SnapshotClass as S},
};
use std::collections::BTreeSet;

pub(super) fn declare(builder: &mut RegistryBuilder) {
    builder.declare_artifact_profile("relations", BTreeSet::new());
    relation(
        builder,
        N::Reference,
        "artifact_profiles",
        S::Model,
        &["kind"],
        vec![
            column("kind", T::native(arrow_schema::DataType::Utf8)),
            column("required_relations", T::list(T::id())),
        ],
        "Declared scientific product completeness, including selected empty relations.",
    );
}
pub(super) fn declare_profiles(builder: &mut RegistryBuilder) {
    let common = BTreeSet::from([
        "runtime.diagnostics_findings".to_owned(),
        "provenance.derivations".to_owned(),
    ]);
    builder.declare_artifact_profile(
        "run",
        BTreeSet::from([
            "runtime.run_lineage".to_owned(),
            "runtime.candidate_assessments".to_owned(),
            "runtime.modeling_checks".to_owned(),
        ]),
    );
    let mut source = common.clone();
    source.extend(
        builder
            .declared_relations()
            .iter()
            .filter(|relation| {
                matches!(
                    relation.authority,
                    crate::model::Authority::Authored | crate::model::Authority::Reference
                ) && relation.snapshot_class != S::Sidecar
            })
            .map(|relation| relation.key.qualified_name()),
    );
    builder.declare_artifact_profile("source", source.clone());
    builder.declare_artifact_profile("case", source);
    builder.declare_artifact_profile("diagnostics", common);
    builder.declare_artifact_profile("inspection", BTreeSet::new());
    let scientific = BTreeSet::from([
        "runtime.scientific_contexts".to_owned(),
        "runtime.candidate_assessments".to_owned(),
        "runtime.run_lineage".to_owned(),
        "runtime.resolved_numerics".to_owned(),
        "runtime.modeling_checks".to_owned(),
        "runtime.modeling_reports".to_owned(),
        "runtime.modeling_findings".to_owned(),
    ]);
    for (kind, relations) in [
        (
            "final-state",
            &[
                "runtime.trajectory_endpoints",
                "runtime.computation_runs",
                "runtime.solve_variables",
                "runtime.solve_runs",
            ][..],
        ),
        (
            "trajectory",
            &[
                "runtime.trajectory_endpoints",
                "runtime.simulation_samples",
                "runtime.simulation_events",
                "runtime.modeling_trajectory_modes",
                "runtime.response_sensitivities",
                "runtime.computation_runs",
            ][..],
        ),
        (
            "local-analysis",
            &[
                "runtime.route_decisions",
                "runtime.local_validity",
                "runtime.parametric_sensitivities",
                "runtime.response_sensitivities",
                "runtime.reduced_hessians",
                "runtime.parameter_covariances",
                "runtime.parameter_intervals",
                "runtime.profile_points",
                "runtime.response_directions",
                "runtime.propagated_covariances",
                "runtime.fit_parameters",
                "runtime.fit_observations",
                "runtime.solve_runs",
                "runtime.computation_runs",
                "runtime.solve_metrics",
            ][..],
        ),
    ] {
        let mut required = scientific.clone();
        required.extend(relations.iter().map(|name| (*name).to_owned()));
        builder.declare_artifact_profile(kind, required);
    }
}
