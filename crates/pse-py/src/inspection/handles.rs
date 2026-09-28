// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Publications selected by the operational catalog (reader-leased) or by an export
//! manifest (offline).
use super::{
    errors,
    runtime::{self, Runtime},
    settings::EngineSettings,
    stream::TableStream,
};
use datafusion::common::ResolvedTableReference;
use pse_catalog::delta::publication::Publication as NativePublication;
use pse_columnar::CancellationToken;
use pse_engine::EngineError;
use pse_runtime::workflow::ReaderLeaseGuard;
use pyo3::prelude::*;
use std::{
    num::NonZeroUsize,
    sync::{Arc, Mutex},
};

/// The selected publication and, for a catalog read, the lease protecting it.
#[derive(Debug)]
struct Selected {
    publication: Arc<NativePublication>,
    lease: Option<Arc<ReaderLeaseGuard>>,
}

type Opened = (Arc<NativePublication>, Option<Arc<ReaderLeaseGuard>>);

/// One publication's exact member selection, opened under a catalog reader lease or
/// from an export manifest.
#[pyclass(frozen, module = "pse._native")]
#[derive(Debug)]
pub(crate) struct Publication {
    selected: Mutex<Option<Selected>>,
    runtime: Arc<Runtime>,
}
impl Publication {
    pub(crate) fn leased(
        publication: Arc<NativePublication>,
        lease: Arc<ReaderLeaseGuard>,
        runtime: Arc<Runtime>,
    ) -> Self {
        Self {
            selected: Mutex::new(Some(Selected {
                publication,
                lease: Some(lease),
            })),
            runtime,
        }
    }
    fn selected(&self) -> Result<Opened, EngineError> {
        self.selected
            .lock()
            .map_err(|_| errors::invalid("publication lock poisoned"))?
            .as_ref()
            .map(|selected| (selected.publication.clone(), selected.lease.clone()))
            .ok_or_else(errors::closed)
    }
    fn record<T>(
        &self,
        py: Python<'_>,
        field: impl Fn(&pse_relations::generated::runtime::publication_manifests::Row) -> T,
    ) -> PyResult<T> {
        let (publication, _) = self
            .selected()
            .map_err(|error| errors::diagnostic(py, &error))?;
        Ok(field(publication.record()))
    }
}
#[pymethods]
impl Publication {
    /// The publication identity (32 hexadecimal digits).
    #[getter]
    fn publication_id(&self, py: Python<'_>) -> PyResult<String> {
        self.record(py, |record| record.publication_id.to_hex())
    }
    /// The workspace identity.
    #[getter]
    fn workspace_id(&self, py: Python<'_>) -> PyResult<String> {
        self.record(py, |record| record.workspace_id.to_hex())
    }
    /// The parent publication, if any.
    #[getter]
    fn parent_publication_id(&self, py: Python<'_>) -> PyResult<Option<String>> {
        self.record(py, |record| {
            record.parent_publication_id.map(|id| id.to_hex())
        })
    }
    /// The durable attempt the publication publishes.
    #[getter]
    fn attempt_id(&self, py: Python<'_>) -> PyResult<String> {
        self.record(py, |record| record.attempt_id.to_hex())
    }
    fn tables(&self, py: Python<'_>) -> PyResult<Vec<super::TableName>> {
        Ok(self
            .selected()
            .map_err(|error| errors::diagnostic(py, &error))?
            .0
            .session()
            .inspection_tables()
            .into_iter()
            .map(Into::into)
            .collect())
    }
    fn table(
        &self,
        py: Python<'_>,
        catalog: &str,
        schema: &str,
        table: &str,
    ) -> PyResult<TableStream> {
        py.detach(|| {
            let (publication, lease) = self.selected()?;
            if let Some(lease) = &lease {
                lease
                    .check()
                    .map_err(|error| errors::invalid(&error.to_string()))?;
            }
            let reference = ResolvedTableReference {
                catalog: catalog.into(),
                schema: schema.into(),
                table: table.into(),
            };
            publication.member(&reference)?;
            let batch_size = NonZeroUsize::new(self.runtime.shared.budget().execution.batch_size)
                .ok_or_else(|| errors::invalid("batch size must be positive"))?;
            // A lapsed catalog lease cancels the reads it protected.
            let cancel = lease.map_or_else(CancellationToken::new, |lease| {
                lease.cancellation().clone()
            });
            let reader =
                self.runtime
                    .executor
                    .block_on(pse_catalog::inspection::TableReader::new(
                        publication.session(),
                        &reference,
                        batch_size,
                        cancel,
                    ))?;
            Ok(TableStream::new(reader, Arc::clone(&self.runtime)))
        })
        .map_err(|error: EngineError| errors::diagnostic(py, &error))
    }
    fn cache_usage(&self, py: Python<'_>) -> PyResult<Vec<super::CacheReport>> {
        let report = self
            .runtime
            .shared
            .report()
            .map_err(|error| errors::diagnostic(py, &error))?;
        Ok(report.caches.into_iter().map(Into::into).collect())
    }
    fn resource_usage(&self, py: Python<'_>) -> PyResult<super::ResourceReport> {
        self.runtime
            .shared
            .report()
            .map(Into::into)
            .map_err(|error| errors::diagnostic(py, &error))
    }
    /// Release the selection (and its catalog lease once no stream holds it); existing
    /// streams keep their own ownership.
    fn close(&self, py: Python<'_>) -> PyResult<()> {
        py.detach(|| {
            let released = self
                .selected
                .lock()
                .map_err(|_| errors::invalid("publication lock poisoned"))?
                .take();
            // The lease is released on the runtime that owns its renewal.
            if let Some(released) = released {
                let _enter = self.runtime.executor.enter();
                drop(released);
            }
            Ok(())
        })
        .map_err(|error: EngineError| errors::diagnostic(py, &error))
    }
}
/// Open an exported publication offline: exactly the members its manifest names, while
/// the export's lease has not expired. No operational store is contacted.
#[pyfunction]
#[pyo3(signature = (location, settings))]
pub(crate) fn open_export(
    py: Python<'_>,
    location: &str,
    settings: &EngineSettings,
) -> PyResult<Publication> {
    let opened = py.detach(|| {
        let location = url::Url::parse(location)
            .map_err(|_| errors::invalid("an export manifest location is an absolute URI"))?;
        let runtime = runtime::acquire(settings)?;
        let publication = runtime.executor.block_on(pse_runtime::workflow::open_export(
            location,
            Arc::clone(&runtime.registry),
            &runtime.sessions,
            &CancellationToken::new(),
        ));
        Ok::<_, EngineError>((publication, runtime))
    });
    let (publication, runtime) = opened.map_err(|error| errors::diagnostic(py, &error))?;
    let publication = publication.map_err(|error| errors::diagnostic(py, &error))?;
    Ok(Publication {
        selected: Mutex::new(Some(Selected {
            publication: Arc::new(publication),
            lease: None,
        })),
        runtime,
    })
}
