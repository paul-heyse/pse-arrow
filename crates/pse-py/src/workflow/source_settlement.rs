// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact original source acknowledgement recovery at the Python integration.
use super::*;
use crate::documents::DocumentValue;
use std::cell::RefCell;

impl NativeRuntime {
    pub(super) fn source_settlement_native(
        &self,
        py: Python<'_>,
        problem: &str,
        operation: &str,
    ) -> PyResult<DocumentValue<native::SourceSettlement>> {
        let stop = CancelSource::new();
        blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |recording| async move {
                recording.source_settlement(problem, operation).await
            }),
            || stop.cancel(),
        )
        .map(DocumentValue)
    }
}

/// Drain the original authoring future after a Python signal and retain actual
/// acknowledged source identity on that signal instead of discarding the result.
pub(super) fn blocking_source<F>(
    py: Python<'_>,
    runtime: &runtime::Runtime,
    future: F,
    cancel: impl Fn(),
) -> PyResult<native::ModelingPackage>
where
    F: Future<Output = Result<native::ModelingPackage, native::WorkflowError>> + Send,
{
    let acknowledged = RefCell::new(None::<(String, String, String)>);
    let result = blocking_on_with_recovery(py, runtime.executor, future, cancel, |value| {
        let receipt = value.canonical_revision()?;
        acknowledged.replace(Some((
            receipt.problem.clone(),
            receipt.operation.clone(),
            receipt.key.clone(),
        )));
        Some(("source_revision", receipt.key.clone(), None))
    });
    if let Err(error) = &result
        && let Some((problem, operation, revision)) = acknowledged.into_inner()
    {
        error.value(py).setattr("source_problem", problem)?;
        error.value(py).setattr("source_operation", operation)?;
        error.value(py).setattr("source_revision", revision)?;
    }
    result
}
