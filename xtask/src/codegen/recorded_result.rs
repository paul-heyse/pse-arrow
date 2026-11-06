// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-forward passive contract generation, independent of runtime compilation.
use anyhow::Result;
#[cfg(any(feature = "product-schemas", test))]
use anyhow::ensure;
use buffa_codegen::generated::descriptor::FileDescriptorSet;
use pse_codegen::codegen::{GeneratedTree, boundary_projection, provenance};
use pse_contract::boundary_projection::ProjectionGraph;
use std::path::Path;

pub(super) const PACKAGE: &str = "pse.engineering.v1";
pub(super) const FILE: &str = "recorded_result.proto";
pub(super) const DESTINATION: &str = "crates/pse-runtime/src/generated/recorded-result";
pub(super) const MODULE: &str = "crate::recorded_result_projection::pb";
const PRODUCER: &str = "pse-recorded-result-v1";

pub(super) fn graph() -> Result<ProjectionGraph> {
    Ok(ProjectionGraph::of::<
        pse_model::recorded_result::PackResultSelection,
    >()?)
}

/// Always lower current model declarations; never read an earlier published descriptor.
pub(super) fn output(imports: &FileDescriptorSet) -> Result<boundary_projection::ProjectionOutput> {
    let graph = graph()?;
    let config = boundary_projection::default_config(&format!("{MODULE}::__buffa_fds"));
    Ok(boundary_projection::generate_with_config(
        &graph,
        PACKAGE,
        FILE,
        &imports.file,
        &config,
    )?)
}

/// Admit the complete exact file closure and its disjoint native ownership before suppliers run.
#[cfg(any(feature = "product-schemas", test))]
pub(super) fn engineering_imports(
    local: &ProjectionGraph,
    imports: &FileDescriptorSet,
) -> Result<FileDescriptorSet> {
    let foreign = graph()?;
    for name in foreign.definitions().keys() {
        ensure!(
            !local.definitions().contains_key(name),
            "recorded-result owner {name} also registered by engineering"
        );
    }
    let fresh = output(imports)?;
    let source = fresh.generation.descriptors();
    ensure!(
        source
            .file
            .iter()
            .filter(|file| file.name.as_deref() == Some(FILE))
            .count()
            == 1,
        "recorded-result source must own exactly one {FILE}"
    );
    for original in &imports.file {
        ensure!(
            source.file.iter().any(|file| file == original),
            "recorded-result generation changed original import {:?}",
            original.name
        );
    }
    Ok(source.clone())
}

/// Exact type mappings preserve the selected package while excluding imported PB ownership.
#[cfg(any(feature = "product-schemas", test))]
pub(super) fn extern_paths() -> Result<Vec<(String, String)>> {
    Ok(graph()?
        .definitions()
        .keys()
        .map(|name| {
            (
                format!(".{PACKAGE}.{name}"),
                format!("{MODULE}::pse::engineering::v1::{name}"),
            )
        })
        .collect())
}

pub(super) fn generate(root: &Path) -> Result<GeneratedTree> {
    let imports = super::contract_metadata::imports(root)?;
    let graph = graph()?;
    let output = output(&imports)?;
    let mut tree = super::proto_projection::publish_projection_owned(
        &graph,
        super::proto_projection::ProjectionTarget {
            package: PACKAGE,
            file: FILE,
            destination: DESTINATION,
            rust_module: MODULE,
            document_root: super::RECORDED_RESULT_DOCUMENT_ROOT,
            generator_path: "xtask/src/codegen/recorded_result.rs",
            generator_symbol: "generate",
        },
        output,
        PRODUCER,
    )?;
    let source = boundary_projection::native_adapters_scoped(
        &graph,
        &["RecordedResultSelection"],
        &format!("{MODULE}::pse::engineering::v1"),
        "pse_runtime",
        boundary_projection::AdapterVisibility::Crate,
    )?;
    super::proto_projection::insert(
        &mut tree,
        Path::new(DESTINATION),
        "native-adapters.rs",
        source.as_bytes(),
    )?;
    let owner = &graph.definitions()["RecordedResultSelection"];
    tree.relationships.push(provenance::SourceRelationship {
        declaration_key: owner.name.clone(),
        declaration_kind: "native_projection_adapter".into(),
        source_path: Some(owner.source.into()),
        source_symbol: Some(owner.rust_type.into()),
        source_role: "adapter_root".into(),
        source_identity: None,
        generator_path: "crates/pse-codegen/src/codegen/boundary_projection/adapters.rs".into(),
        generator_symbol: "native_adapters_scoped".into(),
        output_path: format!("{DESTINATION}/native-adapters.rs"),
        direction: Some(format!("{:?}", owner.direction)),
        contract_sections: vec!["blueprint §4.7".into()],
        basis: "declared".into(),
    });
    tree.files
        .extend(provenance::sidecar(PRODUCER, &tree.relationships)?.files);
    Ok(tree)
}

