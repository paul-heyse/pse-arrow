// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Registry types rendered into explicit Rust types and native Arrow codecs.

use arrow_schema::DataType;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use crate::SchemaError;
use crate::model::{ExtensionUse, FieldContract};

pub(super) fn ident(name: &str) -> Ident {
    // Registry declaration admission restricts names to identifiers. Raw identifiers
    // preserve column spellings such as `type` in generated code and serde.
    format_ident!("r#{}", name)
}

pub(crate) fn pascal(name: &str) -> String {
    let normalized = name
        .split('_')
        .map(|part| {
            if part
                .chars()
                .filter(char::is_ascii_alphabetic)
                .all(|ch| ch.is_ascii_uppercase())
            {
                part.to_ascii_lowercase()
            } else {
                part.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("_");
    let mut result = String::new();
    let mut capital = true;
    for character in normalized.chars() {
        if character.is_ascii_alphanumeric() {
            result.push(if capital {
                character.to_ascii_uppercase()
            } else {
                character
            });
            capital = false;
        } else {
            capital = true;
        }
    }
    if result.starts_with(|character: char| character.is_ascii_digit()) || result == "Self" {
        result.insert_str(0, "Member");
    }
    result
}

pub(super) fn structure(name: &str, fields: &[(String, TokenStream, String)]) -> TokenStream {
    let name = format_ident!("{}", name);
    let names = fields
        .iter()
        .map(|(name, _, _)| ident(name))
        .collect::<Vec<_>>();
    let types = fields.iter().map(|(_, ty, _)| ty).collect::<Vec<_>>();
    let docs = fields.iter().map(|(_, _, doc)| doc).collect::<Vec<_>>();
    let count = names.len();
    let positions = (0..count).collect::<Vec<_>>();
    let equality = if count == 0 {
        quote!(true)
    } else {
        quote!(#(crate::SemanticEq::semantic_eq(&self.#names, &other.#names))&&*)
    };
    quote! {
            /// A row or nested value projected from the registry declaration.
            #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
            #[serde(deny_unknown_fields)]
            #[allow(clippy::struct_field_names, reason = "field names are the authoritative relation contract")]
            pub struct #name { #(#[doc = #docs] pub #names: #types,)* }
            impl crate::SemanticEq for #name {
                fn semantic_eq(&self, other: &Self) -> bool { #equality }
            }
            impl PartialEq for #name {
                fn eq(&self, other: &Self) -> bool { crate::SemanticEq::semantic_eq(self, other) }
            }

    impl crate::columnar::ArrowValue for #name {
                fn append(&self, output: &mut dyn arrow_array::builder::ArrayBuilder) -> Result<(), crate::RelationError> {
                    let output = crate::columnar::builder::<arrow_array::builder::StructBuilder>(output)?;
                    let children = output.field_builders_mut();
                    #(crate::columnar::ArrowValue::append(&self.#names, children[#positions].as_mut())?;)*
                    output.append(true);
                    Ok(())
                }
                fn append_null(output: &mut dyn arrow_array::builder::ArrayBuilder) -> Result<(), crate::RelationError> {
                    let output = crate::columnar::builder::<arrow_array::builder::StructBuilder>(output)?;
                    let children = output.field_builders_mut();
                    #(<#types as crate::columnar::ArrowValue>::append_null(children[#positions].as_mut())?;)*
                    output.append(false);
                    Ok(())
                }
                fn read(input: &dyn arrow_array::Array, index: usize) -> Result<Self, crate::RelationError> {
                    crate::columnar::visible(input, index)?;
                    let input = crate::columnar::array::<arrow_array::StructArray>(input)?;
                    Ok(Self { #(#names: <#types as crate::columnar::ArrowValue>::read(input.column(#positions).as_ref(), index)?,)* })
                }
            }
        }
}

pub(super) fn logical(
    ty: &FieldContract,
    stem: &str,
    declarations: &mut Vec<TokenStream>,
) -> Result<TokenStream, SchemaError> {
    super::super::native::render(
        &mut RustPolicy { declarations },
        ty,
        stem,
        super::super::native::Mode::Domain,
    )
}
pub(super) fn storage(
    ty: &DataType,
    stem: &str,
    declarations: &mut Vec<TokenStream>,
) -> Result<TokenStream, SchemaError> {
    super::super::native::storage(&mut RustPolicy { declarations }, ty, stem)
}
pub(super) fn optional(ty: TokenStream, nullable: bool) -> TokenStream {
    if nullable { quote!(Option<#ty>) } else { ty }
}
struct RustPolicy<'a> {
    declarations: &'a mut Vec<TokenStream>,
}
impl super::super::native::Policy for RustPolicy<'_> {
    type Value = TokenStream;
    fn nullable(&mut self, value: TokenStream, nullable: bool) -> TokenStream {
        optional(value, nullable)
    }
    fn leaf(
        &mut self,
        ty: &FieldContract,
        _stem: &str,
        mode: super::super::native::Mode,
    ) -> Result<Option<TokenStream>, SchemaError> {
        if let Some(name) = super::super::enum_name(ty.field()) {
            let name = format_ident!("{}", pascal(name));
            return Ok(Some(quote!(crate::generated::enums::#name)));
        }
        if matches!(mode, super::super::native::Mode::Domain) {
            if let Some(use_) = ty.extension() {
                if matches!(
                    use_,
                    ExtensionUse::DimensionVector
                        | ExtensionUse::QuantityValue
                        | ExtensionUse::Bound
                        | ExtensionUse::SourceSpan
                ) {
                    let name = format_ident!("{}", pascal(&use_.name()));
                    return Ok(Some(quote!(crate::generated::extension_values::#name)));
                }
            } else if matches!(
                ty.data_type(),
                DataType::FixedSizeBinary(_) | DataType::Dictionary(..)
            ) {
                return Err(super::error(format!(
                    "no native language codec for {}; domain meaning requires an explicit declaration",
                    ty.data_type()
                )));
            }
        }
        Ok(Some(match ty.data_type() {
            DataType::Boolean => quote!(bool),
            DataType::Int16 => quote!(i16),
            DataType::Int32 => quote!(i32),
            DataType::Int64 | DataType::Timestamp(..) => quote!(i64),
            DataType::UInt8 => quote!(u8),
            DataType::UInt16 => quote!(u16),
            DataType::UInt32 => quote!(u32),
            DataType::UInt64 => quote!(u64),
            DataType::Float64 => quote!(f64),
            DataType::Utf8 => quote!(String),
            DataType::FixedSizeBinary(16) => quote!(pse_ids::SemanticId),
            DataType::FixedSizeBinary(32) => quote!(pse_ids::ContentHash),
            DataType::List(_) | DataType::FixedSizeList(..) | DataType::Struct(_) => {
                return Ok(None);
            }
            other => return Err(super::error(format!("no typed codec for {other}"))),
        }))
    }
    fn container(
        &mut self,
        ty: &FieldContract,
        stem: &str,
        children: Vec<TokenStream>,
    ) -> Result<TokenStream, SchemaError> {
        Ok(match ty.data_type() {
            DataType::List(_) => {
                let child = &children[0];
                quote!(Vec<#child>)
            }
            DataType::FixedSizeList(_, width) => {
                let child = &children[0];
                let width =
                    usize::try_from(width).map_err(|error| super::error(error.to_string()))?;
                quote!([#child; #width])
            }
            DataType::Struct(fields) => {
                let fields = fields
                    .iter()
                    .zip(children)
                    .map(|(field, ty)| (field.name().to_owned(), ty, field.name().to_owned()))
                    .collect::<Vec<_>>();
                self.declarations.push(structure(stem, &fields));
                self.declarations
                    .push(super::alternative::accessors(ty, stem)?);
                let name = format_ident!("{stem}");
                quote!(#name)
            }
            other => return Err(super::error(format!("no typed container for {other}"))),
        })
    }
}
