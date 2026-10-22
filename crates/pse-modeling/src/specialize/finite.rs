// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual finite facts captured by expansion owners, before member hashing discards tuples.
use super::{DeclarationId, InstanceId, Result, Value, invalid};
use crate::{
    Type,
    resolved::{Carrier, CarrierKind},
};
use pse_ids::SemanticId;
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

/// One declared finite family in an actual instance and outer coordinate context.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FamilyKey {
    /// Actual definition instance, kept outside reusable body identities.
    pub instance: InstanceId,
    /// Original owning declaration.
    pub declaration: DeclarationId,
    /// Exact generated source field role of this family.
    pub role: String,
    /// Enclosing coordinates in declared order, before this family's own binders.
    pub parents: Vec<Value>,
}

/// One actual ragged axis occurrence, after its preceding coordinates have been selected.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DomainKey {
    /// Declared finite family.
    pub family: FamilyKey,
    /// Axis position in declared order.
    pub position: usize,
    /// This family's earlier selected coordinates, preserving actual ragged context.
    pub prefix: Vec<Value>,
}

/// An actual member selection, independent of its hashed identity.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemberKey {
    /// Actual definition instance.
    pub instance: InstanceId,
    /// Original member declaration.
    pub declaration: DeclarationId,
    /// Exact typed coordinates in declaration order.
    pub tuple: Vec<Value>,
}

/// Distinguish source absence from a lazy member that no selected consumer demanded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemberState {
    /// An actual numerical symbol, including an admitted alias to another symbol.
    Realized(SemanticId),
    /// The owning structural predicate explicitly removed this source occurrence.
    Absent,
    /// No numerical realization was demanded; no absence is inferred.
    NotDemanded,
}

/// The actual ordered reduction domain and exact filter-selected membership.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReductionSelection {
    /// Original declared carrier, including empty-set element meaning and source order.
    pub carrier: Carrier,
    /// Values whose static filters admitted a term, in semantic order.
    pub selected: Arc<[Value]>,
}
/// Immutable typed carriers and tuple realizations issued by actual specialization producers.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FiniteFacts {
    domains: BTreeMap<DomainKey, Carrier>,
    families: BTreeMap<FamilyKey, Arc<[Vec<Value>]>>,
    members: BTreeMap<MemberKey, SemanticId>,
    absent: BTreeSet<MemberKey>,
}
impl FiniteFacts {
    /// Actual domain type and semantic order, retaining empty sets' declared kind.
    pub fn domain(&self, key: &DomainKey) -> Result<&Carrier> {
        self.domains.get(key).ok_or_else(|| {
            invalid(
                key.family.declaration,
                "required finite domain fact is absent",
            )
        })
    }
    /// Exact expanded tuples, including an empty family without invented membership.
    pub fn family(&self, key: &FamilyKey) -> Result<&[Vec<Value>]> {
        self.families
            .get(key)
            .map(AsRef::as_ref)
            .ok_or_else(|| invalid(key.declaration, "required finite family fact is absent"))
    }
    /// Inspect actual realized, absent or undemanded state without converting absence to zero.
    pub fn member(&self, key: &MemberKey) -> MemberState {
        if let Some(id) = self.members.get(key) {
            MemberState::Realized(*id)
        } else if self.absent.contains(key) {
            MemberState::Absent
        } else {
            MemberState::NotDemanded
        }
    }
    /// A selected numerical dependency requires an actual symbol realization.
    pub fn require_member(&self, key: &MemberKey) -> Result<SemanticId> {
        match self.member(key) {
            MemberState::Realized(id) => Ok(id),
            MemberState::Absent => Err(invalid(
                key.declaration,
                "required numerical member is explicitly absent",
            )),
            MemberState::NotDemanded => Err(invalid(
                key.declaration,
                "required numerical member realization was not produced",
            )),
        }
    }
    /// Enumerate actual carrier facts for selected-region resolution and inspection.
    pub fn domains(&self) -> impl Iterator<Item = (&DomainKey, &Carrier)> {
        self.domains.iter()
    }
    /// Known retained extent; shared finite row cells are conservatively charged.
    pub fn retained_bytes(&self) -> usize {
        fn values(values: &[Value]) -> usize {
            values.iter().fold(
                values.len().saturating_mul(size_of::<Value>()),
                |bytes, value| bytes.saturating_add(value.retained_bytes()),
            )
        }
        fn family(key: &FamilyKey) -> usize {
            key.role.capacity().saturating_add(values(&key.parents))
        }
        self.domains
            .iter()
            .fold(size_of::<Self>(), |bytes, (key, carrier)| {
                bytes
                    .saturating_add(size_of::<(DomainKey, Carrier)>() + 96)
                    .saturating_add(family(&key.family))
                    .saturating_add(values(&key.prefix))
                    .saturating_add(crate::resolved::extent::ty(&carrier.element))
                    .saturating_add(values(&carrier.members))
            })
            .saturating_add(self.families.iter().fold(0usize, |bytes, (key, tuples)| {
                bytes
                    .saturating_add(size_of::<(FamilyKey, Arc<[Vec<Value>]>)>() + 96)
                    .saturating_add(family(key))
                    .saturating_add(tuples.iter().fold(
                        tuples.len().saturating_mul(size_of::<Vec<Value>>()),
                        |bytes, tuple| bytes.saturating_add(values(tuple)),
                    ))
            }))
            .saturating_add(self.members.iter().fold(0usize, |bytes, (key, _)| {
                bytes
                    .saturating_add(size_of::<(MemberKey, SemanticId)>() + 96)
                    .saturating_add(values(&key.tuple))
            }))
            .saturating_add(self.absent.iter().fold(0usize, |bytes, key| {
                bytes
                    .saturating_add(size_of::<MemberKey>() + 96)
                    .saturating_add(values(&key.tuple))
            }))
    }
}

