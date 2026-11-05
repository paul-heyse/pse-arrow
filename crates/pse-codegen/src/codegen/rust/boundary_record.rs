// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared projection of top-level and nested actual registry records.

use crate::{
    SchemaError,
    model::{BoundaryShape, FieldContract, TaggedAlternative},
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub(super) fn render(
    shape: &BoundaryShape,
    columns: &[FieldContract],
    row: &syn::Ident,
    fields: &[(String, TokenStream, String)],
    field_stems: &[String],
    mode: super::super::native::Mode,
) -> Result<TokenStream, SchemaError> {
    if fields.len() != columns.len()
        || field_stems.len() != columns.len()
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
    let mut has_local_rules = false;
    for (((field_name, ty, _), column), field_stem) in fields.iter().zip(columns).zip(field_stems) {
        // Present malformed annotations must not disappear through convenience defaults.
        column.checked_role()?;
        column.checked_extension()?;
        column.identity_contract()?;
        column.canonical_policy()?;
        column.expression_source_contract()?;
        let ty = super::semantic::model_type(ty)?;
        let native_projection = match mode {
            super::super::native::Mode::Domain => {
                super::types::projection_type(column, field_stem)?
            }
            super::super::native::Mode::Storage => {
                super::types::projection_type_in(column, field_stem, mode)?
            }
        };
        let native_projection = super::types::projection_tokens(&native_projection);
        if TaggedAlternative::from_field(column.field())?.is_some()
            && BoundaryShape::from_field(column.field())?.is_none()
        {
            return Err(super::error(format!(
                "{name}.{field_name}: native alternatives need explicit wire presentation"
            )));
        }
        let number = column.boundary_tag().ok_or_else(|| {
            super::error(format!(
                "{name}.{field_name} has no source-owned boundary number"
            ))
        })?;
        let field = super::types::ident(field_name);
        let rules = super::types::field_rules(column)?
            .into_iter()
            .map(|(_, tokens)| tokens)
            .collect::<Vec<_>>();
        has_local_rules |= !rules.is_empty();
        let mut metadata = column
            .field()
            .metadata()
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect::<std::collections::BTreeMap<_, _>>();
        // Resolve scalar FK nullability at its actual native owner, not from a later wire type.
        if column.checked_fk()?.is_some()
            && let Some(reference) = crate::model::ReferenceContract::for_contract(column)?
        {
            metadata.insert(
                crate::model::reference::KEY_REFERENCE.into(),
                reference.canonical()?,
            );
        }
        let native_field = column.field().clone().with_metadata(
            metadata
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
        );
        metadata.insert(
            super::super::boundary_projection::REGISTRY_FIELD_SOURCE.into(),
            crate::model::field::render_field(&native_field)?,
        );
        let facets = metadata
            .into_iter()
            .map(|(key, value)| quote!((#key.into(), #value.into())))
            .collect::<Vec<_>>();
        definitions.push(
            quote!({
                let projected = pse_contract::boundary_projection::Field::of::<#ty>(graph, #field_name, #number,
                    pse_contract::boundary_projection::Direction::#direction, vec![#(#rules),*])?;
                if projected.ty != #native_projection {
                    return Err(pse_contract::boundary_projection::ProjectionError::invalid(#field_name,
                        "compiler-resolved native field differs from its source Rust/Arrow projection"));
                }
                projected.with_named_facets(std::collections::BTreeMap::from([#(#facets),*]))
            }),
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
    let needs_recursive_validation = recursive_validation_requirement(&types, has_local_rules);
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
            const NEEDS_RECURSIVE_VALIDATION: bool = #needs_recursive_validation;
            fn validate_at(&self, path: &str) -> Result<(), pse_contract::boundary_projection::ValidationError> {
                if Self::NEEDS_RECURSIVE_VALIDATION { #(#recursive)* }
                Ok(())
            }
        }
    })
}

fn recursive_validation_requirement(types: &[TokenStream], has_local_rules: bool) -> TokenStream {
    if has_local_rules {
        return quote!(true);
    }
    let mut seen = std::collections::BTreeSet::new();
    let mut types = types.iter().filter(|ty| seen.insert(ty.to_string()));
    let Some(first) = types.next() else {
        return quote!(false);
    };
    quote!(
        <#first as pse_contract::boundary_projection::ValidateProjection>::NEEDS_RECURSIVE_VALIDATION
        #(|| <#types as pse_contract::boundary_projection::ValidateProjection>::NEEDS_RECURSIVE_VALIDATION)*
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::ToTokens;

    fn evaluate_requirement(expression: &syn::Expr, recursive_types: &[&str]) -> bool {
        match expression {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Bool(value),
                ..
            }) => value.value,
            syn::Expr::Path(path) => {
                let ty = path
                    .qself
                    .as_ref()
                    .unwrap()
                    .ty
                    .to_token_stream()
                    .to_string();
                recursive_types.contains(&ty.as_str())
            }
            syn::Expr::Binary(binary) if matches!(binary.op, syn::BinOp::Or(_)) => {
                evaluate_requirement(&binary.left, recursive_types)
                    || evaluate_requirement(&binary.right, recursive_types)
            }
            other => panic!(
                "unexpected validation requirement: {}",
                other.to_token_stream()
            ),
        }
    }

    #[test]
    fn recursive_validation_requirement_preserves_local_and_distinct_type_requirements() {
        for types in [
            vec![],
            vec![quote!(Alpha)],
            vec![quote!(Alpha), quote!(Alpha), quote!(Beta)],
        ] {
            for has_local_rules in [false, true] {
                let emitted = recursive_validation_requirement(&types, has_local_rules);
                let expression: syn::Expr = syn::parse2(emitted.clone()).unwrap();
                for recursive_types in [vec![], vec!["Alpha"], vec!["Beta"], vec!["Alpha", "Beta"]]
                {
                    assert_eq!(
                        evaluate_requirement(&expression, &recursive_types),
                        has_local_rules
                            || types
                                .iter()
                                .any(|ty| recursive_types.contains(&ty.to_string().as_str())),
                        "types: {types:?}, local: {has_local_rules}, recursive: {recursive_types:?}"
                    );
                }
                assert!(emitted.to_string().matches("< Alpha as").count() <= 1);
            }
        }
    }

    #[test]
    fn local_rule_enables_validation_of_every_original_field() {
        let columns = vec![
            FieldContract::nonnegative(7)
                .with_name("bounded")
                .with_boundary_tag(1),
            FieldContract::native(arrow_schema::DataType::Utf8)
                .with_name("text")
                .with_boundary_tag(2),
        ];
        let emitted = render(
            &BoundaryShape::native(
                file!(),
                1,
                pse_contract::boundary_projection::Direction::Both,
            ),
            &columns,
            &format_ident!("LocalRuleRecord"),
            &[
                ("bounded".into(), quote!(i64), String::new()),
                ("text".into(), quote!(String), String::new()),
            ],
            &["Bounded".into(), "Text".into()],
            super::super::super::native::Mode::Domain,
        )
        .unwrap();
        syn::parse2::<syn::File>(emitted.clone()).unwrap();
        let emitted = emitted.to_string();
        assert!(emitted.contains("NEEDS_RECURSIVE_VALIDATION : bool = true"));
        assert!(emitted.contains("SignedRange"));
        assert!(emitted.contains("& self . bounded"));
        assert!(emitted.contains("& self . text"));
        assert_eq!(
            emitted.matches(":: validate_field (").count(),
            columns.len()
        );
    }
}
