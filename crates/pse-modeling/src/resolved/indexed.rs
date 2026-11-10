// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-issued contraction correspondence captured before scalar expansion.
use super::*;
use crate::specialize::{InstanceId, Value};

/// One original tensor argument. Fixed physical coordinates never become dummy labels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContractionAxis {
    /// The original directly bound reduction coordinate.
    Dummy,
    /// An original resolved fixed external coordinate.
    Fixed(Value),
}
/// An actual indexed family occurrence and its ordered selected scalar members.
#[derive(Clone, Debug, PartialEq)]
pub struct ContractionFactor {
    /// Actual original specialized family instance.
    pub instance: InstanceId,
    /// Actual original indexed declaration.
    pub declaration: DeclarationId,
    /// Original argument order, distinguishing bound and external coordinates.
    pub axes: Vec<ContractionAxis>,
    /// Actual selected scalar members in original carrier order.
    pub components: Vec<SemanticId>,
    /// Original authored occurrence; never inferred from its expanded scalar text.
    pub source: Source,
}
/// Original selected finite sum of two indexed factors. This is source correspondence,
/// not another executable tensor language or a reconstruction from equal scalar text.
#[derive(Clone, Debug, PartialEq)]
pub struct OriginalContraction {
    /// Original authored reduction occurrence.
    pub source: Source,
    /// Original bound coordinate spelling, retained for correspondence.
    pub binder_spelling: String,
    /// Original resolved carrier and its typed complete membership.
    pub carrier: Carrier,
    /// Membership admitted by the original selection/filter.
    pub selected: Arc<[Value]>,
    /// Original supported multiplication operands, in source order.
    pub factors: [ContractionFactor; 2],
    /// Original inferred reduction prototype.
    pub prototype: Type,
    /// Whether an original filter restricted the carrier.
    pub filtered: bool,
}
impl OriginalContraction {
    /// Retained source correspondence storage, independently of reusable mathematics.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.binder_spelling.capacity()
            + self.source.field.role.capacity()
            + self.carrier.retained_bytes()
            + self
                .selected
                .iter()
                .map(Value::retained_bytes)
                .sum::<usize>()
            + self
                .factors
                .iter()
                .map(|factor| {
                    factor.axes.capacity() * size_of::<ContractionAxis>()
                        + factor.components.capacity() * size_of::<SemanticId>()
                        + factor.source.field.role.capacity()
                })
                .sum::<usize>()
            + extent::ty(&self.prototype)
    }
}
