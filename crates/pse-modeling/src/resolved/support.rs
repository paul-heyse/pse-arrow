// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Resolver-owned physical products. Numerical construction consumes these exact admissions.
use super::*;
#[path = "support_record.rs"]
pub(super) mod record;
pub use record::SupportRecord;
#[path = "provider_support.rs"]
mod provider_support;
use pse_quantity::{
    CanonicalConversionPlan, PhysicalPreconditions, QuantityRegistry, ResolvedInference,
    ResolvedPhysicalContract,
    infer::{Exponent, OpRequest},
    literal::LiteralContext,
    scheme::Scheme,
};

/// A literal's actual admitted representation conversion, independent of numerical construction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiteralSupport {
    /// Resolver-issued canonical conversion, retaining its complete admitted target.
    pub conversion: CanonicalConversionPlan,
}

/// One explicitly declared body-only view, admitted by the existing quantity boundary owner.
#[derive(Clone, Debug)]
pub struct BodyInputSupport {
    /// Actual typed source authorization relationship, published before the function escapes.
    pub relationship: crate::derived::Relationship,
    /// Exact nominal input, body view and numerical scale.
    pub boundary: pse_quantity::AdmittedOutputBoundary,
}
/// Exact ordinary input boundaries of one selected function application.
#[derive(Clone, Debug)]
pub struct CallSupport {
    /// Actual selected immutable function in this syntax owner.
    pub function: FunctionIx,
    /// Complete declared nominal result before explicit partials or body-only views.
    pub nominal_result: ResolvedPhysicalContract,
    /// Exact ordered formal partials, including repeated differentiation positions.
    pub wrt: Vec<usize>,
    /// Ordinary nominal call boundaries, separate from internal body authorizations.
    pub inputs: Vec<OutputSupport>,
    /// Declared reconstruction-only result view, if that exact owner consumes it.
    pub body_result: Option<BodyInputSupport>,
    /// Actual declared result/formal partial operation, never inferred by lowering.
    pub partial: Option<ResolvedInference>,
}
/// Exact ordinary boundaries for the original hard range of a demanded scalar.
#[derive(Clone, Debug)]
pub struct RangeSupport {
    /// Actual lower endpoint into the original scalar's full nominal contract.
    pub lower: OutputSupport,
    /// Actual upper endpoint into the original scalar's full nominal contract.
    pub upper: OutputSupport,
    /// Actual value-minus-lower arithmetic admission for the original lower guard.
    pub lower_difference: ResolvedInference,
    /// Actual upper-minus-value arithmetic admission for the original upper guard.
    pub upper_difference: ResolvedInference,
}
/// One reached node's complete semantic type and actual physical operation, when applicable.
#[derive(Clone, Debug)]
pub struct NodeSupport {
    /// Nominal refinements remain present, including actual transfer owners.
    pub ty: Type,
    /// Complete resolved physical contract, including representation and lexical indices.
    pub contract: ResolvedPhysicalContract,
    /// Actual arithmetic admission, never a claimed expected type in its place.
    pub operation: Option<ResolvedInference>,
    /// Actual literal conversion, if this node is a source literal.
    pub literal: Option<LiteralSupport>,
    /// Actual declared member boundary, separate from arithmetic and literal admission.
    pub boundary: Option<OutputSupport>,
    /// Actual function call admission, if this node selects a complete function.
    pub call: Option<CallSupport>,
    /// Source-issued hard range, applied at this original scalar reference.
    pub range: Option<RangeSupport>,
    /// Exact retained literal exponent for numerical power/domain construction.
    pub exponent: Option<pse_quantity::Ratio>,
}

/// The physical quantity owner retains the ordinary result boundary admission.
pub use pse_quantity::AdmittedContractBoundary as OutputSupport;

