// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Immutable Arrow buffers whose last reader retains their reservation (ADR-0055).
//!
//! These helpers establish allocation ownership and Arrow representation validity;
//! they do not establish schema-registry, key, reference or domain validity. A producer
//! reserves before allocating and attaches that reservation before releasing a batch.
//! Query export retains native allocations and attaches an explicit result claim without
//! copying values. This cannot retroactively account native allocations before export.

use std::collections::BTreeMap;
use std::ptr::NonNull;
use std::sync::{Arc, Mutex, Weak};

use arrow::array::{ArrayData, make_array};
use arrow_array::{RecordBatch, RecordBatchOptions};
use arrow_buffer::{BooleanBuffer, Buffer, NullBuffer};
use bytes::Bytes;

use crate::{
    AllocationLease, CancellationToken, CanonError, Envelope, EnvelopeBound, MemoryPool,
    ReserveError,
};

/// Immutable Arrow storage with an attached result reservation. Construction is
/// operation-backed; this owner carries no schema or semantic validity claim.
#[derive(Clone, Debug)]
pub struct OwnedRecordBatch {
    batch: RecordBatch,
    ownership: AllocationScope,
}

impl OwnedRecordBatch {
    /// Rename native fields without changing their arrays or allocation owners.
    /// This carries allocation evidence only, not semantic field admission.
    /// # Errors
    /// The replacement schema is incompatible with the actual arrays.
    pub fn with_schema(self, schema: arrow_schema::SchemaRef) -> Result<Self, CanonError> {
        let batch = RecordBatch::try_new_with_options(
            schema,
            self.batch.columns().to_vec(),
            &RecordBatchOptions::new().with_row_count(Some(self.batch.num_rows())),
        )?;
        Ok(Self {
            batch,
            ownership: self.ownership,
        })
    }
    /// Admit an unproven result through a reserved copy. Internal producers use
    /// their explicit allocation scope or transfer an already owned batch.
    /// # Errors
    /// Cancellation, extent overflow, resource refusal or Arrow validation failure.
    pub fn export(
        batch: RecordBatch,
        pool: &Arc<dyn MemoryPool>,
        cancel: &CancellationToken,
    ) -> Result<Self, CanonError> {
        let owned = Self::copy(&batch, pool, cancel);
        drop(batch);
        owned
    }
    /// Copy external storage into independent, pre-admitted native allocations.
    /// # Errors
    /// Cancellation, extent overflow, resource refusal or Arrow validation failure.
    pub fn copy(
        batch: &RecordBatch,
        pool: &Arc<dyn MemoryPool>,
        cancel: &CancellationToken,
    ) -> Result<Self, CanonError> {
        copy_owned(batch, "result:owned-copy", pool, cancel, Envelope::DEFAULT)
    }
    /// Attach a producer's pre-acquired native allocation reservation.
    /// # Errors
    /// Insufficient reserved extent or invalid Arrow representation.
    pub fn from_reserved(
        batch: RecordBatch,
        reservation: crate::MemoryReservation,
    ) -> Result<Self, CanonError> {
        AllocationScope::default().attach(batch, Some(reservation), None)
    }
    /// Exact owner scope; importing it never transfers the originating pool charge.
    pub const fn ownership(&self) -> &AllocationScope {
        &self.ownership
    }
    /// Full retained extents of this chunk's actual allocation owners.
    /// # Errors
    /// Missing provenance, poisoned ownership state or extent overflow.
    pub fn retained_bytes(&self) -> Result<usize, CanonError> {
        self.ownership.retained_bytes(&self.batch)
    }
    /// Project unchanged Arrow columns and their allocation evidence.
    /// # Errors
    /// An out-of-range column.
    pub fn project(&self, positions: &[usize]) -> Result<Self, CanonError> {
        Ok(Self {
            batch: self.batch.project(positions)?,
            ownership: self.ownership.clone(),
        })
    }
    /// Select rows while retaining the full backing allocation extent.
    /// # Errors
    /// An out-of-range row selection.
    pub fn slice(&self, offset: usize, length: usize) -> Result<Self, CanonError> {
        if offset
            .checked_add(length)
            .is_none_or(|end| end > self.batch.num_rows())
        {
            return Err(CanonError::Internal(
                "owned slice outside row domain".into(),
            ));
        }
        Ok(Self {
            batch: self.batch.slice(offset, length),
            ownership: self.ownership.clone(),
        })
    }
    /// Deliberately compact a small view; source and destination coexist under quota.
    /// # Errors
    /// Cancellation, allocation or Arrow failure.
    pub fn compact(
        &self,
        pool: &Arc<dyn MemoryPool>,
        cancel: &CancellationToken,
    ) -> Result<Self, CanonError> {
        // Native kernels trim primitive/string offsets; nested values use Arrow's
        // own slicing/copy semantics and retain their honestly admitted extents.
        let extent = self
            .batch
            .get_array_memory_size()
            .checked_mul(8)
            .and_then(|n| n.checked_add(self.batch.num_rows().checked_mul(8)?))
            .and_then(|n| n.checked_add(4096))
            .ok_or_else(size_overflow)?;
        let reservation = crate::MemoryConsumer::new("result:compact").register(pool);
        reservation.try_grow(extent)?;
        cancel.checkpoint()?;
        let indices = arrow_array::UInt64Array::from_iter_values(0..self.batch.num_rows() as u64);
        let columns = self
            .batch
            .columns()
            .iter()
            .map(|array| arrow::compute::take(array.as_ref(), &indices, None))
            .collect::<Result<Vec<_>, _>>()?;
        let batch = RecordBatch::try_new_with_options(
            self.batch.schema(),
            columns,
            &RecordBatchOptions::new().with_row_count(Some(self.batch.num_rows())),
        )?;
        let scope = AllocationScope::default();
        scope.import(self)?;
        let compacted = scope.attach_reserved(batch, reservation)?;
        if compacted.retained_bytes()? < self.retained_bytes()? {
            Ok(compacted)
        } else {
            Ok(self.clone())
        }
    }
    /// Replace only schema-level metadata, retaining exact fields and row count.
    /// # Errors
    /// Invalid Arrow representation.
    pub fn with_schema_metadata(
        self,
        metadata: std::collections::HashMap<String, String>,
    ) -> Result<Self, CanonError> {
        let schema = Arc::new(arrow_schema::Schema::new_with_metadata(
            self.batch.schema().fields().clone(),
            metadata,
        ));
        Ok(Self {
            batch: RecordBatch::try_new_with_options(
                schema,
                self.batch.columns().to_vec(),
                &RecordBatchOptions::new().with_row_count(Some(self.batch.num_rows())),
            )?,
            ownership: self.ownership,
        })
    }
    /// Borrow the exact immutable batch; arrays retain their own leases.
    pub const fn batch(&self) -> &RecordBatch {
        &self.batch
    }
    /// Transfer native storage; import ownership before an internal raw handoff.
    pub fn into_batch(self) -> RecordBatch {
        self.batch
    }
}
impl std::ops::Deref for OwnedRecordBatch {
    type Target = RecordBatch;
    fn deref(&self) -> &Self::Target {
        &self.batch
    }
}
impl std::borrow::Borrow<RecordBatch> for OwnedRecordBatch {
    fn borrow(&self) -> &RecordBatch {
        &self.batch
    }
}
impl PartialEq for OwnedRecordBatch {
    fn eq(&self, other: &Self) -> bool {
        self.batch == other.batch
    }
}

