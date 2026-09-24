// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Free-index identity: which binders an expression still ranges over
//! (blueprint §6.3, §7.1 constraint 2, §8.3).
//!
//! Indexed mathematics survives until a backend requires scalarization, so a typed node
//! carries not only a quantity type but the set of *bound indices* it is still free in. A
//! species balance over 1 000 cells is one node whose free-index set contains the cell
//! binder; P12 is what turns that into 1 000 rows.
//!
//! The distinction this module exists to keep is between a **domain** and a **binder**. A
//! `Gather` of `x[i]` and a `Gather` of `x[j]` over the same species domain have different
//! index identities and may not be added, which is why [`BoundIndexRef`] carries a
//! [`BoundIndexId`] beside its [`DomainId`]: §8.3's "same index identities" is an identity
//! comparison, not a shape comparison.

use std::collections::BTreeMap;

use crate::enums::DomainKind;
use crate::ids::{BoundIndexId, DomainId};

/// One binder an expression is free in (blueprint §6.9 `bound_index_id`).
///
/// Ordering is lexicographic on the fields in declaration order, so `bound_index` is the
/// primary key and a [`BTreeMap`] of these iterates in binder-identity order. A
/// well-formed set holds at most one entry per `bound_index`; [`IndexSet::insert`] reports
/// a second entry for the same binder rather than overwriting it, because two domains for
/// one binder is a compiler bug, not a merge.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct BoundIndexRef {
    /// The binding occurrence: which `i` this is, not which set it ranges over.
    pub bound_index: BoundIndexId,
    /// The domain the binder ranges over.
    pub domain: DomainId,
    /// What that domain indexes, carried so a check does not need the registry.
    pub kind: DomainKind,
}

impl BoundIndexRef {
    /// A reference to one binder over one domain.
    pub const fn new(bound_index: BoundIndexId, domain: DomainId, kind: DomainKind) -> Self {
        Self {
            bound_index,
            domain,
            kind,
        }
    }
}

/// The set of binders an expression is free in (blueprint §8.3 "index identities").
///
/// Backed by a [`BTreeMap`], so iteration order is binder-identity order and never
/// insertion order or hash order: the set reaches derivations and stage keys, and §5.3's
/// closing rule forbids a hash-container order from deciding what those contain.
///
/// ```
/// use pse_ids::SemanticId;
/// use pse_quantity::{BoundIndexId, BoundIndexRef, DomainId, DomainKind, IndexSet};
///
/// let cell = BoundIndexRef::new(
///     BoundIndexId::from_id(SemanticId::from_bytes([1; 16])),
///     DomainId::from_id(SemanticId::from_bytes([9; 16])),
///     DomainKind::Cell,
/// );
/// let mut set = IndexSet::new();
/// assert!(set.insert(cell).is_ok());
/// assert!(set.contains(&cell));
/// assert_eq!(set.len(), 1);
/// ```
#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct IndexSet(BTreeMap<BoundIndexId, BoundIndexRef>);

/// Two different domains were bound to one binder identity.
///
/// Not a [`crate::QuantityError`]: this is a malformed index set, reported to the caller
/// that is assembling it so the caller can attach the node it came from.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BinderConflict {
    /// The entry already in the set.
    pub existing: BoundIndexRef,
    /// The entry that was offered for the same binder.
    pub offered: BoundIndexRef,
}

impl IndexSet {
    /// The empty index set: a scalar.
    pub const fn new() -> Self {
        Self(BTreeMap::new())
    }

    /// Adds a binder.
    ///
    /// Returns `Ok(true)` when the binder was new, `Ok(false)` when the identical entry was
    /// already present.
    ///
    /// # Errors
    ///
    /// [`BinderConflict`] when the set already binds this identity to a different domain or
    /// kind.
    pub fn insert(&mut self, entry: BoundIndexRef) -> Result<bool, BinderConflict> {
        if let Some(existing) = self.get(entry.bound_index) {
            if *existing == entry {
                return Ok(false);
            }
            return Err(BinderConflict {
                existing: *existing,
                offered: entry,
            });
        }
        self.0.insert(entry.bound_index, entry);
        Ok(true)
    }

    /// The entry for a binder identity, if the set binds it.
    pub fn get(&self, bound_index: BoundIndexId) -> Option<&BoundIndexRef> {
        self.0.get(&bound_index)
    }

    /// Is this exact entry in the set?
    pub fn contains(&self, entry: &BoundIndexRef) -> bool {
        self.0.get(&entry.bound_index) == Some(entry)
    }

    /// Is this binder identity in the set, whatever domain it ranges over?
    pub fn contains_index(&self, bound_index: BoundIndexId) -> bool {
        self.get(bound_index).is_some()
    }

    /// The binders in either set.
    ///
    /// # Errors
    ///
    /// [`BinderConflict`] when the two sets bind one identity to different domains — the
    /// case §8.3 refuses rather than merges.
    pub fn union(&self, other: &Self) -> Result<Self, BinderConflict> {
        let mut out = self.clone();
        for entry in other.0.values() {
            out.insert(*entry)?;
        }
        Ok(out)
    }

    /// The binders in this set that the other set does not bind.
    ///
    /// Difference is by binder identity, so removing `i` removes it whatever domain the
    /// other set recorded for it; this is what a reduction over `i` does to its body.
    #[must_use]
    pub fn difference(&self, other: &Self) -> Self {
        Self(
            self.0
                .iter()
                .filter(|(key, _)| !other.contains_index(**key))
                .map(|(key, value)| (*key, *value))
                .collect(),
        )
    }

    /// Is every binder of this set bound by `other` to the same domain?
    pub fn is_subset(&self, other: &Self) -> bool {
        self.iter().all(|entry| other.contains(entry))
    }

