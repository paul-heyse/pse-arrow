// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact Delta roots replace manifest/hash and mutable-ref inspection handles.
use super::{
    errors,
    runtime::{self, Runtime},
    settings::EngineSettings,
    stream::TableStream,
};
use datafusion::common::ResolvedTableReference;
use pse_catalog::{
    CatalogError,
    delta::publication::{Publication as NativePublication, PublicationRoot},
};
use pse_ids::CancellationToken;
use pyo3::prelude::*;
use std::{
    num::NonZeroUsize,
    sync::{Arc, Mutex},
};

/// One exact Delta control version and its complete selected member vector.
#[pyclass(frozen, module = "pse._native")]
#[derive(Debug)]
pub(crate) struct Publication {
    publication: Mutex<Option<Arc<NativePublication>>>,
    runtime: Arc<Runtime>,
}
impl Publication {
    fn selected(&self) -> Result<Arc<NativePublication>, CatalogError> {
        self.publication
            .lock()
            .map_err(|_| errors::invalid("publication lock poisoned"))?
            .as_ref()
            .cloned()
            .ok_or_else(errors::closed)
    }
}
#[pymethods]
impl Publication {
    #[getter]
    fn location(&self, py: Python<'_>) -> PyResult<String> {
        Ok(self
            .selected()
            .map_err(|error| errors::diagnostic(py, &error))?
            .root()
            .location
            .to_string())
    }
    #[getter]
    fn version(&self, py: Python<'_>) -> PyResult<i64> {
        Ok(self
            .selected()
            .map_err(|error| errors::diagnostic(py, &error))?
            .root()
            .version)
    }
    fn tables(&self, py: Python<'_>) -> PyResult<Vec<(String, String, String)>> {
        Ok(self
            .selected()
            .map_err(|error| errors::diagnostic(py, &error))?
            .session()
            .inspection_tables())
    }
    fn table(
        &self,
        py: Python<'_>,
        catalog: &str,
        schema: &str,
        table: &str,
    ) -> PyResult<TableStream> {
        py.detach(|| {
            let publication = self.selected()?;
            let reference = ResolvedTableReference {
                catalog: catalog.into(),
                schema: schema.into(),
                table: table.into(),
            };
            publication.member(&reference)?;
            let batch_size = NonZeroUsize::new(self.runtime.shared.budget().execution.batch_size)
                .ok_or_else(|| errors::invalid("batch size must be positive"))?;
            let reader =
                self.runtime
                    .executor
                    .block_on(pse_catalog::inspection::TableReader::new(
                        publication.session(),
                        &reference,
                        batch_size,
                        CancellationToken::new(),
                    ))?;
            Ok(TableStream::new(reader, Arc::clone(&self.runtime)))
        })
        .map_err(|error: CatalogError| errors::diagnostic(py, &error))
    }
    fn cache_usage(&self, py: Python<'_>) -> PyResult<Vec<super::CacheReport>> {
        let report = self
            .runtime
            .shared
            .report()
            .map_err(|error| errors::diagnostic(py, &error))?;
        Ok(report.caches.into_iter().map(Into::into).collect())
    }
    fn resource_usage(&self, py: Python<'_>) -> PyResult<(usize, usize, usize, Option<u64>)> {
        let report = self
            .runtime
            .shared
            .report()
            .map_err(|error| errors::diagnostic(py, &error))?;
        Ok((
            report.limit_bytes,
            report.pool_reserved_now,
            report.pool_peak_bytes,
            report.process_peak_rss_bytes,
        ))
    }
    fn close(&self, py: Python<'_>) -> PyResult<()> {
        self.publication
            .lock()
            .map_err(|_| errors::diagnostic(py, &errors::invalid("publication lock poisoned")))?
            .take();
        Ok(())
    }
}
/// Open one explicit existing Delta publication. No latest-version lookup is implicit.
#[pyfunction]
#[pyo3(signature = (location, version, settings))]
pub(crate) fn open_publication(
    py: Python<'_>,
    location: &str,
    version: i64,
    settings: &EngineSettings,
) -> PyResult<Publication> {
    py.detach(|| {
        if version < 0 {
            return Err(errors::invalid("publication version must be nonnegative"));
        }
        let location = url::Url::parse(location)
            .map_err(|_| errors::invalid("publication location must be an absolute URI"))?;
        let runtime = runtime::acquire(settings)?;
        let publication = runtime.executor.block_on(NativePublication::open(
            PublicationRoot { location, version },
            Arc::clone(&runtime.registry),
            &runtime.sessions,
            &CancellationToken::new(),
        ))?;
        Ok(Publication {
            publication: Mutex::new(Some(Arc::new(publication))),
            runtime,
        })
    })
    .map_err(|error: CatalogError| errors::diagnostic(py, &error))
}
