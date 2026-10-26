// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual typed receiver conversion/cancellation glue; public adapters are generated.
use super::*;

impl NativeRuntime {
    /// Browse bounded compact metadata, including terminal studies.
    pub(super) fn _host_study_workbench_page(
        &self,
        py: Python<'_>,
        request: &[u8],
    ) -> PyResult<documents::DocumentValue<native::StudyWorkbenchPage>> {
        self.study_workbench_page_native(py, request)
    }
    /// Freeze a complete bounded cohort before protected analytical reads.
    pub(super) fn _host_freeze_study_cohort(
        &self,
        py: Python<'_>,
        request: &[u8],
    ) -> PyResult<NativeFrozenStudyCohort> {
        self.freeze_study_cohort_native(py, request)
    }
    /// Open or reconnect passive observation of one exact selected study.
    pub(super) fn _host_observe_study(
        &self,
        py: Python<'_>,
        request: &[u8],
        cursor: Option<&[u8]>,
    ) -> PyResult<NativeStudyObserver> {
        self.observe_study_native(py, request, cursor)
    }
    /// Reopen an exact retained canonical source selection without authored document replay.
    pub(super) fn _host_modeling_revision(
        &self,
        py: Python<'_>,
        revision: &str,
        physical: &NativePhysicalContext,
    ) -> PyResult<NativeModelingPackage> {
        let connection_stop = CancelSource::new();
        let inner = blocking(
            py,
            &self.owner,
            self.recording_operation(&connection_stop, |recording| async move {
                (async {
                    let revision = recording
                        .canonical_store()?
                        .revision(revision)
                        .await?
                        .ok_or_else(|| {
                            native::WorkflowError::Input(
                                "canonical modeling revision absent".into(),
                            )
                        })?;
                    recording
                        .modeling_revision(revision, physical.inner.clone(), Default::default())
                        .await
                })
                .await
            }),
            || connection_stop.cancel(),
        )?;
        Ok(NativeModelingPackage::from_revision(
            self.owner.clone(),
            self.recording_runtime(py)?.execution_services(),
            inner,
        ))
    }
    /// Capture the project-scoped recorded context cut, independent of source sequence.
    pub(super) fn _host_project_context_cut(
        &self,
        py: Python<'_>,
        project: &str,
    ) -> PyResult<documents::DocumentValue<pse_operations::canonical_contexts::ContextCut>> {
        let stop = CancelSource::new();
        let value = blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                Ok(runtime
                    .canonical_store()?
                    .project_context_cut(project)
                    .await?)
            }),
            || stop.cancel(),
        )?;
        Ok(documents::DocumentValue(value))
    }
    /// Query original immutable context at an explicit UTC microsecond instant and cut.
    pub(super) fn _host_project_context_select(
        &self,
        py: Python<'_>,
        cut: documents::DocumentInput<pse_operations::canonical_contexts::ContextCut>,
        scope: &str,
        effective_at_us: i64,
    ) -> PyResult<
        Option<documents::DocumentValue<pse_operations::canonical_contexts::ContextSelection>>,
    > {
        let stop = CancelSource::new();
        let value = blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                Ok(runtime
                    .canonical_store()?
                    .select_project_context(&cut.0, scope, effective_at_us)
                    .await?)
            }),
            || stop.cancel(),
        )?;
        Ok(value.map(documents::DocumentValue))
    }
    /// Exact immutable metadata remains available for tombstones and hidden projects.
    pub(super) fn _host_project_context_record(
        &self,
        py: Python<'_>,
        key: &str,
    ) -> PyResult<
        Option<documents::DocumentValue<pse_operations::canonical_contexts::ProjectContext>>,
    > {
        let stop = CancelSource::new();
        let value = blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                Ok(runtime
                    .canonical_store()?
                    .project_context_record(key)
                    .await?)
            }),
            || stop.cancel(),
        )?;
        Ok(value.map(documents::DocumentValue))
    }
    /// Reopen original persisted source and physical closures without caller-supplied documents.
    pub(super) fn _host_project_context_reopen(
        &self,
        py: Python<'_>,
        selection: documents::DocumentInput<pse_operations::canonical_contexts::ContextSelection>,
    ) -> PyResult<NativeModelingPackage> {
        let stop = CancelSource::new();
        let cancelled = &stop;
        let value = blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                runtime
                    .project_context_reopen(&selection.0, cancelled)
                    .await
            }),
            || stop.cancel(),
        )?;
        Ok(NativeModelingPackage::from_revision(
            self.owner.clone(),
            self.recording_runtime(py)?.execution_services(),
            value,
        ))
    }
    pub(super) fn _host_project_closure(
        &self,
        py: Python<'_>,
        request: documents::DocumentInput<native::ProjectExportRequest>,
        physical: Option<&NativePhysicalContext>,
    ) -> PyResult<documents::DocumentValue<native::ProjectClosureExplanation>> {
        self.project_closure_native(py, request, physical)
    }
    pub(super) fn _host_project_export(
        &self,
        py: Python<'_>,
        destination: std::path::PathBuf,
        request: documents::DocumentInput<native::ProjectExportRequest>,
        physical: Option<&NativePhysicalContext>,
    ) -> PyResult<documents::DocumentValue<native::ProjectManifest>> {
        self.project_export_native(py, destination, request, physical)
    }
    pub(super) fn _host_project_import_issue(
        &self,
        py: Python<'_>,
        bundle: &NativeProjectBundle,
        title: &str,
        branch_name: &str,
    ) -> PyResult<documents::DocumentValue<native::ProjectImportIntent>> {
        self.project_import_issue_native(py, bundle, title, branch_name)
    }
    pub(super) fn _host_project_import(
        &self,
        py: Python<'_>,
        bundle: &NativeProjectBundle,
        intent: documents::DocumentInput<native::ProjectImportIntent>,
    ) -> PyResult<documents::DocumentValue<native::ProjectImportReport>> {
        self.project_import_native(py, bundle, intent)
    }
    pub(super) fn _host_project_import_status(
        &self,
        py: Python<'_>,
        intent: documents::DocumentInput<native::ProjectImportIntent>,
    ) -> PyResult<Option<documents::DocumentValue<native::ProjectImportReport>>> {
        self.project_import_status_native(py, intent)
    }
    pub(super) fn _host_project_import_recover(
        &self,
        py: Python<'_>,
        intent: documents::DocumentInput<native::ProjectImportIntent>,
        original: documents::DocumentInput<pse_operations::canonical_artifacts::Artifact>,
    ) -> PyResult<documents::DocumentValue<native::ProjectImportRecoveryReport>> {
        self.project_import_recover_native(py, intent, original)
    }
    pub(super) fn _host_project_import_open(
        &self,
        py: Python<'_>,
        intent: documents::DocumentInput<native::ProjectImportIntent>,
    ) -> PyResult<NativeModelingPackage> {
        self.project_import_open_native(py, intent)
    }
    pub(super) fn _host_lifecycle_select(
        &self,
        py: Python<'_>,
        target: documents::DocumentInput<pse_operations::canonical_lifecycle::LifecycleTarget>,
    ) -> PyResult<documents::DocumentValue<pse_operations::canonical_lifecycle::LifecycleSelection>>
    {
        self.lifecycle_select_native(py, target)
    }
    pub(super) fn _host_lifecycle_explain(
        &self,
        py: Python<'_>,
        selection: documents::DocumentInput<
            pse_operations::canonical_lifecycle::LifecycleSelection,
        >,
    ) -> PyResult<documents::DocumentValue<pse_operations::canonical_lifecycle::LifecycleExplanation>>
    {
        self.lifecycle_explain_native(py, selection)
    }
    pub(super) fn _host_lifecycle_release(
        &self,
        py: Python<'_>,
        selection: documents::DocumentInput<
            pse_operations::canonical_lifecycle::LifecycleSelection,
        >,
    ) -> PyResult<documents::DocumentValue<pse_operations::canonical_lifecycle::LifecycleRelease>>
    {
        self.lifecycle_release_native(py, selection)
    }
    pub(super) fn _host_research_dataset_create(
        &self,
        py: Python<'_>,
        request: documents::DocumentInput<native::ResearchDatasetRequest>,
    ) -> PyResult<documents::DocumentValue<pse_operations::canonical_research::ResearchDataset>>
    {
        self.research_dataset_create_native(py, request)
    }
    pub(super) fn _host_research_selection_create(
        &self,
        py: Python<'_>,
        request: documents::DocumentInput<native::ResearchSelectionRequest>,
    ) -> PyResult<documents::DocumentValue<pse_operations::canonical_research::ResearchSelection>>
    {
        self.research_selection_create_native(py, request)
    }
    pub(super) fn _host_research_read(
        &self,
        py: Python<'_>,
        request: documents::DocumentInput<native::ResearchReadRequest>,
    ) -> PyResult<documents::DocumentValue<native::ResearchReadResult>> {
        self.research_read_native(py, request)
    }
    pub(super) fn _host_research_dataset_open(
        &self,
        py: Python<'_>,
        key: &str,
        context: documents::DocumentInput<pse_operations::canonical_contexts::ContextSelection>,
    ) -> PyResult<NativeModelingPackage> {
        self.research_dataset_open_native(py, key, context)
    }
    pub(super) fn _host_research_selection_open(
        &self,
        py: Python<'_>,
        key: &str,
    ) -> PyResult<NativeModelingPackage> {
        self.research_selection_open_native(py, key)
    }
    pub(super) fn _host_research_publication_preview(
        &self,
        py: Python<'_>,
        request: documents::DocumentInput<native::ResearchPublicationRequest>,
    ) -> PyResult<documents::DocumentValue<native::ResearchPublicationPreview>> {
        self.research_publication_preview_native(py, request)
    }
    pub(super) fn _host_research_publish(
        &self,
        py: Python<'_>,
        intent: documents::DocumentInput<native::ResearchPublicationIntent>,
    ) -> PyResult<documents::DocumentValue<pse_operations::canonical_research::ResearchPublication>>
    {
        self.research_publish_native(py, intent)
    }
    pub(super) fn _host_research_publication_open(
        &self,
        py: Python<'_>,
        key: &str,
    ) -> PyResult<NativeModelingPackage> {
        self.research_publication_open_native(py, key)
    }
    /// Load the selected exact source through the normal source owner; no latest lookup.
    pub(super) fn _host_project_context_package(
        &self,
        py: Python<'_>,
        selection: documents::DocumentInput<pse_operations::canonical_contexts::ContextSelection>,
        physical: &NativePhysicalContext,
    ) -> PyResult<NativeModelingPackage> {
        let stop = CancelSource::new();
        let cancelled = &stop;
        let value = blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                runtime
                    .project_context_package(&selection.0, physical.inner.clone(), cancelled)
                    .await
            }),
            || stop.cancel(),
        )?;
        Ok(NativeModelingPackage::from_revision(
            self.owner.clone(),
            self.recording_runtime(py)?.execution_services(),
            value,
        ))
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "An exact historical context append carries source, interval, correction and original operation independently across the Python boundary"
    )]
    pub(super) fn _host_project_context_append(
        &self,
        py: Python<'_>,
        operation: &str,
        project: documents::DocumentInput<pse_operations::canonical_projects::Project>,
        scope: &str,
        source: &NativeModelingPackage,
        effective_from_us: i64,
        effective_to_us: Option<i64>,
        corrects: Option<&str>,
        deleted: bool,
    ) -> PyResult<documents::DocumentValue<pse_operations::canonical_contexts::ProjectContext>>
    {
        let stop = CancelSource::new();
        let value = blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                runtime
                    .project_record_context(
                        operation,
                        &project.0,
                        scope,
                        &source.inner,
                        effective_from_us,
                        effective_to_us,
                        corrects,
                        deleted,
                    )
                    .await
            }),
            || stop.cancel(),
        )?;
        Ok(documents::DocumentValue(value))
    }
    /// Create project organization using an original caller-issued command identity.
    pub(super) fn _host_project_create(
        &self,
        py: Python<'_>,
        operation: &str,
        project: &str,
        title: &str,
    ) -> PyResult<
        documents::DocumentValue<pse_operations::canonical_projects::ProjectAcknowledgement>,
    > {
        let stop = CancelSource::new();
        let value = blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                Ok(runtime
                    .canonical_store()?
                    .create_project(operation, project, title)
                    .await?)
            }),
            || stop.cancel(),
        )?;
        Ok(documents::DocumentValue(value))
    }
    /// Read one compact project record without hydrating scientific payloads.
    pub(super) fn _host_project_record(
        &self,
        py: Python<'_>,
        project: &str,
    ) -> PyResult<Option<documents::DocumentValue<pse_operations::canonical_projects::Project>>>
    {
        let stop = CancelSource::new();
        let value = blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                Ok(runtime.canonical_store()?.project(project).await?)
            }),
            || stop.cancel(),
        )?;
        Ok(value.map(documents::DocumentValue))
    }
    pub(super) fn _host_project_branch_record(
        &self,
        py: Python<'_>,
        branch: &str,
    ) -> PyResult<Option<documents::DocumentValue<pse_operations::canonical_projects::Branch>>>
    {
        let stop = CancelSource::new();
        let value = blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                Ok(runtime.canonical_store()?.project_branch(branch).await?)
            }),
            || stop.cancel(),
        )?;
        Ok(value.map(documents::DocumentValue))
    }
    pub(super) fn _host_project_branches(
        &self,
        py: Python<'_>,
        project: &str,
        cursor: Option<documents::DocumentInput<pse_operations::canonical_projects::ProjectCursor>>,
    ) -> PyResult<documents::DocumentValue<pse_operations::canonical_projects::BranchPage>> {
        let stop = CancelSource::new();
        let value = blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                Ok(runtime
                    .canonical_store()?
                    .project_branches(project, cursor.as_ref().map(|v| &v.0))
                    .await?)
            }),
            || stop.cancel(),
        )?;
        Ok(documents::DocumentValue(value))
    }
    pub(super) fn _host_project_fork(
        &self,
        py: Python<'_>,
        project: &str,
        branch: &str,
        problem: &str,
        operation: &str,
        source: &NativeModelingPackage,
    ) -> PyResult<String> {
        let stop = CancelSource::new();
        let value = blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                let project = runtime
                    .canonical_store()?
                    .project(project)
                    .await?
                    .ok_or_else(|| native::WorkflowError::Input("project unavailable".into()))?;
                runtime
                    .project_fork(&project, branch, problem, operation, &source.inner)
                    .await
            }),
            || stop.cancel(),
        )?;
        Ok(value.key)
    }
    pub(super) fn _host_project_publication_intent(
        &self,
        py: Python<'_>,
        project: &str,
        branch: &str,
        expected: &str,
        operation: &str,
        draft: Option<&str>,
    ) -> PyResult<documents::DocumentValue<pse_operations::canonical_projects::ProjectPublication>>
    {
        let stop = CancelSource::new();
        let value =
            blocking(
                py,
                &self.owner,
                self.recording_operation(&stop, |runtime| async move {
                    let store = runtime.canonical_store()?;
                    let project = store.project(project).await?.ok_or_else(|| {
                        native::WorkflowError::Input("project unavailable".into())
                    })?;
                    let branch = store
                        .project_branch(branch)
                        .await?
                        .ok_or_else(|| native::WorkflowError::Input("branch unavailable".into()))?;
                    let draft = match draft {
                        Some(key) => Some(store.project_draft(key).await?.ok_or_else(|| {
                            native::WorkflowError::Input("draft unavailable".into())
                        })?),
                        None => None,
                    };
                    Ok(store.project_publication_intent(
                        &project,
                        &branch,
                        expected,
                        draft.as_ref(),
                        operation,
                    )?)
                }),
                || stop.cancel(),
            )?;
        Ok(documents::DocumentValue(value))
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "The typed Python boundary forwards the original exact draft operation premises without changing its public call contract"
    )]
    pub(super) fn _host_project_draft_save_intent(
        &self,
        py: Python<'_>,
        operation: &str,
        project: &str,
        branch: &str,
        key: &str,
        base: &str,
        content: &[u8],
        expected: Option<u64>,
    ) -> PyResult<documents::DocumentValue<native::ProjectDraftSaveIntent>> {
        let stop = CancelSource::new();
        let value = blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                let store = runtime.canonical_store()?;
                let project = store
                    .project(project)
                    .await?
                    .ok_or_else(|| native::WorkflowError::Input("project unavailable".into()))?;
                let branch = store
                    .project_branch(branch)
                    .await?
                    .ok_or_else(|| native::WorkflowError::Input("branch unavailable".into()))?;
                runtime
                    .project_draft_save_intent(
                        operation, &project, &branch, key, base, expected, content,
                    )
                    .await
            }),
            || stop.cancel(),
        )?;
        Ok(documents::DocumentValue(value))
    }
    pub(super) fn _host_project_draft_save(
        &self,
        py: Python<'_>,
        intent: documents::DocumentInput<native::ProjectDraftSaveIntent>,
        content: &[u8],
    ) -> PyResult<
        documents::DocumentValue<pse_operations::canonical_projects::ProjectAcknowledgement>,
    > {
        let stop = CancelSource::new();
        let value = blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                runtime.project_save_draft_intent(&intent.0, content).await
            }),
            || stop.cancel(),
        )?;
        Ok(documents::DocumentValue(value))
    }
    pub(super) fn _host_project_draft_record(
        &self,
        py: Python<'_>,
        key: &str,
    ) -> PyResult<Option<documents::DocumentValue<pse_operations::canonical_projects::Draft>>> {
        let stop = CancelSource::new();
        let value = blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                Ok(runtime.canonical_store()?.project_draft(key).await?)
            }),
            || stop.cancel(),
        )?;
        Ok(value.map(documents::DocumentValue))
    }
    pub(super) fn _host_project_draft_bytes<'py>(
        &self,
        py: Python<'py>,
        key: &str,
    ) -> PyResult<Bound<'py, pyo3::types::PyBytes>> {
        let stop = CancelSource::new();
        let cancelled = &stop;
        let body = blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                runtime.project_draft_bytes(key, cancelled).await
            }),
            || stop.cancel(),
        )?;
        Ok(pyo3::types::PyBytes::new(py, &body))
    }
    pub(super) fn _host_project_select(
        &self,
        py: Python<'_>,
        request: documents::DocumentInput<native::ProjectSelectRequest>,
        physical: &NativePhysicalContext,
    ) -> PyResult<documents::DocumentValue<native::ProjectSelection>> {
        let stop = CancelSource::new();
        let cancelled = &stop;
        let value = blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                runtime
                    .project_select(&request.0, physical.inner.clone(), cancelled)
                    .await
            }),
            || stop.cancel(),
        )?;
        Ok(documents::DocumentValue(value))
    }
    pub(super) fn _host_project_open_selection(
        &self,
        py: Python<'_>,
        selection: documents::DocumentInput<native::ProjectSelection>,
        physical: &NativePhysicalContext,
    ) -> PyResult<NativeModelingPackage> {
        let stop = CancelSource::new();
        let cancelled = &stop;
        let value = blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                runtime
                    .project_open_selection(&selection.0, physical.inner.clone(), cancelled)
                    .await
            }),
            || stop.cancel(),
        )?;
        Ok(NativeModelingPackage::from_revision(
            self.owner.clone(),
            self.recording_runtime(py)?.execution_services(),
            value,
        ))
    }
    pub(super) fn _host_project_navigate(
        &self,
        py: Python<'_>,
        operation: &str,
        project: documents::DocumentInput<pse_operations::canonical_projects::Project>,
        request: documents::DocumentInput<
            pse_operations::canonical_projects::ProjectNavigationRequest,
        >,
    ) -> PyResult<
        documents::DocumentValue<pse_operations::canonical_projects::ProjectAcknowledgement>,
    > {
        let stop = CancelSource::new();
        let value = blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                Ok(runtime
                    .canonical_store()?
                    .project_navigate(operation, &project.0, &request.0.action)
                    .await?)
            }),
            || stop.cancel(),
        )?;
        Ok(documents::DocumentValue(value))
    }
    pub(super) fn _host_project_metadata(
        &self,
        py: Python<'_>,
        request: documents::DocumentInput<
            pse_operations::canonical_projects::ProjectMetadataRequest,
        >,
    ) -> PyResult<documents::DocumentValue<pse_operations::canonical_projects::ProjectMetadataPage>>
    {
        let stop = CancelSource::new();
        let value = blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                Ok(runtime
                    .canonical_store()?
                    .project_metadata(request.0.kind, &request.0.owner, request.0.cursor.as_ref())
                    .await?)
            }),
            || stop.cancel(),
        )?;
        Ok(documents::DocumentValue(value))
    }
    pub(super) fn _host_project_draft_version(
        &self,
        py: Python<'_>,
        key: &str,
    ) -> PyResult<Option<documents::DocumentValue<pse_operations::canonical_projects::DraftVersion>>>
    {
        let stop = CancelSource::new();
        let value = blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                Ok(runtime
                    .canonical_store()?
                    .project_draft_version(key)
                    .await?)
            }),
            || stop.cancel(),
        )?;
        Ok(value.map(documents::DocumentValue))
    }
    pub(super) fn _host_project_draft_version_bytes<'py>(
        &self,
        py: Python<'py>,
        key: &str,
    ) -> PyResult<Bound<'py, pyo3::types::PyBytes>> {
        let stop = CancelSource::new();
        let cancelled = &stop;
        let body = blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                runtime.project_draft_version_bytes(key, cancelled).await
            }),
            || stop.cancel(),
        )?;
        Ok(pyo3::types::PyBytes::new(py, &body))
    }
    pub(super) fn _host_project_draft_delta(
        &self,
        py: Python<'_>,
        source: &NativeModelingPackage,
        delta: documents::DocumentInput<native::SourceDelta>,
    ) -> PyResult<documents::DocumentValue<native::ProjectDraftDelta>> {
        let stop = CancelSource::new();
        let value = blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                runtime.project_draft_delta(&source.inner, delta.0)
            }),
            || stop.cancel(),
        )?;
        Ok(documents::DocumentValue(value))
    }
    pub(super) fn _host_project_admit_draft(
        &self,
        py: Python<'_>,
        operation: &str,
        key: &str,
        physical: &NativePhysicalContext,
    ) -> PyResult<NativeModelingPackage> {
        let stop = CancelSource::new();
        let token = &stop;
        let value = blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                runtime
                    .project_admit_draft(operation, key, physical.inner.clone(), token)
                    .await
            }),
            || stop.cancel(),
        )?;
        Ok(NativeModelingPackage::from_revision(
            self.owner.clone(),
            self.recording_runtime(py)?.execution_services(),
            value,
        ))
    }
    pub(super) fn _host_project_acknowledgement(
        &self,
        py: Python<'_>,
        operation: &str,
    ) -> PyResult<
        Option<
            documents::DocumentValue<pse_operations::canonical_projects::ProjectAcknowledgement>,
        >,
    > {
        let stop = CancelSource::new();
        let value = blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                Ok(runtime
                    .canonical_store()?
                    .project_acknowledgement(operation)
                    .await?)
            }),
            || stop.cancel(),
        )?;
        Ok(value.map(documents::DocumentValue))
    }
    pub(super) fn _host_project_publish_draft(
        &self,
        py: Python<'_>,
        intent: documents::DocumentInput<pse_operations::canonical_projects::ProjectPublication>,
        physical: &NativePhysicalContext,
    ) -> PyResult<NativeModelingPackage> {
        let stop = CancelSource::new();
        let publication_stop = &stop;
        let value = source_settlement::blocking_source(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                runtime
                    .project_publish_draft(&intent.0, physical.inner.clone(), publication_stop)
                    .await
            }),
            || stop.cancel(),
        )?;
        Ok(NativeModelingPackage::from_revision(
            self.owner.clone(),
            self.recording_runtime(py)?.execution_services(),
            value,
        ))
    }
    pub(super) fn _host_project_merge_preview(
        &self,
        py: Python<'_>,
        base: &NativeModelingPackage,
        target: &NativeModelingPackage,
        proposal: &NativeModelingPackage,
        resolutions: Vec<documents::DocumentInput<native::ProjectMergeResolution>>,
    ) -> PyResult<documents::DocumentValue<native::ProjectMergePreview>> {
        let stop = CancelSource::new();
        let resolutions = resolutions.into_iter().map(|r| r.0).collect::<Vec<_>>();
        let value = blocking(
            py,
            &self.owner,
            self.inner.project_merge_preview(
                &base.inner,
                &target.inner,
                &proposal.inner,
                &resolutions,
                &stop,
            ),
            || stop.cancel(),
        )?;
        Ok(documents::DocumentValue(value))
    }
    pub(super) fn _host_project_merge_intent(
        &self,
        py: Python<'_>,
        publication: documents::DocumentInput<
            pse_operations::canonical_projects::ProjectPublication,
        >,
        base: &NativeModelingPackage,
        target: &NativeModelingPackage,
        proposal: &NativeModelingPackage,
        resolutions: Vec<documents::DocumentInput<native::ProjectMergeResolution>>,
    ) -> PyResult<documents::DocumentValue<native::ProjectMergeIntent>> {
        let stop = CancelSource::new();
        let decision_stop = &stop;
        let resolutions = resolutions.into_iter().map(|r| r.0).collect::<Vec<_>>();
        let value = blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                runtime
                    .project_merge_intent(
                        publication.0,
                        &base.inner,
                        &target.inner,
                        &proposal.inner,
                        resolutions,
                        decision_stop,
                    )
                    .await
            }),
            || stop.cancel(),
        )?;
        Ok(documents::DocumentValue(value))
    }
    pub(super) fn _host_project_merge_draft_intent(
        &self,
        py: Python<'_>,
        publication: documents::DocumentInput<
            pse_operations::canonical_projects::ProjectPublication,
        >,
        base: &NativeModelingPackage,
        target: &NativeModelingPackage,
        version: documents::DocumentInput<pse_operations::canonical_projects::DraftVersion>,
        resolutions: Vec<documents::DocumentInput<native::ProjectMergeResolution>>,
    ) -> PyResult<documents::DocumentValue<native::ProjectMergeIntent>> {
        let stop = CancelSource::new();
        let decision_stop = &stop;
        let resolutions = resolutions.into_iter().map(|r| r.0).collect::<Vec<_>>();
        let value = blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                runtime
                    .project_merge_draft_intent(
                        publication.0,
                        &base.inner,
                        &target.inner,
                        &version.0,
                        resolutions,
                        decision_stop,
                    )
                    .await
            }),
            || stop.cancel(),
        )?;
        Ok(documents::DocumentValue(value))
    }
    pub(super) fn _host_project_undo_intent(
        &self,
        py: Python<'_>,
        publication: documents::DocumentInput<
            pse_operations::canonical_projects::ProjectPublication,
        >,
        target: &NativeModelingPackage,
        restore: &NativeModelingPackage,
    ) -> PyResult<documents::DocumentValue<native::ProjectMergeIntent>> {
        let stop = CancelSource::new();
        let decision_stop = &stop;
        let value = blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                runtime
                    .project_undo_intent(
                        publication.0,
                        &target.inner,
                        &restore.inner,
                        decision_stop,
                    )
                    .await
            }),
            || stop.cancel(),
        )?;
        Ok(documents::DocumentValue(value))
    }
    pub(super) fn _host_project_publish_merge(
        &self,
        py: Python<'_>,
        intent: documents::DocumentInput<native::ProjectMergeIntent>,
        physical: &NativePhysicalContext,
    ) -> PyResult<NativeModelingPackage> {
        let stop = CancelSource::new();
        let publication_stop = &stop;
        let value = source_settlement::blocking_source(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                runtime
                    .project_publish_merge(&intent.0, physical.inner.clone(), publication_stop)
                    .await
            }),
            || stop.cancel(),
        )?;
        Ok(NativeModelingPackage::from_revision(
            self.owner.clone(),
            self.recording_runtime(py)?.execution_services(),
            value,
        ))
    }
}
