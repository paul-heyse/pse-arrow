// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-owned native child protocol over the existing inherited channel.
use anyhow::{Context, Result};
use pse_codegen::codegen::{GeneratedTree, boundary_projection, provenance};
use std::path::Path;

pub(super) fn generate(root: &Path) -> Result<GeneratedTree> {
    super::code_intelligence::prepared_plugins(root, &root.join(".local/plan61-tools/bin"))?;
    let graph = pse_runtime::math::native_root_protocol::projection_graph()?;
    let imports = super::contract_metadata::imports(root)?;
    let destination = Path::new("crates/pse-runtime/src/generated/native-root");
    let package = "pse.native_root.v2";
    let mut tree = super::proto_projection::projection(
        root,
        &graph,
        super::proto_projection::ProjectionTarget {
            package,
            file: "native_root.proto",
            destination: "crates/pse-runtime/src/generated/native-root",
            rust_module: "crate::math::native_root_generated::pb",
            document_root: super::ENGINEERING_DOCUMENT_ROOT,
            generator_path: "xtask/src/codegen/proto_projection.rs",
            generator_symbol: "projection",
        },
        &imports,
    )?;
    let roots = [
        "NativeRootInput",
        "NativeRootHello",
        "NativeRootRequest",
        "NativeRootResponse",
    ];
    let adapters = boundary_projection::native_adapters_scoped(
        &graph,
        &roots,
        "crate::math::native_root_generated::pb::pse::native_root::v2",
        "pse_runtime",
        boundary_projection::AdapterVisibility::Crate,
    )?;
    super::proto_projection::insert(
        &mut tree,
        destination,
        "native-adapters.rs",
        adapters.as_bytes(),
    )?;
    for name in roots {
        let owner = graph
            .definitions()
            .get(name)
            .context("native child protocol root missing from its source graph")?;
        tree.relationships.push(provenance::SourceRelationship {
            declaration_key: name.into(),
            declaration_kind: "native_projection_adapter".into(),
            source_path: Some(owner.source.into()),
            source_symbol: Some(owner.rust_type.into()),
            source_role: "typed_owner".into(),
            source_identity: None,
            generator_path: "crates/pse-codegen/src/codegen/boundary_projection/adapters.rs".into(),
            generator_symbol: "native_adapters_scoped".into(),
            output_path: destination
                .join("native-adapters.rs")
                .to_string_lossy()
                .into_owned(),
            direction: Some(owner.direction.as_str().into()),
            contract_sections: vec!["blueprint §4.7".into()],
            basis: "declared".into(),
        });
    }
    let sidecar = provenance::sidecar(
        &super::proto_projection::producer(package),
        &tree.relationships,
    )?;
    // The common publisher already declares this producer's sidecar root.
    tree.files.extend(sidecar.files);
    Ok(tree)
}
