// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Projections of compiled source owners. No authored second wire schema.

use super::{
    contract_metadata::imports,
    proto_projection::{ProjectionTarget, insert, producer, projection},
};
use anyhow::{Context, Result, bail};
use pse_codegen::codegen::{GeneratedTree, boundary_projection, provenance};
use pse_contract::boundary_projection::{ProjectionGraph, Service, TypeRef};
use std::{collections::BTreeSet, path::Path};

pub(super) fn generate(root: &Path) -> Result<GeneratedTree> {
    // These are pinned upstream option definitions, not the host service's meanings.
    let imports = imports(root)?;
    let graph = scientific_graph()?;
    let mut tree = projection(
        &graph,
        ProjectionTarget {
            package: "pse.scientific_worker.v1",
            file: "scientific_worker.proto",
            destination: "crates/pse-runtime/src/generated/scientific-worker",
            rust_module: "crate::scientific_worker::generated::pb",
            document_root: super::ENGINEERING_DOCUMENT_ROOT,
            generator_path: "xtask/src/codegen/proto_projection.rs",
            generator_symbol: "projection",
        },
        &imports,
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
    let graph = engineering_graph()?;
    let recorded_imports = super::recorded_result::engineering_imports(&graph, &imports)?;
    let mut config = boundary_projection::default_config(
        "crate::host_operations::engineering::generated::pb::__buffa_fds",
    );
    config
        .extern_paths
        .extend(super::recorded_result::extern_paths()?);
    let output = boundary_projection::generate_with_dependencies(
        &graph,
        "pse.engineering.v1",
        "engineering.proto",
        &recorded_imports.file,
        &[super::recorded_result::FILE],
        &config,
    )?;
    let mut engineering = super::proto_projection::publish_projection(
        &graph,
        ProjectionTarget {
            package: "pse.engineering.v1",
            file: "engineering.proto",
            destination: "crates/pse-runtime/src/generated/engineering",
            rust_module: "crate::host_operations::engineering::generated::pb",
            document_root: super::ENGINEERING_DOCUMENT_ROOT,
            generator_path: "xtask/src/codegen/proto_projection.rs",
            generator_symbol: "projection",
        },
        output,
    )?;
    let mut adapter_roots = BTreeSet::from([
        "PreparationDiscoveryRequest",
        "PreparationDiscoveryReport",
        "StructuralValue",
        "OriginalSourceFileRequest",
        "OriginalSourceFile",
        "PhysicalInspectionRequest",
        "PhysicalInspectionDocument",
        "CanonicalResultManifest",
        "CanonicalResultBlockMetadata",
        "CanonicalResultSetMetadata",
        "CanonicalResultBatchMetadata",
        "CanonicalResultOutputMetadata",
        "CanonicalResultSetDescriptor",
        "CanonicalProtectedResultSnapshot",
        "ScientificPermissionSnapshot",
        "ScientificProductPermissionSnapshot",
        "ScientificMemberPermissionSnapshot",
    ]);
    for service in graph.services().values() {
        for method in &service.methods {
            for owner in [&method.input, &method.output] {
                let TypeRef::Named(name) = owner else {
                    bail!(
                        "{}.{}: native service adapter requires its named source owner",
                        service.name,
                        method.name
                    );
                };
                adapter_roots.insert(name.as_str());
            }
        }
    }
    let adapter_roots = adapter_roots.into_iter().collect::<Vec<_>>();
    for (roots, filename, visibility) in [(
        adapter_roots.as_slice(),
        "native-adapters.rs",
        boundary_projection::AdapterVisibility::Public,
    )] {
        let adapters = boundary_projection::native_adapters_scoped(
            &graph,
            roots,
            "crate::host_operations::engineering::generated::pb::pse::engineering::v1",
            "pse_runtime",
            visibility,
        )?;
        let destination = Path::new("crates/pse-runtime/src/generated/engineering");
        insert(&mut engineering, destination, filename, adapters.as_bytes())?;
        for name in roots {
            let owner = graph
                .definitions()
                .get(*name)
                .context("native adapter owner absent from its graph")?;
            engineering
                .relationships
                .push(provenance::SourceRelationship {
                    declaration_key: owner.name.clone(),
                    declaration_kind: "native_projection_adapter".into(),
                    source_path: Some(owner.source.into()),
                    source_symbol: Some(owner.rust_type.into()),
                    source_role: "adapter_root".into(),
                    source_identity: None,
                    generator_path:
                        "crates/pse-codegen/src/codegen/boundary_projection/adapters.rs".into(),
                    generator_symbol: "native_adapters_scoped".into(),
                    output_path: destination.join(filename).to_string_lossy().into_owned(),
                    direction: Some(format!("{:?}", owner.direction)),
                    contract_sections: vec!["blueprint §4.7".into()],
                    basis: "declared".into(),
                });
        }
    }
    engineering.files.extend(
        provenance::sidecar(&producer("pse.engineering.v1"), &engineering.relationships)?.files,
    );
    super::proto_projection::merge_independent(&mut tree, engineering)?;
    let reference_graph = reference_graph()?;
    let mut reference = projection(
        &reference_graph,
        ProjectionTarget {
            package: "pse.reference.v1",
            file: "reference.proto",
            destination: "crates/pse-runtime/src/generated/reference-exchange",
            rust_module: "crate::reference_exchange::generated::pb",
            document_root: super::ENGINEERING_DOCUMENT_ROOT,
            generator_path: "xtask/src/codegen/proto_projection.rs",
            generator_symbol: "projection",
        },
        &imports,
    )?;
    let reference_roots = ["ReferenceEnvelope", "ReferenceCaseContract"];
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
    super::proto_projection::merge_independent(&mut tree, reference)?;
    Ok(tree)
}
fn scientific_graph() -> Result<ProjectionGraph> {
    let mut graph = ProjectionGraph::of::<pse_runtime::scientific_worker::protocol::Request>()?;
    graph.register::<pse_runtime::scientific_worker::protocol::Response>()?;
    graph.register::<pse_kernels::portable::ProviderRecord>()?;
    let service = Service::new("ScientificWorker").unary::<
        pse_runtime::scientific_worker::protocol::Request,
        pse_runtime::scientific_worker::protocol::Response,
    >(&mut graph, "Exchange")?.effects(pse_runtime::scientific_worker::protocol::EXCHANGE_EFFECTS)?;
    graph.service(service)?;
    Ok(graph)
}

fn engineering_graph() -> Result<ProjectionGraph> {
    let mut graph = pse_runtime::host_operations::engineering_contract::projection()?;
    graph.register::<pse_model::generated::runtime::canonical_result_manifests::Row>()?;
    graph.register::<pse_model::generated::runtime::canonical_result_blocks::Row>()?;
    graph.register::<pse_model::generated::runtime::canonical_result_sets::Row>()?;
    graph.register::<pse_model::generated::runtime::canonical_result_batches::Row>()?;
    graph.register::<pse_model::generated::runtime::canonical_result_block_outputs::Row>()?;
    graph.register::<pse_operations::canonical_results::ResultRead>()?;
    graph.register::<pse_runtime::workflow::ModelingDeclarationRequest>()?;
    graph.register::<pse_runtime::workflow::PreparationDiscoveryReport>()?;
    graph.register::<pse_runtime::workflow::PhysicalInspectionRequest>()?;
    graph.register::<pse_runtime::workflow::PhysicalInspectionDocument>()?;
    pse_runtime::host_operations::project::ProjectAction::preparation_discovery_projection(
        &mut graph,
    )?;
    Ok(graph)
}

fn reference_graph() -> Result<ProjectionGraph> {
    let mut graph = pse_model::reference_exchange::reference_projection()?;
    graph.register::<pse_runtime::reference_exchange::ReferenceCaseContract>()?;
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use buffa::Message as _;
    use buffa_descriptor::generated::descriptor::FileDescriptorSet;
    use buffa_descriptor::{DescriptorPool, DynamicMessage, Value};
    use std::sync::Arc;

    #[test]
    fn protected_result_snapshot_exports_facts_without_an_inverse_or_live_custody() {
        use pse_contract::boundary_projection::{ConstructionRole, Shape};
        let graph = engineering_graph().unwrap();
        let definition = &graph.definitions()["CanonicalProtectedResultSnapshot"];
        assert_eq!(definition.construction, ConstructionRole::ObservationOnly);
        let Shape::Record(fields) = &definition.shape else {
            panic!("snapshot must be a record")
        };
        assert_eq!(
            fields
                .iter()
                .map(|field| field.name.as_str())
                .collect::<Vec<_>>(),
            ["run", "attempt", "manifest", "sets"]
        );
        let adapters = boundary_projection::native_adapters(
            &graph,
            &["CanonicalProtectedResultSnapshot"],
            "crate::pb",
            "pse_runtime",
        )
        .unwrap();
        assert!(adapters.contains("fn encode_canonical_protected_result_snapshot"));
        assert!(!adapters.contains("fn decode_canonical_protected_result_snapshot"));
        assert!(!adapters.contains("ReadOwner"));
    }

    #[test]
    fn recorded_result_selection_keeps_original_context_without_native_adoption() {
        use pse_contract::boundary_projection::{ConstructionRole, Direction, Shape};
        let graph = super::super::recorded_result::graph().unwrap();
        for owner in [
            "RecordedResultSelection",
            "RecordedResultPredicate",
            "RecordedResultContext",
        ] {
            let definition = &graph.definitions()[owner];
            assert_eq!(definition.direction, Direction::Output);
            assert_eq!(definition.construction, ConstructionRole::ObservationOnly);
        }
        let Shape::Record(fields) = &graph.definitions()["RecordedResultSelection"].shape else {
            panic!("recorded selection must be a record")
        };
        assert_eq!(
            fields
                .iter()
                .find(|field| field.name == "context")
                .unwrap()
                .tag,
            11
        );
        let adapters = boundary_projection::native_adapters_scoped(
            &graph,
            &["RecordedResultSelection"],
            "crate::pb",
            "pse_runtime",
            boundary_projection::AdapterVisibility::Crate,
        )
        .unwrap();
        assert!(adapters.contains("pub(crate) fn encode_recorded_result_selection"));
        for name in [
            "recorded_result_selection",
            "recorded_result_predicate",
            "recorded_result_context",
            "recorded_scientific_usability",
        ] {
            assert!(!adapters.contains(&format!("fn decode_{name}")));
        }
    }

    #[test]
    fn recorded_result_adapter_has_one_generated_owner_outside_service_adoption() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let passive = super::super::recorded_result::generate(root).unwrap();
        let tree = generate(root).unwrap();
        let path = Path::new(super::super::recorded_result::DESTINATION).join("native-adapters.rs");
        let adapter = std::str::from_utf8(&passive.files[&path]).unwrap();
        assert!(adapter.contains("pub(crate) fn encode_recorded_result_selection"));
        assert!(!adapter.contains("fn decode_recorded_"));
        assert!(!tree.files.contains_key(&path));
        let service = std::str::from_utf8(
            &tree.files
                [Path::new("crates/pse-runtime/src/generated/engineering/native-adapters.rs")],
        )
        .unwrap();
        assert!(!service.contains("RecordedResultSelection"));
        assert!(!tree.files.contains_key(Path::new(
            "crates/pse-runtime/src/generated/engineering/recorded-result-adapters.rs"
        )));
    }

    #[test]
    fn all_selected_world_metadata_has_typed_native_coverage() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let imports = imports(root).unwrap();
        let mut failures = Vec::new();
        for (package, file, graph) in [
            (
                "pse.scientific_worker.v1",
                "scientific_worker.proto",
                scientific_graph().unwrap(),
            ),
            (
                "pse.engineering.v1",
                "engineering.proto",
                engineering_graph().unwrap(),
            ),
            (
                "pse.reference.v1",
                "reference.proto",
                reference_graph().unwrap(),
            ),
        ] {
            if let Err(error) =
                boundary_projection::descriptors_with_imports(&graph, package, file, &imports.file)
            {
                failures.push(format!("{package}: {error}"));
            }
        }
        assert!(failures.is_empty(), "{failures:?}");
    }

    #[test]
    fn original_action_metadata_retains_version_two_observation_separate_from_rpc() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let mut graph = ProjectionGraph::default();
        pse_runtime::host_operations::project::ProjectAction::preparation_discovery_projection(
            &mut graph,
        )
        .unwrap();
        graph.register::<pse_runtime::host_operations::engineering_contract::EngineeringDiscoverPreparation>().unwrap();
        graph
            .register::<pse_runtime::workflow::PreparationDiscoveryReport>()
            .unwrap();
        let native = pse_runtime::host_operations::engineering_contract::projection().unwrap();
        let method = native
            .services()
            .values()
            .flat_map(|service| &service.methods)
            .find(|method| method.name == "DiscoverPreparation")
            .unwrap()
            .clone();
        graph
            .service(Service {
                name: "Engineering".into(),
                methods: vec![method],
            })
            .unwrap();
        let set = boundary_projection::descriptors_with_imports(
            &graph,
            "pse.engineering.v1",
            "engineering.proto",
            &imports(root).unwrap().file,
        )
        .unwrap();
        let set = FileDescriptorSet::decode_from_slice(&set.encode_to_vec()).unwrap();
        let pool = Arc::new(DescriptorPool::new(set.clone()).unwrap());
        let action = pool
            .message_by_name("pse.engineering.v1.PreparationDiscoveryAction")
            .unwrap();
        let options =
            DynamicMessage::from_options(Arc::clone(&pool), action.options().unwrap()).unwrap();
        let extension = pool.extension_by_name("pse.contract.v1.message").unwrap();
        let Value::Message(metadata) = options.field_by_number(extension.field().number()).unwrap()
        else {
            panic!("typed message metadata");
        };
        assert_eq!(metadata.field_by_number(3), Some(&Value::U32(2)));
        let Value::Message(effects) = metadata.field_by_number(6).unwrap() else {
            panic!("typed source action effects");
        };
        assert_eq!(effects.field_by_number(1), Some(&Value::EnumNumber(1)));
        assert_eq!(effects.field_by_number(2), Some(&Value::Bool(true)));
        assert_eq!(effects.field_by_number(3), Some(&Value::Bool(true)));
        assert_eq!(effects.field_by_number(4), Some(&Value::Bool(false)));
        let file = set
            .file
            .iter()
            .find(|file| file.name.as_deref() == Some("engineering.proto"))
            .unwrap();
        let method = file
            .service
            .iter()
            .flat_map(|service| &service.method)
            .find(|method| method.name.as_deref() == Some("DiscoverPreparation"))
            .unwrap();
        let options =
            DynamicMessage::from_options(Arc::clone(&pool), method.options.as_option().unwrap())
                .unwrap();
        let extension = pool.extension_by_name("pse.contract.v1.method").unwrap();
        assert!(matches!(
            options.field_by_number(extension.field().number()),
            Some(Value::Message(_))
        ));
    }
}