/// Producer-local allocation provenance. Weak entries cannot pin an unrelated
/// output; actual buffers hold leases. There is no process-global discovery table.
#[derive(Clone, Debug, Default)]
pub struct AllocationScope(Arc<Mutex<Owners>>);
impl AllocationScope {
    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Owners>, CanonError> {
        self.0
            .lock()
            .map_err(|_| CanonError::Internal("allocation scope lock poisoned".into()))
    }
    /// Import an explicitly retained producer scope. Weak allocation entries do
    /// not retain its values; no process-wide search or allocation-map sum runs.
    /// # Errors
    /// A poisoned ownership scope.
    pub fn import_scope(&self, source: &Self) -> Result<(), CanonError> {
        if Arc::ptr_eq(&self.0, &source.0) {
            return Ok(());
        }
        let imported = source.lock()?.clone();
        let mut target = self.lock()?;
        for (key, owner) in imported {
            if let Some(owner) = owner.upgrade() {
                merge_owner(&mut target, key, &owner)?;
            }
        }
        Ok(())
    }
    /// Import a checked handoff's allocation owners, without rewrapping storage.
    /// # Errors
    /// Missing ownership evidence or a poisoned ownership scope.
    pub fn import(&self, owned: &OwnedRecordBatch) -> Result<(), CanonError> {
        if Arc::ptr_eq(&self.0, &owned.ownership.0) {
            return Ok(());
        }
        let mut buffers = BTreeMap::new();
        for array in owned.columns() {
            native_buffers(&array.to_data(), &mut buffers)?;
        }
        let imported = {
            let source = owned.ownership.lock()?;
            buffers
                .keys()
                .map(|key| {
                    source
                        .get(key)
                        .and_then(Weak::upgrade)
                        .map(|owner| (*key, owner))
                        .ok_or_else(|| {
                            CanonError::Internal("owned buffer provenance absent".into())
                        })
                })
                .collect::<Result<Vec<_>, _>>()?
        };
        let mut target = self.lock()?;
        for (key, owner) in imported {
            merge_owner(&mut target, key, &owner)?;
        }
        Ok(())
    }
    /// Export native output, carrying known owners and copying unproven buffers.
    /// # Errors
    /// Reservation, cancellation, extent or representation failure.
    pub fn export(
        &self,
        batch: RecordBatch,
        pool: &Arc<dyn MemoryPool>,
        cancel: &CancellationToken,
    ) -> Result<OwnedRecordBatch, CanonError> {
        cancel.checkpoint()?;
        let copied = {
            let owners = self.lock()?;
            let needed = batch.columns().iter().try_fold(0usize, |total, array| {
                checked_sum(
                    total,
                    data_size(&array.to_data(), &|buffer| {
                        if owners
                            .get(&(buffer.data_ptr().as_ptr() as usize))
                            .and_then(Weak::upgrade)
                            .is_some()
                        {
                            Ok(0)
                        } else {
                            rounded_capacity(buffer.len())
                        }
                    })?,
                )
            })?;
            let reservation = crate::MemoryConsumer::new("result:unproven-copy").register(pool);
            reservation.try_grow(needed)?;
            let columns = batch
                .columns()
                .iter()
                .map(|array| {
                    transform_data(array.to_data(), &mut |buffer| {
                        cancel.checkpoint()?;
                        if owners
                            .get(&(buffer.data_ptr().as_ptr() as usize))
                            .and_then(Weak::upgrade)
                            .is_some()
                        {
                            Ok(buffer)
                        } else {
                            Ok(Buffer::from_slice_ref(buffer.as_slice()))
                        }
                    })
                    .map(make_array)
                })
                .collect::<Result<Vec<_>, _>>()?;
            (
                RecordBatch::try_new_with_options(
                    batch.schema(),
                    columns,
                    &RecordBatchOptions::new().with_row_count(Some(batch.num_rows())),
                )?,
                reservation,
            )
        };
        drop(batch);
        self.attach(copied.0, Some(copied.1), None)
    }
    /// Admit explicitly native-origin allocations under this producer. Foreign
    /// ingress must use export/copy; Arrow capacity cannot prove foreign extents.
    /// # Errors
    /// Resource refusal, extent overflow or Arrow representation failure.
    pub fn retain_native(
        &self,
        batch: RecordBatch,
        pool: &Arc<dyn MemoryPool>,
    ) -> Result<OwnedRecordBatch, CanonError> {
        self.attach(batch, None, Some(pool))
    }
    /// Transfer a reservation while preserving imported input allocation owners.
    /// # Errors
    /// Insufficient reservation, invalid extent or Arrow representation.
    pub fn attach_reserved(
        &self,
        batch: RecordBatch,
        reservation: crate::MemoryReservation,
    ) -> Result<OwnedRecordBatch, CanonError> {
        self.attach(batch, Some(reservation), None)
    }
    fn retained_bytes(&self, batch: &RecordBatch) -> Result<usize, CanonError> {
        let owners = self.lock()?;
        let mut allocations = BTreeMap::new();
        for array in batch.columns() {
            retained_allocations_with_owners(&array.to_data(), &mut allocations, &owners)?;
        }
        allocations.into_values().try_fold(0, checked_sum)
    }
    fn attach(
        &self,
        batch: RecordBatch,
        supplied: Option<crate::MemoryReservation>,
        pool: Option<&Arc<dyn MemoryPool>>,
    ) -> Result<OwnedRecordBatch, CanonError> {
        attach_native(batch, supplied, pool, self)
    }
}

