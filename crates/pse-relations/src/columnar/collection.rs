// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Heterogeneous generated columns with reserve-before-growth ownership.

use std::any::{Any, TypeId};
use std::collections::{BTreeMap, btree_map::Entry};
use std::fmt;

use pse_ids::{CancellationToken, CanonError, MemoryReserver, Reservation, ReservationLease};
use pse_schema::{Registry, model::RelationKey};

use super::{FieldCheckedBatch, RelationRow, allocation_add, mismatch};
use crate::RelationError;

type ErasedBuilder = Box<dyn Any + Send>;
type Finish = fn(ErasedBuilder) -> Result<FieldCheckedBatch, RelationError>;

struct Columns {
    // Drop native columns before releasing their construction reservation.
    builder: ErasedBuilder,
    reservation: Box<dyn Reservation>,
    key: RelationKey,
    finish: Finish,
}

/// Appends generated rows directly to their declared native Arrow builders.
///
/// Type erasure stores builders only; no rows or field values pass through an erased
/// representation. Generated local checks establish [`FieldCheckedBatch`]. Relational
/// key, reference, ordinal and domain obligations remain separate native plans.
///
/// The caller accounts for the typed row before handing it to this collection. Each
/// append reserves native column growth first. Final arrays, including detached nested
/// children, retain the same reservation after the collection has been consumed.
pub struct Collection<'a> {
    columns: BTreeMap<TypeId, Columns>,
    registry: &'a Registry,
    reserver: &'a dyn MemoryReserver,
    cancel: &'a CancellationToken,
}

impl fmt::Debug for Collection<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Collection")
            .field("relations", &self.columns.len())
            .finish_non_exhaustive()
    }
}

impl<'a> Collection<'a> {
    /// Starts an empty collection using the caller's shared resource configuration.
    pub fn new(
        registry: &'a Registry,
        reserver: &'a dyn MemoryReserver,
        cancel: &'a CancellationToken,
    ) -> Self {
        Self {
            columns: BTreeMap::new(),
            registry,
            reserver,
            cancel,
        }
    }

    /// Includes the exact generated declaration even when no rows will be appended.
    ///
    /// # Errors
    /// Cancellation, resource exhaustion, or a mismatched generated declaration.
    pub fn ensure<T: RelationRow>(&mut self) -> Result<(), RelationError> {
        self.cancel.checkpoint()?;
        if self.columns.contains_key(&TypeId::of::<T>()) {
            return Ok(());
        }
        let mut reservation = self.reserver.open("relations:generated-columns");
        reservation
            .try_grow(allocation_add(
                T::builder_allocation_size(),
                8 * size_of::<Columns>(),
            )?)
            .map_err(CanonError::from)?;
        self.cancel.checkpoint()?;
        let key = T::relation(self.registry)?.key;
        if self.columns.values().any(|columns| columns.key == key) {
            return Err(mismatch("one generated row type per collected relation"));
        }
        let builder = Box::new(T::builder(self.registry, 0)?);
        self.columns.insert(
            TypeId::of::<T>(),
            Columns {
                builder,
                reservation,
                key,
                finish: finish::<T>,
            },
        );
        Ok(())
    }

    /// Checks and appends one typed row after reserving its native storage growth.
    /// No key deduplication or sorting is implied by construction.
    ///
    /// # Errors
    /// Cancellation, resource exhaustion, declaration mismatch, or invalid local values.
    pub fn push<T: RelationRow>(&mut self, row: T) -> Result<(), RelationError> {
        self.ensure::<T>()?;
        let extent = row
            .allocation_size()?
            .max(T::minimum_row_allocation_size())
            .checked_mul(8)
            .ok_or_else(|| mismatch("representable generated column growth"))?;
        let columns = self
            .columns
            .get_mut(&TypeId::of::<T>())
            .ok_or_else(|| mismatch("an initialized generated builder"))?;
        // Covers geometric buffer growth, dictionary interning and simultaneous finish
        // scratch. The generated extent also counts masked nested child storage.
        columns
            .reservation
            .try_grow(extent)
            .map_err(CanonError::from)?;
        self.cancel.checkpoint()?;
        let builder = columns
            .builder
            .downcast_mut::<T::Builder>()
            .ok_or_else(|| mismatch("the exact generated builder for this row"))?;
        T::push(builder, row)
    }

    /// Completes each generated builder and transfers its reservation into its arrays.
    ///
    /// # Errors
    /// Cancellation, Arrow construction failure, or insufficient reserved buffer extent.
    pub fn finish(self) -> Result<BTreeMap<RelationKey, FieldCheckedBatch>, RelationError> {
        let mut result = BTreeMap::new();
        for columns in self.columns.into_values() {
            self.cancel.checkpoint()?;
            let Columns {
                builder,
                reservation,
                key,
                finish,
            } = columns;
            let mut checked = finish(builder)?;
            let spec = self
                .registry
                .relation_by_id(checked.relation_id())
                .ok_or_else(|| mismatch("a generated result declared in this registry"))?;
            if spec.key != key {
                return Err(mismatch("the declared generated builder result"));
            }
            checked.check_declaration(self.registry, spec)?;
            checked.batch = pse_ids::owned_buffer::attach_reservation(
                checked.batch,
                ReservationLease::new(reservation),
            )?;
            checked.leased = true;
            match result.entry(key) {
                Entry::Vacant(entry) => {
                    entry.insert(checked);
                }
                Entry::Occupied(_) => return Err(mismatch("one completed batch per relation")),
            }
        }
        self.cancel.checkpoint()?;
        Ok(result)
    }
}

fn finish<T: RelationRow>(builder: ErasedBuilder) -> Result<FieldCheckedBatch, RelationError> {
    let builder = builder
        .downcast::<T::Builder>()
        .map_err(|_| mismatch("the exact generated builder at completion"))?;
    T::finish(*builder)
}
