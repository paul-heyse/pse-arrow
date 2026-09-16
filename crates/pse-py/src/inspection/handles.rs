// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use std::{
    num::NonZeroUsize,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use pse_catalog::{
    Catalog, CatalogError, RefName, SystemClock, TrustLevel, snapshot::ManifestRef,
    store::refs::RefState,
};
use pse_ids::{CancellationToken, ContentHash, EncodingChecksum, SnapshotId};
use pyo3::prelude::*;

use super::{
    errors,
    runtime::{self, Runtime},
    settings::EngineSettings,
    stream::TableStream,
};

/// An immutable inspection capability over one existing local store.
#[pyclass(frozen, module = "pse._native")]
#[derive(Debug)]
pub(crate) struct Store {
    runtime: Arc<Runtime>,
    catalog: Mutex<Option<Arc<Catalog>>>,
    cancel: CancellationToken,
}

#[derive(Debug)]
struct Pinned {
    snapshot: Arc<pse_catalog::Snapshot>,
    session: pse_catalog::session::SnapshotSession,
    reference: Option<RefState>,
}

/// An exact admitted snapshot whose identity cannot follow later ref updates.
#[pyclass(frozen, module = "pse._native")]
#[derive(Debug)]
pub(crate) struct Snapshot {
    pinned: Mutex<Option<Arc<Pinned>>>,
    runtime: Arc<Runtime>,
}

impl Store {
    fn catalog(&self) -> Result<Arc<Catalog>, CatalogError> {
        self.cancel.checkpoint()?;
        self.catalog
            .lock()
            .map_err(|_| errors::invalid("store lock poisoned"))?
            .as_ref()
            .cloned()
            .ok_or_else(errors::closed)
    }
    fn snapshot(
        &self,
        snapshot: Arc<pse_catalog::Snapshot>,
        reference: Option<RefState>,
    ) -> Result<Snapshot, CatalogError> {
        let session = self
            .runtime
            .sessions
            .inspect_snapshot(&snapshot, &self.cancel)?;
        Ok(Snapshot {
            pinned: Mutex::new(Some(Arc::new(Pinned {
                snapshot,
                reference,
                session,
            }))),
            runtime: Arc::clone(&self.runtime),
        })
    }
}

#[pymethods]
impl Store {
    #[pyo3(signature = (ref_name="main"))]
    fn head(&self, py: Python<'_>, ref_name: &str) -> PyResult<Snapshot> {
        py.detach(|| {
            let catalog = self.catalog()?;
            let name = RefName::parse(ref_name)?;
            let (reference, snapshot) = self
                .runtime
                .executor
                .block_on(catalog.read_pinned_ref(&name, &self.cancel))?
                .ok_or_else(|| CatalogError::Admission {
                    path: "inspection ref".to_owned(),
                    reason: format!("ref {ref_name:?} is absent"),
                })?;
            self.snapshot(snapshot, Some(reference))
        })
        .map_err(|error: CatalogError| errors::diagnostic(py, &error))
    }

    #[pyo3(name = "snapshot")]
    fn open_snapshot(
        &self,
        py: Python<'_>,
        snapshot_id: &str,
        manifest_checksum: &str,
    ) -> PyResult<Snapshot> {
        let reference = ManifestRef {
            snapshot_id: SnapshotId(
                ContentHash::parse_prefixed(snapshot_id)
                    .map_err(|error| errors::diagnostic(py, &error))?,
            ),
            manifest_checksum: EncodingChecksum(
                ContentHash::parse_prefixed(manifest_checksum)
                    .map_err(|error| errors::diagnostic(py, &error))?,
            ),
        };
        py.detach(|| {
            let catalog = self.catalog()?;
            let snapshot = self
                .runtime
                .executor
                .block_on(catalog.read_pinned_manifest(reference, &self.cancel))?;
            self.snapshot(snapshot, None)
        })
        .map_err(|error: CatalogError| errors::diagnostic(py, &error))
    }

