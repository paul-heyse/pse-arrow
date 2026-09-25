// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Split generated values from native codecs using the generator's Rust AST.
use std::collections::BTreeSet;
use std::path::PathBuf;

use proc_macro2::TokenStream;
use quote::quote;

use crate::SchemaError;
use crate::codegen::GeneratedTree;

const NATIVE: &str = "crates/pse-relations/src/generated";
const MODEL: &str = "crates/pse-model/src/generated";

/// Serde is emitted only for actual document and durable receipt consumers.
fn serialization_consumers(reg: &crate::Registry) -> BTreeSet<String> {
    reg.documents()
        .iter()
        .flat_map(|doc| {
            doc.sections
                .iter()
                .map(|section| section.relation.to_owned())
        })
        .chain([
            "runtime.publications".into(),
            "runtime.native_dependencies".into(),
        ])
        .collect()
}

/// Rewrite Rust paths, never text in strings, attributes or unrelated identifiers.
struct ModelPaths;
impl syn::visit_mut::VisitMut for ModelPaths {
    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        if path.leading_colon.is_none()
            && path.segments.len() >= 2
            && path.segments[0].ident == "crate"
            && path.segments[1].ident == "RelationError"
        {
            path.segments[1].ident = syn::Ident::new("ModelError", path.segments[1].ident.span());
            syn::visit_mut::visit_path_mut(self, path);
        } else if *path == syn::parse_quote!(crate::columnar::mismatch) {
            *path = syn::parse_quote!(crate::malformed);
        } else {
            syn::visit_mut::visit_path_mut(self, path);
        }
    }
}

fn semantic_item(item: &syn::Item, names: &BTreeSet<String>, relative: &std::path::Path) -> bool {
    match item {
        syn::Item::Struct(item) => names.contains(&item.ident.to_string()),
        syn::Item::Enum(_) => true,
        syn::Item::Type(item) => {
            item.ident == "Row" || relative == std::path::Path::new("extension_values.rs")
        }
        syn::Item::Impl(item) => {
            let target = match item.self_ty.as_ref() {
                syn::Type::Path(ty) => ty.path.get_ident().map(ToString::to_string),
                _ => None,
            };
            target.is_some_and(|target| names.contains(&target))
                && item.trait_.as_ref().is_none_or(|(path, _)| {
                    path.segments.last().is_some_and(|part| {
                        matches!(
                            part.ident.to_string().as_str(),
                            "FromStr" | "PartialEq" | "SemanticEq"
                        )
                    })
                })
        }
        _ => false,
    }
}

