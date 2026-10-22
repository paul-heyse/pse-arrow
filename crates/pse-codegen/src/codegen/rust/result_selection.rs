// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Scientific selection inventories projected from the actual field declarations.
use crate::{Registry, SchemaError};
use proc_macro2::TokenStream;
use quote::quote;

pub(super) fn render(reg: &Registry) -> Result<TokenStream, SchemaError> {
    let mut entries = Vec::new();
    for relation in reg.relations() {
        let mut run = None;
        let mut owner = None;
        let mut step = None;
        let mut experiment = None;
        let mut values = Vec::new();
        let mut order = std::collections::BTreeMap::new();
        for column in &relation.columns {
            use crate::model::field_facets::{RESULT_SELECTION, RESULT_SORT};
            if column.field().metadata().contains_key(RESULT_SELECTION)
                && column.result_selection_role().is_none()
                || column.field().metadata().contains_key(RESULT_SORT)
                    && column.result_sort_ordinal().is_none()
            {
                return Err(super::error(format!(
                    "{}.{} has an invalid result selection facet",
                    relation.key.qualified_name(),
                    column.name()
                )));
            }
            if let Some(ordinal) = column.result_sort_ordinal()
                && order.insert(ordinal, column.name()).is_some()
            {
                return Err(super::error(format!(
                    "{} repeats result sort ordinal {ordinal}",
                    relation.key.qualified_name()
                )));
            }
            use crate::model::ResultSelectionRole as Role;
            let target = match column.result_selection_role() {
                Some(Role::Run) => Some(&mut run),
                Some(Role::Owner) => Some(&mut owner),
                Some(Role::Step) => Some(&mut step),
                Some(Role::Experiment) => Some(&mut experiment),
                Some(Role::Value) => {
                    if column.data_type() != arrow_schema::DataType::Float64 {
                        return Err(super::error(format!(
                            "{}.{} selects a non-Float64 value",
                            relation.key.qualified_name(),
                            column.name()
                        )));
                    }
                    values.push(column.name());
                    None
                }
                None => None,
            };
            if let Some(target) = target
                && target.replace(column.name()).is_some()
            {
                return Err(super::error(format!(
                    "{} repeats a result selection role",
                    relation.key.qualified_name()
                )));
            }
        }
        if run.is_none() && owner.is_none() && values.is_empty() && order.is_empty() {
            continue;
        }
        let run = run.ok_or_else(|| {
            super::error(format!(
                "{} has no result run",
                relation.key.qualified_name()
            ))
        })?;
        let owner = owner.ok_or_else(|| {
            super::error(format!(
                "{} has no result owner",
                relation.key.qualified_name()
            ))
        })?;
        if values.is_empty() || step.is_some() && experiment.is_some() {
            return Err(super::error(format!(
                "{} has an incomplete result selection contract",
                relation.key.qualified_name()
            )));
        }
        if order.keys().copied().ne(0..order.len()) {
            return Err(super::error(format!(
                "{} has noncontiguous result sort ordinals",
                relation.key.qualified_name()
            )));
        }
        let optional =
            |name: Option<&str>| name.map_or_else(|| quote!(None), |name| quote!(Some(#name)));
        let step = optional(step);
        let experiment = optional(experiment);
        let order = order.values().collect::<Vec<_>>();
        let bytes = relation.id.as_bytes();
        entries.push(quote!(ResultSelection {
            relation: pse_ids::SemanticId::from_bytes([#(#bytes),*]),
            run: #run, owner: #owner, step: #step, experiment: #experiment,
            sort: &[#(#order),*], values: &[#(#values),*],
        }));
    }
    Ok(quote! {
        /// Registry-owned scalar access and new-batch ordering contract.
        #[derive(Clone, Copy, Debug)]
        pub struct ResultSelection {
            /// Exact scientific relation identity.
            pub relation: pse_ids::SemanticId,
            /// Original execution identity column.
            pub run: &'static str,
            /// Scientific output identity column.
            pub owner: &'static str,
            /// Original solve step, when this relation has one.
            pub step: Option<&'static str>,
            /// Original experiment partition, when this relation has one.
            pub experiment: Option<&'static str>,
            /// New scalar batch order; empty for dense trajectories with their own ordering.
            pub sort: &'static [&'static str],
            /// Float64 fields eligible for scientific access predicates.
            pub values: &'static [&'static str],
        }
        /// Complete declared selection inventory.
        pub const RESULT_SELECTIONS: &[ResultSelection] = &[#(#entries),*];
        /// Resolve an exact relation; an undeclared relation has no scalar access contract.
        pub fn for_relation(relation: pse_ids::SemanticId) -> Option<&'static ResultSelection> {
            RESULT_SELECTIONS.iter().find(|entry| entry.relation == relation)
        }
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn result_selection_inventory_comes_from_the_scientific_declarations() {
        let registry = &crate::catalog::assemble().unwrap();
        let parsed = syn::parse2::<syn::File>(super::render(registry).unwrap()).unwrap();
        let entries = parsed
            .items
            .iter()
            .find_map(|item| match item {
                syn::Item::Const(item) if item.ident == "RESULT_SELECTIONS" => Some(&item.expr),
                _ => None,
            })
            .unwrap();
        let syn::Expr::Reference(entries) = entries.as_ref() else {
            panic!("inventory is a static slice")
        };
        let syn::Expr::Array(entries) = entries.expr.as_ref() else {
            panic!("inventory has concrete entries")
        };
        let fields = entries
            .elems
            .iter()
            .map(|entry| {
                let syn::Expr::Struct(entry) = entry else {
                    panic!("typed selection entry")
                };
                &entry.fields
            })
            .collect::<Vec<_>>();
        assert_eq!(fields.len(), 5);
        let field = |fields: &syn::punctuated::Punctuated<syn::FieldValue, syn::token::Comma>,
                     name: &str| {
            fields
                .iter()
                .find(|field| matches!(&field.member, syn::Member::Named(member) if member == name))
                .unwrap()
                .expr
                .clone()
        };
        let strings = |value: syn::Expr| {
            let syn::Expr::Reference(value) = value else {
                panic!("declared string slice")
            };
            let syn::Expr::Array(value) = *value.expr else {
                panic!("literal string array")
            };
            value
                .elems
                .into_iter()
                .map(|value| match value {
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(value),
                        ..
                    }) => value.value(),
                    _ => panic!("literal field name"),
                })
                .collect::<Vec<_>>()
        };
        let orders = fields
            .iter()
            .map(|fields| strings(field(fields, "sort")))
            .collect::<Vec<_>>();
        assert!(orders.contains(&vec![
            "run_id".to_owned(),
            "experiment_id".into(),
            "observation_id".into()
        ]));
        assert!(
            orders.contains(&Vec::new()),
            "dense trajectory ordering stays separate"
        );
        assert!(
            fields
                .iter()
                .any(|fields| strings(field(fields, "values")) == ["value", "scale"])
        );
    }
}