/// The original allocation is dropped before the lease, in declaration order.
struct LeasedBuffer {
    buffer: Buffer,
    _owner: Arc<BufferOwner>,
}

/// Keeps the real allocation alive, preventing address reuse while its weak
/// inventory entry can be upgraded. Drop storage before releasing native capacity.
#[derive(Debug)]
struct BufferOwner {
    storage: Buffer,
    lease: Arc<AllocationLease>,
}

type Owners = BTreeMap<usize, Weak<BufferOwner>>;
fn merge_owner(
    target: &mut Owners,
    key: usize,
    owner: &Arc<BufferOwner>,
) -> Result<(), CanonError> {
    if let Some(existing) = target.get(&key).and_then(Weak::upgrade)
        && !Arc::ptr_eq(&existing, owner)
    {
        return Err(CanonError::Internal(
            "conflicting retained allocation ownership".into(),
        ));
    }
    target.insert(key, Arc::downgrade(owner));
    Ok(())
}
fn extent(buffer: &Buffer) -> Result<usize, CanonError> {
    Ok(buffer
        .capacity()
        .max(checked_sum(buffer.ptr_offset(), buffer.len())?))
}
fn native_buffers(data: &ArrayData, out: &mut BTreeMap<usize, Buffer>) -> Result<(), CanonError> {
    for buffer in data
        .buffers()
        .iter()
        .chain(data.nulls().map(NullBuffer::buffer))
    {
        if extent(buffer)? == 0 {
            continue;
        }
        match out.entry(buffer.data_ptr().as_ptr() as usize) {
            std::collections::btree_map::Entry::Vacant(e) => {
                e.insert(buffer.clone());
            }
            std::collections::btree_map::Entry::Occupied(mut e) => {
                if extent(buffer)? > extent(e.get())? {
                    e.insert(buffer.clone());
                }
            }
        }
    }
    for child in data.child_data() {
        native_buffers(child, out)?;
    }
    Ok(())
}
fn attach_native(
    batch: RecordBatch,
    supplied: Option<crate::MemoryReservation>,
    pool: Option<&Arc<dyn MemoryPool>>,
    scope: &AllocationScope,
) -> Result<OwnedRecordBatch, CanonError> {
    let mut buffers = BTreeMap::new();
    for array in batch.columns() {
        native_buffers(&array.to_data(), &mut buffers)?;
    }
    let mut inventory = scope.lock()?;
    let mut retained = BTreeMap::new();
    let mut needed = 0usize;
    for (key, buffer) in &buffers {
        if let Some(owner) = inventory.get(key).and_then(Weak::upgrade) {
            // Custom/foreign buffers may conceal backing capacity. A larger view
            // requires explicit admission rather than silently extending a lease.
            if extent(buffer)? > owner.lease.size() {
                return Err(CanonError::Internal(
                    "foreign view exceeds admitted allocation extent".into(),
                ));
            }
            retained.insert(*key, owner);
        } else {
            needed = checked_sum(needed, extent(buffer)?)?;
        }
    }
    let reservation = if let Some(reservation) = supplied {
        reservation
    } else {
        let pool = pool.ok_or_else(|| CanonError::Internal("native pool absent".into()))?;
        let reservation = crate::MemoryConsumer::new("result:retained-native").register(pool);
        reservation.try_grow(needed)?;
        reservation
    };
    if reservation.size() < needed {
        return Err(ReserveError::Exhausted {
            owner: "owned-buffer:attach".into(),
            requested: needed,
            reserved: reservation.size(),
            limit_hint: "pre-admitted native capacity".into(),
        }
        .into());
    }
    for (key, buffer) in buffers {
        if let std::collections::btree_map::Entry::Vacant(entry) = retained.entry(key) {
            let bytes = extent(&buffer)?;
            let owner = Arc::new(BufferOwner {
                storage: buffer,
                lease: AllocationLease::new(reservation.split(bytes)),
            });
            inventory.insert(key, Arc::downgrade(&owner));
            entry.insert(owner);
        }
    }
    let (schema, columns, row_count) = batch.into_parts();
    let arrays = columns
        .into_iter()
        .map(|array| {
            transform_data(array.to_data(), &mut |buffer| {
                if extent(&buffer)? == 0 {
                    return Ok(buffer);
                }
                let owner = retained
                    .get(&(buffer.data_ptr().as_ptr() as usize))
                    .ok_or_else(|| CanonError::Internal("native buffer owner absent".into()))?
                    .clone();
                let wrapped = Buffer::from(Bytes::from_owner(LeasedBuffer {
                    buffer,
                    _owner: owner.clone(),
                }));
                inventory.insert(wrapped.data_ptr().as_ptr() as usize, Arc::downgrade(&owner));
                Ok(wrapped)
            })
            .map(make_array)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(OwnedRecordBatch {
        batch: RecordBatch::try_new_with_options(
            schema,
            arrays,
            &RecordBatchOptions::new().with_row_count(Some(row_count)),
        )?,
        ownership: scope.clone(),
    })
}

impl AsRef<[u8]> for LeasedBuffer {
    fn as_ref(&self) -> &[u8] {
        self.buffer.as_slice()
    }
}

/// Counts Arrow-reported extents of value, offset, validity and dictionary allocations.
/// Use `OwnedRecordBatch::retained_bytes` for exact producer-owned extents. For foreign/custom
/// storage these are lower bounds, not hidden-capacity certification: use `copy_batch`
/// at foreign ingress. Native-origin producers must establish their allocation contract.
/// Shared allocations are counted once using Arrow's actual allocation base pointer,
/// with the maximum exposed extent across their slices. This is allocation accounting,
/// not value equality or semantic validation. Hidden external backing extents remain
/// the external owner's responsibility.
///
/// # Errors
/// [`CanonError::Envelope`] if checked addressable-size arithmetic overflows.
pub fn retained_buffer_bytes(batch: &RecordBatch) -> Result<usize, CanonError> {
    let mut allocations = BTreeMap::new();
    for array in batch.columns() {
        retained_allocations(&array.to_data(), &mut allocations)?;
    }
    allocations.into_values().try_fold(0, checked_sum)
}

/// Allocation accounting across a retained batch set, including shared slices.
/// This is an address/extent inventory, never a semantic identity or cache key.
#[derive(Debug, Default)]
pub struct RetainedBuffers {
    allocations: BTreeMap<usize, usize>,
    bytes: usize,
}
impl RetainedBuffers {
    /// Record additional buffer extents. The caller must retain all previously
    /// recorded batches until this inventory is cleared/dropped.
    /// # Errors
    /// Checked extent arithmetic overflows.
    pub fn additional(&mut self, batch: &RecordBatch) -> Result<usize, CanonError> {
        self.additional_with_owners(batch, &Owners::new())
    }
    /// Count exact backing extents incrementally across retained owned chunks.
    /// # Errors
    /// Extent overflow or poisoned allocation provenance.
    pub fn additional_owned(&mut self, batch: &OwnedRecordBatch) -> Result<usize, CanonError> {
        let owners = batch.ownership.lock()?;
        self.additional_with_owners(batch.batch(), &owners)
    }
    fn additional_with_owners(
        &mut self,
        batch: &RecordBatch,
        owners: &Owners,
    ) -> Result<usize, CanonError> {
        let mut current = BTreeMap::new();
        let mut containers = 0usize;
        for array in batch.columns() {
            let data = array.to_data();
            retained_allocations_with_owners(&data, &mut current, owners)?;
            containers = checked_sum(
                containers,
                data.get_array_memory_size()
                    .saturating_sub(data.get_buffer_memory_size()),
            )?;
        }
        let mut additional = 0usize;
        for (pointer, extent) in current {
            match self.allocations.entry(pointer.as_ptr() as usize) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    additional = checked_sum(additional, checked_sum(extent, 96)?)?;
                    entry.insert(extent);
                }
                std::collections::btree_map::Entry::Occupied(mut entry) => {
                    if extent > *entry.get() {
                        additional = checked_sum(additional, extent - *entry.get())?;
                        entry.insert(extent);
                    }
                }
            }
        }
        self.bytes = checked_sum(self.bytes, additional)?;
        checked_sum(additional, containers)
    }
}

