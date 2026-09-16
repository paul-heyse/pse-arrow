// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use crate::TemplateError;
use pse_ids::SemanticId;
use pse_mathir::{DomainRef, ExprGraph, GuardRef, NodeId, Payload, ValueRef, index::DomainFacts};
use pse_quantity::{BoundIndexId, BoundIndexRef, DomainId, IndexSet, QuantityTypeId};
use std::collections::{BTreeMap, BTreeSet};

/// An admitted concrete replacement for one exact declared source reference.
#[derive(Clone, Debug)]
pub enum BindingValue {
    /// Actual instantiated symbol identity (including aliases to an existing symbol).
    Symbol(SemanticId),
    /// Declared scalar constant. The ordinary graph validates its payload and values.
    Literal {
        /// Integer or finite unit-bearing literal payload.
        payload: Payload,
        /// Complete declared type claim, independently checked by P10.
        quantity: Option<QuantityTypeId>,
    },
    /// Actual finite domain member; valid only as an indexed coordinate.
    Member(SemanticId),
}

/// Actual instantiated group meaning, not a label or shape fingerprint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupBinding {
    /// Actual group identity.
    pub group: SemanticId,
    /// Exact domains in declared axis order.
    pub axes: Vec<DomainId>,
    /// Complete admitted tuple-to-symbol correspondence.
    pub members: BTreeMap<Vec<SemanticId>, SemanticId>,
}

/// Actual member/group reached by a checked instance-relative path.
#[derive(Clone, Debug)]
pub enum PathBinding {
    /// Scalar target or a fully bound configuration value.
    Value(BindingValue),
    /// Complete cross-child or local group with its exact ordered factors and providers.
    Group(GroupBinding),
}

/// Actual finite Boolean mask and the lexical coordinate map at one use.
#[derive(Clone, Debug)]
pub struct PredicateMask {
    /// Declared mask group with complete actual finite members.
    pub group: GroupBinding,
    /// Exact source binder identity for every ordered group axis.
    pub coordinates: Vec<BoundIndexId>,
}

/// A finite evaluated subscript correspondence, supplied by the shared coordinate evaluator.
#[derive(Clone, Debug)]
pub struct EvaluatedGather {
    /// Complete actual free-tuple to provider map.
    pub group: GroupBinding,
    /// Source binders in the exact order of the new group axes.
    pub coordinates: Vec<BoundIndexId>,
}

