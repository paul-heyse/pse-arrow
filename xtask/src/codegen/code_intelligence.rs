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
    tree.roots.extend(fixture.roots);
    tree.files.extend(fixture.files);
    tree.relationships.extend(fixture.relationships);
    production_inputs.verify()?;
    fixture_inputs.verify()?;
    Ok(tree)
}

fn corpus(root: &Path, source: &str, destination: &str) -> Result<(GeneratedTree, Inputs)> {
    let source = root.join(source);
    let inputs = Inputs::capture(&source)?;
    let output = tempfile::tempdir()?;
    let tools = root.join(".local/plan61-tools/bin");
    prepared_plugins(root, &tools)?;
    let mut paths = vec![tools];
    paths.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
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
    reject_conflicting_files(&set)?;
    let files: Vec<String> = inputs
        .files
        .keys()
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "proto")
        })
        .map(|path| path.to_string_lossy().into_owned())
        .collect();
    let mut preflight =
        Command::new(root.join(".local/plan61-tools/bin/protoc-gen-protovalidate-buffa"));
    preflight.arg("--preflight").arg(&descriptor);
    for file in &files {
        preflight.arg("--file").arg(file);
    }
    let admission = preflight
        .output()
        .context("execute descriptor-bound CEL preflight")?;
    admit_preflight(&admission)?;
    run(
        Command::new("buf")
            .arg("generate")
            .arg(&image)
            .arg("--template")
            .arg(source.join("buf.gen.yaml"))
            .arg("--output")
            .arg(output.path())
            .current_dir(&source)
            .env("PATH", std::env::join_paths(paths)?),
        "generate checked code-intelligence contracts",
    )?;
    let generated_root = std::fs::read_to_string(output.path().join("mod.rs"))?;
    let runtime = runtime_descriptors(&set, &generated_root)?;
    // descriptor.bin remains Buf's original complete source-aware compiler output.
    // descriptor-runtime.bin is the actual pinned shared-pool reflection artifact;
    // these two worlds must differ only in genuine SourceCodeInfo.
    std::fs::write(
        output.path().join("descriptor-runtime.bin"),
        runtime.encode_to_vec(),
    )?;
    std::fs::remove_file(&image)?;
    let destination = PathBuf::from(destination);
    let mut tree = GeneratedTree::empty(vec![destination.clone()]);
    collect(output.path(), output.path(), &destination, &mut tree)?;
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
    tree.roots.extend(origins.roots);
    tree.files.extend(origins.files);
    associations::generate(root, &source, &inputs, &bytes, &set, &mut tree)?;
    Ok((tree, inputs))
}

/// Read the actual pinned packaging supplier's shared reflection artifact, rather
/// than reproduce its feature configuration or infer it from generated type names.
fn runtime_descriptors(
    source: &FileDescriptorSet,
    generated_root: &str,
) -> Result<FileDescriptorSet> {
    let syntax = syn::parse_file(generated_root)?;
    let roots = syntax
        .items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Mod(module) if module.ident == "__buffa_fds" => Some(module),
            _ => None,
        })
        .collect::<Vec<_>>();
    let [root] = roots.as_slice() else {
        bail!("pinned shared descriptor publisher must emit exactly one __buffa_fds root");
    };
    let (_, items) = root
        .content
        .as_ref()
        .context("shared descriptor root must be inline")?;
    let descriptors = items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Const(item) if item.ident == "FILE_DESCRIPTOR_SET_BYTES" => Some(item),
            _ => None,
        })
        .collect::<Vec<_>>();
    let [descriptor] = descriptors.as_slice() else {
        bail!("pinned shared descriptor root must emit exactly one complete artifact");
    };
    let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::ByteStr(bytes),
        ..
    }) = descriptor.expr.as_ref()
    else {
        bail!("pinned shared descriptor artifact must be a literal byte string");
    };
    let mut runtime = FileDescriptorSet::decode_from_slice(&bytes.value())?;
    reject_conflicting_files(&runtime)?;
    anyhow::ensure!(
        source.file.len() == runtime.file.len(),
        "shared descriptor publication changed the file population"
    );
    for (original, effective) in source.file.iter().zip(&runtime.file) {
        anyhow::ensure!(
            original.name == effective.name,
            "shared descriptor publication changed file order or identity"
        );
        anyhow::ensure!(
            effective.source_code_info.as_option().is_none(),
            "shared runtime descriptor unexpectedly retains source information"
        );
        let mut compact = original.clone();
        compact.source_code_info = Default::default();
        // The pinned protoc plugin explicitly refuses feature overrides in shared
        // pool mode. Preserve that limit: do not publish mismatched effective
        // options/features or silently drop source-retained content.
        anyhow::ensure!(
            compact == *effective,
            "shared descriptor runtime content differs from the complete compiler world: {}",
            original.name.as_deref().unwrap_or("")
        );
    }
    // The supplier accepts a file slice. Retain the compiler artifact's outer-set
    // unknown fields too, rather than silently dropping them during that crossing.
    runtime.__buffa_unknown_fields = source.__buffa_unknown_fields.clone();
    Ok(runtime)
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
        for file in ["buf.yaml", "buf.gen.yaml", "buf.lock"] {
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
    Ok(())
}

