// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded relocation of already constructed closed DAGs; no AST or source lookup occurs.
use super::construction::Construction;
use super::*;

impl Construction {
    /// Contextual anchor products are already selected at their actual finite inputs.
    /// Import the demanded roots once, preserving DAG sharing and all retained admissions.
    pub(crate) fn import_closed(
        &mut self,
        source: &Construction,
        roots: &[NodeIx],
    ) -> Result<Vec<NodeIx>> {
        let mut import = Import {
            source,
            target: self,
            nodes: BTreeMap::new(),
            predicates: BTreeMap::new(),
            binders: BTreeMap::new(),
            carriers: BTreeMap::new(),
            evidence: BTreeMap::new(),
            active: std::collections::BTreeSet::new(),
            depth: 0,
        };
        roots.iter().map(|root| import.node(*root)).collect()
    }
}
struct Import<'a> {
    source: &'a Construction,
    target: &'a mut Construction,
    nodes: BTreeMap<NodeIx, NodeIx>,
    predicates: BTreeMap<PredicateIx, PredicateIx>,
    binders: BTreeMap<BinderIx, BinderIx>,
    carriers: BTreeMap<CarrierIx, CarrierIx>,
    evidence: BTreeMap<EvidenceIx, EvidenceIx>,
    active: std::collections::BTreeSet<NodeIx>,
    depth: usize,
}
impl Import<'_> {
    fn carrier(&mut self, slot: CarrierIx) -> Result<CarrierIx> {
        if let Some(target) = self.carriers.get(&slot) {
            return Ok(*target);
        }
        let value = self.source.carrier_at(slot)?.clone();
        let target = self.target.carrier(value)?;
        self.carriers.insert(slot, target);
        Ok(target)
    }
    fn binder(&mut self, slot: BinderIx) -> Result<BinderIx> {
        if let Some(target) = self.binders.get(&slot) {
            return Ok(*target);
        }
        let source = self.source;
        let value = match source.binding_at(slot)? {
            Binding::Value(root) => Binding::Value(self.node(*root)?),
            Binding::Coordinate(carrier) => Binding::Coordinate(self.carrier(*carrier)?),
            Binding::Accumulator => Binding::Accumulator,
            Binding::Item => Binding::Item,
        };
        let target = self.target.binding(value)?;
        self.binders.insert(slot, target);
        Ok(target)
    }
    fn nodes(&mut self, values: &[NodeIx]) -> Result<Vec<NodeIx>> {
        values.iter().map(|root| self.node(*root)).collect()
    }
    fn predicate(&mut self, slot: PredicateIx) -> Result<PredicateIx> {
        if let Some(target) = self.predicates.get(&slot) {
            return Ok(*target);
        }
        if self.depth >= 128 {
            return Err(crate::ModelingError::Budget(
                "closed predicate import depth".into(),
            ));
        }
        self.depth += 1;
        let source = self.source;
        let predicate = match source.predicate_at(slot)? {
            ResolvedPredicate::Compare { op, lhs, rhs } => ResolvedPredicate::Compare {
                op: *op,
                lhs: self.node(*lhs)?,
                rhs: self.node(*rhs)?,
            },
            ResolvedPredicate::In { value, domain } => ResolvedPredicate::In {
                value: self.node(*value)?,
                domain: self.carrier(*domain)?,
            },
            ResolvedPredicate::And(lhs, rhs) => {
                ResolvedPredicate::And(self.predicate(*lhs)?, self.predicate(*rhs)?)
            }
            ResolvedPredicate::Or(lhs, rhs) => {
                ResolvedPredicate::Or(self.predicate(*lhs)?, self.predicate(*rhs)?)
            }
            ResolvedPredicate::Not(value) => ResolvedPredicate::Not(self.predicate(*value)?),
            ResolvedPredicate::Atom(value) => ResolvedPredicate::Atom(self.node(*value)?),
            ResolvedPredicate::Bool(value) => ResolvedPredicate::Bool(*value),
            ResolvedPredicate::Null => ResolvedPredicate::Null,
        };
        self.depth -= 1;
        let source = self.source.predicate_source(slot)?.clone();
        let target = self.target.predicate(predicate, source)?;
        self.predicates.insert(slot, target);
        Ok(target)
    }
    fn evidence(&mut self, slot: EvidenceIx) -> Result<EvidenceIx> {
        if let Some(target) = self.evidence.get(&slot) {
            return Ok(*target);
        }
        let source = self.source;
        let usage = source.evidence_at(slot)?;
        let node = usage.node.clone();
        let predicates = usage
            .predicates
            .iter()
            .map(|root| self.predicate(*root))
            .collect::<Result<_>>()?;
        let inputs = self.nodes(&usage.inputs)?;
        let target = self.target.evidence(EvidenceUse {
            node,
            predicates,
            inputs,
        })?;
        self.evidence.insert(slot, target);
        Ok(target)
    }
    fn node(&mut self, slot: NodeIx) -> Result<NodeIx> {
        if let Some(target) = self.nodes.get(&slot) {
            return Ok(*target);
        }
        if self.depth >= 128 || !self.active.insert(slot) {
            return Err(crate::ModelingError::Budget(
                "closed operation import cycle or depth".into(),
            ));
        }
        self.depth += 1;
        let source = self.source;
        let node = match source.operation_at(slot)? {
            Node::Number(value) => Node::Number(value.clone()),
            Node::Reference(Reference::Symbol(_)) => {
                return Err(invalid(
                    SemanticId::NIL,
                    "closed contextual graph imports cannot capture a caller scalar",
                ));
            }
            Node::Reference(Reference::Indexed { .. }) => {
                return Err(invalid(
                    SemanticId::NIL,
                    "closed contextual graph imports require their selected scalar members",
                ));
            }
            Node::Reference(Reference::Lexical(binding)) => {
                Node::Reference(Reference::Lexical(self.binder(*binding)?))
            }
            Node::Neg(root) => Node::Neg(self.node(*root)?),
            Node::Indicator(value) => Node::Indicator(self.predicate(*value)?),
            Node::Binary { op, lhs, rhs } => Node::Binary {
                op: *op,
                lhs: self.node(*lhs)?,
                rhs: self.node(*rhs)?,
            },
            Node::Call { function, args } => Node::Call {
                function: *function,
                args: self.nodes(args)?,
            },
            Node::Provider { selection, args } => Node::Provider {
                selection: *selection,
                args: self.nodes(args)?,
            },
            Node::Apply { function, args } => {
                let product = source.function_at(*function)?.product.clone();
                let function = self.target.function(product)?;
                Node::Apply {
                    function,
                    args: self.nodes(args)?,
                }
            }
            Node::Partial {
                function,
                wrt,
                args,
            } => {
                let product = source.function_at(*function)?.product.clone();
                let function = self.target.function(product)?;
                Node::Partial {
                    function,
                    wrt: wrt.clone(),
                    args: self.nodes(args)?,
                }
            }
            Node::Conditional {
                guard,
                then,
                otherwise,
            } => Node::Conditional {
                guard: self.predicate(*guard)?,
                then: self.node(*then)?,
                otherwise: self.node(*otherwise)?,
            },
            Node::Let { bindings, body } => Node::Let {
                bindings: bindings
                    .iter()
                    .map(|binding| self.binder(*binding))
                    .collect::<Result<_>>()?,
                body: self.node(*body)?,
            },
            Node::Evidence { value, evidence } => Node::Evidence {
                value: self.node(*value)?,
                evidence: self.evidence(*evidence)?,
            },
            Node::Sequence {
                value,
                prerequisites,
            } => Node::Sequence {
                value: self.node(*value)?,
                prerequisites: self.nodes(prerequisites)?,
            },
            Node::Reduce {
                kind,
                binder,
                filter,
                body,
            } => Node::Reduce {
                kind: *kind,
                binder: self.binder(*binder)?,
                filter: filter.map(|filter| self.predicate(filter)).transpose()?,
                body: self.node(*body)?,
            },
            Node::Fold {
                binder,
                filter,
                value,
                accumulator,
                item,
                step,
            } => Node::Fold {
                binder: self.binder(*binder)?,
                filter: filter.map(|filter| self.predicate(filter)).transpose()?,
                value: self.node(*value)?,
                accumulator: self.binder(*accumulator)?,
                item: self.binder(*item)?,
                step: self.node(*step)?,
            },
            Node::Derivative { body, domain } => Node::Derivative {
                body: self.node(*body)?,
                domain: self.carrier(*domain)?,
            },
        };
        let attribution = self.source.node_source(slot)?.clone();
        let target = self.target.node(node, attribution)?;
        if let Some(admission) = source.admission_at(slot) {
            self.target.admit(target, admission.clone())?;
        }
        if let Some(ty) = source.literal_at(slot) {
            self.target.literal(target, ty.clone())?;
        }
        if let Some(fold) = source.fold_at(slot) {
            let carrier = self.carrier(fold.carrier)?;
            self.target.finite_fold(
                target,
                carrier,
                fold.selected.to_vec(),
                fold.prototype.clone(),
            )?;
        }
        self.depth -= 1;
        self.active.remove(&slot);
        self.nodes.insert(slot, target);
        Ok(target)
    }
}
