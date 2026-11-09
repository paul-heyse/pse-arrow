// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Projections of compiled source owners. No authored second wire schema.

use super::{
    contract_metadata::imports,
    proto_projection::{ProjectionTarget, insert, producer},
};
use anyhow::{Context, Result, bail};
use pse_codegen::codegen::{GeneratedTree, boundary_projection, provenance};
use pse_contract::boundary_projection::{ProjectionGraph, TypeRef};
use std::{collections::BTreeSet, path::Path};

mod selection;
use selection::{CONTROL, EXPANDED, EXPLICIT, INSPECTION, MAIN, PACKAGE, SHARED};

const SHARED_MODULE: &str = "crate::engineering_types::pb";
const SHARED_DESTINATION: &str = "crates/pse-host/src/generated/engineering-shared";
const HOST_MODULE: &str = "crate::engineering_client::pb";
const HOST_DESTINATION: &str = "crates/pse-host/src/generated/engineering-explicit";
const INSPECTION_MODULE: &str = "crate::host_operations::engineering::inspection::pb";
const INSPECTION_DESTINATION: &str = "crates/pse-runtime/src/generated/engineering-inspection";
const INSPECTION_PRODUCER: &str = "pse-engineering-inspection";
const INSPECTION_ROOT: &str = "ModelingInspection";
const BROAD_MODULE: &str = "crate::host_operations::engineering::generated::broad";
const BROAD_DESTINATION: &str = "crates/pse-runtime/src/generated/engineering";

pub(super) fn generate(root: &Path) -> Result<GeneratedTree> {
    // These are pinned upstream option definitions, not the host service's meanings.
    let imports = imports(root)?;
    let graph = engineering_graph()?;
    let recorded_imports = super::recorded_result::engineering_imports(&graph, &imports)?;
    let source = boundary_projection::descriptors_with_imports(
        &graph, PACKAGE, MAIN, &recorded_imports.file,
    )?;
    let source = selection::partition(source)?;
    let mut tree = compile_selection(
        &graph,
        &selection::world(&source, &[SHARED])?,
        &[SHARED],
        SHARED_MODULE,
        SHARED_DESTINATION,
        "docs/generated/engineering/shared",
        "pse-engineering-shared",
        &[],
    )?;
    let explicit = compile_selection(
        &graph,
        &selection::world(&source, &[EXPLICIT, CONTROL])?,
        &[EXPLICIT, CONTROL],
        HOST_MODULE,
        HOST_DESTINATION,
        "docs/generated/engineering/explicit",
        "pse-engineering-explicit",
        &external_types(&source, SHARED, SHARED_MODULE)?,
    )?;
    super::proto_projection::merge_independent(&mut tree, explicit)?;

    let shared_externs = external_types(
        &source, SHARED, "::pse_host::engineering_types::pb",
    )?;
    let mut inspection = compile_selection(
        &graph,
        &selection::world(&source, &[INSPECTION])?,
        &[INSPECTION],
        INSPECTION_MODULE,
        INSPECTION_DESTINATION,
        "docs/generated/engineering/inspection",
        INSPECTION_PRODUCER,
        &shared_externs,
    )?;
    publish_adapters(&graph, &mut inspection, AdapterOutput {
        roots: &[INSPECTION_ROOT],
        imports: &[],
        wire: "crate::host_operations::engineering::inspection::types",
        destination: INSPECTION_DESTINATION,
        producer: INSPECTION_PRODUCER,
    })?;
    super::proto_projection::merge_independent(&mut tree, inspection)?;

    let mut config = boundary_projection::default_config(&format!("{BROAD_MODULE}::__buffa_fds"));
    config
        .extern_paths
        .extend(super::recorded_result::extern_paths()?);
    config.extern_paths.extend(shared_externs);
    config.extern_paths.extend(external_types(&source, EXPLICIT, "::pse_host::engineering_client::pb")?);
    config.extern_paths.extend(external_types(&source, INSPECTION, INSPECTION_MODULE)?);
    let output = generation_output(&graph, &source, &[MAIN, EXPANDED], &config)?;
    let mut engineering = super::proto_projection::publish_projection(
        &graph,
        ProjectionTarget {
            package: PACKAGE,
            file: MAIN,
            destination: BROAD_DESTINATION,
            rust_module: BROAD_MODULE,
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
        "CanonicalAttemptMetadata",
        "EngineeringFailure",
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
    adapter_roots.remove(INSPECTION_ROOT);
    let adapter_roots = adapter_roots.into_iter().collect::<Vec<_>>();
    publish_adapters(&graph, &mut engineering, AdapterOutput {
        roots: &adapter_roots,
        imports: &[INSPECTION_ROOT],
        wire: "crate::host_operations::engineering::generated::pb::pse::engineering::v1",
        destination: BROAD_DESTINATION,
        producer: &producer(PACKAGE),
    })?;
    super::proto_projection::merge_independent(&mut tree, engineering)?;
    Ok(tree)
}

struct AdapterOutput<'a> {
    roots: &'a [&'a str],
    imports: &'a [&'a str],
    wire: &'a str,
    destination: &'a str,
    producer: &'a str,
}