fn admit_preflight(result: &std::process::Output) -> Result<()> {
    if !result.status.success() || result.stdout != b"pse-cel-preflight-v1\n" {
        bail!(
            "CEL preflight refused or returned an unknown protocol: {}",
            String::from_utf8_lossy(&result.stderr)
        );
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

fn reject_conflicting_files(set: &FileDescriptorSet) -> Result<()> {
    let mut files = std::collections::BTreeMap::new();
    for file in &set.file {
        let name = file.name.as_deref().context("descriptor filename absent")?;
        let encoded = file.encode_to_vec();
        if files
            .insert(name, encoded.clone())
            .is_some_and(|previous| previous != encoded)
        {
            bail!("conflicting descriptor filename: {name}");
        }
    }
    Ok(())
}

pub(super) fn prepared_plugins(root: &Path, binaries: &Path) -> Result<()> {
    let metadata: toml::Value = toml::from_str(
        &std::fs::read_to_string(root.join(".local/plan61-tools/.crates.toml"))
            .context("prepare exact Plan61 generators explicitly in .local/plan61-tools")?,
    )?;
    let installed = metadata
        .get("v1")
        .and_then(toml::Value::as_table)
        .context("Plan61 generator installation metadata has no v1 table")?;
    for (package, version, binary) in [
        ("protoc-gen-buffa", "0.9.2", "protoc-gen-buffa"),
        (
            "protoc-gen-buffa-packaging",
            "0.9.2",
            "protoc-gen-buffa-packaging",
        ),
        ("connectrpc-codegen", "0.9.0", "protoc-gen-connect-rust"),
        (
            "protoc-gen-protovalidate-buffa",
            "0.7.2",
            "protoc-gen-protovalidate-buffa",
        ),
    ] {
        let prefix = format!("{package} {version} (");
        let matched = installed.iter().any(|(key, files)| {
            key.starts_with(&prefix)
                && (package != "protoc-gen-protovalidate-buffa"
                    || key
                        .strip_prefix(&prefix)
                        .and_then(|source| source.strip_suffix(')'))
                        .and_then(|source| source.strip_prefix("path+"))
                        .and_then(|source| url::Url::parse(source).ok())
                        .and_then(|url| url.to_file_path().ok())
                        .is_some_and(|source| {
                            source == root.join("vendor/protoc-gen-protovalidate-buffa")
                        }))
                && files
                    .as_array()
                    .is_some_and(|files| files.iter().any(|file| file.as_str() == Some(binary)))
        });
        if !matched || !binaries.join(binary).is_file() {
            bail!(
                "prepare {package} {version} ({binary}) explicitly in .local/plan61-tools; global tools are not selected"
            );
        }
    }
    Ok(())
}

fn collect(
    base: &Path,
    directory: &Path,
    destination: &Path,
    tree: &mut GeneratedTree,
) -> Result<()> {
    for entry in std::fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            collect(base, &path, destination, tree)?;
        } else {
            let mut bytes = std::fs::read(&path)?;
            if path.extension().is_some_and(|extension| extension == "rs") {
                bytes = qualify_generated_rust(std::str::from_utf8(&bytes)?)?.into_bytes();
            }
            tree.files
                .insert(destination.join(path.strip_prefix(base)?), bytes);
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
    fn shared_root(set: &FileDescriptorSet) -> String {
        let bytes = syn::LitByteStr::new(
            &set.encode_to_vec(),
            syn::parse_str::<syn::Ident>("artifact").unwrap().span(),
        );
        quote::quote!(pub mod __buffa_fds { pub const FILE_DESCRIPTOR_SET_BYTES: &[u8] = #bytes; })
            .to_string()
    }
    #[test]
    fn code_intelligence_artifacts_preserve_source_and_exact_shared_runtime_content() {
        use buffa_descriptor::generated::descriptor::{SourceCodeInfo, source_code_info::Location};
        let mut source = FileDescriptorSet {
            file: vec![
                FileDescriptorProto {
                    name: Some("dependency.proto".into()),
                    syntax: Some("proto3".into()),
                    ..Default::default()
                },
                FileDescriptorProto {
                    name: Some("main.proto".into()),
                    dependency: vec!["dependency.proto".into()],
                    syntax: Some("proto3".into()),
                    source_code_info: SourceCodeInfo {
                        location: vec![Location {
                            path: vec![],
                            leading_comments: Some("original compiler source".into()),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }
                    .into(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        // Independently specified file options: explicit presence feature plus
        // unknown option100. Neither is allowed to vanish during publication.
        let mut main = source.file[1].encode_to_vec();
        main.extend_from_slice(&[0x42, 0x08, 0x92, 0x03, 0x02, 0x08, 0x01, 0xa0, 0x06, 0x09]);
        source.file[1] = FileDescriptorProto::decode_from_slice(&main).unwrap();
        let mut runtime = source.clone();
        for file in &mut runtime.file {
            file.source_code_info = Default::default();
        }
        let root = shared_root(&runtime);
        let original = source.encode_to_vec();
        assert_eq!(runtime_descriptors(&source, &root).unwrap(), runtime);
        assert_eq!(source.encode_to_vec(), original);
        assert_ne!(source.encode_to_vec(), runtime.encode_to_vec());
        source.__buffa_unknown_fields.push(buffa::UnknownField {
            number: 100,
            data: buffa::UnknownFieldData::Varint(9),
        });
        let retained = runtime_descriptors(&source, &root).unwrap();
        assert_eq!(
            retained.__buffa_unknown_fields,
            source.__buffa_unknown_fields
        );
        for change in 0..5 {
            let mut changed = runtime.clone();
            match change {
                0 => changed.file.reverse(),
                1 => {
                    changed.file.pop();
                }
                2 => changed.file[1].dependency.clear(),
                3 => changed.file[1].options = Default::default(),
                _ => changed.file[1].source_code_info = source.file[1].source_code_info.clone(),
            }
            assert!(
                runtime_descriptors(&source, &shared_root(&changed)).is_err(),
                "change {change}"
            );
        }
        assert!(runtime_descriptors(&source, "mod __buffa_fds {}").is_err());
        assert!(
            runtime_descriptors(
                &source,
                "mod __buffa_fds { const FILE_DESCRIPTOR_SET_BYTES: &[u8] = other(); }"
            )
            .is_err()
        );
    }
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
    use buffa_descriptor::generated::descriptor::FileDescriptorProto;

    #[test]
    fn plan61_input_guard_refuses_content_population_and_configuration_changes() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        for file in ["a.proto", "buf.yaml", "buf.gen.yaml", "buf.lock"] {
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
    fn plan61_unpatched_plugin_empty_success_is_not_preflight_admission() {
        use std::os::unix::process::ExitStatusExt as _;
        let mut response = std::process::Output {
            status: std::process::ExitStatus::from_raw(0),
            stdout: Vec::new(),
            stderr: Vec::new(),
        };
        assert!(admit_preflight(&response).is_err());
        response.stdout = b"pse-cel-preflight-v1\n".to_vec();
        assert!(admit_preflight(&response).is_ok());
        response.status = std::process::ExitStatus::from_raw(256);
        assert!(admit_preflight(&response).is_err());
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
            for fault in ["invalid CEL", "conflicting descriptor", "plugin failed"] {
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

    #[test]
    fn plan61_descriptor_conflicts_fail_before_pool_linking() {
        let file = FileDescriptorProto {
            name: Some("same.proto".into()),
            package: Some("first".into()),
            ..Default::default()
        };
        let mut set = FileDescriptorSet {
            file: vec![file.clone(), file],
            ..Default::default()
        };
        assert!(reject_conflicting_files(&set).is_ok());
        set.file[1].package = Some("second".into());
        assert!(
            reject_conflicting_files(&set)
                .unwrap_err()
                .to_string()
                .contains("same.proto")
        );
    }

    #[test]
    fn plan61_same_version_registry_validator_is_not_the_corrected_generator() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let binaries = root.join(".local/plan61-tools/bin");
        std::fs::create_dir_all(&binaries).unwrap();
        let mut table = toml::map::Map::new();
        for (name, version, binary) in [
            ("protoc-gen-buffa", "0.9.2", "protoc-gen-buffa"),
            (
                "protoc-gen-buffa-packaging",
                "0.9.2",
                "protoc-gen-buffa-packaging",
            ),
            ("connectrpc-codegen", "0.9.0", "protoc-gen-connect-rust"),
            (
                "protoc-gen-protovalidate-buffa",
                "0.7.2",
                "protoc-gen-protovalidate-buffa",
            ),
        ] {
            std::fs::write(binaries.join(binary), b"unqualified executable").unwrap();
            table.insert(
                format!("{name} {version} (registry+https://github.com/rust-lang/crates.io-index)"),
                toml::Value::Array(vec![toml::Value::String(binary.into())]),
            );
        }
        let metadata = toml::Value::Table(toml::map::Map::from_iter([(
            "v1".into(),
            toml::Value::Table(table),
        )]));
        std::fs::write(
            root.join(".local/plan61-tools/.crates.toml"),
            toml::to_string(&metadata).unwrap(),
        )
        .unwrap();
        assert!(prepared_plugins(root, &binaries).is_err());
    }
}
