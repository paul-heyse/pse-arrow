// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One direct construction walk over producer-issued typed graph edges and physical facts.
#[path = "direct_functions.rs"]
mod functions;
use super::{AdmittedBody, LocalOccurrence as Occurrence, SelectedProviders};
use functions::WitnessPass;
use pse_authoring::dsl::{BinaryOp, CompareOp};
use pse_ids::{ContentHash, FramedHasher, SemanticId};
use pse_math::{
    MathError,
    binding::BodySpec,
    guarded::Comparison,
    typed::{Binary, BodyBuilder, BodyLimits, Guard, TypedValue},
};
use pse_modeling::resolved::{
    BinderIx, Binding, Node, NodeIx, PhysicalSupport, PredicateIx, Reference, ResolvedPredicate,
    ResolvedSyntax, SymbolIx,
};
use pse_quantity::{QuantityRegistry, infer::InvariantChecker};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

/// Direct immutable preparation inputs. No source syntax, function-name lookup or expected-type hints.
#[derive(Debug)]
pub struct Request<'a> {
    /// Current diagnostic definition, outside reusable arithmetic identity.
    pub definition: SemanticId,
    /// Compact selected direct construction owner.
    pub syntax: &'a ResolvedSyntax,
    /// Exact ordered independent scalar layout.
    pub inputs: &'a [SymbolIx],
    /// Producer-issued actual consumed physical contracts and operations.
    pub support: &'a PhysicalSupport,
    /// Exact actual provider/source-contract joins.
    pub providers: &'a SelectedProviders,
    /// Complete observed physical context identity.
    pub physical: ContentHash,
    /// Complete structural specialization identity.
    pub structure: ContentHash,
    /// Finite numerical construction budget.
    pub limits: BodyLimits,
}
fn contract(error: impl std::fmt::Display) -> MathError {
    MathError::Contract(error.to_string())
}
impl Request<'_> {
    /// Admit the exact producer product and retain only a bounded encoded recipe for replay.
    pub fn admit(
        &self,
        registry: &QuantityRegistry,
        checker: &dyn InvariantChecker,
        cancelled: &Arc<AtomicBool>,
    ) -> Result<AdmittedBody, MathError> {
        let ((mut body, proofs), receipts) = pse_quantity::resolved::receipts::capture(|| {
            pse_math::typed::receipts::capture(|| self.admit_inner(registry, checker, cancelled))
        })?;
        body.portable = match super::portable::capture(self, &body, receipts, proofs) {
            Ok(payload) => Some(payload),
            Err(MathError::ByteLimit {
                resource: "portable body payload",
                ..
            }) => None,
            Err(error) => return Err(error),
        };
        Ok(body)
    }
    /// Construct exactly this selected direct graph, consuming its already admitted facts.
    pub(super) fn admit_inner(
        &self,
        registry: &QuantityRegistry,
        checker: &dyn InvariantChecker,
        cancelled: &Arc<AtomicBool>,
    ) -> Result<AdmittedBody, MathError> {
        let _phase = pse_diagnostics::observation::phase("compiler.typed_body_lowering");
        if self.syntax.outputs.is_empty() {
            return Err(MathError::Contract("empty resolved output group".into()));
        }
        if self.support.syntax() != self.syntax.identity().map_err(contract)? {
            return Err(MathError::Contract(
                "resolved support belongs to a different exact syntax product".into(),
            ));
        }
        if self.inputs.len() > self.limits.slots
            || self.inputs.iter().copied().collect::<BTreeSet<_>>().len() != self.inputs.len()
        {
            return Err(MathError::Limit("resolved formal layout"));
        }
        if self
            .support
            .inputs()
            .iter()
            .map(|(slot, _)| *slot)
            .ne(self.inputs.iter().copied())
        {
            return Err(contract(
                "resolved formal layout differs from its exact prepared input contracts",
            ));
        }
        let mut builder = BodyBuilder::new(
            pse_math::context()?,
            registry,
            checker,
            self.inputs.len(),
            self.limits,
        )?;
        let mut lower = Lower::root(self, cancelled.as_ref())?;
        // Explicit generic root assertions have a source occurrence, without inventing
        // a scientific form/data/closure lineage. Function/range owners issue those.
        let mut prerequisites = Vec::with_capacity(self.syntax.validity.len());
        for guard in &self.syntax.validity {
            let source = lower.source(true, guard.position())?;
            let mut local = lower.fork();
            let mut predicate = local.indicator(*guard, &mut builder)?;
            builder.require(
                &mut predicate,
                pse_math::guarded::Condition::Positive,
                pse_kernels::DerivativeOrder::Value,
                source,
            )?;
            lower.occurrences.extend(local.occurrences);
            prerequisites.push(predicate);
        }
        let mut values = Vec::with_capacity(self.syntax.outputs.len());
        for (ordinal, root) in self.syntax.outputs.iter().enumerate() {
            let value = lower.node(*root, &mut builder)?;
            let mut value = builder.contract_boundary_admitted(
                value,
                self.support.output(ordinal).map_err(contract)?,
            )?;
            for prerequisite in &prerequisites {
                value = builder.with_prerequisite(value, prerequisite);
            }
            values.push(value);
        }
        let quantities = values
            .iter()
            .map(TypedValue::quantity)
            .collect::<Result<Vec<_>, _>>()?;
        let mut hash = FramedHasher::new(pse_ids::Frame::MathTypedDefinitionV10);
        hash.str("direct-resolved-construction")
            .hash(&self.support.syntax());
        self.support.frame(&mut hash);
        self.providers.frame(&mut hash);
        self.syntax
            .frame_input_layout(self.inputs, &mut hash)
            .map_err(contract)?;
        builder.frame_admissions(&mut hash);
        let admissions = builder.admission_identity();
        let ranges = self.support.checked_ranges(self.syntax).map_err(contract)?;
        let members = ranges.iter().map(|range| range.member).collect();
        let sources = ranges.iter().map(|range| range.source).collect();
        let math = Arc::new(
            builder
                .prepare(&values)?
                .with_checked_members(members)?
                .with_checked_sources(sources)?
                .with_occurrence_tokens(
                    self.syntax
                        .occurrence_tokens(self.limits.occurrences)
                        .map_err(contract)?,
                )?,
        );
        let spec = BodySpec {
            definition: hash.finish_hash(),
            structure: self.structure,
            physical: self.physical,
            admissions,
            providers: math
                .providers()
                .iter()
                .map(pse_kernels::ProviderSpec::identity)
                .collect(),
            policy: pse_math::binding::guarded_real_policy(),
        };
        Ok(AdmittedBody {
            semantic_identity: None,
            portable: None,
            spec,
            quantities,
            occurrences: lower.occurrences,
            math,
        })
    }
}
/// Caches are local to an actual lazy region; branch forks cannot publish branch-local slots outward.
#[derive(Clone)]
pub(super) struct Lower<'a> {
    pub(super) syntax: &'a ResolvedSyntax,
    pub(super) support: &'a PhysicalSupport,
    pub(super) providers: &'a SelectedProviders,
    pub(super) cancelled: &'a AtomicBool,
    pub(super) inputs: BTreeMap<SymbolIx, usize>,
    pub(super) arguments: BTreeMap<SymbolIx, TypedValue>,
    pub(super) nodes: BTreeMap<NodeIx, TypedValue>,
    pub(super) lexical: BTreeMap<BinderIx, TypedValue>,
    pub(super) active: BTreeSet<NodeIx>,
    pub(super) path: Vec<usize>,
    pub(super) node_ordinals: BTreeMap<NodeIx, usize>,
    pub(super) predicate_ordinals: BTreeMap<PredicateIx, usize>,
    pub(super) occurrences: Vec<Occurrence>,
    pub(super) ranges: BTreeMap<SymbolIx, pse_modeling::resolved::CheckedRange>,
    pub(super) depth: usize,
    pub(super) limit: usize,
    witness: Option<WitnessPass>,
}
impl<'a> Lower<'a> {
    fn root(request: &'a Request<'a>, cancelled: &'a AtomicBool) -> Result<Self, MathError> {
        let layout = request.syntax.canonical_layout().map_err(contract)?;
        Ok(Self {
            syntax: request.syntax,
            support: request.support,
            providers: request.providers,
            cancelled,
            inputs: request
                .inputs
                .iter()
                .enumerate()
                .map(|(ordinal, slot)| (*slot, ordinal))
                .collect(),
            arguments: BTreeMap::new(),
            nodes: BTreeMap::new(),
            lexical: BTreeMap::new(),
            active: BTreeSet::new(),
            path: vec![],
            node_ordinals: layout
                .nodes
                .into_iter()
                .enumerate()
                .map(|(ordinal, slot)| (slot, ordinal))
                .collect(),
            predicate_ordinals: layout
                .predicates
                .into_iter()
                .enumerate()
                .map(|(ordinal, slot)| (slot, ordinal))
                .collect(),
            occurrences: vec![],
            ranges: request
                .support
                .checked_ranges(request.syntax)
                .map_err(contract)?
                .into_iter()
                .map(|range| (range.symbol, range))
                .collect(),
            depth: 0,
            limit: request.limits.occurrences,
            witness: None,
        })
    }
    pub(super) fn fork(&self) -> Self {
        let mut fork = self.clone();
        fork.occurrences.clear();
        fork
    }
    pub(super) fn source(
        &mut self,
        predicate: bool,
        position: usize,
    ) -> Result<SemanticId, MathError> {
        if self.cancelled.load(Ordering::Relaxed) {
            return Err(MathError::Cancelled);
        }
        if self.occurrences.len() >= self.limit || self.depth >= 128 {
            return Err(MathError::Limit(
                "resolved construction occurrences or depth",
            ));
        }
        let ordinal = if predicate {
            *self
                .predicate_ordinals
                .get(&PredicateIx::from(position))
                .ok_or_else(|| contract("predicate is outside canonical source correspondence"))?
        } else {
            *self
                .node_ordinals
                .get(&NodeIx::from(position))
                .ok_or_else(|| contract("operation is outside canonical source correspondence"))?
        };
        let id = pse_modeling::resolved::occurrence_token(&self.path, predicate, ordinal);
        self.occurrences.push(Occurrence { id });
        Ok(id)
    }
    pub(super) fn node(
        &mut self,
        root: NodeIx,
        builder: &mut BodyBuilder<'_>,
    ) -> Result<TypedValue, MathError> {
        if let Some(value) = self.nodes.get(&root) {
            return Ok(value.clone());
        }
        if !self.active.insert(root) {
            return Err(contract("cyclic resolved numerical definition or range"));
        }
        self.depth += 1;
        let value = self.node_inner(root, builder);
        self.depth -= 1;
        self.active.remove(&root);
        let value = value?;
        self.nodes.insert(root, value.clone());
        Ok(value)
    }
    fn node_inner(
        &mut self,
        root: NodeIx,
        builder: &mut BodyBuilder<'_>,
    ) -> Result<TypedValue, MathError> {
        let source = self.source(false, root.position())?;
        let support = self.support.node(root).map_err(contract)?.clone();
        let operation = || {
            support
                .operation
                .clone()
                .ok_or_else(|| contract("demanded operation lacks its exact producer admission"))
        };
        match self
            .syntax
            .nodes
            .get(root)
            .ok_or_else(|| contract("demanded operation is outside its owner"))?
            .clone()
        {
            Node::Number(number) => builder.literal_admitted(
                number.value,
                &support
                    .literal
                    .as_ref()
                    .ok_or_else(|| contract("resolved literal lacks conversion support"))?
                    .conversion,
                operation()?,
                source,
            ),
            Node::Reference(Reference::Symbol(slot)) => {
                let mut value = if let Some(argument) = self.arguments.get(&slot) {
                    argument.clone()
                } else if let Some(definition) = self
                    .syntax
                    .symbols
                    .at(slot)
                    .ok_or_else(|| contract("resolved scalar is outside its owner"))?
                    .definition
                {
                    let value = self.node(definition, builder)?;
                    let value = builder.contract_boundary_admitted(
                        value,
                        support.boundary.as_ref().ok_or_else(|| {
                            contract("defined member lacks exact declared boundary")
                        })?,
                    )?;
                    builder.bind(value)?
                } else {
                    let input = self.support.input(slot).map_err(contract)?;
                    if input != &support.contract {
                        return Err(contract(
                            "demanded input differs from its exact prepared contract",
                        ));
                    }
                    builder.input_admitted(
                        *self.inputs.get(&slot).ok_or_else(|| {
                            contract("demanded independent scalar lacks its exact formal slot")
                        })?,
                        input.clone(),
                        source,
                    )?
                };
                if let Some(range) = &support.range {
                    let endpoints = self.syntax.member_ranges.get(&slot).ok_or_else(|| {
                        contract("range admission lost actual original endpoints")
                    })?;
                    let lower = self.node(endpoints.lower, builder)?;
                    let upper = self.node(endpoints.upper, builder)?;
                    let lower = builder.contract_boundary_admitted(lower, &range.lower)?;
                    let upper = builder.contract_boundary_admitted(upper, &range.upper)?;
                    let tokens = self.ranges.get(&slot).ok_or_else(|| {
                        contract("demanded range lacks exact checked member and source tokens")
                    })?;
                    value = builder.within_range_admitted(
                        value,
                        lower,
                        upper,
                        range.lower_difference.clone(),
                        range.upper_difference.clone(),
                        Arc::new(pse_model::diagnostic::ValidityLineage {
                            layer: endpoints.layer,
                            source: tokens.source,
                            form: None,
                            sets: vec![],
                            variables: vec![],
                            members: vec![tokens.member],
                        }),
                    )?;
                }
                Ok(value)
            }
            Node::Reference(Reference::Lexical(slot)) => {
                if let Some(value) = self.lexical.get(&slot) {
                    return Ok(value.clone());
                }
                match self
                    .syntax
                    .binders
                    .get(slot)
                    .ok_or_else(|| contract("lexical binding is outside its owner"))?
                {
                    Binding::Value(root) => {
                        let value = self.node(*root, builder)?;
                        let value = builder.bind(value)?;
                        self.lexical.insert(slot, value.clone());
                        Ok(value)
                    }
                    _ => Err(contract(
                        "unselected finite lexical coordinate reached scalar construction",
                    )),
                }
            }
            Node::Neg(root) => {
                let value = self.node(root, builder)?;
                builder.negate_admitted(value, operation()?, source)
            }
            Node::Binary { op, lhs, rhs } => {
                let left = self.node(lhs, builder)?;
                let right = self.node(rhs, builder)?;
                let op = match op {
                    BinaryOp::Add => Binary::Add,
                    BinaryOp::Sub => Binary::Sub,
                    BinaryOp::Mul => Binary::Mul,
                    BinaryOp::Div => Binary::Div,
                    BinaryOp::Pow => Binary::Pow,
                };
                builder.binary_admitted(op, left, right, support.exponent, operation()?, source)
            }
            Node::Call { function, args } => match function.implementation() {
                pse_math::Implementation::Unary(_) if args.len() == 1 => {
                    let value = self.node(args[0], builder)?;
                    builder.unary_admitted(function, value, operation()?, source)
                }
                pse_math::Implementation::Extremum { minimum } if args.len() == 2 => {
                    let left = self.node(args[0], builder)?;
                    let right = self.node(args[1], builder)?;
                    builder.extremum_admitted(minimum, left, right, operation()?, source)
                }
                _ => Err(contract(
                    "resolved builtin requires its admitted scalar constructor",
                )),
            },
            Node::Conditional {
                guard,
                then,
                otherwise,
            } => {
                let guard = self.guard(guard, builder)?;
                let mut yes = self.fork();
                let mut no = self.fork();
                let value = builder.conditional_admitted(
                    guard,
                    |builder| yes.node(then, builder),
                    |builder| no.node(otherwise, builder),
                    operation()?,
                    source,
                )?;
                self.occurrences.extend(yes.occurrences);
                self.occurrences.extend(no.occurrences);
                Ok(value)
            }
            Node::Indicator(predicate) => self.indicator(predicate, builder),
            Node::Let { body, .. } => self.node(body, builder),
            Node::Sequence {
                value,
                prerequisites,
            } => {
                let prerequisites = prerequisites
                    .into_iter()
                    .map(|root| self.node(root, builder))
                    .collect::<Result<Vec<_>, _>>()?;
                let mut value = self.node(value, builder)?;
                for prerequisite in prerequisites {
                    value = builder.with_prerequisite(value, &prerequisite);
                }
                Ok(value)
            }
            Node::Evidence { value, evidence } => {
                let usage = self
                    .syntax
                    .evidence
                    .get(evidence)
                    .ok_or_else(|| contract("scientific evidence is outside its exact owner"))?
                    .clone();
                let mut local = self.fork();
                let assumption = builder.applicability(Arc::new(usage.node), |builder| {
                    let predicates = usage
                        .predicates
                        .into_iter()
                        .map(|predicate| local.indicator(predicate, builder))
                        .collect::<Result<Vec<_>, _>>()?;
                    let inputs = usage
                        .inputs
                        .into_iter()
                        .map(|root| local.node(root, builder))
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok((predicates, inputs))
                })?;
                self.occurrences.extend(local.occurrences);
                let value = self.node(value, builder)?;
                Ok(builder.with_assumption(value, &assumption))
            }
            Node::Apply { function, args } => {
                self.function(root, function, &args, &[], builder, source)
            }
            Node::Partial {
                function,
                args,
                wrt,
            } => self.function(root, function, &args, &wrt, builder, source),
            Node::Provider { selection, args } => {
                let ordinal = *self.node_ordinals.get(&root).ok_or_else(|| {
                    contract("direct provider lacks its canonical operation coordinate")
                })?;
                let prepared = self
                    .providers
                    .direct(self.syntax.identity().map_err(contract)?, ordinal)?;
                if prepared.descriptor().spec().identity() != selection.identity {
                    return Err(contract(
                        "direct provider descriptor differs from its prepared application",
                    ));
                }
                let args = args
                    .into_iter()
                    .map(|root| self.node(root, builder))
                    .collect::<Result<Vec<_>, _>>()?;
                let value = builder
                    .provider_partial_admitted(prepared, &args, source)?
                    .get(selection.output)
                    .cloned()
                    .ok_or_else(|| contract("direct provider selected output disappeared"))?;
                builder.contract_boundary_admitted(
                    value,
                    support.boundary.as_ref().ok_or_else(|| {
                        contract("direct provider lacks its original declared result boundary")
                    })?,
                )
            }
            Node::Reference(Reference::Indexed { .. })
            | Node::Reduce { .. }
            | Node::Fold { .. }
            | Node::Derivative { .. } => Err(contract(
                "unselected finite or continuous source operation reached scalar construction",
            )),
        }
    }
    fn boolean(
        &self,
        value: bool,
        builder: &mut BodyBuilder<'_>,
        source: SemanticId,
    ) -> Result<TypedValue, MathError> {
        let support = self.support.boolean().map_err(contract)?;
        builder.literal_admitted(
            if value { 1.0 } else { 0.0 },
            &support.conversion,
            support.literal.clone(),
            source,
        )
    }
    fn guard(
        &mut self,
        root: PredicateIx,
        builder: &mut BodyBuilder<'_>,
    ) -> Result<Guard, MathError> {
        let source = self.source(true, root.position())?;
        match self
            .syntax
            .predicates
            .get(root)
            .ok_or_else(|| contract("resolved guard is outside its owner"))?
            .clone()
        {
            ResolvedPredicate::Bool(value) => builder.fixed_guard(value, source),
            ResolvedPredicate::Compare { op, lhs, rhs } => {
                let left = self.node(lhs, builder)?;
                let right = self.node(rhs, builder)?;
                let (op, left, right) = match op {
                    CompareOp::Lt => (Comparison::Lt, left, right),
                    CompareOp::Le => (Comparison::Le, left, right),
                    CompareOp::Gt => (Comparison::Lt, right, left),
                    CompareOp::Ge => (Comparison::Le, right, left),
                    CompareOp::Eq => (Comparison::Eq, left, right),
                    CompareOp::NotEq => (Comparison::Ne, left, right),
                };
                builder.compare(op, &left, &right, source)
            }
            _ => {
                let value = self.indicator(root, builder)?;
                let one = self.boolean(true, builder, source)?;
                builder.compare(Comparison::Eq, &value, &one, source)
            }
        }
    }
    fn indicator(
        &mut self,
        root: PredicateIx,
        builder: &mut BodyBuilder<'_>,
    ) -> Result<TypedValue, MathError> {
        let source = self.source(true, root.position())?;
        let admission = self
            .support
            .boolean()
            .map_err(contract)?
            .conditional
            .clone();
        let predicate = self
            .syntax
            .predicates
            .get(root)
            .ok_or_else(|| contract("resolved indicator is outside its owner"))?
            .clone();
        let conjunction = matches!(&predicate, ResolvedPredicate::And(..));
        match predicate {
            ResolvedPredicate::Bool(value) => self.boolean(value, builder, source),
            ResolvedPredicate::Compare { .. } => {
                let guard = self.guard(root, builder)?;
                let yes = self.fork();
                let no = self.fork();
                builder.conditional_admitted(
                    guard,
                    |builder| yes.boolean(true, builder, source),
                    |builder| no.boolean(false, builder, source),
                    admission,
                    source,
                )
            }
            ResolvedPredicate::And(lhs, rhs) | ResolvedPredicate::Or(lhs, rhs) => {
                let guard = self.guard(lhs, builder)?;
                let mut yes = self.fork();
                let mut no = self.fork();
                let value = builder.conditional_admitted(
                    guard,
                    |builder| {
                        if conjunction {
                            yes.indicator(rhs, builder)
                        } else {
                            yes.boolean(true, builder, source)
                        }
                    },
                    |builder| {
                        if conjunction {
                            no.boolean(false, builder, source)
                        } else {
                            no.indicator(rhs, builder)
                        }
                    },
                    admission,
                    source,
                )?;
                self.occurrences.extend(yes.occurrences);
                self.occurrences.extend(no.occurrences);
                Ok(value)
            }
            ResolvedPredicate::Not(inner) => {
                let guard = self.guard(inner, builder)?;
                let yes = self.fork();
                let no = self.fork();
                builder.conditional_admitted(
                    guard,
                    |builder| yes.boolean(false, builder, source),
                    |builder| no.boolean(true, builder, source),
                    admission,
                    source,
                )
            }
            ResolvedPredicate::Atom(_) | ResolvedPredicate::In { .. } | ResolvedPredicate::Null => {
                Err(contract(
                    "unselected Boolean or finite source predicate reached scalar construction",
                ))
            }
        }
    }
}
