// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Compiled dispatch from the actual message and validator module trees.

use anyhow::{Context, Result, ensure};
use pse_codegen::codegen::GeneratedTree;
use quote::ToTokens as _;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Component, Path, PathBuf},
};

#[derive(Default)]
struct Population {
    names: BTreeMap<String, syn::Path>,
    validators: BTreeSet<String>,
}

fn owned_type(module: &syn::Path, implementation: &syn::ItemImpl) -> Result<Option<syn::Path>> {
    // Views carry lifetimes and cannot be the owned decode target of this facade.
    if !implementation.generics.params.is_empty() {
        return Ok(None);
    }
    let syn::Type::Path(ty) = implementation.self_ty.as_ref() else {
        anyhow::bail!("generated dispatch requires an explicit Rust type path");
    };
    ensure!(
        ty.qself.is_none() && ty.path.leading_colon.is_none(),
        "generated dispatch requires a module-relative type"
    );
    if ty
        .path
        .segments
        .iter()
        .any(|part| !matches!(part.arguments, syn::PathArguments::None))
    {
        return Ok(None);
    }
    let mut path = module.clone();
    path.segments.extend(ty.path.segments.clone());
    Ok(Some(path))
}

impl Population {
    fn file(
        &mut self,
        tree: &GeneratedTree,
        file: &Path,
        module: &syn::Path,
        stack: &mut BTreeSet<PathBuf>,
    ) -> Result<()> {
        ensure!(
            stack.insert(file.into()),
            "recursive generated include: {}",
            file.display()
        );
        let bytes = tree
            .files
            .get(file)
            .with_context(|| format!("generated dispatch include absent: {}", file.display()))?;
        let syntax = syn::parse_file(std::str::from_utf8(bytes)?)?;
        self.items(tree, file, &syntax.items, module, stack)?;
        stack.remove(file);
        Ok(())
    }

    fn items(
        &mut self,
        tree: &GeneratedTree,
        file: &Path,
        items: &[syn::Item],
        module: &syn::Path,
        stack: &mut BTreeSet<PathBuf>,
    ) -> Result<()> {
        for item in items {
            match item {
                syn::Item::Mod(item) => {
                    let mut nested = module.clone();
                    nested.segments.push(item.ident.clone().into());
                    let (_, items) = item
                        .content
                        .as_ref()
                        .context("generated dispatch requires inline package modules")?;
                    self.items(tree, file, items, &nested, stack)?;
                }
                syn::Item::Macro(item) if item.mac.path.is_ident("include") => {
                    let name = syn::parse2::<syn::LitStr>(item.mac.tokens.clone())?.value();
                    let relative = Path::new(&name);
                    ensure!(
                        !relative.as_os_str().is_empty()
                            && relative
                                .components()
                                .all(|part| matches!(part, Component::Normal(_))),
                        "unsafe generated dispatch include: {name}"
                    );
                    self.file(
                        tree,
                        &file
                            .parent()
                            .context("generated source parent absent")?
                            .join(relative),
                        module,
                        stack,
                    )?;
                }
                syn::Item::Impl(item) => self.implementation(module, item)?,
                _ => {}
            }
        }
        Ok(())
    }

    fn implementation(&mut self, module: &syn::Path, item: &syn::ItemImpl) -> Result<()> {
        let Some((trait_path, _)) = &item.trait_ else {
            return Ok(());
        };
        let Some(trait_name) = trait_path.segments.last() else {
            return Ok(());
        };
        if trait_name.ident != "MessageName" && trait_name.ident != "Validate" {
            return Ok(());
        }
        let identity = trait_path.to_token_stream().to_string();
        ensure!(
            identity == ":: buffa :: MessageName"
                || identity == ":: protovalidate_buffa :: Validate",
            "unsupported emitted dispatch trait path: {identity}"
        );
        let Some(path) = owned_type(module, item)? else {
            return Ok(());
        };
        let key = path.to_token_stream().to_string();
        if identity == ":: protovalidate_buffa :: Validate" {
            ensure!(
                self.validators.insert(key.clone()),
                "duplicate emitted validator: {key}"
            );
            return Ok(());
        }
        let names = item
            .items
            .iter()
            .filter_map(|item| {
                let syn::ImplItem::Const(item) = item else {
                    return None;
                };
                if item.ident != "FULL_NAME" {
                    return None;
                }
                let syn::Expr::Lit(value) = &item.expr else {
                    return None;
                };
                let syn::Lit::Str(value) = &value.lit else {
                    return None;
                };
                Some(value.value())
            })
            .collect::<Vec<_>>();
        let [name] = names.as_slice() else {
            anyhow::bail!("emitted MessageName has no unique literal FULL_NAME: {key}");
        };
        ensure!(
            self.names.insert(name.clone(), path).is_none(),
            "duplicate emitted message identity: {name}"
        );
        Ok(())
    }
}

