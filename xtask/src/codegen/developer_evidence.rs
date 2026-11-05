// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native developer observations; original files and performing authority stay in the host.

use anyhow::{Context, Result};
use pse_codegen::codegen::{GeneratedTree, boundary_projection, provenance};
use std::path::Path;

pub(super) fn generate(root: &Path) -> Result<GeneratedTree> {
    let imports = super::contract_metadata::imports(root)?;
    let graph = pse_host::test_inventory::developer_evidence_projection()?;
    let package = "pse.developer.evidence.v1";
    let destination = Path::new("crates/pse-host/src/generated/developer-evidence");
    let mut tree = super::proto_projection::projection(
        &graph,
        super::proto_projection::ProjectionTarget {
            package,
            file: "developer_evidence.proto",
            destination: "crates/pse-host/src/generated/developer-evidence",
            rust_module: "crate::developer_evidence_contracts::pb",
            document_root: super::DEVELOPER_EVIDENCE_DOCUMENT_ROOT,
            generator_path: "xtask/src/codegen/proto_projection.rs",
            generator_symbol: "projection",
        },
        &imports,
    )?;
    let dispatch = super::proto_dispatch::generate(
        &tree,
        destination,
        "crate::developer_evidence_contracts::pb",
    )?;
    super::proto_projection::insert(&mut tree, destination, "dispatch.rs", &dispatch)?;
    for owner in graph.definitions().values() {
        tree.relationships.push(provenance::SourceRelationship {
            declaration_key: owner.name.clone(),
            declaration_kind: "compiled_message_dispatch".into(),
            source_path: Some(owner.source.into()),
            source_symbol: Some(owner.rust_type.into()),
            source_role: "semantic_owner".into(),
            source_identity: None,
            generator_path: "xtask/src/codegen/proto_dispatch.rs".into(),
            generator_symbol: "generate".into(),
            output_path: destination
                .join("dispatch.rs")
                .to_string_lossy()
                .into_owned(),
            direction: Some(owner.direction.as_str().into()),
            contract_sections: vec!["blueprint §4.7".into()],
            basis: "declared".into(),
        });
    }
    let roots = [
        "CapturedWorkflowReport",
        "CapturedWorkflowEvidence",
        "CapturedWorkflowDirectoryManifest",
    ];
    let adapters = boundary_projection::native_adapters_scoped(
        &graph,
        &roots,
        "crate::developer_evidence_contracts::pb::pse::developer::evidence::v1",
        "pse_host",
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
            .context("developer observation root missing from its native graph")?;
        tree.relationships.push(provenance::SourceRelationship {
            declaration_key: name.into(),
            declaration_kind: "native_projection_adapters".into(),
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
    // The shared publisher already owns this exact provenance root. Refresh only
    // its contents after adding native adapter relationships.
    tree.files.extend(sidecar.files);
    Ok(tree)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn developer_bundle_uses_complete_native_roots_and_source_policy() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let tree = generate(root).unwrap();
        super::super::validate_tree(&tree).unwrap();
        assert_eq!(
            tree.roots
                .iter()
                .filter(|root| root.as_path()
                    == Path::new("docs/generated/source-relationships/pse-developer-evidence-v1"))
                .count(),
            1
        );
        let directory = Path::new("crates/pse-host/src/generated/developer-evidence");
        let policy = std::str::from_utf8(&tree.files[&directory.join("native-policy.rs")]).unwrap();
        assert!(policy.contains("INTERPRETATIONS"));
        assert!(policy.contains("CapturedWorkflowEvidence"));
        let adapters =
            std::str::from_utf8(&tree.files[&directory.join("native-adapters.rs")]).unwrap();
        for root in [
            "captured_workflow_report",
            "captured_workflow_evidence",
            "captured_workflow_directory_manifest",
        ] {
            assert!(adapters.contains(&format!("pub(crate) fn encode_{root}")));
            assert!(!adapters.contains(&format!("fn decode_{root}")));
        }
        assert!(policy.contains("CapturedWorkflowDirectoryManifest"));
        assert!(adapters.contains("NativeFrom<"));
        assert!(adapters.contains("unknown_fields(&wire.__buffa_unknown_fields"));
        assert!(!adapters.contains("fn decode_workflow_report_observation"));
        assert!(tree.files.contains_key(&directory.join("descriptor.bin")));
        assert!(
            tree.files
                .contains_key(&directory.join("validation/mod.rs"))
        );
    }
}
