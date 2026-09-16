// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Resolve an ambiguous literal occurrence against an actual binary operation.
use super::{Driver, Frame};
use crate::{MathIrError, Node, NodeId, Opcode, Payload};
use pse_quantity::infer::{OpRequest, Operand, infer_with_evidence};
use pse_quantity::literal::{LiteralContext, resolve_literal};
use pse_quantity::{IndexSet, QuantityError, QuantityTypeId};

impl Driver<'_, '_> {
    pub(super) fn additive_expected(
        &self,
        frame: &Frame,
        slot: usize,
    ) -> Result<Option<QuantityTypeId>, MathIrError> {
        let Some(sibling) = frame.values.get(1 - slot).and_then(Option::as_ref) else {
            return Ok(frame.request.expected);
        };
        let registry = self.input.registry;
        let mut key = self
            .ty(frame.request.node, sibling.quantity_type)?
            .key
            .clone();
        let operation = if frame.node.opcode == Opcode::Sub {
            OpRequest::Sub
        } else {
            OpRequest::Add
        };
        let mut choices = Vec::new();
        for candidate in registry.quantity_types() {
            key.scale_kind = candidate.key.scale_kind;
            if candidate.key != key {
                continue;
            }
            let mut operands = [Operand {
                quantity_type: sibling.quantity_type,
                indices: &sibling.indices,
            }; 2];
            operands[slot].quantity_type = candidate.id;
            let inferred = infer_with_evidence(
                &operation,
                &operands,
                registry,
                self.input.symbols.invariant_checker(frame.request.node),
            );
            let result = match inferred {
                Ok(result) => result,
                Err(QuantityError::Incompatible { .. }) => continue,
                Err(source) => {
                    return Err(MathIrError::Quantity {
                        node: frame.request.node,
                        source,
                    });
                }
            };
            if let Some(expected) = frame.node.quantity_type.or(frame.request.expected) {
                match pse_quantity::admission::require_same_contract(
                    expected,
                    result.result,
                    registry,
                ) {
                    Ok(()) => {}
                    Err(QuantityError::ContractMismatch { .. }) => continue,
                    Err(source) => {
                        return Err(MathIrError::Quantity {
                            node: frame.request.node,
                            source,
                        });
                    }
                }
            }
            // Adjacent additive context preserves its complete type when admissible.
            // Point addition instead requires the registered difference contract.
            if candidate.id == sibling.quantity_type {
                return Ok(Some(candidate.id));
            }
            choices.push(candidate.id);
        }
        Ok(if let [only] = choices.as_slice() {
            Some(*only)
        } else {
            None
        })
    }

    pub(super) fn literal_priority(&self, mut node: NodeId) -> u8 {
        loop {
            match self.input.graph.node(node) {
                Ok(Node {
                    quantity_type: Some(_),
                    ..
                }) => return 0,
                Ok(Node {
                    opcode: Opcode::Broadcast,
                    children,
                    ..
                }) => {
                    let Some(child) = children.first() else {
                        return 0;
                    };
                    node = *child;
                }
                Ok(Node {
                    payload: Payload::IntConst { .. },
                    ..
                }) => return 1,
                Ok(Node {
                    payload: Payload::FloatConst { .. },
                    ..
                }) => return 2,
                _ => return 0,
            }
        }
    }

    pub(super) fn literal_operand_expected(
        &self,
        frame: &Frame,
        slot: usize,
    ) -> Result<Option<QuantityTypeId>, MathIrError> {
        let operation = match frame.node.opcode {
            Opcode::Mul => OpRequest::Mul,
            Opcode::Div => OpRequest::Div,
            _ => return Ok(None),
        };
        if frame.dependencies.len() != 2 {
            return Ok(None);
        }
        let Some(sibling) = frame.values[1 - slot].as_ref() else {
            return Ok(None);
        };
        let literal = self.input.graph.node(frame.dependencies[slot].node)?;
        let Payload::FloatConst { unit, .. } = literal.payload else {
            return Ok(None);
        };
        if literal.quantity_type.is_some() {
            return Ok(None);
        }
        let registry = self.input.registry;
        let candidates = match resolve_literal(unit, LiteralContext::Free, registry) {
            Ok(_) => return Ok(None),
            Err(QuantityError::AmbiguousLiteral { candidates, .. }) => candidates,
            Err(source) => {
                return Err(MathIrError::Quantity {
                    node: frame.dependencies[slot].node,
                    source,
                });
            }
        };
        let scalar = IndexSet::new();
        let mut selected = None;
        for candidate in candidates {
            let admissible = || {
                resolve_literal(
                    unit,
                    LiteralContext::Explicit {
                        quantity_type: candidate,
                    },
                    registry,
                )?;
                let mut operands = [Operand {
                    quantity_type: sibling.quantity_type,
                    indices: &sibling.indices,
                }; 2];
                operands[slot] = Operand {
                    quantity_type: candidate,
                    indices: &scalar,
                };
                let result = infer_with_evidence(
                    &operation,
                    &operands,
                    registry,
                    self.input.symbols.invariant_checker(frame.request.node),
                )?;
                if let Some(expected) = frame.node.quantity_type.or(frame.request.expected) {
                    pse_quantity::admission::require_same_contract(
                        expected,
                        result.result,
                        registry,
                    )?;
                }
                Ok(())
            };
            match admissible() {
                Ok(()) => {
                    if selected.replace(candidate).is_some() {
                        return Ok(None);
                    }
                }
                Err(
                    QuantityError::OperationUnsupported {
                        ordered_matches: 0,
                        swapped_matches: 0,
                        ..
                    }
                    | QuantityError::Incompatible { .. }
                    | QuantityError::ContractMismatch { .. },
                ) => {}
                // Registry errors and unresolved operation prerequisites must remain visible.
                Err(source) => {
                    return Err(MathIrError::Quantity {
                        node: frame.request.node,
                        source,
                    });
                }
            }
        }
        Ok(selected)
    }
}
