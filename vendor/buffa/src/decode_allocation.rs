//! Opt-in pre-growth accounting for logical backing storage, not allocator/RSS.
use alloc::{boxed::Box, vec::Vec};
use crate::{DecodeContext, DecodeError};

// Observe the actual helper reservation sites without replacing the allocator.
#[cfg(all(test, feature = "std"))]
std::thread_local! {
    static RESERVATION_ATTEMPTS: core::cell::Cell<usize> = const { core::cell::Cell::new(0) };
}

#[inline]
fn reservation_attempt() {
    #[cfg(all(test, feature = "std"))]
    RESERVATION_ATTEMPTS.with(|attempts| attempts.set(attempts.get() + 1));
}

/// Admit the full new requested backing layout before a fallible exact reserve.
/// Spare capacity costs nothing. Growth doubles capacity (checked), or takes the
/// required length when larger. Every previous charge remains spent, including
/// when the allocator can grow in place. ZST vectors have no backing allocation.
pub fn reserve_vec<T>(out: &mut Vec<T>, additional: usize, ctx: DecodeContext<'_>) -> Result<(), DecodeError> {
    if ctx.remaining_allocation_budget().is_none() {
        out.reserve(additional);
        return Ok(());
    }
    let required = out.len().checked_add(additional).ok_or(DecodeError::AllocationLayoutOverflow)?;
    if required <= out.capacity() { return Ok(()); }
    let capacity = out.capacity().checked_mul(2).ok_or(DecodeError::AllocationLayoutOverflow)?
        .max(required);
    let layout = core::alloc::Layout::array::<T>(capacity).map_err(|_| DecodeError::AllocationLayoutOverflow)?;
    ctx.register_allocation_bytes(layout.size())?;
    // The pinned standard Global Vec::try_reserve_exact/grow_exact records the
    // requested capacity. Allocator usable size/metadata are outside this claim.
    reservation_attempt();
    out.try_reserve_exact(capacity - out.len()).map_err(|_| DecodeError::AllocationReserveFailed)
}

/// Fallible single-value backing allocation under the same phase budget.
pub fn boxed<T>(value: T, ctx: DecodeContext<'_>) -> Result<Box<T>, DecodeError> {
    if ctx.remaining_allocation_budget().is_none() { return Ok(Box::new(value)); }
    let layout = core::alloc::Layout::new::<T>();
    ctx.register_allocation_bytes(layout.size())?;
    let mut storage = Vec::new();
    reservation_attempt();
    storage.try_reserve_exact(1).map_err(|_| DecodeError::AllocationReserveFailed)?;
    storage.push(value);
    let raw = Box::into_raw(storage.into_boxed_slice()) as *mut T;
    // SAFETY: exactly one initialized T, same layout/alignment as Box<T>.
    // Exact Global reserve makes capacity one, so into_boxed_slice does not
    // shrink/reallocate; ZST storage also has the correct aligned dangling ptr.
    Ok(unsafe { Box::from_raw(raw) })
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::Cell;

    #[test]
    fn allocation_exact_fit_short_spare_and_overlap() {
        let fields = Cell::new(100);
        let bytes = Cell::new(4);
        let ctx = DecodeContext::new(10, &fields).with_allocation_budget(&bytes);
        let mut out = Vec::<u32>::new();
        reserve_vec(&mut out, 1, ctx).unwrap();
        assert_eq!(out.capacity(), 1);
        out.push(7);
        let ptr = out.as_ptr();
        bytes.set(7);
        assert_eq!(reserve_vec(&mut out, 1, ctx), Err(DecodeError::AllocationLimitExceeded));
        assert_eq!((out.as_ptr(), out.capacity(), out.len()), (ptr, 1, 1));
        assert_eq!(out, [7]);
        bytes.set(8);
        reserve_vec(&mut out, 1, ctx).unwrap();
        assert_eq!((out.capacity(), bytes.get()), (2, 0));
        reserve_vec(&mut out, 1, ctx).unwrap(); // unused spare slot, no allocation
        assert_eq!(bytes.get(), 0);
    }

    #[cfg(feature = "std")]
    #[test]
    fn allocation_refusal_precedes_actual_reservation_site() {
        let fields = Cell::new(100);
        let bytes = Cell::new(7);
        let ctx = DecodeContext::new(10, &fields).with_allocation_budget(&bytes);
        RESERVATION_ATTEMPTS.with(|attempts| attempts.set(0));
        let mut out = Vec::<u64>::new();
        assert_eq!(reserve_vec(&mut out, 1, ctx), Err(DecodeError::AllocationLimitExceeded));
        assert_eq!(boxed(1u64, ctx), Err(DecodeError::AllocationLimitExceeded));
        RESERVATION_ATTEMPTS.with(|attempts| assert_eq!(attempts.get(), 0));
        bytes.set(8);
        reserve_vec(&mut out, 1, ctx).unwrap();
        reserve_vec(&mut out, 1, ctx).unwrap(); // spare capacity
        RESERVATION_ATTEMPTS.with(|attempts| assert_eq!(attempts.get(), 1));
    }

    #[cfg(all(feature = "std", target_os = "linux", target_pointer_width = "64"))]
    #[test]
    fn allocation_real_reserve_failure_retains_conservative_debit() {
        let fields = Cell::new(100);
        let bytes = Cell::new(usize::MAX);
        let ctx = DecodeContext::new(10, &fields).with_allocation_budget(&bytes);
        let mut out = Vec::<u8>::new();
        // A valid Layout, but larger than the supported Linux virtual address
        // space. The real fallible allocator refuses without touching pages.
        let requested = isize::MAX as usize;
        assert_eq!(reserve_vec(&mut out, requested, ctx), Err(DecodeError::AllocationReserveFailed));
        assert_eq!((out.len(), out.capacity()), (0, 0));
        assert_eq!(bytes.get(), usize::MAX - requested);
    }

    #[test]
    fn allocation_overflow_shared_descendants_boxes_and_zst() {
        let fields = Cell::new(100);
        let bytes = Cell::new(16);
        let ctx = DecodeContext::new(10, &fields).with_allocation_budget(&bytes);
        assert_eq!(reserve_vec(&mut Vec::<u64>::new(), usize::MAX, ctx), Err(DecodeError::AllocationLayoutOverflow));
        assert_eq!(bytes.get(), 16);
        let boxed = boxed(3u64, ctx.descend().unwrap()).unwrap();
        assert_eq!(*boxed, 3);
        let mut sibling = Vec::<u64>::new();
        reserve_vec(&mut sibling, 1, ctx).unwrap();
        assert_eq!(bytes.get(), 0);
        assert_eq!(super::boxed(1u8, ctx).unwrap_err(), DecodeError::AllocationLimitExceeded);
        reserve_vec(&mut Vec::<()> ::new(), 3, ctx).unwrap();
        assert_eq!(bytes.get(), 0);
    }
}
