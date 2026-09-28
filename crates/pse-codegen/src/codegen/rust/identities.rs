// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed entity ids generated from the registry's identity declarations (ADR-0115).
//!
//! `pse-model` receives the newtypes, declared through `pse_ids::semantic_id_newtype!`,
//! with value equality, heap usage and framing identical to the base value's, so a
//! generated row that types its identity columns frames exactly as before. `pse-relations`
//! receives their Arrow codecs, which delegate to the base value's.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::Registry;
use crate::model::IdentityBase;

use super::types::pascal;

const MODEL: &str = "crates/pse-model/src/generated/identities.rs";
const NATIVE: &str = "crates/pse-relations/src/generated/identities.rs";

/// The generated Rust type name of an identity: `attempt` is `AttemptId`.
pub(crate) fn type_name(identity: &str) -> String {
    format!("{}Id", pascal(identity))
}

/// The path a generated row uses for the identity's type, in either crate.
pub(super) fn path(identity: &str) -> TokenStream {
    let name = format_ident!("{}", type_name(identity));
    quote!(crate::generated::identities::#name)
}

pub(super) fn emit(tree: &mut crate::codegen::GeneratedTree, reg: &Registry) -> Result<(), crate::SchemaError> {
    let mut declarations = Vec::new();
    let mut model_traits = Vec::new();
    let mut codecs = Vec::new();
    let mut names = Vec::new();
    for identity in reg.identities() {
        let name = format_ident!("{}", type_name(identity.name));
        let doc = format!(
            "{} (entity identity `{}`, ADR-0115).",
            identity.doc.trim_end_matches('.'),
            identity.name
        );
        let (declaration, base) = match identity.base {
            IdentityBase::SemanticId => (quote!(#[doc = #doc] #name), quote!(pse_ids::SemanticId)),
            IdentityBase::ContentHash => (
                quote!(#[doc = #doc] #name: ContentHash),
                quote!(pse_ids::ContentHash),
            ),
        };
        declarations.push(quote!(pse_ids::semantic_id_newtype! { #declaration }));
        model_traits.push(quote! {
            impl crate::SemanticEq for #name {
                fn semantic_eq(&self, other: &Self) -> bool { self == other }
            }
            impl crate::HeapUsage for #name {
                fn heap_bytes(&self) -> usize { 0 }
            }
            impl crate::SemanticFrame for #name {
                fn frame(&self, hash: &mut pse_ids::FramedHasher) {
                    crate::SemanticFrame::frame(&self.as_id(), hash);
                }
            }
        });
        codecs.push(quote! {
            impl crate::columnar::ArrowValue for #name {
                fn append(&self, output: &mut dyn arrow_array::builder::ArrayBuilder) -> Result<(), crate::RelationError> {
                    crate::columnar::ArrowValue::append(&self.as_id(), output)
                }
                fn append_null(output: &mut dyn arrow_array::builder::ArrayBuilder) -> Result<(), crate::RelationError> {
                    <#base as crate::columnar::ArrowValue>::append_null(output)
                }
                fn read(input: &dyn arrow_array::Array, index: usize) -> Result<Self, crate::RelationError> {
                    <#base as crate::columnar::ArrowValue>::read(input, index).map(Self::from_id)
                }
            }
        });
        names.push(name);
    }
    super::emit(
        tree,
        MODEL,
        quote! {
            #(#declarations)*
            #(#model_traits)*
        },
    )?;
    super::emit(
        tree,
        NATIVE,
        quote! {
            /// Registry-generated typed ids; their Arrow codecs are the base value's.
            pub use pse_model::generated::identities::{#(#names),*};
            #(#codecs)*
        },
    )
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "pure generation over a fixture registry")]

    use std::path::PathBuf;

    use crate::codegen::{Language, generate};
    use crate::model::{
        Authority, EnumDecl, EnumMember, FieldContract as F, IdentityDecl, Namespace,
        RelationDecl, SnapshotClass,
    };
    use crate::{Registry, RegistryBuilder};

    fn relation(name: &'static str, key: &'static str, columns: Vec<F>) -> RelationDecl {
        RelationDecl::new(
            Namespace::Authored,
            name,
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "a fixture relation",
        )
        .pk(&[key])
        .columns(columns)
    }

    fn fixture() -> Registry {
        let mut builder = RegistryBuilder::new();
        builder
            .declare_enum(EnumDecl::platform(
                "BoundKind",
                vec![
                    EnumMember::new("finite", "finite"),
                    EnumMember::new("unbounded", "unbounded"),
                ],
            ))
            .declare_identity(IdentityDecl::new("widget", "A widget"))
            .declare_identity(IdentityDecl::new("bundle", "A content-addressed bundle"))
            .declare_relation(relation(
                "widgets",
                "widget_id",
                vec![
                    F::key("widget_id", F::id(), "the widget").with_identity("widget"),
                    F::payload(
                        "seen_at",
                        F::native(crate::model::extension::timestamp_micros_storage()),
                        "an instant",
                    ),
                    F::payload("bundle", F::hash(), "the bundle").with_identity("bundle"),
                ],
            ))
            .declare_relation(relation(
                "parts",
                "part_id",
                vec![
                    F::key("part_id", F::id(), "the part"),
                    F::reference("widget_id", F::id(), "the widget")
                        .with_fk("authored.widgets", "widget_id")
                        .optional(),
                ],
            ));
        builder.build().unwrap()
    }

    fn text(tree: &crate::codegen::GeneratedTree, path: &str) -> String {
        String::from_utf8(tree.files[&PathBuf::from(path)].clone()).unwrap()
    }

    #[test]
    fn identity_columns_render_typed_ids() {
        let registry = fixture();
        let rust = generate(&registry, Language::Rust).unwrap();
        let ids = text(&rust, "crates/pse-model/src/generated/identities.rs");
        assert!(ids.contains("pse_ids::semantic_id_newtype!"), "{ids}");
        let tokens = ids.split_whitespace().collect::<String>();
        assert!(tokens.contains("WidgetId}"), "{ids}");
        assert!(tokens.contains("BundleId:ContentHash}"), "{ids}");
        assert!(ids.contains("impl crate::SemanticFrame for WidgetId"));
        let codecs = text(&rust, "crates/pse-relations/src/generated/identities.rs");
        assert!(
            codecs.contains("pub use pse_model::generated::identities::{BundleId, WidgetId}"),
            "{codecs}"
        );
        assert!(codecs.contains("impl crate::columnar::ArrowValue for WidgetId"));
        // The owning key, the inheriting reference and an unowned carrier are typed.
        let parts = text(&rust, "crates/pse-model/src/generated/authored/parts.rs");
        assert!(
            parts.contains("Option<crate::generated::identities::WidgetId>"),
            "{parts}"
        );
        assert!(parts.contains("pub r#part_id: pse_ids::SemanticId"), "{parts}");
        let widgets = text(&rust, "crates/pse-model/src/generated/authored/widgets.rs");
        assert!(widgets.contains("pub r#widget_id: crate::generated::identities::WidgetId"));
        assert!(widgets.contains("pub r#bundle: crate::generated::identities::BundleId"));
        for root in [
            "crates/pse-model/src/generated",
            "crates/pse-relations/src/generated",
        ] {
            assert!(text(&rust, &format!("{root}/mod.rs")).contains("pub mod identities;"));
        }
        let python = generate(&registry, Language::Python).unwrap();
        let aliases = text(&python, "python/pse/contracts/identities.py");
        assert!(aliases.contains("WidgetId = NewType(\"WidgetId\", v.SemanticId)"));
        assert!(aliases.contains("BundleId = NewType(\"BundleId\", v.ContentHash)"));
        let rows = text(&python, "python/pse/contracts/authored.py");
        assert!(rows.contains("from pse.contracts import identities as i"));
        assert!(rows.contains("widget_id: i.WidgetId | None"), "{rows}");
    }

    /// A value nested in a column carries an identity without owning it; the nested
    /// struct field is typed in Rust and annotated with the alias in Python.
    #[test]
    fn nested_identity_values_render_typed_ids() {
        let mut builder = RegistryBuilder::new();
        builder
            .declare_enum(EnumDecl::platform(
                "BoundKind",
                vec![
                    EnumMember::new("finite", "finite"),
                    EnumMember::new("unbounded", "unbounded"),
                ],
            ))
            .declare_identity(IdentityDecl::new("widget", "A widget"))
            .declare_relation(relation(
                "boxes",
                "box_id",
                vec![
                    F::key("box_id", F::id(), "the box"),
                    F::payload(
                        "contents",
                        F::list(F::structure(vec![
                            F::id().with_name("widget_id").with_identity("widget"),
                            F::id().with_name("label_id").optional(),
                        ])),
                        "the widgets in the box",
                    ),
                ],
            ));
        let registry = builder.build().unwrap();
        assert_eq!(registry.identity("widget").unwrap().owner, None);
        let rust = generate(&registry, Language::Rust).unwrap();
        let model = text(&rust, "crates/pse-model/src/generated/authored/boxes.rs");
        assert!(
            model.contains("pub r#widget_id: crate::generated::identities::WidgetId"),
            "{model}"
        );
        assert!(
            model.contains("pub r#label_id: Option<pse_ids::SemanticId>"),
            "{model}"
        );
        let python = generate(&registry, Language::Python).unwrap();
        let rows = text(&python, "python/pse/contracts/authored.py");
        assert!(rows.contains("from pse.contracts import identities as i"), "{rows}");
        assert!(rows.contains("widget_id: i.WidgetId"), "{rows}");
    }

    #[test]
    fn microsecond_timestamps_render_codecs() {
        let registry = fixture();
        let rust = generate(&registry, Language::Rust).unwrap();
        let native = text(&rust, "crates/pse-relations/src/generated/authored/widgets.rs");
        assert!(
            native.contains("arrow_array::TimestampMicrosecondArray"),
            "{native}"
        );
        let model = text(&rust, "crates/pse-model/src/generated/authored/widgets.rs");
        assert!(model.contains("pub r#seen_at: i64"), "{model}");
        let python = generate(&registry, Language::Python).unwrap();
        assert!(text(&python, "python/pse/contracts/authored.py").contains("seen_at: datetime"));
    }
}
