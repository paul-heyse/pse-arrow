// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use anyhow::{Context, Result, bail, ensure};
use pyo3_introspection::model::{Class, Constant, Expr, Function, Module};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};
use syn::{Item, Meta, Token, UseTree, punctuated::Punctuated};

type ClassNames = BTreeMap<String, BTreeSet<String>>;

pub(super) fn read_class_names(directory: &Path) -> Result<ClassNames> {
    let mut names = ClassNames::new();
    read_classes(directory, &mut names)?;
    Ok(names)
}

fn read_classes(directory: &Path, names: &mut ClassNames) -> Result<()> {
    let mut entries = fs::read_dir(directory)?.collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            read_classes(&path, names)?;
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            class_names(&syn::parse_file(&fs::read_to_string(&path)?)?.items, names).with_context(
                || format!("reading native class declarations from {}", path.display()),
            )?;
        }
    }
    Ok(())
}

fn class_names(items: &[Item], names: &mut ClassNames) -> Result<()> {
    for item in items {
        let (ident, attributes) = match item {
            Item::Struct(item) => (&item.ident, &item.attrs),
            Item::Enum(item) => (&item.ident, &item.attrs),
            Item::Mod(item) => {
                if let Some((_, items)) = &item.content {
                    class_names(items, names)?;
                }
                continue;
            }
            _ => continue,
        };
        for attribute in attributes.iter().filter(|attribute| {
            attribute
                .path()
                .segments
                .last()
                .is_some_and(|segment| segment.ident == "pyclass")
        }) {
            let mut renamed = None;
            if matches!(attribute.meta, Meta::List(_)) {
                for option in
                    attribute.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?
                {
                    if !option.path().is_ident("name") {
                        continue;
                    }
                    let Meta::NameValue(option) = option else {
                        bail!("native class name must be a string literal");
                    };
                    let syn::Expr::Lit(value) = option.value else {
                        bail!("native class name must be a string literal");
                    };
                    let syn::Lit::Str(value) = value.lit else {
                        bail!("native class name must be a string literal");
                    };
                    ensure!(
                        renamed.replace(value.value()).is_none(),
                        "duplicate native class name option"
                    );
                }
            }
            names
                .entry(ident.to_string())
                .or_default()
                .insert(renamed.unwrap_or_else(|| ident.to_string()));
        }
    }
    Ok(())
}