fn publish_adapters(graph: &ProjectionGraph, tree: &mut GeneratedTree, output: AdapterOutput<'_>) -> Result<()> {
    let AdapterOutput { roots, imports, wire, destination, producer } = output;
    let adapters = boundary_projection::native_adapters_with_imports(
        graph,
        roots,
        imports,
        wire,
        "pse_runtime",
        "crate::host_operations::engineering::NativeAdapterScope",
        boundary_projection::AdapterVisibility::Public,
    )?;
    let destination = Path::new(destination);
    let filename = "native-adapters.rs";
    insert(tree, destination, filename, adapters.as_bytes())?;
    let output_path = destination.join(filename).to_string_lossy().into_owned();
    for name in roots {
        let owner = graph.definitions().get(*name).context("native adapter owner absent from its graph")?;
        tree.relationships.push(provenance::SourceRelationship {
            declaration_key: owner.name.clone(),
            declaration_kind: "native_projection_adapter".into(),
            source_path: Some(owner.source.into()),
            source_symbol: Some(owner.rust_type.into()),
            source_role: "adapter_root".into(),
            source_identity: None,
            generator_path: "crates/pse-codegen/src/codegen/boundary_projection/adapters.rs".into(),
            generator_symbol: "native_adapters_with_imports".into(),
            output_path: output_path.clone(),
            direction: Some(format!("{:?}", owner.direction)),
            contract_sections: vec!["blueprint §4.7".into()],
            basis: "declared".into(),
        });
    }
    tree.files.extend(provenance::sidecar(producer, &tree.relationships)?.files);
    Ok(())
}

fn external_types(
    source: &buffa_codegen::generated::descriptor::FileDescriptorSet,
    file_name: &str,
    module: &str,
) -> Result<Vec<(String, String)>> {
    let file = source.file.iter()
        .find(|file| file.name.as_deref() == Some(file_name))
        .with_context(|| format!("missing engineering file {file_name}"))?;
    file.message_type.iter().map(|message| {
        let name = message.name.as_deref().context("unnamed engineering message")?;
        Ok((format!(".{PACKAGE}.{name}"), format!("{module}::pse::engineering::v1::{name}")))
    }).collect()
}

fn generation_output(
    graph: &ProjectionGraph,
    source: &buffa_codegen::generated::descriptor::FileDescriptorSet,
    targets: &[&str],
    config: &buffa_codegen::CodeGenConfig,
) -> Result<boundary_projection::ProjectionOutput> {
    let targets = targets.iter().map(|name| (*name).to_owned()).collect::<Vec<_>>();
    let generation = boundary_projection::ProjectionGeneration::new(source, &targets, config)?;
    let support = generation.support("descriptor.bin")?;
    Ok(boundary_projection::ProjectionOutput {
        generation,
        descriptor_set: support.descriptor_set,
        descriptor_bytes: support.descriptor_bytes,
        source_descriptor_bytes: support.source_descriptor_bytes,
        support_sources: support.sources,
        documentation: boundary_projection::owner_reference(graph, PACKAGE),
    })
}

