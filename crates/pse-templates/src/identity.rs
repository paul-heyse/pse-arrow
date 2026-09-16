// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Shared identity formulas for actual ordered domain products and group projections.
//! Identity never proves that a supplied factor/member inventory is correct (ADR-0063).
use pse_ids::SemanticId;

/// Identity of an exact ordered domain factor vector, including the scalar empty product.
/// Actual factors, uniqueness and source membership remain independently validated.
pub fn domain_product_id(domains: &[SemanticId]) -> SemanticId {
    let mut name = String::from("pse:domain-product:v1:");
    for domain in domains {
        name.push_str(&domain.to_hex());
    }
    pse_ids::named_id(SemanticId::NIL, &name)
}

/// Identity of a projection of one actual group on ordered fixed axis members.
/// The caller validates sorted unique axis positions and actual member correspondence.
pub fn projected_group_id(group: SemanticId, fixed: &[(u16, SemanticId)]) -> SemanticId {
    let mut name = String::from("pse:group-projection:v1:");
    for (axis, member) in fixed {
        name.push_str(&axis.to_string());
        name.push('=');
        name.push_str(&member.to_hex());
        name.push(';');
    }
    pse_ids::named_id(group, &name)
}

/// Actual group for one declared symbol in one actual instance.
pub fn symbol_group_id(instance: SemanticId, declaration: SemanticId) -> SemanticId {
    pse_ids::named_id(
        instance,
        &format!("pse:symbol-group:v1:{}", declaration.to_hex()),
    )
}

/// Actual cross-child group for one source-relative path occurrence.
pub fn group_collection_id(instance: SemanticId, source: SemanticId, path: u64) -> SemanticId {
    pse_ids::named_id(
        instance,
        &format!("pse:group-collection:v1:{}:{path}", source.to_hex()),
    )
}

/// Actual lexical binder for one source binder in one instance occurrence.
pub fn bound_index_id(instance: SemanticId, source: SemanticId) -> SemanticId {
    pse_ids::named_id(instance, &format!("pse:bound-index:v1:{}", source.to_hex()))
}

/// Outer declared axis shared across exact source fields of the same declaration.
pub fn free_index_id(instance: SemanticId, declaration: SemanticId, position: u16) -> SemanticId {
    pse_ids::named_id(
        instance,
        &format!("pse:free-index:v1:{}:{position}", declaration.to_hex()),
    )
}

/// Source occurrence and exact fixed environment of an arithmetic reindexing.
pub fn reindexed_group_id(
    instance: SemanticId,
    source: SemanticId,
    node: u64,
    fixed: &std::collections::BTreeMap<SemanticId, SemanticId>,
) -> SemanticId {
    let mut name = format!("pse:group-reindex:v1:{}:{node}:", source.to_hex());
    for (binder, member) in fixed {
        name.push_str(&binder.to_hex());
        name.push('=');
        name.push_str(&member.to_hex());
        name.push(';');
    }
    pse_ids::named_id(instance, &name)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn products_and_projections_preserve_exact_order_and_empty_scalar_identity() {
        let a = SemanticId::from_bytes([1; 16]);
        let b = SemanticId::from_bytes([2; 16]);
        assert_eq!(
            domain_product_id(&[]),
            pse_ids::named_id(SemanticId::NIL, "pse:domain-product:v1:")
        );
        assert_ne!(domain_product_id(&[a, b]), domain_product_id(&[b, a]));
        assert_ne!(
            projected_group_id(a, &[(0, b)]),
            projected_group_id(a, &[(1, b)])
        );
        assert_eq!(
            domain_product_id(&[a]),
            pse_ids::named_id(
                SemanticId::NIL,
                &format!("pse:domain-product:v1:{}", a.to_hex())
            )
        );
    }
}
