// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Immutable Arrow buffers whose last reader retains their reservation (ADR-0055).
//!
//! These helpers establish allocation ownership and Arrow representation validity;
//! they do not establish schema-registry, key, reference or domain validity. A producer
//! reserves before allocating and attaches that reservation before releasing a batch.
//! Query results use the separately reserved copy path, so source/output coexistence
//! remains accounted without inferring allocation identity from pointer equality.

use std::sync::Arc;

use arrow::array::{ArrayData, make_array};
use arrow_array::{RecordBatch, RecordBatchOptions};
use arrow_buffer::{BooleanBuffer, Buffer, NullBuffer};
use bytes::Bytes;

use crate::{
    CancellationToken, CanonError, Envelope, EnvelopeBound, MemoryReserver, Reservation,
    ReservationLease, ReserveError,
};

/// The original allocation is dropped before the lease, in declaration order.
struct LeasedBuffer {
    buffer: Buffer,
    _lease: Arc<ReservationLease>,
}

impl AsRef<[u8]> for LeasedBuffer {
    fn as_ref(&self) -> &[u8] {
        self.buffer.as_slice()
    }
}

/// Counts the conservatively retained extent of every value, offset, validity and
/// dictionary buffer. Shared buffers may be counted more than once; external backing
/// extents that are not exposed by Arrow remain the external owner's responsibility.
///
/// # Errors
/// [`CanonError::Envelope`] if checked addressable-size arithmetic overflows.
pub fn retained_buffer_bytes(batch: &RecordBatch) -> Result<usize, CanonError> {
    batch.columns().iter().try_fold(0_usize, |total, array| {
        checked_sum(total, retained_size(&array.to_data())?)
    })
}

