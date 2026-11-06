// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit production contracts and discriminating representation fixtures.
use anyhow::{Context, Result, bail};
use buffa::Message as _;
use buffa_descriptor::generated::descriptor::FileDescriptorSet;
use pse_codegen::codegen::GeneratedTree;
use sha2::Digest as _;
use std::{
    path::{Path, PathBuf},
    process::Command,
};
mod associations;

pub(super) fn generate(root: &Path) -> Result<GeneratedTree> {
    let (mut tree, production_inputs) = corpus(
        root,
        "crates/pse-host/proto",
        "crates/pse-host/src/generated/code-intelligence",
    )?;
    let (fixture, fixture_inputs) = corpus(
        root,
        "crates/pse-host/tests/fixtures/plan61",
        "crates/pse-host/src/generated/plan61",
    )?;
    super::proto_projection::merge_independent(&mut tree, fixture)?;
    production_inputs.verify()?;
    fixture_inputs.verify()?;
    Ok(tree)
}

fn corpus(root: &Path, source: &str, destination: &str) -> Result<(GeneratedTree, Inputs)> {
    let source = root.join(source);
    let inputs = Inputs::capture(&source)?;
    let output = tempfile::tempdir()?;
    let image = output.path().join("image.binpb");
    run(
        Command::new("buf")
            .args(["build", "-o"])
            .arg(&image)
            .current_dir(&source),
        "compile code-intelligence image",
    )?;
    let descriptor = output.path().join("descriptor.bin");
    run(
        Command::new("buf")
            .args(["build"])
            .arg(&image)
            .args(["--as-file-descriptor-set", "-o"])
            .arg(&descriptor)
            .current_dir(&source),
        "derive complete descriptor from compiled image",
    )?;
    let bytes = std::fs::read(&descriptor)?;
    let set = FileDescriptorSet::decode_from_slice(&bytes)?;
    let files: Vec<String> = inputs
        .files
        .keys()
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "proto")
        })
        .map(|path| path.to_string_lossy().into_owned())
        .collect();
    let rust_module = if destination.ends_with("code-intelligence") {
        "crate::code_intelligence_contracts::pb"
    } else {
        "crate::plan61_qualification::pb"
    };
    let mut config = pse_codegen::codegen::boundary_projection::default_config(&format!(
        "{rust_module}::__buffa_fds"
    ));
    // Authored files retain their original per-file message/module identities.
    config.file_per_package = false;
    let generation = pse_codegen::codegen::boundary_projection::ProjectionGeneration::new(
        &set, &files, &config,
    )?;
    anyhow::ensure!(
        generation.descriptors() == &set,
        "authored shared descriptor interpretation differs from the complete compiler world"
    );
    for warning in &generation.warnings {
        eprintln!("{} (Buffa): {warning}", source.display());
    }
    let support = generation.support("descriptor-runtime.bin")?;
    let suppliers = super::proto_projection::composition::generate(&generation, rust_module)
        .with_context(|| format!("{}: compose admitted authored corpus", source.display()))?;
    let destination = PathBuf::from(destination);
    let sources = generation
        .files
        .iter()
        .map(|file| (file.name.as_str(), file.content.as_str()))
        .chain(
            support
                .sources
                .iter()
                .map(|(name, source)| (name.as_str(), source.as_str())),
        );
    let reserved: &[&str] = if destination.ends_with("code-intelligence") {
        &["descriptor.bin", "descriptor-runtime.bin", "dispatch.rs"]
    } else {
        &["descriptor.bin", "descriptor-runtime.bin"]
    };
    suppliers.admit(
        sources.clone().map(|(name, _)| name),
        &destination,
        reserved,
    )?;
    let mut tree = GeneratedTree::empty(vec![destination.clone()]);
    for (name, source) in sources {
        super::proto_projection::insert(&mut tree, &destination, name, source.as_bytes())?;
    }
    super::proto_projection::insert(&mut tree, &destination, "descriptor.bin", &bytes)?;
    super::proto_projection::insert(
        &mut tree,
        &destination,
        "descriptor-runtime.bin",
        &support.descriptor_bytes,
    )?;
    suppliers.insert(&mut tree, &destination)?;
    if destination.ends_with("code-intelligence") {
        let dispatch = super::proto_dispatch::generate(&tree, &destination, rust_module)?;
        super::proto_projection::insert(&mut tree, &destination, "dispatch.rs", &dispatch)?;
    }
    for file in files {
        let origin = pse_codegen::codegen::provenance::SourceRelationship {
            declaration_key: file.clone(),
            declaration_kind: "protobuf_file".into(),
            source_path: Some(
                source
                    .join(&file)
                    .strip_prefix(root)?
                    .to_string_lossy()
                    .into_owned(),
            ),
            source_symbol: None,
            source_role: "authored_wire_contract".into(),
            source_identity: Some(format!(
                "sha256:{}",
                sha2::Sha256::digest(&inputs.files[Path::new(&file)])
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>()
            )),
            generator_path: "xtask/src/codegen/code_intelligence.rs".into(),
            generator_symbol: "corpus".into(),
            output_path: destination
                .join("descriptor.bin")
                .to_string_lossy()
                .into_owned(),
            direction: None,
            contract_sections: vec!["blueprint §21.7".into()],
            basis: "declared".into(),
        };
        tree.relationships.push(origin.clone());
        let mut runtime_origin = origin.clone();
        runtime_origin.output_path = destination
            .join("descriptor-runtime.bin")
            .to_string_lossy()
            .into_owned();
        tree.relationships.push(runtime_origin);
        for output in tree
            .files
            .keys()
            .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
        {
            let mut contributor = origin.clone();
            contributor.source_role = "authored_corpus_contributor".into();
            contributor.output_path = output.to_string_lossy().into_owned();
            tree.relationships.push(contributor);
        }
    }
    let origins = pse_codegen::codegen::provenance::sidecar(
        if destination.ends_with("code-intelligence") {
            "code-intelligence"
        } else {
            "plan61-fixtures"
        },
        &tree.relationships,
    )?;
    super::proto_projection::merge_independent(&mut tree, origins)?;
    associations::generate(root, &source, &inputs, &bytes, &set, &mut tree)?;
    Ok((tree, inputs))
}