    /// The binders, in binder-identity order.
    pub fn iter(&self) -> std::collections::btree_map::Values<'_, BoundIndexId, BoundIndexRef> {
        self.0.values()
    }

    /// How many binders the set holds.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Is this a scalar — no free indices at all?
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl IndexSet {
    /// Collects binders without dropping conflicting identities.
    ///
    /// # Errors
    /// Returns the first [`BinderConflict`] encountered.
    pub fn try_from_iter(
        iter: impl IntoIterator<Item = BoundIndexRef>,
    ) -> Result<Self, BinderConflict> {
        let mut out = Self::new();
        for entry in iter {
            out.insert(entry)?;
        }
        Ok(out)
    }
}

impl<'a> IntoIterator for &'a IndexSet {
    type Item = &'a BoundIndexRef;
    type IntoIter = std::collections::btree_map::Values<'a, BoundIndexId, BoundIndexRef>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use pse_ids::SemanticId;

    use super::{BoundIndexRef, IndexSet};
    use crate::enums::DomainKind;
    use crate::ids::{BoundIndexId, DomainId};

    fn binder(index: u8, domain: u8, kind: DomainKind) -> BoundIndexRef {
        BoundIndexRef::new(
            BoundIndexId::from_id(SemanticId::from_bytes([index; 16])),
            DomainId::from_id(SemanticId::from_bytes([domain; 16])),
            kind,
        )
    }

    #[test]
    fn an_empty_set_is_a_scalar() {
        let set = IndexSet::new();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
        assert_eq!(set.iter().count(), 0);
    }

    #[test]
    fn inserting_the_same_entry_twice_is_idempotent() {
        let entry = binder(1, 9, DomainKind::Species);
        let mut set = IndexSet::new();
        assert_eq!(set.insert(entry), Ok(true));
        assert_eq!(set.insert(entry), Ok(false));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn one_binder_may_not_range_over_two_domains() {
        let species = binder(1, 9, DomainKind::Species);
        let cells = binder(1, 8, DomainKind::Cell);
        let mut set = IndexSet::new();
        assert_eq!(set.insert(species), Ok(true));
        let conflict = set
            .insert(cells)
            .expect_err("a binder conflict is reported");
        assert_eq!(conflict.existing, species);
        assert_eq!(conflict.offered, cells);
    }

    #[test]
    fn union_collects_both_sides() {
        let species = binder(1, 9, DomainKind::Species);
        let phase = binder(2, 7, DomainKind::Phase);
        let left = IndexSet::try_from_iter([species]).expect("consistent binders");
        let right = IndexSet::try_from_iter([phase]).expect("consistent binders");
        let both = left.union(&right).expect("disjoint binders unite");
        assert_eq!(both.len(), 2);
        assert!(both.contains(&species));
        assert!(both.contains(&phase));
        assert_eq!(both.union(&left), Ok(both.clone()));
    }

    #[test]
    fn union_refuses_a_binder_conflict() {
        let left = IndexSet::try_from_iter([binder(1, 9, DomainKind::Species)])
            .expect("consistent binders");
        let right =
            IndexSet::try_from_iter([binder(1, 8, DomainKind::Cell)]).expect("consistent binders");
        assert!(left.union(&right).is_err());
    }

    /// Reducing over a binder removes it from the body's free indices.
    #[test]
    fn difference_removes_by_binder_identity() {
        let species = binder(1, 9, DomainKind::Species);
        let phase = binder(2, 7, DomainKind::Phase);
        let body = IndexSet::try_from_iter([species, phase]).expect("consistent binders");
        let bound =
            IndexSet::try_from_iter([binder(1, 8, DomainKind::Cell)]).expect("consistent binders");
        let remaining = body.difference(&bound);
        assert_eq!(remaining.len(), 1);
        assert!(remaining.contains(&phase));
        assert!(!remaining.contains_index(species.bound_index));
    }

    #[test]
    fn subset_compares_complete_entries() {
        let species = binder(1, 9, DomainKind::Species);
        let phase = binder(2, 7, DomainKind::Phase);
        let small = IndexSet::try_from_iter([species]).expect("consistent binders");
        let large = IndexSet::try_from_iter([species, phase]).expect("consistent binders");
        assert!(small.is_subset(&large));
        assert!(!large.is_subset(&small));
        assert!(IndexSet::new().is_subset(&small));
    }

    /// Iteration order is binder-identity order, not insertion order.
    #[test]
    fn iteration_order_is_binder_identity_order() {
        let first = binder(1, 9, DomainKind::Species);
        let second = binder(2, 7, DomainKind::Phase);
        let forwards = IndexSet::try_from_iter([first, second]).expect("consistent binders");
        let backwards = IndexSet::try_from_iter([second, first]).expect("consistent binders");
        let order: Vec<_> = forwards.iter().copied().collect();
        let reverse_order: Vec<_> = backwards.iter().copied().collect();
        assert_eq!(order, vec![first, second]);
        assert_eq!(order, reverse_order);
    }
}

#[cfg(test)]
mod construction_tests {
    use super::*;
    use pse_ids::SemanticId;
    #[test]
    fn collection_rejects_conflicts_in_both_orders() {
        let a = BoundIndexRef::new(
            BoundIndexId::from_id(SemanticId::NIL),
            DomainId::from_id(SemanticId::NIL),
            DomainKind::Species,
        );
        let b = BoundIndexRef {
            kind: DomainKind::Cell,
            ..a
        };
        assert!(IndexSet::try_from_iter([a, b]).is_err());
        assert!(IndexSet::try_from_iter([b, a]).is_err());
        assert_eq!(
            IndexSet::try_from_iter([a, a]).expect("same binder").len(),
            1
        );
    }
}
