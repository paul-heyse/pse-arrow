// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-owned study policy projections for native and WASI consumers.
use anyhow::Result;
use pse_codegen::codegen::{GeneratedTree, provenance};

pub(super) fn append(tree: &mut GeneratedTree) -> Result<()> {
    let graph = pse_model::study::native_projection_graph()?;
    let root = std::path::PathBuf::from("crates/pse-study-module/src/generated");
    let source = pse_codegen::codegen::boundary_projection::surreal::native_value_adapters(
        &graph,
        "crate::native",
    )?;
    tree.roots.push(root.clone());
    tree.files
        .insert(root.join("native.rs"), source.into_bytes());
    let mut relationships = Vec::new();
    for owner in graph.definitions().values() {
        for output in [
            "crates/pse-study-module/src/generated/native.rs",
            "crates/pse-operations/src/generated/surreal.rs",
            "crates/pse-operations/src/generated/surreal.surql",
        ] {
            relationships.push(provenance::SourceRelationship {
                declaration_key: owner.name.clone(),
                declaration_kind: "rust_projection".into(),
                source_path: Some(owner.source.into()),
                source_symbol: Some(owner.rust_type.into()),
                source_role: "semantic_owner".into(),
                source_identity: None,
                generator_path: "xtask/src/codegen/study_native.rs".into(),
                generator_symbol: "append".into(),
                output_path: output.into(),
                direction: Some(format!("{:?}", owner.direction)),
                contract_sections: vec!["blueprint §4.7".into()],
                basis: "declared".into(),
            });
        }
    }
    let sidecar = provenance::sidecar("study-native", &relationships)?;
    tree.roots.extend(sidecar.roots);
    tree.files.extend(sidecar.files);
    tree.relationships.extend(relationships);
    Ok(())
}
