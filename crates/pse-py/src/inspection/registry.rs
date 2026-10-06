// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Read-only registry reflection uses the actual declared Arrow contract.

use super::{EngineSettings,TableStream,errors,runtime};
use pyo3::prelude::*;

/// Read one declared reference relation from the compiled registry.
#[pyfunction]
#[pyo3(signature=(name,*,settings))]
pub(crate) fn registry_table(py:Python<'_>,name:&str,settings:&EngineSettings)->PyResult<TableStream> {
    py.detach(|| {
        let runtime=runtime::acquire(settings)?;
        let (key,batch)=runtime.registry.schema_batches().iter().find(|(key,_)|key.qualified_name()==name)
            .ok_or_else(||errors::invalid("unknown declared registry reflection relation"))?;
        let spec=runtime.registry.relation_by_key(*key).ok_or_else(||errors::invalid("registry reflection declaration absent"))?;
        let validation=runtime.sessions.validation_context(&runtime.registry)?;
        let checked=pse_relations::columnar::FieldCheckedBatch::admit(&runtime.registry,spec,batch.clone(),&validation,&pse_columnar::CancellationToken::new())
            .map_err(|error|pse_engine::EngineError::Semantic(std::sync::Arc::new(error)))?;
        TableStream::from_batch(checked)
    }).map_err(|error|errors::diagnostic(py,&error))
}
