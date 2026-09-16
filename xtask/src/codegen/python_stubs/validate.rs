// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use anyhow::{Context, Result, bail, ensure};
use pyo3_introspection::model::{Class, Constant, Expr, Function, Module};
use std::collections::BTreeSet;
use syn::{Item, UseTree};

pub(super) fn complete(module: &mut Module, source: &str, exceptions: Vec<Class>) -> Result<()> {
    let expected = exports(source)?;
    let mut names = inventory(module);
    let mut supplemented = false;
    for class in exceptions {
        ensure!(
            expected.contains(&class.name),
            "diagnostic class is not a declared native export"
        );
        ensure!(
            names.insert(class.name.clone()),
            "diagnostic supplement duplicates an introspected native class"
        );
        module.classes.push(class);
        supplemented = true;
    }
    ensure!(
        names == expected,
        "native introspection differs from actual declared exports: expected {expected:?}, got {names:?}; rebuild the extension"
    );
    // Function-based initialization is rejected in exports(). The only supported
    // missing metadata fragment is the typed native-exception declaration above.
    ensure!(
        !module.incomplete || supplemented,
        "native module introspection is incomplete"
    );
    module.incomplete = false;
    ensure!(
        module.modules.is_empty(),
        "native boundary submodule needs its own declared generation contract"
    );
    for function in &module.functions {
        function_types(function)?;
    }
    for class in &module.classes {
        class_types(class)?;
    }
    for attribute in &module.attributes {
        if let Some(annotation) = &attribute.annotation {
            complete_type(annotation)?;
        } else {
            ensure!(
                attribute.value.is_some(),
                "native attribute {} has no type or constant value",
                attribute.name
            );
        }
    }
    module
        .classes
        .sort_by(|left, right| left.name.cmp(&right.name));
    module
        .functions
        .sort_by(|left, right| left.name.cmp(&right.name));
    module
        .attributes
        .sort_by(|left, right| left.name.cmp(&right.name));
    for class in &mut module.classes {
        class
            .methods
            .sort_by(|left, right| left.name.cmp(&right.name));
        class
            .attributes
            .sort_by(|left, right| left.name.cmp(&right.name));
        clear_class_docs(class);
    }
    // Runtime documentation remains on the native objects. Stub files contain
    // signatures only, as required by the repository's Python stub contract.
    module.docstring = None;
    for function in &mut module.functions {
        function.docstring = None;
    }
    for attribute in &mut module.attributes {
        attribute.docstring = None;
    }
    Ok(())
}

