// @generated from the Rust project operation owner and typed helpers; do not edit
#![rustfmt::skip]
#![allow(
    clippy::all,
    reason = "mechanical public PyO3 projection over typed receiver glue"
)]
use super::*;
#[pymethods]
impl NativeRuntime {
    /// Browse bounded compact metadata, including terminal studies.
    fn study_workbench_page(
        &self,
        py: Python<'_>,
        request: &[u8],
    ) -> PyResult<documents::DocumentValue<native::StudyWorkbenchPage>> {
        self._host_study_workbench_page(py, request)
    }
    /// Freeze a complete bounded cohort before protected analytical reads.
    fn freeze_study_cohort(
        &self,
        py: Python<'_>,
        request: &[u8],
    ) -> PyResult<NativeFrozenStudyCohort> {
        self._host_freeze_study_cohort(py, request)
    }
    /// Open or reconnect passive observation of one exact selected study.
    #[pyo3(signature = (request, *, cursor = None))]
    fn observe_study(
        &self,
        py: Python<'_>,
        request: &[u8],
        cursor: Option<&[u8]>,
    ) -> PyResult<NativeStudyObserver> {
        self._host_observe_study(py, request, cursor)
    }
    /// Reopen an exact retained canonical source selection without authored document replay.
    fn modeling_revision(
        &self,
        py: Python<'_>,
        revision: &str,
        physical: &NativePhysicalContext,
    ) -> PyResult<NativeModelingPackage> {
        self._host_modeling_revision(py, revision, physical)
    }
    /// Capture the project-scoped recorded context cut, independent of source sequence.
    fn project_context_cut(
        &self,
        py: Python<'_>,
        project: &str,
    ) -> PyResult<
        documents::DocumentValue<pse_operations::canonical_contexts::ContextCut>,
    > {
        self._host_project_context_cut(py, project)
    }
    /// Query original immutable context at an explicit UTC microsecond instant and cut.
    fn project_context_select(
        &self,
        py: Python<'_>,
        cut: documents::DocumentInput<pse_operations::canonical_contexts::ContextCut>,
        scope: &str,
        effective_at_us: i64,
    ) -> PyResult<
        Option<
            documents::DocumentValue<
                pse_operations::canonical_contexts::ContextSelection,
            >,
        >,
    > {
        self._host_project_context_select(py, cut, scope, effective_at_us)
    }
    /// Exact immutable metadata remains available for tombstones and hidden projects.
    fn project_context_record(
        &self,
        py: Python<'_>,
        key: &str,
    ) -> PyResult<
        Option<
            documents::DocumentValue<pse_operations::canonical_contexts::ProjectContext>,
        >,
    > {
        self._host_project_context_record(py, key)
    }
    /// Reopen original persisted source and physical closures without caller-supplied documents.
    fn project_context_reopen(
        &self,
        py: Python<'_>,
        selection: documents::DocumentInput<
            pse_operations::canonical_contexts::ContextSelection,
        >,
    ) -> PyResult<NativeModelingPackage> {
        self._host_project_context_reopen(py, selection)
    }
    #[pyo3(signature = (request, physical = None))]
    fn project_closure(
        &self,
        py: Python<'_>,
        request: documents::DocumentInput<native::ProjectExportRequest>,
        physical: Option<&NativePhysicalContext>,
    ) -> PyResult<documents::DocumentValue<native::ProjectClosureExplanation>> {
        self._host_project_closure(py, request, physical)
    }
    #[pyo3(signature = (destination, request, physical = None))]
    fn project_export(
        &self,
        py: Python<'_>,
        destination: std::path::PathBuf,
        request: documents::DocumentInput<native::ProjectExportRequest>,
        physical: Option<&NativePhysicalContext>,
    ) -> PyResult<documents::DocumentValue<native::ProjectManifest>> {
        self._host_project_export(py, destination, request, physical)
    }
    #[pyo3(
        signature = (
            bundle,
            *,
            title,
            branch_name = pse_runtime::host_operations::project::DEFAULT_BRANCH
        )
    )]
    fn project_import_issue(
        &self,
        py: Python<'_>,
        bundle: &NativeProjectBundle,
        title: &str,
        branch_name: &str,
    ) -> PyResult<documents::DocumentValue<native::ProjectImportIntent>> {
        self._host_project_import_issue(py, bundle, title, branch_name)
    }
    fn project_import(
        &self,
        py: Python<'_>,
        bundle: &NativeProjectBundle,
        intent: documents::DocumentInput<native::ProjectImportIntent>,
    ) -> PyResult<documents::DocumentValue<native::ProjectImportReport>> {
        self._host_project_import(py, bundle, intent)
    }
    fn project_import_status(
        &self,
        py: Python<'_>,
        intent: documents::DocumentInput<native::ProjectImportIntent>,
    ) -> PyResult<Option<documents::DocumentValue<native::ProjectImportReport>>> {
        self._host_project_import_status(py, intent)
    }
    fn project_import_recover(
        &self,
        py: Python<'_>,
        intent: documents::DocumentInput<native::ProjectImportIntent>,
        original: documents::DocumentInput<pse_operations::canonical_artifacts::Artifact>,
    ) -> PyResult<documents::DocumentValue<native::ProjectImportRecoveryReport>> {
        self._host_project_import_recover(py, intent, original)
    }
    fn project_import_open(
        &self,
        py: Python<'_>,
        intent: documents::DocumentInput<native::ProjectImportIntent>,
    ) -> PyResult<NativeModelingPackage> {
        self._host_project_import_open(py, intent)
    }
    fn lifecycle_select(
        &self,
        py: Python<'_>,
        target: documents::DocumentInput<
            pse_operations::canonical_lifecycle::LifecycleTarget,
        >,
    ) -> PyResult<
        documents::DocumentValue<pse_operations::canonical_lifecycle::LifecycleSelection>,
    > {
        self._host_lifecycle_select(py, target)
    }
    fn lifecycle_explain(
        &self,
        py: Python<'_>,
        selection: documents::DocumentInput<
            pse_operations::canonical_lifecycle::LifecycleSelection,
        >,
    ) -> PyResult<
        documents::DocumentValue<
            pse_operations::canonical_lifecycle::LifecycleExplanation,
        >,
    > {
        self._host_lifecycle_explain(py, selection)
    }
    fn lifecycle_release(
        &self,
        py: Python<'_>,
        selection: documents::DocumentInput<
            pse_operations::canonical_lifecycle::LifecycleSelection,
        >,
    ) -> PyResult<
        documents::DocumentValue<pse_operations::canonical_lifecycle::LifecycleRelease>,
    > {
        self._host_lifecycle_release(py, selection)
    }
    fn research_dataset_create(
        &self,
        py: Python<'_>,
        request: documents::DocumentInput<native::ResearchDatasetRequest>,
    ) -> PyResult<
        documents::DocumentValue<pse_operations::canonical_research::ResearchDataset>,
    > {
        self._host_research_dataset_create(py, request)
    }
    fn research_selection_create(
        &self,
        py: Python<'_>,
        request: documents::DocumentInput<native::ResearchSelectionRequest>,
    ) -> PyResult<
        documents::DocumentValue<pse_operations::canonical_research::ResearchSelection>,
    > {
        self._host_research_selection_create(py, request)
    }
    fn research_read(
        &self,
        py: Python<'_>,
        request: documents::DocumentInput<native::ResearchReadRequest>,
    ) -> PyResult<documents::DocumentValue<native::ResearchReadResult>> {
        self._host_research_read(py, request)
    }
    fn research_dataset_open(
        &self,
        py: Python<'_>,
        key: &str,
        context: documents::DocumentInput<
            pse_operations::canonical_contexts::ContextSelection,
        >,
    ) -> PyResult<NativeModelingPackage> {
        self._host_research_dataset_open(py, key, context)
    }
    fn research_selection_open(
        &self,
        py: Python<'_>,
        key: &str,
    ) -> PyResult<NativeModelingPackage> {
        self._host_research_selection_open(py, key)
    }
    fn research_publication_preview(
        &self,
        py: Python<'_>,
        request: documents::DocumentInput<native::ResearchPublicationRequest>,
    ) -> PyResult<documents::DocumentValue<native::ResearchPublicationPreview>> {
        self._host_research_publication_preview(py, request)
    }
    fn research_publish(
        &self,
        py: Python<'_>,
        intent: documents::DocumentInput<native::ResearchPublicationIntent>,
    ) -> PyResult<
        documents::DocumentValue<pse_operations::canonical_research::ResearchPublication>,
    > {
        self._host_research_publish(py, intent)
    }
    fn research_publication_open(
        &self,
        py: Python<'_>,
        key: &str,
    ) -> PyResult<NativeModelingPackage> {
        self._host_research_publication_open(py, key)
    }
    /// Load the selected exact source through the normal source owner; no latest lookup.
    fn project_context_package(
        &self,
        py: Python<'_>,
        selection: documents::DocumentInput<
            pse_operations::canonical_contexts::ContextSelection,
        >,
        physical: &NativePhysicalContext,
    ) -> PyResult<NativeModelingPackage> {
        self._host_project_context_package(py, selection, physical)
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "An exact historical context append carries source, interval, correction and original operation independently across the Python boundary"
    )]
    #[pyo3(
        signature = (
            operation,
            project,
            scope,
            source,
            effective_from_us,
            *,
            effective_to_us = None,
            corrects = None,
            deleted = false
        )
    )]
    fn project_context_append(
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
    ) -> PyResult<
        documents::DocumentValue<pse_operations::canonical_contexts::ProjectContext>,
    > {
        self._host_project_context_append(
            py,
            operation,
            project,
            scope,
            source,
            effective_from_us,
            effective_to_us,
            corrects,
            deleted,
        )
    }
    /// Create project organization using an original caller-issued command identity.
    fn project_create(
        &self,
        py: Python<'_>,
        operation: &str,
        project: &str,
        title: &str,
    ) -> PyResult<
        documents::DocumentValue<
            pse_operations::canonical_projects::ProjectAcknowledgement,
        >,
    > {
        self._host_project_create(py, operation, project, title)
    }
    /// Read one compact project record without hydrating scientific payloads.
    fn project_record(
        &self,
        py: Python<'_>,
        project: &str,
    ) -> PyResult<
        Option<documents::DocumentValue<pse_operations::canonical_projects::Project>>,
    > {
        self._host_project_record(py, project)
    }
    fn project_branch_record(
        &self,
        py: Python<'_>,
        branch: &str,
    ) -> PyResult<
        Option<documents::DocumentValue<pse_operations::canonical_projects::Branch>>,
    > {
        self._host_project_branch_record(py, branch)
    }
    #[pyo3(signature = (project, *, cursor = None))]
    fn project_branches(
        &self,
        py: Python<'_>,
        project: &str,
        cursor: Option<
            documents::DocumentInput<pse_operations::canonical_projects::ProjectCursor>,
        >,
    ) -> PyResult<
        documents::DocumentValue<pse_operations::canonical_projects::BranchPage>,
    > {
        self._host_project_branches(py, project, cursor)
    }
    fn project_fork(
        &self,
        py: Python<'_>,
        project: &str,
        branch: &str,
        problem: &str,
        operation: &str,
        source: &NativeModelingPackage,
    ) -> PyResult<String> {
        self._host_project_fork(py, project, branch, problem, operation, source)
    }
    #[pyo3(signature = (project, branch, expected, operation, *, draft = None))]
    fn project_publication_intent(
        &self,
        py: Python<'_>,
        project: &str,
        branch: &str,
        expected: &str,
        operation: &str,
        draft: Option<&str>,
    ) -> PyResult<
        documents::DocumentValue<pse_operations::canonical_projects::ProjectPublication>,
    > {
        self._host_project_publication_intent(
            py,
            project,
            branch,
            expected,
            operation,
            draft,
        )
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "The typed Python boundary forwards the original exact draft operation premises without changing its public call contract"
    )]
    #[pyo3(
        signature = (operation, project, branch, key, base, content, *, expected = None)
    )]
    fn project_draft_save_intent(
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
        self._host_project_draft_save_intent(
            py,
            operation,
            project,
            branch,
            key,
            base,
            content,
            expected,
        )
    }
    fn project_draft_save(
        &self,
        py: Python<'_>,
        intent: documents::DocumentInput<native::ProjectDraftSaveIntent>,
        content: &[u8],
    ) -> PyResult<
        documents::DocumentValue<
            pse_operations::canonical_projects::ProjectAcknowledgement,
        >,
    > {
        self._host_project_draft_save(py, intent, content)
    }
    fn project_draft_record(
        &self,
        py: Python<'_>,
        key: &str,
    ) -> PyResult<
        Option<documents::DocumentValue<pse_operations::canonical_projects::Draft>>,
    > {
        self._host_project_draft_record(py, key)
    }
    fn project_draft_bytes<'py>(
        &self,
        py: Python<'py>,
        key: &str,
    ) -> PyResult<Bound<'py, pyo3::types::PyBytes>> {
        self._host_project_draft_bytes(py, key)
    }
    fn project_select(
        &self,
        py: Python<'_>,
        request: documents::DocumentInput<native::ProjectSelectRequest>,
        physical: &NativePhysicalContext,
    ) -> PyResult<documents::DocumentValue<native::ProjectSelection>> {
        self._host_project_select(py, request, physical)
    }
    fn project_open_selection(
        &self,
        py: Python<'_>,
        selection: documents::DocumentInput<native::ProjectSelection>,
        physical: &NativePhysicalContext,
    ) -> PyResult<NativeModelingPackage> {
        self._host_project_open_selection(py, selection, physical)
    }
    fn project_navigate(
        &self,
        py: Python<'_>,
        operation: &str,
        project: documents::DocumentInput<pse_operations::canonical_projects::Project>,
        request: documents::DocumentInput<
            pse_operations::canonical_projects::ProjectNavigationRequest,
        >,
    ) -> PyResult<
        documents::DocumentValue<
            pse_operations::canonical_projects::ProjectAcknowledgement,
        >,
    > {
        self._host_project_navigate(py, operation, project, request)
    }
    fn project_metadata(
        &self,
        py: Python<'_>,
        request: documents::DocumentInput<
            pse_operations::canonical_projects::ProjectMetadataRequest,
        >,
    ) -> PyResult<
        documents::DocumentValue<pse_operations::canonical_projects::ProjectMetadataPage>,
    > {
        self._host_project_metadata(py, request)
    }
    fn project_draft_version(
        &self,
        py: Python<'_>,
        key: &str,
    ) -> PyResult<
        Option<
            documents::DocumentValue<pse_operations::canonical_projects::DraftVersion>,
        >,
    > {
        self._host_project_draft_version(py, key)
    }
    fn project_draft_version_bytes<'py>(
        &self,
        py: Python<'py>,
        key: &str,
    ) -> PyResult<Bound<'py, pyo3::types::PyBytes>> {
        self._host_project_draft_version_bytes(py, key)
    }
    fn project_draft_delta(
        &self,
        py: Python<'_>,
        source: &NativeModelingPackage,
        delta: documents::DocumentInput<native::SourceDelta>,
    ) -> PyResult<documents::DocumentValue<native::ProjectDraftDelta>> {
        self._host_project_draft_delta(py, source, delta)
    }
    fn project_admit_draft(
        &self,
        py: Python<'_>,
        operation: &str,
        key: &str,
        physical: &NativePhysicalContext,
    ) -> PyResult<NativeModelingPackage> {
        self._host_project_admit_draft(py, operation, key, physical)
    }
    fn project_acknowledgement(
        &self,
        py: Python<'_>,
        operation: &str,
    ) -> PyResult<
        Option<
            documents::DocumentValue<
                pse_operations::canonical_projects::ProjectAcknowledgement,
            >,
        >,
    > {
        self._host_project_acknowledgement(py, operation)
    }
    fn project_publish_draft(
        &self,
        py: Python<'_>,
        intent: documents::DocumentInput<
            pse_operations::canonical_projects::ProjectPublication,
        >,
        physical: &NativePhysicalContext,
    ) -> PyResult<NativeModelingPackage> {
        self._host_project_publish_draft(py, intent, physical)
    }
    fn project_merge_preview(
        &self,
        py: Python<'_>,
        base: &NativeModelingPackage,
        target: &NativeModelingPackage,
        proposal: &NativeModelingPackage,
        resolutions: Vec<documents::DocumentInput<native::ProjectMergeResolution>>,
    ) -> PyResult<documents::DocumentValue<native::ProjectMergePreview>> {
        self._host_project_merge_preview(py, base, target, proposal, resolutions)
    }
    fn project_merge_intent(
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
        self._host_project_merge_intent(
            py,
            publication,
            base,
            target,
            proposal,
            resolutions,
        )
    }
    fn project_merge_draft_intent(
        &self,
        py: Python<'_>,
        publication: documents::DocumentInput<
            pse_operations::canonical_projects::ProjectPublication,
        >,
        base: &NativeModelingPackage,
        target: &NativeModelingPackage,
        version: documents::DocumentInput<
            pse_operations::canonical_projects::DraftVersion,
        >,
        resolutions: Vec<documents::DocumentInput<native::ProjectMergeResolution>>,
    ) -> PyResult<documents::DocumentValue<native::ProjectMergeIntent>> {
        self._host_project_merge_draft_intent(
            py,
            publication,
            base,
            target,
            version,
            resolutions,
        )
    }
    fn project_undo_intent(
        &self,
        py: Python<'_>,
        publication: documents::DocumentInput<
            pse_operations::canonical_projects::ProjectPublication,
        >,
        target: &NativeModelingPackage,
        restore: &NativeModelingPackage,
    ) -> PyResult<documents::DocumentValue<native::ProjectMergeIntent>> {
        self._host_project_undo_intent(py, publication, target, restore)
    }
    fn project_publish_merge(
        &self,
        py: Python<'_>,
        intent: documents::DocumentInput<native::ProjectMergeIntent>,
        physical: &NativePhysicalContext,
    ) -> PyResult<NativeModelingPackage> {
        self._host_project_publish_merge(py, intent, physical)
    }
}