/// One immutable source occurrence's complete actual binding environment.
/// The Arrow adapter constructs these values from admitted generated row views.
#[derive(Clone, Debug)]
pub struct InstantiationEnvironment {
    /// Actual owning instance; diagnostic labels never enter this key.
    pub instance: SemanticId,
    /// Explicit complete reference bindings, including template parameters and aliases.
    pub values: BTreeMap<ValueRef, BindingValue>,
    /// Optional selected-method coefficients with no actual data; a demanded read refuses.
    pub unbound_parameters: BTreeSet<SemanticId>,
    /// Actual domains for each used template-local or already actual domain reference.
    pub domains: BTreeMap<DomainRef, DomainId>,
    /// Actual domain content and coordinate meaning.
    pub domain_facts: BTreeMap<DomainId, DomainFacts>,
    /// Source lexical identity to actual occurrence binder and domain.
    pub binders: BTreeMap<BoundIndexId, BoundIndexRef>,
    /// The equation's actual free-index environment, before inner reductions introduce binders.
    pub free_indices: IndexSet,
    /// Explicitly bound outer coordinates when realizing one scalar symbol member body.
    pub fixed_indices: BTreeMap<BoundIndexId, SemanticId>,
    /// Exact source group declaration to realized group correspondence.
    pub groups: BTreeMap<SemanticId, GroupBinding>,
    /// Exact source occurrence paths after actual selected-template traversal.
    pub paths: BTreeMap<(SemanticId, u64), PathBinding>,
    /// Admitted source node to its exact finite arithmetic subscript interpretation.
    pub evaluated_gathers: BTreeMap<NodeId, EvaluatedGather>,
    /// Explicit numeric coordinate values mapped to actual domain members.
    pub integer_members: BTreeMap<(DomainId, i64), SemanticId>,
    /// Decided predicate outcomes for this actual instance.
    pub predicates: BTreeMap<(SemanticId, u64), bool>,
    /// Index-dependent outcomes represented by declared actual Boolean symbol groups.
    pub predicate_masks: BTreeMap<(SemanticId, u64), PredicateMask>,
    /// Exact source kernel-binding identity to realized binding identity.
    pub kernel_bindings: BTreeMap<SemanticId, SemanticId>,
    /// Exact implicit-system declaration to actual instance correspondence.
    pub implicit_systems: BTreeMap<SemanticId, SemanticId>,
}
impl InstantiationEnvironment {
    /// Empty explicit environment; missing demanded bindings fail rather than default.
    pub fn new(instance: SemanticId) -> Self {
        Self {
            instance,
            values: BTreeMap::new(),
            unbound_parameters: BTreeSet::new(),
            domains: BTreeMap::new(),
            domain_facts: BTreeMap::new(),
            binders: BTreeMap::new(),
            free_indices: IndexSet::new(),
            fixed_indices: BTreeMap::new(),
            groups: BTreeMap::new(),
            paths: BTreeMap::new(),
            evaluated_gathers: BTreeMap::new(),
            integer_members: BTreeMap::new(),
            predicates: BTreeMap::new(),
            predicate_masks: BTreeMap::new(),
            kernel_bindings: BTreeMap::new(),
            implicit_systems: BTreeMap::new(),
        }
    }
    pub(crate) fn invalid(&self, detail: impl Into<String>) -> TemplateError {
        TemplateError::Binding {
            instance: self.instance,
            detail: detail.into(),
        }
    }
    pub(crate) fn require_group_bindings(&self, group: &GroupBinding) -> Result<(), TemplateError> {
        if group
            .members
            .values()
            .any(|symbol| self.unbound_parameters.contains(symbol))
        {
            return Err(self.invalid("demanded method parameter coordinate has no actual data"));
        }
        Ok(())
    }
    pub(crate) fn domain(&self, reference: &DomainRef) -> Result<DomainId, TemplateError> {
        let id = self
            .domains
            .get(reference)
            .copied()
            .or({
                if let DomainRef::Actual(id) = reference {
                    Some(*id)
                } else {
                    None
                }
            })
            .ok_or_else(|| self.invalid(format!("domain {reference:?} is unbound")))?;
        self.domain_facts
            .get(&id)
            .ok_or_else(|| self.invalid("actual domain facts are absent"))?;
        Ok(id)
    }
    pub(crate) fn predicate(
        &self,
        source_id: SemanticId,
        predicate: u64,
    ) -> Result<bool, TemplateError> {
        self.predicates.get(&(source_id, predicate)).copied().ok_or(
            TemplateError::GuardUndecidable {
                instance: self.instance,
                source_id,
                predicate,
            },
        )
    }
    /// Materialize a decided scalar predicate or an actual finite Boolean mask.
    /// # Errors
    /// Missing actual outcomes, incomplete masks or use outside the declared lexical domain.
    pub fn predicate_guard(
        &self,
        source_id: SemanticId,
        predicate_id: u64,
        graph: &mut ExprGraph,
        indices: &IndexSet,
    ) -> Result<NodeId, TemplateError> {
        self.guard(
            Some(GuardRef::Predicate {
                source_id,
                predicate_id,
            }),
            graph,
            &BTreeMap::new(),
            indices,
        )?
        .and_then(GuardRef::math)
        .ok_or_else(|| self.invalid("predicate did not produce an actual mathematical guard"))
    }
    pub(crate) fn guard(
        &self,
        guard: Option<GuardRef>,
        graph: &mut ExprGraph,
        mapping: &BTreeMap<NodeId, NodeId>,
        indices: &IndexSet,
    ) -> Result<Option<GuardRef>, TemplateError> {
        guard
            .map(|guard| match guard {
                GuardRef::Math(node) => mapping
                    .get(&node)
                    .copied()
                    .map(GuardRef::Math)
                    .ok_or_else(|| self.invalid("guard dependency was not rebuilt")),
                GuardRef::Predicate {
                    source_id,
                    predicate_id,
                } => {
                    if let Some(mask) = self.predicate_masks.get(&(source_id, predicate_id)) {
                        if mask.coordinates.len() != mask.group.axes.len() {
                            return Err(self.invalid("predicate mask coordinate arity differs"));
                        }
                        let coordinates = mask
                            .coordinates
                            .iter()
                            .zip(&mask.group.axes)
                            .enumerate()
                            .map(|(position, (source, domain))| {
                                let actual = self
                                    .binders
                                    .get(source)
                                    .ok_or_else(|| self.invalid("mask binder absent"))?;
                                if actual.domain != *domain || !indices.contains(actual) {
                                    return Err(self.invalid(
                                        "mask binder is outside its exact lexical domain",
                                    ));
                                }
                                Ok((
                                    actual.bound_index,
                                    u16::try_from(position)
                                        .map_err(|_| self.invalid("mask position overflow"))?,
                                ))
                            })
                            .collect::<Result<Vec<_>, TemplateError>>()?;
                        graph
                            .insert(
                                pse_mathir::Opcode::Gather,
                                Payload::Gather {
                                    group: mask.group.group,
                                    coordinate_map: coordinates,
                                },
                                &[],
                                Some(self.instance),
                            )
                            .map(GuardRef::Math)
                            .map_err(Into::into)
                    } else {
                        graph
                            .int_const(i64::from(self.predicate(source_id, predicate_id)?))
                            .map(GuardRef::Math)
                            .map_err(Into::into)
                    }
                }
            })
            .transpose()
    }
    /// Validate actual binder and group membership contracts before rebuilding any node.
    /// # Errors
    /// Missing domains, repeated actual binders, invalid tuple arity or foreign members.
    pub fn validate(&self) -> Result<(), TemplateError> {
        if self
            .predicates
            .keys()
            .any(|key| self.predicate_masks.contains_key(key))
        {
            return Err(self.invalid("predicate has both scalar and indexed interpretations"));
        }
        let mut identities = BTreeSet::new();
        for binding in self.binders.values() {
            let facts = self
                .domain_facts
                .get(&binding.domain)
                .ok_or_else(|| self.invalid("binder domain absent"))?;
            if facts.kind != binding.kind || !identities.insert(binding.bound_index) {
                return Err(
                    self.invalid("binder has wrong domain kind or duplicate actual identity")
                );
            }
        }
        for (source, member) in &self.fixed_indices {
            let binding = self
                .binders
                .get(source)
                .ok_or_else(|| self.invalid("fixed index lacks an actual source binder"))?;
            if self.free_indices.contains(binding)
                || !self
                    .domain_facts
                    .get(&binding.domain)
                    .is_some_and(|facts| facts.members.contains(member))
            {
                return Err(self
                    .invalid("fixed index overlaps free scope or names a foreign actual member"));
            }
        }
        for binding in &self.free_indices {
            if !self.binders.values().any(|value| value == binding) {
                return Err(self.invalid("equation free index lacks an exact declared binding"));
            }
        }
        for group in self
            .groups
            .values()
            .chain(self.paths.values().filter_map(|path| match path {
                PathBinding::Group(group) => Some(group),
                PathBinding::Value(_) => None,
            }))
            .chain(self.predicate_masks.values().map(|mask| &mask.group))
            .chain(self.evaluated_gathers.values().map(|gather| &gather.group))
        {
            for tuple in group.members.keys() {
                if tuple.len() != group.axes.len() {
                    return Err(self.invalid("group member arity differs"));
                }
                for (member, axis) in tuple.iter().zip(&group.axes) {
                    let facts = self
                        .domain_facts
                        .get(axis)
                        .ok_or_else(|| self.invalid("group domain absent"))?;
                    if !facts.members.contains(member) {
                        return Err(self.invalid("group contains a foreign domain member"));
                    }
                }
            }
        }
        Ok(())
    }
}
