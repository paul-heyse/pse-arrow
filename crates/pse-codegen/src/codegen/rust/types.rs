// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Registry types rendered into explicit Rust types and native Arrow codecs.

use arrow_schema::DataType;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use pse_contract::boundary_projection::{TypeRef, Scalar, LocalRule};

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
    let docs = fields
        .iter()
        .map(|(name, _, doc)| if doc.trim().is_empty() { name } else { doc })
        .collect::<Vec<_>>();
    let count = names.len();
    let positions = (0..count).collect::<Vec<_>>();
    let equality = if count == 0 {
        quote!(true)
    } else {
        quote!(#(crate::SemanticEq::semantic_eq(&self.#names, &other.#names))&&*)
    };
    quote! {
            /// A row or nested value projected from the registry declaration.
            #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
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
        &mut RustPolicy {
            declarations,
            local: false,
            emit: true,
        },
        ty,
        stem,
        super::super::native::Mode::Domain,
    ).map(|(rust, _, _)| rust)
}
/// A named structure's body, rendered inside the `structures` module itself: the named
/// structures it nests are its siblings and are referenced unqualified.
pub(super) fn named_structure(
    ty: &FieldContract,
    stem: &str,
    declarations: &mut Vec<TokenStream>,
) -> Result<TokenStream, SchemaError> {
    super::super::native::render(
        &mut RustPolicy {
            declarations,
            local: true,
            emit: true,
        },
        ty,
        stem,
        super::super::native::Mode::Domain,
    ).map(|(rust, _, _)| rust)
}
pub(super) fn boundary_storage(
    ty: &FieldContract,
    stem: &str,
    declarations: &mut Vec<TokenStream>,
) -> Result<TokenStream, SchemaError> {
    super::super::native::render(
        &mut RustPolicy {
            declarations,
            local: false,
            emit: true,
        },
        ty,
        stem,
        super::super::native::Mode::Storage,
    ).map(|(rust, _, _)| rust)
}
/// The very same renderer supplies the native projection meaning; no parallel
/// Arrow-to-contract table exists in the descriptor interpreter.
pub(crate) fn projection_type(ty: &FieldContract, stem: &str) -> Result<TypeRef, SchemaError> {
    projection_type_in(ty, stem, super::super::native::Mode::Domain)
}
pub(crate) fn projection_type_in(ty: &FieldContract, stem: &str, mode: super::super::native::Mode) -> Result<TypeRef, SchemaError> {
    if crate::model::CollectionContract::from_field(ty.field())?.is_some_and(|contract| contract.unique) {
        return Err(super::error("unique native collections need checked local and wire lowering".into()));
    }
    let mut declarations = Vec::new();
    let mut policy = RustPolicy { declarations: &mut declarations, local: false, emit: false };
    let value = super::super::native::render(&mut policy, ty, stem, mode)?;
    Ok(if ty.nullable() { TypeRef::Optional(Box::new(value.1)) } else { value.1 })
}
/// Actual composite storage alias selection owned by this same renderer.
pub(crate) fn projection_storage(ty: &FieldContract) -> Result<Option<(String, FieldContract)>, SchemaError> {
    match ty.extension() {
        Some(use_ @ (ExtensionUse::DimensionVector | ExtensionUse::QuantityValue | ExtensionUse::Bound | ExtensionUse::SourceSpan)) => Ok(Some((pascal(&use_.name()), use_.boundary_storage()?))),
        _ => Ok(None),
    }
}

pub(super) fn relation_field_stem(stem: &str, name: &str) -> String { format!("{stem}Field{}", pascal(name)) }