/// Attach an existing native allocation owner to every output buffer without
/// copying or opening another reservation. This only preserves lifetime; callers
/// remain responsible for the owner's actual memory/validity contract.
struct LeasedBytes {
    bytes: Vec<u8>,
    lease: Arc<AllocationLease>,
}
impl AsRef<[u8]> for LeasedBytes {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

/// Attaches a reservation acquired before allocation to an owned byte vector. Byte
/// clones and slices retain the lease until the underlying vector can be freed.
///
/// # Errors
/// [`CanonError::Reservation`] if the lease does not cover the vector's capacity.
pub fn attach_bytes(bytes: Vec<u8>, lease: Arc<AllocationLease>) -> Result<Bytes, CanonError> {
    let owner = LeasedBytes { bytes, lease };
    if owner.lease.size() < owner.bytes.capacity() {
        return Err(ReserveError::Exhausted {
            owner: "owned-buffer:bytes".to_owned(),
            requested: owner.bytes.capacity(),
            reserved: owner.lease.size(),
            limit_hint: "the pre-acquired reservation must cover byte capacity".to_owned(),
        }
        .into());
    }
    Ok(Bytes::from_owner(owner))
}

/// Attaches a pre-acquired reservation to every value, offset and validity buffer.
///
/// Array children include dictionary values. Clones and slices of any exposed buffer
/// retain the same lease, without a second reservation. The caller must have reserved
/// the backing allocation before constructing it; an external buffer's hidden backing
/// extent cannot be discovered here. The visible retained extent is checked as a minimum.
///
/// # Errors
///
/// [`CanonError::Reservation`] if the lease is too small; [`CanonError::Arrow`] if the
/// rebuilt Arrow representation fails validation; [`CanonError::Envelope`] on overflow.
pub fn attach_reservation(
    batch: RecordBatch,
    reservation: crate::MemoryReservation,
) -> Result<RecordBatch, CanonError> {
    Ok(OwnedRecordBatch::from_reserved(batch, reservation)?.into_batch())
}

/// Copies raw encoding bytes into an aligned buffer retaining its reservation.
///
/// This is the explicit alignment-copy boundary for strict IPC decoding. Decoder
/// configuration must still require alignment; this helper does not decode or validate
/// artifact semantics. Source bytes remain live and are accounted separately.
///
/// # Errors
/// [`CanonError::Envelope`], [`CanonError::Reservation`] or [`CanonError::Cancelled`].
pub fn copy_aligned_buffer(
    bytes: &[u8],
    owner: &str,
    pool: &Arc<dyn MemoryPool>,
    cancel: &CancellationToken,
) -> Result<Buffer, CanonError> {
    cancel.checkpoint()?;
    let needed = rounded_capacity(bytes.len())?;
    let reservation = crate::MemoryConsumer::new(owner).register(pool);
    reservation.try_grow(needed)?;
    cancel.checkpoint()?;
    let buffer = Buffer::from_slice_ref(bytes);
    cancel.checkpoint()?;
    let owner = Arc::new(BufferOwner {
        storage: buffer.clone(),
        lease: AllocationLease::new(reservation),
    });
    let result = Buffer::from(Bytes::from_owner(LeasedBuffer {
        buffer,
        _owner: owner,
    }));
    Ok(result)
}

/// Copies a result into newly reserved, aligned, immutable Arrow buffers.
///
/// Uses the default envelope and the `result:owned-copy` consumer. Sources remain live
/// during copying; their existing engine reservations are not transferred or guessed.
///
/// # Errors
///
/// [`CanonError::Envelope`], [`CanonError::Reservation`], [`CanonError::Cancelled`] or
/// [`CanonError::Arrow`]. All acquired reservations release on error.
pub fn copy_batch(
    batch: &RecordBatch,
    pool: &Arc<dyn MemoryPool>,
    cancel: &CancellationToken,
) -> Result<RecordBatch, CanonError> {
    copy_batch_with_envelope(batch, "result:owned-copy", pool, cancel, Envelope::DEFAULT)
}

/// The configurable form of [`copy_batch`], with explicit owner and selected bounds.
///
/// # Errors
///
/// The same failures as [`copy_batch`]. Representation limits remain checked.
pub fn copy_batch_with_envelope(
    batch: &RecordBatch,
    owner: &str,
    pool: &Arc<dyn MemoryPool>,
    cancel: &CancellationToken,
    envelope: Envelope,
) -> Result<RecordBatch, CanonError> {
    Ok(copy_owned(batch, owner, pool, cancel, envelope)?.into_batch())
}
fn copy_owned(
    batch: &RecordBatch,
    owner: &str,
    pool: &Arc<dyn MemoryPool>,
    cancel: &CancellationToken,
    envelope: Envelope,
) -> Result<OwnedRecordBatch, CanonError> {
    cancel.checkpoint()?;
    let envelope = Envelope::new(envelope.max_rows, envelope.max_normalized_bytes)?;
    if batch.num_rows() as u64 > envelope.max_rows {
        return Err(CanonError::Envelope {
            what: EnvelopeBound::Rows,
            limit: envelope.max_rows,
            actual: batch.num_rows() as u64,
        });
    }
    let needed = batch.columns().iter().try_fold(0_usize, |total, array| {
        cancel.checkpoint()?;
        checked_sum(total, copied_size(&array.to_data())?)
    })?;
    if needed as u64 > envelope.max_normalized_bytes {
        return Err(CanonError::Envelope {
            what: EnvelopeBound::Bytes,
            limit: envelope.max_normalized_bytes,
            actual: needed as u64,
        });
    }
    let reservation = crate::MemoryConsumer::new(owner).register(pool);
    reservation.try_grow(needed)?;
    cancel.checkpoint()?;
    let arrays = batch
        .columns()
        .iter()
        .map(|array| {
            transform_data(array.to_data(), &mut |buffer| {
                cancel.checkpoint()?;
                Ok(Buffer::from_slice_ref(buffer.as_slice()))
            })
            .map(make_array)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let copy = RecordBatch::try_new_with_options(
        batch.schema(),
        arrays,
        &RecordBatchOptions::new().with_row_count(Some(batch.num_rows())),
    )?;
    cancel.checkpoint()?;
    OwnedRecordBatch::from_reserved(copy, reservation)
}

fn transform_data(
    data: ArrayData,
    transform: &mut impl FnMut(Buffer) -> Result<Buffer, CanonError>,
) -> Result<ArrayData, CanonError> {
    let buffers = data
        .buffers()
        .iter()
        .cloned()
        .map(&mut *transform)
        .collect::<Result<Vec<_>, _>>()?;
    let children = data
        .child_data()
        .iter()
        .cloned()
        .map(|child| transform_data(child, transform))
        .collect::<Result<Vec<_>, _>>()?;
    let nulls = data
        .nulls()
        .map(|nulls| {
            transform(nulls.buffer().clone()).map(|buffer| {
                NullBuffer::new(BooleanBuffer::new(buffer, nulls.offset(), nulls.len()))
            })
        })
        .transpose()?;
    Ok(data
        .into_builder()
        .buffers(buffers)
        .child_data(children)
        .nulls(nulls)
        .build()?)
}

fn retained_allocations(
    data: &ArrayData,
    allocations: &mut BTreeMap<NonNull<u8>, usize>,
) -> Result<(), CanonError> {
    retained_allocations_with_owners(data, allocations, &Owners::new())
}
fn retained_allocations_with_owners(
    data: &ArrayData,
    allocations: &mut BTreeMap<NonNull<u8>, usize>,
    inventory: &Owners,
) -> Result<(), CanonError> {
    for buffer in data
        .buffers()
        .iter()
        .chain(data.nulls().map(NullBuffer::buffer))
    {
        let (identity, extent) = if let Some(owner) = inventory
            .get(&(buffer.data_ptr().as_ptr() as usize))
            .and_then(Weak::upgrade)
        {
            if extent(buffer)? > owner.lease.size() {
                return Err(CanonError::Internal(
                    "view exceeds retained allocation extent".into(),
                ));
            }
            (owner.storage.data_ptr(), owner.lease.size())
        } else {
            (buffer.data_ptr(), extent(buffer)?)
        };
        allocations
            .entry(identity)
            .and_modify(|current| *current = (*current).max(extent))
            .or_insert(extent);
    }
    for child in data.child_data() {
        retained_allocations_with_owners(child, allocations, inventory)?;
    }
    Ok(())
}

fn copied_size(data: &ArrayData) -> Result<usize, CanonError> {
    // Arrow 59.3 MutableBuffer rounds byte capacity to a multiple of 64. Checking this
    // before Buffer::from_slice_ref also avoids its infallible overflow/panic path.
    data_size(data, &|buffer| rounded_capacity(buffer.len()))
}

fn rounded_capacity(len: usize) -> Result<usize, CanonError> {
    len.checked_add(63)
        .map(|n| n & !63)
        .filter(|n| isize::try_from(*n).is_ok())
        .ok_or_else(size_overflow)
}

fn data_size(
    data: &ArrayData,
    size: &impl Fn(&Buffer) -> Result<usize, CanonError>,
) -> Result<usize, CanonError> {
    let mut total = 0;
    for buffer in data.buffers() {
        total = checked_sum(total, size(buffer)?)?;
    }
    if let Some(nulls) = data.nulls() {
        total = checked_sum(total, size(nulls.buffer())?)?;
    }
    for child in data.child_data() {
        total = checked_sum(total, data_size(child, size)?)?;
    }
    Ok(total)
}

fn checked_sum(left: usize, right: usize) -> Result<usize, CanonError> {
    left.checked_add(right).ok_or_else(size_overflow)
}

fn size_overflow() -> CanonError {
    CanonError::Envelope {
        what: EnvelopeBound::Bytes,
        limit: isize::MAX as u64,
        actual: u64::MAX,
    }
}

#[cfg(test)]
mod tests {
    use arrow_array::types::{Int8Type, Int32Type};
    use arrow_array::{
        ArrayRef, DictionaryArray, Int8Array, Int32Array, ListArray, StringArray, StructArray,
    };
    use arrow_schema::{DataType, Field, Schema};

    use super::*;

    fn fixture() -> RecordBatch {
        let dictionary: ArrayRef = Arc::new(
            DictionaryArray::<Int8Type>::try_new(
                Int8Array::from(vec![Some(0), None, Some(1)]),
                Arc::new(StringArray::from(vec!["Liq", "Vap"])),
            )
            .expect("dictionary values and indices are valid"),
        );
        let list: ArrayRef = Arc::new(ListArray::from_iter_primitive::<Int32Type, _, _>(vec![
            Some(vec![Some(1), None]),
            None,
            Some(vec![Some(2)]),
        ]));
        let items = Arc::new(Field::new(
            "items",
            DataType::List(Arc::new(Field::new("item", DataType::Int32, true))),
            true,
        ));
        let nested: ArrayRef = Arc::new(StructArray::from(vec![(Arc::clone(&items), list)]));
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new(
                "phase",
                DataType::Dictionary(Box::new(DataType::Int8), Box::new(DataType::Utf8)),
                true,
            ),
            Field::new("nested", DataType::Struct(vec![items].into()), false),
        ]));
        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(Int32Array::from(vec![1, 2, 3])),
                dictionary,
                nested,
            ],
        )
        .expect("fixture matches declared schema")
    }

    fn buffers(data: &ArrayData, out: &mut Vec<Buffer>) {
        out.extend(data.buffers().iter().cloned());
        if let Some(nulls) = data.nulls() {
            out.push(nulls.buffer().clone());
        }
        for child in data.child_data() {
            buffers(child, out);
        }
    }

    fn batch_buffers(batch: &RecordBatch) -> Vec<Buffer> {
        let mut found = Vec::new();
        for column in batch.columns() {
            buffers(&column.to_data(), &mut found);
        }
        found
    }

    #[test]
    fn every_nested_dictionary_and_validity_buffer_keeps_the_reservation() {
        let source = fixture().slice(1, 2);
        let count = batch_buffers(&source).len();
        assert!(
            count >= 9,
            "fixture exercises nested buffers and dictionary storage"
        );
        for selected in 0..count {
            let budget: Arc<dyn MemoryPool> = Arc::new(crate::GreedyMemoryPool::new(1 << 20));
            let copy = copy_batch(&source, &budget, &CancellationToken::new()).expect("copy fits");
            assert_eq!(copy, source);
            let charged = budget.reserved();
            assert!(charged > 0);
            let mut retained = batch_buffers(&copy);
            let buffer = retained.remove(selected);
            let selected_capacity = rounded_capacity(buffer.len()).unwrap();
            let slice = buffer.slice_with_length(0, buffer.len().min(1));
            drop(buffer);
            drop(retained);
            drop(copy);
            assert_eq!(
                budget.reserved(),
                selected_capacity,
                "saved buffer {selected} retains only its backing allocation"
            );
            drop(slice);
            assert_eq!(
                budget.reserved(),
                0,
                "last buffer {selected} releases the lease"
            );
        }
    }

    #[test]
    fn array_and_batch_clones_do_not_charge_again_or_release_early() {
        let budget: Arc<dyn MemoryPool> = Arc::new(crate::GreedyMemoryPool::new(1 << 20));
        let copy = copy_batch(&fixture(), &budget, &CancellationToken::new()).expect("copy fits");
        let charged = budget.reserved();
        let slice = copy.slice(1, 1);
        let saved_array = Arc::clone(copy.column(1));
        drop(copy);
        assert_eq!(budget.reserved(), charged);
        let mut owned = Vec::new();
        buffers(&saved_array.to_data(), &mut owned);
        let expected = owned
            .iter()
            .map(|buffer| rounded_capacity(buffer.len()).unwrap())
            .sum::<usize>();
        drop(owned);
        drop(slice);
        assert_eq!(budget.reserved(), expected);
        drop(saved_array);
        assert_eq!(budget.reserved(), 0);
    }

    #[test]
    fn result_copy_accounts_for_coexistence_with_the_source() {
        let budget: Arc<dyn MemoryPool> = Arc::new(crate::GreedyMemoryPool::new(1 << 20));
        let source =
            copy_batch(&fixture(), &budget, &CancellationToken::new()).expect("source fits");
        let once = budget.reserved();
        let result = copy_batch(&source, &budget, &CancellationToken::new()).expect("result fits");
        assert_eq!(source, result);
        assert_eq!(budget.reserved(), 2 * once);
        drop(source);
        assert_eq!(budget.reserved(), once);
        drop(result);
        assert_eq!(budget.reserved(), 0);
    }

    #[test]
    fn insufficient_budget_and_cancelled_work_release_all_claims() {
        let source = fixture();
        let budget: Arc<dyn MemoryPool> = Arc::new(crate::GreedyMemoryPool::new(1));
        assert!(matches!(
            copy_batch(&source, &budget, &CancellationToken::new()),
            Err(CanonError::NativeResource(_))
        ));
        assert_eq!(budget.reserved(), 0);
        let cancel = CancellationToken::new();
        cancel.cancel();
        assert!(matches!(
            copy_batch(&source, &budget, &cancel),
            Err(CanonError::Cancelled)
        ));
        assert_eq!(budget.reserved(), 0);
        assert!(
            attach_reservation(
                source,
                crate::MemoryConsumer::new("too-small").register(&budget)
            )
            .is_err()
        );
        assert_eq!(budget.reserved(), 0);
    }

    #[test]
    fn query_export_retains_native_buffers_without_copying_values() {
        let source = fixture();
        let required = retained_buffer_bytes(&source).expect("bounded source");
        let original_buffers = batch_buffers(&source);
        let too_small: Arc<dyn MemoryPool> = Arc::new(crate::GreedyMemoryPool::new(required - 1));
        assert!(
            AllocationScope::default()
                .retain_native(source.clone(), &too_small)
                .is_err()
        );
        assert_eq!(too_small.reserved(), 0);
        let enough: Arc<dyn MemoryPool> = Arc::new(crate::GreedyMemoryPool::new(required));
        let exported = AllocationScope::default()
            .retain_native(source, &enough)
            .expect("native retained buffers fit without a second copy");
        for (original, exported) in original_buffers.iter().zip(batch_buffers(&exported)) {
            assert_eq!(
                original.as_ptr(),
                exported.as_ptr(),
                "native allocation retained"
            );
            assert_eq!(original.len(), exported.len());
        }
        assert_eq!(enough.reserved(), required);
        drop(exported);
        assert_eq!(enough.reserved(), 0);
    }

    #[test]
    fn shared_columns_and_sliced_buffers_charge_each_actual_allocation_once() {
        let original: ArrayRef = Arc::new(Int32Array::from(vec![1, 2, 3, 4]));
        let schema = Arc::new(Schema::new(vec![
            Field::new("left", DataType::Int32, false),
            Field::new("right", DataType::Int32, false),
        ]));
        let batch = RecordBatch::try_new(schema, vec![original.slice(0, 2), original.slice(1, 2)])
            .expect("shared sliced columns");
        let required = original.to_data().buffers()[0].capacity();
        assert_eq!(retained_buffer_bytes(&batch).expect("bounded"), required);
        let budget: Arc<dyn MemoryPool> = Arc::new(crate::GreedyMemoryPool::new(required));
        let exported = AllocationScope::default()
            .retain_native(batch, &budget)
            .expect("one allocation claim fits");
        assert_eq!(budget.reserved(), required);
        let retained = Arc::clone(exported.column(1));
        drop(exported);
        assert_eq!(budget.reserved(), required);
        drop(retained);
        assert_eq!(budget.reserved(), 0);
    }

    #[test]
    fn copy_bounds_are_checked_before_reservation() {
        let budget: Arc<dyn MemoryPool> = Arc::new(crate::GreedyMemoryPool::new(1 << 20));
        for envelope in [
            Envelope::new(1, 1 << 20).expect("lower bound"),
            Envelope::new(10, 1).expect("lower bound"),
        ] {
            assert!(matches!(
                copy_batch_with_envelope(
                    &fixture(),
                    "bounded",
                    &budget,
                    &CancellationToken::new(),
                    envelope
                ),
                Err(CanonError::Envelope { .. })
            ));
            assert_eq!(budget.reserved(), 0);
        }
    }

    #[test]
    fn strict_decode_alignment_copy_retains_bytes_and_budget_through_slices() {
        let budget: Arc<dyn MemoryPool> = Arc::new(crate::GreedyMemoryPool::new(64));
        let source = [0_u8, 1, 2, 3, 4];
        let buffer = copy_aligned_buffer(
            &source[1..],
            "ipc:aligned-copy",
            &budget,
            &CancellationToken::new(),
        )
        .expect("copy fits");
        assert_eq!(buffer.as_slice(), &[1, 2, 3, 4]);
        assert_eq!(buffer.as_ptr().align_offset(64), 0);
        assert_eq!(budget.reserved(), 64);
        let tail = buffer.slice(1);
        drop(buffer);
        assert_eq!(budget.reserved(), 64);
        drop(tail);
        assert_eq!(budget.reserved(), 0);
        assert!(
            copy_aligned_buffer(&[0_u8; 65], "too-large", &budget, &CancellationToken::new())
                .is_err()
        );
        assert_eq!(budget.reserved(), 0);
    }

    #[derive(Debug)]
    struct CancellingPool {
        budget: Arc<dyn MemoryPool>,
        cancel: CancellationToken,
    }

    impl std::fmt::Display for CancellingPool {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "CancellingPool")
        }
    }
    impl MemoryPool for CancellingPool {
        fn name(&self) -> &'static str {
            "CancellingPool"
        }
        fn register(&self, c: &crate::MemoryConsumer) {
            self.budget.register(c);
        }
        fn unregister(&self, c: &crate::MemoryConsumer) {
            self.budget.unregister(c);
        }
        fn grow(&self, r: &crate::MemoryReservation, n: usize) {
            self.budget.grow(r, n);
        }
        fn shrink(&self, r: &crate::MemoryReservation, n: usize) {
            self.budget.shrink(r, n);
        }
        fn try_grow(
            &self,
            r: &crate::MemoryReservation,
            n: usize,
        ) -> datafusion_common::Result<()> {
            self.budget.try_grow(r, n)?;
            self.cancel.cancel();
            Ok(())
        }
        fn reserved(&self) -> usize {
            self.budget.reserved()
        }
        fn memory_limit(&self) -> datafusion_execution::memory_pool::MemoryLimit {
            self.budget.memory_limit()
        }
    }

    #[test]
    fn cancellation_after_acquiring_budget_releases_before_copying() {
        let budget: Arc<dyn MemoryPool> = Arc::new(crate::GreedyMemoryPool::new(1 << 20));
        let cancel = CancellationToken::new();
        let pool: Arc<dyn MemoryPool> = Arc::new(CancellingPool {
            budget: Arc::clone(&budget),
            cancel: cancel.clone(),
        });
        assert!(matches!(
            copy_batch(&fixture(), &pool, &cancel),
            Err(CanonError::Cancelled)
        ));
        assert_eq!(budget.reserved(), 0);
    }

    #[test]
    fn cancellation_after_acquiring_export_claim_releases_native_result() {
        let budget: Arc<dyn MemoryPool> = Arc::new(crate::GreedyMemoryPool::new(1 << 20));
        let cancel = CancellationToken::new();
        let pool: Arc<dyn MemoryPool> = Arc::new(CancellingPool {
            budget: Arc::clone(&budget),
            cancel: cancel.clone(),
        });
        assert!(matches!(
            OwnedRecordBatch::export(fixture(), &pool, &cancel),
            Err(CanonError::Cancelled)
        ));
        assert_eq!(budget.reserved(), 0);
    }
}