struct Inputs {
    root: PathBuf,
    files: std::collections::BTreeMap<PathBuf, Vec<u8>>,
}
impl Inputs {
    fn capture(root: &Path) -> Result<Self> {
        let mut authored = Vec::new();
        authored_files(root, root, &mut authored)?;
        let mut files = std::collections::BTreeMap::new();
        for file in authored {
            files.insert(PathBuf::from(&file), std::fs::read(root.join(file))?);
        }
        for file in ["buf.yaml", "buf.lock"] {
            files.insert(PathBuf::from(file), std::fs::read(root.join(file))?);
        }
        Ok(Self {
            root: root.into(),
            files,
        })
    }
    fn verify(&self) -> Result<()> {
        if Self::capture(&self.root)?.files != self.files {
            bail!(
                "code-intelligence source/configuration changed during generation; output was not published"
            );
        }
        Ok(())
    }
}

fn run(command: &mut Command, meaning: &str) -> Result<()> {
    let result = command.output().with_context(|| meaning.to_owned())?;
    if !result.status.success() {
        bail!("{meaning}: {}", String::from_utf8_lossy(&result.stderr));
    }
    if !result.stderr.is_empty() {
        eprintln!("{meaning}: {}", String::from_utf8_lossy(&result.stderr));
    }
    Ok(())
}

fn authored_files(base: &Path, directory: &Path, files: &mut Vec<String>) -> Result<()> {
    for entry in std::fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_symlink() {
            bail!(
                "authored schema input must not be a symlink: {}",
                path.display()
            );
        }
        if path.is_dir() {
            authored_files(base, &path, files)?;
        } else if path
            .extension()
            .is_some_and(|extension| extension == "proto")
        {
            files.push(path.strip_prefix(base)?.to_string_lossy().into_owned());
        }
    }
    Ok(())
}

