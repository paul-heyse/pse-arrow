// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Construction from already selected typed edges. This module never resolves an AST.
use super::*;

/// Inspection syntax and its executable root travel together during specialization.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Rewritten {
    pub(crate) inspection: Expr,
    pub(crate) root: NodeIx,
}

/// A predicate retains its inspection syntax independently of its executable edges.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RewrittenPredicate {
    pub(crate) inspection: Predicate,
    pub(crate) root: PredicateIx,
}

/// An actual selected relation, including its original orientation and equation sense.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RewrittenEquation {
    pub(crate) inspection: dsl::Equation,
    pub(crate) lhs: NodeIx,
    pub(crate) sense: dsl::EquationSense,
    pub(crate) rhs: NodeIx,
}

/// Ordered relation roots, separate from the inspection equation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Relation {
    /// Original left operand.
    pub lhs: NodeIx,
    /// Authored relation sense.
    pub sense: dsl::EquationSense,
    /// Original right operand.
    pub rhs: NodeIx,
}
impl RewrittenEquation {
    pub(crate) fn relation(&self) -> Relation {
        Relation {
            lhs: self.lhs,
            sense: self.sense,
            rhs: self.rhs,
        }
    }
}
/// A selected application prepares its child roots before returning its parent operation.
#[derive(Debug)]
pub(crate) struct RewrittenKind {
    pub(crate) node: Node,
    pub(crate) inspection: ExprKind,
}
impl RewrittenKind {
    pub(crate) fn into_parts(self) -> (Node, ExprKind) {
        (self.node, self.inspection)
    }
}
/// A source-boundary name points to its actual scalar or lexical role.
#[derive(Clone, Debug)]
pub(crate) struct ScalarSelection {
    pub(crate) reference: Reference,
    pub(crate) inspection: Expr,
}

