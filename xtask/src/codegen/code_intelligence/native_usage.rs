// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Compose one Buf-authored service world with its native census declarations.

use super::{Inputs, Result};
use anyhow::{Context, ensure};
use buffa_descriptor::generated::descriptor::{FileDescriptorProto, FileDescriptorSet};
use pse_codegen::codegen::{GeneratedTree, boundary_projection, provenance};
use pse_contract::boundary_projection::ProjectionGraph;
use std::{collections::BTreeSet, fmt::Write as _, path::{Path, PathBuf}};

pub(super) const FILE: &str = "pse/code_intelligence/v1/native_usage.proto";
const PACKAGE: &str = "pse.code_intelligence.v1";
const ADAPTERS: &str = "native-adapters.rs";
const POLICY: &str = "native-policy.rs";

pub(super) fn stage(directory: &Path, inputs: &Inputs, graph: &ProjectionGraph) -> Result<PathBuf> {
    let directory = directory.join("authored");
    ensure!(!inputs.files.contains_key(Path::new(FILE)), "native census filename is already authored");
    for (path, bytes) in &inputs.files {
        let target = directory.join(path);
        std::fs::create_dir_all(target.parent().context("staged Buf input parent")?)?;
        std::fs::write(target, bytes)?;
    }
    // Buf resolves the authored service references by top-level name. This shell
    // has no field declarations and never reaches publication or code generation.
    let mut source = format!("syntax = \"proto3\";\npackage {PACKAGE};\n");
    for name in graph.definitions().keys() {
        writeln!(source, "message {name} {{}}")?;
    }
    let shell = directory.join(FILE);
    std::fs::create_dir_all(shell.parent().context("staged native census parent")?)?;
    std::fs::write(shell, source)?;
    Ok(directory)
}

pub(super) fn link(
    root: &Path,
    graph: &ProjectionGraph,
    authored: &mut FileDescriptorSet,
    targets: &[String],
) -> Result<()> {
    let mut imports = FileDescriptorSet::default();
    imports.file.extend(authored.file.iter().filter(|file| {
        file.name.as_deref().is_some_and(|name| name != FILE && !targets.iter().any(|target| target == name))
    }).cloned());
    super::super::contract_metadata::append_native_options(root, &mut imports)?;
    let lowered = boundary_projection::descriptors_with_imports(graph, PACKAGE, FILE, &imports.file)?;
    replace_shell(authored, lowered, targets)
}

fn replace_shell(
    authored: &mut FileDescriptorSet,
    lowered: FileDescriptorSet,
    targets: &[String],
) -> Result<()> {
    let shell = unique_file(&authored.file, FILE)?;
    let native = unique_file(&lowered.file, FILE)?;
    let shell_names = shell.message_type.iter().map(|message| message.name.as_deref()).collect::<BTreeSet<_>>();
    let native_names = native.message_type.iter().map(|message| message.name.as_deref()).collect::<BTreeSet<_>>();
    ensure!(
        shell.package.as_deref() == Some(PACKAGE)
            && shell.dependency.is_empty()
            && shell.enum_type.is_empty()
            && shell.service.is_empty()
            && shell.extension.is_empty()
            && shell_names.len() == shell.message_type.len()
            && shell_names == native_names
            && shell.message_type.iter().all(|message| {
                message == &buffa_descriptor::generated::descriptor::DescriptorProto {
                    name: message.name.clone(), ..Default::default()
                }
            }),
        "Buf native census shell differs from the source-owned symbol inventory"
    );
    let mut required = BTreeSet::from([FILE.to_owned()]);
    for file in lowered.file.iter().rev() {
        if file.name.as_ref().is_some_and(|name| required.contains(name)) {
            required.extend(file.dependency.iter().cloned());
        }
    }
    let (targets, mut linked): (Vec<_>, Vec<_>) = std::mem::take(&mut authored.file)
        .into_iter()
        .filter(|file| file.name.as_deref() != Some(FILE))
        .partition(|file| file.name.as_ref().is_some_and(|name| targets.contains(name)));
    ensure!(targets.iter().any(|file| file.dependency.iter().any(|dependency| dependency == FILE)), "authored service does not import the native census");
    for file in lowered.file {
        let name = file.name.as_deref().context("native import filename")?;
        if !required.contains(name) {
            continue;
        }
        if let Some(original) = linked.iter().find(|original| original.name == file.name) {
            ensure!(original == &file, "native census import differs from original Buf descriptor: {name}");
        } else {
            linked.push(file);
        }
    }
    linked.extend(targets);
    authored.file = linked;
    Ok(())
}