pub(crate) fn field_rules(field: &FieldContract) -> Result<Vec<(LocalRule, TokenStream)>, SchemaError> {
    let mut rules = Vec::new();
    if let Some(range) = crate::model::IntegerRange::from_field(field.field())? {
        let (min, max) = (range.minimum, range.maximum);
        rules.push((LocalRule::SignedRange { min, max }, quote!(pse_contract::boundary_projection::LocalRule::SignedRange { min: #min, max: #max })));
    }
    if let Some(collection) = crate::model::CollectionContract::from_field(field.field())?
        && (collection.minimum != 0 || collection.maximum.is_some()) {
        let min = u64::try_from(collection.minimum).map_err(|error| super::error(error.to_string()))?;
        let max = collection.maximum.map(u64::try_from).transpose().map_err(|error| super::error(error.to_string()))?.unwrap_or(u64::MAX);
        rules.push((LocalRule::Length { min, max }, quote!(pse_contract::boundary_projection::LocalRule::Length { min: #min, max: #max })));
    }
    Ok(rules)
}
pub(crate) fn alternative_field_rules(field: &FieldContract) -> Result<Vec<(LocalRule, TokenStream)>, SchemaError> {
    let mut rules = vec![(LocalRule::Required, quote!(pse_contract::boundary_projection::LocalRule::Required))];
    rules.extend(field_rules(field)?);
    Ok(rules)
}

pub(super) fn projection_tokens(ty: &TypeRef) -> TokenStream {
    let prefix = quote!(pse_contract::boundary_projection::TypeRef);
    match ty {
        TypeRef::Scalar(scalar) => { let scalar = format_ident!("{scalar:?}"); quote!(#prefix::Scalar(pse_contract::boundary_projection::Scalar::#scalar)) },
        TypeRef::Named(name) => quote!(#prefix::Named(#name.into())),
        TypeRef::Optional(inner) => { let inner = projection_tokens(inner); quote!(#prefix::Optional(Box::new(#inner))) },
        TypeRef::Repeated(inner) => { let inner = projection_tokens(inner); quote!(#prefix::Repeated(Box::new(#inner))) },
        TypeRef::Set(inner) => { let inner = projection_tokens(inner); quote!(#prefix::Set(Box::new(#inner))) },
        TypeRef::Map(key, value) => { let key = projection_tokens(key); let value = projection_tokens(value); quote!(#prefix::Map(Box::new(#key),Box::new(#value))) },
    }
}
pub(super) fn optional(ty: TokenStream, nullable: bool) -> TokenStream {
    if nullable { quote!(Option<#ty>) } else { ty }
}
struct RustPolicy<'a> {
    declarations: &'a mut Vec<TokenStream>,
    /// Whether the declarations are emitted into the `structures` module.
    local: bool,
    emit: bool,
}
impl super::super::native::Policy for RustPolicy<'_> {
    type Value = (TokenStream, TypeRef, String);
    fn nullable(&mut self, (rust, ty, argument): Self::Value, nullable: bool) -> Self::Value {
        if nullable { (optional(rust, true), TypeRef::Optional(Box::new(ty)), pse_contract::boundary_projection::optional_argument_name(&argument)) }
        else { (rust, ty, argument) }
    }
    fn leaf(
        &mut self,
        ty: &FieldContract,
        _stem: &str,
        mode: super::super::native::Mode,
    ) -> Result<Option<Self::Value>, SchemaError> {
        if let Some(name) = super::super::enum_name(ty.field()) {
            let rust_name = format_ident!("{}", pascal(name));
            return Ok(Some((quote!(crate::generated::enums::#rust_name), TypeRef::Named(name.into()), name.into())));
        }
        if matches!(mode, super::super::native::Mode::Domain) {
            // A named structure is emitted once and referenced by name (Plan 22 X11).
            if let Some(name) = ty.structure_name() {
                let projection = crate::model::BoundaryShape::from_field(ty.field())?.and_then(|shape| shape.name).unwrap_or_else(|| name.into());
                let name = format_ident!("{}", name);
                return Ok(Some((if self.local {
                    quote!(#name)
                } else {
                    quote!(crate::generated::structures::#name)
                }, TypeRef::Named(projection.clone()), projection)));
            }
            // A nested identity value is typed by the entity it names, as a column is
            // (ADR-0115); its value and codec are the base identity's.
            if let Some(identity) = ty.identity() {
                return Ok(Some((super::identities::path(identity), super::identities::projection_type(identity), super::identities::type_name(identity))));
            }
            if let Some((name, storage)) = projection_storage(ty)? {
                let emit = self.emit;
                self.emit = false;
                let projected = super::super::native::render(self, &storage, &format!("Extension{name}"), super::super::native::Mode::Storage);
                self.emit = emit;
                let (_, projection, argument) = projected?;
                let name = format_ident!("{name}");
                return Ok(Some((quote!(crate::generated::extension_values::#name), projection, argument)));
            }
            if ty.extension().is_none() && matches!(ty.data_type(), DataType::FixedSizeBinary(_) | DataType::Dictionary(..)) {
                return Err(super::error(format!("no native language codec for {}; domain meaning requires an explicit declaration", ty.data_type())));
            }
        }
        primitive(ty)
    }
    fn container(
        &mut self,
        ty: &FieldContract,
        stem: &str,
        children: Vec<Self::Value>,
        mode: super::super::native::Mode,
    ) -> Result<Self::Value, SchemaError> {
        Ok(match ty.data_type() {
            DataType::List(_) => {
                let (child, projection, argument) = &children[0];
                let projection = pse_contract::boundary_projection::sequence_type(projection.clone());
                (quote!(Vec<#child>), projection, pse_contract::boundary_projection::sequence_argument_name(argument))
            }
            DataType::FixedSizeList(_, width) => {
                let (child, projection, argument) = &children[0];
                let width =
                    usize::try_from(width).map_err(|error| super::error(error.to_string()))?;
                (quote!([#child; #width]), TypeRef::Named(pse_contract::boundary_projection::fixed_array_name(projection, argument, width)), pse_contract::boundary_projection::array_argument_name(argument, width))
            }
            DataType::Struct(fields) => {
                let fields = fields
                    .iter()
                    .zip(children)
                    .map(|(field, (rust, _, _))| (field.name().to_owned(), rust, field.name().to_owned()))
                    .collect::<Vec<_>>();
                if self.emit {
                    self.declarations.push(structure(stem, &fields));
                    self.declarations.push(super::alternative::accessors(ty, stem, self.local)?);
                    self.declarations.push(super::boundary_nested::render(ty, stem, &fields, mode)?);
                }
                let name = format_ident!("{stem}");
                let projection = crate::model::BoundaryShape::from_field(ty.field())?.and_then(|shape| shape.name).unwrap_or_else(|| stem.into());
                (quote!(#name), TypeRef::Named(projection.clone()), projection)
            }
            other => return Err(super::error(format!("no typed container for {other}"))),
        })
    }
}

fn primitive(ty: &FieldContract) -> Result<Option<(TokenStream, TypeRef, String)>, SchemaError> {
    let (rust, scalar) = match ty.data_type() {
        DataType::Boolean => (quote!(bool), Scalar::Bool), DataType::Int16 => (quote!(i16), Scalar::I16),
        DataType::Int32 => (quote!(i32), Scalar::I32), DataType::Int64 | DataType::Timestamp(..) => (quote!(i64), Scalar::I64),
        DataType::UInt8 => (quote!(u8), Scalar::U8), DataType::UInt16 => (quote!(u16), Scalar::U16),
        DataType::UInt32 => (quote!(u32), Scalar::U32), DataType::UInt64 => (quote!(u64), Scalar::U64),
        DataType::Float64 => (quote!(f64), Scalar::F64Bits), DataType::Utf8 => (quote!(String), Scalar::String),
        DataType::Binary => (quote!(pse_model::Bytes), Scalar::Bytes),
        DataType::FixedSizeBinary(16) => (quote!(pse_ids::SemanticId), Scalar::SemanticId),
        DataType::FixedSizeBinary(32) => (quote!(pse_ids::ContentHash), Scalar::ContentHash),
        DataType::List(_) | DataType::FixedSizeList(..) | DataType::Struct(_) => return Ok(None),
        other => return Err(super::error(format!("no typed codec for {other}"))),
    };
    let projection = TypeRef::Scalar(scalar);
    let argument = pse_contract::boundary_projection::projection_argument_name(&projection);
    Ok(Some((rust, projection, argument)))
}

#[cfg(test)]
mod paired_tests {
    use super::*;
    use pse_contract::boundary_projection::{ProjectionType, ProjectionGraph};

    fn agrees<T: ProjectionType>(field: FieldContract) {
        let actual = T::projection(&mut ProjectionGraph::default()).unwrap();
        assert_eq!(projection_type(&field, "Example").unwrap(), actual);
    }
    #[test]
    fn actual_renderer_pairs_scalar_container_and_nominal_projection_meaning() {
        agrees::<bool>(FieldContract::native(DataType::Boolean));
        agrees::<i16>(FieldContract::native(DataType::Int16));
        agrees::<i32>(FieldContract::native(DataType::Int32));
        agrees::<i64>(FieldContract::native(DataType::Int64));
        agrees::<i64>(FieldContract::native(DataType::Timestamp(arrow_schema::TimeUnit::Microsecond, None)));
        agrees::<u8>(FieldContract::native(DataType::UInt8));
        agrees::<u16>(FieldContract::native(DataType::UInt16));
        agrees::<u32>(FieldContract::native(DataType::UInt32));
        agrees::<u64>(FieldContract::native(DataType::UInt64));
        agrees::<f64>(FieldContract::native(DataType::Float64));
        agrees::<String>(FieldContract::native(DataType::Utf8));
        agrees::<pse_model::Bytes>(FieldContract::native(DataType::Binary));
        agrees::<pse_ids::SemanticId>(FieldContract::id());
        agrees::<pse_ids::ContentHash>(FieldContract::hash());
        agrees::<pse_quantity::UnitId>(FieldContract::id().with_identity("unit"));
        agrees::<pse_model::generated::identities::PackageId>(FieldContract::id().with_identity("package"));
        agrees::<pse_model::generated::enums::PresolvePass>(FieldContract::enumeration("PresolvePass"));
        agrees::<pse_model::generated::extension_values::QuantityValue>(FieldContract::extended(ExtensionUse::QuantityValue));
        agrees::<Vec<u8>>(FieldContract::list(FieldContract::native(DataType::UInt8).with_name("item")));
        agrees::<[Option<Vec<u8>>; 2]>(FieldContract::fixed_list(FieldContract::list(FieldContract::native(DataType::UInt8).with_name("item")).with_name("item").optional(), 2));
        agrees::<[Option<pse_model::Bytes>; 2]>(FieldContract::fixed_list(FieldContract::native(DataType::Binary).with_name("item").optional(), 2));
        agrees::<Option<Vec<Option<i64>>>>(FieldContract::list(FieldContract::native(DataType::Int64).with_name("item").optional()).optional());
        assert_ne!(projection_type(&FieldContract::id(), "Example").unwrap(), projection_type(&FieldContract::native(DataType::Binary), "Example").unwrap());
    }
}