fn clear_class_docs(class: &mut Class) {
    class.docstring = None;
    for method in &mut class.methods {
        method.docstring = None;
    }
    for attribute in &mut class.attributes {
        attribute.docstring = None;
    }
    for inner in &mut class.inner_classes {
        clear_class_docs(inner);
    }
}
fn inventory(module: &Module) -> BTreeSet<String> {
    module
        .classes
        .iter()
        .map(|item| item.name.clone())
        .chain(module.functions.iter().map(|item| item.name.clone()))
        .chain(module.attributes.iter().map(|item| item.name.clone()))
        .collect()
}
fn exports(source: &str) -> Result<BTreeSet<String>> {
    let module = syn::parse_file(source)?
        .items
        .into_iter()
        .find_map(|item| {
            let Item::Mod(item) = item else {
                return None;
            };
            (item.ident == "_native").then_some(item)
        })
        .context("declarative _native module absent")?;
    let (_, items) = module
        .content
        .context("native module must be inline for introspection")?;
    let mut names = BTreeSet::new();
    for item in items {
        match item {
            Item::Use(item)
                if item
                    .attrs
                    .iter()
                    .any(|attr| attr.path().is_ident("pymodule_export")) =>
            {
                use_names(&item.tree, &mut names)?;
            }
            Item::Const(item)
                if item
                    .attrs
                    .iter()
                    .any(|attr| attr.path().is_ident("pymodule_export")) =>
            {
                names.insert(item.ident.to_string());
            }
            _ => bail!(
                "native module must export explicit declarations; dynamic initializers cannot define a complete stub"
            ),
        }
    }
    Ok(names)
}
fn use_names(tree: &UseTree, names: &mut BTreeSet<String>) -> Result<()> {
    match tree {
        UseTree::Path(path) => use_names(&path.tree, names)?,
        UseTree::Name(name) => {
            ensure!(
                names.insert(name.ident.to_string()),
                "duplicate native export"
            );
        }
        UseTree::Rename(name) => {
            ensure!(
                names.insert(name.rename.to_string()),
                "duplicate native export"
            );
        }
        UseTree::Group(group) => {
            for tree in &group.items {
                use_names(tree, names)?;
            }
        }
        UseTree::Glob(_) => {
            bail!("native wildcard exports cannot establish an exact stub inventory")
        }
    }
    Ok(())
}
fn class_types(class: &Class) -> Result<()> {
    for function in &class.methods {
        function_types(function)?;
    }
    for attribute in &class.attributes {
        complete_type(
            attribute
                .annotation
                .as_ref()
                .context("untyped native class attribute")?,
        )?;
    }
    for inner in &class.inner_classes {
        class_types(inner)?;
    }
    Ok(())
}
fn function_types(function: &Function) -> Result<()> {
    let arguments = &function.arguments;
    for argument in arguments
        .positional_only_arguments
        .iter()
        .chain(&arguments.arguments)
        .chain(&arguments.keyword_only_arguments)
    {
        if argument.name != "self" && argument.name != "cls" {
            complete_type(argument.annotation.as_ref().with_context(|| {
                format!(
                    "{}.{} has no native annotation",
                    function.name, argument.name
                )
            })?)?;
        }
    }
    for variable in arguments.vararg.iter().chain(&arguments.kwarg) {
        complete_type(
            variable
                .annotation
                .as_ref()
                .context("untyped native variadic argument")?,
        )?;
    }
    complete_type(
        function
            .returns
            .as_ref()
            .with_context(|| format!("{} has no return annotation", function.name))?,
    )
}
fn complete_type(annotation: &Expr) -> Result<()> {
    match annotation {
        Expr::Name { id } => ensure!(
            !["Any", "Incomplete", "dict", "list"].contains(&id.as_str()),
            "unconstrained native type {id}"
        ),
        Expr::Attribute { attr, .. } => ensure!(
            !["Any", "Incomplete"].contains(&attr.as_str()),
            "incomplete native annotation"
        ),
        Expr::BinOp { left, right, .. } => {
            complete_type(left)?;
            complete_type(right)?;
        }
        Expr::Tuple { elts } | Expr::List { elts } => {
            for value in elts {
                complete_type(value)?;
            }
        }
        Expr::Subscript { slice, .. } => complete_type(slice)?,
        Expr::Constant {
            value: Constant::Str(value),
        } => {
            ensure!(
                !value
                    .split(|character: char| !character.is_alphanumeric() && character != '_')
                    .any(|token| token == "Any" || token == "Incomplete"),
                "unconstrained string annotation"
            );
        }
        Expr::Constant { .. } => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn module() -> Module {
        Module {
            name: "_native".to_owned(),
            modules: Vec::new(),
            classes: Vec::new(),
            functions: Vec::new(),
            attributes: Vec::new(),
            incomplete: true,
            docstring: None,
        }
    }
    #[test]
    fn missing_or_dynamic_native_exports_refuse_generation() {
        assert!(
            complete(
                &mut module(),
                "mod _native { #[pymodule_export] use super::Store; }",
                Vec::new()
            )
            .is_err()
        );
        assert!(exports("mod _native { #[pymodule_init] fn init() {} }").is_err());
        assert!(exports("mod _native { #[pymodule_export] use super::*; }").is_err());
        assert!(
            complete_type(&Expr::Name {
                id: "Any".to_owned()
            })
            .is_err()
        );
    }
    #[test]
    fn native_exports_come_from_actual_declarative_syntax() {
        assert_eq!(exports("mod _native { #[pymodule_export] use super::{Store, source::Snapshot}; #[pymodule_export] const __version__: &str = VERSION; }").unwrap(),
            BTreeSet::from(["Store".to_owned(), "Snapshot".to_owned(), "__version__".to_owned()]));
    }
}
