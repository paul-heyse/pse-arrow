// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The modeling document's remaining local checks, derived from its declaration.
//! No native-array admission or relational assurance is established here.
use crate::{
    Registry, SchemaError,
    model::{
        CollectionContract, ExtensionUse, FieldContract, IntegerRange, ReferenceContract,
        RelationSpec, TaggedAlternative, field_facets,
    },
};
use arrow_schema::DataType;
use proc_macro2::TokenStream;
use quote::quote;

pub(super) fn render(
    _registry: &Registry,
    spec: &RelationSpec,
) -> Result<TokenStream, SchemaError> {
    if spec.key.qualified_name() != "authored.modeling_declarations" {
        return Ok(TokenStream::new());
    }
    if !spec.checks.is_empty() {
        return Err(super::error(
            "typed modeling admission requires native residual evaluation for row SQL".into(),
        ));
    }
    let checks = spec
        .columns
        .iter()
        .map(|field| {
            let name = super::types::ident(field.name());
            field_checks(field, quote!(&row.#name), field.name())
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(quote! {
        /// Admit immutable generated modeling values without constructing an Arrow
        /// batch. None retains native evaluation under an unqualified planner.
        /// Keys/references and subsequent raw-array admission remain independent.
        /// # Errors
        /// Foreign declaration/context, invalid typed values or cancellation.
        #[allow(clippy::too_many_lines, reason = "one declaration-derived recursive modeling validator")]
        pub fn admit_typed_rows(
            registry: &pse_schema::Registry,
            rows: &[Row],
            validation: &crate::validate::ValidationContext,
            cancel: &pse_columnar::CancellationToken,
        ) -> Result<Option<crate::columnar::LocalFieldAdmission>, crate::RelationError> {
            cancel.checkpoint()?;
            let spec = spec(registry)?;
            if !validation.typed_modeling_local_eligible() {
                return Ok(None);
            }
            // Preparation remains under the actual current owner. Eligibility
            // additionally establishes the equivalent pinned builtin semantics.
            let prepared = validation.relation(registry, spec)?;
            for row in rows {
                cancel.checkpoint()?;
                #(#checks)*
            }
            cancel.checkpoint()?;
            Ok(Some(crate::columnar::LocalFieldAdmission::from_generated_typed_values(
                registry.contract(spec)?, prepared,
            )?))
        }
    })
}

fn field_checks(
    field: &FieldContract,
    value: TokenStream,
    path: &str,
) -> Result<TokenStream, SchemaError> {
    supported(field, path)?;
    let checks = present_checks(field, quote!(value), path)?;
    if checks.is_empty() {
        return Ok(TokenStream::new());
    }
    if field.nullable() {
        optional_checks(value, checks)
    } else {
        Ok(quote! { { let value = #value; #checks } })
    }
}

fn optional_checks(value: TokenStream, checks: TokenStream) -> Result<TokenStream, SchemaError> {
    let mut body = syn::parse2::<syn::Block>(quote!({ #checks }))
        .map_err(|error| super::error(error.to_string()))?;
    let mut condition = quote!(let Some(value) = #value);
    // Merge only a sole nested guard with no alternative. Sibling checks and
    // statements keep their declared evaluation/error order, and each later
    // `value` binding still sees the preceding optional value before shadowing.
    while let [syn::Stmt::Expr(syn::Expr::If(inner), None)] = body.stmts.as_slice()
        && inner.else_branch.is_none()
    {
        let next = &inner.cond;
        let next = if matches!(next.as_ref(), syn::Expr::Binary(binary) if matches!(binary.op, syn::BinOp::Or(_)))
        {
            quote!((#next))
        } else {
            quote!(#next)
        };
        condition = quote!(#condition && #next);
        body = inner.then_branch.clone();
    }
    Ok(quote!(if #condition #body))
}

fn supported(field: &FieldContract, path: &str) -> Result<(), SchemaError> {
    let allowed = [
        field_facets::EXTENSION,
        field_facets::PARAMETER,
        field_facets::ROLE,
        field_facets::DOC,
        field_facets::STRUCTURE,
        field_facets::EXPRESSION_ROLE,
        field_facets::EXPRESSION_ORDINAL,
        field_facets::EXPRESSION_BINDER,
        field_facets::EXPRESSION_CASES,
        field_facets::IDENTITY,
        field_facets::IDENTITY_OWNER,
        field_facets::FK_RELATION,
        field_facets::FK_COLUMN,
        field_facets::KEY_INTEGER_RANGE,
        field_facets::KEY_COLLECTION,
        field_facets::KEY_TAGGED_ALTERNATIVE,
    ];
    for key in field.field().metadata().keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(super::error(format!(
                "unsupported typed modeling facet {key} at {path}"
            )));
        }
    }
    if ReferenceContract::for_contract(field)?.is_some() {
        return Err(super::error(format!(
            "typed modeling reference presence requires native evaluation at {path}"
        )));
    }
    if field
        .field()
        .metadata()
        .contains_key(field_facets::EXTENSION)
        && field.extension().is_none()
    {
        return Err(super::error(format!(
            "unknown typed modeling extension at {path}"
        )));
    }
    if let Some(extension) = field.extension() {
        if !matches!(extension, ExtensionUse::Enum(_) | ExtensionUse::SemanticId | ExtensionUse::ExprDsl(_)) {
            return Err(super::error(format!(
                "unsupported typed modeling extension at {path}"
            )));
        }
        // These admitted domain leaves have primitive declared storage. Never
        // hide a new nested field/facet behind an extension's terminal handling.
        if field.data_type() != extension.spec().storage() || !field.children().is_empty() {
            return Err(super::error(format!(
                "typed modeling extension requires its exact primitive declaration at {path}"
            )));
        }
    }
    if CollectionContract::from_field(field.field())?.is_some_and(|collection| collection.unique) {
        return Err(super::error(format!(
            "typed modeling uniqueness requires native evaluation at {path}"
        )));
    }
    Ok(())
}

fn present_checks(
    field: &FieldContract,
    value: TokenStream,
    path: &str,
) -> Result<TokenStream, SchemaError> {
    let mut checks = Vec::new();
    if let Some(range) = IntegerRange::from_field(field.field())? {
        let (minimum, maximum) = (range.minimum, range.maximum);
        let reason = format!("typed modeling integer outside declared domain at {path}");
        checks.push(quote! { if !(#minimum..=#maximum).contains(#value) { return Err(crate::columnar::mismatch(#reason)); } });
    }
    if TaggedAlternative::from_field(field.field())?.is_some() {
        checks.push(quote! { #value.selected()?; });
    }
    if let Some(collection) = CollectionContract::from_field(field.field())?
        && !matches!(field.data_type(), DataType::FixedSizeList(..))
    {
        // from_field already checks that a FixedSizeList width satisfies its
        // declared bounds; generated [T; N] then establishes that cardinality.
        let minimum =
            usize::try_from(collection.minimum).map_err(|error| super::error(error.to_string()))?;
        let reason = format!("typed modeling collection outside declared cardinality at {path}");
        if minimum > 0 {
            checks.push(quote! { if #value.len() < #minimum { return Err(crate::columnar::mismatch(#reason)); } });
        }
        if let Some(maximum) = collection.maximum {
            let maximum =
                usize::try_from(maximum).map_err(|error| super::error(error.to_string()))?;
            checks.push(quote! { if #value.len() > #maximum { return Err(crate::columnar::mismatch(#reason)); } });
        }
    }
    if field.extension().is_some() {
        // Generated enums/typed IDs establish their values under the exact
        // generated declaration. Native preparation still admits their metadata.
        return Ok(quote!(#(#checks)*));
    }
    match field.data_type() {
        DataType::Float32 | DataType::Float64 => {
            let reason = format!("typed modeling nonfinite number at {path}");
            checks.push(quote! { if !#value.is_finite() { return Err(crate::columnar::mismatch(#reason)); } });
        }
        DataType::Struct(_) => {
            for child in field.children() {
                let name = super::types::ident(child.name());
                checks.push(field_checks(
                    &child,
                    quote!(&#value.#name),
                    &format!("{path}/{}", child.name()),
                )?);
            }
        }
        DataType::List(_) | DataType::FixedSizeList(..) => {
            let child = field
                .children()
                .into_iter()
                .next()
                .ok_or_else(|| super::error(format!("missing typed list child at {path}")))?;
            let body = field_checks(&child, quote!(item), &format!("{path}/*"))?;
            if !body.is_empty() {
                checks.push(quote! { for item in #value { cancel.checkpoint()?; #body } });
            }
        }
        DataType::Boolean
        | DataType::Int8
        | DataType::Int16
        | DataType::Int32
        | DataType::Int64
        | DataType::UInt8
        | DataType::UInt16
        | DataType::UInt32
        | DataType::Utf8 => {}
        kind => {
            return Err(super::error(format!(
                "unsupported typed modeling storage {kind} at {path}"
            )));
        }
    }
    Ok(quote!(#(#checks)*))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "fixed generation refusal fixtures")]
    use super::*;
    #[test]
    fn unsupported_facets_sql_and_uniqueness_cannot_silently_generate_admission() {
        let registry = pse_schema::catalog::assemble().unwrap();
        let original = registry.relation("authored.modeling_declarations").unwrap();
        let mut spec = original.clone();
        assert!(render(&registry, &spec).is_ok());
        spec.checks.insert("future".into(), "ordinal > 0".into());
        assert!(render(&registry, &spec).is_err());
        let mut spec = original.clone();
        spec.columns[4] = spec.columns[4].clone().with_quantity("FutureQuantity");
        assert!(render(&registry, &spec).is_err());
        let field = FieldContract::list(FieldContract::native(DataType::Int64))
            .with_collection(CollectionContract::SET);
        assert!(field_checks(&field, quote!(value), "set").is_err());
    }
    #[test]
    fn fixed_arrays_cannot_hide_excluded_width_or_unsupported_nested_facets() {
        let fixed = FieldContract::fixed_list(FieldContract::native(DataType::Int64), 2);
        for collection in [
            CollectionContract {
                minimum: 3,
                maximum: None,
                ..CollectionContract::SEQUENCE
            },
            CollectionContract {
                maximum: Some(1),
                ..CollectionContract::SEQUENCE
            },
        ] {
            assert!(
                field_checks(
                    &fixed.clone().with_collection(collection),
                    quote!(value),
                    "fixed"
                )
                .is_err()
            );
        }
        let child = FieldContract::from_field(
            arrow_schema::Field::new("child", DataType::Int64, false).with_metadata(
                std::collections::HashMap::from([("future.meaning".into(), "unknown".into())]),
            ),
        );
        assert!(
            field_checks(
                &FieldContract::structure(vec![child.clone()]),
                quote!(value),
                "plain"
            )
            .is_err()
        );
        let disguised = FieldContract::enumeration("Choice");
        let disguised = FieldContract::from_field(
            disguised
                .field()
                .clone()
                .with_data_type(DataType::Struct(vec![child.into_field()].into())),
        );
        assert!(field_checks(&disguised, quote!(value), "disguised").is_err());
    }
}