fn unique_file<'a>(files: &'a [FileDescriptorProto], name: &str) -> Result<&'a FileDescriptorProto> {
    let mut matches = files.iter().filter(|file| file.name.as_deref() == Some(name));
    let file = matches.next().with_context(|| format!("missing descriptor: {name}"))?;
    ensure!(matches.next().is_none(), "duplicate descriptor: {name}");
    Ok(file)
}

pub(super) fn publish(root: &Path, graph: &ProjectionGraph, destination: &Path, tree: &mut GeneratedTree) -> Result<()> {
    let roots = ["NativeDependencyUsageQuery", "NativeDependencyUsageAnswer"];
    let adapters = boundary_projection::native_adapters_scoped(
        graph, &roots, "crate::code_intelligence_contracts::pb::pse::code_intelligence::v1",
        "pse_host", boundary_projection::AdapterVisibility::Crate,
    )?;
    let policy = boundary_projection::native_policy_source(graph, PACKAGE, pse_contract::BoundaryDirection::Both)?;
    for (name, bytes) in [(ADAPTERS, adapters.as_bytes()), (POLICY, policy.as_bytes())] {
        super::super::proto_projection::insert(tree, destination, name, bytes)?;
    }
    for owner in graph.definitions().values() {
        let source = std::fs::read(root.join(owner.source))?;
        let source_identity = format!("sha256:{}", super::source_sha256(&source));
        for path in tree.files.keys() {
            tree.relationships.push(provenance::SourceRelationship {
                declaration_key: owner.name.clone(),
                declaration_kind: "rust_projection".into(),
                source_path: Some(owner.source.into()),
                source_symbol: Some(owner.rust_type.into()),
                source_role: "typed_owner".into(),
                source_identity: Some(source_identity.clone()),
                generator_path: "xtask/src/codegen/code_intelligence/native_usage.rs".into(),
                generator_symbol: "publish".into(),
                output_path: path.to_string_lossy().into_owned(),
                direction: Some(owner.direction.as_str().into()),
                contract_sections: vec!["blueprint §21.7".into()],
                basis: "declared".into(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use buffa_descriptor::generated::descriptor::{DescriptorProto, FieldDescriptorProto};

    const AUTHORED: &str = "pse/code_intelligence/v1/contracts.proto";
    const IMPORT: &str = "required.proto";

    fn file(name: &str) -> FileDescriptorProto {
        FileDescriptorProto { name: Some(name.into()), package: Some(PACKAGE.into()), ..Default::default() }
    }

    fn worlds() -> (FileDescriptorSet, FileDescriptorSet) {
        let mut service = file(AUTHORED);
        service.dependency.push(FILE.into());
        let mut shell = file(FILE);
        shell.message_type.push(DescriptorProto { name: Some("NativeQuery".into()), ..Default::default() });
        let mut real = shell.clone();
        real.dependency.push(IMPORT.into());
        real.message_type[0].field.push(FieldDescriptorProto { name: Some("actual".into()), number: Some(1), ..Default::default() });
        let import = file(IMPORT);
        (
            FileDescriptorSet { file: vec![import.clone(), shell, service], ..Default::default() },
            FileDescriptorSet { file: vec![import, real], ..Default::default() },
        )
    }

    #[test]
    fn links_exact_native_definitions_before_the_authored_importer() {
        let (mut authored, native) = worlds();
        replace_shell(&mut authored, native.clone(), &[AUTHORED.into()]).unwrap();
        assert_eq!(&authored.file[..2], &native.file);
        assert_eq!(authored.file[2].name.as_deref(), Some(AUTHORED));
        assert_eq!(authored.file[1].message_type[0].field[0].name.as_deref(), Some("actual"));
    }

    #[test]
    fn refuses_symbol_collision_shell_fields_and_foreign_import_content() {
        let (authored, native) = worlds();
        let mut duplicate = authored.clone();
        duplicate.file.push(duplicate.file[1].clone());
        assert!(replace_shell(&mut duplicate, native.clone(), &[AUTHORED.into()]).is_err());
        let mut contaminated = authored.clone();
        contaminated.file[1].message_type[0].field = native.file[1].message_type[0].field.clone();
        assert!(replace_shell(&mut contaminated, native.clone(), &[AUTHORED.into()]).is_err());
        let mut drifted = authored;
        drifted.file[0].package = Some("foreign".into());
        assert!(replace_shell(&mut drifted, native, &[AUTHORED.into()]).is_err());
    }
}