#[cfg(test)]
mod pivot_unit {
    use super::*;
    use arrow_array::{ArrayRef, Int32Array};
    use arrow_schema::{DataType, Field, Schema};
    fn shared() -> RecordBatch {
        let array: ArrayRef = Arc::new(Int32Array::from(vec![1, 2, 3, 4]));
        RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("a", DataType::Int32, false),
                Field::new("b", DataType::Int32, false),
            ])),
            vec![array.clone(), array],
        )
        .unwrap()
    }
    #[test]
    fn repeated_exports_share_one_allocation_and_preserve_origin_pool() {
        let pool: Arc<dyn MemoryPool> = Arc::new(crate::GreedyMemoryPool::new(1024));
        let other: Arc<dyn MemoryPool> = Arc::new(crate::GreedyMemoryPool::new(0));
        let input = shared();
        let needed = retained_buffer_bytes(&input).unwrap();
        let first = AllocationScope::default()
            .retain_native(input, &pool)
            .unwrap();
        assert_eq!(pool.reserved(), needed);
        let next = first
            .ownership()
            .export(
                first.slice(1, 1).unwrap().into_batch(),
                &other,
                &CancellationToken::new(),
            )
            .unwrap();
        assert_eq!(pool.reserved(), needed);
        assert_eq!(other.reserved(), 0);
        let foreign = arrow::ffi::FFI_ArrowArray::new(&next.column(0).to_data());
        drop(first);
        drop(next);
        assert_eq!(pool.reserved(), needed);
        drop(foreign);
        assert_eq!(pool.reserved(), 0);
    }
    #[test]
    fn metadata_projection_preserves_buffers_and_native_owner() {
        let pool: Arc<dyn MemoryPool> = Arc::new(crate::GreedyMemoryPool::new(1024));
        let batch = shared();
        let schema = Arc::new(Schema::new_with_metadata(
            batch.schema().fields().clone(),
            [("source".to_owned(), "declared".to_owned())]
                .into_iter()
                .collect(),
        ));
        let batch = batch.with_schema(schema).unwrap();
        let owned = OwnedRecordBatch::export(batch, &pool, &CancellationToken::new()).unwrap();
        let before = pool.reserved();
        let pointer = owned.batch().column(0).to_data().buffers()[0].as_ptr();
        let projected = owned
            .with_schema_metadata(std::collections::HashMap::new())
            .unwrap();
        assert!(projected.batch().schema().metadata().is_empty());
        assert_eq!(
            projected.batch().column(0).to_data().buffers()[0].as_ptr(),
            pointer
        );
        assert_eq!(pool.reserved(), before);
        drop(projected);
        assert_eq!(pool.reserved(), 0);
    }
    #[test]
    fn concurrent_exports_do_not_duplicate_allocation_admission() {
        let pool: Arc<dyn MemoryPool> = Arc::new(crate::GreedyMemoryPool::new(1024));
        let input = shared();
        let needed = retained_buffer_bytes(&input).unwrap();
        let barrier = Arc::new(std::sync::Barrier::new(8));
        let scope = AllocationScope::default();
        let threads = (0..8)
            .map(|_| {
                let batch = input.clone();
                let pool = pool.clone();
                let barrier = barrier.clone();
                let scope = scope.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    scope.retain_native(batch, &pool).unwrap()
                })
            })
            .collect::<Vec<_>>();
        drop(input);
        let batches = threads
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(pool.reserved(), needed);
        drop(batches);
        assert_eq!(pool.reserved(), 0);
    }
    #[test]
    fn copy_has_independent_capacity_and_failed_export_has_no_owner() {
        let pool: Arc<dyn MemoryPool> = Arc::new(crate::GreedyMemoryPool::new(1024));
        let refused: Arc<dyn MemoryPool> = Arc::new(crate::GreedyMemoryPool::new(0));
        assert!(
            AllocationScope::default()
                .retain_native(shared(), &refused)
                .is_err()
        );
        assert_eq!(refused.reserved(), 0);
        let batch = AllocationScope::default()
            .retain_native(shared(), &pool)
            .unwrap();
        let before = pool.reserved();
        let copy = copy_batch(&batch, &pool, &CancellationToken::new()).unwrap();
        assert!(pool.reserved() > before);
        drop(copy);
        assert_eq!(pool.reserved(), before);
        drop(batch);
        assert_eq!(pool.reserved(), 0);
    }
}