/// Quantity-owner facts for numerical Boolean indicators and lazy predicate branches.
#[derive(Clone, Debug)]
pub struct BooleanSupport {
    /// Complete registered neutral indicator contract.
    pub contract: ResolvedPhysicalContract,
    /// Exact canonical zero/one conversion.
    pub conversion: CanonicalConversionPlan,
    /// Actual literal admission, consumed for either Boolean literal.
    pub literal: ResolvedInference,
    /// Exact neutral branch result operation, including original lazy guard barriers.
    pub conditional: ResolvedInference,
}
/// Immutable support for one exact reached resolved syntax product.
#[derive(Clone, Debug)]
pub struct PhysicalSupport {
    syntax: ContentHash,
    nodes: BTreeMap<NodeIx, NodeSupport>,
    order: Vec<NodeIx>,
    outputs: Vec<OutputSupport>,
    inputs: Vec<(SymbolIx, ResolvedPhysicalContract)>,
    premises: PhysicalEvidence,
    boolean: Option<BooleanSupport>,
    reduction: Option<ResolvedInference>,
    reduction_output: Option<OutputSupport>,
}
/// Temporary direct-graph inquiry with the literal contexts it actually consumed.
pub(crate) struct InferredView {
    pub(crate) outputs: Vec<NodeSupport>,
    pub(crate) literals: BTreeMap<NodeIx, Type>,
}
/// All already-issued physical facts for one selected function's direct graph.
pub(crate) struct FunctionSupportRequest<'a> {
    pub(crate) body: Option<NodeIx>,
    pub(crate) result: &'a Type,
    pub(crate) literals: &'a BTreeMap<NodeIx, Type>,
    pub(crate) operation: Option<&'a crate::PhysicalOperation>,
    pub(crate) at: DeclarationId,
    pub(crate) obligations: &'a BTreeMap<NodeIx, crate::expression::admission::PhysicalAdmission>,
    pub(crate) reduction: Option<&'a crate::expression::admission::PhysicalAdmission>,
    pub(crate) signature: &'a [Type],
    pub(crate) formals: &'a [SymbolIx],
    pub(crate) meaning: Option<&'a OperationMeaning>,
}
impl PhysicalSupport {
    /// Prepare a scalar reached graph with its actual physical context and contextual literals.
    /// Application/indexed products must be supplied by their owning producer before that cutover.
    pub fn prepare_scalar(
        syntax: &ResolvedSyntax,
        registry: &QuantityRegistry,
        facts: &PhysicalPreconditions,
        expected: &[Type],
        literal_contracts: &BTreeMap<NodeIx, Type>,
    ) -> Result<Self> {
        prepare_scalar(syntax, registry, facts, expected, literal_contracts)
    }
    pub(crate) fn infer_view(
        syntax: View<'_>,
        context: &crate::TypeContext<'_>,
        literals: &BTreeMap<NodeIx, Type>,
        authority: Option<&crate::PhysicalOperation>,
        owner: DeclarationId,
        expected: Option<&Type>,
    ) -> Result<InferredView> {
        let context = crate::TypeContext {
            admissions: None,
            formula_authority: context.formula_authority.clone(),
            quantities: context.quantities,
            preconditions: context.preconditions,
            scope: context.scope,
        };
        let mut producer = Producer {
            syntax,
            context,
            body_authority: authority,
            owner,
            literal_contracts: literals,
            nodes: BTreeMap::new(),
            order: Vec::new(),
            active: std::collections::BTreeSet::new(),
            predicates: BTreeMap::new(),
            boolean: None,
            depth: 0,
        };
        let outputs = syntax
            .outputs
            .iter()
            .map(|root| producer.expression(*root, expected))
            .collect::<Result<_>>()?;
        for predicate in syntax.validity {
            producer.predicate(*predicate)?;
        }
        let literals = producer
            .nodes
            .into_iter()
            .filter_map(|(root, support)| support.literal.is_some().then_some((root, support.ty)))
            .collect();
        Ok(InferredView { outputs, literals })
    }
    /// Prepare one complete selected function from its emitted graph. The original
    /// operation descriptor grants only its declared nominal output authorization.
    pub(crate) fn prepare_function(
        syntax: &ResolvedSyntax,
        context: &crate::TypeContext<'_>,
        request: FunctionSupportRequest<'_>,
    ) -> Result<(
        Self,
        Option<pse_quantity::AdmittedOutputBoundary>,
        Vec<Option<BodyInputSupport>>,
    )> {
        let FunctionSupportRequest {
            body,
            result,
            literals,
            operation,
            at,
            obligations,
            reduction,
            signature,
            formals,
            meaning,
        } = request;
        let identity = syntax.identity()?;
        let (
            (nodes, order, outputs, boundary, body_inputs, boolean, reduction, reduction_output),
            premises,
        ) = pse_quantity::premises::capture(context.quantities, context.preconditions, || {
            let context = crate::TypeContext {
                admissions: None,
                formula_authority: matches!(
                    operation,
                    Some(crate::PhysicalOperation::Response { .. })
                )
                .then(|| pse_quantity::PhysicalFormulaAuthority::response(at.as_id())),
                quantities: context.quantities,
                preconditions: context.preconditions,
                scope: context.scope,
            };
            let mut producer = Producer {
                syntax: syntax.view(),
                context,
                body_authority: operation,
                owner: at,
                literal_contracts: literals,
                nodes: BTreeMap::new(),
                order: Vec::new(),
                active: std::collections::BTreeSet::new(),
                predicates: BTreeMap::new(),
                boolean: None,
                depth: 0,
            };
            if signature.len() != formals.len() {
                return Err(invalid(
                    at,
                    "physical function nominal/body input arity differs",
                ));
            }
            let layout = syntax.canonical_layout()?;
            let mut body_inputs = Vec::with_capacity(formals.len());
            for (ordinal, (nominal, slot)) in signature.iter().zip(formals).enumerate() {
                let body = &syntax
                    .symbols
                    .at(*slot)
                    .ok_or_else(|| {
                        invalid(at, "physical function body input is outside its owner")
                    })?
                    .ty;
                // Reading both exact contracts here joins their quantity premises to this capture.
                let source = producer.contract(nominal, at)?;
                let target = producer.contract(body, at)?;
                if nominal == body {
                    body_inputs.push(None);
                    continue;
                }
                let meaning = meaning.ok_or_else(|| {
                    invalid(
                        at,
                        "a changed body input view lacks its declared operation authority",
                    )
                })?;
                let mut hash = FramedHasher::new(pse_ids::Frame::ModelingResolvedFunctionV1);
                meaning.frame(&layout, &mut hash, at)?;
                frame_type(nominal, &mut hash);
                frame_type(body, &mut hash);
                let relationship = crate::derived::Relationship::FunctionBodyInputView {
                    owner: at,
                    ordinal: ordinal as u64,
                    meaning: hash.finish_hash(),
                };
                let boundary = pse_quantity::AdmittedOutputBoundary::declared_contract(
                    relationship.identity(),
                    source,
                    target,
                )
                .map_err(|error| invalid(at, error.to_string()))?;
                body_inputs.push(Some(BodyInputSupport {
                    relationship,
                    boundary,
                }));
            }
            let mut outputs = Vec::with_capacity(syntax.outputs.len());
            let mut boundary = None;
            for root in &syntax.outputs {
                let is_body = body == Some(*root);
                let value = producer.expression(
                    *root,
                    if is_body && operation.is_none() {
                        Some(result)
                    } else {
                        None
                    },
                )?;
                let target = if is_body {
                    if let Some(operation) = operation {
                        boundary = Some(operation.admit_result(
                            &value.ty,
                            result,
                            &producer.context,
                            at,
                        )?);
                        value.contract.clone()
                    } else {
                        if value.ty.physical_refinement() != result.physical_refinement() {
                            return Err(invalid(
                                at,
                                "selected function body changes its declared nominal physical role",
                            ));
                        }
                        producer.contract(result, at)?
                    }
                } else {
                    value.contract.clone()
                };
                outputs.push(
                    OutputSupport::prepare(&value.contract, &target, producer.context.quantities)
                        .map_err(|error| invalid(at, error.to_string()))?,
                );
            }
            for predicate in &syntax.validity {
                producer.predicate(*predicate)?;
            }
            // Replay exact checked source obligations inside the same premise capture.
            // The direct graph independently supplies its actual operand/result contracts.
            for (root, obligation) in obligations {
                let Some(node) = producer.nodes.get(root) else {
                    continue;
                };
                let retained = obligation.specialize(
                    producer.context.quantities,
                    &BTreeMap::new(),
                    producer.context.preconditions,
                    producer.context.formula_authority.as_ref(),
                    at,
                )?;
                let actual=node.operation.as_ref().ok_or_else(||invalid(at,"source physical obligation lacks its demanded emitted arithmetic operation"))?;
                if retained.operands != actual.operands || retained.result != actual.result {
                    return Err(invalid(
                        at,
                        "emitted arithmetic differs from its exact original physical operands or result",
                    ));
                }
            }
            let reduction = reduction
                .map(|reduction| {
                    reduction.specialize(
                        producer.context.quantities,
                        &BTreeMap::new(),
                        producer.context.preconditions,
                        None,
                        at,
                    )
                })
                .transpose()?;
            let reduction_output = reduction
                .as_ref()
                .map(|reduction| {
                    OutputSupport::prepare(
                        &reduction.result,
                        &producer.contract(result, at)?,
                        producer.context.quantities,
                    )
                    .map_err(|error| invalid(at, error.to_string()))
                })
                .transpose()?;
            Ok::<_, crate::ModelingError>((
                producer.nodes,
                producer.order,
                outputs,
                boundary,
                body_inputs,
                producer.boolean,
                reduction,
                reduction_output,
            ))
        })?;
        let premises = premises.ok_or_else(|| {
            invalid(
                at,
                "selected function lacks its complete actual physical premise capture",
            )
        })?;
        let support = Self {
            syntax: identity,
            nodes,
            order,
            outputs,
            inputs: Vec::new(),
            premises: PhysicalEvidence::capture(premises),
            boolean,
            reduction,
            reduction_output,
        }
        .with_inputs(syntax, formals, context)?;
        Ok((support, boundary, body_inputs))
    }
    /// Prepare the actual region outputs, with no compiler-supplied expected type authority.
    pub(crate) fn prepare_reached(
        syntax: &ResolvedSyntax,
        context: &crate::TypeContext<'_>,
        literals: &BTreeMap<NodeIx, Type>,
        obligations: &BTreeMap<NodeIx, crate::expression::admission::PhysicalAdmission>,
    ) -> Result<Self> {
        prepare_outputs(syntax, context, None, literals, obligations)
    }
    /// Select a term's already admitted facts from its parent relation context.
    /// Only a pure coordinate mapping runs; the term's type is never re-inferred.
    pub(super) fn select_relocated(
        &self,
        relocation: &selection::Relocation,
        identity: ContentHash,
        root: NodeIx,
        registry: &QuantityRegistry,
    ) -> Result<Self> {
        let mut nodes = BTreeMap::new();
        let mut order = Vec::new();
        for slot in &self.order {
            let Some(local) = relocation.nodes.get(slot).copied() else {
                continue;
            };
            let mut value = self.nodes.get(slot).cloned().ok_or_else(|| {
                invalid(
                    SemanticId::NIL,
                    "admitted parent support lost an ordered node",
                )
            })?;
            if let Some(call) = &mut value.call {
                call.function = relocation.function(call.function)?;
            }
            nodes.insert(local, value);
            order.push(local);
        }
        let value = self.node(root)?;
        let output = OutputSupport::prepare(&value.contract, &value.contract, registry)
            .map_err(|error| invalid(SemanticId::NIL, error.to_string()))?;
        Ok(Self {
            syntax: identity,
            nodes,
            order,
            outputs: vec![output],
            inputs: self
                .inputs
                .iter()
                .filter_map(|(slot, value)| {
                    relocation
                        .symbols
                        .get(slot)
                        .map(|local| (*local, value.clone()))
                })
                .collect(),
            premises: self.premises.clone(),
            boolean: self.boolean.clone(),
            reduction: self.reduction.clone(),
            reduction_output: self.reduction_output.clone(),
        })
    }
    /// Ordered source-issued nominal input contracts, including genuinely unused ports.
    pub fn inputs(&self) -> &[(SymbolIx, ResolvedPhysicalContract)] {
        &self.inputs
    }
    /// Exact nominal contract of a prepared independent scalar slot.
    pub fn input(&self, slot: SymbolIx) -> Result<&ResolvedPhysicalContract> {
        self.inputs
            .iter()
            .find(|(local, _)| *local == slot)
            .map(|(_, value)| value)
            .ok_or_else(|| invalid(SemanticId::NIL, "prepared nominal input contract is absent"))
    }
    /// Populate the exact ordered layout before keying numerical construction. Reached
    /// contracts are reused; only an actually unused declared port is resolved here.
    pub(super) fn with_inputs(
        mut self,
        syntax: &ResolvedSyntax,
        slots: &[SymbolIx],
        context: &crate::TypeContext<'_>,
    ) -> Result<Self> {
        if self.syntax != syntax.identity()? {
            return Err(invalid(
                SemanticId::NIL,
                "input support belongs to a different syntax",
            ));
        }
        let (inputs, premises) =
            pse_quantity::premises::capture(context.quantities, context.preconditions, || {
                if !self.observe(context.quantities, context.preconditions)? {
                    return Err(invalid(
                        SemanticId::NIL,
                        "input support has stale live physical premises",
                    ));
                }
                retained_inputs(syntax, slots, &self.nodes, &self.inputs, context)
            })?;
        self.inputs = inputs;
        self.premises =
            PhysicalEvidence::capture(premises.ok_or_else(|| {
                invalid(SemanticId::NIL, "input contract capture is unsupported")
            })?);
        Ok(self)
    }
    /// Exact admission comparison after an independently issued typed graph correspondence.
    pub(super) fn same_node(&self, left: NodeIx, other: &Self, right: NodeIx) -> Result<bool> {
        let mut a = FramedHasher::new(pse_ids::Frame::ModelingResolvedBodyV1);
        let mut b = FramedHasher::new(pse_ids::Frame::ModelingResolvedBodyV1);
        match (self.nodes.get(&left), other.nodes.get(&right)) {
            (None, None) => return Ok(true),
            (Some(left), Some(right)) => {
                frame_node_support(left, &mut a);
                frame_node_support(right, &mut b);
            }
            _ => return Ok(false),
        }
        Ok(a.finish_hash() == b.finish_hash())
    }
    /// Relocate already-issued admissions, unioning only genuinely live captures.
    /// Every collision must be justified by the caller's typed correspondence and
    /// preserve exact physical facts. Original operation admission never runs here.
    pub(super) fn combine_relocated(
        parts: &[(&Self, &selection::Relocation)],
        syntax: &ResolvedSyntax,
        slots: &[SymbolIx],
        context: &crate::TypeContext<'_>,
    ) -> Result<Self> {
        let ((nodes, order, outputs, inputs, boolean, reduction, reduction_output), premises) =
            pse_quantity::premises::capture(context.quantities, context.preconditions, || {
                let mut nodes = BTreeMap::new();
                let mut order = Vec::new();
                let mut outputs = Vec::new();
                let mut prior_inputs = Vec::new();
                let mut boolean: Option<BooleanSupport> = None;
                for (support, map) in parts {
                    if !support.observe(context.quantities, context.preconditions)? {
                        return Err(invalid(
                            SemanticId::NIL,
                            "composed support has stale live physical premises",
                        ));
                    }
                    if support.reduction.is_some() || support.reduction_output.is_some() {
                        return Err(invalid(
                            SemanticId::NIL,
                            "function reduction support cannot masquerade as a selected scalar region",
                        ));
                    }
                    for old in &support.order {
                        let Some(local) = map.nodes.get(old).copied() else {
                            continue;
                        };
                        let mut value = support.node(*old)?.clone();
                        if let Some(call) = &mut value.call {
                            call.function = map.function(call.function)?;
                        }
                        if let Some(previous) = nodes.get(&local) {
                            let mut a = FramedHasher::new(pse_ids::Frame::ModelingResolvedBodyV1);
                            let mut b = FramedHasher::new(pse_ids::Frame::ModelingResolvedBodyV1);
                            frame_node_support(previous, &mut a);
                            frame_node_support(&value, &mut b);
                            if a.finish_hash() != b.finish_hash() {
                                return Err(invalid(
                                    SemanticId::NIL,
                                    "typed merged occurrence has conflicting physical admissions",
                                ));
                            }
                        } else {
                            nodes.insert(local, value);
                            order.push(local);
                        }
                    }
                    for (old, value) in &support.inputs {
                        if let Some(local) = map.symbols.get(old) {
                            prior_inputs.push((*local, value.clone()));
                        }
                    }
                    outputs.extend(support.outputs.iter().cloned());
                    if let Some(value) = &support.boolean {
                        if let Some(previous) = &boolean {
                            if previous.contract != value.contract
                                || previous.conversion != value.conversion
                                || pse_quantity::resolved::receipts::InferenceRecord::capture(
                                    &previous.literal,
                                ) != pse_quantity::resolved::receipts::InferenceRecord::capture(
                                    &value.literal,
                                )
                                || pse_quantity::resolved::receipts::InferenceRecord::capture(
                                    &previous.conditional,
                                ) != pse_quantity::resolved::receipts::InferenceRecord::capture(
                                    &value.conditional,
                                )
                            {
                                return Err(invalid(
                                    SemanticId::NIL,
                                    "composed regions disagree on their actual Boolean constructor",
                                ));
                            }
                        } else {
                            boolean = Some(value.clone());
                        }
                    }
                }
                let inputs = retained_inputs(syntax, slots, &nodes, &prior_inputs, context)?;
                Ok::<_, crate::ModelingError>((nodes, order, outputs, inputs, boolean, None, None))
            })?;
        if outputs.len() != syntax.outputs.len() {
            return Err(invalid(
                SemanticId::NIL,
                "composed output admissions lost original order",
            ));
        }
        Ok(Self {
            syntax: syntax.identity()?,
            nodes,
            order,
            outputs,
            inputs,
            boolean,
            reduction,
            reduction_output,
            premises: PhysicalEvidence::capture(premises.ok_or_else(|| {
                invalid(
                    SemanticId::NIL,
                    "composition did not issue a complete live physical capture",
                )
            })?),
        })
    }
    /// Exact original hard ranges actually retained by this support preparation.
    /// Unused lexical bindings do not acquire eager validity obligations.
    pub fn checked_ranges(&self, syntax: &ResolvedSyntax) -> Result<Vec<CheckedRange>> {
        if self.syntax != syntax.identity()? {
            return Err(invalid(
                SemanticId::NIL,
                "range support belongs to a different exact syntax",
            ));
        }
        let required = self
            .nodes
            .iter()
            .filter_map(|(root, support)| {
                support.range.as_ref()?;
                match syntax.nodes.get(*root) {
                    Some(Node::Reference(Reference::Symbol(symbol))) => Some(*symbol),
                    _ => None,
                }
            })
            .collect::<std::collections::BTreeSet<_>>();
        Ok(syntax
            .canonical_layout()?
            .symbols
            .into_iter()
            .filter(|value| required.contains(&value.symbol))
            .map(|value| CheckedRange {
                symbol: value.symbol,
                member: crate::derived::Relationship::CheckedMember {
                    role: value.role,
                    ordinal: value.ordinal as u64,
                }
                .identity(),
                source: crate::derived::Relationship::CheckedValiditySource {
                    role: value.role,
                    ordinal: value.ordinal as u64,
                }
                .identity(),
            })
            .collect())
    }
    /// Exact syntax whose node coordinates this product admits.
    pub fn syntax(&self) -> ContentHash {
        self.syntax
    }
    /// Require an actual reached admission. An omitted or unrelated node is not an expected type.
    pub fn node(&self, slot: NodeIx) -> Result<&NodeSupport> {
        self.nodes
            .get(&slot)
            .ok_or_else(|| invalid(SemanticId::NIL, "required resolved physical node is absent"))
    }
    /// Model-owned numerical Boolean constructor admissions; absence is an explicit refusal.
    pub fn boolean(&self) -> Result<&BooleanSupport> {
        self.boolean
            .as_ref()
            .ok_or_else(|| invalid(SemanticId::NIL, "resolved Boolean support is absent"))
    }
    /// Original finite-reduction prototype operation issued during selected function preparation.
    pub fn reduction(&self) -> Result<&ResolvedInference> {
        self.reduction.as_ref().ok_or_else(|| {
            invalid(
                SemanticId::NIL,
                "selected finite reduction admission is absent",
            )
        })
    }
    /// Actual complete synthesized-reduction result into its declared nominal contract.
    pub fn reduction_output(&self) -> Result<&OutputSupport> {
        self.reduction_output.as_ref().ok_or_else(|| {
            invalid(
                SemanticId::NIL,
                "selected finite reduction output boundary is absent",
            )
        })
    }
    /// Actually issued reconstruction-result relationships; the owning specialization publishes them.
    pub(crate) fn result_views(&self) -> impl Iterator<Item = crate::derived::Relationship> + '_ {
        self.nodes.values().filter_map(|value| {
            value
                .call
                .as_ref()
                .and_then(|call| call.body_result.as_ref())
                .map(|view| view.relationship)
        })
    }
    /// Actual resolver read values, issued by the synchronous successful preparation.
    pub fn premises(&self) -> Result<&pse_quantity::premises::PhysicalPremises> {
        self.premises.live()
    }
    /// Actual live preparation context. Strict restored products cannot enter Salsa preparation.
    pub fn context(&self) -> Result<(&QuantityRegistry, &PhysicalPreconditions)> {
        Ok(self.premises()?.context())
    }
    /// Observe actual live consumed questions; qualified replay has no live observation authority.
    pub fn observe(
        &self,
        registry: &QuantityRegistry,
        facts: &PhysicalPreconditions,
    ) -> Result<bool> {
        Ok(self.premises()?.observe(registry, facts))
    }
    /// Require the actual result boundary at this ordered requested output.
    pub fn output(&self, index: usize) -> Result<&OutputSupport> {
        self.outputs.get(index).ok_or_else(|| {
            invalid(
                SemanticId::NIL,
                "required resolved output boundary is absent",
            )
        })
    }
    /// Known retained support extent. Shared premises are conservatively charged to this product.
    pub fn retained_bytes(&self) -> usize {
        self.nodes.values().fold(
            size_of::<Self>()
                .saturating_add(
                    self.nodes
                        .len()
                        .saturating_mul(size_of::<(NodeIx, NodeSupport)>().saturating_add(96)),
                )
                .saturating_add(self.order.capacity().saturating_mul(size_of::<NodeIx>()))
                .saturating_add(
                    self.inputs
                        .capacity()
                        .saturating_mul(size_of::<(SymbolIx, ResolvedPhysicalContract)>()),
                )
                .saturating_add(self.inputs.iter().fold(0usize, |bytes, (_, value)| {
                    bytes.saturating_add(value.heap_bytes())
                }))
                .saturating_add(
                    self.outputs
                        .capacity()
                        .saturating_mul(size_of::<OutputSupport>()),
                )
                .saturating_add(self.outputs.iter().fold(0usize, |bytes, output| {
                    bytes.saturating_add(output.heap_bytes())
                }))
                .saturating_add(self.premises.retained_bytes())
                .saturating_add(self.boolean.as_ref().map_or(0, |boolean| {
                    boolean
                        .contract
                        .heap_bytes()
                        .saturating_add(boolean.conversion.heap_bytes())
                        .saturating_add(extent::inference(&boolean.literal))
                        .saturating_add(extent::inference(&boolean.conditional))
                })),
            |bytes, value| {
                bytes
                    .saturating_add(extent::ty(&value.ty))
                    .saturating_add(value.contract.heap_bytes())
                    .saturating_add(value.operation.as_ref().map_or(0, extent::inference))
                    .saturating_add(
                        value
                            .literal
                            .as_ref()
                            .map_or(0, |literal| literal.conversion.heap_bytes()),
                    )
                    .saturating_add(
                        value
                            .boundary
                            .as_ref()
                            .map_or(0, |boundary| boundary.heap_bytes()),
                    )
                    .saturating_add(value.range.as_ref().map_or(0, |range| {
                        range
                            .lower
                            .heap_bytes()
                            .saturating_add(range.upper.heap_bytes())
                            .saturating_add(extent::inference(&range.lower_difference))
                            .saturating_add(extent::inference(&range.upper_difference))
                    }))
                    .saturating_add(value.call.as_ref().map_or(0, |call| {
                        call.wrt
                            .capacity()
                            .saturating_mul(size_of::<usize>())
                            .saturating_add(
                                call.inputs
                                    .capacity()
                                    .saturating_mul(size_of::<OutputSupport>()),
                            )
                            .saturating_add(call.inputs.iter().fold(0usize, |bytes, boundary| {
                                bytes.saturating_add(boundary.heap_bytes())
                            }))
                    }))
            },
        )
    }
    /// Frame actual products in reached traversal order, independent of source or arena ordinals.
    pub fn frame(&self, hash: &mut FramedHasher) {
        hash.str("resolved_physical_support").hash(&self.syntax);
        hash.u64(self.inputs.len() as u64);
        for (_, value) in &self.inputs {
            value.frame(hash);
        }
        self.premises.frame(hash);
        hash.bool(self.boolean.is_some());
        if let Some(boolean) = &self.boolean {
            boolean.contract.frame(hash);
            boolean.literal.frame(hash);
            boolean.conditional.frame(hash);
            hash.id(&boolean.conversion.quantity().as_id())
                .id(&boolean.conversion.source_unit().as_id())
                .id(&boolean.conversion.canonical_unit().as_id())
                .u64(boolean.conversion.scale().to_bits())
                .u64(boolean.conversion.offset().to_bits());
        }
        hash.bool(self.reduction.is_some());
        if let Some(reduction) = &self.reduction {
            reduction.frame(hash);
        }
        hash.bool(self.reduction_output.is_some());
        if let Some(output) = &self.reduction_output {
            output.frame(hash);
        }
        hash.u64(self.order.len() as u64);
        for slot in &self.order {
            if let Some(support) = self.nodes.get(slot) {
                frame_node_support(support, hash);
            }
        }
        hash.u64(self.outputs.len() as u64);
        for output in &self.outputs {
            output.frame(hash);
        }
    }
}