/// Mutable only while one region is constructed. The scalar table is attached once at finish,
/// so allocating a scalar cannot detach and copy a second shared symbol table on every insert.
#[derive(Debug)]
pub(crate) struct Construction {
    pub(super) arena: Arena,
    // Construction-local lookup only. The owning Arcs remain in arena.functions until finish.
    // These addresses never enter a semantic frame or a published product.
    pub(super) functions: BTreeMap<usize, FunctionIx>,
    allowance: usize,
    pub(super) used: usize,
    pub(super) admissions: BTreeMap<NodeIx, crate::expression::admission::PhysicalAdmission>,
    pub(super) literals: BTreeMap<NodeIx, Type>,
}
impl Construction {
    pub(crate) fn new(allowance: usize) -> Self {
        Self {
            arena: Arena::default(),
            functions: BTreeMap::new(),
            allowance,
            used: 0,
            admissions: BTreeMap::new(),
            literals: BTreeMap::new(),
        }
    }
    pub(crate) fn member_range(&mut self, slot: SymbolIx, range: MemberRange) -> Result<()> {
        self.node_edge(range.lower)?;
        self.node_edge(range.upper)?;
        self.reserve(3)?;
        if self.arena.member_ranges.insert(slot, range).is_some() {
            return Err(invalid(
                SemanticId::NIL,
                "one scalar has two original hard ranges",
            ));
        }
        Ok(())
    }
    pub(super) fn reserve(&mut self, amount: usize) -> Result<()> {
        let used = self
            .used
            .checked_add(amount)
            .filter(|used| *used <= self.allowance)
            .ok_or_else(|| {
                crate::ModelingError::Budget("resolved construction allowance exceeded".into())
            })?;
        self.used = used;
        Ok(())
    }
    fn node_edge(&self, slot: NodeIx) -> Result<()> {
        self.arena.nodes.get(slot).map(|_| ()).ok_or_else(|| {
            invalid(
                SemanticId::NIL,
                "construction operation edge is outside its owner",
            )
        })
    }
    fn predicate_edge(&self, slot: PredicateIx) -> Result<()> {
        self.arena.predicates.get(slot).map(|_| ()).ok_or_else(|| {
            invalid(
                SemanticId::NIL,
                "construction predicate edge is outside its owner",
            )
        })
    }
    fn binder_edge(&self, slot: BinderIx) -> Result<()> {
        self.arena.binders.get(slot).map(|_| ()).ok_or_else(|| {
            invalid(
                SemanticId::NIL,
                "construction binding edge is outside its owner",
            )
        })
    }
    fn carrier_edge(&self, slot: CarrierIx) -> Result<()> {
        self.arena.carriers.get(slot).map(|_| ()).ok_or_else(|| {
            invalid(
                SemanticId::NIL,
                "construction carrier edge is outside its owner",
            )
        })
    }
    pub(crate) fn node(&mut self, node: Node, source: Source) -> Result<NodeIx> {
        let edges = match &node {
            Node::Reference(Reference::Indexed { indices, .. }) => indices.len(),
            Node::Call { args, .. } | Node::Apply { args, .. } | Node::Provider { args, .. } => {
                args.len()
            }
            Node::Partial { args, wrt, .. } => args.len().saturating_add(wrt.len()),
            Node::Let { bindings, .. } => bindings.len().saturating_add(1),
            Node::Sequence { prerequisites, .. } => prerequisites.len().saturating_add(1),
            Node::Evidence { .. } | Node::Binary { .. } => 2,
            Node::Conditional { .. } => 3,
            Node::Reduce { filter, .. } => 2 + usize::from(filter.is_some()),
            Node::Fold { filter, .. } => 5 + usize::from(filter.is_some()),
            Node::Derivative { .. } => 2,
            Node::Neg(_) | Node::Indicator(_) | Node::Reference(Reference::Lexical(_)) => 1,
            Node::Number(_) | Node::Reference(Reference::Symbol(_)) => 0,
        };
        match &node {
            Node::Number(value) if !value.value.is_finite() => {
                return Err(invalid(
                    source.field.declaration,
                    "nonfinite constructed literal",
                ));
            }
            Node::Number(_) | Node::Reference(Reference::Symbol(_)) => {}
            Node::Reference(Reference::Lexical(binding)) => self.binder_edge(*binding)?,
            Node::Reference(Reference::Indexed { group, indices }) => {
                if self.arena.groups.get(*group).is_none() {
                    return Err(invalid(
                        source.field.declaration,
                        "construction indexed family is outside its owner",
                    ));
                }
                for index in indices {
                    self.node_edge(*index)?;
                }
            }
            Node::Neg(value) => self.node_edge(*value)?,
            Node::Indicator(value) => self.predicate_edge(*value)?,
            Node::Binary { lhs, rhs, .. } => {
                self.node_edge(*lhs)?;
                self.node_edge(*rhs)?;
            }
            Node::Call { args, .. } | Node::Provider { args, .. } => {
                for arg in args {
                    self.node_edge(*arg)?;
                }
            }
            Node::Apply { function, args } | Node::Partial { function, args, .. } => {
                let selected = self.arena.functions.get(*function).ok_or_else(|| {
                    invalid(
                        source.field.declaration,
                        "construction function is outside its owner",
                    )
                })?;
                if selected.product.signature.len() != args.len() {
                    return Err(invalid(
                        source.field.declaration,
                        "constructed function argument arity differs",
                    ));
                }
                if let Node::Partial { wrt, .. } = &node
                    && wrt.iter().any(|ordinal| *ordinal >= args.len())
                {
                    return Err(invalid(
                        source.field.declaration,
                        "constructed partial is outside the ordered signature",
                    ));
                }
                for arg in args {
                    self.node_edge(*arg)?;
                }
            }
            Node::Reduce {
                binder,
                filter,
                body,
                ..
            } => {
                self.binder_edge(*binder)?;
                if let Some(filter) = filter {
                    self.predicate_edge(*filter)?;
                }
                self.node_edge(*body)?;
            }
            Node::Fold {
                binder,
                filter,
                value,
                accumulator,
                item,
                step,
            } => {
                for binder in [binder, accumulator, item] {
                    self.binder_edge(*binder)?;
                }
                if let Some(filter) = filter {
                    self.predicate_edge(*filter)?;
                }
                self.node_edge(*value)?;
                self.node_edge(*step)?;
            }
            Node::Derivative { body, domain } => {
                self.node_edge(*body)?;
                self.carrier_edge(*domain)?;
            }
            Node::Conditional {
                guard,
                then,
                otherwise,
            } => {
                self.predicate_edge(*guard)?;
                self.node_edge(*then)?;
                self.node_edge(*otherwise)?;
            }
            Node::Let { bindings, body } => {
                for binding in bindings {
                    self.binder_edge(*binding)?;
                }
                self.node_edge(*body)?;
            }
            Node::Evidence { value, evidence } => {
                self.node_edge(*value)?;
                if self.arena.evidence.get(*evidence).is_none() {
                    return Err(invalid(
                        source.field.declaration,
                        "constructed evidence is outside its owner",
                    ));
                }
            }
            Node::Sequence {
                value,
                prerequisites,
            } => {
                self.node_edge(*value)?;
                for prerequisite in prerequisites {
                    self.node_edge(*prerequisite)?;
                }
            }
        }
        self.reserve(edges.saturating_add(1))?;
        let slot = NodeIx::from(self.arena.nodes.len());
        self.arena.nodes.push(node);
        self.arena.node_sources.insert(slot, source);
        Ok(slot)
    }
    pub(crate) fn binary(
        &mut self,
        op: BinaryOp,
        lhs: Rewritten,
        rhs: Rewritten,
        source: Source,
    ) -> Result<Rewritten> {
        self.pair(
            Node::Binary {
                op,
                lhs: lhs.root,
                rhs: rhs.root,
            },
            ExprKind::Binary {
                op,
                lhs: Box::new(lhs.inspection),
                rhs: Box::new(rhs.inspection),
            },
            source,
        )
    }
    pub(crate) fn neg(&mut self, value: Rewritten, source: Source) -> Result<Rewritten> {
        self.pair(
            Node::Neg(value.root),
            ExprKind::Neg(Box::new(value.inspection)),
            source,
        )
    }
    pub(crate) fn number(&mut self, number: dsl::Number, source: Source) -> Result<Rewritten> {
        self.pair(
            Node::Number(number.clone()),
            ExprKind::Number(number),
            source,
        )
    }
    pub(crate) fn sum(&mut self, values: Vec<Rewritten>, source: Source) -> Result<Rewritten> {
        let mut values = values.into_iter();
        let mut result = values.next().ok_or_else(|| {
            invalid(
                source.field.declaration,
                "empty constructed sum lacks a typed neutral",
            )
        })?;
        for value in values {
            result = self.binary(BinaryOp::Add, result, value, source.clone())?;
        }
        Ok(result)
    }
    pub(crate) fn compare(
        &mut self,
        op: CompareOp,
        lhs: Rewritten,
        rhs: Rewritten,
        source: Source,
    ) -> Result<RewrittenPredicate> {
        let span = source.span;
        let root = self.predicate(
            ResolvedPredicate::Compare {
                op,
                lhs: lhs.root,
                rhs: rhs.root,
            },
            source,
        )?;
        Ok(RewrittenPredicate {
            root,
            inspection: Predicate {
                kind: PredicateKind::Compare {
                    op,
                    lhs: Box::new(lhs.inspection),
                    rhs: Box::new(rhs.inspection),
                },
                span,
            },
        })
    }
    pub(crate) fn and(
        &mut self,
        lhs: RewrittenPredicate,
        rhs: RewrittenPredicate,
        source: Source,
    ) -> Result<RewrittenPredicate> {
        let span = source.span;
        let root = self.predicate(ResolvedPredicate::And(lhs.root, rhs.root), source)?;
        Ok(RewrittenPredicate {
            root,
            inspection: Predicate {
                kind: PredicateKind::And(Box::new(lhs.inspection), Box::new(rhs.inspection)),
                span,
            },
        })
    }
    pub(crate) fn pair(
        &mut self,
        node: Node,
        inspection: ExprKind,
        source: Source,
    ) -> Result<Rewritten> {
        let span = source.span;
        Ok(Rewritten {
            root: self.node(node, source)?,
            inspection: Expr {
                kind: inspection,
                span,
            },
        })
    }
    pub(crate) fn predicate(
        &mut self,
        predicate: ResolvedPredicate,
        source: Source,
    ) -> Result<PredicateIx> {
        match &predicate {
            ResolvedPredicate::Compare { lhs, rhs, .. } => {
                self.node_edge(*lhs)?;
                self.node_edge(*rhs)?;
            }
            ResolvedPredicate::In { value, domain } => {
                self.node_edge(*value)?;
                self.carrier_edge(*domain)?;
            }
            ResolvedPredicate::And(lhs, rhs) | ResolvedPredicate::Or(lhs, rhs) => {
                self.predicate_edge(*lhs)?;
                self.predicate_edge(*rhs)?;
            }
            ResolvedPredicate::Not(value) => self.predicate_edge(*value)?,
            ResolvedPredicate::Atom(value) => self.node_edge(*value)?,
            ResolvedPredicate::Bool(_) | ResolvedPredicate::Null => {}
        }
        self.reserve(match &predicate {
            ResolvedPredicate::Compare { .. }
            | ResolvedPredicate::In { .. }
            | ResolvedPredicate::And(..)
            | ResolvedPredicate::Or(..) => 3,
            ResolvedPredicate::Not(_) | ResolvedPredicate::Atom(_) => 2,
            ResolvedPredicate::Bool(_) | ResolvedPredicate::Null => 1,
        })?;
        let slot = PredicateIx::from(self.arena.predicates.len());
        self.arena.predicates.push(predicate);
        self.arena.predicate_sources.insert(slot, source);
        Ok(slot)
    }
    pub(crate) fn evidence(&mut self, usage: EvidenceUse) -> Result<EvidenceIx> {
        if !usage
            .node
            .valid_capture(usage.predicates.len(), usage.inputs.len(), self.allowance)
        {
            return Err(invalid(
                usage.node.claim.form,
                "constructed evidence capture is invalid",
            ));
        }
        for predicate in &usage.predicates {
            self.predicate_edge(*predicate)?;
        }
        for input in &usage.inputs {
            self.node_edge(*input)?;
        }
        self.reserve(
            1usize
                .saturating_add(usage.predicates.len())
                .saturating_add(usage.inputs.len()),
        )?;
        let slot = EvidenceIx::from(self.arena.evidence.len());
        self.arena.evidence.push(usage);
        Ok(slot)
    }
    pub(crate) fn binding(&mut self, binding: Binding) -> Result<BinderIx> {
        match binding {
            Binding::Value(value) => self.node_edge(value)?,
            Binding::Coordinate(carrier) => self.carrier_edge(carrier)?,
            Binding::Accumulator | Binding::Item => {}
        }
        self.reserve(
            1 + usize::from(matches!(
                binding,
                Binding::Value(_) | Binding::Coordinate(_)
            )),
        )?;
        let slot = BinderIx::from(self.arena.binders.len());
        self.arena.binders.push(binding);
        Ok(slot)
    }
    pub(crate) fn function(&mut self, product: Arc<FunctionProduct>) -> Result<FunctionIx> {
        let owner = Arc::as_ptr(&product) as usize;
        if let Some(slot) = self.functions.get(&owner) {
            return Ok(*slot);
        }
        self.reserve(product.formals.len().saturating_add(1))?;
        let slot = FunctionIx::from(self.arena.functions.len());
        self.arena.functions.push(FunctionSelection {
            identity: product.identity,
            declaration: product.declaration,
            formals: product
                .formals
                .iter()
                .map(|slot| product.syntax.symbols[*slot].lineage.path.clone())
                .collect(),
            product,
        });
        self.functions.insert(owner, slot);
        Ok(slot)
    }
    pub(crate) fn carrier(&mut self, carrier: Carrier) -> Result<CarrierIx> {
        self.reserve(carrier.members.len().saturating_add(1))?;
        let slot = CarrierIx::from(self.arena.carriers.len());
        self.arena.carriers.push(carrier);
        Ok(slot)
    }
    pub(crate) fn finite_fold(
        &mut self,
        root: NodeIx,
        carrier: CarrierIx,
        selected: Vec<crate::specialize::Value>,
        prototype: Type,
    ) -> Result<()> {
        if !matches!(self.arena.nodes.get(root), Some(Node::Let { .. })) {
            return Err(invalid(
                SemanticId::NIL,
                "finite fold lacks its directly constructed value bindings",
            ));
        }
        let domain =
            self.arena.carriers.get(carrier).ok_or_else(|| {
                invalid(SemanticId::NIL, "finite fold carrier is outside its owner")
            })?;
        if selected.is_empty() {
            return Err(invalid(
                SemanticId::NIL,
                "finite fold requires at least one selected member",
            ));
        }
        let mut members = domain.members.iter();
        for value in &selected {
            if !members.any(|member| member == value) {
                return Err(invalid(
                    SemanticId::NIL,
                    "finite fold selection changes carrier membership or semantic order",
                ));
            }
        }
        if self.arena.finite_folds.contains_key(&root) {
            return Err(invalid(
                SemanticId::NIL,
                "finite fold root already owns its selection",
            ));
        }
        self.reserve(selected.len().saturating_add(1))?;
        self.arena.finite_folds.insert(
            root,
            FiniteFold {
                carrier,
                selected: selected.into(),
                prototype,
            },
        );
        Ok(())
    }
    pub(crate) fn layout(
        &self,
        symbols: &crate::specialize::Symbols,
        outputs: &[NodeIx],
        validity: &[PredicateIx],
    ) -> Result<CanonicalLayout> {
        framing::layout_parts(symbols, &self.arena, outputs, validity)
    }
    pub(crate) fn admit(
        &mut self,
        root: NodeIx,
        admission: crate::expression::admission::PhysicalAdmission,
    ) -> Result<()> {
        self.node_edge(root)?;
        if self.admissions.contains_key(&root) {
            return Err(invalid(
                SemanticId::NIL,
                "one constructed operation has duplicate physical obligations",
            ));
        }
        self.reserve(1)?;
        self.admissions.insert(root, admission);
        Ok(())
    }
    pub(crate) fn literal(&mut self, root: NodeIx, ty: Type) -> Result<()> {
        if !matches!(self.arena.nodes.get(root), Some(Node::Number(_))) {
            return Err(invalid(
                SemanticId::NIL,
                "literal contract requires its actual constructed literal",
            ));
        }
        if self
            .literals
            .get(&root)
            .is_some_and(|previous| previous != &ty)
        {
            return Err(invalid(
                SemanticId::NIL,
                "one literal has conflicting concrete contracts",
            ));
        }
        self.reserve(1)?;
        self.literals.insert(root, ty);
        Ok(())
    }
    pub(crate) fn take_support(
        &mut self,
    ) -> (
        BTreeMap<NodeIx, crate::expression::admission::PhysicalAdmission>,
        BTreeMap<NodeIx, Type>,
    ) {
        (
            std::mem::take(&mut self.admissions),
            std::mem::take(&mut self.literals),
        )
    }
    pub(crate) fn has_effect(
        &self,
        root: NodeIx,
        symbols: &crate::specialize::Symbols,
    ) -> Result<bool> {
        effects(&self.arena, symbols, &[root], &[])
    }
    pub(crate) fn operation_at(&self, slot: NodeIx) -> Result<&Node> {
        self.arena.nodes.get(slot).ok_or_else(|| {
            invalid(
                SemanticId::NIL,
                "required imported operation is outside its owner",
            )
        })
    }
    pub(crate) fn predicate_at(&self, slot: PredicateIx) -> Result<&ResolvedPredicate> {
        self.arena.predicates.get(slot).ok_or_else(|| {
            invalid(
                SemanticId::NIL,
                "required imported predicate is outside its owner",
            )
        })
    }
    pub(crate) fn binding_at(&self, slot: BinderIx) -> Result<&Binding> {
        self.arena.binders.get(slot).ok_or_else(|| {
            invalid(
                SemanticId::NIL,
                "required imported binding is outside its owner",
            )
        })
    }
    pub(super) fn carrier_at(&self, slot: CarrierIx) -> Result<&Carrier> {
        self.arena.carriers.get(slot).ok_or_else(|| {
            invalid(
                SemanticId::NIL,
                "required imported carrier is outside its owner",
            )
        })
    }
    pub(crate) fn evidence_at(&self, slot: EvidenceIx) -> Result<&EvidenceUse> {
        self.arena.evidence.get(slot).ok_or_else(|| {
            invalid(
                SemanticId::NIL,
                "required imported evidence is outside its owner",
            )
        })
    }
    pub(crate) fn function_at(&self, slot: FunctionIx) -> Result<&FunctionSelection> {
        self.arena.functions.get(slot).ok_or_else(|| {
            invalid(
                SemanticId::NIL,
                "required imported function is outside its owner",
            )
        })
    }
    pub(crate) fn node_source(&self, slot: NodeIx) -> Result<&Source> {
        self.arena.node_sources.get(&slot).ok_or_else(|| {
            invalid(
                SemanticId::NIL,
                "required imported operation attribution is absent",
            )
        })
    }
    pub(crate) fn predicate_source(&self, slot: PredicateIx) -> Result<&Source> {
        self.arena.predicate_sources.get(&slot).ok_or_else(|| {
            invalid(
                SemanticId::NIL,
                "required imported predicate attribution is absent",
            )
        })
    }
    pub(super) fn admission_at(
        &self,
        slot: NodeIx,
    ) -> Option<&crate::expression::admission::PhysicalAdmission> {
        self.admissions.get(&slot)
    }
    pub(super) fn literal_at(&self, slot: NodeIx) -> Option<&Type> {
        self.literals.get(&slot)
    }
    pub(super) fn fold_at(&self, slot: NodeIx) -> Option<&FiniteFold> {
        self.arena.finite_folds.get(&slot)
    }
    pub(crate) fn copy_facts(&mut self, from: NodeIx, to: NodeIx) -> Result<()> {
        if let Some(value) = self.admissions.get(&from).cloned() {
            self.admit(to, value)?;
        }
        if let Some(value) = self.literals.get(&from).cloned() {
            self.literal(to, value)?;
        }
        if let Some(value) = self.arena.finite_folds.get(&from).cloned() {
            self.finite_fold(to, value.carrier, value.selected.to_vec(), value.prototype)?;
        }
        Ok(())
    }
    pub(crate) fn view<'a>(
        &'a self,
        symbols: &'a crate::specialize::Symbols,
        outputs: &'a [NodeIx],
        validity: &'a [PredicateIx],
    ) -> View<'a> {
        View {
            symbols,
            arena: &self.arena,
            outputs,
            validity,
        }
    }
    pub(crate) fn reads(
        &self,
        symbols: &crate::specialize::Symbols,
        root: NodeIx,
    ) -> Result<RegionReads> {
        let roots = [root];
        let outputs = [RegionOutput::Value(root)];
        selection::reads_view(
            self.view(symbols, &roots, &[]),
            RegionRequest {
                outputs: &outputs,
                validity: &[],
            },
        )
    }
    pub(crate) fn literal_contracts(&self) -> &BTreeMap<NodeIx, Type> {
        &self.literals
    }
    pub(crate) fn get(&self, root: NodeIx) -> Option<&Node> {
        self.arena.nodes.get(root)
    }
    pub(crate) fn finish(
        self,
        symbols: crate::specialize::Symbols,
        outputs: Vec<NodeIx>,
        validity: Vec<PredicateIx>,
    ) -> Result<ResolvedSyntax> {
        for root in &outputs {
            self.node_edge(*root)?;
        }
        for root in &validity {
            self.predicate_edge(*root)?;
        }
        for node in &self.arena.nodes {
            if let Node::Reference(Reference::Symbol(slot)) = node
                && symbols.at(*slot).is_none()
            {
                return Err(invalid(
                    SemanticId::NIL,
                    "constructed scalar edge is outside its authoritative table",
                ));
            }
        }
        for group in &self.arena.groups {
            for (_, member) in &group.members {
                if member.is_some_and(|slot| symbols.at(slot).is_none()) {
                    return Err(invalid(
                        SemanticId::NIL,
                        "constructed indexed member is outside its authoritative table",
                    ));
                }
            }
        }
        Ok(ResolvedSyntax {
            symbols,
            arena: Arc::new(self.arena),
            outputs,
            validity,
        })
    }
}

