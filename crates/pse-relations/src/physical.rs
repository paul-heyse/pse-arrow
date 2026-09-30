// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Pure projection of selected checked physical declarations into the quantity owner.
mod decode;
mod units;
pub use decode::inventory;
pub use units::{UnitReconciliation, reconcile_units};

/// Declaration families consumed by value compatibility; operation families are separate.
pub const VALUE_INPUTS: &[&str] = &[
    "authored.modeling_declarations",
    "reference.units",
    "normalized.units",
    "reference.quantity_kinds",
    "reference.bases",
    "reference.reference_states",
    "reference.quantity_types",
];

fn invalid(detail: impl Into<String>) -> PhysicalProjectionError {
    pse_quantity::QuantityError::InferencePrecondition {
        rule: "quantity.relation_admission",
        detail: detail.into(),
    }
    .into()
}

/// Original physical projection failure, without a dependency on an execution owner.
#[derive(Debug, thiserror::Error)]
pub enum PhysicalProjectionError {
    /// The quantity definition or operation is invalid.
    #[error(transparent)]
    Quantity(#[from] pse_quantity::QuantityError),
    /// The selected row does not meet its declaration.
    #[error(transparent)]
    Relation(#[from] crate::RelationError),
    /// Cancellation or allocation failure.
    #[error(transparent)]
    Canon(#[from] pse_columnar::CanonError),
    /// The source declaration is unavailable.
    #[error(transparent)]
    Schema(#[from] pse_schema::SchemaError),
}
pse_diagnostics::impl_diagnostic! {
    PhysicalProjectionError,
    code(_this){None},
    forward(this){match this {
        Self::Quantity(e)=>Some(e), Self::Relation(e)=>Some(e),
        Self::Canon(e)=>Some(e), Self::Schema(e)=>Some(e),
    }},
    help(_this){None},related(_this){None},source(_this){None}
}
