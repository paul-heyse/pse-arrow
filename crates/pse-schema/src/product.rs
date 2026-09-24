// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Durable support closure over the sole registry declarations.
use crate::{
    Registry, SchemaError,
    model::{FieldContract, ReferenceContract},
};
use pse_ids::SemanticId;
use std::collections::BTreeSet;

/// Include declared references and relational invariant inputs, preserving empty members.
/// This is a declaration closure, never a scan or a second execution graph.
/// # Errors
/// An unknown requested relation or malformed reference declaration.
pub fn support_closure(
    registry: &Registry,
    roots: &BTreeSet<SemanticId>,
) -> Result<BTreeSet<SemanticId>, SchemaError> {
    let mut selected = BTreeSet::new();
    let mut pending: Vec<_> = roots.iter().copied().collect();
    while let Some(id) = pending.pop() {
        if !selected.insert(id) {
            continue;
        }
        let spec = registry
            .relation_by_id(id)
            .ok_or_else(|| SchemaError::UnknownReference {
                context: "product support".into(),
                reference: id.to_string(),
            })?;
        let mut fields = spec.columns.clone();
        while let Some(field) = fields.pop() {
            if let Some(reference) = ReferenceContract::for_contract(&field)? {
                add(registry, &reference.relation, &mut pending)?;
            }
            // Source spans are implicit contracts, not ordinary scalar FKs.
            if field.extension().is_some_and(|extension| {
                matches!(extension, crate::model::ExtensionUse::SourceSpan)
            }) {
                add(registry, "authored.documents", &mut pending)?;
            }
            if field.extension().is_some_and(|extension| {
                matches!(extension, crate::model::ExtensionUse::QuantityValue)
            }) {
                add(registry, "reference.quantity_types", &mut pending)?;
                add(registry, "reference.units", &mut pending)?;
            }
            fields.extend(FieldContract::children(&field));
        }
        for invariant in registry
            .invariants()
            .iter()
            .filter(|value| value.relation == spec.key.qualified_name())
        {
            for input in &invariant.inputs {
                add(registry, input, &mut pending)?;
            }
        }
    }
    Ok(selected)
}
fn add(registry: &Registry, name: &str, pending: &mut Vec<SemanticId>) -> Result<(), SchemaError> {
    let spec = registry
        .relation(name)
        .ok_or_else(|| SchemaError::UnknownReference {
            context: "product support".into(),
            reference: name.into(),
        })?;
    pending.push(spec.id);
    Ok(())
}

#[cfg(test)]
mod durability_unit {
    use super::*;
    #[test]
    fn product_roots_do_not_select_all_stages_and_support_closes_idempotently() {
        let registry = crate::catalog::assemble().unwrap();
        let roots = registry.artifact_profile("source").unwrap();
        assert!(
            roots
                .iter()
                .all(|id| registry.relation_by_id(*id).unwrap().key.namespace
                    != crate::model::Namespace::Normalized)
        );
        let closure = support_closure(&registry, roots).unwrap();
        assert!(roots.is_subset(&closure));
        assert_eq!(support_closure(&registry, &closure).unwrap(), closure);
        assert!(closure.len() < registry.relations().len());
        assert!(
            registry
                .artifact_profile("run")
                .unwrap()
                .iter()
                .all(|id| registry.relation_by_id(*id).unwrap().key.name != "solver_outcomes")
        );
    }
}
