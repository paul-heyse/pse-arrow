// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Projections of compiled source owners. No authored second wire schema.

use anyhow::{Context, Result, bail};
use buffa::Message as _;
use buffa_descriptor::generated::{
    compiler::{CodeGeneratorRequest, CodeGeneratorResponse},
    descriptor::FileDescriptorSet,
};
use pse_codegen::codegen::{GeneratedTree, boundary_projection, provenance};
use pse_contract::boundary_projection::{ProjectionGraph, Service, TypeRef};
use std::{
    collections::BTreeSet,
    io::Write as _,
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
};

pub(super) fn generate(root: &Path) -> Result<GeneratedTree> {
    super::code_intelligence::prepared_plugins(root, &root.join(".local/plan61-tools/bin"))?;
    // These are pinned upstream option definitions, not the host service's meanings.
    let imports = imports(root)?;
    let mut graph = ProjectionGraph::of::<pse_runtime::scientific_worker::protocol::Request>()?;
    graph.register::<pse_runtime::scientific_worker::protocol::Response>()?;
    let service = Service::new("ScientificWorker").unary::<
        pse_runtime::scientific_worker::protocol::Request,
        pse_runtime::scientific_worker::protocol::Response,
    >(&mut graph, "Exchange")?
    .effects(pse_runtime::scientific_worker::protocol::EXCHANGE_EFFECTS)?;
    graph.service(service)?;
    let mut tree = projection(
        root,
        &graph,
        "pse.scientific_worker.v1",
        "scientific_worker.proto",
        "crates/pse-runtime/src/generated/scientific-worker",
        "crate::scientific_worker::generated::pb",
        &imports,
    )?;
    let graph = pse_runtime::host_operations::engineering_contract::projection()?;
    let mut graph = graph;
    graph.register::<pse_runtime::workflow::ModelingDeclarationRequest>()?;
    graph.register::<pse_runtime::workflow::PreparationDiscoveryReport>()?;
    graph.register::<pse_runtime::workflow::PhysicalInspectionRequest>()?;
    graph.register::<pse_runtime::workflow::PhysicalInspectionDocument>()?;
    let (_, effects) =
        pse_runtime::host_operations::project::ProjectAction::preparation_discovery_projection(
            &mut graph,
        )?;
    let mut engineering = projection(
        root,
        &graph,
        "pse.engineering.v1",
        "engineering.proto",
        "crates/pse-runtime/src/generated/engineering",
        "crate::host_operations::engineering::generated::pb",
        &imports,
    )?;
    let mut adapter_roots = BTreeSet::from([
        "PreparationDiscoveryRequest",
        "PreparationDiscoveryReport",
        "StructuralValue",
        "OriginalSourceFileRequest",
        "OriginalSourceFile",
        "PhysicalInspectionRequest",
        "PhysicalInspectionDocument",
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
    let adapters = boundary_projection::native_adapters(
        &graph,
        &adapter_roots,
        "crate::host_operations::engineering::generated::pb::pse::engineering::v1",
        "pse_runtime",
    )?;
    insert(
        &mut engineering,
        Path::new("crates/pse-runtime/src/generated/engineering"),
        "native-adapters.rs",
        adapters.as_bytes(),
    )?;
    for name in adapter_roots {
        let owner = graph
            .definitions()
            .get(name)
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
                generator_path: "crates/pse-codegen/src/codegen/boundary_projection/adapters.rs"
                    .into(),
                generator_symbol: "native_adapters".into(),
                output_path: "crates/pse-runtime/src/generated/engineering/native-adapters.rs"
                    .into(),
                direction: Some(format!("{:?}", owner.direction)),
                contract_sections: vec!["blueprint §4.7".into()],
                basis: "declared".into(),
            });
    }
    engineering.files.insert(
        "crates/pse-runtime/src/generated/engineering/action-effects.json".into(),
        serde_json::to_vec_pretty(&effects)?,
    );
    engineering
        .relationships
        .push(provenance::SourceRelationship {
            declaration_key: "ProjectAction::PreparationDiscovery".into(),
            declaration_kind: "operation_effects".into(),
            source_path: Some("crates/pse-runtime/src/host_operations/project.rs".into()),
            source_symbol: Some("ProjectAction::preparation_discovery_projection".into()),
            source_role: "effect_owner".into(),
            source_identity: None,
            generator_path: "xtask/src/codegen/engineering.rs".into(),
            generator_symbol: "generate".into(),
            output_path: "crates/pse-runtime/src/generated/engineering/action-effects.json".into(),
            direction: Some("Input".into()),
            contract_sections: vec!["blueprint §4.7".into()],
            basis: "declared".into(),
        });
    engineering.files.extend(
        provenance::sidecar(&producer("pse.engineering.v1"), &engineering.relationships)?.files,
    );
    tree.roots.extend(engineering.roots);
    tree.files.extend(engineering.files);
    tree.relationships.extend(engineering.relationships);
    let mut reference_graph = pse_model::reference_exchange::reference_projection()?;
    reference_graph.register::<pse_runtime::reference_exchange::ReferenceCaseContract>()?;
    let mut reference = projection(
        root,
        &reference_graph,
        "pse.reference.v1",
        "reference.proto",
        "crates/pse-runtime/src/generated/reference-exchange",
        "crate::reference_exchange::generated::pb",
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
    tree.roots.extend(reference.roots);
    tree.files.extend(reference.files);
    tree.relationships.extend(reference.relationships);
    Ok(tree)
}

fn imports(root: &Path) -> Result<FileDescriptorSet> {
    let scratch = tempfile::tempdir()?;
    let path = scratch.path().join("imports.bin");
    let output = Command::new("buf")
        .args(["build", "--as-file-descriptor-set", "-o"])
        .arg(&path)
        .current_dir(root.join("crates/pse-host/proto"))
        .output()?;
    if !output.status.success() {
        bail!(
            "compile pinned validation imports: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let mut set = FileDescriptorSet::decode_from_slice(&std::fs::read(path)?)?;
    set.file.retain(|file| {
        file.name
            .as_deref()
            .is_some_and(|name| !name.starts_with("pse/"))
    });
    Ok(set)
}

fn projection(
    root: &Path,
    graph: &ProjectionGraph,
    package: &str,
    file: &str,
    destination: &str,
    rust_module: &str,
    imports: &FileDescriptorSet,
) -> Result<GeneratedTree> {
    let config = boundary_projection::default_config(&format!("{rust_module}::__buffa_fds"));
    let output =
        boundary_projection::generate_with_config(graph, package, file, &imports.file, &config)?;
    for warning in &output.warnings {
        eprintln!("{package}: {warning}");
    }
    let destination = PathBuf::from(destination);
    let mut tree = GeneratedTree::empty(vec![destination.clone()]);
    for (name, source) in output.sources {
        insert(&mut tree, &destination, &name, source.as_bytes())?;
    }
    tree.files.insert(
        destination.join("descriptor.bin"),
        output.descriptor_bytes.clone(),
    );
    let descriptor = tempfile::NamedTempFile::new()?;
    std::fs::write(descriptor.path(), &output.descriptor_bytes)?;
    let preflight =
        Command::new(root.join(".local/plan61-tools/bin/protoc-gen-protovalidate-buffa"))
            .arg("--preflight")
            .arg(descriptor.path())
            .arg("--file")
            .arg(file)
            .output()?;
    if !preflight.status.success() || preflight.stdout != b"pse-cel-preflight-v1\n" {
        bail!(
            "{package} CEL preflight refused: {}",
            String::from_utf8_lossy(&preflight.stderr)
        );
    }
    for (plugin, subdirectory, parameter) in [
        (
            "protoc-gen-connect-rust",
            "services",
            format!("extern_path=.={rust_module}"),
        ),
        (
            "protoc-gen-protovalidate-buffa",
            "validation",
            format!("proto_module={rust_module}"),
        ),
    ] {
        let request = CodeGeneratorRequest {
            file_to_generate: vec![file.into()],
            proto_file: output.descriptor_set.file.clone(),
            parameter: Some(parameter),
            ..Default::default()
        };
        let response = plugin_response(root, plugin, &request)?;
        for generated in response.file {
            if generated.insertion_point.is_some() {
                bail!("{plugin} returned unsupported insertion point");
            }
            let name = generated.name.context("plugin output name absent")?;
            insert(
                &mut tree,
                &destination.join(subdirectory),
                &name,
                generated
                    .content
                    .context("plugin output content absent")?
                    .as_bytes(),
            )?;
        }
    }
    let docs = PathBuf::from(super::ENGINEERING_DOCUMENT_ROOT).join(package);
    tree.files.insert(
        docs.join("index.md"),
        format!(
            "<!-- @generated by pse-schema; do not edit -->\n{}",
            output.documentation
        )
        .into_bytes(),
    );
    tree.roots.push(docs);
    for definition in graph.definitions().values() {
        for path in tree.files.keys() {
            tree.relationships.push(provenance::SourceRelationship {
                declaration_key: definition.name.clone(),
                declaration_kind: "rust_projection".into(),
                source_path: Some(definition.source.into()),
                source_symbol: Some(definition.rust_type.into()),
                source_role: "semantic_owner".into(),
                source_identity: None,
                generator_path: "xtask/src/codegen/engineering.rs".into(),
                generator_symbol: "projection".into(),
                output_path: path.to_string_lossy().into_owned(),
                direction: Some(format!("{:?}", definition.direction)),
                contract_sections: vec!["blueprint §4.7".into()],
                basis: "declared".into(),
            });
        }
    }
    let sidecar = provenance::sidecar(&producer(package), &tree.relationships)?;
    tree.roots.extend(sidecar.roots);
    tree.files.extend(sidecar.files);
    Ok(tree)
}

fn producer(package: &str) -> String {
    package.replace(['.', '_'], "-")
}

fn plugin_response(
    root: &Path,
    name: &str,
    request: &CodeGeneratorRequest,
) -> Result<CodeGeneratorResponse> {
    let mut child = Command::new(root.join(".local/plan61-tools/bin").join(name))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let bytes = request.encode_to_vec();
    let mut stdin = child.stdin.take().context("plugin input unavailable")?;
    // Drain output concurrently: a generated response can exceed pipe capacity.
    let writer = std::thread::spawn(move || stdin.write_all(&bytes));
    let output = child.wait_with_output()?;
    writer
        .join()
        .map_err(|_| anyhow::anyhow!("plugin writer panicked"))??;
    if !output.status.success() {
        bail!("{name}: {}", String::from_utf8_lossy(&output.stderr));
    }
    let response = CodeGeneratorResponse::decode_from_slice(&output.stdout)?;
    if let Some(error) = &response.error {
        if !error.is_empty() {
            bail!("{name}: {error}");
        }
    }
    Ok(response)
}

fn insert(tree: &mut GeneratedTree, destination: &Path, name: &str, bytes: &[u8]) -> Result<()> {
    let relative = Path::new(name);
    if relative
        .components()
        .any(|part| !matches!(part, Component::Normal(_)))
    {
        bail!("unsafe generated output: {name}");
    }
    let path = destination.join(relative);
    let bytes = if path.extension().is_some_and(|extension| extension == "rs") {
        super::code_intelligence::qualify_generated_rust(std::str::from_utf8(bytes)?)?.into_bytes()
    } else {
        bytes.to_vec()
    };
    if tree.files.insert(path, bytes).is_some() {
        bail!("duplicate generated output: {name}");
    }
    Ok(())
}
