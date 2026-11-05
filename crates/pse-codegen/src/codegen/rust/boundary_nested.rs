// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Typed projections of actual nested records and selected native tagged payloads.
use crate::{
    SchemaError,
    model::{BoundaryShape, FieldContract, TaggedAlternative},
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub(super) fn render(
    field: &FieldContract,
    stem: &str,
    fields: &[(String, TokenStream, String)],
    mode: super::super::native::Mode,
) -> Result<TokenStream, SchemaError> {
    let Some(shape) = BoundaryShape::from_field(field.field())? else {
        return Ok(TokenStream::new());
    };
    let row = format_ident!("{stem}");
    if let Some(alternative) = TaggedAlternative::from_field(field.field())? {
        alternatives(field, &shape, &alternative, &row, fields)
    } else {
        let children = field.children();
        let stems = children
            .iter()
            .map(|child| super::super::native::child_stem(field, stem, child.field()))
            .collect::<Vec<_>>();
        super::boundary_record::render(
            &shape,
            &children,
            &row,
            fields,
            &stems,
            super::super::native::child_mode(field, mode),
        )
    }
}

fn payload_type(ty: &TokenStream) -> Result<TokenStream, SchemaError> {
    let ty: syn::Type = syn::parse2(ty.clone()).map_err(|error| super::error(error.to_string()))?;
    let syn::Type::Path(path) = ty else {
        return Err(super::error(
            "native tagged payload must be optional".into(),
        ));
    };
    let Some(segment) = path.path.segments.last() else {
        return Err(super::error("tagged payload type absent".into()));
    };
    let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return Err(super::error(
            "native tagged payload must be optional".into(),
        ));
    };
    if segment.ident != "Option" || arguments.args.len() != 1 {
        return Err(super::error(
            "native tagged payload requires exactly one optional type".into(),
        ));
    }
    let Some(syn::GenericArgument::Type(ty)) = arguments.args.first() else {
        return Err(super::error("tagged payload type absent".into()));
    };
    super::semantic::model_type(&quote!(#ty))
}

fn alternatives(
    contract: &FieldContract,
    shape: &BoundaryShape,
    alternative: &TaggedAlternative,
    row: &syn::Ident,
    fields: &[(String, TokenStream, String)],
) -> Result<TokenStream, SchemaError> {
    let name = shape.name.clone().unwrap_or_else(|| row.to_string());
    let source = &shape.source;
    let version = shape.version;
    let direction = format_ident!("{}", format!("{:?}", shape.direction));
    let reserved_tags = &shape.reserved_tags;
    let reserved_names = &shape.reserved_names;
    let discriminator = super::types::ident(&alternative.discriminator);
    let selected = format_ident!("{row}Selected");
    let mut definitions = Vec::new();
    let mut accessors = Vec::new();
    let mut validation = Vec::new();
    for (tag, arm) in &alternative.arms {
        let number = shape
            .alternatives
            .iter()
            .find_map(|(name, number)| (name == tag).then_some(number))
            .ok_or_else(|| {
                super::error(format!(
                    "{name}.{tag}: explicit wire alternative number absent"
                ))
            })?;
        let variant = format_ident!("{}", super::types::pascal(tag));
        let constructor = format_ident!("from_{tag}");
        let receiver_mut = arm.as_ref().map(|_| quote!(mut));
        let (definition, field_layout, types, extract, construct, check) = if let Some(arm) = arm {
            let payload = super::types::ident(arm);
            let (_, ty, _) = fields
                .iter()
                .find(|(name, _, _)| name == arm)
                .ok_or_else(|| super::error(format!("{name}.{tag}: native payload absent")))?;
            let ty = payload_type(ty)?;
            let column = contract
                .children()
                .into_iter()
                .find(|field| field.name() == arm)
                .ok_or_else(|| {
                    super::error("native alternative source payload is absent".into())
                })?;
            let rules = super::types::alternative_field_rules(&column)?
                .into_iter()
                .map(|(_, tokens)| tokens)
                .collect::<Vec<_>>();
            (
                quote!(
                    vec![pse_contract::boundary_projection::Field::of::<#ty>(graph,"value",1,
                    pse_contract::boundary_projection::Direction::#direction,
                    vec![#(#rules),*])?]
                ),
                quote!(&[(1, "value")]),
                quote!((#ty,)),
                quote!(match self.#payload.take() {Some(value)=>Ok((value,)),None=>Err(self)}),
                quote! { let (value,)=fields; Self::#constructor(value) },
                quote!(#selected::#variant(value)=>pse_contract::boundary_projection::validate_field(value,
                    &pse_contract::boundary_projection::field_path(path,#tag), &[#(#rules),*])),
            )
        } else {
            (
                quote!(vec![]),
                quote!(&[]),
                quote!(()),
                quote!(Ok(())),
                quote! { let ()=fields; Self::#constructor() },
                quote!(#selected::#variant=>Ok(())),
            )
        };
        definitions.push(quote!(pse_contract::boundary_projection::Alternative {name:#tag.into(),tag:#number,fields:#definition}));
        validation.push(check);
        accessors.push(quote! {
            impl pse_contract::boundary_projection::ProjectionAlternative<#number> for #row {
                const VERSION:u32=#version;
                const FIELD_LAYOUT:&'static[(u32,&'static str)]=#field_layout;
                type Fields=#types;
                fn into_projection_alternative(#receiver_mut self)->Result<Self::Fields,Self> {
                    if self.#discriminator.as_str()!=#tag||self.selected().is_err() {return Err(self);}
                    #extract
                }
                fn from_projection_alternative(fields:Self::Fields)->Self {#construct}
            }
        });
    }
    Ok(quote! {
        impl pse_contract::boundary_projection::ProjectionType for #row {
            fn projection(graph:&mut pse_contract::boundary_projection::ProjectionGraph)->Result<pse_contract::boundary_projection::TypeRef,pse_contract::boundary_projection::ProjectionError> {
                graph.declare::<Self>(#name,#source,#version,pse_contract::boundary_projection::Direction::#direction,
                    &[#(#reserved_tags),*],&[#(#reserved_names),*],|graph|Ok(pse_contract::boundary_projection::Shape::Alternatives(vec![#(#definitions),*])))
            }
        }
        impl pse_contract::boundary_projection::ValidateProjection for #row {
            fn validate_at(&self,path:&str)->Result<(),pse_contract::boundary_projection::ValidationError> {
                match self.selected().map_err(|_|pse_contract::boundary_projection::ValidationError::Rule {
                    path:path.into(),rule:"tagged_alternative",reason:"native tag must select exactly its declared payload".into(),
                })? {#(#validation),*}
            }
        }
        #(#accessors)*
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow_schema::DataType;
    use pse_contract::boundary_projection::Direction as BoundaryDirection;

    #[test]
    fn native_selected_payloads_supply_checked_oneof_access_without_another_native_type() {
        let payload = FieldContract::structure(vec![
            FieldContract::nonnegative(7)
                .with_name("count")
                .with_boundary_tag(9),
        ])
        .with_name("payload")
        .optional()
        .with_boundary_shape(BoundaryShape::native(file!(), 1, BoundaryDirection::Both));
        let value = FieldContract::structure(vec![
            FieldContract::native(DataType::Utf8).with_name("kind"),
            payload,
        ])
        .with_alternative(
            &TaggedAlternative::new("kind", [("present".into(), "payload".into())])
                .with_unit("absent"),
        )
        .with_boundary_shape(
            BoundaryShape::native(file!(), 1, BoundaryDirection::Both)
                .alternatives([("present".into(), 4), ("absent".into(), 11)]),
        );
        let mut declarations = Vec::new();
        let actual =
            super::super::types::logical(&value, "OriginalValue", &mut declarations).unwrap();
        assert_eq!(actual.to_string(), "OriginalValue");
        let file: syn::File = syn::parse2(quote!(#(#declarations)*)).unwrap();
        let mut alternatives = Vec::new();
        for item in &file.items {
            if let syn::Item::Impl(item) = item
                && let Some((path, _)) = &item.trait_
                && path
                    .segments
                    .last()
                    .is_some_and(|part| part.ident == "ProjectionAlternative")
            {
                alternatives.push(item);
            }
        }
        assert_eq!(alternatives.len(), 2);
        let emitted = quote!(#(#alternatives)*).to_string();
        assert!(emitted.contains("self . selected () . is_err ()"));
        assert!(emitted.contains("from_present") && emitted.contains("from_absent"));
        let records = quote!(#(#declarations)*).to_string();
        assert!(records.contains("SignedRange") && records.contains("9u32"));
        assert!(records.contains("Shape :: Alternatives"));
    }
}
