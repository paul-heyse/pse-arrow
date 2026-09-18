// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Source projection and normalization signatures. Actual dependencies are native plans.
use crate::{
    RegistryBuilder,
    model::{
        AlgorithmDecl, ArgumentSpec, Authority, Determinism, Namespace, ResultSpec, SnapshotClass,
    },
};

/// Declare source projection and normalization over explicit typed native arguments.
pub fn declare(builder: &mut RegistryBuilder) {
    let relations = builder.declared_relations().to_vec();
    let primitives: Vec<_> = relations
        .iter()
        .filter(|relation| {
            matches!(
                relation.authority,
                Authority::Authored | Authority::Reference
            ) && relation.snapshot_class != SnapshotClass::Sidecar
        })
        .collect();
    builder.declare_algorithm(
        AlgorithmDecl::new("source", "1", Determinism::Deterministic)
            .outputs(
                primitives
                    .iter()
                    .map(|relation| ResultSpec {
                        port: relation.key.name.into(),
                        relation: relation.key.qualified_name(),
                    })
                    .collect(),
            )
            .diagnostics(vec!["authoring.parse", "authoring.reference"]),
    );
    for (name, relation) in [
        ("source_edit", "authored.document_edits"),
        ("source_rename", "authored.rename_requests"),
    ] {
        builder.declare_algorithm(
            AlgorithmDecl::new(name, "1", Determinism::Deterministic)
                .inputs(vec![ArgumentSpec {
                    port: "request".into(),
                    relation: relation.into(),
                    required: true,
                    consumption: crate::model::algorithm::InputConsumption::Whole,
                }])
                .outputs(vec![ResultSpec {
                    port: "documents".into(),
                    relation: "authored.documents".into(),
                }])
                .diagnostics(vec!["authoring.reference", "authoring.parse"]),
        );
    }
    let conditions = builder
        .declared_invariants()
        .iter()
        .filter(|invariant| {
            primitives
                .iter()
                .any(|relation| relation.key.qualified_name() == invariant.relation)
        })
        .map(|invariant| format!("{}:{}", invariant.relation, invariant.name))
        .collect();
    builder.declare_algorithm(
        AlgorithmDecl::new("P3", "1", Determinism::Deterministic)
            .inputs(
                primitives
                    .iter()
                    .map(|relation| ArgumentSpec {
                        port: relation.key.name.into(),
                        relation: relation.key.qualified_name(),
                        required: true,
                        consumption: crate::model::algorithm::InputConsumption::Whole,
                    })
                    .collect(),
            )
            .outputs(
                relations
                    .iter()
                    .filter(|relation| {
                        relation.key.namespace == Namespace::Normalized
                            && relation.snapshot_class != SnapshotClass::Sidecar
                    })
                    .map(|relation| ResultSpec {
                        port: relation.key.name.into(),
                        relation: relation.key.qualified_name(),
                    })
                    .collect(),
            )
            .conditions(conditions, vec![])
            .diagnostics(vec!["authoring.reference", "compile.math"]),
    );
}