#[cfg(test)]
mod tests {
    use super::*;
    use buffa_codegen::SharedCorpusContext;

    fn root() -> &'static Path {
        Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap()
    }

    #[test]
    fn recorded_result_model_owns_exactly_four_passive_native_declarations() {
        let graph = graph().unwrap();
        assert_eq!(graph.definitions().len(), 4);
        for definition in graph.definitions().values() {
            assert!(
                definition
                    .rust_type
                    .starts_with("pse_model::recorded_result::")
            );
            assert_eq!(definition.version, 1);
            assert_eq!(
                definition.direction,
                pse_contract::BoundaryDirection::Output
            );
        }
        let tree = generate(root()).unwrap();
        let adapters =
            std::str::from_utf8(&tree.files[&Path::new(DESTINATION).join("native-adapters.rs")])
                .unwrap();
        assert!(adapters.contains("pse_model::recorded_result::PackResultSelection"));
        assert!(!adapters.contains("fn decode_recorded_"));
    }

    #[test]
    fn recorded_result_publication_survives_the_enclosing_markdown_writer() {
        let tree = generate(root()).unwrap();
        let checkout = tempfile::tempdir().unwrap();
        super::super::write_tree(checkout.path(), &tree).unwrap();
        let markdown = GeneratedTree::empty(pse_codegen::codegen::Language::Markdown.roots());
        super::super::write_tree(checkout.path(), &markdown).unwrap();
        // Publication retains every output, including nested documentation and
        // producer sidecars, when the broad documentation owner prunes stale files.
        for (path, expected) in &tree.files {
            assert_eq!(
                std::fs::read(checkout.path().join(path)).unwrap(),
                *expected,
                "{}",
                path.display()
            );
        }
    }

    #[test]
    fn recorded_result_complete_imports_have_one_owner_and_foreign_context_refuses() {
        let imports = super::super::contract_metadata::imports(root()).unwrap();
        let local =
            ProjectionGraph::of::<pse_model::engineering::EngineeringDescribeRequest>().unwrap();
        let imported = engineering_imports(&local, &imports).unwrap();
        let early = output(&imports).unwrap();
        let early_file = early
            .generation
            .descriptors()
            .file
            .iter()
            .find(|file| file.name.as_deref() == Some(FILE))
            .unwrap();
        assert_eq!(
            imported
                .file
                .iter()
                .find(|file| file.name.as_deref() == Some(FILE)),
            Some(early_file)
        );
        assert!(engineering_imports(&graph().unwrap(), &imports).is_err());
        let mut config = boundary_projection::default_config("crate::full::__buffa_fds");
        config.extern_paths = extern_paths().unwrap();
        assert_eq!(config.extern_paths.len(), 4);
        let paths = buffa_codegen::context::CodeGenContext::new(
            &imported.file,
            &config,
            &config.extern_paths,
        );
        assert_eq!(
            paths.rust_type(".pse.engineering.v1.RecordedScientificUsability.Alternative1"),
            Some(
                "crate::recorded_result_projection::pb::pse::engineering::v1::recorded_scientific_usability::Alternative1"
            )
        );
        let full = boundary_projection::generate_with_dependencies(
            &local,
            PACKAGE,
            "engineering.proto",
            &imported.file,
            &[FILE],
            &config,
        )
        .unwrap();
        let descriptor = full
            .generation
            .descriptors()
            .file
            .iter()
            .find(|file| file.name.as_deref() == Some("engineering.proto"))
            .unwrap();
        assert!(
            descriptor
                .dependency
                .iter()
                .any(|dependency| dependency == FILE)
        );
        assert!(!descriptor.message_type.iter().any(|message| {
            graph()
                .unwrap()
                .definitions()
                .contains_key(message.name.as_deref().unwrap())
        }));
        let source = full.source("pse.engineering.v1.rs").unwrap();
        assert!(!source.contains("struct RecordedResultSelection"));
        assert!(!source.contains("impl ::buffa_descriptor::reflect::Reflectable for Recorded"));
        // Same FQN does not make a smaller corpus a valid context for the full world.
        config.shared_corpus_context = Some(SharedCorpusContext::new(
            &early.generation.descriptors().file,
            &config,
        ));
        assert!(
            boundary_projection::generate_with_dependencies(
                &local,
                PACKAGE,
                "engineering.proto",
                &imported.file,
                &[FILE],
                &config
            )
            .is_err()
        );
        assert!(
            boundary_projection::generate_with_dependencies(
                &local,
                PACKAGE,
                "engineering.proto",
                &imports.file,
                &[FILE],
                &config
            )
            .is_err()
        );
        assert!(
            boundary_projection::generate_with_dependencies(
                &local,
                PACKAGE,
                "engineering.proto",
                &imported.file,
                &[FILE, FILE],
                &config
            )
            .is_err()
        );
    }
}
