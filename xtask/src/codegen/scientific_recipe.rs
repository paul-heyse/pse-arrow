// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Private passive Recipe publication; qualification and production activation stay native.

use anyhow::Result;
use pse_codegen::codegen::{GeneratedTree, boundary_projection, provenance};
use std::path::Path;

const PACKAGE: &str = "pse.scientific_recipe.v4";
const FILE: &str = "scientific_recipe.proto";
const DESTINATION: &str = "crates/pse-compiler/src/generated/scientific-recipe";
const RUST_MODULE: &str = "crate::typed_math::portable::generated::pb";
const WIRE_ROOT: &str = "crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4";

pub(super) fn generate(root: &Path) -> Result<GeneratedTree> {
    super::code_intelligence::prepared_plugins(root, &root.join(".local/plan61-tools/bin"))?;
    let graph = pse_compiler::typed_math::scientific_recipe_projection()?;
    let imports = super::contract_metadata::imports(root)?;
    let mut config = boundary_projection::default_config(&format!("{RUST_MODULE}::__buffa_fds"));
    config.generate_views = true;
    config.lazy_views = true;
    config.preserve_unknown_fields = true;
    let output =
        boundary_projection::generate_with_config(&graph, PACKAGE, FILE, &imports.file, &config)?;
    let roots = ["ScientificRecipe"];
    let adapters = boundary_projection::native_adapters_scoped(
        &graph,
        &roots,
        WIRE_ROOT,
        "pse_compiler",
        boundary_projection::AdapterVisibility::Parent,
    )?;
    // Naming must come from the very descriptor world and profile that emitted
    // these messages, including all compiler-owned nested wrappers.
    let view_adapters = boundary_projection::native_view_adapters(
        &graph,
        &roots,
        boundary_projection::NativeViewTarget {
            descriptors: &output.descriptor_set,
            selected_file: FILE,
            config: &config,
            wire_root: WIRE_ROOT,
            owner_crate: "pse_compiler",
            visibility: boundary_projection::AdapterVisibility::Parent,
        },
    )?;
    let mut tree = super::proto_projection::publish_projection(
        root,
        &graph,
        super::proto_projection::ProjectionTarget {
            package: PACKAGE,
            file: FILE,
            destination: DESTINATION,
            rust_module: RUST_MODULE,
            document_root: super::ENGINEERING_DOCUMENT_ROOT,
            generator_path: "xtask/src/codegen/scientific_recipe.rs",
            generator_symbol: "generate",
        },
        output,
    )?;
    let destination = Path::new(DESTINATION);
    for (name, source, symbol) in [
        ("native-adapters.rs", adapters, "native_adapters_scoped"),
        (
            "native-view-adapters.rs",
            view_adapters,
            "native_view_adapters",
        ),
    ] {
        super::proto_projection::insert(&mut tree, destination, name, source.as_bytes())?;
        for owner in graph.definitions().values() {
            tree.relationships.push(provenance::SourceRelationship {
                declaration_key: owner.name.clone(),
                declaration_kind: "native_projection_adapter".into(),
                source_path: Some(owner.source.into()),
                source_symbol: Some(owner.rust_type.into()),
                source_role: "typed_owner".into(),
                source_identity: None,
                generator_path: "crates/pse-codegen/src/codegen/boundary_projection/adapters.rs"
                    .into(),
                generator_symbol: symbol.into(),
                output_path: destination.join(name).to_string_lossy().into_owned(),
                direction: Some(owner.direction.as_str().into()),
                contract_sections: vec!["blueprint §4.7".into()],
                basis: "declared".into(),
            });
        }
    }
    let sidecar = provenance::sidecar(
        &super::proto_projection::producer(PACKAGE),
        &tree.relationships,
    )?;
    // The shared publisher already declares this producer's provenance root.
    tree.files.extend(sidecar.files);
    Ok(tree)
}

#[cfg(test)]
mod tests {
    use super::*;
    use buffa::Message as _;
    use buffa_descriptor::generated::descriptor::FileDescriptorSet;

    #[test]
    fn scientific_recipe_bundle_publishes_actual_private_graph_and_both_views() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let tree = generate(root).unwrap();
        super::super::validate_tree(&tree).unwrap();
        let destination = Path::new(DESTINATION);
        let descriptors =
            FileDescriptorSet::decode_from_slice(&tree.files[&destination.join("descriptor.bin")])
                .unwrap();
        let file = descriptors
            .file
            .iter()
            .find(|file| file.name.as_deref() == Some(FILE))
            .unwrap();
        assert_eq!(file.package.as_deref(), Some(PACKAGE));
        let graph = pse_compiler::typed_math::scientific_recipe_projection().unwrap();
        for owner in graph.definitions().values() {
            assert!(
                file.message_type
                    .iter()
                    .any(|message| message.name.as_deref() == Some(owner.name.as_str()))
                    || file.enum_type
                        .iter()
                        .any(|enumeration| enumeration.name.as_deref() == Some(owner.name.as_str())),
                "missing actual native owner {}",
                owner.name
            );
        }
        let recipe = file
            .message_type
            .iter()
            .find(|message| message.name.as_deref() == Some("ScientificRecipe"))
            .unwrap();
        assert_eq!(recipe.field.len(), 17);
        let sources = tree
            .files
            .iter()
            .filter(|(path, _)| path.starts_with(destination))
            .filter_map(|(_, bytes)| std::str::from_utf8(bytes).ok())
            .collect::<Vec<_>>()
            .join("\n");
        for expected in [
            "ScientificRecipeView",
            "ScientificRecipeLazyView",
            "__buffa_unknown_fields",
        ] {
            assert!(sources.contains(expected), "missing {expected}");
        }
        let adapters =
            std::str::from_utf8(&tree.files[&destination.join("native-adapters.rs")]).unwrap();
        assert!(adapters.contains("pub(super) fn encode_scientific_recipe"));
        assert!(adapters.contains("pub(super) fn decode_scientific_recipe"));
        assert!(!adapters.contains("pub fn decode_scientific_recipe"));
        let borrowed =
            std::str::from_utf8(&tree.files[&destination.join("native-view-adapters.rs")]).unwrap();
        assert!(borrowed.contains("pub(super) fn decode_scientific_recipe_view"));
        assert!(borrowed.contains("decode_view_root"));
        assert!(borrowed.contains("unknown_fields_view"));
        assert!(!borrowed.contains("to_owned_message"));
        assert!(
            tree.files
                .contains_key(&destination.join("validation/mod.rs"))
        );
        let policy =
            std::str::from_utf8(&tree.files[&destination.join("native-policy.rs")]).unwrap();
        assert!(policy.contains("ScientificRecipe"));
        let provenance_root =
            Path::new("docs/generated/source-relationships/pse-scientific-recipe-v4");
        assert_eq!(
            tree.roots
                .iter()
                .filter(|root| *root == provenance_root)
                .count(),
            1
        );
        for name in ["native-adapters.rs", "native-view-adapters.rs"] {
            assert!(tree.relationships.iter().any(|relationship| {
                relationship.declaration_key == "ScientificRecipe"
                    && relationship.output_path == destination.join(name).to_string_lossy()
            }));
        }
    }
}
