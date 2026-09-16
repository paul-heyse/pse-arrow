// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Guard-aware ordered literal folding (blueprint §7.3–§7.4, ADR-0047).
use crate::{ExprGraph, MathIrError, Node, NodeId, Opcode, Payload};
use pse_quantity::{QuantityRegistry, UnitId, numeric::exact_f64_from_i64};
use std::collections::{BTreeMap, BTreeSet};

/// Changes and deferred checks; a deferred check is never success evidence.
#[derive(Clone, Debug, Default)]
pub struct FoldReport {
    /// Number of replaced literal operations.
    pub folded: usize,
    /// Nodes whose guarded/domain/numeric checks remain for their execution context.
    pub deferred_static_checks: Vec<NodeId>,
}
/// Fold eligible literal operations after physical inference, retaining ordered evaluation.
///
/// Explicit roots distinguish an unconditional use from a use only under a guard. A node
/// reachable through any guarded region is retained; its unconditional uses still undergo
/// static domain checking. Changes are atomic if a static error is found.
///
/// # Errors
/// Rejects malformed references, unresolved quantity contracts and unconditional static
/// domain violations. Overflow and guarded operations stay in the graph.
pub fn fold_literals(
    graph: &mut ExprGraph,
    roots: &[NodeId],
    registry: &QuantityRegistry,
) -> Result<FoldReport, MathIrError> {
    fold_literals_with_bindings(graph, roots, registry, &BTreeMap::new())
}
/// Guarded folding with external kernel input references included in every execution region.
///
/// # Errors
/// Rejects dangling bindings and the same unconditional static failures as [`fold_literals`].
pub fn fold_literals_with_bindings(
    graph: &mut ExprGraph,
    roots: &[NodeId],
    registry: &QuantityRegistry,
    bindings: &BTreeMap<pse_ids::SemanticId, crate::relations::vec_sink::KernelBinding>,
) -> Result<FoldReport, MathIrError> {
    let order = crate::topo::postorder_with_bindings(graph, roots, bindings)?;
    let contexts = execution_contexts(graph, roots, bindings)?;
    let mut candidate = graph.clone();
    let mut report = FoldReport::default();
    for id in order {
        let node = candidate.node(id)?.clone();
        let mode = contexts.get(&id).copied().unwrap_or(0);
        let children = node
            .children
            .iter()
            .map(|child| candidate.node(*child))
            .collect::<Result<Vec<_>, _>>()?;
        if mode & 2 != 0 {
            check_known_domains(id, &node, &children, registry, &candidate)?;
        }
        if children.iter().any(|child| {
            !matches!(
                child.payload,
                Payload::FloatConst { .. } | Payload::IntConst { .. }
            )
        }) {
            continue;
        }
        if mode & 2 == 0 {
            report.deferred_static_checks.push(id);
            continue;
        }
        let values = children
            .iter()
            .map(|child| literal(child, registry))
            .collect::<Result<Option<Vec<_>>, _>>()?;
        let Some(values) = values else {
            continue;
        };
        let numbers: Vec<_> = values.iter().map(|(value, _)| *value).collect();
        check_domain(id, node.opcode, &numbers)?;
        if !pse_schema::math::operators::operator_spec(node.opcode).foldable {
            continue;
        }
        if mode & 1 != 0 {
            report.deferred_static_checks.push(id);
            continue;
        }
        let Some(quantity_type) = node.quantity_type else {
            continue;
        };
        let target = registry
            .quantity_type(quantity_type)
            .map_err(|source| MathIrError::Quantity { node: id, source })?
            .canonical_unit;
        if !units_agree(&node, &values, target, registry, id)? {
            continue;
        }
        let Some(value) = evaluate(node.opcode, &numbers, &node.payload) else {
            continue;
        };
        if !value.is_finite() {
            report.deferred_static_checks.push(id);
            continue;
        }
        // Every supported evaluator below uses the original operation's explicit order.
        // Integer nodes remain integer only when their exact integer operation succeeds.
        let payload = if children
            .iter()
            .all(|child| matches!(child.payload, Payload::IntConst { .. }))
            && node.opcode != Opcode::UnitConvert
        {
            let ints: Vec<_> = children
                .iter()
                .filter_map(|child| match child.payload {
                    Payload::IntConst { value } => Some(value),
                    _ => None,
                })
                .collect();
            let Some(integer) = evaluate_integer(node.opcode, &ints) else {
                continue;
            };
            if exact_f64_from_i64(integer).is_none_or(|expected| {
                pse_ids::canonical_f64_bits(expected) != pse_ids::canonical_f64_bits(value)
            }) {
                continue;
            }
            Payload::IntConst { value: integer }
        } else {
            Payload::FloatConst {
                value,
                unit: target,
            }
        };
        candidate.replace_literal(id, payload)?;
        report.folded += 1;
    }
    candidate.rebuild_structural_index();
    *graph = candidate;
    Ok(report)
}
// Bit 1: guarded occurrence. Bit 2: unconditional occurrence. Neither erases the other.
fn execution_contexts(
    graph: &ExprGraph,
    roots: &[NodeId],
    bindings: &BTreeMap<pse_ids::SemanticId, crate::relations::vec_sink::KernelBinding>,
) -> Result<BTreeMap<NodeId, u8>, MathIrError> {
    let mut seen = BTreeSet::new();
    let mut contexts = BTreeMap::new();
    let mut stack: Vec<_> = roots.iter().map(|id| (*id, false)).collect();
    while let Some((id, guarded)) = stack.pop() {
        if !seen.insert((id, guarded)) {
            continue;
        }
        let node = graph.node(id)?;
        *contexts.entry(id).or_default() |= if guarded { 1 } else { 2 };
        let body_guarded = guarded
            || matches!(
                node.opcode,
                Opcode::Conditional
                    | Opcode::SumOver
                    | Opcode::ProdOver
                    | Opcode::MinOver
                    | Opcode::MaxOver
                    | Opcode::Integral
            );
        for child in &node.children {
            stack.push((*child, body_guarded));
        }
        for dependency in node.payload.referenced_nodes() {
            stack.push((dependency, guarded));
        }
        if let Payload::KernelCall { kernel_binding, .. } = node.payload {
            let binding = bindings
                .get(&kernel_binding)
                .ok_or(MathIrError::UnknownBinding {
                    node: id,
                    binding: kernel_binding,
                })?;
            for (_, input) in &binding.inputs {
                stack.push((*input, guarded));
            }
        }
    }
    Ok(contexts)
}
fn literal(node: &Node, registry: &QuantityRegistry) -> Result<Option<(f64, UnitId)>, MathIrError> {
    match node.payload {
        Payload::FloatConst { value, unit } => Ok(Some((value, unit))),
        Payload::IntConst { value } => {
            let Some(value) = exact_f64_from_i64(value) else {
                return Ok(None);
            };
            let Some(quantity) = node.quantity_type.or(registry.neutral_dimensionless()) else {
                return Ok(None);
            };
            let ty = registry
                .quantity_type(quantity)
                .map_err(|source| MathIrError::Quantity {
                    node: NodeId(0),
                    source,
                })?;
            Ok(Some((value, ty.canonical_unit)))
        }
        _ => Ok(None),
    }
}
fn check_known_domains(
    id: NodeId,
    node: &Node,
    children: &[&Node],
    registry: &QuantityRegistry,
    graph: &ExprGraph,
) -> Result<(), MathIrError> {
    let value = |index: usize| {
        children
            .get(index)
            .map(|child| literal(child, registry))
            .transpose()
            .map(Option::flatten)
    };
    if let Payload::WeightedMean {
        pairs,
        normalization: pse_quantity::WeightNormalization::DivideBySum,
        ..
    } = &node.payload
    {
        let mut sum = 0.0;
        for pair in pairs {
            let Some((weight, _)) = literal(graph.node(pair.weight)?, registry)? else {
                return Ok(());
            };
            sum += weight;
            if !sum.is_finite() {
                return Ok(());
            }
        }
        if sum == 0.0 {
            return Err(MathIrError::StaticDomain {
                node: id,
                opcode: node.opcode,
                restriction: "a nonzero ordered weight sum",
                value: sum,
            });
        }
    }
    match node.opcode {
        Opcode::Div => {
            if let Some((denominator, _)) = value(1)? {
                check_domain(id, node.opcode, &[0.0, denominator])?;
            }
        }
        Opcode::Sqrt | Opcode::Log | Opcode::Log10 | Opcode::Asin | Opcode::Acos => {
            if let Some((argument, _)) = value(0)? {
                check_domain(id, node.opcode, &[argument])?;
            }
        }
        Opcode::Pow => {
            if let (Some((base, _)), Some((exponent, _))) = (value(0)?, value(1)?) {
                check_domain(id, node.opcode, &[base, exponent])?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn check_domain(id: NodeId, opcode: Opcode, values: &[f64]) -> Result<(), MathIrError> {
    let failing = match (opcode, values) {
        (Opcode::Div, [_, denominator]) if *denominator == 0.0 => {
            Some(("a nonzero divisor", *denominator))
        }
        (Opcode::Sqrt, [value]) if *value < 0.0 => Some(("a nonnegative argument", *value)),
        (Opcode::Log | Opcode::Log10, [value]) if *value <= 0.0 => {
            Some(("a positive argument", *value))
        }
        (Opcode::Asin | Opcode::Acos, [value]) if !(-1.0..=1.0).contains(value) => {
            Some(("an argument in [-1, 1]", *value))
        }
        (Opcode::Pow, [base, exponent])
            if (*base < 0.0 && exponent.fract() != 0.0) || (*base == 0.0 && *exponent <= 0.0) =>
        {
            Some(("a base in the real power domain", *base))
        }
        _ => None,
    };
    if let Some((restriction, value)) = failing {
        return Err(MathIrError::StaticDomain {
            node: id,
            opcode,
            restriction,
            value,
        });
    }
    Ok(())
}
fn units_agree(
    node: &Node,
    values: &[(f64, UnitId)],
    target: UnitId,
    registry: &QuantityRegistry,
    id: NodeId,
) -> Result<bool, MathIrError> {
    let lookup = |unit| {
        registry
            .unit(unit)
            .map_err(|source| MathIrError::Quantity { node: id, source })
    };
    let result = lookup(target)?;
    let units = values
        .iter()
        .map(|(_, unit)| lookup(*unit))
        .collect::<Result<Vec<_>, _>>()?;
    let same = |unit: &&pse_quantity::Unit| unit.id == target;
    match (node.opcode, units.as_slice()) {
        (Opcode::Add | Opcode::Sub, [_, _]) | (Opcode::Neg | Opcode::Abs, [_]) => {
            Ok(units.iter().all(same))
        }
        (Opcode::Mul, [left, right]) => Ok(left
            .dimension
            .mul(&right.dimension)
            .map_err(pse_quantity::QuantityError::from)
            .map_err(|source| MathIrError::Quantity { node: id, source })?
            == result.dimension
            && pse_ids::canonical_f64_bits(left.scale_to_canonical * right.scale_to_canonical)
                == pse_ids::canonical_f64_bits(result.scale_to_canonical)
            && !left.is_affine
            && !right.is_affine),
        (Opcode::Div, [left, right]) => Ok(left
            .dimension
            .div(&right.dimension)
            .map_err(pse_quantity::QuantityError::from)
            .map_err(|source| MathIrError::Quantity { node: id, source })?
            == result.dimension
            && pse_ids::canonical_f64_bits(left.scale_to_canonical / right.scale_to_canonical)
                == pse_ids::canonical_f64_bits(result.scale_to_canonical)
            && !left.is_affine
            && !right.is_affine),
        (Opcode::Sqrt, [input]) => Ok(input
            .dimension
            .root(2)
            .map_err(pse_quantity::QuantityError::from)
            .map_err(|source| MathIrError::Quantity { node: id, source })?
            == result.dimension
            && pse_ids::canonical_f64_bits(input.scale_to_canonical.sqrt())
                == pse_ids::canonical_f64_bits(result.scale_to_canonical)
            && !input.is_affine),
        (Opcode::UnitConvert, [input]) => {
            let Payload::UnitConvert(spec) = node.payload else {
                return Ok(false);
            };
            let Some(ty) = node.quantity_type else {
                return Ok(false);
            };
            let ty = registry
                .quantity_type(ty)
                .map_err(|source| MathIrError::Quantity { node: id, source })?;
            let expected = pse_quantity::convert_spec_for_type(input, result, &ty.key)
                .map_err(|source| MathIrError::Quantity { node: id, source })?;
            Ok(spec.same_contract(&expected))
        }
        _ => Ok(false),
    }
}
fn evaluate(opcode: Opcode, values: &[f64], payload: &Payload) -> Option<f64> {
    match (opcode, values) {
        (Opcode::Add, [a, b]) => Some(a + b),
        (Opcode::Sub, [a, b]) => Some(a - b),
        (Opcode::Mul, [a, b]) => Some(a * b),
        (Opcode::Div, [a, b]) => Some(a / b),
        (Opcode::Neg, [a]) => Some(-a),
        (Opcode::Abs, [a]) => Some(a.abs()),
        (Opcode::Sqrt, [a]) => Some(a.sqrt()),
        (Opcode::UnitConvert, [a]) => match payload {
            Payload::UnitConvert(spec) => Some(pse_quantity::convert_value(spec, *a)),
            _ => None,
        },
        _ => None,
    }
}
fn evaluate_integer(opcode: Opcode, values: &[i64]) -> Option<i64> {
    match (opcode, values) {
        (Opcode::Add, [a, b]) => a.checked_add(*b),
        (Opcode::Sub, [a, b]) => a.checked_sub(*b),
        (Opcode::Mul, [a, b]) => a.checked_mul(*b),
        (Opcode::Div, [a, b]) if a.checked_rem(*b) == Some(0) => a.checked_div(*b),
        (Opcode::Neg, [a]) => a.checked_neg(),
        (Opcode::Abs, [a]) => a.checked_abs(),
        _ => None,
    }
}
