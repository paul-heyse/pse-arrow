// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Ordered, framed node identities (blueprint §7.4, ADR-0030).
//!
//! These digests identify content; they establish neither graph validity nor physical
//! correctness. Sharing and reuse still compare complete contracts and dependencies.
use crate::{MathIrError, Node, NodeId};
use pse_ids::{ContentHash, derive::context, derive_hash};
use pse_quantity::QuantityTypeId;

/// Hash an untyped structure independently of its temporary node numbers and scope.
/// `dependency_hashes` contains ordinary children first, then `payload.referenced_nodes()`.
///
/// # Errors
/// Rejects an incomplete/excess dependency list. This is not semantic admission.
pub fn structural_hash(
    node: &Node,
    dependency_hashes: &[ContentHash],
) -> Result<ContentHash, MathIrError> {
    hash_node(node, None, dependency_hashes)
}
/// Hash a typed structure, including the complete quantity contract's registry identity.
/// `dependency_hashes` uses the same ordering as [`structural_hash`].
///
/// # Errors
/// Rejects an incomplete/excess dependency list. A caller must establish type validity.
pub fn subtree_hash(
    node: &Node,
    quantity_type: QuantityTypeId,
    dependency_hashes: &[ContentHash],
) -> Result<ContentHash, MathIrError> {
    hash_node(node, Some(quantity_type), dependency_hashes)
}
/// Hash a kernel call with every actual binding value and ordered input dependency.
/// The binding declaration is content; its ID alone cannot stand in for those facts.
///
/// # Errors
/// Rejects a mismatched call/binding or incomplete dependency list.
pub fn subtree_hash_with_binding(
    node: &Node,
    quantity_type: Option<QuantityTypeId>,
    dependency_hashes: &[ContentHash],
    binding: &crate::relations::vec_sink::KernelBinding,
) -> Result<ContentHash, MathIrError> {
    if !matches!(node.payload, crate::Payload::KernelCall { kernel_binding, .. } if kernel_binding == binding.binding)
    {
        return Err(MathIrError::malformed(
            "kernel hash requires the call's actual binding",
        ));
    }
    if dependency_hashes.len() != binding.inputs.len() || !node.children.is_empty() {
        return Err(MathIrError::malformed(
            "kernel hash requires all ordered bound inputs",
        ));
    }
    let mut payload = Vec::new();
    crate::graph::encode_payload(&mut payload, &node.payload);
    let mut bound = Vec::new();
    bound.extend_from_slice(binding.binding.as_bytes());
    bound.extend_from_slice(binding.kernel.as_bytes());
    bound.extend_from_slice(binding.scope.as_bytes());
    bound.extend_from_slice(&(binding.parameters.len() as u64).to_le_bytes());
    for (name, symbol, value, unit) in &binding.parameters {
        frame(&mut bound, name.as_bytes());
        optional_id(&mut bound, *symbol);
        if let Some(value) = value {
            bound.push(1);
            bound.extend_from_slice(&pse_ids::canonical_f64_bits(*value).to_le_bytes());
        } else {
            bound.push(0);
        }
        optional_id(&mut bound, unit.map(pse_quantity::UnitId::as_id));
    }
    bound.extend_from_slice(&(binding.inputs.len() as u64).to_le_bytes());
    for ((name, _), hash) in binding.inputs.iter().zip(dependency_hashes) {
        frame(&mut bound, name.as_bytes());
        bound.extend_from_slice(hash.as_bytes());
    }
    let mut quantity = Vec::new();
    optional_id(&mut quantity, quantity_type.map(QuantityTypeId::as_id));
    Ok(derive_hash(
        context::MATHIR_NODE,
        &[node.opcode.as_str().as_bytes(), &payload, &quantity, &bound],
    ))
}
fn frame(bytes: &mut Vec<u8>, value: &[u8]) {
    bytes.extend_from_slice(&(value.len() as u64).to_le_bytes());
    bytes.extend_from_slice(value);
}
fn optional_id(bytes: &mut Vec<u8>, value: Option<pse_ids::SemanticId>) {
    if let Some(value) = value {
        bytes.push(1);
        bytes.extend_from_slice(value.as_bytes());
    } else {
        bytes.push(0);
    }
}
fn hash_node(
    node: &Node,
    quantity_type: Option<QuantityTypeId>,
    dependencies: &[ContentHash],
) -> Result<ContentHash, MathIrError> {
    if matches!(node.payload, crate::Payload::KernelCall { .. }) {
        return Err(MathIrError::malformed(
            "kernel node hashing requires resolved binding input dependencies",
        ));
    }
    let payload_refs = node.payload.referenced_nodes();
    if dependencies.len() != node.children.len() + payload_refs.len() {
        return Err(MathIrError::malformed(
            "node hashing requires every ordered child and payload-reference hash",
        ));
    }
    let mut payload = node.payload.clone();
    // Encode reference slots, not artifact-local numbers. Each slot's actual dependency
    // hash follows in the separately counted dependency frame.
    let mut ordinal = 0;
    payload.map_node_references(|_| {
        let id = NodeId(ordinal);
        ordinal += 1;
        Ok(id)
    })?;
    let mut encoded = Vec::new();
    crate::graph::encode_payload(&mut encoded, &payload);
    let mut types = Vec::new();
    if let Some(ty) = quantity_type {
        types.push(1);
        types.extend_from_slice(ty.as_id().as_bytes());
    } else {
        types.push(0);
    }
    let mut references = Vec::new();
    references.extend_from_slice(&(node.children.len() as u64).to_le_bytes());
    references.extend_from_slice(&(payload_refs.len() as u64).to_le_bytes());
    for dependency in dependencies {
        references.extend_from_slice(dependency.as_bytes());
    }
    Ok(derive_hash(
        context::MATHIR_NODE,
        &[
            node.opcode.as_str().as_bytes(),
            &encoded,
            &types,
            &references,
        ],
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Opcode, Payload};
    use pse_ids::SemanticId;
    fn literal(value: f64) -> Node {
        Node {
            opcode: Opcode::Const,
            payload: Payload::FloatConst {
                value,
                unit: pse_quantity::UnitId::from_id(SemanticId::NIL),
            },
            children: vec![],
            quantity_type: None,
            scope: None,
        }
    }
    #[test]
    fn float_sign_order_and_quantity_contract_enter_identity_but_scope_does_not() {
        let positive = structural_hash(&literal(0.0), &[]).expect("leaf");
        let negative = structural_hash(&literal(-0.0), &[]).expect("leaf");
        assert_ne!(positive, negative);
        let mut sum = Node {
            opcode: Opcode::Add,
            payload: Payload::None,
            children: vec![NodeId(1), NodeId(2)],
            quantity_type: None,
            scope: None,
        };
        let forward = structural_hash(&sum, &[positive, negative]).expect("two refs");
        assert_ne!(
            forward,
            structural_hash(&sum, &[negative, positive]).expect("two refs")
        );
        sum.scope = Some(SemanticId::NIL);
        sum.children = vec![NodeId(7), NodeId(9)];
        assert_eq!(
            forward,
            structural_hash(&sum, &[positive, negative]).expect("renumbered")
        );
        assert_ne!(
            forward,
            subtree_hash(
                &sum,
                QuantityTypeId::from_id(SemanticId::NIL),
                &[positive, negative]
            )
            .expect("typed")
        );
        assert!(structural_hash(&sum, &[positive]).is_err());
    }
    #[test]
    fn payload_references_are_real_ordered_hash_dependencies() {
        let a = structural_hash(&literal(1.0), &[]).expect("leaf");
        let b = structural_hash(&literal(2.0), &[]).expect("leaf");
        let node = Node {
            opcode: Opcode::Conditional,
            payload: Payload::Conditional {
                guard: NodeId(99).into(),
            },
            children: vec![NodeId(1), NodeId(2)],
            quantity_type: None,
            scope: None,
        };
        assert!(structural_hash(&node, &[a, b]).is_err());
        assert_ne!(
            structural_hash(&node, &[a, b, a]).expect("guard"),
            structural_hash(&node, &[a, b, b]).expect("changed guard")
        );
    }
}