#[allow(clippy::too_many_arguments, reason = "publication names are explicit and each scope has one output owner")]
fn compile_selection(
    graph: &ProjectionGraph,
    source: &buffa_codegen::generated::descriptor::FileDescriptorSet,
    targets: &[&str],
    rust_module: &str,
    destination: &str,
    document_root: &str,
    producer: &str,
    extern_paths: &[(String, String)],
) -> Result<GeneratedTree> {
    let owned = source.file.iter().filter(|file| {
        file.name.as_deref().is_some_and(|name| targets.contains(&name))
    }).collect::<Vec<_>>();
    let owners = owned.iter().flat_map(|file| &file.message_type)
        .filter_map(|message| message.name.as_deref()).collect();
    let services = owned.iter().flat_map(|file| &file.service)
        .filter_map(|service| service.name.as_deref()).collect();
    let selected = graph.publication_scope(&owners, &services);
    let mut config = boundary_projection::default_config(&format!("{rust_module}::__buffa_fds"));
    config.extern_paths.extend_from_slice(extern_paths);
    let output = generation_output(&selected, source, targets, &config)?;
    super::proto_projection::publish_projection_owned(
        &selected,
        ProjectionTarget {
            package: PACKAGE,
            file: targets.first().copied().context("selected engineering files missing")?,
            destination,
            rust_module,
            document_root,
            generator_path: "xtask/src/codegen/engineering.rs",
            generator_symbol: "compile_selection",
        },
        output,
        producer,
    )
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

#[cfg(test)]
mod tests {
    use super::*;
    use buffa::Message as _;
    use buffa_descriptor::generated::descriptor::FileDescriptorSet;
    use buffa_descriptor::{DescriptorPool, DynamicMessage, Value};
    use std::sync::Arc;

    #[test]
    fn selected_study_and_inspection_worlds_preserve_one_message_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let imports = imports(root).unwrap();
        let graph = engineering_graph().unwrap();
        let imports = super::super::recorded_result::engineering_imports(&graph, &imports).unwrap();
        let original = boundary_projection::descriptors_with_imports(&graph, PACKAGE, MAIN, &imports.file).unwrap();
        let source = selection::partition(original).unwrap();
        let explicit = selection::world(&source, &[EXPLICIT, CONTROL]).unwrap();
        let inspection = selection::world(&source, &[INSPECTION]).unwrap();
        assert!(inspection.file.iter().all(|file| !matches!(file.name.as_deref(), Some(EXPLICIT | CONTROL))));
        let shared = selection::world(&source, &[SHARED]).unwrap();
        let shared = DescriptorPool::new(shared).unwrap();
        let explicit = DescriptorPool::new(explicit).unwrap();
        let inspection = DescriptorPool::new(inspection).unwrap();
        let full = DescriptorPool::new(source).unwrap();

        for name in ["EngineeringPrepareExplicitStudy", "ExplicitStudyDefinition", "EngineeringStudyStatus"] {
            let qualified = format!("{PACKAGE}.{name}");
            assert!(explicit.message_by_name(&qualified).is_some(), "{qualified}");
            assert!(full.message_by_name(&qualified).is_some(), "{qualified}");
        }
        for name in ["SourceDelta", "ExpandedStudyDefinition", "EngineeringEditSource", "ModelingInspection"] {
            let qualified = format!("{PACKAGE}.{name}");
            assert!(explicit.message_by_name(&qualified).is_none(), "{qualified}");
            assert!(full.message_by_name(&qualified).is_some(), "{qualified}");
        }
        assert!(inspection.message_by_name("pse.engineering.v1.ModelingInspection").is_some());
        assert!(inspection.message_by_name("pse.engineering.v1.ExpandedStudyDefinition").is_none());
        let source = "pse.engineering.v1.OperationSource";
        assert!(shared.message_by_name(source).is_some());
        assert!(inspection.message_by_name(source).is_some());
        assert!(explicit.message_by_name(source).is_some());
        assert!(shared.message_by_name("pse.engineering.v1.StudyPoint").is_none());
        let binding = full.message_by_name("pse.engineering.v1.ModelingIndexBinding").unwrap();
        assert_eq!(binding.fields().iter().map(|field| (field.name(), field.number())).collect::<Vec<_>>(),
            [("name", 1), ("domain", 2)]);
    }

    #[test]
    fn changing_source_inspection_rederives_only_its_narrow_world() {
        use buffa_codegen::generated::descriptor::{FieldDescriptorProto, field_descriptor_proto::{Label, Type}};

        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let graph = engineering_graph().unwrap();
        let imports = imports(root).unwrap();
        let imports = super::super::recorded_result::engineering_imports(&graph, &imports).unwrap();
        let source = boundary_projection::descriptors_with_imports(&graph, PACKAGE, MAIN, &imports.file).unwrap();
        let before = selection::partition(source.clone()).unwrap();
        let changed = |name| {
            let mut source = source.clone();
            let file = source.file.iter_mut().find(|file| file.name.as_deref() == Some(MAIN)).unwrap();
            let message = file.message_type.iter_mut().find(|message| message.name.as_deref() == Some(name)).unwrap();
            message.field.push(FieldDescriptorProto {
                name: Some("new_observation".into()),
                number: Some(123),
                r#type: Some(Type::TYPE_STRING),
                label: Some(Label::LABEL_OPTIONAL),
                ..Default::default()
            });
            selection::partition(source).unwrap()
        };
        let after = changed(INSPECTION_ROOT);
        let file = |world: &buffa_codegen::generated::descriptor::FileDescriptorSet, name| {
            world.file.iter().find(|file| file.name.as_deref() == Some(name)).unwrap().clone()
        };
        assert_eq!(file(&before, MAIN), file(&after, MAIN));
        assert_eq!(selection::world(&before, &[EXPLICIT, CONTROL]).unwrap(), selection::world(&after, &[EXPLICIT, CONTROL]).unwrap());
        assert_ne!(file(&before, INSPECTION), file(&after, INSPECTION));
        let narrow = DescriptorPool::new(selection::world(&after, &[INSPECTION]).unwrap()).unwrap();
        assert!(narrow.message_by_name("pse.engineering.v1.ModelingInspection").unwrap().fields().iter().any(|field| field.number() == 123));

        let service_only = changed("EngineeringAdmitSource");
        assert_ne!(file(&before, MAIN), file(&service_only, MAIN));
        assert_eq!(selection::world(&before, &[INSPECTION]).unwrap(), selection::world(&service_only, &[INSPECTION]).unwrap());
        assert_eq!(selection::world(&before, &[EXPLICIT, CONTROL]).unwrap(), selection::world(&service_only, &[EXPLICIT, CONTROL]).unwrap());
    }

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

    #[cfg(all(feature = "scientific-worker-contracts", feature = "reference-contracts"))]
    #[test]
    fn all_selected_world_metadata_has_typed_native_coverage() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let imports = imports(root).unwrap();
        let mut failures = Vec::new();
        for (package, file, graph) in [
            (
                "pse.scientific_worker.v2",
                "scientific_worker.proto",
                super::super::scientific_worker::graph().unwrap(),
            ),
            (
                "pse.engineering.v1",
                "engineering.proto",
                engineering_graph().unwrap(),
            ),
            (
                "pse.reference.v1",
                "reference.proto",
                super::super::reference_exchange::graph().unwrap(),
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
