// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Operation-shaped native values derived from the field policy owner.
use crate::{
    SchemaError,
    model::{CanonicalCreatePresence, CanonicalInputPolicy, RelationSpec},
};
use arrow_schema::DataType;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::fmt::Write as _;

fn projection_rust_type(
    root: &str,
    graph: Option<&pse_contract::boundary_projection::ProjectionGraph>,
) -> Result<TokenStream, SchemaError> {
    let owner = graph
        .and_then(|graph| graph.definitions().get(root))
        .ok_or_else(|| SchemaError::Codegen {
            language: "surreal",
            reason: format!("source-owned native projection {root} is unavailable"),
        })?;
    let path = owner.public_rust_path.as_deref().unwrap_or(owner.rust_type);
    path.parse()
        .map_err(|error: proc_macro2::LexError| SchemaError::Codegen {
            language: "surreal",
            reason: error.to_string(),
        })
}

pub(super) fn rust_field_type(
    field: &pse_schema::model::FieldContract,
    graph: Option<&pse_contract::boundary_projection::ProjectionGraph>,
) -> Result<TokenStream, SchemaError> {
    let base = if let Some(root) = field.canonical_projection() {
        if field.value_type().data_type() != DataType::Binary {
            return Err(SchemaError::Codegen {
                language: "surreal",
                reason: format!(
                    "native projection {} requires its declared portable binary boundary",
                    field.name()
                ),
            });
        }
        projection_rust_type(root, graph)?
    } else if let Some(domain) = field.enum_name() {
        let name = format_ident!("{}", super::rust::types::pascal(domain));
        quote!(pse_model::generated::enums::#name)
    } else {
        match field.value_type().data_type() {
            DataType::Utf8 => quote!(String),
            DataType::Binary => quote!(pse_model::Bytes),
            DataType::FixedSizeBinary(16) => quote!(pse_ids::SemanticId),
            DataType::UInt64 => quote!(u64),
            DataType::Int64 | DataType::Timestamp(_, _) => quote!(i64),
            DataType::Boolean => quote!(bool),
            DataType::Float64 => quote!(f64),
            other => {
                return Err(SchemaError::Codegen {
                    language: "surreal",
                    reason: format!("unsupported native row field {other:?}"),
                });
            }
        }
    };
    Ok(if field.nullable() {
        quote!(Option<#base>)
    } else {
        base
    })
}

pub(super) fn definitions(relation: &RelationSpec) -> Result<String, SchemaError> {
    let mut create = Vec::new();
    let mut replace = Vec::new();
    let mut merge_checks = Vec::new();
    let mut merge_keys = Vec::new();
    for field in &relation.columns {
        let policy = CanonicalInputPolicy::for_field(field)?;
        let base = if let Some(root) = field.canonical_projection() {
            super::boundary_projection::surreal::native_type_name(root)
        } else if let Some(domain) = field.enum_name() {
            super::surreal::native_enum_name(domain)
        } else {
            match field.value_type().data_type() {
                DataType::Utf8 => "string",
                DataType::Binary | DataType::FixedSizeBinary(16) => "bytes",
                DataType::UInt64 => "u64",
                DataType::Int64 | DataType::Timestamp(_, _) => "int",
                DataType::Boolean => "bool",
                DataType::Float64 => "{bits: bytes, projection: float}",
                other => {
                    return Err(SchemaError::Codegen {
                        language: "surreal",
                        reason: format!("unsupported native input shape: {other:?}"),
                    });
                }
            }
            .to_owned()
        };
        let optional = format!("option<{base}>");
        let name = field.name();
        let create_kind = if policy.create == CanonicalCreatePresence::Required {
            &base
        } else {
            &optional
        };
        create.push(format!("{name}: {create_kind}"));
        if policy.mutable {
            merge_keys.push(format!("'{name}'"));
            let admitted = format!("(<{base}>$value.{name})=$value.{name}");
            let supplied = if policy.removable {
                format!("($value.{name}=NONE OR ({admitted}))")
            } else {
                format!("($value.{name}!=NONE AND ({admitted}))")
            };
            merge_checks.push(format!(
                "('{name}' NOT IN object::keys($value) OR {supplied})"
            ));
            replace.push(format!(
                "{name}: {}",
                if policy.removable { &optional } else { &base }
            ));
        }
        if let Some(target) = field
            .fk()
            .and_then(|fk| fk.relation.strip_prefix("runtime."))
        {
            let link = field.graph_endpoint().map(str::to_owned).or_else(|| {
                relation
                    .canonical
                    .filter(|p| p.generation_references)
                    .map(|_| format!("{name}_record"))
            });
            if let Some(link) = link {
                let base = format!("record<{target}>");
                let optional = format!("option<{base}>");
                create.push(format!(
                    "{link}: {}",
                    if policy.create == CanonicalCreatePresence::Required {
                        &base
                    } else {
                        &optional
                    }
                ));
                if policy.mutable {
                    merge_keys.push(format!("'{link}'"));
                    let admitted = format!("(<{base}>$value.{link})=$value.{link}");
                    let supplied = if policy.removable {
                        format!("($value.{link}=NONE OR ({admitted}))")
                    } else {
                        format!("($value.{link}!=NONE AND ({admitted}))")
                    };
                    merge_checks.push(format!(
                        "('{link}' NOT IN object::keys($value) OR {supplied})"
                    ));
                    replace.push(format!(
                        "{link}: {}",
                        if policy.removable { &optional } else { &base }
                    ));
                }
            }
        }
    }
    let mut ddl = String::new();
    for (operation, fields) in [("create", create), ("replace", replace)] {
        let _ = writeln!(
            ddl,
            "DEFINE TYPE pse_{}_{operation} AS {{{}}};",
            relation.key.name,
            fields.join(", ")
        );
    }
    // Literal object coercion may populate optional keys with NONE. Merge must
    // preserve original membership so absence remains Keep, not Remove.
    let mut assertions = vec![format!(
        "object::keys($value) ALLINSIDE [{}]",
        merge_keys.join(", ")
    )];
    assertions.extend(merge_checks);
    let _ = writeln!(
        ddl,
        "DEFINE TYPE pse_{}_merge AS object ASSERT {};",
        relation.key.name,
        assertions.join(" AND ")
    );
    Ok(ddl)
}

pub(super) fn support() -> TokenStream {
    quote! {
        /// Mutation of a required value; omission leaves the original unchanged.
        #[derive(Clone, Debug, Default)]
        pub enum NativeUpdate<T> {
            /// Preserve the original value.
            #[default]
            Keep,
            /// Supply the new declared value.
            Set(T),
        }
        /// Mutation of an optional value; removal is distinct from omission.
        #[derive(Clone, Debug, Default)]
        pub enum NativeMerge<T> {
            /// Preserve the original value.
            #[default]
            Keep,
            /// Supply a present declared value.
            Set(T),
            /// Remove the optional field using native NONE.
            Remove,
        }
    }
}

pub(super) fn relation(
    relation: &RelationSpec,
    row: &TokenStream,
    graph: Option<&pse_contract::boundary_projection::ProjectionGraph>,
) -> Result<TokenStream, SchemaError> {
    let stem = format!("Runtime{}", super::rust::types::pascal(relation.key.name));
    let create = format_ident!("{stem}Create");
    let merge = format_ident!("{stem}Merge");
    let replace = format_ident!("{stem}Replace");
    let create_fn = format_ident!("create_{}", relation.key.name);
    let merge_fn = format_ident!("merge_{}", relation.key.name);
    let replace_fn = format_ident!("replace_{}", relation.key.name);
    let mut create_fields = Vec::new();
    let mut create_from = Vec::new();
    let mut create_values = Vec::new();
    let mut merge_fields = Vec::new();
    let mut merge_values = Vec::new();
    let mut replace_fields = Vec::new();
    let mut replace_values = Vec::new();
    for field in &relation.columns {
        let name = field.name();
        let ident = format_ident!("r#{name}");
        let policy = CanonicalInputPolicy::for_field(field)?;
        let (base, codec) = match field.value_type().data_type() {
            DataType::Utf8 => (quote!(String), "string"),
            DataType::Binary => (quote!(pse_model::Bytes), "bytes"),
            DataType::FixedSizeBinary(16) => (quote!(pse_ids::SemanticId), "id"),
            DataType::UInt64 => (quote!(u64), "uint"),
            DataType::Int64 | DataType::Timestamp(_, _) => (quote!(i64), "int"),
            DataType::Boolean => (quote!(bool), "boolean"),
            DataType::Float64 => (quote!(f64), "finite"),
            other => {
                return Err(SchemaError::Codegen {
                    language: "surreal",
                    reason: format!("unsupported input field {name}: {other:?}"),
                });
            }
        };
        let base = if let Some(root) = field.canonical_projection() {
            projection_rust_type(root, graph)?
        } else if let Some(identity) = field.identity() {
            let ty = format_ident!("{}", super::rust::identities::type_name(identity));
            quote!(pse_model::generated::identities::#ty)
        } else if let Some(domain) = field.enum_name() {
            let ty = format_ident!("{}", super::rust::types::pascal(domain));
            quote!(pse_model::generated::enums::#ty)
        } else {
            base
        };
        let encode = format_ident!("encode_{codec}");
        let owned = if matches!(codec, "uint" | "int" | "boolean" | "finite" | "id") {
            quote!(*value)
        } else {
            quote!(value.clone())
        };
        let value = if let Some(root) = field.canonical_projection() {
            let encode = format_ident!("encode_{}", super::boundary_projection::snake_case(root));
            quote!(pse_study_module::generated::#encode(value.clone()).map_err(|_|crate::canonical_codec::CodecError::Domain(#name))?)
        } else if field.enum_name().is_some() {
            quote!(crate::canonical_codec::encode_string(
                value.as_str().to_owned()
            )?)
        } else {
            quote!(crate::canonical_codec::#encode(#owned)?)
        };
        let optional_create = policy.create != CanonicalCreatePresence::Required;
        let ty = if optional_create {
            quote!(Option<&'a #base>)
        } else {
            quote!(&'a #base)
        };
        create_fields.push(quote!(#[doc = #name] pub #ident: #ty));
        let from = if optional_create && !field.nullable() {
            quote!(Some(&row.#ident))
        } else if optional_create {
            quote!(row.#ident.as_ref())
        } else {
            quote!(&row.#ident)
        };
        create_from.push(quote!(#ident: #from));
        let insertion = quote!(object.insert(#name, #value););
        let mut projections = Vec::new();
        let mut removals = Vec::new();
        if let Some(target) = field
            .fk()
            .and_then(|fk| fk.relation.strip_prefix("runtime."))
        {
            if let Some(endpoint) = field.graph_endpoint() {
                projections.push(quote!(object.insert(#endpoint, crate::canonical_codec::record_link(#target, value));));
                removals.push(quote!(object.insert(#endpoint, surrealdb::types::Value::None);));
            } else if relation.canonical.is_some_and(|p| p.generation_references) {
                let link = format!("{name}_record");
                projections.push(quote!(object.insert(#link, crate::canonical_codec::record_link(#target, value));));
                removals.push(quote!(object.insert(#link, surrealdb::types::Value::None);));
            }
        }
        create_values.push(if optional_create {
            quote!(if let Some(value) = input.#ident { #insertion #(#projections)* })
        } else {
            quote!({ let value = input.#ident; #insertion #(#projections)* })
        });
        if policy.mutable {
            let merge_ty = if policy.removable {
                quote!(NativeMerge<#base>)
            } else {
                quote!(NativeUpdate<#base>)
            };
            let carrier = if policy.removable {
                quote!(NativeMerge)
            } else {
                quote!(NativeUpdate)
            };
            merge_fields.push(quote!(#[doc = #name] pub #ident: #merge_ty));
            let remove = if policy.removable {
                quote!(NativeMerge::Remove => { object.insert(#name, surrealdb::types::Value::None); #(#removals)* })
            } else {
                quote!()
            };
            merge_values.push(quote!(match &input.#ident { #carrier::Keep => {}, #carrier::Set(value) => { #insertion #(#projections)* }, #remove }));
            let ty = if policy.removable {
                quote!(Option<#base>)
            } else {
                base
            };
            replace_fields.push(quote!(#[doc = #name] pub #ident: #ty));
            replace_values.push(if policy.removable {
                quote!(if let Some(value) = &input.#ident { #insertion #(#projections)* } else { object.insert(#name, surrealdb::types::Value::None); #(#removals)* })
            } else { quote!({ let value = &input.#ident; #insertion #(#projections)* }) });
        }
    }
    let mutable_object = if merge_values.is_empty() {
        quote!(let object = surrealdb::types::Object::new(); let _ = input;)
    } else {
        quote!(let mut object = surrealdb::types::Object::new();)
    };
    Ok(quote! {
        /// Creation input, including immutable values admitted at initialization.
        #[derive(Clone, Debug)]
        #[allow(clippy::struct_field_names, reason = "field names are the declared operation input")]
        pub struct #create<'a> { #(#create_fields,)* }
        impl<'a> From<&'a #row> for #create<'a> {
            fn from(row: &'a #row) -> Self { Self { #(#create_from,)* } }
        }
        /// Partial mutation of mutable fields; default leaves all fields unchanged.
        #[derive(Clone, Debug, Default)]
        #[allow(clippy::struct_field_names, reason = "field names are the declared operation input")]
        pub struct #merge { #(#merge_fields,)* }
        /// Complete mutable payload; immutable context is supplied by the operation owner.
        #[derive(Clone, Debug)]
        #[allow(clippy::struct_field_names, reason = "field names are the declared operation input")]
        pub struct #replace { #(#replace_fields,)* }
        /// Convert creation intent to native values; database admission remains independent.
        pub fn #create_fn(input: &#create<'_>) -> Result<surrealdb::types::Object, crate::canonical_codec::CodecError> {
            let mut object = surrealdb::types::Object::new(); #(#create_values)* Ok(object)
        }
        /// Convert partial mutation with explicit native removal.
        pub fn #merge_fn(input: &#merge) -> Result<surrealdb::types::Object, crate::canonical_codec::CodecError> {
            #mutable_object #(#merge_values)* Ok(object)
        }
        /// Convert a complete mutable replacement payload.
        pub fn #replace_fn(input: &#replace) -> Result<surrealdb::types::Object, crate::canonical_codec::CodecError> {
            #mutable_object #(#replace_values)* Ok(object)
        }
    })
}
