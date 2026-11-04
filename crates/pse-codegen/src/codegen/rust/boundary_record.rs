// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared projection of top-level and nested actual registry records.

use crate::{
    SchemaError,
    model::{BoundaryShape, CollectionContract, FieldContract, IntegerRange, TaggedAlternative},
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub(super) fn render(
    shape: &BoundaryShape,
    columns: &[FieldContract],
    row: &syn::Ident,
    fields: &[(String, TokenStream, String)],
) -> Result<TokenStream, SchemaError> {
    if fields.len() != columns.len()
        || fields
            .iter()
            .zip(columns)
            .any(|((name, _, _), column)| name != column.name())
    {
        return Err(super::error(
            "registry boundary must project exactly its actual native fields".into(),
        ));
    }
    let name = shape.name.clone().unwrap_or_else(|| row.to_string());
    let source = &shape.source;
    let version = shape.version;
    let direction = format_ident!("{}", format!("{:?}", shape.direction));
    let reserved_tags = &shape.reserved_tags;
    let reserved_names = &shape.reserved_names;
    let mut definitions = Vec::new();
    let mut layout = Vec::new();
    let mut names = Vec::new();
    let mut types = Vec::new();
    let mut recursive = Vec::new();
    let mut field_rules = Vec::new();
    for ((field_name, ty, _), column) in fields.iter().zip(columns) {
        let ty = super::semantic::model_type(ty)?;
        if TaggedAlternative::from_field(column.field())?.is_some()
            && BoundaryShape::from_field(column.field())?.is_none()
        {
            return Err(super::error(format!(
                "{name}.{field_name}: native alternatives need explicit wire presentation"
            )));
        }
        if CollectionContract::from_field(column.field())?.is_some_and(|contract| contract.unique) {
            return Err(super::error(format!(
                "{name}.{field_name}: unique native collections need checked local and wire lowering"
            )));
        }
        let number = column.boundary_tag().ok_or_else(|| {
            super::error(format!(
                "{name}.{field_name} has no source-owned boundary number"
            ))
        })?;
        let field = super::types::ident(field_name);
        let mut rules = Vec::new();
        if let Some(range) = IntegerRange::from_field(column.field())? {
            let (min, max) = (range.minimum, range.maximum);
            rules.push(
                quote!(pse_contract::boundary_projection::LocalRule::SignedRange { min: #min, max: #max }),
            );
        }
        if let Some(collection) = CollectionContract::from_field(column.field())?
            && (collection.minimum != 0 || collection.maximum.is_some())
        {
            let min = u64::try_from(collection.minimum)
                .map_err(|error| super::error(error.to_string()))?;
            let max = collection
                .maximum
                .map(u64::try_from)
                .transpose()
                .map_err(|error| super::error(error.to_string()))?
                .unwrap_or(u64::MAX);
            rules.push(
                quote!(pse_contract::boundary_projection::LocalRule::Length { min: #min, max: #max }),
            );
        }
        field_rules.push(!rules.is_empty());
        let facets = column
            .field()
            .metadata()
            .iter()
            .collect::<std::collections::BTreeMap<_, _>>()
            .into_iter()
            .map(|(key, value)| quote!((#key.into(), #value.into())))
            .collect::<Vec<_>>();
        definitions.push(
            quote!(pse_contract::boundary_projection::Field::of::<#ty>(graph, #field_name, #number,
            pse_contract::boundary_projection::Direction::#direction, vec![#(#rules),*])?
            .with_named_facets(std::collections::BTreeMap::from([#(#facets),*]))),
        );
        layout.push(quote!((#number, #field_name)));
        names.push(field);
        types.push(ty);
        let field = super::types::ident(field_name);
        recursive.push(
            quote!(pse_contract::boundary_projection::validate_field(&self.#field,
            &pse_contract::boundary_projection::field_path(path, #field_name), &[#(#rules),*])?;),
        );
    }
    Ok(quote! {
        impl pse_contract::boundary_projection::ProjectionType for #row {
            fn projection(graph: &mut pse_contract::boundary_projection::ProjectionGraph)
                -> Result<pse_contract::boundary_projection::TypeRef, pse_contract::boundary_projection::ProjectionError> {
                graph.declare::<Self>(#name, #source, #version,
                    pse_contract::boundary_projection::Direction::#direction, &[#(#reserved_tags),*], &[#(#reserved_names),*],
                    |graph| Ok(pse_contract::boundary_projection::Shape::Record(vec![#(#definitions),*])))
            }
        }
        impl pse_contract::boundary_projection::ProjectionRecord for #row {
            const VERSION: u32 = #version;
            const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[#(#layout),*];
            type Fields = (#(#types,)*);
            fn into_projection_fields(self) -> Self::Fields { (#(self.#names,)*) }
            fn from_projection_fields((#(#names,)*): Self::Fields)
                -> Result<Self, pse_contract::boundary_projection::AdapterError> { Ok(Self { #(#names,)* }) }
        }
        impl pse_contract::boundary_projection::ValidateProjection for #row {
            const NEEDS_RECURSIVE_VALIDATION: bool = false #(|| #field_rules || <#types as pse_contract::boundary_projection::ValidateProjection>::NEEDS_RECURSIVE_VALIDATION)*;
            fn validate_at(&self, path: &str) -> Result<(), pse_contract::boundary_projection::ValidationError> {
                if Self::NEEDS_RECURSIVE_VALIDATION { #(#recursive)* }
                Ok(())
            }
        }
    })
}