struct LeasedBytes {
    bytes: Vec<u8>,
    lease: Arc<ReservationLease>,
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
pub fn attach_bytes(bytes: Vec<u8>, lease: Arc<ReservationLease>) -> Result<Bytes, CanonError> {
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
    lease: Arc<ReservationLease>,
) -> Result<RecordBatch, CanonError> {
    let required = batch.columns().iter().try_fold(0_usize, |total, array| {
        checked_sum(total, retained_size(&array.to_data())?)
    })?;
    if lease.size() < required {
        return Err(ReserveError::Exhausted {
            owner: "owned-buffer:attach".to_owned(),
            requested: required,
            reserved: lease.size(),
            limit_hint: "the existing reservation must cover every retained Arrow buffer"
                .to_owned(),
        }
        .into());
    }
    let (schema, columns, row_count) = batch.into_parts();
    let arrays = columns
        .into_iter()
        .map(|array| {
            transform_data(array.to_data(), &mut |buffer| {
                Ok(Buffer::from(Bytes::from_owner(LeasedBuffer {
                    buffer,
                    _lease: Arc::clone(&lease),
                })))
            })
            .map(make_array)
        })
        .collect::<Result<Vec<_>, _>>()?;
    // The returned buffers are now the only lease holders introduced by this call.
    drop(lease);
    Ok(RecordBatch::try_new_with_options(
        schema,
        arrays,
        &RecordBatchOptions::new().with_row_count(Some(row_count)),
    )?)
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
    reserver: &dyn MemoryReserver,
    cancel: &CancellationToken,
) -> Result<Buffer, CanonError> {
    cancel.checkpoint()?;
    let needed = rounded_capacity(bytes.len())?;
    if needed as u64 > Envelope::PHASE1.max_normalized_bytes {
        return Err(CanonError::Envelope {
            what: EnvelopeBound::Bytes,
            limit: Envelope::PHASE1.max_normalized_bytes,
            actual: needed as u64,
        });
    }
    let mut reservation = reserver.open(owner);
    reservation.try_grow(needed)?;
    cancel.checkpoint()?;
    let buffer = Buffer::from_slice_ref(bytes);
    cancel.checkpoint()?;
    Ok(Buffer::from(Bytes::from_owner(LeasedBuffer {
        buffer,
        _lease: ReservationLease::new(reservation),
    })))
}

/// Temporary input ownership: release the batch before its coexistence claim.
struct EmittedBatch {
    batch: RecordBatch,
    _reservation: Box<dyn Reservation>,
}

/// Exports an emitted query batch while claiming source/result coexistence explicitly.
///
/// The source claim sums retained capacities (visible extent for external owners), even
/// when the engine may still account those buffers. External buffers must already retain
/// their backing allocation's ownership contract. This cannot retroactively cover an
/// unregistered allocation. The source batch and its temporary claim drop together;
/// the returned copy retains only the result reservation.
///
/// # Errors
/// The failures of [`copy_batch`], plus rejection of the temporary source claim.
pub fn export_query_batch(
    batch: RecordBatch,
    reserver: &dyn MemoryReserver,
    cancel: &CancellationToken,
) -> Result<RecordBatch, CanonError> {
    cancel.checkpoint()?;
    let source_size = batch.columns().iter().try_fold(0_usize, |total, array| {
        checked_sum(total, retained_size(&array.to_data())?)
    })?;
    if source_size as u64 > Envelope::PHASE1.max_normalized_bytes {
        return Err(CanonError::Envelope {
            what: EnvelopeBound::Bytes,
            limit: Envelope::PHASE1.max_normalized_bytes,
            actual: source_size as u64,
        });
    }
    let mut reservation = reserver.open("result:emitted-source");
    reservation.try_grow(source_size)?;
    let source = EmittedBatch {
        batch,
        _reservation: reservation,
    };
    let result = copy_batch(&source.batch, reserver, cancel)?;
    drop(source);
    Ok(result)
}

/// Copies a result into newly reserved, aligned, immutable Arrow buffers.
///
/// Uses the phase-1 envelope and the `result:owned-copy` consumer. Sources remain live
/// during copying; their existing engine reservations are not transferred or guessed.
///
/// # Errors
///
/// [`CanonError::Envelope`], [`CanonError::Reservation`], [`CanonError::Cancelled`] or
/// [`CanonError::Arrow`]. All acquired reservations release on error.
pub fn copy_batch(
    batch: &RecordBatch,
    reserver: &dyn MemoryReserver,
    cancel: &CancellationToken,
) -> Result<RecordBatch, CanonError> {
    copy_batch_with_envelope(
        batch,
        "result:owned-copy",
        reserver,
        cancel,
        Envelope::PHASE1,
    )
}

/// The configurable form of [`copy_batch`], with explicit owner and tightened bounds.
///
/// # Errors
///
/// The same failures as [`copy_batch`]. An envelope cannot raise the supported limits.
pub fn copy_batch_with_envelope(
    batch: &RecordBatch,
    owner: &str,
    reserver: &dyn MemoryReserver,
    cancel: &CancellationToken,
    envelope: Envelope,
) -> Result<RecordBatch, CanonError> {
    cancel.checkpoint()?;
    let envelope = Envelope::lowered(envelope.max_rows, envelope.max_normalized_bytes)?;
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
    let mut reservation = reserver.open(owner);
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
    attach_reservation(copy, ReservationLease::new(reservation))
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

fn retained_size(data: &ArrayData) -> Result<usize, CanonError> {
    data_size(data, &|buffer| Ok(buffer.capacity().max(buffer.len())))
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
        limit: Envelope::PHASE1.max_normalized_bytes,
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
    use crate::{FixedBudget, Reservation};

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
            let budget = FixedBudget::new(1 << 20);
            let copy =
                copy_batch(&source, budget.as_ref(), &CancellationToken::new()).expect("copy fits");
            assert_eq!(copy, source);
            let charged = budget.reserved();
            assert!(charged > 0);
            let mut retained = batch_buffers(&copy);
            let buffer = retained.remove(selected);
            let slice = buffer.slice_with_length(0, buffer.len().min(1));
            drop(buffer);
            drop(retained);
            drop(copy);
            assert_eq!(
                budget.reserved(),
                charged,
                "saved buffer {selected} must retain its lease"
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
        let budget = FixedBudget::new(1 << 20);
        let copy =
            copy_batch(&fixture(), budget.as_ref(), &CancellationToken::new()).expect("copy fits");
        let charged = budget.reserved();
        let slice = copy.slice(1, 1);
        let saved_array = Arc::clone(copy.column(1));
        drop(copy);
        assert_eq!(budget.reserved(), charged);
        drop(slice);
        assert_eq!(budget.reserved(), charged);
        drop(saved_array);
        assert_eq!(budget.reserved(), 0);
    }

    #[test]
    fn result_copy_accounts_for_coexistence_with_the_source() {
        let budget = FixedBudget::new(1 << 20);
        let source = copy_batch(&fixture(), budget.as_ref(), &CancellationToken::new())
            .expect("source fits");
        let once = budget.reserved();
        let result =
            copy_batch(&source, budget.as_ref(), &CancellationToken::new()).expect("result fits");
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
        let budget = FixedBudget::new(1);
        assert!(matches!(
            copy_batch(&source, budget.as_ref(), &CancellationToken::new()),
            Err(CanonError::Reservation(_))
        ));
        assert_eq!(budget.reserved(), 0);
        let cancel = CancellationToken::new();
        cancel.cancel();
        assert!(matches!(
            copy_batch(&source, budget.as_ref(), &cancel),
            Err(CanonError::Cancelled)
        ));
        assert_eq!(budget.reserved(), 0);
        assert!(
            attach_reservation(source, ReservationLease::new(budget.open("too-small"))).is_err()
        );
        assert_eq!(budget.reserved(), 0);
    }

    #[test]
    fn query_export_requires_both_source_and_result_capacity() {
        let source = fixture();
        let required = source
            .columns()
            .iter()
            .map(|array| copied_size(&array.to_data()).expect("bounded"))
            .sum::<usize>();
        let source_required = source
            .columns()
            .iter()
            .map(|array| retained_size(&array.to_data()).expect("bounded source"))
            .sum::<usize>();
        let too_small = FixedBudget::new(source_required + required - 1);
        assert!(
            export_query_batch(
                source.clone(),
                too_small.as_ref(),
                &CancellationToken::new()
            )
            .is_err()
        );
        assert_eq!(too_small.reserved(), 0);
        let enough = FixedBudget::new(source_required + required);
        let exported = export_query_batch(source, enough.as_ref(), &CancellationToken::new())
            .expect("coexistence fits");
        assert_eq!(enough.reserved(), required, "only the output claim remains");
        drop(exported);
        assert_eq!(enough.reserved(), 0);
    }

    #[test]
    fn copy_bounds_are_checked_before_reservation() {
        let budget = FixedBudget::new(1 << 20);
        for envelope in [
            Envelope::lowered(1, 1 << 20).expect("lower bound"),
            Envelope::lowered(10, 1).expect("lower bound"),
        ] {
            assert!(matches!(
                copy_batch_with_envelope(
                    &fixture(),
                    "bounded",
                    budget.as_ref(),
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
        let budget = FixedBudget::new(64);
        let source = [0_u8, 1, 2, 3, 4];
        let buffer = copy_aligned_buffer(
            &source[1..],
            "ipc:aligned-copy",
            budget.as_ref(),
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
            copy_aligned_buffer(
                &[0_u8; 65],
                "too-large",
                budget.as_ref(),
                &CancellationToken::new()
            )
            .is_err()
        );
        assert_eq!(budget.reserved(), 0);
    }

    #[derive(Debug)]
    struct CancellingReserver {
        budget: Arc<FixedBudget>,
        cancel: CancellationToken,
    }

    impl MemoryReserver for CancellingReserver {
        fn open(&self, owner: &str) -> Box<dyn Reservation> {
            Box::new(CancellingReservation {
                inner: self.budget.open(owner),
                cancel: self.cancel.clone(),
            })
        }
    }

    #[derive(Debug)]
    struct CancellingReservation {
        inner: Box<dyn Reservation>,
        cancel: CancellationToken,
    }

    impl Reservation for CancellingReservation {
        fn try_grow(&mut self, bytes: usize) -> Result<(), ReserveError> {
            self.inner.try_grow(bytes)?;
            self.cancel.cancel();
            Ok(())
        }
        fn shrink(&mut self, bytes: usize) {
            self.inner.shrink(bytes);
        }
        fn size(&self) -> usize {
            self.inner.size()
        }
        fn release(&mut self) {
            self.inner.release();
        }
    }

    #[test]
    fn cancellation_after_acquiring_budget_releases_before_copying() {
        let budget = FixedBudget::new(1 << 20);
        let cancel = CancellationToken::new();
        let reserver = CancellingReserver {
            budget: Arc::clone(&budget),
            cancel: cancel.clone(),
        };
        assert!(matches!(
            copy_batch(&fixture(), &reserver, &cancel),
            Err(CanonError::Cancelled)
        ));
        assert_eq!(budget.reserved(), 0);
    }
}
