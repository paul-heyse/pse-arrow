// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-forward reference contracts without runtime or scientific services.

use super::proto_projection::{ProjectionTarget, insert, producer, projection};
use anyhow::{Context, Result};
use buffa_descriptor::generated::descriptor::FileDescriptorSet;
use pse_codegen::codegen::{GeneratedTree, boundary_projection, provenance};
use pse_contract::boundary_projection::ProjectionGraph;
use std::path::Path;

pub(super) fn graph() -> Result<ProjectionGraph> {
    Ok(pse_model::reference_exchange::reference_projection()?)
}

pub(super) fn generate(imports: &FileDescriptorSet) -> Result<GeneratedTree> {
    let reference_graph = graph()?;
    let mut reference = projection(
        &reference_graph,
        ProjectionTarget {
            package: "pse.reference.v1",
            file: "reference.proto",
            destination: "crates/pse-runtime/src/generated/reference-exchange",
            rust_module: "crate::reference_exchange::generated::pb",
            document_root: super::ENGINEERING_DOCUMENT_ROOT,
            generator_path: "xtask/src/codegen/reference_exchange.rs",
            generator_symbol: "generate",
        },
        imports,
    )?;
    let reference_roots = [
        "ReferenceEnvelope",
        "ReferenceCaseContract",
        "ReferenceDegeneracyResult",
    ];
    let reference_adapters = boundary_projection::native_adapters_retaining_wire(
        &reference_graph,
        &reference_roots,
        "crate::reference_exchange::generated::pb::pse::reference::v1",
        "pse_runtime",
    )?;
    insert(
        &mut reference,
        Path::new("crates/pse-runtime/src/generated/reference-exchange"),
        "native-adapters.rs",
        reference_adapters.as_bytes(),
    )?;
    for name in reference_roots {
        let owner = reference_graph
            .definitions()
            .get(name)
            .context("reference adapter owner absent")?;
        reference
            .relationships
            .push(provenance::SourceRelationship {
                declaration_key: name.into(),
                declaration_kind: "native_projection_adapters".into(),
                source_path: Some(owner.source.into()),
                source_symbol: Some(owner.rust_type.into()),
                source_role: "typed_owner".into(),
                source_identity: None,
                generator_path: "crates/pse-codegen/src/codegen/boundary_projection/adapters.rs"
                    .into(),
                generator_symbol: "native_adapters_retaining_wire".into(),
                output_path:
                    "crates/pse-runtime/src/generated/reference-exchange/native-adapters.rs".into(),
                direction: Some(format!("{:?}", owner.direction)),
                contract_sections: vec!["blueprint §4.7".into()],
                basis: "declared".into(),
            });
    }
    reference.files.extend(
        provenance::sidecar(&producer("pse.reference.v1"), &reference.relationships)?.files,
    );
    Ok(reference)
}