#[cfg(test)]
mod integrated_performance_unit {
    use super::*;
    use arrow_array::Int32Array;
    use arrow_schema::{DataType, Field, Schema};

    #[test]
    fn a_small_projection_does_not_pin_other_allocations_and_compaction_is_explicit() {
        let pool: Arc<dyn MemoryPool> = Arc::new(crate::GreedyMemoryPool::new(1 << 20));
        let short: arrow_array::ArrayRef = Arc::new(Int32Array::from(vec![1]));
        let mut values = Vec::with_capacity(4096);
        values.push(7_i32);
        let wide: arrow_array::ArrayRef = Arc::new(Int32Array::from(values));
        let raw = RecordBatch::try_from_iter([("small", short), ("wide", wide)]).unwrap();
        let owned = AllocationScope::default()
            .retain_native(raw, &pool)
            .unwrap();
        let small = owned.project(&[0]).unwrap();
        let wide = owned.project(&[1]).unwrap();
        let small_bytes = small.retained_bytes().unwrap();
        let compact = wide.compact(&pool, &CancellationToken::new()).unwrap();
        assert!(compact.retained_bytes().unwrap() < wide.retained_bytes().unwrap());
        drop(wide);
        drop(owned);
        assert_eq!(
            pool.reserved(),
            small_bytes + compact.retained_bytes().unwrap()
        );
        let exported = small.column(0).clone();
        drop(small);
        drop(compact);
        assert_eq!(pool.reserved(), small_bytes);
        drop(exported);
        assert_eq!(pool.reserved(), 0);
    }

