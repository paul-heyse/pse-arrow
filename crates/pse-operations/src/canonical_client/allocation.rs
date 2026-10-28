// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit custody of existing prepaid response reservations. Composition never
//! allocates bytes from a pool or acquires connection or scientific authority.
use super::RequestContext;
use datafusion_execution::memory_pool::MemoryReservation;
use std::sync::Arc;

/// Existing response allocation custody, including enclosing independent grants.
#[derive(Clone, Debug)]
pub struct ResponseAllocation(Arc<Node>);
#[derive(Debug)]
enum Node {
    Reservation(Arc<MemoryReservation>),
    Scientific(Arc<pse_columnar::AllocationLease>),
    Joined(ResponseAllocation, ResponseAllocation),
}
impl From<Arc<MemoryReservation>> for ResponseAllocation {
    fn from(reservation: Arc<MemoryReservation>) -> Self {
        Self(Arc::new(Node::Reservation(reservation)))
    }
}
impl From<Arc<pse_columnar::AllocationLease>> for ResponseAllocation {
    fn from(lease: Arc<pse_columnar::AllocationLease>) -> Self {
        Self(Arc::new(Node::Scientific(lease)))
    }
}
impl ResponseAllocation {
    /// Retain two independently issued response allocations through the same request.
    /// This acquires no bytes, activity or scientific authority.
    pub fn combine(enclosing: impl Into<Self>, current: impl Into<Self>) -> Self {
        Self::joined(enclosing.into(), current.into())
    }

    pub(crate) fn joined(enclosing: Self, current: Self) -> Self {
        if enclosing.same_owner(&current) {
            return enclosing;
        }
        Self(Arc::new(Node::Joined(enclosing, current)))
    }
    /// Whether both handles retain the exact same original reservation identity.
    pub(crate) fn same_owner(&self, other: &Self) -> bool {
        if Arc::ptr_eq(&self.0, &other.0) {
            return true;
        }
        match (self.0.as_ref(), other.0.as_ref()) {
            (Node::Reservation(first), Node::Reservation(second)) => Arc::ptr_eq(first, second),
            (Node::Scientific(first), Node::Scientific(second)) => Arc::ptr_eq(first, second),
            _ => false,
        }
    }
    pub(crate) fn apply(&self, context: RequestContext) -> RequestContext {
        match self.0.as_ref() {
            Node::Reservation(reservation) => context.with_payload_owner(reservation.clone()),
            Node::Scientific(lease) => context.with_scientific_payload_owner(lease.clone()),
            Node::Joined(enclosing, current) => current.apply(enclosing.apply(context)),
        }
    }
}
