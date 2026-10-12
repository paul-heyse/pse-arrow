// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Read-only registry reflection uses the actual declared Arrow contract.

use super::{EngineSettings, TableStream, errors, runtime};
use pyo3::prelude::*;

/// Read one declared reference relation from the compiled registry.
#[pyfunction]
#[pyo3(signature=(name,*,settings))]
pub(crate) fn registry_table(
    py: Python<'_>,
    name: &str,
    settings: &EngineSettings,
) -> PyResult<TableStream> {
    py.detach(|| {
        let runtime = runtime::acquire(settings)?;
        let checked = runtime
            .model
            .registry_relation(name, &pse_columnar::CancellationToken::new())?;
        TableStream::from_batch(checked)
    })
    .map_err(|error| errors::diagnostic(py, &error))
}