/// Mutable only during one bounded specialization; no cell escapes in the published product.
#[derive(Debug, Default)]
pub(super) struct Collector(RefCell<FiniteFacts>);
impl Collector {
    pub(super) fn domain(
        &self,
        key: DomainKey,
        kind: CarrierKind,
        element: Type,
        members: &[Value],
    ) -> Result<()> {
        let mut facts = self.0.borrow_mut();
        if let Some(prior) = facts.domains.get(&key) {
            return if prior.kind == kind
                && prior.element == element
                && prior.members.as_ref() == members
            {
                Ok(())
            } else {
                Err(invalid(
                    key.family.declaration,
                    "one finite domain occurrence has conflicting actual facts",
                ))
            };
        }
        facts.domains.insert(
            key,
            Carrier {
                kind,
                element,
                members: members.to_vec().into(),
            },
        );
        Ok(())
    }
    pub(super) fn family(
        &self,
        key: FamilyKey,
        coordinates: &[Vec<(String, Value)>],
    ) -> Result<()> {
        let tuples = coordinates
            .iter()
            .map(|row| {
                row.iter()
                    .map(|(_, value)| value.clone())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let mut facts = self.0.borrow_mut();
        if facts
            .families
            .get(&key)
            .is_some_and(|prior| prior.as_ref() != tuples.as_slice())
        {
            return Err(invalid(
                key.declaration,
                "one finite family has conflicting tuple order or membership",
            ));
        }
        facts.families.entry(key).or_insert_with(|| tuples.into());
        Ok(())
    }
    pub(super) fn member(&self, key: MemberKey, id: SemanticId) -> Result<()> {
        let mut facts = self.0.borrow_mut();
        if facts.absent.contains(&key) || facts.members.get(&key).is_some_and(|prior| *prior != id)
        {
            return Err(invalid(
                key.declaration,
                "one finite member has conflicting realization facts",
            ));
        }
        facts.members.insert(key, id);
        Ok(())
    }
    pub(super) fn absent(&self, key: MemberKey) -> Result<()> {
        let mut facts = self.0.borrow_mut();
        if facts.members.contains_key(&key) {
            return Err(invalid(
                key.declaration,
                "a realized finite member cannot also be absent",
            ));
        }
        facts.absent.insert(key);
        Ok(())
    }
    /// Borrow only the actual declared family after its owner enumerated it.
    pub(super) fn actual_domains(
        &self,
        family: &FamilyKey,
    ) -> Result<Vec<(usize, Vec<Value>, Carrier)>> {
        let facts = self.0.borrow();
        if !facts.families.contains_key(family) {
            return Err(invalid(
                family.declaration,
                "indexed argument lacks its actual finite family",
            ));
        }
        Ok(facts
            .domains
            .iter()
            .filter(|(key, _)| &key.family == family)
            .map(|(key, carrier)| (key.position, key.prefix.clone(), carrier.clone()))
            .collect())
    }
    pub(super) fn remap(&self, old: SemanticId, new: SemanticId) {
        for id in self.0.borrow_mut().members.values_mut() {
            if *id == old {
                *id = new;
            }
        }
    }
    pub(super) fn merge(&self, incoming: &FiniteFacts) -> Result<()> {
        let mut facts = self.0.borrow_mut();
        for (key, value) in &incoming.domains {
            if facts.domains.get(key).is_some_and(|prior| prior != value) {
                return Err(invalid(
                    key.family.declaration,
                    "fragment finite domain conflicts with root",
                ));
            }
        }
        for (key, value) in &incoming.families {
            if facts.families.get(key).is_some_and(|prior| prior != value) {
                return Err(invalid(
                    key.declaration,
                    "fragment finite family conflicts with root",
                ));
            }
        }
        for (key, value) in &incoming.members {
            if facts.absent.contains(key)
                || facts.members.get(key).is_some_and(|prior| prior != value)
            {
                return Err(invalid(
                    key.declaration,
                    "fragment finite member conflicts with root",
                ));
            }
        }
        for key in &incoming.absent {
            if facts.members.contains_key(key) {
                return Err(invalid(
                    key.declaration,
                    "fragment finite absence conflicts with root",
                ));
            }
        }
        facts.domains.extend(incoming.domains.clone());
        facts.families.extend(incoming.families.clone());
        facts.members.extend(incoming.members.clone());
        facts.absent.extend(incoming.absent.clone());
        Ok(())
    }
    pub(super) fn finish(self) -> FiniteFacts {
        self.0.into_inner()
    }
}

impl MemberKey {
    pub(super) fn new(
        instance: InstanceId,
        declaration: DeclarationId,
        coordinates: &[(String, Value)],
    ) -> Self {
        Self {
            instance,
            declaration,
            tuple: coordinates.iter().map(|(_, value)| value.clone()).collect(),
        }
    }
}
impl super::Engine<'_, '_> {
    pub(super) fn finite_family(
        &self,
        instance: InstanceId,
        declaration: DeclarationId,
        role: &str,
        env: &super::Environment,
    ) -> FamilyKey {
        let owner = &self.p.declarations[&declaration];
        let parents = if role == crate::expression::member_index_role(owner) {
            Vec::new()
        } else {
            crate::expression::member_index_names(owner)
                .into_iter()
                .filter_map(|name| env.get(name).cloned())
                .collect()
        };
        FamilyKey {
            instance,
            declaration,
            role: role.into(),
            parents,
        }
    }
    pub(super) fn finite_domain_type(
        &self,
        at: DeclarationId,
        env: &super::Environment,
        role: &str,
        position: usize,
    ) -> Result<Type> {
        crate::expression::index_domain_type(
            self.p.static_at(at, role, position)?,
            &self.source_types(at, env)?,
            &self.model.function_contracts(self.p),
            self.c,
            at,
        )
    }
    pub(super) fn finite_domain(
        &self,
        family: &FamilyKey,
        position: usize,
        prefix: &[(String, Value)],
        ty: Type,
        members: &[Value],
    ) -> Result<()> {
        let (kind, element) = match ty {
            Type::Set(element) => (CarrierKind::OrderedSet, *element),
            Type::Continuous(domain, element) => (CarrierKind::Continuous { domain }, *element),
            _ => {
                return Err(invalid(
                    family.declaration,
                    "finite domain has no declared carrier type",
                ));
            }
        };
        self.finite_collector.domain(
            DomainKey {
                family: family.clone(),
                position,
                prefix: prefix.iter().map(|(_, value)| value.clone()).collect(),
            },
            kind,
            element,
            members,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_absence_is_distinct_from_undemanded_and_cannot_alias_a_realization() {
        let collector = Collector::default();
        let key = MemberKey {
            instance: InstanceId::from_bytes([1; 16]),
            declaration: DeclarationId::from_bytes([2; 16]),
            tuple: vec![Value::Integer(3)],
        };
        assert_eq!(collector.0.borrow().member(&key), MemberState::NotDemanded);
        collector.absent(key.clone()).unwrap();
        assert!(
            collector
                .member(key.clone(), SemanticId::from_bytes([4; 16]))
                .is_err()
        );
        let facts = collector.finish();
        assert_eq!(facts.member(&key), MemberState::Absent);
        assert!(facts.require_member(&key).is_err());
    }
}