fn retained_inputs(
    syntax: &ResolvedSyntax,
    slots: &[SymbolIx],
    nodes: &BTreeMap<NodeIx, NodeSupport>,
    prior: &[(SymbolIx, ResolvedPhysicalContract)],
    context: &crate::TypeContext<'_>,
) -> Result<Vec<(SymbolIx, ResolvedPhysicalContract)>> {
    let literals = BTreeMap::new();
    let producer = Producer {
        syntax: syntax.view(),
        context: crate::TypeContext {
            admissions: None,
            formula_authority: context.formula_authority.clone(),
            quantities: context.quantities,
            preconditions: context.preconditions,
            scope: context.scope,
        },
        body_authority: None,
        owner: SemanticId::NIL.into(),
        literal_contracts: &literals,
        nodes: BTreeMap::new(),
        order: Vec::new(),
        active: std::collections::BTreeSet::new(),
        predicates: BTreeMap::new(),
        boolean: None,
        depth: 0,
    };
    let mut seen = std::collections::BTreeSet::new();
    slots.iter().map(|slot| {
        if !seen.insert(*slot) {return Err(invalid(SemanticId::NIL,"ordered numerical input slot is repeated"));}
        let symbol=syntax.symbols.at(*slot).ok_or_else(||invalid(SemanticId::NIL,"ordered input is outside its actual symbol owner"))?;
        if symbol.definition.is_some() || symbol.local_role()==LocalRole::Member {return Err(invalid(symbol.id,"defined member cannot be a numerical input port"));}
        let mut contract=None;
        for (_,value) in prior.iter().filter(|(local,_)|local==slot) {
            if contract.as_ref().is_some_and(|previous|previous!=value) {return Err(invalid(symbol.id,"same exact input has conflicting retained nominal contracts"));}
            contract=Some(value.clone());
        }
        for (root,value) in nodes {
            if matches!(syntax.nodes.get(*root),Some(Node::Reference(Reference::Symbol(local))) if local==slot) {
                if value.ty!=symbol.ty {return Err(invalid(symbol.id,"retained input type differs from its actual declaration"));}
                if contract.as_ref().is_some_and(|previous|previous!=&value.contract) {return Err(invalid(symbol.id,"same exact input has conflicting node contracts"));}
                contract=Some(value.contract.clone());
            }
        }
        let contract=match contract {Some(value)=>value,None=>producer.contract(&symbol.ty,SemanticId::NIL.into())?};
        Ok((*slot,contract))
    }).collect()
}

