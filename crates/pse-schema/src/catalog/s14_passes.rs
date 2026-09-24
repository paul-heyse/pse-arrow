// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual authored-source projection signature.
use crate::{
    RegistryBuilder,
    model::{
        AlgorithmDecl, Authority, Determinism, ResultSpec, SnapshotClass,
    },
};

/// Declare source projection over explicit typed native arguments.
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
}
