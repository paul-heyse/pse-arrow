// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Mechanical research projections; scientific admission and publication stay native.
use super::*;
impl NativeRuntime {
    pub(super) fn research_dataset_create_native(
        &self,
        py: Python<'_>,
        request: documents::DocumentInput<native::ResearchDatasetRequest>,
    ) -> PyResult<documents::DocumentValue<pse_operations::canonical_research::ResearchDataset>>
    {
        let stop = CancelSource::new();
        let cancel = &stop;
        blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                runtime.research_dataset_create(&request.0, cancel).await
            }),
            || stop.cancel(),
        )
        .map(documents::DocumentValue)
    }
    pub(super) fn research_selection_create_native(
        &self,
        py: Python<'_>,
        request: documents::DocumentInput<native::ResearchSelectionRequest>,
    ) -> PyResult<documents::DocumentValue<pse_operations::canonical_research::ResearchSelection>>
    {
        let stop = CancelSource::new();
        let cancel = &stop;
        blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                runtime.research_selection_create(&request.0, cancel).await
            }),
            || stop.cancel(),
        )
        .map(documents::DocumentValue)
    }
    pub(super) fn research_read_native(
        &self,
        py: Python<'_>,
        request: documents::DocumentInput<native::ResearchReadRequest>,
    ) -> PyResult<documents::DocumentValue<native::ResearchReadResult>> {
        let stop = CancelSource::new();
        blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                runtime.research_read(&request.0).await
            }),
            || stop.cancel(),
        )
        .map(documents::DocumentValue)
    }
    pub(super) fn research_dataset_open_native(
        &self,
        py: Python<'_>,
        key: &str,
        context: documents::DocumentInput<pse_operations::canonical_contexts::ContextSelection>,
    ) -> PyResult<NativeModelingPackage> {
        let stop = CancelSource::new();
        let cancel = &stop;
        let package = blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                runtime.research_dataset_open(key, &context.0, cancel).await
            }),
            || stop.cancel(),
        )?;
        Ok(NativeModelingPackage::from_revision(
            self.owner.clone(),
            self.recording_runtime(py)?.execution_services(),
            package,
        ))
    }
    pub(super) fn research_selection_open_native(
        &self,
        py: Python<'_>,
        key: &str,
    ) -> PyResult<NativeModelingPackage> {
        let stop = CancelSource::new();
        let cancel = &stop;
        let (package, _) = blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                runtime.research_selection_open(key, cancel).await
            }),
            || stop.cancel(),
        )?;
        Ok(NativeModelingPackage::from_revision(
            self.owner.clone(),
            self.recording_runtime(py)?.execution_services(),
            package,
        ))
    }
    pub(super) fn research_publication_preview_native(
        &self,
        py: Python<'_>,
        request: documents::DocumentInput<native::ResearchPublicationRequest>,
    ) -> PyResult<documents::DocumentValue<native::ResearchPublicationPreview>> {
        let stop = CancelSource::new();
        let cancel = &stop;
        blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                runtime
                    .research_publication_preview(&request.0, cancel)
                    .await
            }),
            || stop.cancel(),
        )
        .map(documents::DocumentValue)
    }
    pub(super) fn research_publish_native(
        &self,
        py: Python<'_>,
        intent: documents::DocumentInput<native::ResearchPublicationIntent>,
    ) -> PyResult<documents::DocumentValue<pse_operations::canonical_research::ResearchPublication>>
    {
        let stop = CancelSource::new();
        let cancel = &stop;
        blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                runtime.research_publish(&intent.0, cancel).await
            }),
            || stop.cancel(),
        )
        .map(documents::DocumentValue)
    }
    pub(super) fn research_publication_open_native(
        &self,
        py: Python<'_>,
        key: &str,
    ) -> PyResult<NativeModelingPackage> {
        let stop = CancelSource::new();
        let cancel = &stop;
        let package = blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                runtime.research_publication_open(key, cancel).await
            }),
            || stop.cancel(),
        )?;
        Ok(NativeModelingPackage::from_revision(
            self.owner.clone(),
            self.recording_runtime(py)?.execution_services(),
            package,
        ))
    }
}