fn value_traits(model: &[syn::Item]) -> Vec<TokenStream> {
    model.iter().filter_map(|item| match item {
            syn::Item::Struct(item) => {
                let name = &item.ident;
                let fields = item.fields.iter().filter_map(|f| f.ident.as_ref()).collect::<Vec<_>>();
                Some(quote! {
                    impl crate::SemanticFrame for #name {
                        fn frame(&self, hash: &mut pse_ids::FramedHasher) { #(hash.str(stringify!(#fields)); crate::SemanticFrame::frame(&self.#fields, hash);)* }
                    }
                    impl crate::HeapUsage for #name {
                    fn heap_bytes(&self) -> usize { 0usize #(.saturating_add(crate::HeapUsage::heap_bytes(&self.#fields)))* }
                } })
            }
            syn::Item::Enum(item) if item.variants.iter().all(|v| matches!(v.fields, syn::Fields::Unit)) => {
                let name = &item.ident;
                Some(quote! { impl crate::HeapUsage for #name { fn heap_bytes(&self) -> usize { 0 } } impl crate::SemanticFrame for #name { fn frame(&self, hash: &mut pse_ids::FramedHasher) { hash.str(self.as_str()); } } })
            }
            _ => None,
        }).collect::<Vec<_>>()
}

fn value_names(file: &syn::File) -> BTreeSet<String> {
    file.items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Impl(item)
                if item.trait_.as_ref().is_some_and(|(path, _)| {
                    path.segments
                        .last()
                        .is_some_and(|part| part.ident == "ArrowValue")
                }) =>
            {
                if let syn::Type::Path(ty) = item.self_ty.as_ref() {
                    ty.path.get_ident().map(ToString::to_string)
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect::<BTreeSet<_>>()
}

pub(super) fn split(tree: &mut GeneratedTree, reg: &crate::Registry) -> Result<(), SchemaError> {
    use syn::visit_mut::VisitMut;
    let consumers = serialization_consumers(reg);
    let paths = tree
        .files
        .keys()
        .filter(|path| path.starts_with(NATIVE))
        .cloned()
        .collect::<Vec<_>>();
    let mut namespaces = BTreeSet::new();
    for path in paths {
        let relative = path
            .strip_prefix(NATIVE)
            .map_err(|error| super::error(error.to_string()))?;
        let file = syn::parse_file(
            std::str::from_utf8(&tree.files[&path])
                .map_err(|error| super::error(error.to_string()))?,
        )
        .map_err(|error| super::error(error.to_string()))?;
        let names = value_names(&file);
        if names.is_empty() {
            continue;
        }
        let mut native = Vec::new();
        let mut model = Vec::new();
        let mut exported = Vec::new();
        for item in file.items {
            let semantic = semantic_item(&item, &names, relative);
            if semantic {
                match &item {
                    syn::Item::Struct(item) => exported.push(item.ident.clone()),
                    syn::Item::Enum(item) => exported.push(item.ident.clone()),
                    syn::Item::Type(item) => exported.push(item.ident.clone()),
                    _ => {}
                }
                let mut item = item;
                ModelPaths.visit_item_mut(&mut item);
                let consumer = relative
                    .with_extension("")
                    .to_string_lossy()
                    .replace('/', ".");
                if relative.components().count() > 1
                    && !consumers.contains(&consumer)
                    && let syn::Item::Struct(value) = &mut item
                {
                    value.attrs.retain(|attr| {
                        !attr.path().is_ident("derive") && !attr.path().is_ident("serde")
                    });
                    value.attrs.push(syn::parse_quote!(#[derive(Clone, Debug)]));
                }
                model.push(item);
            } else {
                native.push(item);
            }
        }
        let usage = value_traits(&model);
        let segments = relative
            .with_extension("")
            .components()
            .map(|part| super::types::ident(&part.as_os_str().to_string_lossy()))
            .collect::<Vec<_>>();
        if let Some(parent) = relative
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            namespaces.insert(parent.to_path_buf());
        }
        super::emit(
            tree,
            MODEL.to_owned() + "/" + relative.to_string_lossy().as_ref(),
            quote!(#(#model)* #(#usage)*),
        )?;
        super::emit(
            tree,
            path,
            quote! {
                /// Registry-generated semantic values; native codecs remain local.
                pub use pse_model::generated::#(#segments)::* :: {#(#exported),*};
                #(#native)*
            },
        )?;
    }
    // Namespace module declarations are shared declaration projections, not value copies.
    for namespace in &namespaces {
        let relative = namespace.join("mod.rs");
        let bytes = tree
            .files
            .get(&PathBuf::from(NATIVE).join(&relative))
            .cloned()
            .ok_or_else(|| super::error("missing generated namespace".into()))?;
        tree.files
            .insert(PathBuf::from(MODEL).join(relative), bytes);
    }
    let modules = namespaces
        .iter()
        .map(|path| super::types::ident(&path.to_string_lossy()))
        .collect::<Vec<_>>();
    super::emit(
        tree,
        format!("{MODEL}/mod.rs"),
        quote! {
            //! Plain semantic values projected from the registry.
            #![allow(clippy::doc_markdown, reason = "registry documentation is projected verbatim")]
            #![allow(clippy::too_many_lines, reason = "complete generated tagged alternatives and relation inventories follow the registry")]
            #(#[doc = "Registry namespace values."] pub mod #modules;)*
            /// Declared closed enumerations.
            pub mod enums;
            /// Declared extension values.
            pub mod extension_values;
        },
    )?;
    Ok(())
}

/// Closed typed phase transport, generated from exactly the same relation inventory.
pub(super) fn facts(tree: &mut GeneratedTree, reg: &crate::Registry) -> Result<(), SchemaError> {
    let variants = reg
        .relations()
        .iter()
        .map(|spec| {
            super::types::ident(&format!(
                "{}{}",
                super::types::pascal(spec.key.namespace.as_str()),
                super::types::pascal(spec.key.name)
            ))
        })
        .collect::<Vec<_>>();
    let namespaces = reg
        .relations()
        .iter()
        .map(|spec| super::types::ident(spec.key.namespace.as_str()))
        .collect::<Vec<_>>();
    let names = reg
        .relations()
        .iter()
        .map(|spec| super::types::ident(spec.key.name))
        .collect::<Vec<_>>();
    let ids = reg
        .relations()
        .iter()
        .map(|spec| {
            let bytes = spec.id.as_bytes();
            quote!(pse_ids::SemanticId::from_bytes([#(#bytes),*]))
        })
        .collect::<Vec<_>>();
    let (key_comparisons, key_frames) = key_fragments(reg);
    super::emit(
        tree,
        format!("{MODEL}/facts.rs"),
        quote! {
            /// Typed relation values at the synchronous/relational compiler boundary.
            #[derive(Clone, Debug, PartialEq)]
            pub enum FactBatch { #(#[doc = stringify!(#variants)] #variants(Vec<super::#namespaces::#names::Row>),)* }
            impl FactBatch {
                /// Exact declared relation identity.
                pub fn relation(&self) -> pse_ids::SemanticId { match self { #(Self::#variants(_) => #ids,)* } }
                /// Number of values, preserving empty relation membership.
                pub fn len(&self) -> usize { match self { #(Self::#variants(rows) => rows.len(),)* } }
                /// Whether this selected relation contains zero rows.
                pub fn is_empty(&self) -> bool { self.len() == 0 }
                /// Frame the complete typed sequence, including relation and empty membership.
                /// This is a lookup identity; reuse still compares complete values.
                pub fn semantic_identity(&self) -> pse_ids::ContentHash {
                    let mut hash = pse_ids::FramedHasher::new("pse:typed-facts:v1");
                    hash.id(&self.relation()).u64(self.len() as u64);
                    match self { #(Self::#variants(rows) => {
                        for row in rows { crate::SemanticFrame::frame(row, &mut hash); }
                    },)* }
                    hash.finish_hash()
                }
                /// One exact generated row, retaining its relation identity.
                pub fn row(&self, index: usize) -> Option<Self> { match self { #(Self::#variants(rows) => rows.get(index).map(|row| Self::#variants(vec![row.clone()])),)* } }
                /// Full canonical semantic comparison without allocating row copies.
                pub fn same_row(&self, index: usize, other: &Self, other_index: usize) -> bool {
                    match (self, other) { #((Self::#variants(left), Self::#variants(right)) => {
                        match (left.get(index), right.get(other_index)) {
                            (Some(left), Some(right)) => crate::SemanticEq::semantic_eq(left, right), _ => false,
                        }
                    },)* _ => false }
                }
                /// Conservative clone extent, checked before allocating a one-row batch.
                pub fn row_bytes(&self, index: usize) -> Option<usize> { match self {
                    #(Self::#variants(rows) => rows.get(index).map(|row| crate::HeapUsage::owned_bytes(row).saturating_add(size_of::<Self>())),)*
                } }
                /// Complete primary-key equality; hash collisions do not merge keys.
                pub fn same_key(&self, index: usize, other: &Self, other_index: usize) -> bool {
                    match (self, other) { #((Self::#variants(left), Self::#variants(right)) => {
                        match (left.get(index), right.get(other_index)) {
                            (Some(left), Some(right)) => #key_comparisons,
                            _ => false,
                        }
                    },)* _ => false }
                }
                /// Primary-key lookup bucket. Always confirm with same_key before reuse.
                pub fn key_bucket(&self, index: usize) -> Option<pse_ids::ContentHash> {
                    let mut hash = pse_ids::FramedHasher::new("pse:typed-row-key:v1");
                    hash.id(&self.relation());
                    match self { #(Self::#variants(rows) => { let row = rows.get(index)?; #key_frames },)* }
                    Some(hash.finish_hash())
                }
                /// Concatenate ordered batches of the same declared relation.
                /// # Errors
                /// The incoming relation identity differs.
                pub fn append(&mut self, other: Self) -> Result<(), crate::ModelError> { match (self, other) { #((Self::#variants(left), Self::#variants(mut right)) => { left.append(&mut right); Ok(()) },)* _ => Err(crate::malformed("phase relation identity differs")) } }
            }
            impl crate::HeapUsage for FactBatch {
                fn heap_bytes(&self) -> usize { match self { #(Self::#variants(rows) => crate::HeapUsage::heap_bytes(rows),)* } }
            }
        },
    )?;
    native_facts(tree, &variants, &namespaces, &names, &ids)?;
    for root in [MODEL, NATIVE] {
        let path = PathBuf::from(root).join("mod.rs");
        let bytes = tree
            .files
            .get_mut(&path)
            .ok_or_else(|| super::error("missing fact namespace root".into()))?;
        bytes.extend_from_slice(b"\n/// Generated typed relational handoff.\npub mod facts;\n");
    }
    Ok(())
}

fn native_facts(
    tree: &mut GeneratedTree,
    variants: &[proc_macro2::Ident],
    namespaces: &[proc_macro2::Ident],
    names: &[proc_macro2::Ident],
    ids: &[TokenStream],
) -> Result<(), SchemaError> {
    super::emit(
        tree,
        format!("{NATIVE}/facts.rs"),
        quote! {
            /// Decode exact checked native values into generated typed phase transport.
            /// # Errors
            /// Unknown relation, incompatible checked owner or malformed physical values.
            pub fn decode(batch: &crate::columnar::FieldCheckedBatch) -> Result<pse_model::generated::facts::FactBatch, crate::RelationError> {
                use crate::columnar::RelationRow;
                #(if batch.relation_id() == #ids { return Ok(pse_model::generated::facts::FactBatch::#variants(super::#namespaces::#names::Row::rows(batch)?)); })*
                Err(crate::columnar::mismatch("registered phase relation"))
            }
            /// Materialize one complete typed relation through the shared budgeted
            /// native builders. Empty membership, order and repeated rows are retained.
            /// This establishes local fields/values, not relational key/reference validity.
            /// # Errors
            /// Invalid local values, cancellation or native allocation refusal.
            pub fn encode(
                values: &pse_model::generated::facts::FactBatch,
                registry: &pse_schema::Registry,
                pool: &std::sync::Arc<dyn pse_columnar::MemoryPool>,
                cancel: &pse_columnar::CancellationToken,
            ) -> Result<crate::columnar::FieldCheckedBatch, crate::RelationError> {
                match values {
                    #(pse_model::generated::facts::FactBatch::#variants(rows) =>
                        crate::columnar::encode_rows(rows, registry, pool, cancel),)*
                }
            }
        },
    )
}

fn key_fragments(reg: &crate::Registry) -> (Vec<TokenStream>, Vec<TokenStream>) {
    reg.relations()
        .iter()
        .map(|spec| {
            let fields = spec
                .primary_key
                .iter()
                .map(|name| super::types::ident(name))
                .collect::<Vec<_>>();
            if fields.is_empty() {
                (
                    quote! { let _ = (left, right); false },
                    quote! { let _ = row; return None; },
                )
            } else {
                let comparisons = fields.iter().map(
                    |field| quote!(crate::SemanticEq::semantic_eq(&left.#field, &right.#field)),
                );
                (
                    quote!(#(#comparisons)&&*),
                    quote!(#(crate::SemanticFrame::frame(&row.#fields, &mut hash);)*),
                )
            }
        })
        .unzip()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn path_rewrite_preserves_literals_and_qualified_variant_suffixes() {
        use syn::visit_mut::VisitMut;
        let mut file: syn::File = syn::parse_quote! {
            fn check(value: crate::RelationError) -> crate::RelationError {
                let _text = "crate :: RelationError crate :: columnar :: mismatch";
                let _other = unrelated::RelationError::Variant;
                let _value = crate::columnar::mismatch("x");
                crate::RelationError::EnumMember { value }
            }
        };
        ModelPaths.visit_file_mut(&mut file);
        let expected: syn::File = syn::parse_quote! {
            fn check(value: crate::ModelError) -> crate::ModelError {
                let _text = "crate :: RelationError crate :: columnar :: mismatch";
                let _other = unrelated::RelationError::Variant;
                let _value = crate::malformed("x");
                crate::ModelError::EnumMember { value }
            }
        };
        assert_eq!(file, expected);
    }
}
