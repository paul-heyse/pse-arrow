// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Mechanical existing project operation projections over actual typed receiver helpers.
use anyhow::{Context, Result, bail, ensure};
use pse_codegen::codegen::{GeneratedTree, provenance};
use quote::quote;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write as _,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use syn::parse::Parser as _;

pub(super) fn append(tree: &mut GeneratedTree) -> Result<()> {
    let root = crate::workspace_root()?;
    let declarations = pse_runtime::host_operations::project::methods();
    let glue_path = "crates/pse-py/src/workflow/project_operation_glue.rs";
    let file = syn::parse_file(&fs::read_to_string(root.join(glue_path))?)?;
    let mut helpers = BTreeMap::new();
    for item in file.items {
        if let syn::Item::Impl(implementation) = item {
            for item in implementation.items {
                if let syn::ImplItem::Fn(function) = item {
                    ensure!(
                        function.sig.ident.to_string().starts_with("_host_"),
                        "project helper must remain private receiver glue"
                    );
                    let name = function.sig.ident.to_string();
                    ensure!(
                        helpers.insert(name.clone(), function).is_none(),
                        "duplicate project helper {name}"
                    );
                }
            }
        }
    }
    let mut wrappers = Vec::new();
    let mut projection = Vec::new();
    let mut names = BTreeSet::new();
    for declaration in declarations {
        ensure!(
            names.insert(declaration.name),
            "duplicate public project operation {}",
            declaration.name
        );
        let helper_name = format!("_host_{}", declaration.name);
        let helper = helpers
            .remove(&helper_name)
            .with_context(|| format!("missing typed project receiver {helper_name}"))?;
        let receiver_description = helper
            .attrs
            .iter()
            .filter_map(|attribute| {
                if !attribute.path().is_ident("doc") {
                    return None;
                }
                let syn::Meta::NameValue(value) = &attribute.meta else {
                    return None;
                };
                let syn::Expr::Lit(value) = &value.value else {
                    return None;
                };
                let syn::Lit::Str(value) = &value.lit else {
                    return None;
                };
                Some(value.value().trim().to_owned())
            })
            .collect::<Vec<_>>()
            .join("\n");
        let mut projected = serde_json::to_value(declaration)?;
        projected["receiver_description"] = receiver_description.into();
        projection.push(projected);
        let mut wrapper = helper.clone();
        wrapper.vis = syn::Visibility::Inherited;
        wrapper.sig.ident = syn::Ident::new(declaration.name, helper.sig.ident.span());
        let arguments = helper
            .sig
            .inputs
            .iter()
            .filter_map(|argument| match argument {
                syn::FnArg::Receiver(_) => None,
                syn::FnArg::Typed(argument) => Some(match argument.pat.as_ref() {
                    syn::Pat::Ident(pattern) => Ok(pattern.ident.clone()),
                    _ => Err(anyhow::anyhow!(
                        "unsupported project receiver argument pattern"
                    )),
                }),
            })
            .collect::<Result<Vec<_>>>()?;
        let helper_ident = &helper.sig.ident;
        wrapper.block = syn::parse_quote!({ self.#helper_ident(#(#arguments),*) });
        if let Some(signature) = declaration.pyo3_signature {
            let attributes =
                syn::Attribute::parse_outer.parse_str(&format!("#[pyo3({signature})]"))?;
            wrapper.attrs.extend(attributes);
        }
        wrappers.push(wrapper);
    }
    ensure!(
        helpers.is_empty(),
        "unpublished project receiver helpers remain: {:?}",
        helpers.keys().collect::<Vec<_>>()
    );
    let generated: syn::File = syn::parse2(quote! {
        #![rustfmt::skip]
        #![allow(clippy::all, reason = "mechanical public PyO3 projection over typed receiver glue")]
        use super::*;
        #[pymethods]
        impl NativeRuntime { #(#wrappers)* }
    })?;
    let rust_path = PathBuf::from("crates/pse-py/src/generated/project_operations.rs");
    tree.files.insert(rust_path.clone(), format!("// @generated from the Rust project operation owner and typed helpers; do not edit\n{}", prettyplease::unparse(&generated)).into_bytes());
    let python_root = PathBuf::from("python/pse/generated");
    if !tree.roots.contains(&python_root) {
        tree.roots.push(python_root.clone());
    }
    insert_python(tree, &root, python_root.join("__init__.py"), b"# @generated; existing public project operation projections\n\"\"\"Generated existing public project operation projections.\"\"\"\n")?;
    let defaults = format!(
        "# @generated from Rust project operation defaults; do not edit\n\"\"\"Calling defaults projected from the Rust project operation owner.\"\"\"\nDEFAULT_BRANCH = {:?}\nDEFAULT_DOCUMENT_BYTES = {}\n__all__ = (\"DEFAULT_BRANCH\", \"DEFAULT_DOCUMENT_BYTES\")\n",
        pse_runtime::host_operations::project::DEFAULT_BRANCH,
        pse_runtime::host_operations::project::DEFAULT_DOCUMENT_BYTES
    );
    insert_python(
        tree,
        &root,
        python_root.join("project_operation_defaults.py"),
        defaults.as_bytes(),
    )?;
    let input = serde_json::to_vec(&projection)?;
    let output = project_python(&root, &input, false)?;
    let python_path = python_root.join("project_operations.py");
    insert_python(tree, &root, python_path.clone(), &output)?;
    super::add_python_manifest(tree, &python_root)?;
    for (source, symbol, output) in [
        (
            "crates/pse-runtime/src/host_operations/project.rs",
            "methods",
            &rust_path,
        ),
        (glue_path, "NativeRuntime", &rust_path),
        (
            "crates/pse-runtime/src/host_operations/project.rs",
            "methods",
            &python_path,
        ),
        (
            "python/pse/_project_operations_glue.py",
            "ProjectRuntimeGlue",
            &python_path,
        ),
        (
            "python/pse/_research.py",
            "ResearchRuntimeGlue",
            &python_path,
        ),
        (
            "python/pse/_study_workbench.py",
            "StudyWorkbenchGlue",
            &python_path,
        ),
    ] {
        tree.relationships.push(provenance::SourceRelationship {
            declaration_key: format!("{source}::{symbol}"),
            declaration_kind: "project_operation_projection".into(),
            source_path: Some(source.into()),
            source_symbol: Some(symbol.into()),
            source_role: "definition".into(),
            source_identity: None,
            generator_path: "xtask/src/codegen/project_operations.rs".into(),
            generator_symbol: "append".into(),
            output_path: output.to_string_lossy().into_owned(),
            direction: Some("input_output".into()),
            contract_sections: Vec::new(),
            basis: "declared".into(),
        });
    }
    Ok(())
}
fn insert_python(
    tree: &mut GeneratedTree,
    root: &Path,
    path: PathBuf,
    source: &[u8],
) -> Result<()> {
    let formatted = super::python_stubs::format_projection(root, source, &path.to_string_lossy())?;
    project_python(root, &formatted, true)?;
    tree.files.insert(path, formatted);
    Ok(())
}
fn project_python(root: &Path, input: &[u8], validate_floor: bool) -> Result<Vec<u8>> {
    let mut command = Command::new(super::python_stubs::tool(root, "python"));
    command
        .arg(root.join("scripts/project_operation_projection.py"))
        .arg(root);
    if validate_floor {
        command.arg("--validate-floor");
    }
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("project facade projection and declared Python floor validation")?;
    child
        .stdin
        .take()
        .context("project facade projection stdin unavailable")?
        .write_all(input)?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        bail!(
            "project facade interface projection refused: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(output.stdout)
}