    #[test]
    fn wrapped_2624_byte_allocation_keeps_its_extent_for_a_408_byte_view() {
        let mut values = Vec::with_capacity(656);
        values.extend(0_i32..102);
        let array = Int32Array::new(values.into(), None);
        let batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new("n", DataType::Int32, false)])),
            vec![Arc::new(array)],
        )
        .unwrap();
        assert_eq!(retained_buffer_bytes(&batch).unwrap(), 2624);
        let pool: Arc<dyn MemoryPool> = Arc::new(crate::GreedyMemoryPool::new(2624));
        let scope = AllocationScope::default();
        let owned = scope.retain_native(batch, &pool).unwrap();
        assert_eq!(owned.column(0).to_data().buffers()[0].len(), 408);
        assert_eq!(owned.retained_bytes().unwrap(), 2624);
        let wrapped = owned.slice(1, 1).unwrap();
        assert_eq!(wrapped.retained_bytes().unwrap(), 2624);
        let escaped = wrapped.column(0).to_data().buffers()[0].clone();
        drop(owned);
        drop(wrapped);
        assert_eq!(pool.reserved(), 2624);
        drop(escaped);
        assert_eq!(pool.reserved(), 0);
    }

    #[test]
    fn foreign_hidden_capacity_uses_reserved_ingress_copy_instead_of_guessing() {
        let mut hidden = Vec::with_capacity(2624);
        hidden.resize(408, 0u8);
        let buffer = Buffer::from(Bytes::from(hidden));
        let array = Int32Array::new(arrow_buffer::ScalarBuffer::new(buffer, 0, 102), None);
        let batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new("n", DataType::Int32, false)])),
            vec![Arc::new(array)],
        )
        .unwrap();
        // Arrow 59.3 custom capacity reports the visible size, despite its
        // public method documentation saying zero. It cannot certify backing.
        assert_eq!(retained_buffer_bytes(&batch).unwrap(), 408);
        let pool: Arc<dyn MemoryPool> = Arc::new(crate::GreedyMemoryPool::new(4096));
        // Foreign ingress uses the explicit copy boundary, never native-origin
        // allocation admission. This isolates hidden backing without guessing it.
        let copied = copy_batch(&batch, &pool, &CancellationToken::new()).unwrap();
        assert_ne!(
            batch.column(0).to_data().buffers()[0].as_ptr(),
            copied.column(0).to_data().buffers()[0].as_ptr()
        );
        assert!(retained_buffer_bytes(&copied).unwrap() >= 408);
        drop(copied);
        assert_eq!(pool.reserved(), 0);
        let tiny: Arc<dyn MemoryPool> = Arc::new(crate::GreedyMemoryPool::new(1));
        assert!(copy_batch(&batch, &tiny, &CancellationToken::new()).is_err());
        assert_eq!(tiny.reserved(), 0);
    }
}
