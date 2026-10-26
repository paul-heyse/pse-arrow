// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Opaque canonical text keys and native table classifications from declarations.
use quote::{format_ident, quote};
use crate::model::CanonicalKeyKind;

pub(crate) fn type_name(kind: CanonicalKeyKind) -> String { format!("{}Key", super::types::pascal(kind.as_str())) }

#[cfg(test)]
mod tests {
    use crate::codegen::{generate, Language};
    #[test]
    fn canonical_generated_contract_retains_nominal_text_keys_and_original_inline_owner() {
        let registry = crate::catalog::assemble().unwrap();
        let tree = generate(&registry, Language::Rust).unwrap();
        let output = std::str::from_utf8(&tree.files[std::path::Path::new("crates/pse-model/src/generated/canonical.rs")]).unwrap();
        assert!(output.contains("pub struct RunKey(String)"));
        assert!(output.contains("pub struct AttemptKey(String)"));
        assert!(!output.contains("SemanticId"));
        assert!(output.contains("\"canonical_inline_payloads\""));
        assert!(!output.split("APPLICATION_BACKUP_TABLES").nth(1).unwrap().split(';').next().unwrap().contains("canonical_file_owners"));
        let enums = std::str::from_utf8(&tree.files[std::path::Path::new("crates/pse-model/src/generated/enums.rs")]).unwrap();
        assert!(enums.contains("pub const fn attempt_state(self) -> AttemptState"));
        assert!(enums.contains("pub const fn study_point_state(self, partial_usable: bool)"));
    }
    #[test]
    fn same_field_spelling_does_not_invent_a_lookup_domain_for_scientific_bytes() {
        let registry = crate::catalog::assemble().unwrap();
        let scientific = registry.relation("runtime.canonical_research_selections").unwrap().column("selection").unwrap();
        assert_eq!(scientific.data_type(), arrow_schema::DataType::Binary);
        assert_eq!(scientific.canonical_key(), None);
        let lookup = registry.relation("runtime.canonical_research_publications").unwrap().column("selection").unwrap();
        assert_eq!(lookup.data_type(), arrow_schema::DataType::Utf8);
        assert_eq!(lookup.canonical_key(), Some(crate::model::CanonicalKeyKind::ResearchSelection));
    }
}
pub(super) fn emit(tree: &mut crate::codegen::GeneratedTree, registry: &crate::Registry) -> Result<(), crate::SchemaError> {
    let keys = CanonicalKeyKind::ALL.iter().copied().map(|kind| {
        let name = format_ident!("{}", type_name(kind));
        let constructors = kind.constructors().iter().map(|constructor| {
            let method = format_ident!("{}", constructor.method);
            let input = format_ident!("{}", type_name(constructor.input));
            let prefix = constructor.prefix;
            quote! {
                /// Construct the declared exact opaque association key.
                pub fn #method(input: &#input) -> Self { Self(format!("{}{}", #prefix, input.as_str())) }
            }
        });
        quote! {
            /// An opaque canonical lookup key; never a scientific semantic identity.
            #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
            #[serde(transparent)]
            pub struct #name(String);
            impl #name {
                #(#constructors)*
                /// Preserve the exact boundary spelling without inferring authority.
                pub fn from_text(value: impl Into<String>) -> Self { Self(value.into()) }
                /// Borrow the exact lookup spelling.
                pub fn as_str(&self) -> &str { &self.0 }
                /// Recover the owned transport spelling.
                pub fn into_string(self) -> String { self.0 }
            }
            impl From<String> for #name { fn from(value: String) -> Self { Self(value) } }
            impl From<&str> for #name { fn from(value: &str) -> Self { Self(value.to_owned()) } }
            impl From<#name> for String { fn from(value: #name) -> Self { value.0 } }
            impl std::ops::Deref for #name { type Target = str; fn deref(&self) -> &str { &self.0 } }
            impl AsRef<str> for #name { fn as_ref(&self) -> &str { &self.0 } }
            impl std::borrow::Borrow<str> for #name { fn borrow(&self) -> &str { &self.0 } }
            impl std::fmt::Display for #name { fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { out.write_str(&self.0) } }
            impl crate::SemanticEq for #name { fn semantic_eq(&self, other: &Self) -> bool { self == other } }
            impl crate::HeapUsage for #name { fn heap_bytes(&self) -> usize { self.0.capacity() } }
            impl crate::SemanticFrame for #name { fn frame(&self, hash: &mut pse_ids::FramedHasher) { hash.str(&self.0); } }
        }
    });
    let required = registry.relations().iter().filter(|relation| relation.canonical.is_some_and(|policy| policy.backup == crate::model::CanonicalBackup::Required)).map(|relation| relation.key.name).collect::<Vec<_>>();
    let classes = registry.relations().iter().filter_map(|relation| relation.canonical.map(|policy| {
        let table = relation.key.name;
        let retention = format!("{:?}", policy.retention);
        let backup = format!("{:?}", policy.backup);
        quote!((#table, #retention, #backup))
    }));
    super::emit(tree, "crates/pse-model/src/generated/canonical.rs", quote! {
        #(#keys)*
        /// Required original application owners, including empty tables.
        pub const APPLICATION_BACKUP_TABLES: &[&str] = &[#(#required),*];
        /// Native table lifetime and backup classifications.
        pub const TABLE_CLASSIFICATIONS: &[(&str, &str, &str)] = &[#(#classes),*];
    })?;
    let names = CanonicalKeyKind::ALL.iter().copied().map(|kind| format_ident!("{}", type_name(kind))).collect::<Vec<_>>();
    let codecs = names.iter().map(|name| quote! {
        impl crate::columnar::ArrowValue for #name {
            fn append(&self, output: &mut dyn arrow_array::builder::ArrayBuilder) -> Result<(), crate::RelationError> { crate::columnar::append_string(output, Some(self.as_str())) }
            fn append_null(output: &mut dyn arrow_array::builder::ArrayBuilder) -> Result<(), crate::RelationError> { crate::columnar::append_string(output, None) }
            fn read(input: &dyn arrow_array::Array, index: usize) -> Result<Self, crate::RelationError> { crate::columnar::read_string(input, index).map(Self::from) }
        }
    });
    super::emit(tree, "crates/pse-relations/src/generated/canonical.rs", quote! {
        pub use pse_model::generated::canonical::{#(#names),*};
        #(#codecs)*
    })
}