/// Keep upstream code intact while qualifying its explicit lint escapes and fixed-data
/// assertions. Byte-span edits preserve generated comments and source spelling.
pub(super) fn qualify_generated_rust(source: &str) -> Result<String> {
    use syn::{spanned::Spanned as _, visit::Visit as _};
    struct Expects {
        count: usize,
        unknown: Vec<String>,
    }
    impl<'ast> syn::visit::Visit<'ast> for Expects {
        fn visit_expr_method_call(&mut self, expression: &'ast syn::ExprMethodCall) {
            if expression.method == "expect" {
                self.count += 1;
                let message = expression.args.first().and_then(|argument| match argument {
                    syn::Expr::Lit(literal) => match &literal.lit {
                        syn::Lit::Str(value) => Some(value.value()),
                        _ => None,
                    },
                    _ => None,
                });
                if !matches!(
                    message.as_deref(),
                    Some(
                        "buffa-codegen emitted a decodable FileDescriptorSet"
                            | "generated message is registered in the embedded descriptor pool"
                            | "generated view type is registered in the embedded descriptor pool"
                            | "view re-encodes to bytes decodable against its own descriptor"
                    )
                ) {
                    self.unknown
                        .push(message.unwrap_or_else(|| "non-literal expect".into()));
                }
            }
            syn::visit::visit_expr_method_call(self, expression);
        }
    }
    struct Qualifier<'a> {
        source: &'a str,
        edits: Vec<(usize, String)>,
        errors: Vec<String>,
    }
    impl Qualifier<'_> {
        fn function(&mut self, name: &str, start: usize, block: &syn::Block) {
            let mut expects = Expects {
                count: 0,
                unknown: Vec::new(),
            };
            expects.visit_block(block);
            self.errors.extend(expects.unknown);
            if expects.count != 0 {
                if !matches!(
                    name,
                    "descriptor_pool" | "__buffa_reflect_message_index" | "to_dynamic"
                ) {
                    self.errors.push(format!(
                        "expect outside pinned descriptor/reflection functions: {name}"
                    ));
                    return;
                }
                let line = self.source[..start]
                    .rfind('\n')
                    .map_or(0, |newline| newline + 1);
                let indentation = &self.source[line..start];
                self.edits.push((start, format!("#[allow(clippy::expect_used, reason = \"Pinned generator asserts only its immutable embedded descriptor/type registration and self-encoding invariants; acquired input decoding remains fallible\")]\n{indentation}")));
            }
        }
    }
    impl<'ast> syn::visit::Visit<'ast> for Qualifier<'_> {
        fn visit_attribute(&mut self, attribute: &'ast syn::Attribute) {
            if attribute.path().is_ident("allow") {
                let arguments = attribute.parse_args_with(
                    syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
                );
                match arguments {
                    Ok(arguments)
                        if !arguments
                            .iter()
                            .any(|argument| argument.path().is_ident("reason")) =>
                    {
                        let range = attribute.span().byte_range();
                        if let Some(relative) = self.source[range.clone()].rfind(')') {
                            let position = range.start + relative;
                            let comma = if arguments.is_empty() || arguments.trailing_punct() {
                                ""
                            } else {
                                ","
                            };
                            self.edits.push((position, format!("{comma} reason = \"Pinned protobuf generator emits portable schema spelling and implementation scaffolding\"")));
                        } else {
                            self.errors
                                .push("allow attribute has no closing delimiter".into());
                        }
                    }
                    Ok(_) => {}
                    Err(error) => self.errors.push(error.to_string()),
                }
            }
            syn::visit::visit_attribute(self, attribute);
        }
        fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
            self.function(
                &function.sig.ident.to_string(),
                function.span().byte_range().start,
                &function.block,
            );
            syn::visit::visit_item_fn(self, function);
        }
        fn visit_impl_item_fn(&mut self, function: &'ast syn::ImplItemFn) {
            self.function(
                &function.sig.ident.to_string(),
                function.span().byte_range().start,
                &function.block,
            );
            syn::visit::visit_impl_item_fn(self, function);
        }
    }
    let syntax = syn::parse_file(source)?;
    let mut qualifier = Qualifier {
        source,
        edits: Vec::new(),
        errors: Vec::new(),
    };
    qualifier.visit_file(&syntax);
    anyhow::ensure!(
        qualifier.errors.is_empty(),
        "unsupported generated Rust lint invariant: {:?}",
        qualifier.errors
    );
    qualifier
        .edits
        .sort_by_key(|(position, _)| std::cmp::Reverse(*position));
    let mut output = source.to_owned();
    for (position, insertion) in qualifier.edits {
        output.insert_str(position, &insertion);
    }
    syn::parse_file(&output).context("qualified generated Rust must remain valid syntax")?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generated_lint_qualification_preserves_spelling_and_limits_expect_permissions() {
        let source = "// source comment\n#![allow(dead_code)]\n#[allow(unused_imports,)]\nmod schema {}\nfn descriptor_pool() { parsed.expect(\"buffa-codegen emitted a decodable FileDescriptorSet\"); }\n";
        let output = qualify_generated_rust(source).unwrap();
        assert!(output.starts_with("// source comment\n"));
        assert_eq!(output.matches("reason =").count(), 3);
        assert!(output.contains("allow(clippy::expect_used"));
        assert!(
            output
                .contains("parsed.expect(\"buffa-codegen emitted a decodable FileDescriptorSet\")")
        );
        assert!(
            qualify_generated_rust("fn parse() { input.expect(\"untrusted input\"); }").is_err()
        );
        let ordinary = qualify_generated_rust("fn parse() { let value = decode()?; } ").unwrap();
        assert!(!ordinary.contains("expect_used"));
    }

    #[test]
    fn plan61_input_guard_refuses_content_population_and_configuration_changes() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        for file in ["a.proto", "buf.yaml", "buf.lock"] {
            std::fs::write(root.join(file), b"original").unwrap();
        }
        let initial = Inputs::capture(root).unwrap();
        assert!(initial.verify().is_ok());
        std::fs::write(root.join("a.proto"), b"edited").unwrap();
        assert!(initial.verify().is_err());
        std::fs::write(root.join("a.proto"), b"original").unwrap();
        std::fs::write(root.join("new.proto"), b"added").unwrap();
        assert!(initial.verify().is_err());
        std::fs::remove_file(root.join("new.proto")).unwrap();
        std::fs::write(root.join("buf.lock"), b"changed import").unwrap();
        assert!(initial.verify().is_err());
        std::fs::write(root.join("buf.lock"), b"original").unwrap();
        std::fs::remove_file(root.join("a.proto")).unwrap();
        assert!(initial.verify().is_err());
    }

    #[test]
    fn plan61_failed_admission_does_not_publish_or_remove_existing_outputs() {
        let directory = tempfile::tempdir().unwrap();
        let output = directory
            .path()
            .join("crates/pse-host/src/generated/code-intelligence");
        std::fs::create_dir_all(&output).unwrap();
        let original = output.join("previous.rs");
        std::fs::write(&original, b"previous complete generation").unwrap();
        assert!(
            super::super::run(
                directory.path(),
                false,
                Some(crate::Target::CodeIntelligence)
            )
            .is_err()
        );
        assert_eq!(
            std::fs::read(&original).unwrap(),
            b"previous complete generation"
        );
        assert_eq!(std::fs::read_dir(output).unwrap().count(), 1);
    }

    #[test]
    fn plan61_selected_and_aggregate_stage_faults_preserve_current_and_stale_outputs() {
        for aggregate in [false, true] {
            for fault in ["invalid CEL", "conflicting descriptor", "supplier failed"] {
                let directory = tempfile::tempdir().unwrap();
                let root = directory.path();
                let output = root.join("generated");
                std::fs::create_dir_all(&output).unwrap();
                std::fs::write(output.join("current.rs"), b"current generation").unwrap();
                std::fs::write(output.join("stale.rs"), b"owned stale output").unwrap();
                let consumed = std::cell::Cell::new(false);
                let result = super::super::stage_code_intelligence(
                    root,
                    |_| -> Result<GeneratedTree> { bail!("{fault}") },
                    |tree| {
                        consumed.set(true);
                        if aggregate {
                            let staged = vec![tree];
                            for tree in staged {
                                super::super::write_tree(root, &tree)?;
                            }
                            Ok(())
                        } else {
                            super::super::write_tree(root, &tree)
                        }
                    },
                );
                assert!(result.unwrap_err().to_string().contains(fault));
                assert!(!consumed.get());
                assert_eq!(
                    std::fs::read(output.join("current.rs")).unwrap(),
                    b"current generation"
                );
                assert_eq!(
                    std::fs::read(output.join("stale.rs")).unwrap(),
                    b"owned stale output"
                );
            }
        }
    }
}
