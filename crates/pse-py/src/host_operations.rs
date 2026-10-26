// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original compiled operation catalogue, shared with source-only host projections.
use pyo3::prelude::*;

/// Passive copied metadata; this never constructs a runtime, service or provider.
#[pyfunction]
pub(crate) fn host_operation_catalogue() -> crate::documents::DocumentValue<pse_runtime::host_operations::OperationCatalogue> {
    crate::documents::DocumentValue(pse_runtime::host_operations::catalogue())
}
