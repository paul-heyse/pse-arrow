// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Recursive manifest wire projection; codecs retain the catalog's native role types.

use std::collections::BTreeSet;

use proc_macro2::{Ident, TokenStream};
use quote::quote;

use crate::SchemaError;
use crate::model::{ManifestField, ManifestRustBinding, ManifestSpec, ManifestType};

use super::{error, types};

pub(super) fn render(spec: &ManifestSpec) -> Result<TokenStream, SchemaError> {
    let mut emitter = Emitter::default();
    emitter.object("Manifest", spec.fields())?;
    let declarations = emitter.declarations;
    let version = spec.version;
    Ok(quote! {
        //! Generated envelope shape. Identity and semantic validity require catalog admission.
        /// The declared manifest wire version.
        pub const MANIFEST_VERSION: &str = #version;
        #(#declarations)*
    })
}

#[derive(Default)]
struct Emitter {
    declarations: Vec<TokenStream>,
    names: BTreeSet<String>,
}

impl Emitter {
    fn object(&mut self, stem: &str, fields: &[ManifestField]) -> Result<TokenStream, SchemaError> {
        if !self.names.insert(stem.to_owned()) {
            return Err(error(format!("manifest Rust type name collision: {stem}")));
        }
        let name = identifier(stem)?;
        let fields = fields
            .iter()
            .map(|field| self.field(stem, field))
            .collect::<Result<Vec<_>, _>>()?;
        let doc = format!("Declared manifest object `{stem}`; decoded values require admission.");
        self.declarations.push(quote! {
            #[doc = #doc]
            #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
            #[serde(deny_unknown_fields)]
            #[allow(clippy::struct_field_names, reason = "field names are the declared manifest wire contract")]
            pub struct #name { #(#fields)* }
        });
        Ok(quote!(#name))
    }

    fn field(&mut self, parent: &str, field: &ManifestField) -> Result<TokenStream, SchemaError> {
        let name = identifier(&format!("r#{}", field.name))?;
        let stem = format!("{parent}{}", types::pascal(field.name));
        // Emit nested wire declarations even when a native codec owns the outer value.
        let declared = self.ty(&stem, &field.ty)?;
        let (ty, codec) = if let Some(binding) = field.rust {
            if !binding.accepts(&field.ty) {
                return Err(error(format!(
                    "incompatible manifest native binding at {stem}"
                )));
            }
            native(binding)
        } else {
            (declared, wire_codec(&field.ty))
        };
        let codec = codec.map(|path| quote!(#[serde(with = #path)]));
        let wire_name = field.name;
        let doc = field.doc;
        Ok(quote! {
            #[doc = #doc]
            #[serde(rename = #wire_name)]
            #codec
            pub #name: #ty,
        })
    }

    fn ty(&mut self, stem: &str, ty: &ManifestType) -> Result<TokenStream, SchemaError> {
        Ok(match ty {
            ManifestType::Text | ManifestType::Timestamp => quote!(String),
            ManifestType::U32 => quote!(u32),
            ManifestType::U64 => quote!(u64),
            ManifestType::Bool => quote!(bool),
            ManifestType::Id => quote!(pse_ids::SemanticId),
            ManifestType::Hash => quote!(pse_ids::ContentHash),
            ManifestType::List(inner) => {
                let inner = self.ty(&format!("{stem}Item"), inner)?;
                quote!(Vec<#inner>)
            }
            ManifestType::Optional(inner) => {
                let inner = self.ty(stem, inner)?;
                quote!(Option<#inner>)
            }
            ManifestType::Struct(fields) => self.object(stem, fields)?,
        })
    }
}

fn identifier(name: &str) -> Result<Ident, SchemaError> {
    syn::parse_str(name).map_err(|cause| {
        error(format!(
            "unsupported manifest Rust identifier {name}: {cause}"
        ))
    })
}

fn wire_codec(ty: &ManifestType) -> Option<&'static str> {
    match ty {
        ManifestType::Id => Some("crate::store::manifest::semantic_id_text"),
        ManifestType::Hash => Some("crate::store::manifest::hash_text"),
        _ => None,
    }
}

fn native(binding: ManifestRustBinding) -> (TokenStream, Option<&'static str>) {
    let (ty, codec) = match binding {
        ManifestRustBinding::SnapshotKind => (
            quote!(pse_ids::SnapshotKind),
            "crate::store::manifest::snapshot_kind_text",
        ),
        ManifestRustBinding::SnapshotId => (
            quote!(pse_ids::SnapshotId),
            "crate::store::manifest::snapshot_id_text",
        ),
        ManifestRustBinding::LogicalHash => (
            quote!(pse_ids::LogicalHash),
            "crate::store::manifest::logical_hash_text",
        ),
        ManifestRustBinding::EncodingChecksum => (
            quote!(pse_ids::EncodingChecksum),
            "crate::store::manifest::encoding_checksum_text",
        ),
        ManifestRustBinding::SchemaVersion => (
            quote!(pse_ids::SchemaVersion),
            "crate::store::manifest::schema_version_number",
        ),
        ManifestRustBinding::EncodingFormat => (
            quote!(crate::store::layout::EncodingFormat),
            "crate::store::manifest::encoding_format_text",
        ),
        ManifestRustBinding::SnapshotParents => (
            quote!(Vec<pse_ids::SnapshotParent>),
            "crate::store::manifest::parents_wire",
        ),
    };
    (ty, Some(codec))
}
