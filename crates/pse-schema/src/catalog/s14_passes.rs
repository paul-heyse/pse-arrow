// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Closed Wave 1 pass declarations (blueprint §14.1, ADR-0056).

use std::collections::BTreeSet;

use crate::RegistryBuilder;
use crate::model::{
    Authority, Determinism, InputPort, Namespace, OutputPort, PassDecl, PortSource, RelationDecl,
    SnapshotClass,
};

/// Project complete ports from the authoritative relation and document inventories.
///
/// P0–P2 operate on unpublished candidates. P3 begins from an admitted model/case
/// revision and therefore pins its primitive inputs directly. P10 is available only
/// in a complete predecessor fixture registry, never by inventing P4–P9 producers.
#[expect(
    clippy::too_many_lines,
    reason = "one declaration projects the closed four-pass inventory"
)]
pub fn declare(builder: &mut RegistryBuilder) {
    let relations = builder.declared_relations().to_vec();
    let document_relations: BTreeSet<_> = builder
        .declared_documents()
        .iter()
        .flat_map(|document| document.sections.iter().map(|section| section.relation))
        .collect();
    let authored: Vec<_> = relations
        .iter()
        .filter(|relation| {
            relation.authority == Authority::Authored
                || document_relations.contains(relation.key.qualified_name().as_str())
        })
        .collect();
    let primitives: Vec<_> = relations
        .iter()
        .filter(|relation| {
            matches!(
                relation.authority,
                Authority::Authored | Authority::Reference
            )
        })
        .collect();
    let p0_inputs = primitives
        .iter()
        .filter(|relation| {
            relation.key.qualified_name() == "authored.packages"
                || relation.key.name.starts_with("schema_")
        })
        .map(|relation| input(relation, PortSource::Pinned))
        .collect();
    builder.declare_pass(
        PassDecl::new("P0", "1", Determinism::Deterministic)
            .inputs(p0_inputs)
            .outputs(vec![output_named(
                "package_graph",
                "normalized.package_graph",
            )])
            .diagnostics(vec!["authoring.reference", "validation.invariant"]),
    );

    let mut p1_inputs: Vec<_> = authored
        .iter()
        .map(|relation| input(relation, PortSource::Pinned))
        .collect();
    p1_inputs.push(InputPort {
        port: "package_graph",
        relation: "normalized.package_graph".to_owned(),
        source: PortSource::Derived {
            pass: "P0",
            port: "package_graph",
        },
        required: true,
    });
    builder.declare_pass(
        PassDecl::new("P1", "1", Determinism::Deterministic)
            .inputs(p1_inputs)
            .outputs(authored.iter().map(|relation| output(relation)).collect())
            .diagnostics(vec!["authoring.parse", "authoring.reference"]),
    );

    let conditions: Vec<_> = builder
        .declared_invariants()
        .iter()
        .filter(|invariant| {
            primitives
                .iter()
                .any(|relation| relation.key.qualified_name() == invariant.relation)
        })
        .map(|invariant| format!("{}:{}", invariant.relation, invariant.name))
        .collect();
    let p2_inputs = primitives
        .iter()
        .map(|relation| {
            let source = if authored.iter().any(|member| member.key == relation.key) {
                PortSource::Derived {
                    pass: "P1",
                    port: relation.key.name,
                }
            } else {
                PortSource::Pinned
            };
            input(relation, source)
        })
        .collect();
    builder.declare_pass(
        PassDecl::new("P2", "1", Determinism::Deterministic)
            .inputs(p2_inputs)
            .outputs(vec![
                output_named("findings", "runtime.diagnostics_findings"),
                output_named("undecided", "inferred.undecided"),
            ])
            .conditions(vec![], conditions.clone())
            .diagnostics(vec!["validation.invariant"])
            .executes_plans(),
    );
    builder.declare_pass(
        PassDecl::new("P3", "1", Determinism::Deterministic)
            .inputs(
                primitives
                    .iter()
                    .filter(|relation| relation.snapshot_class != SnapshotClass::Sidecar)
                    .map(|relation| input(relation, PortSource::Pinned))
                    .collect(),
            )
            .outputs(
                relations
                    .iter()
                    .filter(|relation| relation.key.namespace == Namespace::Normalized)
                    .map(output)
                    .collect(),
            )
            .conditions(
                conditions
                    .into_iter()
                    .filter(|condition| {
                        let relation_name = condition
                            .split_once(':')
                            .map_or(condition.as_str(), |(relation, _)| relation);
                        primitives.iter().any(|relation| {
                            relation.key.qualified_name() == relation_name
                                && relation.snapshot_class != SnapshotClass::Sidecar
                        })
                    })
                    .collect(),
                vec![],
            )
            .diagnostics(vec!["authoring.reference", "compile.math"]),
    );
}

fn input(relation: &RelationDecl, source: PortSource) -> InputPort {
    InputPort {
        port: relation.key.name,
        relation: relation.key.qualified_name(),
        source,
        required: true,
    }
}

fn output(relation: &RelationDecl) -> OutputPort {
    OutputPort {
        port: relation.key.name,
        relation: relation.key.qualified_name(),
    }
}

fn output_named(port: &'static str, relation: &str) -> OutputPort {
    OutputPort {
        port,
        relation: relation.to_owned(),
    }
}