    fn cancel(&self) {
        self.cancel.cancel();
    }

    fn close(&self, py: Python<'_>) -> PyResult<()> {
        self.cancel.cancel();
        self.catalog
            .lock()
            .map_err(|_| errors::diagnostic(py, &errors::invalid("store lock poisoned")))?
            .take();
        Ok(())
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
}

impl Snapshot {
    fn pin(&self) -> Result<Arc<Pinned>, CatalogError> {
        self.pinned
            .lock()
            .map_err(|_| errors::invalid("snapshot lock poisoned"))?
            .as_ref()
            .cloned()
            .ok_or_else(errors::closed)
    }
}

#[pymethods]
impl Snapshot {
    #[getter]
    fn snapshot_id(&self, py: Python<'_>) -> PyResult<String> {
        Ok(self
            .pin()
            .map_err(|error| errors::diagnostic(py, &error))?
            .snapshot
            .snapshot_id()
            .0
            .to_prefixed())
    }
    #[getter]
    fn manifest_checksum(&self, py: Python<'_>) -> PyResult<String> {
        Ok(self
            .pin()
            .map_err(|error| errors::diagnostic(py, &error))?
            .snapshot
            .manifest_ref()
            .manifest_checksum
            .0
            .to_prefixed())
    }
    #[getter]
    fn revision_id(&self, py: Python<'_>) -> PyResult<Option<String>> {
        Ok(self
            .pin()
            .map_err(|error| errors::diagnostic(py, &error))?
            .reference
            .as_ref()
            .and_then(RefState::revision_ref)
            .map(|revision| revision.revision_id.to_string()))
    }
    fn tables(&self, py: Python<'_>) -> PyResult<Vec<(String, String)>> {
        let pinned = self.pin().map_err(|error| errors::diagnostic(py, &error))?;
        Ok(pinned.session.inspection_tables())
    }
    #[pyo3(signature = (relation, name=None, *, port=None))]
    fn table(
        &self,
        py: Python<'_>,
        relation: &str,
        name: Option<&str>,
        port: Option<&str>,
    ) -> PyResult<TableStream> {
        let result = py.detach(|| {
            let pinned = self.pin()?;
            let qualified =
                name.map_or_else(|| relation.to_owned(), |name| format!("{relation}.{name}"));
            let batch_size =
                NonZeroUsize::new(self.runtime.shared.budget().execution.batch_size)
                    .ok_or_else(|| errors::invalid("configured batch size must be positive"))?;
            let reader =
                self.runtime
                    .executor
                    .block_on(pse_catalog::inspection::TableReader::new(
                        &pinned.session,
                        &qualified,
                        port,
                        batch_size,
                        CancellationToken::new(),
                    ))?;
            Ok(TableStream::new(reader, Arc::clone(&self.runtime)))
        });
        result.map_err(|error: CatalogError| errors::diagnostic(py, &error))
    }
    fn close(&self, py: Python<'_>) -> PyResult<()> {
        self.pinned
            .lock()
            .map_err(|_| errors::diagnostic(py, &errors::invalid("snapshot lock poisoned")))?
            .take();
        Ok(())
    }
}

#[pyfunction]
#[pyo3(signature = (path: "str", settings))]
pub(crate) fn open_store(
    py: Python<'_>,
    path: PathBuf,
    settings: &EngineSettings,
) -> PyResult<Store> {
    py.detach(|| {
        let runtime = runtime::acquire(settings)?;
        let catalog = Catalog::open_existing_local(
            &path,
            Arc::clone(&runtime.registry),
            TrustLevel::Untrusted,
            Arc::new(SystemClock),
            Arc::clone(&runtime.sessions),
        )?
        .with_limits(settings.limits)?
        .with_semantic_validator(Arc::clone(&runtime.validator));
        Ok(Store {
            runtime,
            catalog: Mutex::new(Some(Arc::new(catalog))),
            cancel: CancellationToken::new(),
        })
    })
    .map_err(|error: CatalogError| errors::diagnostic(py, &error))
}
