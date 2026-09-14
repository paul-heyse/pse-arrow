// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Uncached physical canonicalization (blueprint §7.4, §8.3).
use crate::equation::{EquationRecord, Sense};
use crate::infer::{KernelBindings, SymbolTypeSource};
use crate::relations::vec_sink::QuantitySelection;
use crate::{CanonicalGraph, ExprGraph, MathIrError, Node, NodeId, Opcode, Payload};
use pse_ids::SemanticId;
use pse_quantity::infer::{
    BuiltInRule, Exponent, Inferred, OpRequest, Operand, OperationSelection,
};
use pse_quantity::literal::LiteralContext;
use pse_quantity::{
    BoundIndexRef, ConversionId, ConversionKind, IndexSet, QuantityRegistry, QuantityTypeId, Ratio,
    UnitConvertSpec, UnitId,
};
use std::collections::{BTreeMap, BTreeSet};

/// The only package-graph numerical policy: authored order and guarded failures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Policy {
    /// Preserve argument order, signed zero and evaluation regions.
    Strict,
}
static NO_BINDINGS: KernelBindings = BTreeMap::new();
/// Complete predecessor inputs; selection rows are claims to revalidate, never proof.
#[derive(Clone, Copy)]
pub struct CanonicalizeInput<'a> {
    /// Source graph.
    pub graph: &'a ExprGraph,
    /// Indexed equations and their bound/filter references.
    pub equations: &'a [EquationRecord],
    /// Additional expression roots in caller-defined order.
    pub roots: &'a [NodeId],
    /// Actual symbol/domain/group/kernel declarations and scoped invariant checker.
    pub symbols: &'a dyn SymbolTypeSource,
    /// Admitted quantity declarations.
    pub registry: &'a QuantityRegistry,
    /// Complete kernel binding relation rows.
    pub kernel_bindings: &'a KernelBindings,
    /// Predecessor selection claims, needed for explicit physical conversion edges.
    pub selections: &'a [QuantitySelection],
}
impl std::fmt::Debug for CanonicalizeInput<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CanonicalizeInput")
            .field("nodes", &self.graph.len())
            .field("roots", &self.roots)
            .field("equations", &self.equations.len())
            .finish_non_exhaustive()
    }
}
impl<'a> CanonicalizeInput<'a> {
    /// A scalar-graph input with explicit roots and no equations/bindings/selection claims.
    pub fn new(
        graph: &'a ExprGraph,
        roots: &'a [NodeId],
        symbols: &'a dyn SymbolTypeSource,
        registry: &'a QuantityRegistry,
    ) -> Self {
        Self {
            graph,
            roots,
            symbols,
            registry,
            equations: &[],
            kernel_bindings: &NO_BINDINGS,
            selections: &[],
        }
    }
}
/// Infer every reachable occurrence before typed sharing, then fold under execution guards
/// and number exact typed structures. Recomputes semantic facts on every call.
///
/// # Errors
/// Rejects invalid source facts, missing/ambiguous quantity rules, incompatible complete
/// types, unbound indices, malformed references and unconditional static domain failures.
pub fn canonicalize(
    input: CanonicalizeInput<'_>,
    _policy: Policy,
) -> Result<CanonicalGraph, MathIrError> {
    let all: Vec<_> = input.graph.iter().map(|(id, _)| id).collect();
    crate::topo::postorder_with_bindings(input.graph, &all, input.kernel_bindings)?;
    for (id, node) in input.graph.iter() {
        if let Payload::SymbolRef { symbol } = &node.payload {
            symbol.require_symbol(id)?;
        }
        if let Some(guard) = node.payload.guard() {
            guard.require_math(id)?;
        }
        if let Some(domain) = node.payload.domain() {
            domain.require_actual(id)?;
        }
    }
    let mut driver = Driver::new(&input)?;
    let mut roots = Vec::new();
    for root in input.roots {
        roots.push(driver.visit(&Request::root(*root))?.node);
    }
    let mut equations = Vec::new();
    for equation in input.equations {
        let typed = driver.equation(equation)?;
        roots.extend(typed.referenced_nodes());
        equations.push(typed);
    }
    let fold = crate::fold::fold_literals_with_bindings(
        &mut driver.graph,
        &roots,
        input.registry,
        &driver.bindings,
    )?;
    for node in fold.deferred_static_checks {
        if let Some(selection) = driver.selections.get_mut(&node) {
            selection.deferred_static_check = true;
        }
    }
    let indices = driver
        .values
        .iter()
        .map(|(id, value)| (*id, value.indices.clone()))
        .collect();
    let mut output = crate::canonical::number_typed_graph_with_bindings(
        &driver.graph,
        &roots,
        &indices,
        input.registry,
        &driver.bindings,
    )?;
    let map = |id: NodeId| {
        output
            .node_mapping()
            .get(&id)
            .copied()
            .ok_or_else(|| MathIrError::malformed_at(id, "typed node was not retained"))
    };
    for equation in &mut equations {
        equation.map_node_references(map)?;
    }
    let mut selected = BTreeMap::new();
    for (_, mut selection) in driver.selections {
        let Some(node) = output.node_mapping().get(&selection.node).copied() else {
            continue;
        };
        selection.node = node;
        // Folding replaces the selected operation with a literal value.
        if output.node(node)?.opcode == Opcode::Const {
            selection.operation = None;
            selection.builtin = Some(BuiltInRule::Literal);
            selection.permutation.clear();
            selection.conversions.clear();
        }
        if let Some(previous) = selected.insert(node, selection.clone())
            && previous != selection
        {
            return Err(MathIrError::malformed_at(
                node,
                "exactly shared expressions have different physical derivations",
            ));
        }
    }
    output.roots.truncate(input.roots.len());
    output.equations = equations;
    output.selections = selected.into_values().collect();
    Ok(output)
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Request {
    node: NodeId,
    expected: Option<QuantityTypeId>,
    environment: IndexSet,
    representation: Option<UnitId>,
    preserve_representation: bool,
    guarded: bool,
}
impl Request {
    fn root(node: NodeId) -> Self {
        Self {
            node,
            expected: None,
            environment: IndexSet::new(),
            representation: None,
            preserve_representation: false,
            guarded: false,
        }
    }
}
#[derive(Clone, Debug)]
struct Value {
    node: NodeId,
    quantity_type: QuantityTypeId,
    indices: IndexSet,
    unit: UnitId,
}
struct Frame {
    request: Request,
    node: Node,
    dependencies: Vec<Request>,
    order: Vec<usize>,
    next: usize,
    values: Vec<Option<Value>>,
}
struct Driver<'a, 'b> {
    input: &'b CanonicalizeInput<'a>,
    graph: ExprGraph,
    resolved: BTreeMap<Request, Value>,
    values: BTreeMap<NodeId, Value>,
    selections: BTreeMap<NodeId, QuantitySelection>,
    claimed: BTreeMap<NodeId, &'a QuantitySelection>,
    bindings: KernelBindings,
}
impl<'a, 'b> Driver<'a, 'b> {
    fn new(input: &'b CanonicalizeInput<'a>) -> Result<Self, MathIrError> {
        let mut claimed = BTreeMap::new();
        for selection in input.selections {
            input.graph.node(selection.node)?;
            if claimed.insert(selection.node, selection).is_some() {
                return Err(MathIrError::malformed_at(
                    selection.node,
                    "duplicate predecessor selection",
                ));
            }
        }
        Ok(Self {
            input,
            graph: ExprGraph::new(),
            resolved: BTreeMap::new(),
            values: BTreeMap::new(),
            selections: BTreeMap::new(),
            claimed,
            bindings: KernelBindings::new(),
        })
    }
    fn ty(
        &self,
        node: NodeId,
        ty: QuantityTypeId,
    ) -> Result<&pse_quantity::QuantityType, MathIrError> {
        q(node, self.input.registry.quantity_type(ty))
    }
    fn domain(
        &self,
        node: NodeId,
        id: pse_quantity::DomainId,
    ) -> Result<&crate::index::DomainFacts, MathIrError> {
        let domain = self
            .input
            .symbols
            .domain(id)
            .ok_or(MathIrError::UnknownBinding {
                node,
                binding: id.as_id(),
            })?;
        let mut members = BTreeSet::new();
        if domain.members.iter().any(|id| !members.insert(*id)) {
            return Err(MathIrError::malformed_at(
                node,
                "duplicate domain member identity",
            ));
        }
        Ok(domain)
    }
    fn visit(&mut self, request: &Request) -> Result<Value, MathIrError> {
        if let Some(value) = self.resolved.get(request) {
            return Ok(value.clone());
        }
        let mut stack = vec![self.frame(request.clone())?];
        while let Some(mut frame) = stack.pop() {
            if frame.next == frame.order.len() {
                let key = frame.request.clone();
                let value = self.finish(frame)?;
                self.resolved.insert(key, value);
                continue;
            }
            let slot = frame.order[frame.next];
            let mut dependency = frame.dependencies[slot].clone();
            if dependency.expected.is_none() {
                dependency.expected = Self::sibling_expected(&frame, slot);
            }
            if let Some(value) = self.resolved.get(&dependency).cloned() {
                frame.values[slot] = Some(value);
                frame.next += 1;
                stack.push(frame);
            } else {
                stack.push(frame);
                stack.push(self.frame(dependency)?);
            }
        }
        self.resolved.get(request).cloned().ok_or_else(|| {
            MathIrError::malformed_at(request.node, "occurrence inference produced no value")
        })
    }
    fn sibling_expected(frame: &Frame, slot: usize) -> Option<QuantityTypeId> {
        if matches!(
            frame.node.opcode,
            Opcode::Add | Opcode::Sub | Opcode::Conditional | Opcode::SmoothMax | Opcode::SmoothMin
        ) && slot < 2
        {
            frame
                .values
                .get(1 - slot)
                .and_then(Option::as_ref)
                .map(|v| v.quantity_type)
                .or(frame.request.expected)
        } else if matches!(
            frame.node.opcode,
            Opcode::Neg | Opcode::Abs | Opcode::SmoothAbs
        ) {
            frame.request.expected
        } else if frame.node.opcode == Opcode::WeightedMean && slot % 2 == 1 {
            frame
                .values
                .iter()
                .enumerate()
                .find_map(|(i, v)| {
                    if i % 2 == 1 {
                        v.as_ref().map(|v| v.quantity_type)
                    } else {
                        None
                    }
                })
                .or(frame.request.expected)
        } else {
            None
        }
    }
    fn frame(&self, request: Request) -> Result<Frame, MathIrError> {
        let node = self.input.graph.node(request.node)?.clone();
        let mut dependencies: Vec<_> = node
            .children
            .iter()
            .copied()
            .chain(node.payload.referenced_nodes())
            .map(|node| Request {
                node,
                expected: None,
                environment: request.environment.clone(),
                representation: None,
                preserve_representation: false,
                guarded: request.guarded,
            })
            .collect();
        if node.opcode == Opcode::Conditional {
            for dependency in dependencies.iter_mut().take(node.children.len()) {
                dependency.guarded = true;
            }
        }
        match &node.payload {
            Payload::Reduction {
                domain,
                bound_index,
                ..
            }
            | Payload::Integral {
                domain,
                bound_index,
                ..
            } => {
                let domain = domain.require_actual(request.node)?;
                let facts = self.domain(request.node, domain)?;
                let bound = crate::index::bind(
                    request.node,
                    &request.environment,
                    *bound_index,
                    domain,
                    facts,
                )?;
                for dependency in &mut dependencies {
                    dependency.environment = bound.clone();
                }
                if let Some(body) = dependencies.first_mut() {
                    body.guarded = true;
                }
            }
            Payload::UnitConvert(_) | Payload::PendingUnitConvert { .. } => {
                if let Some(child) = dependencies.first_mut() {
                    child.preserve_representation = true;
                    if let Some(conversion) = self.claimed_conversion(request.node)? {
                        child.expected =
                            Some(q(request.node, self.input.registry.conversion(conversion))?.from);
                    } else {
                        child.expected = node.quantity_type.or(request.expected);
                    }
                }
            }
            Payload::PiecewiseLinear { input, .. } => {
                if let Some(child) = dependencies.first_mut() {
                    child.expected = Some(*input);
                }
            }
            Payload::KernelCall { kernel_binding, .. } => {
                self.kernel_dependencies(&request, *kernel_binding, &mut dependencies)?;
            }
            _ => {}
        }
        let mut order: Vec<_> = (0..dependencies.len()).collect();
        // Resolve nonliteral siblings first only for typing; authored argument order never changes.
        if matches!(
            node.opcode,
            Opcode::Add
                | Opcode::Sub
                | Opcode::Conditional
                | Opcode::WeightedMean
                | Opcode::SmoothMax
                | Opcode::SmoothMin
        ) {
            order.sort_by_key(|slot| {
                matches!(
                    self.input
                        .graph
                        .node(dependencies[*slot].node)
                        .map(|n| &n.payload),
                    Ok(Payload::IntConst { .. } | Payload::FloatConst { .. })
                )
            });
        }
        let values = vec![None; dependencies.len()];
        Ok(Frame {
            request,
            node,
            dependencies,
            order,
            next: 0,
            values,
        })
    }
    fn kernel_dependencies(
        &self,
        request: &Request,
        kernel_binding: SemanticId,
        dependencies: &mut Vec<Request>,
    ) -> Result<(), MathIrError> {
        let binding =
            self.input
                .kernel_bindings
                .get(&kernel_binding)
                .ok_or(MathIrError::UnknownBinding {
                    node: request.node,
                    binding: kernel_binding,
                })?;
        let contract = self.input.symbols.kernel_contract(binding.kernel).ok_or(
            MathIrError::UnknownBinding {
                node: request.node,
                binding: binding.kernel,
            },
        )?;
        if contract.inputs.len() != binding.inputs.len() {
            return Err(MathIrError::malformed_at(
                request.node,
                "kernel input count disagrees with specification",
            ));
        }
        for ((name, input), port) in binding.inputs.iter().zip(&contract.inputs) {
            if name != &port.name {
                return Err(MathIrError::malformed_at(
                    request.node,
                    "kernel input name/order disagrees with specification",
                ));
            }
            dependencies.push(Request {
                node: *input,
                expected: Some(port.quantity_type),
                environment: request.environment.clone(),
                representation: Some(port.unit),
                preserve_representation: false,
                guarded: request.guarded,
            });
        }
        Ok(())
    }
    fn claimed_conversion(&self, node: NodeId) -> Result<Option<ConversionId>, MathIrError> {
        let Some(selection) = self.claimed.get(&node) else {
            return Ok(None);
        };
        if selection.conversions.is_empty() {
            return Ok(None);
        }
        if selection.operation.is_some()
            || selection.builtin != Some(BuiltInRule::UnitConvert)
            || !selection.permutation.is_empty()
        {
            return Err(MathIrError::malformed_at(
                node,
                "physical unit edge requires the built-in conversion selection",
            ));
        }
        match selection.conversions.as_slice() {
            [(0, id)] => Ok(Some(*id)),
            _ => Err(MathIrError::malformed_at(
                node,
                "physical unit edge requires exactly one named operand-zero conversion",
            )),
        }
    }
    fn resolve_pending_payload(
        &self,
        old: NodeId,
        payload: &Payload,
        values: &[Value],
    ) -> Result<Option<Payload>, MathIrError> {
        if let Payload::PendingSmoothOp { eps, unit } = payload {
            let value = values
                .first()
                .ok_or_else(|| MathIrError::malformed_at(old, "missing smoothing operand"))?;
            let original = self.input.graph.node(old)?;
            if value.unit != self.ty(old, value.quantity_type)?.canonical_unit {
                return Err(MathIrError::malformed_at(
                    old,
                    "smoothing operand is not in its canonical coordinate",
                ));
            }
            let eps = q(
                old,
                pse_quantity::smoothing::resolve_epsilon(
                    original.opcode,
                    value.quantity_type,
                    *eps,
                    Some(*unit),
                    self.input.registry,
                ),
            )?;
            return Ok(Some(Payload::SmoothOp { eps }));
        }
        let Payload::PendingUnitConvert { to } = payload else {
            return Ok(None);
        };
        if self.claimed_conversion(old)?.is_some() {
            return Err(MathIrError::malformed_at(
                old,
                "pending representation conversion cannot assert a physical conversion selection",
            ));
        }
        let value = values
            .first()
            .ok_or_else(|| MathIrError::malformed_at(old, "missing conversion operand"))?;
        Ok(Some(Payload::UnitConvert(
            self.representation_spec(old, value, *to)?,
        )))
    }
    fn finish(&mut self, frame: Frame) -> Result<Value, MathIrError> {
        let old = frame.request.node;
        let values = frame
            .values
            .into_iter()
            .map(|v| v.ok_or_else(|| MathIrError::malformed_at(old, "missing inferred dependency")))
            .collect::<Result<Vec<_>, _>>()?;
        let mut payload = frame.node.payload.clone();
        let mut children: Vec<_> = values
            .iter()
            .take(frame.node.children.len())
            .map(|v| v.node)
            .collect();
        let mut refs = values.iter().skip(frame.node.children.len());
        payload.map_node_references(|_| {
            refs.next()
                .map(|v| v.node)
                .ok_or_else(|| MathIrError::malformed_at(old, "missing typed payload reference"))
        })?;
        let mut resolved = frame.node.clone();
        if let Some(converted) = self.resolve_pending_payload(old, &payload, &values)? {
            payload = converted;
            resolved.payload = payload.clone();
        }
        let (inferred, unit) = self.infer_node(&frame.request, &resolved, &values)?;
        if let Some(claimed) = frame.node.quantity_type {
            q(
                old,
                pse_quantity::admission::require_same_contract(
                    claimed,
                    inferred.result,
                    self.input.registry,
                ),
            )?;
        }
        if !matches!(
            payload,
            Payload::UnitConvert(_) | Payload::KernelCall { .. }
        ) {
            for conversion in &inferred.conversions {
                let position = usize::from(conversion.operand);
                let value = values.get(position).ok_or_else(|| {
                    MathIrError::malformed_at(old, "conversion references absent operand")
                })?;
                let converted = if self.is_explicit_conversion(value.node, conversion.conversion) {
                    value.clone()
                } else {
                    let source_operand =
                        frame.node.children.get(position).copied().ok_or_else(|| {
                            MathIrError::malformed_at(
                                old,
                                "conversion operand is not an expression argument",
                            )
                        })?;
                    if q(old, self.input.registry.conversion(conversion.conversion))?.kind
                        == ConversionKind::Kernel
                    {
                        self.convert_kernel(
                            old,
                            conversion.operand,
                            source_operand,
                            value,
                            conversion.conversion,
                            frame.node.scope,
                        )?
                    } else {
                        self.convert_physical(old, value, conversion.conversion, frame.node.scope)?
                    }
                };
                let child = children.get_mut(position).ok_or_else(|| {
                    MathIrError::malformed_at(
                        old,
                        "payload conversion requires explicit predecessor edge",
                    )
                })?;
                *child = converted.node;
            }
        }
        let id = self.graph.insert_typed(
            frame.node.opcode,
            payload,
            &children,
            inferred.result,
            frame.node.scope,
        )?;
        self.record(id, &inferred)?;
        let value = Value {
            node: id,
            quantity_type: inferred.result,
            indices: inferred.indices,
            unit,
        };
        self.values.insert(id, value.clone());
        if frame.request.preserve_representation {
            return Ok(value);
        }
        let desired = frame
            .request
            .representation
            .unwrap_or(self.ty(old, value.quantity_type)?.canonical_unit);
        self.convert_representation(old, value, desired, frame.node.scope)
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one exhaustive node-to-quantity-request dispatch"
    )]
    fn infer_node(
        &mut self,
        request: &Request,
        node: &Node,
        values: &[Value],
    ) -> Result<(Inferred, UnitId), MathIrError> {
        let id = request.node;
        let logical_values = self.logical_operands(id, values)?;
        let operands: Vec<_> = logical_values
            .iter()
            .take(node.children.len())
            .map(|v| Operand {
                quantity_type: v.quantity_type,
                indices: &v.indices,
            })
            .collect();
        let infer = |op: &OpRequest<'_>, args: &[Operand<'_>]| {
            q(
                id,
                pse_quantity::infer::infer_with_evidence(
                    op,
                    args,
                    self.input.registry,
                    self.input.symbols.invariant_checker(id),
                ),
            )
        };
        let result = match &node.payload {
            Payload::FloatConst { unit, .. } => {
                let expected = node.quantity_type.or(request.expected);
                let context = expected.map_or(LiteralContext::Free, |quantity_type| {
                    LiteralContext::Explicit { quantity_type }
                });
                let result = infer(
                    &OpRequest::Literal {
                        unit: *unit,
                        context,
                    },
                    &[],
                )?;
                return Ok((result, *unit));
            }
            Payload::IntConst { .. } => {
                let quantity = node
                    .quantity_type
                    .or(request.expected)
                    .or(self.input.registry.neutral_dimensionless())
                    .ok_or_else(|| {
                        MathIrError::malformed_at(
                            id,
                            "integer literal has no explicit neutral or expected contract",
                        )
                    })?;
                let unit = self.ty(id, quantity)?.canonical_unit;
                let result = infer(
                    &OpRequest::Literal {
                        unit,
                        context: LiteralContext::Explicit {
                            quantity_type: quantity,
                        },
                    },
                    &[],
                )?;
                return Ok((result, unit));
            }
            Payload::SymbolRef { symbol } => {
                let symbol = symbol.require_symbol(id)?;
                let quantity =
                    self.input
                        .symbols
                        .symbol_type(symbol)
                        .ok_or(MathIrError::UnknownBinding {
                            node: id,
                            binding: symbol,
                        })?;
                let ty = self.ty(id, quantity)?;
                if !ty.key.shape.is_empty() {
                    return Err(MathIrError::malformed_at(
                        id,
                        "scalar symbol reference carries an indexed type; use Gather",
                    ));
                }
                let unit = self
                    .input
                    .symbols
                    .symbol_unit(symbol)
                    .unwrap_or(ty.canonical_unit);
                q(
                    id,
                    pse_quantity::convert_spec_for_type(
                        self.input
                            .registry
                            .unit(unit)
                            .map_err(|source| MathIrError::Quantity { node: id, source })?,
                        self.input
                            .registry
                            .unit(ty.canonical_unit)
                            .map_err(|source| MathIrError::Quantity { node: id, source })?,
                        &ty.key,
                    ),
                )?;
                return Ok((
                    builtin(quantity, IndexSet::new(), BuiltInRule::Binding),
                    unit,
                ));
            }
            Payload::Affine {
                constant_quantity_type,
                constant_unit,
                terms,
                ..
            } => {
                let signs: Vec<_> = terms
                    .iter()
                    .map(|term| if term.coefficient < 0.0 { -1 } else { 1 })
                    .collect();
                if let (Some(quantity), Some(unit)) = (constant_quantity_type, constant_unit) {
                    let ty = self.ty(id, *quantity)?;
                    if ty.canonical_unit != *unit || !ty.key.shape.is_empty() {
                        return Err(MathIrError::malformed_at(
                            id,
                            "Affine constant must declare a scalar complete type in its canonical unit",
                        ));
                    }
                    let scalar = IndexSet::new();
                    let mut with_constant = vec![Operand {
                        quantity_type: *quantity,
                        indices: &scalar,
                    }];
                    with_constant.extend(operands.iter().copied());
                    infer(
                        &OpRequest::Affine {
                            term_signs: &signs,
                            has_constant: true,
                        },
                        &with_constant,
                    )?
                } else {
                    infer(
                        &OpRequest::Affine {
                            term_signs: &signs,
                            has_constant: false,
                        },
                        &operands,
                    )?
                }
            }
            Payload::WeightedMean {
                normalization,
                unit_sum_invariant,
                ..
            } => {
                let pairs: Vec<_> = values
                    .iter()
                    .map(|v| Operand {
                        quantity_type: v.quantity_type,
                        indices: &v.indices,
                    })
                    .collect();
                infer(
                    &OpRequest::WeightedMean {
                        normalization: *normalization,
                        certified_invariant: *unit_sum_invariant,
                    },
                    &pairs,
                )?
            }
            Payload::Reduction {
                kind,
                domain,
                bound_index,
                filter,
            } => {
                let domain = domain.require_actual(id)?;
                let facts = self.domain(id, domain)?;
                if !request.guarded
                    && facts.members.is_empty()
                    && matches!(
                        kind,
                        pse_quantity::ReductionKind::Min | pse_quantity::ReductionKind::Max
                    )
                {
                    return Err(MathIrError::StaticDomain {
                        node: id,
                        opcode: node.opcode,
                        restriction: "a nonempty reduction domain",
                        value: 0.0,
                    });
                }
                if filter.is_some() {
                    self.guard(values.last().ok_or_else(|| {
                        MathIrError::malformed_at(id, "missing reduction filter")
                    })?)?;
                }
                infer(
                    &OpRequest::Reduce {
                        kind: *kind,
                        bound: BoundIndexRef::new(*bound_index, domain, facts.kind),
                    },
                    &operands,
                )?
            }
            Payload::Gather {
                group,
                coordinate_map,
            } => {
                let (quantity, coordinates) =
                    self.gather(id, *group, coordinate_map, &request.environment)?;
                infer(
                    &OpRequest::Gather {
                        group_type: quantity,
                        coordinates: &coordinates,
                    },
                    &[],
                )?
            }
            Payload::Broadcast {
                domain,
                bound_index,
            } => {
                let domain = domain.require_actual(id)?;
                let facts = self.domain(id, domain)?;
                if request
                    .environment
                    .get(*bound_index)
                    .is_some_and(|bound| bound.domain != domain || bound.kind != facts.kind)
                {
                    return Err(MathIrError::malformed_at(
                        id,
                        "broadcast binder disagrees with lexical domain",
                    ));
                }
                infer(
                    &OpRequest::Broadcast {
                        index: BoundIndexRef::new(*bound_index, domain, facts.kind),
                    },
                    &operands,
                )?
            }
            Payload::Derivative { wrt_domain, order } => {
                let facts = self.continuous(id, wrt_domain.require_actual(id)?)?;
                infer(
                    &OpRequest::Derivative {
                        domain_unit: facts.0,
                        domain_kind: facts.1,
                        order: *order,
                    },
                    &operands,
                )?
            }
            Payload::Integral {
                domain,
                bound_index,
                filter,
                ..
            } => {
                if filter.is_some() {
                    self.guard(values.last().ok_or_else(|| {
                        MathIrError::malformed_at(id, "missing integral filter")
                    })?)?;
                }
                let facts = self.continuous(id, domain.require_actual(id)?)?;
                infer(
                    &OpRequest::Integral {
                        domain_unit: facts.0,
                        bound: BoundIndexRef::new(
                            *bound_index,
                            domain.require_actual(id)?,
                            facts.1,
                        ),
                    },
                    &operands,
                )?
            }
            Payload::PendingSmoothOp { .. } => {
                return Err(MathIrError::malformed_at(
                    id,
                    "unresolved smoothing tolerance reached physical inference",
                ));
            }
            Payload::SmoothOp { eps } => infer(
                &OpRequest::Smooth {
                    opcode: node.opcode,
                    eps: *eps,
                },
                &operands,
            )?,
            Payload::Conditional { .. } => {
                self.guard(
                    values
                        .last()
                        .ok_or_else(|| MathIrError::malformed_at(id, "missing guard"))?,
                )?;
                infer(&OpRequest::Conditional, &operands)?
            }
            Payload::KernelCall {
                kernel_binding,
                output_ordinal,
            } => return self.kernel(id, *kernel_binding, *output_ordinal, values),
            Payload::ImplicitRef {
                implicit_system,
                unknown_ordinal,
            } => {
                let unknown = self
                    .input
                    .symbols
                    .implicit_unknown(*implicit_system, *unknown_ordinal)
                    .ok_or(MathIrError::UnknownBinding {
                        node: id,
                        binding: *implicit_system,
                    })?;
                infer(&OpRequest::ImplicitRef { unknown }, &[])?
            }
            Payload::PendingUnitConvert { .. } => {
                return Err(MathIrError::malformed_at(
                    id,
                    "pending conversion must be resolved after child typing",
                ));
            }
            Payload::PendingGather { .. } => {
                return Err(MathIrError::malformed_at(
                    id,
                    "pending indexed read requires actual instance/domain lowering before P10",
                ));
            }
            Payload::UnitConvert(spec) => {
                let value = values
                    .first()
                    .ok_or_else(|| MathIrError::malformed_at(id, "missing conversion operand"))?;
                if value.unit != spec.from {
                    return Err(MathIrError::NonCanonicalUnitEscapes {
                        node: id,
                        unit: value.unit,
                    });
                }
                if let Some(conversion) = self.claimed_conversion(id)? {
                    let (quantity, expected) = self.physical_spec(id, value, conversion)?;
                    if !spec.same_contract(&expected) {
                        return Err(MathIrError::malformed_at(
                            id,
                            "physical conversion coefficients disagree with named rule",
                        ));
                    }
                    let mut selected =
                        builtin(quantity, value.indices.clone(), BuiltInRule::UnitConvert);
                    selected
                        .conversions
                        .push(pse_quantity::infer::OperandConversion {
                            operand: 0,
                            conversion,
                        });
                    return Ok((selected, spec.to));
                }
                let expected = self.representation_spec(id, value, spec.to)?;
                if !spec.same_contract(&expected) {
                    return Err(MathIrError::malformed_at(
                        id,
                        "representation conversion disagrees with actual units or scale kind",
                    ));
                }
                return Ok((
                    builtin(
                        value.quantity_type,
                        value.indices.clone(),
                        BuiltInRule::UnitConvert,
                    ),
                    spec.to,
                ));
            }
            Payload::PiecewiseLinear { input, output, .. } => infer(
                &OpRequest::PiecewiseLinear {
                    input: *input,
                    output: *output,
                },
                &operands,
            )?,
            Payload::None => {
                let op = match node.opcode {
                    Opcode::Add => OpRequest::Add,
                    Opcode::Sub => OpRequest::Sub,
                    Opcode::Mul => OpRequest::Mul,
                    Opcode::Div => OpRequest::Div,
                    Opcode::Neg => OpRequest::Neg,
                    Opcode::Abs => OpRequest::Abs,
                    Opcode::Sqrt => OpRequest::Sqrt,
                    Opcode::Pow => OpRequest::Pow {
                        exponent: self.exponent(id, values.get(1))?,
                    },
                    Opcode::Exp
                    | Opcode::Log
                    | Opcode::Log10
                    | Opcode::Sin
                    | Opcode::Cos
                    | Opcode::Tan
                    | Opcode::Asin
                    | Opcode::Acos
                    | Opcode::Atan
                    | Opcode::Sinh
                    | Opcode::Cosh
                    | Opcode::Tanh
                    | Opcode::Erf => OpRequest::Transcendental(node.opcode),
                    _ => {
                        return Err(MathIrError::malformed_at(
                            id,
                            "operator lacks its required payload",
                        ));
                    }
                };
                infer(&op, &operands)?
            }
        };
        let unit = self.ty(id, result.result)?.canonical_unit;
        Ok((result, unit))
    }
    fn explicit_conversion_input(
        &self,
        mut node: NodeId,
        conversion: ConversionId,
    ) -> Option<NodeId> {
        loop {
            let value = self.graph.node(node).ok()?;
            let selected = self.selections.get(&node)?;
            if selected.conversions.as_slice() == [(0, conversion)] {
                return match &value.payload {
                    Payload::UnitConvert(_)
                        if selected.builtin == Some(BuiltInRule::UnitConvert) =>
                    {
                        value.children.first().copied()
                    }
                    Payload::KernelCall { kernel_binding, .. }
                        if selected.builtin == Some(BuiltInRule::Binding) =>
                    {
                        self.bindings
                            .get(kernel_binding)?
                            .inputs
                            .first()
                            .map(|(_, input)| *input)
                    }
                    _ => None,
                };
            }
            if matches!(value.payload, Payload::UnitConvert(_)) && selected.conversions.is_empty() {
                node = *value.children.first()?;
            } else {
                return None;
            }
        }
    }
    fn is_explicit_conversion(&self, node: NodeId, conversion: ConversionId) -> bool {
        self.explicit_conversion_input(node, conversion).is_some()
    }
    fn logical_operands(&self, id: NodeId, values: &[Value]) -> Result<Vec<Value>, MathIrError> {
        let mut logical = values.to_vec();
        let Some(claim) = self.claimed.get(&id) else {
            return Ok(logical);
        };
        if claim.operation.is_none() {
            return Ok(logical);
        }
        for (position, conversion) in &claim.conversions {
            let Some(value) = logical.get_mut(usize::from(*position)) else {
                return Err(MathIrError::malformed_at(
                    id,
                    "claimed operation conversion references absent operand",
                ));
            };
            if let Some(child) = self.explicit_conversion_input(value.node, *conversion) {
                *value = self.values.get(&child).cloned().ok_or_else(|| {
                    MathIrError::malformed_at(
                        child,
                        "physical conversion input has no established contract",
                    )
                })?;
            }
        }
        Ok(logical)
    }
    fn representation_spec(
        &self,
        id: NodeId,
        value: &Value,
        to: UnitId,
    ) -> Result<UnitConvertSpec, MathIrError> {
        let from = q(id, self.input.registry.unit(value.unit))?;
        let to = q(id, self.input.registry.unit(to))?;
        q(
            id,
            pse_quantity::convert_spec_for_type(from, to, &self.ty(id, value.quantity_type)?.key),
        )
    }
    fn physical_spec(
        &self,
        id: NodeId,
        value: &Value,
        conversion: ConversionId,
    ) -> Result<(QuantityTypeId, UnitConvertSpec), MathIrError> {
        let rule = q(id, self.input.registry.conversion(conversion))?;
        q(
            id,
            pse_quantity::admission::require_same_contract(
                rule.from,
                value.quantity_type,
                self.input.registry,
            ),
        )?;
        if rule.kind == ConversionKind::Kernel {
            return Err(MathIrError::malformed_at(
                id,
                "kernel quantity conversion requires a resolved KernelCall",
            ));
        }
        let from = self.ty(id, rule.from)?.canonical_unit;
        if value.unit != from {
            return Err(MathIrError::NonCanonicalUnitEscapes {
                node: value.node,
                unit: value.unit,
            });
        }
        let to = self.ty(id, rule.to)?.canonical_unit;
        let scale = rule
            .scale
            .ok_or_else(|| MathIrError::malformed_at(id, "physical scale coefficient absent"))?;
        let offset = rule.offset.unwrap_or(0.0);
        Ok((
            rule.to,
            UnitConvertSpec {
                from,
                to,
                scale,
                offset,
            },
        ))
    }
    fn check_kernel_conversion(
        &self,
        id: NodeId,
        binding: &crate::relations::vec_sink::KernelBinding,
        contract: &crate::infer::KernelContract,
        output: QuantityTypeId,
        values: &[Value],
        conversion: ConversionId,
    ) -> Result<(), MathIrError> {
        let rule = q(id, self.input.registry.conversion(conversion))?;
        if rule.kind != ConversionKind::Kernel
            || rule.kernel != Some(binding.kernel)
            || values.len() != 1
            || contract.inputs.len() != 1
        {
            return Err(MathIrError::malformed_at(
                id,
                "named conversion does not match this unary bound kernel",
            ));
        }
        q(
            id,
            pse_quantity::admission::require_same_contract(
                rule.from,
                values[0].quantity_type,
                self.input.registry,
            ),
        )?;
        q(
            id,
            pse_quantity::admission::require_same_contract(rule.to, output, self.input.registry),
        )?;
        for required in &rule.required_parameters {
            if !binding
                .parameters
                .iter()
                .any(|(name, _, _, _)| name == required)
            {
                return Err(MathIrError::malformed_at(
                    id,
                    "named conversion is missing its required parameter binding",
                ));
            }
        }
        Ok(())
    }
    fn convert_kernel(
        &mut self,
        id: NodeId,
        operand: u16,
        source_operand: NodeId,
        value: &Value,
        conversion: ConversionId,
        scope: Option<SemanticId>,
    ) -> Result<Value, MathIrError> {
        let binding_id = self
            .input
            .symbols
            .conversion_binding(id, operand, conversion)
            .ok_or_else(|| {
                MathIrError::malformed_at(
                    id,
                    "quantity kernel conversion has no explicit bound-kernel mapping",
                )
            })?;
        let binding =
            self.input
                .kernel_bindings
                .get(&binding_id)
                .ok_or(MathIrError::UnknownBinding {
                    node: id,
                    binding: binding_id,
                })?;
        if binding.inputs.len() != 1 || binding.inputs[0].1 != source_operand {
            return Err(MathIrError::malformed_at(
                id,
                "conversion binding input is not the actual source operand",
            ));
        }
        let contract = self.input.symbols.kernel_contract(binding.kernel).ok_or(
            MathIrError::UnknownBinding {
                node: id,
                binding: binding.kernel,
            },
        )?;
        let rule = q(id, self.input.registry.conversion(conversion))?;
        let outputs: Vec<_> = contract
            .outputs
            .iter()
            .enumerate()
            .filter(|(_, port)| port.quantity_type == rule.to)
            .collect();
        let [(ordinal, output)] = outputs.as_slice() else {
            return Err(MathIrError::malformed_at(
                id,
                "conversion requires exactly one declared output with its result type",
            ));
        };
        self.check_kernel_conversion(
            id,
            binding,
            contract,
            output.quantity_type,
            std::slice::from_ref(value),
            conversion,
        )?;
        let input = contract
            .inputs
            .first()
            .ok_or_else(|| MathIrError::malformed_at(id, "conversion kernel has no input"))?;
        let represented = self.convert_representation(id, value.clone(), input.unit, scope)?;
        let ordinal = u16::try_from(*ordinal)
            .map_err(|_| MathIrError::malformed_at(id, "kernel output ordinal exceeds u16"))?;
        let (mut inferred, unit) = self.kernel(id, binding_id, ordinal, &[represented])?;
        inferred
            .conversions
            .push(pse_quantity::infer::OperandConversion {
                operand: 0,
                conversion,
            });
        let node = self.graph.insert_typed(
            Opcode::KernelCall,
            Payload::KernelCall {
                kernel_binding: binding_id,
                output_ordinal: ordinal,
            },
            &[],
            inferred.result,
            scope,
        )?;
        self.record(node, &inferred)?;
        let value = Value {
            node,
            quantity_type: inferred.result,
            indices: inferred.indices,
            unit,
        };
        self.values.insert(node, value.clone());
        let canonical = self.ty(id, value.quantity_type)?.canonical_unit;
        self.convert_representation(id, value, canonical, scope)
    }
    fn convert_physical(
        &mut self,
        id: NodeId,
        value: &Value,
        conversion: ConversionId,
        scope: Option<SemanticId>,
    ) -> Result<Value, MathIrError> {
        let (quantity, spec) = self.physical_spec(id, value, conversion)?;
        let node = self.graph.insert_typed(
            Opcode::UnitConvert,
            Payload::UnitConvert(spec),
            &[value.node],
            quantity,
            scope,
        )?;
        let mut inferred = builtin(quantity, value.indices.clone(), BuiltInRule::UnitConvert);
        inferred
            .conversions
            .push(pse_quantity::infer::OperandConversion {
                operand: 0,
                conversion,
            });
        self.record(node, &inferred)?;
        let result = Value {
            node,
            quantity_type: quantity,
            indices: value.indices.clone(),
            unit: spec.to,
        };
        self.values.insert(node, result.clone());
        Ok(result)
    }
    fn convert_representation(
        &mut self,
        id: NodeId,
        value: Value,
        to: UnitId,
        scope: Option<SemanticId>,
    ) -> Result<Value, MathIrError> {
        if value.unit == to {
            return Ok(value);
        }
        let spec = self.representation_spec(id, &value, to)?;
        let node = self.graph.insert_typed(
            Opcode::UnitConvert,
            Payload::UnitConvert(spec),
            &[value.node],
            value.quantity_type,
            scope,
        )?;
        self.record(
            node,
            &builtin(
                value.quantity_type,
                value.indices.clone(),
                BuiltInRule::UnitConvert,
            ),
        )?;
        let result = Value {
            node,
            unit: to,
            ..value
        };
        self.values.insert(node, result.clone());
        Ok(result)
    }
    fn record(&mut self, node: NodeId, inferred: &Inferred) -> Result<(), MathIrError> {
        let (operation, builtin, permutation) = match &inferred.selected {
            OperationSelection::BuiltIn(rule) => (None, Some(*rule), vec![]),
            OperationSelection::Registered {
                operation,
                operand_permutation,
            } => (Some(*operation), None, operand_permutation.clone()),
        };
        let row = QuantitySelection {
            node,
            operation,
            builtin,
            permutation,
            conversions: inferred
                .conversions
                .iter()
                .map(|c| (c.operand, c.conversion))
                .collect(),
            deferred_static_check: false,
        };
        if let Some(existing) = self.selections.get(&node)
            && existing != &row
        {
            return Err(MathIrError::malformed_at(
                node,
                "shared typed node has inconsistent physical selections",
            ));
        }
        self.selections.insert(node, row);
        Ok(())
    }
    fn guard(&self, value: &Value) -> Result<(), MathIrError> {
        let node = self.graph.node(value.node)?;
        let ty = self.ty(value.node, value.quantity_type)?;
        let kind = q(value.node, self.input.registry.kind(ty.key.kind))?;
        if !kind.dimension.is_dimensionless()
            || kind.addition_kind != pse_quantity::QuantityAdditionKind::Additive
            || ty.key.basis.is_some()
            || ty.key.reference_state.is_some()
            || ty.key.subject_kind.is_some()
        {
            return Err(MathIrError::GuardNotBoolean { node: value.node });
        }
        let valid = match node.payload {
            Payload::IntConst { value: 0 | 1 } => true,
            Payload::SymbolRef { .. } => {
                self.input.symbols.boolean_kind()
                    == Some(self.ty(value.node, value.quantity_type)?.key.kind)
            }
            _ => false,
        };
        if valid && value.indices.is_empty() {
            Ok(())
        } else {
            Err(MathIrError::GuardNotBoolean { node: value.node })
        }
    }
    fn exponent(&self, id: NodeId, value: Option<&Value>) -> Result<Exponent, MathIrError> {
        let value = value.ok_or_else(|| MathIrError::malformed_at(id, "missing power exponent"))?;
        match self.graph.node(value.node)?.payload {
            Payload::IntConst { value } => {
                let Ok(value) = i16::try_from(value) else {
                    return Ok(Exponent::Symbolic);
                };
                let ratio = Ratio::from_parts(value, 1).map_err(|_| {
                    MathIrError::malformed_at(id, "integer exponent is not representable")
                })?;
                Ok(Exponent::Rational(ratio))
            }
            Payload::FloatConst { value, .. } => {
                // Only exact binary rationals within the registry ratio representation.
                for denominator in [
                    1i16, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384,
                ] {
                    let numerator = value * f64::from(denominator);
                    if let Some(integer) = pse_quantity::numeric::exact_i64_from_f64(numerator)
                        && let Ok(integer) = i16::try_from(integer)
                    {
                        let ratio = Ratio::new(i32::from(integer), i32::from(denominator))
                            .map_err(|_| MathIrError::malformed_at(id, "invalid exact exponent"))?;
                        return Ok(Exponent::Rational(ratio));
                    }
                }
                Ok(Exponent::Symbolic)
            }
            _ => Ok(Exponent::Symbolic),
        }
    }
    fn continuous(
        &self,
        id: NodeId,
        domain: pse_quantity::DomainId,
    ) -> Result<(UnitId, pse_quantity::DomainKind), MathIrError> {
        let facts = self.domain(id, domain)?;
        if !facts.continuous {
            return Err(MathIrError::malformed_at(
                id,
                "calculus requires an actual continuous domain",
            ));
        }
        let unit = facts.unit.ok_or_else(|| {
            MathIrError::malformed_at(id, "continuous domain has no coordinate unit")
        })?;
        q(id, self.input.registry.unit(unit))?;
        Ok((unit, facts.kind))
    }
    fn gather(
        &self,
        id: NodeId,
        group_id: SemanticId,
        coordinates: &[(pse_quantity::BoundIndexId, u16)],
        environment: &IndexSet,
    ) -> Result<(QuantityTypeId, Vec<BoundIndexRef>), MathIrError> {
        let group = self
            .input
            .symbols
            .group(group_id)
            .ok_or(MathIrError::UnknownBinding {
                node: id,
                binding: group_id,
            })?;
        if coordinates.len() != group.domains.len() {
            return Err(MathIrError::malformed_at(
                id,
                "gather coordinate count disagrees with product",
            ));
        }
        let mut ordered = vec![None; coordinates.len()];
        for (bound, position) in coordinates {
            let index = crate::index::resolve_binder(id, *bound, environment)?;
            let position = usize::from(*position);
            if group.domains.get(position) != Some(&index.domain)
                || ordered.get(position).is_none_or(Option::is_some)
            {
                return Err(MathIrError::malformed_at(
                    id,
                    "gather has a duplicate/incorrect coordinate axis",
                ));
            }
            ordered[position] = Some(index);
        }
        let mut valid = BTreeSet::new();
        for tuple in &group.valid_tuples {
            if tuple.len() != group.domains.len() || !valid.insert(tuple) {
                return Err(MathIrError::malformed_at(
                    id,
                    "invalid or duplicate group tuple",
                ));
            }
            for (member, domain) in tuple.iter().zip(&group.domains) {
                if !self.domain(id, *domain)?.members.contains(member) {
                    return Err(MathIrError::malformed_at(
                        id,
                        "group tuple contains an unknown domain member",
                    ));
                }
            }
            if !group.members.contains_key(tuple) {
                return Err(MathIrError::malformed_at(
                    id,
                    "valid tuple lacks its group symbol",
                ));
            }
        }
        if group.members.len() != valid.len() {
            return Err(MathIrError::malformed_at(
                id,
                "group member set differs from valid tuples",
            ));
        }
        let mut scalar_key = self.ty(id, group.quantity_type)?.key.clone();
        scalar_key.shape.clear();
        let scalar = q(id, self.input.registry.resolve_key(&scalar_key))?;
        for symbol in group.members.values() {
            let ty =
                self.input
                    .symbols
                    .symbol_type(*symbol)
                    .ok_or(MathIrError::UnknownBinding {
                        node: id,
                        binding: *symbol,
                    })?;
            q(
                id,
                pse_quantity::admission::require_same_contract(scalar, ty, self.input.registry),
            )?;
        }
        let ordered = ordered
            .into_iter()
            .map(|v| v.ok_or_else(|| MathIrError::malformed_at(id, "gather axis missing")))
            .collect::<Result<Vec<_>, _>>()?;
        Ok((group.quantity_type, ordered))
    }
    fn kernel(
        &mut self,
        id: NodeId,
        binding_id: SemanticId,
        output: u16,
        values: &[Value],
    ) -> Result<(Inferred, UnitId), MathIrError> {
        let binding =
            self.input
                .kernel_bindings
                .get(&binding_id)
                .ok_or(MathIrError::UnknownBinding {
                    node: id,
                    binding: binding_id,
                })?;
        let contract = self.input.symbols.kernel_contract(binding.kernel).ok_or(
            MathIrError::UnknownBinding {
                node: id,
                binding: binding.kernel,
            },
        )?;
        if binding.binding != binding_id
            || binding.inputs.len() != contract.inputs.len()
            || binding
                .inputs
                .iter()
                .zip(&contract.inputs)
                .any(|((name, _), port)| name != &port.name)
        {
            return Err(MathIrError::malformed_at(
                id,
                "kernel binding identity or ordered input names disagree with declaration",
            ));
        }
        let output = contract
            .outputs
            .get(usize::from(output))
            .ok_or_else(|| MathIrError::malformed_at(id, "kernel output ordinal absent"))?;
        self.kernel_parameters(id, binding, contract)?;
        let inputs: Vec<_> = contract.inputs.iter().map(|p| p.quantity_type).collect();
        let operands: Vec<_> = values
            .iter()
            .map(|v| Operand {
                quantity_type: v.quantity_type,
                indices: &v.indices,
            })
            .collect();
        let mut inferred = q(
            id,
            pse_quantity::infer::infer_with_evidence(
                &OpRequest::KernelCall {
                    declared_inputs: &inputs,
                    declared_output: output.quantity_type,
                },
                &operands,
                self.input.registry,
                self.input.symbols.invariant_checker(id),
            ),
        )?;
        if self.input.graph.node(id)?.opcode == Opcode::KernelCall
            && let Some(selection) = self.claimed.get(&id)
            && !selection.conversions.is_empty()
        {
            if selection.operation.is_some()
                || selection.builtin != Some(BuiltInRule::Binding)
                || !selection.permutation.is_empty()
            {
                return Err(MathIrError::malformed_at(
                    id,
                    "kernel conversion requires its built-in binding selection",
                ));
            }
            let [(0, conversion)] = selection.conversions.as_slice() else {
                return Err(MathIrError::malformed_at(
                    id,
                    "kernel conversion requires one named input-zero rule",
                ));
            };
            self.check_kernel_conversion(
                id,
                binding,
                contract,
                output.quantity_type,
                values,
                *conversion,
            )?;
            inferred
                .conversions
                .push(pse_quantity::infer::OperandConversion {
                    operand: 0,
                    conversion: *conversion,
                });
        }
        let mut remapped = binding.clone();
        for ((_, input), value) in remapped.inputs.iter_mut().zip(values) {
            *input = value.node;
        }
        if let Some(existing) = self.bindings.get(&binding_id)
            && existing != &remapped
        {
            return Err(MathIrError::malformed_at(
                id,
                "one binding resolved to different occurrence inputs",
            ));
        }
        self.bindings.insert(binding_id, remapped);
        Ok((inferred, output.unit))
    }
    fn kernel_parameters(
        &self,
        id: NodeId,
        binding: &crate::relations::vec_sink::KernelBinding,
        contract: &crate::infer::KernelContract,
    ) -> Result<(), MathIrError> {
        self.kernel_ports(id, contract)?;
        if binding.parameters.len() != contract.parameters.len() {
            return Err(MathIrError::malformed_at(
                id,
                "kernel parameter bindings are incomplete",
            ));
        }
        for ((name, symbol, value, unit), port) in
            binding.parameters.iter().zip(&contract.parameters)
        {
            if name != &port.name {
                return Err(MathIrError::malformed_at(
                    id,
                    "kernel parameter name/order differs from declaration",
                ));
            }
            match (symbol, value, unit) {
                (Some(symbol), None, None) => {
                    let ty = self.input.symbols.symbol_type(*symbol).ok_or(
                        MathIrError::UnknownBinding {
                            node: id,
                            binding: *symbol,
                        },
                    )?;
                    q(
                        id,
                        pse_quantity::admission::require_same_contract(
                            port.quantity_type,
                            ty,
                            self.input.registry,
                        ),
                    )?;
                    let actual = self
                        .input
                        .symbols
                        .symbol_unit(*symbol)
                        .unwrap_or(self.ty(id, ty)?.canonical_unit);
                    if actual != port.unit {
                        return Err(MathIrError::malformed_at(
                            id,
                            "kernel symbolic parameter unit differs from declared port",
                        ));
                    }
                }
                (None, Some(value), Some(unit)) if value.is_finite() && *unit == port.unit => {}
                _ => {
                    return Err(MathIrError::malformed_at(
                        id,
                        "kernel parameter requires one finite literal or declared symbol in port units",
                    ));
                }
            }
        }
        Ok(())
    }
    fn kernel_ports(
        &self,
        id: NodeId,
        contract: &crate::infer::KernelContract,
    ) -> Result<(), MathIrError> {
        for ports in [&contract.inputs, &contract.outputs, &contract.parameters] {
            let mut names = BTreeSet::new();
            for port in ports {
                if port.name.is_empty() || !names.insert(&port.name) {
                    return Err(MathIrError::malformed_at(
                        id,
                        "kernel port names must be nonempty and unique within each declaration group",
                    ));
                }
            }
        }
        for port in contract
            .inputs
            .iter()
            .chain(&contract.outputs)
            .chain(&contract.parameters)
        {
            let quantity = self.ty(id, port.quantity_type)?;
            if quantity.key.shape != port.shape {
                return Err(MathIrError::malformed_at(
                    id,
                    "kernel port shape disagrees with its complete quantity contract",
                ));
            }
            let unit = q(id, self.input.registry.unit(port.unit))?;
            let canonical = q(id, self.input.registry.unit(quantity.canonical_unit))?;
            q(
                id,
                pse_quantity::convert_spec_for_type(unit, canonical, &quantity.key),
            )?;
        }
        Ok(())
    }
    fn equation(&mut self, source: &EquationRecord) -> Result<EquationRecord, MathIrError> {
        let mut environment = IndexSet::new();
        for (position, index) in source.free_indices.iter().enumerate() {
            if position != usize::from(index.position) {
                return Err(MathIrError::malformed_at(
                    source.body,
                    "equation axis order is not contiguous",
                ));
            }
            environment = crate::index::bind(
                source.body,
                &environment,
                index.bound_index,
                index.domain,
                self.domain(source.body, index.domain)?,
            )?;
        }
        let body = self.visit(&Request {
            node: source.body,
            expected: None,
            environment: environment.clone(),
            representation: None,
            preserve_representation: false,
            guarded: false,
        })?;
        if body.indices != environment {
            return Err(MathIrError::malformed_at(
                source.body,
                "equation free indices differ from its body",
            ));
        }
        let mut equation = source.clone();
        equation.body = body.node;
        let mut residual = None;
        for bound in [&mut equation.lower, &mut equation.upper] {
            if let Some(node) = *bound {
                let value = self.visit(&Request {
                    node,
                    expected: Some(body.quantity_type),
                    environment: environment.clone(),
                    representation: None,
                    preserve_representation: false,
                    guarded: false,
                })?;
                let inferred = q(
                    node,
                    pse_quantity::infer::infer_with_evidence(
                        &OpRequest::Sub,
                        &[
                            Operand {
                                quantity_type: body.quantity_type,
                                indices: &body.indices,
                            },
                            Operand {
                                quantity_type: value.quantity_type,
                                indices: &value.indices,
                            },
                        ],
                        self.input.registry,
                        self.input.symbols.invariant_checker(node),
                    ),
                )?;
                if residual.is_some_and(|prior| prior != inferred.result) {
                    return Err(MathIrError::malformed_at(
                        node,
                        "equation bounds imply different residual types",
                    ));
                }
                residual = Some(inferred.result);
                *bound = Some(value.node);
            }
        }
        validate_equation_bounds(&equation)?;
        if let Some(claimed) = source.residual_quantity_type
            && Some(claimed) != residual
        {
            return Err(MathIrError::malformed_at(
                source.body,
                "declared residual type differs from subtraction inference",
            ));
        }
        equation.residual_quantity_type = residual;
        if let Some(filter) = source.filter {
            let value = self.visit(&Request {
                node: filter,
                expected: None,
                environment,
                representation: None,
                preserve_representation: false,
                guarded: false,
            })?;
            self.guard(&value)?;
            equation.filter = Some(value.node);
        }
        Ok(equation)
    }
}
fn builtin(result: QuantityTypeId, indices: IndexSet, rule: BuiltInRule) -> Inferred {
    Inferred {
        result,
        indices,
        selected: OperationSelection::BuiltIn(rule),
        conversions: vec![],
    }
}
fn q<T>(node: NodeId, result: Result<T, pse_quantity::QuantityError>) -> Result<T, MathIrError> {
    result.map_err(|source| MathIrError::Quantity { node, source })
}

fn validate_equation_bounds(equation: &EquationRecord) -> Result<(), MathIrError> {
    let valid = match equation.sense {
        Sense::Eq | Sense::Definition => equation.lower.is_some() || equation.upper.is_some(),
        Sense::Le => equation.lower.is_none() && equation.upper.is_some(),
        Sense::Ge => equation.lower.is_some() && equation.upper.is_none(),
        Sense::Range => equation.lower.is_some() && equation.upper.is_some(),
    };
    if !valid {
        return Err(MathIrError::malformed_at(
            equation.body,
            "equation bounds disagree with sense",
        ));
    }
    Ok(())
}