fn frame_node_support(support: &NodeSupport, hash: &mut FramedHasher) {
    frame_type(&support.ty, hash);
    support.contract.frame(hash);
    hash.bool(support.exponent.is_some());
    if let Some(power) = support.exponent {
        hash.part(&power.num().to_le_bytes())
            .part(&power.den().to_le_bytes());
    }
    match &support.operation {
        Some(operation) => {
            hash.str("operation");
            operation.frame(hash);
        }
        None => {
            hash.str("value");
        }
    }
    match &support.literal {
        Some(literal) => {
            hash.str("literal_conversion")
                .id(&literal.conversion.quantity().as_id())
                .id(&literal.conversion.source_unit().as_id())
                .id(&literal.conversion.canonical_unit().as_id())
                .u64(literal.conversion.scale().to_bits())
                .u64(literal.conversion.offset().to_bits());
        }
        None => {
            hash.str("no_literal_conversion");
        }
    }
    if let Some(call) = &support.call {
        hash.str("selected-call");
        call.nominal_result.frame(hash);
        hash.u64(call.wrt.len() as u64);
        for ordinal in &call.wrt {
            hash.u64(*ordinal as u64);
        }
        hash.u64(call.inputs.len() as u64);
        for boundary in &call.inputs {
            boundary.frame(hash);
        }
        hash.bool(call.body_result.is_some());
        if let Some(view) = &call.body_result {
            hash.id(&view.relationship.identity());
            view.boundary.frame(hash);
        }
        hash.bool(call.partial.is_some());
        if let Some(partial) = &call.partial {
            partial.frame(hash);
        }
    } else {
        hash.str("no-selected-call");
    }
    match &support.range {
        Some(range) => {
            hash.str("original-range-boundaries");
            range.lower.frame(hash);
            range.upper.frame(hash);
            range.lower_difference.frame(hash);
            range.upper_difference.frame(hash);
        }
        None => {
            hash.str("no-original-range-boundaries");
        }
    }
    match &support.boundary {
        Some(boundary) => {
            hash.str("member_boundary");
            boundary.frame(hash);
        }
        None => {
            hash.str("no_member_boundary");
        }
    }
}

