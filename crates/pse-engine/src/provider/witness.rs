// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Retained, typed source evidence supplied by the actual provider owner.
use datafusion::common::{Result, TableReference};
use std::{any::Any, sync::Arc};
/// Owner-specific evidence; engine never invents a durable identity for derived values.
pub trait InputWitness: std::fmt::Debug + Send + Sync {
    /// Fully qualified source role for detecting conflicting source revisions.
    fn reference(&self) -> TableReference;
    /// Actual owner-defined typed evidence.
    fn value(&self) -> &dyn Any;
    /// Equality under this owner's source contract.
    fn equivalent(&self, other: &dyn InputWitness) -> bool;
    /// Diagnostic encoding only; never a semantic hash or executable replacement.
    /// # Errors
    /// The source owner cannot encode its diagnostic descriptor.
    fn descriptor(&self) -> Result<Vec<u8>>;
}
/// A source witness retains its real implementation owner through native rewrites.
#[derive(Clone, Debug)]
pub struct SourceWitness(Arc<dyn InputWitness>);
impl SourceWitness {
    /// Attach an owner's implementation to its actual source binding.
    pub fn new(value: Arc<dyn InputWitness>) -> Self {
        Self(value)
    }
    /// Qualified source role.
    pub fn reference(&self) -> TableReference {
        self.0.reference()
    }
    /// Borrow the owner's original value without serialization or reconstruction.
    pub fn value<T: Any>(&self) -> Option<&T> {
        self.0.value().downcast_ref()
    }
    /// Bounded by the enclosing diagnostic codec's allocation admission.
    /// # Errors
    /// The source owner cannot encode its diagnostic descriptor.
    pub fn descriptor(&self) -> Result<Vec<u8>> {
        self.0.descriptor()
    }
}
impl PartialEq for SourceWitness {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0) || self.0.equivalent(other.0.as_ref())
    }
}
impl Eq for SourceWitness {}
/// A real resource owner retained until all dependent native work releases it.
pub trait ExecutionOwner: std::fmt::Debug + Send + Sync {}
impl<T: std::fmt::Debug + Send + Sync> ExecutionOwner for T {}
