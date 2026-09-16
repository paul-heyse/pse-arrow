// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Registry types rendered into explicit Rust types and checked cell codecs.

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
    quote! {
        /// A row or nested value projected from the registry declaration.
        #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        #[allow(clippy::struct_field_names, reason = "field names are the authoritative relation contract")]
        pub struct #name { #(#[doc = #docs] pub #names: #types,)* }
        impl crate::typed::CellCodec for #name {
            fn into_cell(self) -> pse_schema::model::Cell {
                pse_schema::model::Cell::Struct(vec![
                    #(crate::typed::CellCodec::into_cell(self.#names)),*
                ])
            }
            fn from_cell(cell: pse_schema::model::Cell) -> Result<Self, crate::RelationError> {
                let pse_schema::model::Cell::Struct(values) = cell else {
                    return Err(crate::typed::mismatch(stringify!(#name)));
                };
                if values.len() != #count {
                    return Err(crate::typed::mismatch(stringify!(#name)));
                }
                let mut values = values.into_iter();
                Ok(Self { #(#names: <#types as crate::typed::CellCodec>::from_cell(
                    values.next().ok_or_else(|| crate::typed::mismatch(stringify!(#name)))?
                )?,)* })
            }
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
    Ok(match (ty.extension(), ty.data_type()) {
        (None, DataType::List(child)) => {
            let child = optional(
                logical(
                    &FieldContract::from_field((*child).clone()),
                    &format!("{stem}Item"),
                    declarations,
                )?,
                child.is_nullable(),
            );
            quote!(Vec<#child>)
        }
        (None, DataType::FixedSizeList(child, width)) => {
            let child = optional(
                logical(
                    &FieldContract::from_field((*child).clone()),
                    &format!("{stem}Item"),
                    declarations,
                )?,
                child.is_nullable(),
            );
            let width = usize::try_from(width).map_err(|error| super::error(error.to_string()))?;
            quote!([#child; #width])
        }
        (None, DataType::Struct(children)) => {
            let fields = children
                .iter()
                .map(|field| {
                    let name = field.name();
                    let ty = FieldContract::from_field((**field).clone());
                    let nullable = field.is_nullable();
                    let ty = logical(&ty, &format!("{stem}{}", pascal(name)), declarations)?;
                    Ok((name.to_owned(), optional(ty, nullable), name.to_owned()))
                })
                .collect::<Result<Vec<_>, SchemaError>>()?;
            declarations.push(structure(stem, &fields));
            declarations.push(super::alternative::accessors(ty, stem)?);
            let name = format_ident!("{}", stem);
            quote!(#name)
        }
        (Some(ExtensionUse::Enum(name)), _) => {
            let name = format_ident!("{}", pascal(name));
            quote!(crate::generated::enums::#name)
        }
        (Some(use_), _)
            if matches!(
                use_,
                ExtensionUse::DimensionVector
                    | ExtensionUse::QuantityValue
                    | ExtensionUse::Bound
                    | ExtensionUse::SourceSpan
            ) =>
        {
            let name = format_ident!("{}", pascal(&use_.name()));
            quote!(crate::generated::extension_values::#name)
        }
        (None, DataType::FixedSizeBinary(_) | DataType::Dictionary(..)) => {
            return Err(super::error(format!(
                "no native language codec for {}; domain meaning requires an explicit declaration",
                ty.data_type()
            )));
        }
        _ => storage(&ty.data_type(), stem, declarations)?,
    })
}

pub(super) fn optional(ty: TokenStream, nullable: bool) -> TokenStream {
    if nullable { quote!(Option<#ty>) } else { ty }
}

pub(super) fn storage(
    ty: &DataType,
    stem: &str,
    declarations: &mut Vec<TokenStream>,
) -> Result<TokenStream, SchemaError> {
    Ok(match ty {
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
        DataType::List(child) => {
            let child = storage(child.data_type(), &format!("{stem}Item"), declarations)?;
            quote!(Vec<#child>)
        }
        DataType::FixedSizeList(child, width) => {
            let child = storage(child.data_type(), &format!("{stem}Item"), declarations)?;
            let width = usize::try_from(*width).map_err(|error| super::error(error.to_string()))?;
            quote!([#child; #width])
        }
        DataType::Struct(children) => {
            let fields = children
                .iter()
                .map(|field| {
                    let ty = if let Some(name) = super::super::enum_name(field) {
                        let name = format_ident!("{}", pascal(name));
                        quote!(crate::generated::enums::#name)
                    } else {
                        storage(
                            field.data_type(),
                            &format!("{stem}{}", pascal(field.name())),
                            declarations,
                        )?
                    };
                    Ok((
                        field.name().to_owned(),
                        optional(ty, field.is_nullable()),
                        field.name().to_owned(),
                    ))
                })
                .collect::<Result<Vec<_>, SchemaError>>()?;
            declarations.push(structure(stem, &fields));
            let name = format_ident!("{}", stem);
            quote!(#name)
        }
        other => return Err(super::error(format!("no typed codec for {other}"))),
    })
}