/// Prepare scalar support before numerical construction. Unimplemented application products
/// are explicit refusals; this private pause point has no alternate production consumer.
pub(super) fn prepare_scalar(
    syntax: &ResolvedSyntax,
    registry: &QuantityRegistry,
    facts: &PhysicalPreconditions,
    expected: &[Type],
    literals: &BTreeMap<NodeIx, Type>,
) -> Result<PhysicalSupport> {
    let scope = crate::PhysicalScope::default();
    let context = crate::TypeContext {
        admissions: None,
        formula_authority: None,
        quantities: registry,
        preconditions: facts,
        scope: &scope,
    };
    prepare_outputs(syntax, &context, Some(expected), literals, &BTreeMap::new())
}
fn prepare_outputs(
    syntax: &ResolvedSyntax,
    context: &crate::TypeContext<'_>,
    expected: Option<&[Type]>,
    literals: &BTreeMap<NodeIx, Type>,
    obligations: &BTreeMap<NodeIx, crate::expression::admission::PhysicalAdmission>,
) -> Result<PhysicalSupport> {
    if expected.is_some_and(|expected| expected.len() != syntax.outputs.len()) {
        return Err(invalid(SemanticId::NIL, "resolved output contract arity"));
    }
    let identity = syntax.identity()?;
    let ((nodes, order, outputs, boolean), premises) = pse_quantity::premises::capture(
        context.quantities,
        context.preconditions,
        || {
            let context = crate::TypeContext {
                admissions: None,
                formula_authority: context.formula_authority.clone(),
                quantities: context.quantities,
                preconditions: context.preconditions,
                scope: context.scope,
            };
            let mut producer = Producer {
                syntax: syntax.view(),
                context,
                body_authority: None,
                owner: SemanticId::NIL.into(),
                literal_contracts: literals,
                nodes: BTreeMap::new(),
                order: Vec::new(),
                active: std::collections::BTreeSet::new(),
                predicates: BTreeMap::new(),
                boolean: None,
                depth: 0,
            };
            let mut outputs = Vec::with_capacity(syntax.outputs.len());
            for (ordinal, root) in syntax.outputs.iter().enumerate() {
                let expected = expected.map(|expected| &expected[ordinal]);
                let value = producer.expression(*root, expected)?;
                let target = if let Some(expected) = expected {
                    if value.ty.physical_refinement() != expected.physical_refinement() {
                        return Err(invalid(
                            producer.at(*root),
                            "resolved output changes its required nominal physical role",
                        ));
                    }
                    producer.contract(expected, producer.at(*root))?
                } else {
                    value.contract.clone()
                };
                outputs.push(
                    OutputSupport::prepare(&value.contract, &target, producer.context.quantities)
                        .map_err(|error| invalid(producer.at(*root), error.to_string()))?,
                );
            }
            for predicate in &syntax.validity {
                producer.predicate(*predicate)?;
            }
            for (root, obligation) in obligations {
                let at = producer.at(*root);
                let Some(node) = producer.nodes.get(root) else {
                    continue;
                };
                let retained = obligation.specialize(
                    producer.context.quantities,
                    &BTreeMap::new(),
                    producer.context.preconditions,
                    producer.context.formula_authority.as_ref(),
                    at,
                )?;
                let actual=node.operation.as_ref().ok_or_else(||invalid(at,"source physical obligation lacks its demanded emitted arithmetic operation"))?;
                if retained.operands != actual.operands || retained.result != actual.result {
                    return Err(invalid(
                        at,
                        "emitted arithmetic differs from its exact original physical operands or result",
                    ));
                }
            }
            Ok::<_, crate::ModelingError>((
                producer.nodes,
                producer.order,
                outputs,
                producer.boolean,
            ))
        },
    )?;
    let premises = premises.ok_or_else(|| {
        invalid(
            SemanticId::NIL,
            "reached region lacks a complete actual physical premise capture",
        )
    })?;
    Ok(PhysicalSupport {
        syntax: identity,
        nodes,
        order,
        outputs,
        inputs: Vec::new(),
        premises: PhysicalEvidence::capture(premises),
        boolean,
        reduction: None,
        reduction_output: None,
    })
}