/// Borrowed reached inquiry over one owner; child products already retain their own result.
/// Ordinary lets visit only values reached from the result or original guard.
pub(super) fn effects(
    arena: &Arena,
    symbols: &crate::specialize::Symbols,
    outputs: &[NodeIx],
    validity: &[PredicateIx],
) -> Result<bool> {
    let mut nodes = outputs.to_vec();
    let mut predicates = validity.to_vec();
    let mut seen = std::collections::BTreeSet::new();
    let mut guards = std::collections::BTreeSet::new();
    while !nodes.is_empty() || !predicates.is_empty() {
        while let Some(root) = predicates.pop() {
            if !guards.insert(root) {
                continue;
            }
            match arena.predicates.get(root).ok_or_else(|| {
                invalid(
                    SemanticId::NIL,
                    "effect inquiry reached an absent predicate",
                )
            })? {
                ResolvedPredicate::Compare { lhs, rhs, .. } => nodes.extend([*lhs, *rhs]),
                ResolvedPredicate::In { value, .. } | ResolvedPredicate::Atom(value) => {
                    nodes.push(*value)
                }
                ResolvedPredicate::And(lhs, rhs) | ResolvedPredicate::Or(lhs, rhs) => {
                    predicates.extend([*lhs, *rhs])
                }
                ResolvedPredicate::Not(value) => predicates.push(*value),
                ResolvedPredicate::Bool(_) | ResolvedPredicate::Null => {}
            }
        }
        let Some(root) = nodes.pop() else {
            continue;
        };
        if !seen.insert(root) {
            continue;
        }
        match arena.nodes.get(root).ok_or_else(|| {
            invalid(
                SemanticId::NIL,
                "effect inquiry reached an absent typed operation",
            )
        })? {
            Node::Apply { function, args } | Node::Partial { function, args, .. } => {
                let product = &arena
                    .functions
                    .get(*function)
                    .ok_or_else(|| {
                        invalid(
                            SemanticId::NIL,
                            "effect inquiry reached an absent selected function",
                        )
                    })?
                    .product;
                if product.has_applicability_effects {
                    return Ok(true);
                }
                nodes.extend(args);
            }
            Node::Reference(Reference::Symbol(symbol)) => {
                if let Some(root) = symbols
                    .at(*symbol)
                    .ok_or_else(|| {
                        invalid(SemanticId::NIL, "effect inquiry reached an absent scalar")
                    })?
                    .definition
                {
                    nodes.push(root);
                }
            }
            Node::Reference(Reference::Lexical(binder)) => {
                if let Some(Binding::Value(root)) = arena.binders.get(*binder) {
                    nodes.push(*root);
                }
            }
            Node::Reference(Reference::Indexed { indices, .. }) => nodes.extend(indices),
            Node::Neg(root) | Node::Derivative { body: root, .. } => nodes.push(*root),
            Node::Indicator(predicate) => predicates.push(*predicate),
            Node::Binary { lhs, rhs, .. } => nodes.extend([*lhs, *rhs]),
            Node::Call { args, .. } | Node::Provider { args, .. } => nodes.extend(args),
            Node::Conditional {
                guard,
                then,
                otherwise,
            } => {
                predicates.push(*guard);
                nodes.extend([*then, *otherwise]);
            }
            Node::Let { body, .. } => nodes.push(*body),
            Node::Sequence { .. } | Node::Evidence { .. } => return Ok(true),
            Node::Reduce { body, filter, .. } => {
                nodes.push(*body);
                predicates.extend(filter);
            }
            Node::Fold {
                value,
                step,
                filter,
                ..
            } => {
                nodes.extend([*value, *step]);
                predicates.extend(filter);
            }
            Node::Number(_) => {}
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn field() -> crate::expression::occurrences::OccurrenceKey {
        crate::expression::occurrences::OccurrenceKey {
            declaration: SemanticId::from_bytes([3; 16]).into(),
            role: "binding.expression".into(),
            position: 0,
        }
    }
    fn source(expression: &Expr) -> Source {
        Source {
            field: field(),
            span: expression.span,
            coordinate: Some(InventoryCoordinate {
                stage: InventoryStage::Original,
                grammar: InventoryGrammar::Expression,
                occurrence: crate::expression::admission::ExpressionOccurrence::Node {
                    body: 0,
                    position: 0,
                },
            }),
        }
    }
    #[test]
    fn direct_construction_retains_supplied_edges_and_enforces_owner_and_allowance() {
        let expression = dsl::parse_expr("2").unwrap();
        let source = source(&expression);
        let ExprKind::Number(number) = &expression.kind else {
            panic!()
        };
        let mut construction = Construction::new(3);
        assert!(
            construction
                .node(Node::Neg(NodeIx::from(0)), source.clone())
                .is_err()
        );
        let literal = construction
            .pair(
                Node::Number(number.clone()),
                expression.kind.clone(),
                source.clone(),
            )
            .unwrap();
        let root = construction
            .node(Node::Neg(literal.root), source.clone())
            .unwrap();
        assert!(construction.node(Node::Neg(root), source).is_err());
        let syntax = construction
            .finish(crate::specialize::Symbols::default(), vec![root], vec![])
            .unwrap();
        assert_eq!(syntax.nodes[root], Node::Neg(literal.root));
        assert_eq!(literal.inspection, expression);
        assert_eq!(
            syntax.node_sources[&literal.root].coordinate.unwrap().stage,
            InventoryStage::Original
        );
    }
    #[test]
    fn ordinary_unused_locals_do_not_become_demanded_observation_sequences() {
        let expression = dsl::parse_expr("2").unwrap();
        let source = source(&expression);
        let mut construction = Construction::new(64);
        let ExprKind::Number(number) = expression.kind else {
            panic!()
        };
        let literal = construction
            .node(Node::Number(number), source.clone())
            .unwrap();
        let observed = construction
            .node(
                Node::Sequence {
                    value: literal,
                    prerequisites: vec![literal],
                },
                source.clone(),
            )
            .unwrap();
        let binder = construction.binding(Binding::Value(observed)).unwrap();
        let unused = construction
            .node(
                Node::Let {
                    bindings: vec![binder],
                    body: literal,
                },
                source.clone(),
            )
            .unwrap();
        assert!(
            !construction
                .has_effect(unused, &crate::specialize::Symbols::default())
                .unwrap()
        );
        let selected = construction
            .node(Node::Reference(Reference::Lexical(binder)), source.clone())
            .unwrap();
        let used = construction
            .node(
                Node::Let {
                    bindings: vec![binder],
                    body: selected,
                },
                source.clone(),
            )
            .unwrap();
        assert!(
            construction
                .has_effect(used, &crate::specialize::Symbols::default())
                .unwrap()
        );
        let guard = construction
            .predicate(ResolvedPredicate::Atom(observed), source.clone())
            .unwrap();
        let conditional = construction
            .node(
                Node::Conditional {
                    guard,
                    then: literal,
                    otherwise: literal,
                },
                source,
            )
            .unwrap();
        assert!(
            construction
                .has_effect(conditional, &crate::specialize::Symbols::default())
                .unwrap()
        );
    }
}