/// Derive only from supplier-emitted identities and actual packaging. Missing owned
/// validation refuses generation rather than silently leaving a dynamic fallback.
pub(super) fn generate(
    tree: &GeneratedTree,
    destination: &Path,
    rust_module: &str,
) -> Result<Vec<u8>> {
    let mut population = Population::default();
    let module: syn::Path = syn::parse_str(rust_module)?;
    ensure!(
        module.leading_colon.is_none()
            && module
                .segments
                .first()
                .is_some_and(|part| part.ident == "crate"),
        "compiled dispatch requires an actual crate-local message module"
    );
    for root in [
        destination.join("mod.rs"),
        destination.join("validation/mod.rs"),
    ] {
        population.file(tree, &root, &module, &mut BTreeSet::new())?;
    }
    ensure!(
        !population.names.is_empty(),
        "generated dispatch has no owned message population"
    );
    let mut paths = BTreeSet::new();
    let mut arms = Vec::new();
    for (name, ty) in population.names {
        let key = ty.to_token_stream().to_string();
        ensure!(
            paths.insert(key.clone()),
            "emitted Rust type has multiple message identities: {key}"
        );
        ensure!(
            population.validators.contains(&key),
            "emitted owned message has no generated validator: {name} ({key})"
        );
        let tail = ty.segments.iter().skip(1);
        arms.push(quote::quote!(#name => Some($callback::<$crate::#(#tail)::*> $arguments),));
    }
    let source = quote::quote! {
        macro_rules! dispatch {
            ($name:expr, $callback:ident, $arguments:tt) => {
                match $name { #(#arms)* _ => None }
            };
        }
        pub(crate) use dispatch;
    };
    Ok(format!(
        "// @generated from emitted MessageName and Validate implementations; do not edit.\n{}",
        prettyplease::unparse(&syn::parse2(source)?)
    )
    .into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree() -> GeneratedTree {
        let mut tree = GeneratedTree::empty(vec!["world".into()]);
        for (name, body) in [
            ("mod.rs", "pub mod r#type { include!(\"renamed.rs\"); }"),
            (
                "renamed.rs",
                r#"
                impl ::buffa::MessageName for Renamed {
                    const FULL_NAME: &'static str = "source.First";
                }
                pub mod nested {
                    impl ::buffa::MessageName for r#match {
                        const FULL_NAME: &'static str = "source.First.Child";
                    }
                }
                impl<'a> ::buffa::MessageName for RenamedView<'a> {
                    const FULL_NAME: &'static str = "source.First";
                }
            "#,
            ),
            (
                "validation/mod.rs",
                "pub mod r#type { include!(\"rules.rs\"); }",
            ),
            (
                "validation/rules.rs",
                r#"
                impl ::protovalidate_buffa::Validate for Renamed {}
                impl ::protovalidate_buffa::Validate for nested::r#match {}
                impl ::protovalidate_buffa::Validate for RenamedView<'_> {}
            "#,
            ),
        ] {
            tree.files
                .insert(Path::new("world").join(name), body.as_bytes().to_vec());
        }
        tree
    }

    #[test]
    fn emitted_dispatch_uses_actual_renamed_nested_raw_types_and_ignores_views() {
        let bytes = generate(&tree(), Path::new("world"), "crate::wire").unwrap();
        let source = String::from_utf8(bytes).unwrap();
        let compact = source.split_whitespace().collect::<String>();
        assert!(compact.contains("$crate::wire::r#type::Renamed"));
        assert!(compact.contains("$crate::wire::r#type::nested::r#match"));
        assert!(source.contains("\"source.First.Child\""));
        assert!(!source.contains("RenamedView"));
        assert_eq!(compact.matches("=>Some").count(), 2);
    }

    #[test]
    fn emitted_dispatch_refuses_missing_validator_and_ambiguous_message_identity() {
        let mut missing = tree();
        missing.files.insert(
            "world/validation/rules.rs".into(),
            b"impl ::protovalidate_buffa::Validate for Renamed {}".to_vec(),
        );
        assert!(
            generate(&missing, Path::new("world"), "crate::wire")
                .unwrap_err()
                .to_string()
                .contains("source.First.Child")
        );
        let mut duplicate = tree();
        duplicate
            .files
            .get_mut(Path::new("world/renamed.rs"))
            .unwrap()
            .extend_from_slice(
                br#"
            impl ::buffa::MessageName for Other {
                const FULL_NAME: &'static str = "source.First";
            }
        "#,
            );
        assert!(
            generate(&duplicate, Path::new("world"), "crate::wire")
                .unwrap_err()
                .to_string()
                .contains("duplicate emitted message identity")
        );
    }

    #[test]
    fn emitted_dispatch_refuses_absent_recursive_and_escaping_supplier_includes() {
        for include in ["absent.rs", "mod.rs", "../outside.rs"] {
            let mut tree = tree();
            tree.files.insert(
                "world/mod.rs".into(),
                format!("include!({include:?});").into_bytes(),
            );
            assert!(
                generate(&tree, Path::new("world"), "crate::wire").is_err(),
                "{include}"
            );
        }
    }
}
