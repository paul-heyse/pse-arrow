// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native physical inventory and explicit mathematical algorithm adapters.
pub mod inventory;
pub(crate) mod kernel_contract;
pub use inventory::PhysicalInventory;
mod source;
pub use source::RelationSymbolSource;

use crate::CompilerError;
use pse_quantity::QuantityError;

fn invalid(detail: impl Into<String>) -> CompilerError {
    QuantityError::InferencePrecondition {
        rule: "quantity.relation_admission",
        detail: detail.into(),
    }
    .into()
}