pub(super) fn complete(
    module: &mut Module,
    source: &str,
    exceptions: Vec<Class>,
    classes: &ClassNames,
) -> Result<()> {
    let expected = exports(source, classes)?;
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
fn exports(source: &str, classes: &ClassNames) -> Result<BTreeSet<String>> {
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
                use_names(&item.tree, &mut names, classes)?;
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
fn export_name(original: &str, fallback: &str, classes: &ClassNames) -> Result<String> {
    let Some(names) = classes.get(original) else {
        return Ok(fallback.to_owned());
    };
    ensure!(
        names.len() == 1,
        "ambiguous native class export {original}: {names:?}"
    );
    names.first().cloned().context("native class name absent")
}

fn use_names(tree: &UseTree, names: &mut BTreeSet<String>, classes: &ClassNames) -> Result<()> {
    match tree {
        UseTree::Path(path) => use_names(&path.tree, names, classes)?,
        UseTree::Name(name) => {
            ensure!(
                names.insert(export_name(
                    &name.ident.to_string(),
                    &name.ident.to_string(),
                    classes
                )?),
                "duplicate native export"
            );
        }
        UseTree::Rename(name) => {
            ensure!(
                names.insert(export_name(
                    &name.ident.to_string(),
                    &name.rename.to_string(),
                    classes
                )?),
                "duplicate native export"
            );
        }
        UseTree::Group(group) => {
            for tree in &group.items {
                use_names(tree, names, classes)?;
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
                Vec::new(),
                &ClassNames::new()
            )
            .is_err()
        );
        assert!(
            exports(
                "mod _native { #[pymodule_init] fn init() {} }",
                &ClassNames::new()
            )
            .is_err()
        );
        assert!(
            exports(
                "mod _native { #[pymodule_export] use super::*; }",
                &ClassNames::new()
            )
            .is_err()
        );
        assert!(
            complete_type(&Expr::Name {
                id: "Any".to_owned()
            })
            .is_err()
        );
    }
    #[test]
    fn native_exports_come_from_actual_declarative_syntax() {
        assert_eq!(exports("mod _native { #[pymodule_export] use super::{Store, source::Snapshot}; #[pymodule_export] const __version__: &str = VERSION; }", &ClassNames::new()).unwrap(),
            BTreeSet::from(["Store".to_owned(), "Snapshot".to_owned(), "__version__".to_owned()]));
    }

    #[test]
    fn renamed_classes_match_callable_metadata_and_reject_stale_exports() {
        let declarations = syn::parse_file(
            r#"
            #[pyclass(frozen, name = "_OwnedStore", module = "native")]
            struct InternalStore;
        "#,
        )
        .unwrap();
        let mut names = ClassNames::new();
        class_names(&declarations.items, &mut names).unwrap();
        let source = "mod _native { #[pymodule_export] use super::InternalStore; }";
        let mut actual = module();
        actual.incomplete = false;
        actual.classes.push(Class {
            name: "_OwnedStore".to_owned(),
            bases: Vec::new(),
            methods: vec![Function {
                name: "value".to_owned(),
                decorators: Vec::new(),
                arguments: pyo3_introspection::model::Arguments {
                    positional_only_arguments: vec![pyo3_introspection::model::Argument {
                        name: "self".to_owned(),
                        default_value: None,
                        annotation: None,
                    }],
                    arguments: Vec::new(),
                    vararg: None,
                    keyword_only_arguments: Vec::new(),
                    kwarg: None,
                },
                returns: Some(Expr::Name {
                    id: "int".to_owned(),
                }),
                is_async: false,
                docstring: None,
            }],
            attributes: Vec::new(),
            decorators: Vec::new(),
            inner_classes: Vec::new(),
            docstring: None,
        });
        complete(&mut actual, source, Vec::new(), &names).unwrap();
        let mut incomplete_callable = actual.clone();
        incomplete_callable.classes[0].methods[0].returns = None;
        assert!(complete(&mut incomplete_callable, source, Vec::new(), &names).is_err());
        actual.classes[0].name = "InternalStore".to_owned();
        assert!(complete(&mut actual, source, Vec::new(), &names).is_err());
    }

    #[test]
    fn class_names_follow_ast_options_nested_declarations_and_rust_aliases() {
        let declarations = syn::parse_file(
            r#"
            // #[pyclass(name = "NotADeclaration")]
            #[pyclass] struct Ordinary;
            mod nested {
                #[pyo3::pyclass(module = "native", name = "_Choice", frozen)]
                enum RustChoice { One }
            }
        "#,
        )
        .unwrap();
        let mut names = ClassNames::new();
        class_names(&declarations.items, &mut names).unwrap();
        assert_eq!(exports("mod _native { #[pymodule_export] use super::{Ordinary as Alias, nested::RustChoice as OtherAlias}; }", &names).unwrap(),
            BTreeSet::from(["Ordinary".to_owned(), "_Choice".to_owned()]));
        assert!(
            exports(
                "mod _native { #[pymodule_export] use super::{Ordinary, Ordinary as Alias}; }",
                &names
            )
            .is_err()
        );

        for source in [
            "#[pyclass(name)] struct Wrong;",
            "#[pyclass(name = 42)] struct Wrong;",
            "#[pyclass(name = NAME)] struct Wrong;",
            "#[pyclass(name = \"A\", name = \"B\")] struct Wrong;",
        ] {
            assert!(
                class_names(
                    &syn::parse_file(source).unwrap().items,
                    &mut ClassNames::new()
                )
                .is_err()
            );
        }
        let conflicting = syn::parse_file("mod first { #[pyclass(name = \"A\")] struct Same; } mod second { #[pyclass(name = \"B\")] struct Same; }").unwrap();
        class_names(&conflicting.items, &mut names).unwrap();
        assert!(
            exports(
                "mod _native { #[pymodule_export] use super::first::Same; }",
                &names
            )
            .is_err()
        );
    }

    #[test]
    fn renamed_class_declarations_are_read_from_native_source_files() {
        let directory = tempfile::tempdir().unwrap();
        let nested = directory.path().join("nested");
        fs::create_dir(&nested).unwrap();
        fs::write(directory.path().join("lib.rs"), "#[pyclass] struct Plain;").unwrap();
        fs::write(
            nested.join("classes.rs"),
            "#[pyclass(name = \"_Other\")] struct RustOther;",
        )
        .unwrap();
        fs::write(nested.join("ignored.txt"), "not Rust source").unwrap();
        let names = read_class_names(directory.path()).unwrap();
        assert_eq!(
            exports(
                "mod _native { #[pymodule_export] use super::{Plain, nested::RustOther}; }",
                &names
            )
            .unwrap(),
            BTreeSet::from(["Plain".to_owned(), "_Other".to_owned()])
        );
    }
}
