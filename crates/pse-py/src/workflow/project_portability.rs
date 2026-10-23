// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Typed project exchange and lifecycle projections over their original owners.
use super::*;
use std::path::PathBuf;

#[pyclass(frozen, module = "pse._native")]
#[derive(Debug)]
pub(crate) struct NativeProjectBundle {
    inner: Arc<native::ProjectBundleReader>,
}
#[pymethods]
impl NativeProjectBundle {
    #[new]
    #[pyo3(signature=(path, *, maximum_document_bytes=16_777_216))]
    fn new(py: Python<'_>, path: PathBuf, maximum_document_bytes: usize) -> PyResult<Self> {
        py.detach(|| native::ProjectBundleReader::open(&path, maximum_document_bytes))
            .map(|inner| Self { inner: Arc::new(inner) })
            .map_err(|error| errors::diagnostic(py, &error))
    }
    #[getter]
    fn manifest(&self) -> documents::DocumentValue<native::ProjectManifest> {
        documents::DocumentValue(self.inner.inspect().clone())
    }
    #[getter]
    fn digest(&self) -> &str { self.inner.digest() }
    #[pyo3(signature=(member, *, maximum_bytes=16_777_216))]
    fn member_bytes<'py>(&self, py: Python<'py>, member: &str, maximum_bytes: usize) -> PyResult<Bound<'py, pyo3::types::PyBytes>> {
        let bytes = py.detach(|| {
            if maximum_bytes == 0 || maximum_bytes > 64 << 20 {
                return Err(native::WorkflowError::Input("member byte inspection requires an allowance up to 64MiB; stream larger members through the CLI".into()));
            }
            let bytes = self.inner.member_bytes(member, maximum_bytes)?;
            Ok(bytes)
        }).map_err(|error| errors::diagnostic(py, &error))?;
        Ok(pyo3::types::PyBytes::new(py, &bytes))
    }
}
impl NativeRuntime {
    pub(super) fn project_closure_native(&self, py: Python<'_>, request: documents::DocumentInput<native::ProjectExportRequest>, physical: Option<&NativePhysicalContext>) -> PyResult<documents::DocumentValue<native::ProjectClosureExplanation>> {
        let stop=CancelSource::new(); let cancel=&stop;
        blocking(py,&self.owner,self.recording_operation(&stop,|runtime|async move {
            match physical {Some(physical)=>Box::pin(runtime.project_closure(&request.0,physical.inner.clone(),cancel)).await, None=>Box::pin(runtime.project_closure_metadata(&request.0,cancel)).await}
        }),||stop.cancel()).map(documents::DocumentValue)
    }
    pub(super) fn project_export_native(&self, py: Python<'_>, destination: PathBuf, request: documents::DocumentInput<native::ProjectExportRequest>, physical: Option<&NativePhysicalContext>) -> PyResult<documents::DocumentValue<native::ProjectManifest>> {
        let stop=CancelSource::new(); let cancel=&stop;
        blocking(py,&self.owner,self.recording_operation(&stop,|runtime|async move {
            match physical {Some(physical)=>Box::pin(runtime.project_export(&destination,&request.0,physical.inner.clone(),cancel)).await, None=>Box::pin(runtime.project_export_metadata(&destination,&request.0,cancel)).await}
        }),||stop.cancel()).map(documents::DocumentValue)
    }
    pub(super) fn project_import_issue_native(&self, py: Python<'_>, bundle: &NativeProjectBundle, title: &str, branch_name: &str) -> PyResult<documents::DocumentValue<native::ProjectImportIntent>> {
        let stop=CancelSource::new(); let bundle=bundle.inner.clone();
        blocking(py,&self.owner,self.recording_operation(&stop,|runtime|async move {
            Box::pin(runtime.project_import_issue(&bundle,title,branch_name)).await
        }),||stop.cancel()).map(documents::DocumentValue)
    }
    pub(super) fn project_import_native(&self, py: Python<'_>, bundle: &NativeProjectBundle, intent: documents::DocumentInput<native::ProjectImportIntent>) -> PyResult<documents::DocumentValue<native::ProjectImportReport>> {
        let stop=CancelSource::new(); let cancel=&stop; let bundle=bundle.inner.clone();
        blocking(py,&self.owner,self.recording_operation(&stop,|runtime|async move {
            Box::pin(runtime.project_import(&bundle,&intent.0,cancel)).await
        }),||stop.cancel()).map(documents::DocumentValue)
    }
    pub(super) fn project_import_status_native(&self, py: Python<'_>, intent: documents::DocumentInput<native::ProjectImportIntent>) -> PyResult<Option<documents::DocumentValue<native::ProjectImportReport>>> {
        let stop=CancelSource::new();
        blocking(py,&self.owner,self.recording_operation(&stop,|runtime|async move {
            runtime.project_import_status(&intent.0).await
        }),||stop.cancel()).map(|value| value.map(documents::DocumentValue))
    }
    pub(super) fn project_import_recover_native(&self, py: Python<'_>, intent: documents::DocumentInput<native::ProjectImportIntent>, original: documents::DocumentInput<pse_operations::canonical_artifacts::Artifact>) -> PyResult<documents::DocumentValue<native::ProjectImportRecoveryReport>> {
        let stop=CancelSource::new();let cancel=&stop;
        blocking(py,&self.owner,self.recording_operation(&stop,|runtime|async move {
            Box::pin(runtime.project_import_recover(&intent.0,&original.0,cancel)).await
        }),||stop.cancel()).map(documents::DocumentValue)
    }
    pub(super) fn project_import_open_native(&self, py: Python<'_>, intent: documents::DocumentInput<native::ProjectImportIntent>) -> PyResult<NativeModelingPackage> {
        let stop=CancelSource::new(); let cancel=&stop;
        let package=blocking(py,&self.owner,self.recording_operation(&stop,|runtime|async move {
            Box::pin(runtime.project_import_open(&intent.0,cancel)).await
        }),||stop.cancel())?;
        Ok(NativeModelingPackage::from_revision(self.owner.clone(),package))
    }
    pub(super) fn lifecycle_select_native(&self,py:Python<'_>,target:documents::DocumentInput<pse_operations::canonical_lifecycle::LifecycleTarget>)->PyResult<documents::DocumentValue<pse_operations::canonical_lifecycle::LifecycleSelection>> {
        let stop=CancelSource::new();
        blocking(py,&self.owner,self.recording_operation(&stop,|runtime|async move {
            runtime.lifecycle_select(&target.0).await
        }),||stop.cancel()).map(documents::DocumentValue)
    }
    pub(super) fn lifecycle_explain_native(&self,py:Python<'_>,selection:documents::DocumentInput<pse_operations::canonical_lifecycle::LifecycleSelection>)->PyResult<documents::DocumentValue<pse_operations::canonical_lifecycle::LifecycleExplanation>> {
        let stop=CancelSource::new();
        blocking(py,&self.owner,self.recording_operation(&stop,|runtime|async move {
            runtime.lifecycle_explain(&selection.0).await
        }),||stop.cancel()).map(documents::DocumentValue)
    }
    pub(super) fn lifecycle_release_native(&self,py:Python<'_>,selection:documents::DocumentInput<pse_operations::canonical_lifecycle::LifecycleSelection>)->PyResult<documents::DocumentValue<pse_operations::canonical_lifecycle::LifecycleRelease>> {
        let stop=CancelSource::new();
        blocking(py,&self.owner,self.recording_operation(&stop,|runtime|async move {
            runtime.lifecycle_release(&selection.0).await
        }),||stop.cancel()).map(documents::DocumentValue)
    }
}
