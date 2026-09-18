// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Scalar mathematical facts lower to native expressions; no graph survives preparation.
use crate::{
    EvaluationProgram, NumericsError,
    error::{input, unsupported},
};
use datafusion::{
    arrow::datatypes::{DataType, SchemaRef},
    common::{
        Column, ScalarValue,
        tree_node::{TreeNode, TreeNodeRecursion},
    },
    execution::session_state::SessionState,
    functions::math::expr_fn as math,
    logical_expr::{Expr, lit, when},
};
use pse_ids::{CancellationToken, MemoryReserver, SemanticId};
use pse_mathir::{
    CanonicalGraph, CanonicalNode, NodeId, Opcode, Payload, ValueRef, WeightNormalization,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

/// Borrowed scalar mathematical inputs. The graph is an algorithm view of typed
/// relations; native column bindings select the actual case values. Index expansion,
/// physical admission and kernel binding are prerequisites, never inferred here.
#[derive(Debug)]
pub struct ScalarMath<'a> {
    /// Actual admitted mathematical values, borrowed only during preparation.
    pub graph: &'a CanonicalGraph,
    /// Residual/body roots in problem order; caller retains equation identities/bounds.
    pub roots: &'a [NodeId],
    /// Exact native value expression for each referenced scalar symbol.
    pub symbols: &'a BTreeMap<SemanticId, Expr>,
}
impl ScalarMath<'_> {
    /// Lower scalar facts and prepare native residual/Jacobian execution in one call.
    /// No predecessor graph, mathematical store or ambient session is retained.
    /// # Errors
    /// Unexpanded indices, unresolved symbols, missing native/kernel bindings,
    /// invalid mathematical facts, cancellation, resource refusal or derivative failure.
    pub fn compile(
        &self,
        program_id: SemanticId,
        variables: &[Column],
        schema: SchemaRef,
        state: &SessionState,
        reserver: Arc<dyn MemoryReserver>,
        cancel: CancellationToken,
    ) -> Result<EvaluationProgram, NumericsError> {
        cancel.checkpoint()?;
        let mut allocation = reserver.open("numerics:scalar-math-lowering");
        allocation
            .try_grow(
                self.graph
                    .len()
                    .checked_mul(256)
                    .ok_or_else(|| input("math inventory extent overflow"))?,
            )
            .map_err(pse_ids::CanonError::from)?;
        let needed = self.dependencies()?;
        let mut native = BTreeMap::<NodeId, (Expr, usize)>::new();
        for (id, node) in self.graph.iter().filter(|(id, _)| needed.contains(id)) {
            cancel.checkpoint()?;
            if !node.free_indices.is_empty() {
                return Err(unsupported(format!(
                    "{id} retains free indices; scalar expansion is required"
                )));
            }
            let mut count = match &node.payload {
                Payload::SymbolRef {
                    symbol: ValueRef::ActualSymbol(symbol),
                } => self
                    .symbols
                    .get(symbol)
                    .map(|expression| {
                        crate::expressions::expression_count(std::slice::from_ref(expression))
                    })
                    .transpose()?
                    .unwrap_or(1),
                _ => 1,
            };
            for source in node
                .children
                .iter()
                .copied()
                .chain(node.payload.referenced_nodes())
            {
                let (_, size) = native
                    .get(&source)
                    .ok_or_else(|| input("canonical dependency is absent or unordered"))?;
                count = count
                    .checked_add(*size)
                    .ok_or_else(|| input("native math expression extent overflow"))?;
            }
            // Smoothing and weighted means repeat operands. Reserve a local upper
            // bound before cloning; subsequent nodes use the actual expanded size.
            let extent = count
                .checked_mul(4)
                .and_then(|n| n.checked_add(20))
                .and_then(|n| n.checked_mul(size_of::<Expr>() + 128))
                .ok_or_else(|| input("native math allocation extent overflow"))?;
            allocation
                .try_grow(extent)
                .map_err(pse_ids::CanonError::from)?;
            let expression = self.node(id, node, &native)?;
            let mut size = 0;
            expression
                .apply(|_| {
                    size += 1;
                    Ok(TreeNodeRecursion::Continue)
                })
                .map_err(NumericsError::preparation)?;
            native.insert(id, (expression, size));
        }
        let root_extent = self
            .roots
            .iter()
            .try_fold(0usize, |total, id| {
                let (_, size) = native
                    .get(id)
                    .ok_or_else(|| input("requested scalar root was not lowered"))?;
                total
                    .checked_add(*size)
                    .ok_or_else(|| input("native root extent overflow"))
            })?
            .checked_mul(size_of::<Expr>() + 128)
            .ok_or_else(|| input("native root allocation overflow"))?;
        allocation
            .try_grow(root_extent)
            .map_err(pse_ids::CanonError::from)?;
        let roots = self
            .roots
            .iter()
            .map(|id| {
                native
                    .get(id)
                    .map(|(expr, _)| expr.clone())
                    .ok_or_else(|| input("requested scalar root was not lowered"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        EvaluationProgram::compile(
            program_id, &roots, variables, schema, state, reserver, cancel,
        )
    }
    fn dependencies(&self) -> Result<BTreeSet<NodeId>, NumericsError> {
        let mut needed = BTreeSet::new();
        let mut pending = self.roots.to_vec();
        while let Some(id) = pending.pop() {
            if needed.insert(id) {
                let node = self.graph.node(id)?;
                pending.extend(node.children.iter().copied());
                pending.extend(node.payload.referenced_nodes());
            }
        }
        Ok(needed)
    }
    fn node(
        &self,
        id: NodeId,
        node: &CanonicalNode,
        native: &BTreeMap<NodeId, (Expr, usize)>,
    ) -> Result<Expr, NumericsError> {
        let value = |id: NodeId| {
            native
                .get(&id)
                .map(|(expr, _)| expr.clone())
                .ok_or_else(|| input("scalar operand absent"))
        };
        let arg = |position: usize| {
            node.children
                .get(position)
                .copied()
                .ok_or_else(|| input("scalar argument absent"))
                .and_then(value)
        };
        let result = match (&node.opcode, &node.payload) {
            (Opcode::Const, Payload::FloatConst { value, .. }) => lit(*value),
            (Opcode::Const, Payload::IntConst { value }) => integer(*value)?,
            (
                Opcode::SymbolRef,
                Payload::SymbolRef {
                    symbol: ValueRef::ActualSymbol(symbol),
                },
            ) => self.symbols.get(symbol).cloned().ok_or_else(|| {
                input(format!("scalar symbol {symbol} has no actual case binding"))
            })?,
            (Opcode::Add, _) => arg(0)? + arg(1)?,
            (Opcode::Sub, _) => arg(0)? - arg(1)?,
            (Opcode::Mul, _) => arg(0)? * arg(1)?,
            (Opcode::Div, _) => arg(0)? / arg(1)?,
            (Opcode::Pow, _) => math::power(arg(0)?, arg(1)?),
            (Opcode::Neg, _) => -arg(0)?,
            (
                Opcode::Affine,
                Payload::Affine {
                    constant, terms, ..
                },
            ) => {
                let mut output = lit(*constant);
                for term in terms {
                    output = output + lit(term.coefficient) * value(term.child)?;
                }
                output
            }
            (
                Opcode::WeightedMean,
                Payload::WeightedMean {
                    pairs,
                    normalization: WeightNormalization::DivideBySum,
                    ..
                },
            ) => weighted_mean(pairs, &value)?,
            (Opcode::UnitConvert, Payload::UnitConvert(conversion)) => {
                arg(0)? * lit(conversion.scale) + lit(conversion.offset)
            }
            (Opcode::Conditional, Payload::Conditional { guard }) => {
                let guard = guard.require_math(id)?;
                let test = match &self.graph.node(guard)?.payload {
                    Payload::IntConst { value: 0 } => lit(false),
                    Payload::IntConst { value: 1 } => lit(true),
                    _ => value(guard)?,
                };
                when(test, arg(0)?)
                    .otherwise(arg(1)?)
                    .map_err(NumericsError::preparation)?
            }
            (Opcode::SmoothAbs, Payload::SmoothOp { eps }) => {
                math::sqrt(arg(0)? * arg(0)? + lit(eps * eps))
            }
            (Opcode::SmoothMax | Opcode::SmoothMin, Payload::SmoothOp { eps }) => {
                let distance = arg(0)? - arg(1)?;
                let length = math::sqrt(distance.clone() * distance + lit(eps * eps));
                let sum = arg(0)? + arg(1)?;
                lit(0.5_f64)
                    * if node.opcode == Opcode::SmoothMax {
                        sum + length
                    } else {
                        sum - length
                    }
            }
            (opcode, Payload::None) => unary(*opcode, arg(0)?)?,
            _ => {
                return Err(unsupported(format!(
                    "{} requires its declared lowering or native kernel binding",
                    node.opcode
                )));
            }
        };
        Ok(result)
    }
}
fn integer(value: i64) -> Result<Expr, NumericsError> {
    if !(-(1i64 << 53)..=(1i64 << 53)).contains(&value) {
        return Err(unsupported(
            "integer literal is not exactly representable as Float64",
        ));
    }
    Ok(lit(ScalarValue::Int64(Some(value))
        .cast_to(&DataType::Float64)
        .map_err(NumericsError::preparation)?))
}
fn unary(opcode: Opcode, value: Expr) -> Result<Expr, NumericsError> {
    Ok(match opcode {
        Opcode::Abs => math::abs(value),
        Opcode::Exp => math::exp(value),
        Opcode::Log => math::ln(value),
        Opcode::Log10 => math::log10(value),
        Opcode::Sqrt => math::sqrt(value),
        Opcode::Sin => math::sin(value),
        Opcode::Cos => math::cos(value),
        Opcode::Tan => math::tan(value),
        Opcode::Asin => math::asin(value),
        Opcode::Acos => math::acos(value),
        Opcode::Atan => math::atan(value),
        Opcode::Sinh => math::sinh(value),
        Opcode::Cosh => math::cosh(value),
        Opcode::Tanh => math::tanh(value),
        _ => return Err(unsupported(format!("native scalar binding for {opcode}"))),
    })
}

fn weighted_mean(
    pairs: &[pse_mathir::WeightedPair],
    value: &impl Fn(NodeId) -> Result<Expr, NumericsError>,
) -> Result<Expr, NumericsError> {
    let mut numerator = None;
    let mut denominator = None;
    for pair in pairs {
        let weight = value(pair.weight)?;
        let term = weight.clone() * value(pair.value)?;
        numerator = Some(match numerator {
            Some(prior) => prior + term,
            None => term,
        });
        denominator = Some(match denominator {
            Some(prior) => prior + weight,
            None => weight,
        });
    }
    Ok(numerator.ok_or_else(|| input("empty weighted mean"))?
        / denominator.ok_or_else(|| input("empty weight sum"))?)
}