struct Producer<'a> {
    syntax: View<'a>,
    context: crate::TypeContext<'a>,
    body_authority: Option<&'a crate::PhysicalOperation>,
    owner: DeclarationId,
    literal_contracts: &'a BTreeMap<NodeIx, Type>,
    nodes: BTreeMap<NodeIx, NodeSupport>,
    order: Vec<NodeIx>,
    active: std::collections::BTreeSet<NodeIx>,
    predicates: BTreeMap<PredicateIx, bool>,
    boolean: Option<BooleanSupport>,
    depth: usize,
}
impl Producer<'_> {
    fn boolean(&mut self) -> Result<&BooleanSupport> {
        if self.boolean.is_none() {
            let registry = self.context.quantities;
            let at = self.owner;
            let quantity = registry.neutral_dimensionless().ok_or_else(|| {
                invalid(at, "registered neutral Boolean indicator type is absent")
            })?;
            let unit = registry
                .unit(
                    registry
                        .quantity_type(quantity)
                        .map_err(|error| invalid(at, error.to_string()))?
                        .canonical_unit,
                )
                .map_err(|error| invalid(at, error.to_string()))?;
            let literal = crate::expression::admission::admit_operation(
                &OpRequest::Literal {
                    unit,
                    context: LiteralContext::Explicit {
                        quantity_type: quantity,
                    },
                },
                &[],
                registry,
                self.context.preconditions,
                None,
                at,
            )?;
            let conversion = CanonicalConversionPlan::resolved(registry, quantity, unit)
                .map_err(|error| invalid(at, error.to_string()))?;
            let contract = literal.result.clone();
            let conditional = crate::expression::admission::admit_operation(
                &OpRequest::Conditional,
                &[contract.clone(), contract.clone()],
                registry,
                self.context.preconditions,
                None,
                at,
            )?;
            self.boolean = Some(BooleanSupport {
                contract,
                conversion,
                literal,
                conditional,
            });
        }
        self.boolean
            .as_ref()
            .ok_or_else(|| invalid(self.owner, "Boolean constructor admission was not retained"))
    }
    fn at(&self, slot: NodeIx) -> DeclarationId {
        self.syntax
            .node_sources
            .get(&slot)
            .map_or_else(|| SemanticId::NIL.into(), |source| source.field.declaration)
    }
    fn contract(&self, ty: &Type, at: DeclarationId) -> Result<ResolvedPhysicalContract> {
        if *ty == Type::Integer {
            return ResolvedPhysicalContract::named(
                self.context
                    .quantities
                    .neutral_dimensionless()
                    .ok_or_else(|| invalid(at, "neutral exact-integer physical contract absent"))?,
                pse_quantity::IndexSet::new(),
                self.context.quantities,
            )
            .map_err(|error| invalid(at, error.to_string()));
        }
        ty.quantity_scheme()
            .ok_or_else(|| invalid(at, "resolved numerical value has no physical scheme"))?
            .resolve_contract_with_evidence(
                self.context.quantities,
                &BTreeMap::new(),
                self.context.preconditions,
            )
            .map_err(|error| invalid(at, error.to_string()))
    }
    fn value(&self, ty: Type, at: DeclarationId) -> Result<NodeSupport> {
        let contract = self.contract(&ty, at)?;
        Ok(NodeSupport {
            ty,
            contract,
            operation: None,
            literal: None,
            boundary: None,
            call: None,
            range: None,
            exponent: None,
        })
    }
    fn alias(value: NodeSupport) -> NodeSupport {
        NodeSupport {
            ty: value.ty,
            contract: value.contract,
            operation: None,
            literal: None,
            boundary: None,
            call: None,
            range: None,
            exponent: None,
        }
    }
    fn operation(
        &self,
        request: OpRequest<'_>,
        operands: &[NodeSupport],
        at: DeclarationId,
    ) -> Result<NodeSupport> {
        let refinement = if matches!(request, OpRequest::Conditional) {
            if operands.len() != 2
                || operands[0].ty.physical_refinement() != operands[1].ty.physical_refinement()
            {
                return Err(invalid(
                    at,
                    "resolved conditional changes a nominal physical role",
                ));
            }
            operands[0].ty.physical_refinement().cloned()
        } else {
            crate::contextual::arithmetic_refinement(
                &request,
                &operands
                    .iter()
                    .map(|value| value.ty.clone())
                    .collect::<Vec<_>>(),
                &self.context,
                at,
            )?
        };
        let operation = crate::expression::admission::admit_operation(
            &request,
            &operands
                .iter()
                .map(|value| value.contract.clone())
                .collect::<Vec<_>>(),
            self.context.quantities,
            self.context.preconditions,
            self.context.formula_authority.as_ref(),
            at,
        )?;
        if !operation.conversions.is_empty() {
            return Err(invalid(
                at,
                "resolved input conversion requires an explicit admitted model operation",
            ));
        }
        let quantity = Scheme::from_contract(operation.result.clone());
        let ty = match refinement {
            Some(refinement) => Type::RefinedQuantity {
                quantity,
                refinement,
            },
            None => Type::Quantity(quantity),
        };
        Ok(NodeSupport {
            ty,
            contract: operation.result.clone(),
            operation: Some(operation),
            literal: None,
            boundary: None,
            call: None,
            range: None,
            exponent: None,
        })
    }
    fn expression(&mut self, slot: NodeIx, expected: Option<&Type>) -> Result<NodeSupport> {
        if let Some(support) = self.nodes.get(&slot) {
            return Ok(support.clone());
        }
        if self.depth >= 128 || !self.active.insert(slot) {
            return Err(invalid(
                self.at(slot),
                "resolved physical support is cyclic or exceeds its depth bound",
            ));
        }
        self.depth += 1;
        let result = self.expression_inner(slot, expected);
        self.depth -= 1;
        self.active.remove(&slot);
        let support = result?;
        self.nodes.insert(slot, support.clone());
        self.order.push(slot);
        Ok(support)
    }
    fn expression_inner(&mut self, slot: NodeIx, expected: Option<&Type>) -> Result<NodeSupport> {
        let syntax = self.syntax;
        let node = syntax
            .nodes
            .get(slot)
            .ok_or_else(|| invalid(self.at(slot), "resolved physical node outside its owner"))?;
        let at = self.at(slot);
        match node {
            Node::Number(number) => {
                let registry = self.context.quantities;
                let unit = if let Some(product) = &number.unit {
                    registry
                        .compose(product)
                        .map_err(|error| invalid(at, error.to_string()))?
                } else {
                    let quantity = registry
                        .neutral_dimensionless()
                        .ok_or_else(|| invalid(at, "neutral literal type absent"))?;
                    registry
                        .unit(
                            registry
                                .quantity_type(quantity)
                                .map_err(|error| invalid(at, error.to_string()))?
                                .canonical_unit,
                        )
                        .map_err(|error| invalid(at, error.to_string()))?
                        .clone()
                };
                let explicit = self.literal_contracts.get(&slot).or_else(|| {
                    if number.unit.is_none() {
                        None
                    } else {
                        expected
                    }
                });
                let context = if let Some(explicit) = explicit {
                    LiteralContext::Explicit {
                        quantity_type: self
                            .contract(explicit, at)?
                            .require_named()
                            .map_err(|error| invalid(at, error.to_string()))?,
                    }
                } else if number.unit.is_none() {
                    LiteralContext::Explicit {
                        quantity_type: registry
                            .neutral_dimensionless()
                            .ok_or_else(|| invalid(at, "neutral literal type absent"))?,
                    }
                } else {
                    LiteralContext::Free
                };
                let operation = crate::expression::admission::admit_operation(
                    &OpRequest::Literal {
                        unit: &unit,
                        context,
                    },
                    &[],
                    registry,
                    self.context.preconditions,
                    None,
                    at,
                )?;
                let conversion = CanonicalConversionPlan::resolved(
                    registry,
                    operation
                        .result
                        .require_named()
                        .map_err(|error| invalid(at, error.to_string()))?,
                    &unit,
                )
                .map_err(|error| invalid(at, error.to_string()))?;
                conversion
                    .apply(number.value)
                    .map_err(|error| invalid(at, error.to_string()))?;
                let quantity = Scheme::from_contract(operation.result.clone());
                let ty = match explicit
                    .or(expected)
                    .and_then(Type::physical_refinement)
                    .filter(|refinement| {
                        matches!(refinement, crate::PhysicalRefinement::Transfer { .. })
                    }) {
                    Some(refinement) => Type::RefinedQuantity {
                        quantity,
                        refinement: refinement.clone(),
                    },
                    None => Type::Quantity(quantity),
                };
                Ok(NodeSupport {
                    ty,
                    contract: operation.result.clone(),
                    operation: Some(operation),
                    literal: Some(LiteralSupport { conversion }),
                    boundary: None,
                    call: None,
                    range: None,
                    exponent: None,
                })
            }
            Node::Reference(Reference::Symbol(symbol_slot)) => {
                let symbol = syntax
                    .symbols
                    .at(*symbol_slot)
                    .ok_or_else(|| invalid(at, "resolved physical symbol absent"))?;
                if symbol.local_role() == LocalRole::Member && symbol.definition.is_none() {
                    return Err(invalid(
                        at,
                        "required resolved numerical member definition is absent",
                    ));
                }
                if symbol.local_role() != LocalRole::Member && symbol.definition.is_some() {
                    return Err(invalid(
                        at,
                        "resolved input role cannot also own a member definition",
                    ));
                }
                let mut result = self.value(symbol.ty.clone(), at)?;
                if let Some(definition) = symbol.definition {
                    let value = self.expression(definition, Some(&symbol.ty))?;
                    if value.ty.physical_refinement() != symbol.ty.physical_refinement() {
                        return Err(invalid(
                            at,
                            "resolved member definition changes its required nominal physical role",
                        ));
                    }
                    result.boundary = Some(
                        OutputSupport::prepare(
                            &value.contract,
                            &result.contract,
                            self.context.quantities,
                        )
                        .map_err(|error| invalid(at, error.to_string()))?,
                    );
                }
                if let Some(range) = syntax.member_ranges.get(symbol_slot) {
                    if range.layer != pse_model::generated::enums::ModelingValidityLayer::Closure {
                        return Err(invalid(
                            at,
                            "a scalar hard range must retain its original closure owner",
                        ));
                    }
                    let lower = self.expression(range.lower, Some(&symbol.ty))?;
                    let upper = self.expression(range.upper, Some(&symbol.ty))?;
                    if lower.ty.physical_refinement() != symbol.ty.physical_refinement()
                        || upper.ty.physical_refinement() != symbol.ty.physical_refinement()
                    {
                        return Err(invalid(
                            at,
                            "original hard range endpoints change the target's nominal physical role",
                        ));
                    }
                    let lower_boundary = OutputSupport::prepare(
                        &lower.contract,
                        &result.contract,
                        self.context.quantities,
                    )
                    .map_err(|error| invalid(at, error.to_string()))?;
                    let upper_boundary = OutputSupport::prepare(
                        &upper.contract,
                        &result.contract,
                        self.context.quantities,
                    )
                    .map_err(|error| invalid(at, error.to_string()))?;
                    let (lower_contract, _) = lower_boundary
                        .apply_contract(&lower.contract)
                        .map_err(|error| invalid(at, error.to_string()))?;
                    let (upper_contract, _) = upper_boundary
                        .apply_contract(&upper.contract)
                        .map_err(|error| invalid(at, error.to_string()))?;
                    let lower_difference = crate::expression::admission::admit_operation(
                        &OpRequest::Sub,
                        &[result.contract.clone(), lower_contract],
                        self.context.quantities,
                        self.context.preconditions,
                        None,
                        at,
                    )?;
                    let upper_difference = crate::expression::admission::admit_operation(
                        &OpRequest::Sub,
                        &[upper_contract, result.contract.clone()],
                        self.context.quantities,
                        self.context.preconditions,
                        None,
                        at,
                    )?;
                    result.range = Some(RangeSupport {
                        lower: lower_boundary,
                        upper: upper_boundary,
                        lower_difference,
                        upper_difference,
                    });
                }
                Ok(result)
            }
            Node::Reference(Reference::Lexical(binder)) => match syntax.binders.get(*binder) {
                Some(Binding::Value(value)) => self.expression(*value, expected).map(Self::alias),
                _ => Err(invalid(
                    at,
                    "lexical coordinate requires its complete indexed support product",
                )),
            },
            Node::Indicator(predicate) => {
                self.boolean()?;
                self.predicate(*predicate)?;
                self.value(
                    Type::Quantity(Scheme::Concrete(
                        self.context
                            .quantities
                            .neutral_dimensionless()
                            .ok_or_else(|| {
                                invalid(at, "predicate indicator has no admitted neutral quantity")
                            })?,
                    )),
                    at,
                )
            }
            Node::Neg(value) => {
                let value = self.expression(*value, expected)?;
                self.operation(OpRequest::Neg, &[value], at)
            }
            Node::Binary { op, lhs, rhs } => {
                let additive = matches!(op, BinaryOp::Add | BinaryOp::Sub);
                let left = self.expression(*lhs, if additive { expected } else { None })?;
                let right = self.expression(*rhs, if additive { Some(&left.ty) } else { None })?;
                let request = match op {
                    BinaryOp::Add => OpRequest::Add,
                    BinaryOp::Sub => OpRequest::Sub,
                    BinaryOp::Mul => OpRequest::Mul,
                    BinaryOp::Div => OpRequest::Div,
                    BinaryOp::Pow => OpRequest::Pow {
                        exponent: exponent(syntax, *rhs)
                            .map_or(Exponent::Symbolic, Exponent::Rational),
                    },
                };
                let mut result = self.operation(request, &[left, right], at)?;
                if *op == BinaryOp::Pow {
                    result.exponent = exponent(syntax, *rhs);
                }
                Ok(result)
            }
            Node::Call { function, args } => {
                use pse_quantity::functions::{Implementation, UnaryFunction};
                match function.implementation() {
                    Implementation::Unary(function) if args.len() == 1 => {
                        let value = self.expression(args[0], None)?;
                        let request = match function {
                            UnaryFunction::Exp => {
                                OpRequest::Transcendental(pse_quantity::Opcode::Exp)
                            }
                            UnaryFunction::Log => {
                                OpRequest::Transcendental(pse_quantity::Opcode::Log)
                            }
                            UnaryFunction::Sqrt => OpRequest::Sqrt,
                            UnaryFunction::Abs => OpRequest::Abs,
                            UnaryFunction::Sin => {
                                OpRequest::Transcendental(pse_quantity::Opcode::Sin)
                            }
                            UnaryFunction::Cos => {
                                OpRequest::Transcendental(pse_quantity::Opcode::Cos)
                            }
                        };
                        self.operation(request, &[value], at)
                    }
                    Implementation::Extremum { .. } if args.len() == 2 => {
                        let left = self.expression(args[0], None)?;
                        let right = self.expression(args[1], None)?;
                        self.operation(OpRequest::Conditional, &[left, right], at)
                    }
                    _ => Err(invalid(
                        at,
                        "resolved builtin constructor or arity unavailable",
                    )),
                }
            }
            Node::Conditional {
                guard,
                then,
                otherwise,
            } => {
                self.predicate(*guard)?;
                let yes = self.expression(*then, expected)?;
                let no = self.expression(*otherwise, Some(&yes.ty))?;
                self.operation(OpRequest::Conditional, &[yes, no], at)
            }
            Node::Let { body, .. } => self.expression(*body, expected).map(Self::alias),
            Node::Evidence { value, evidence } => {
                let usage = syntax
                    .evidence
                    .get(*evidence)
                    .ok_or_else(|| invalid(at, "physical evidence capture is outside its owner"))?;
                for predicate in &usage.predicates {
                    self.predicate(*predicate)?;
                }
                for input in &usage.inputs {
                    self.expression(*input, None)?;
                }
                self.expression(*value, expected).map(Self::alias)
            }
            Node::Sequence {
                value,
                prerequisites,
            } => {
                for prerequisite in prerequisites {
                    self.expression(*prerequisite, None)?;
                }
                self.expression(*value, expected).map(Self::alias)
            }
            Node::Apply { function, args } => self.application(*function, args, &[]),
            Node::Partial {
                function,
                args,
                wrt,
            } => self.application(*function, args, wrt),
            Node::Provider { .. }
            | Node::Reference(Reference::Indexed { .. })
            | Node::Reduce { .. }
            | Node::Fold { .. }
            | Node::Derivative { .. } => Err(invalid(
                at,
                "resolved application requires its selected provider or indexed support product",
            )),
        }
    }
    fn application(
        &mut self,
        slot: FunctionIx,
        args: &[NodeIx],
        wrt: &[usize],
    ) -> Result<NodeSupport> {
        let syntax = self.syntax;
        let product = &syntax
            .functions
            .get(slot)
            .ok_or_else(|| {
                invalid(
                    SemanticId::NIL,
                    "selected physical function is outside its owner",
                )
            })?
            .product;
        let at = product.declaration;
        if args.len() != product.signature.len() {
            return Err(invalid(
                at,
                "selected physical function input arity differs",
            ));
        }
        let mut boundaries = Vec::with_capacity(args.len());
        for (arg, formal) in args.iter().zip(&product.signature) {
            let actual = self.expression(*arg, Some(formal))?;
            if actual.ty.physical_refinement() != formal.physical_refinement() {
                return Err(invalid(
                    at,
                    "selected function actual changes its nominal physical role",
                ));
            }
            let expected = self.contract(formal, at)?;
            boundaries.push(
                OutputSupport::prepare(&actual.contract, &expected, self.context.quantities)
                    .map_err(|error| invalid(at, error.to_string()))?,
            );
        }
        let nominal = product.result.clone();
        let mut body_result = None;
        let ty = if wrt.is_empty() {
            let consumed = self.body_authority.map_or_else(
                || nominal.clone(),
                |operation| operation.body_function_result(&nominal).into_owned(),
            );
            if consumed != nominal {
                let relationship = crate::derived::Relationship::ReconstructionResultView {
                    owner: self.owner,
                    callee: product.identity,
                };
                let boundary = pse_quantity::AdmittedOutputBoundary::declared_contract(
                    relationship.identity(),
                    self.contract(&nominal, at)?,
                    self.contract(&consumed, at)?,
                )
                .map_err(|error| invalid(at, error.to_string()))?;
                body_result = Some(BodyInputSupport {
                    relationship,
                    boundary,
                });
            }
            consumed
        } else {
            if product.result.physical_refinement().is_some()
                || product
                    .signature
                    .iter()
                    .any(|argument| argument.physical_refinement().is_some())
            {
                return Err(invalid(
                    at,
                    "differentiate the physical reconstruction rather than a nominal role",
                ));
            }
            let mut result = product
                .result
                .quantity_scheme()
                .cloned()
                .ok_or_else(|| invalid(at, "physical partial result required"))?;
            for ordinal in wrt {
                let argument = product
                    .signature
                    .get(*ordinal)
                    .and_then(Type::quantity_scheme)
                    .ok_or_else(|| {
                        invalid(
                            at,
                            "partial formal ordinal is outside the actual physical signature",
                        )
                    })?;
                result = Scheme::Quotient(
                    Box::new(Scheme::Delta(Box::new(result))),
                    Box::new(Scheme::Delta(Box::new(argument.clone()))),
                );
            }
            Type::Quantity(Scheme::from_contract(
                result
                    .resolve_contract_with_evidence(
                        self.context.quantities,
                        &BTreeMap::new(),
                        self.context.preconditions,
                    )
                    .map_err(|error| invalid(at, error.to_string()))?,
            ))
        };
        let mut support = self.value(ty, at)?;
        let partial = if wrt.is_empty() {
            None
        } else {
            let result = self.contract(&product.result, at)?;
            let arguments = wrt
                .iter()
                .map(|ordinal| {
                    self.contract(
                        product.signature.get(*ordinal).ok_or_else(|| {
                            invalid(at, "partial ordinal outside actual signature")
                        })?,
                        at,
                    )
                })
                .collect::<Result<Vec<_>>>()?;
            let partial = pse_quantity::resolved::infer_partial(
                &result,
                &arguments,
                self.context.quantities,
                self.context.preconditions,
            )
            .map_err(|error| invalid(at, error.to_string()))?;
            if partial.result != support.contract {
                return Err(invalid(
                    at,
                    "selected partial admission differs from its declared complete result",
                ));
            }
            Some(partial)
        };
        support.call = Some(CallSupport {
            function: slot,
            nominal_result: self.contract(&product.result, at)?,
            wrt: wrt.to_vec(),
            inputs: boundaries,
            body_result,
            partial,
        });
        Ok(support)
    }
    fn predicate(&mut self, slot: PredicateIx) -> Result<()> {
        if let Some(complete) = self.predicates.get(&slot) {
            return if *complete {
                Ok(())
            } else {
                Err(invalid(SemanticId::NIL, "resolved physical guard cycle"))
            };
        }
        self.predicates.insert(slot, false);
        if self.depth >= 128 {
            return Err(invalid(
                SemanticId::NIL,
                "resolved physical guard depth exceeded",
            ));
        }
        self.depth += 1;
        let result = self.predicate_inner(slot);
        self.depth -= 1;
        if result.is_ok() {
            self.predicates.insert(slot, true);
        }
        result
    }
    fn predicate_inner(&mut self, slot: PredicateIx) -> Result<()> {
        self.boolean()?;
        let syntax = self.syntax;
        match syntax
            .predicates
            .get(slot)
            .ok_or_else(|| invalid(SemanticId::NIL, "resolved guard outside its owner"))?
        {
            ResolvedPredicate::Compare { lhs, rhs, .. } => {
                let left = self.expression(*lhs, None)?;
                let right = self.expression(*rhs, Some(&left.ty))?;
                if !left.contract.same_meaning(&right.contract)
                    || left.contract.indices() != right.contract.indices()
                    || left.ty.physical_refinement() != right.ty.physical_refinement()
                {
                    return Err(invalid(
                        self.at(*lhs),
                        "resolved comparison complete physical contracts differ",
                    ));
                }
            }
            ResolvedPredicate::And(lhs, rhs) | ResolvedPredicate::Or(lhs, rhs) => {
                self.predicate(*lhs)?;
                self.predicate(*rhs)?;
            }
            ResolvedPredicate::Not(value) => self.predicate(*value)?,
            ResolvedPredicate::Bool(_) => {}
            ResolvedPredicate::In { .. } | ResolvedPredicate::Atom(_) | ResolvedPredicate::Null => {
                return Err(invalid(
                    SemanticId::NIL,
                    "resolved guard requires its complete Boolean or finite membership product",
                ));
            }
        }
        Ok(())
    }
}
fn exponent(syntax: View<'_>, slot: NodeIx) -> Option<pse_quantity::Ratio> {
    exponent_at(syntax, slot, 0)
}
fn exponent_at(syntax: View<'_>, slot: NodeIx, depth: usize) -> Option<pse_quantity::Ratio> {
    if depth >= 128 {
        return None;
    }
    match syntax.nodes.get(slot)? {
        Node::Number(number) if number.unit.is_none() => number
            .integer()
            .and_then(|integer| i16::try_from(integer).ok())
            .and_then(|integer| pse_quantity::Ratio::new(i32::from(integer), 1).ok()),
        Node::Neg(value) => exponent_at(syntax, *value, depth + 1)?
            .checked_mul(pse_quantity::Ratio::new(-1, 1).ok()?)
            .ok(),
        Node::Binary {
            op: BinaryOp::Div,
            lhs,
            rhs,
        } => {
            let numerator = exponent_at(syntax, *lhs, depth + 1)?;
            let denominator = exponent_at(syntax, *rhs, depth + 1)?;
            if numerator.den() != 1 || denominator.den() != 1 {
                return None;
            }
            pse_quantity::Ratio::new(i32::from(numerator.num()), i32::from(denominator.num())).ok()
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct EmptyNames;
    impl Names for EmptyNames {
        fn selection(&self, _: &dsl::Path) -> Option<Selection> {
            None
        }
        fn function(&self, _: &dsl::Path) -> Option<FunctionIx> {
            None
        }
        fn provider(&self, _: &str) -> Option<ProviderSelection> {
            None
        }
    }
    fn syntax(text: &str) -> ResolvedSyntax {
        let source = crate::expression::occurrences::OccurrenceKey {
            declaration: SemanticId::from_bytes([1; 16]).into(),
            role: "function.body".into(),
            position: 0,
        };
        let mut resolver = Resolver::new(ResolvedSyntax::default(), &EmptyNames, source, 100);
        resolver.output(&dsl::parse_expr(text).unwrap()).unwrap();
        resolver.finish()
    }
    #[test]
    fn resolved_support_retains_actual_operations_literal_conversion_and_read_premises() {
        let (registry, _) = crate::kernel_types::physical();
        let facts = PhysicalPreconditions::new(vec![]).unwrap();
        let scalar = registry.neutral_dimensionless().unwrap();
        let syntax = syntax("sqrt(2^2)+1");
        let support = PhysicalSupport::prepare_scalar(
            &syntax,
            &registry,
            &facts,
            &[Type::Quantity(Scheme::Concrete(scalar))],
            &BTreeMap::new(),
        )
        .unwrap();
        assert_eq!(support.syntax(), syntax.identity().unwrap());
        let root = support.node(syntax.outputs[0]).unwrap();
        assert!(root.operation.is_some());
        assert_eq!(root.contract.require_named().unwrap(), scalar);
        assert!(support.nodes.values().any(|node| {
            node.literal
                .as_ref()
                .is_some_and(|literal| literal.conversion.quantity() == scalar)
        }));
        assert!(support.premises().unwrap().matches(&registry, &facts));
        assert!(support.node(NodeIx::from(syntax.nodes.len())).is_err());
    }
    #[test]
    fn resolved_support_checks_original_branches_and_required_output_contracts() {
        let (registry, names) = crate::kernel_types::physical();
        let facts = PhysicalPreconditions::new(vec![]).unwrap();
        let scalar = Type::Quantity(Scheme::Concrete(registry.neutral_dimensionless().unwrap()));
        let length = Type::Quantity(Scheme::Concrete(names["Length"]));
        assert!(
            PhysicalSupport::prepare_scalar(
                &syntax("1"),
                &registry,
                &facts,
                std::slice::from_ref(&length),
                &BTreeMap::new()
            )
            .is_err()
        );
        let original = syntax("if false then 1 {m} else 1 {s}");
        assert!(
            PhysicalSupport::prepare_scalar(
                &original,
                &registry,
                &facts,
                &[length],
                &BTreeMap::new()
            )
            .is_err()
        );
        assert!(
            PhysicalSupport::prepare_scalar(&syntax("1"), &registry, &facts, &[], &BTreeMap::new())
                .is_err()
        );
        let mut cycle = syntax("1");
        let root = cycle.outputs[0];
        cycle.nodes[root] = Node::Neg(root);
        assert!(
            PhysicalSupport::prepare_scalar(&cycle, &registry, &facts, &[scalar], &BTreeMap::new())
                .is_err()
        );
    }
}

/// Captured read evidence or a strict caller-qualified restoration. The distinction cannot
/// be erased: immutable recipe reconstruction never manufactures a live registry context.
#[derive(Clone, Debug)]
pub struct PhysicalEvidence {
    identity: ContentHash,
    live: Option<pse_quantity::premises::PhysicalPremises>,
}
/// Opaque exact captured premise identity. Decode alone supplies no qualified authority.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhysicalEvidenceRecord {
    identity: ContentHash,
}
impl PhysicalEvidence {
    /// The sole normal issuer consumes the complete actual captured questions and answers.
    pub fn capture(premises: pse_quantity::premises::PhysicalPremises) -> Self {
        let mut hash = FramedHasher::new(pse_ids::Frame::ModelingResolvedBodyV1);
        hash.str("captured-physical-premises");
        premises.frame(&mut hash);
        Self {
            identity: hash.finish_hash(),
            live: Some(premises),
        }
    }
    /// Complete owner-computed premise identity, stable under qualified restoration.
    pub fn identity(&self) -> ContentHash {
        self.identity
    }
    /// Require an actual preparation capture; replay receipts are not live observations.
    pub fn live(&self) -> Result<&pse_quantity::premises::PhysicalPremises> {
        self.live.as_ref().ok_or_else(||invalid(SemanticId::NIL,"qualified restored support cannot supply live preparation context or Salsa observations"))
    }
    /// Capture the exact opaque identity record; the enclosing complete record gates restore.
    pub fn record(&self) -> PhysicalEvidenceRecord {
        PhysicalEvidenceRecord {
            identity: self.identity,
        }
    }
    /// Stable complete semantic premise framing.
    pub fn frame(&self, hash: &mut FramedHasher) {
        hash.str("physical-premise-identity").hash(&self.identity);
    }
    /// Known retained extent including actual live captured owners.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>().saturating_add(
            self.live
                .as_ref()
                .map_or(0, pse_quantity::premises::PhysicalPremises::retained_bytes),
        )
    }
}
impl PhysicalEvidenceRecord {
    /// Exact immutable record membership is mandatory. This never creates captured facts.
    pub fn restore(&self) -> Result<PhysicalEvidence> {
        pse_quantity::resolved::receipts::require_record(self)
            .map_err(|error| invalid(SemanticId::NIL, error.to_string()))?;
        Ok(PhysicalEvidence {
            identity: self.identity,
            live: None,
        })
    }
}
