// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-forward worker projection with no dependency on runtime transport.

use super::proto_projection::{ProjectionTarget, insert, producer, projection};
use anyhow::{Context, Result};
use buffa_descriptor::generated::descriptor::FileDescriptorSet;
use pse_codegen::codegen::{GeneratedTree, boundary_projection, provenance};
use pse_contract::boundary_projection::{ProjectionGraph, Service};
use pse_kernels::worker_protocol;
use std::path::Path;

pub(super) fn generate(imports: &FileDescriptorSet) -> Result<GeneratedTree> {
    let graph = graph()?;
    let mut tree = projection(
        &graph,
        ProjectionTarget {
            package: "pse.scientific_worker.v1",
            file: "scientific_worker.proto",
            destination: "crates/pse-runtime/src/generated/scientific-worker",
            rust_module: "crate::scientific_worker::generated::pb",
            document_root: super::ENGINEERING_DOCUMENT_ROOT,
            generator_path: "xtask/src/codegen/scientific_worker.rs",
            generator_symbol: "generate",
        },
        imports,
    )?;
    let scientific_roots = ["Request", "Response"];
    let adapters = boundary_projection::native_adapters_scoped(
        &graph,
        &scientific_roots,
        "crate::scientific_worker::generated::pb::pse::scientific_worker::v1",
        "pse_runtime",
        boundary_projection::AdapterVisibility::Parent,
    )?;
    insert(
        &mut tree,
        Path::new("crates/pse-runtime/src/generated/scientific-worker"),
        "native-adapters.rs",
        adapters.as_bytes(),
    )?;
    for name in scientific_roots {
        let owner = graph
            .definitions()
            .get(name)
            .context("scientific adapter owner absent")?;
        tree.relationships.push(provenance::SourceRelationship {
            declaration_key: name.into(),
            declaration_kind: "native_projection_adapter".into(),
            source_path: Some(owner.source.into()),
            source_symbol: Some(owner.rust_type.into()),
            source_role: "adapter_root".into(),
            source_identity: None,
            generator_path: "crates/pse-codegen/src/codegen/boundary_projection/adapters.rs".into(),
            generator_symbol: "native_adapters_scoped".into(),
            output_path: "crates/pse-runtime/src/generated/scientific-worker/native-adapters.rs"
                .into(),
            direction: Some(format!("{:?}", owner.direction)),
            contract_sections: vec!["blueprint §4.7".into()],
            basis: "declared".into(),
        });
    }
    tree.files.extend(
        provenance::sidecar(&producer("pse.scientific_worker.v1"), &tree.relationships)?.files,
    );
    Ok(tree)
}

pub(super) fn graph() -> Result<ProjectionGraph> {
    let mut graph = ProjectionGraph::of::<worker_protocol::Request>()?;
    graph.register::<worker_protocol::Response>()?;
    graph.register::<pse_kernels::portable::ProviderRecord>()?;
    let service = Service::new("ScientificWorker")
        .unary::<worker_protocol::Request, worker_protocol::Response>(&mut graph, "Exchange")?
        .effects(worker_protocol::EXCHANGE_EFFECTS)?;
    graph.service(service)?;
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scientific_worker_bootstrap_projects_portable_owners_and_actual_service() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let tree = generate(&super::super::contract_metadata::imports(root).unwrap()).unwrap();
        let destination = Path::new("crates/pse-runtime/src/generated/scientific-worker");
        let adapters =
            std::str::from_utf8(&tree.files[&destination.join("native-adapters.rs")]).unwrap();
        assert!(adapters.contains("pse_kernels::worker_protocol::Request"));
        assert!(adapters.contains("pse_kernels::worker_protocol::Response"));
        assert!(!adapters.contains("crate::scientific_worker::protocol"));
        assert!(tree.files.contains_key(&destination.join("descriptor.bin")));
        assert!(
            tree.files
                .contains_key(&destination.join("validation/mod.rs"))
        );
        assert!(
            tree.files
                .contains_key(&destination.join("services/pse.scientific_worker.v1.mod.rs"))
        );
    }
}
