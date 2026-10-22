// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-authorized graph substitution for existing form/inventory transformations.
//! This traverses issued typed edges only; original Expr inspection never drives it.
use super::*;
use crate::resolved::{
    BinderIx, Binding, EvidenceIx, EvidenceUse, Node, NodeIx, PredicateIx, Reference,
    ResolvedPredicate,
};

impl Engine<'_, '_> {
    pub(super) fn substitute_root(
        &mut self,
        root: NodeIx,
        replacements: &BTreeMap<SemanticId, NodeIx>,
        at: DeclarationId,
    ) -> Result<NodeIx> {
        if self.function_symbols.is_some() {
            return Err(invalid(
                at,
                "a model transformation cannot capture a selected function's local symbol owner",
            ));
        }
        for (id, root) in replacements {
            let ty = self
                .model
                .symbols
                .get(id)
                .ok_or_else(|| invalid(at, "substituted scalar is outside the actual model"))?
                .ty
                .clone();
            if self.constructed_type(*root)? != ty {
                return Err(invalid(
                    at,
                    "substitution changes an actual scalar's full physical/nominal contract",
                ));
            }
        }
        Transform {
            engine: self,
            replacements,
            at,
            nodes: BTreeMap::new(),
            predicates: BTreeMap::new(),
            binders: BTreeMap::new(),
            evidence: BTreeMap::new(),
            members: BTreeMap::new(),
            active: BTreeSet::new(),
            depth: 0,
        }
        .node(root)
    }
}
struct Transform<'a, 'p, 'c> {
    engine: &'a mut Engine<'p, 'c>,
    replacements: &'a BTreeMap<SemanticId, NodeIx>,
    at: DeclarationId,
    nodes: BTreeMap<NodeIx, NodeIx>,
    predicates: BTreeMap<PredicateIx, PredicateIx>,
    binders: BTreeMap<BinderIx, BinderIx>,
    evidence: BTreeMap<EvidenceIx, EvidenceIx>,
    members: BTreeMap<SemanticId, NodeIx>,
    active: BTreeSet<NodeIx>,
    depth: usize,
}
impl Transform<'_, '_, '_> {
    fn node_list(&mut self, roots: &[NodeIx]) -> Result<Vec<NodeIx>> {
        roots.iter().map(|root| self.node(*root)).collect()
    }
    fn binder(&mut self, slot: BinderIx) -> Result<BinderIx> {
        if let Some(value) = self.binders.get(&slot) {
            return Ok(*value);
        }
        let original = self.engine.construction.binding_at(slot)?.clone();
        let transformed = match original {
            Binding::Value(root) => Binding::Value(self.node(root)?),
            _ => original.clone(),
        };
        let result = if transformed == original {
            slot
        } else {
            self.engine.construction.binding(transformed)?
        };
        self.binders.insert(slot, result);
        Ok(result)
    }
    fn predicate(&mut self, slot: PredicateIx) -> Result<PredicateIx> {
        if let Some(value) = self.predicates.get(&slot) {
            return Ok(*value);
        }
        if self.depth >= self.engine.limits.depth {
            return Err(ModelingError::Budget(
                "typed transformation predicate depth".into(),
            ));
        }
        self.depth += 1;
        let original = self.engine.construction.predicate_at(slot)?.clone();
        let transformed = match original {
            ResolvedPredicate::Compare { op, lhs, rhs } => ResolvedPredicate::Compare {
                op,
                lhs: self.node(lhs)?,
                rhs: self.node(rhs)?,
            },
            ResolvedPredicate::In { value, domain } => ResolvedPredicate::In {
                value: self.node(value)?,
                domain,
            },
            ResolvedPredicate::And(lhs, rhs) => {
                ResolvedPredicate::And(self.predicate(lhs)?, self.predicate(rhs)?)
            }
            ResolvedPredicate::Or(lhs, rhs) => {
                ResolvedPredicate::Or(self.predicate(lhs)?, self.predicate(rhs)?)
            }
            ResolvedPredicate::Not(value) => ResolvedPredicate::Not(self.predicate(value)?),
            ResolvedPredicate::Atom(value) => ResolvedPredicate::Atom(self.node(value)?),
            ResolvedPredicate::Bool(_) | ResolvedPredicate::Null => original.clone(),
        };
        let result = if transformed == original {
            slot
        } else {
            let source = self.engine.construction.predicate_source(slot)?.clone();
            self.engine.construction.predicate(transformed, source)?
        };
        self.depth -= 1;
        self.predicates.insert(slot, result);
        Ok(result)
    }
    fn evidence(&mut self, slot: EvidenceIx) -> Result<EvidenceIx> {
        if let Some(value) = self.evidence.get(&slot) {
            return Ok(*value);
        }
        let original = self.engine.construction.evidence_at(slot)?.clone();
        let transformed = EvidenceUse {
            node: original.node.clone(),
            predicates: original
                .predicates
                .iter()
                .map(|root| self.predicate(*root))
                .collect::<Result<_>>()?,
            inputs: self.node_list(&original.inputs)?,
        };
        let result = if transformed == original {
            slot
        } else {
            self.engine.construction.evidence(transformed)?
        };
        self.evidence.insert(slot, result);
        Ok(result)
    }
    fn member(
        &mut self,
        id: SemanticId,
        slot: crate::resolved::SymbolIx,
        root: NodeIx,
    ) -> Result<NodeIx> {
        if let Some(value) = self.replacements.get(&id) {
            let value = *value;
            return self.node(value);
        }
        if let Some(value) = self.members.get(&id) {
            return Ok(*value);
        }
        let original = self
            .engine
            .model
            .symbols
            .at(slot)
            .ok_or_else(|| invalid(self.at, "transformed member is outside its scalar owner"))?
            .clone();
        let Some(definition) = original.definition else {
            return Ok(root);
        };
        let transformed = self.node(definition)?;
        if transformed == definition {
            self.members.insert(id, root);
            return Ok(root);
        }
        let meaning = self.engine.current_roots_identity(&[transformed])?;
        let relationship = crate::derived::Relationship::SubstitutedMember {
            member: id,
            meaning,
        };
        let derived = self.engine.model.derivations.publish(relationship)?;
        let selected = if let Some(slot) = self.engine.model.symbols.slot(derived) {
            slot
        } else {
            let slot = crate::resolved::SymbolIx::from(self.engine.model.symbols.len());
            self.engine.reserve(1)?;
            // The declared member boundary is retained. Inlining its definition would
            // silently discard that nominal quantity/representation boundary.
            self.engine.model.symbols.push(Symbol {
                id: derived,
                definition: Some(transformed),
                ty: original.ty,
                role: Kind::Let,
                domain: original.domain,
                expression: original.expression,
                initial: None,
                lineage: original.lineage,
            });
            slot
        };
        let source = self.engine.construction.node_source(root)?.clone();
        let result = self
            .engine
            .construction
            .node(Node::Reference(Reference::Symbol(selected)), source)?;
        self.members.insert(id, result);
        Ok(result)
    }
    fn node(&mut self, root: NodeIx) -> Result<NodeIx> {
        if let Some(value) = self.nodes.get(&root) {
            return Ok(*value);
        }
        if self.depth >= self.engine.limits.depth || !self.active.insert(root) {
            return Err(ModelingError::Budget(
                "typed transformation cycle or depth".into(),
            ));
        }
        self.depth += 1;
        self.engine.checkpoint()?;
        let original = self.engine.construction.operation_at(root)?.clone();
        if let Node::Reference(Reference::Symbol(slot)) = &original {
            let id = self
                .engine
                .model
                .symbols
                .at(*slot)
                .ok_or_else(|| invalid(self.at, "transformed scalar is outside its actual owner"))?
                .id;
            let result = self.member(id, *slot, root)?;
            self.depth -= 1;
            self.active.remove(&root);
            self.nodes.insert(root, result);
            return Ok(result);
        }
        let transformed = match &original {
            Node::Number(_) => original.clone(),
            Node::Reference(Reference::Symbol(_)) => {
                return Err(invalid(
                    self.at,
                    "typed scalar transformation lost its member dispatch",
                ));
            }
            Node::Reference(Reference::Lexical(binding)) => {
                Node::Reference(Reference::Lexical(self.binder(*binding)?))
            }
            Node::Reference(Reference::Indexed { group, indices }) => {
                Node::Reference(Reference::Indexed {
                    group: *group,
                    indices: self.node_list(indices)?,
                })
            }
            Node::Neg(value) => Node::Neg(self.node(*value)?),
            Node::Indicator(predicate) => Node::Indicator(self.predicate(*predicate)?),
            Node::Binary { op, lhs, rhs } => Node::Binary {
                op: *op,
                lhs: self.node(*lhs)?,
                rhs: self.node(*rhs)?,
            },
            Node::Call { function, args } => Node::Call {
                function: *function,
                args: self.node_list(args)?,
            },
            Node::Apply { function, args } => Node::Apply {
                function: *function,
                args: self.node_list(args)?,
            },
            Node::Partial {
                function,
                wrt,
                args,
            } => Node::Partial {
                function: *function,
                wrt: wrt.clone(),
                args: self.node_list(args)?,
            },
            Node::Provider { selection, args } => Node::Provider {
                selection: *selection,
                args: self.node_list(args)?,
            },
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
                    .map(|slot| self.binder(*slot))
                    .collect::<Result<_>>()?,
                body: self.node(*body)?,
            },
            Node::Sequence {
                value,
                prerequisites,
            } => Node::Sequence {
                value: self.node(*value)?,
                prerequisites: self.node_list(prerequisites)?,
            },
            Node::Evidence { value, evidence } => Node::Evidence {
                value: self.node(*value)?,
                evidence: self.evidence(*evidence)?,
            },
            Node::Reduce {
                kind,
                binder,
                filter,
                body,
            } => Node::Reduce {
                kind: *kind,
                binder: self.binder(*binder)?,
                filter: filter.map(|root| self.predicate(root)).transpose()?,
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
                filter: filter.map(|root| self.predicate(root)).transpose()?,
                value: self.node(*value)?,
                accumulator: self.binder(*accumulator)?,
                item: self.binder(*item)?,
                step: self.node(*step)?,
            },
            Node::Derivative { body, domain } => Node::Derivative {
                body: self.node(*body)?,
                domain: *domain,
            },
        };
        let result = if transformed == original {
            root
        } else {
            let source = self.engine.construction.node_source(root)?.clone();
            let value = self.engine.construction.node(transformed, source)?;
            self.engine.construction.copy_facts(root, value)?;
            value
        };
        self.depth -= 1;
        self.active.remove(&root);
        self.nodes.insert(root, result);
        Ok(result)
    }
}
