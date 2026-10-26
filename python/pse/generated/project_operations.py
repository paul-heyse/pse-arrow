# @generated from Rust project operations and typed helper interfaces.
# Do not edit.
"""Public project operations projected from their Rust owner and typed receiver glue."""

from collections.abc import Sequence
from pathlib import Path
from types import FunctionType

from pse._modeling import ModelingPackage
from pse._physical import PhysicalContext
from pse._project_bundle import ProjectBundle
from pse._project_operations_glue import ProjectRuntimeGlue
from pse._research import ResearchRuntimeGlue
from pse._study_workbench import FrozenStudyCohort, StudyObserver, StudyWorkbenchGlue
from pse.contracts.documents import (
    ArtifactIntent,
    ArtifactIntentBody,
    ContextCut,
    ContextSelection,
    ContextSelectionOutput,
    LifecycleExplanationOutput,
    LifecycleReleaseOutput,
    LifecycleSelection,
    LifecycleSelectionOutput,
    LifecycleTarget,
    ProjectAcknowledgementOutput,
    ProjectBranchPageOutput,
    ProjectBranchRecordOutput,
    ProjectClosureExplanationOutput,
    ProjectContextOutput,
    ProjectCursor,
    ProjectCursorBody,
    ProjectDraftDeltaOutput,
    ProjectDraftRecordOutput,
    ProjectDraftSaveIntent,
    ProjectDraftSaveIntentOutput,
    ProjectDraftVersion,
    ProjectDraftVersionOutput,
    ProjectExportRequest,
    ProjectImportIntent,
    ProjectImportIntentOutput,
    ProjectImportRecoveryReportOutput,
    ProjectImportReportOutput,
    ProjectManifestOutput,
    ProjectMergeIntent,
    ProjectMergeIntentOutput,
    ProjectMergePreview,
    ProjectMergeResolution,
    ProjectMetadataPageOutput,
    ProjectMetadataRequest,
    ProjectMetadataRequestOutput,
    ProjectNavigationRequest,
    ProjectPublication,
    ProjectPublicationOutput,
    ProjectRecord,
    ProjectSelection,
    ProjectSelectRequest,
    ResearchDatasetOutput,
    ResearchDatasetRequest,
    ResearchPublication,
    ResearchPublicationIntent,
    ResearchPublicationIntentOutput,
    ResearchPublicationPreviewOutput,
    ResearchPublicationRequest,
    ResearchReadRequest,
    ResearchReadResultOutput,
    ResearchSelection,
    ResearchSelectionRequest,
    SourceDelta,
    SourceDeltaOutput,
    StudyCohortRequest,
    StudyObservationCursor,
    StudyObservationCursorOutput,
    StudyObservationRequest,
    StudyWorkbenchPageOutput,
    StudyWorkbenchRequest,
)
from pse.generated.project_operation_defaults import DEFAULT_BRANCH


def _public_metadata(
    public: object, glue: type[object], helper: str, owner: str, module: str
) -> None:
    original = vars(glue)[helper]
    if not isinstance(public, FunctionType) or not isinstance(original, FunctionType):
        raise TypeError((helper, type(public), type(original)))
    public.__qualname__ = owner + "." + public.__name__
    public.__module__ = module
    public.__annotations__ = dict(original.__annotations__)


class ProjectRuntimeOperations(ProjectRuntimeGlue):
    """One native owner supplies each generated public operation."""

    def modeling_revision(
        self, revision: str, physical: PhysicalContext
    ) -> ModelingPackage:
        """Reopen a retained source revision from canonical storage.

        Effects:
            scope: source_admission
            read_only: false
            idempotent: true
            destructive: false
        """
        return self._host_modeling_revision(revision, physical)

    _public_metadata(
        modeling_revision,
        ProjectRuntimeGlue,
        "_host_modeling_revision",
        "Runtime",
        "pse._workflow",
    )

    def project_context_cut(self, project: str) -> ContextCut:
        """Capture recorded context independently of the source sequence.

        Effects:
            scope: observation
            read_only: true
            idempotent: true
            destructive: false
        """
        return self._host_project_context_cut(project)

    _public_metadata(
        project_context_cut,
        ProjectRuntimeGlue,
        "_host_project_context_cut",
        "Runtime",
        "pse._workflow",
    )

    def project_context_select(
        self, cut: ContextCut, scope: str, effective_at_us: int
    ) -> ContextSelectionOutput | None:
        """Select at an explicit UTC microsecond instant and original recorded cut.

        Effects:
            scope: observation
            read_only: true
            idempotent: true
            destructive: false
        """
        return self._host_project_context_select(cut, scope, effective_at_us)

    _public_metadata(
        project_context_select,
        ProjectRuntimeGlue,
        "_host_project_context_select",
        "Runtime",
        "pse._workflow",
    )

    def project_context_record(self, key: str) -> ProjectContextOutput | None:
        """Read immutable metadata, including hidden or deleted records.

        Effects:
            scope: observation
            read_only: true
            idempotent: true
            destructive: false
        """
        return self._host_project_context_record(key)

    _public_metadata(
        project_context_record,
        ProjectRuntimeGlue,
        "_host_project_context_record",
        "Runtime",
        "pse._workflow",
    )

    def project_context_reopen(
        self, selection: ContextSelection | ContextSelectionOutput
    ) -> ModelingPackage:
        """Reopen the original persisted source and physical closure.

        Effects:
            scope: source_admission
            read_only: false
            idempotent: true
            destructive: false
        """
        return self._host_project_context_reopen(selection)

    _public_metadata(
        project_context_reopen,
        ProjectRuntimeGlue,
        "_host_project_context_reopen",
        "Runtime",
        "pse._workflow",
    )

    def project_closure(
        self, request: ProjectExportRequest, physical: PhysicalContext | None = None
    ) -> ProjectClosureExplanationOutput:
        """Explain exact selected membership through the original owners.

        Effects:
            scope: observation
            read_only: true
            idempotent: true
            destructive: false
        """
        return self._host_project_closure(request, physical)

    _public_metadata(
        project_closure,
        ProjectRuntimeGlue,
        "_host_project_closure",
        "Runtime",
        "pse._workflow",
    )

    def project_export(
        self,
        destination: Path,
        request: ProjectExportRequest,
        physical: PhysicalContext | None = None,
    ) -> ProjectManifestOutput:
        """Copy a protected selected closure for offline inspection and fresh import.

        Effects:
            scope: file_export
            read_only: false
            idempotent: false
            destructive: false
        """
        return self._host_project_export(destination, request, physical)

    _public_metadata(
        project_export,
        ProjectRuntimeGlue,
        "_host_project_export",
        "Runtime",
        "pse._workflow",
    )

    def project_import_issue(
        self, bundle: ProjectBundle, *, title: str, branch_name: str = DEFAULT_BRANCH
    ) -> ProjectImportIntentOutput:
        """Issue a retryable intent bound to this receiving authority and exact pack.

        Effects:
            scope: canonical_command
            read_only: false
            idempotent: true
            destructive: false
        """
        return self._host_project_import_issue(
            bundle, title=title, branch_name=branch_name
        )

    _public_metadata(
        project_import_issue,
        ProjectRuntimeGlue,
        "_host_project_import_issue",
        "Runtime",
        "pse._workflow",
    )

    def project_import(
        self,
        bundle: ProjectBundle,
        intent: ProjectImportIntent | ProjectImportIntentOutput,
    ) -> ProjectImportReportOutput:
        """Admit fresh source and copied evidence before project availability.

        Effects:
            scope: canonical_command
            read_only: false
            idempotent: true
            destructive: false
        """
        return self._host_project_import(bundle, intent)

    _public_metadata(
        project_import,
        ProjectRuntimeGlue,
        "_host_project_import",
        "Runtime",
        "pse._workflow",
    )

    def project_import_status(
        self, intent: ProjectImportIntent | ProjectImportIntentOutput
    ) -> ProjectImportReportOutput | None:
        """Distinguish admitted source from complete copied project custody.

        Effects:
            scope: observation
            read_only: true
            idempotent: true
            destructive: false
        """
        return self._host_project_import_status(intent)

    _public_metadata(
        project_import_status,
        ProjectRuntimeGlue,
        "_host_project_import_status",
        "Runtime",
        "pse._workflow",
    )

    def project_import_recover(
        self,
        intent: ProjectImportIntent | ProjectImportIntentOutput,
        original: ArtifactIntent | ArtifactIntentBody,
    ) -> ProjectImportRecoveryReportOutput:
        """Settle the original occurrence without renewing its clock.

        Effects:
            scope: canonical_command
            read_only: false
            idempotent: true
            destructive: false
        """
        return self._host_project_import_recover(intent, original)

    _public_metadata(
        project_import_recover,
        ProjectRuntimeGlue,
        "_host_project_import_recover",
        "Runtime",
        "pse._workflow",
    )

    def project_import_open(
        self, intent: ProjectImportIntent | ProjectImportIntentOutput
    ) -> ModelingPackage:
        """Reopen owned imported science without the originally supplied pack.

        Effects:
            scope: observation
            read_only: true
            idempotent: true
            destructive: false
        """
        return self._host_project_import_open(intent)

    _public_metadata(
        project_import_open,
        ProjectRuntimeGlue,
        "_host_project_import_open",
        "Runtime",
        "pse._workflow",
    )

    def lifecycle_select(self, target: LifecycleTarget) -> LifecycleSelectionOutput:
        """Freeze original issuing authority and current owner generations.

        Effects:
            scope: observation
            read_only: true
            idempotent: true
            destructive: false
        """
        return self._host_lifecycle_select(target)

    _public_metadata(
        lifecycle_select,
        ProjectRuntimeGlue,
        "_host_lifecycle_select",
        "Runtime",
        "pse._workflow",
    )

    def lifecycle_explain(
        self, selection: LifecycleSelection | LifecycleSelectionOutput
    ) -> LifecycleExplanationOutput:
        """Observe references and known bytes without granting disposal authority.

        Effects:
            scope: observation
            read_only: true
            idempotent: true
            destructive: false
        """
        return self._host_lifecycle_explain(selection)

    _public_metadata(
        lifecycle_explain,
        ProjectRuntimeGlue,
        "_host_lifecycle_explain",
        "Runtime",
        "pse._workflow",
    )

    def lifecycle_release(
        self, selection: LifecycleSelection | LifecycleSelectionOutput
    ) -> LifecycleReleaseOutput:
        """Recheck and dispatch release to its original owner.

        Effects:
            scope: retention_release
            read_only: false
            idempotent: false
            destructive: true
        """
        return self._host_lifecycle_release(selection)

    _public_metadata(
        lifecycle_release,
        ProjectRuntimeGlue,
        "_host_lifecycle_release",
        "Runtime",
        "pse._workflow",
    )

    def project_context_package(
        self,
        selection: ContextSelection | ContextSelectionOutput,
        physical: PhysicalContext,
    ) -> ModelingPackage:
        """Admit this exact source selection without selecting latest.

        Effects:
            scope: source_admission
            read_only: false
            idempotent: true
            destructive: false
        """
        return self._host_project_context_package(selection, physical)

    _public_metadata(
        project_context_package,
        ProjectRuntimeGlue,
        "_host_project_context_package",
        "Runtime",
        "pse._workflow",
    )

    def project_context_append(
        self,
        operation: str,
        project: ProjectRecord,
        scope: str,
        source: ModelingPackage,
        effective_from_us: int,
        *,
        effective_to_us: int | None = None,
        corrects: str | None = None,
        deleted: bool = False,
    ) -> ProjectContextOutput:
        """Append an exact source, effective interval and correction ancestry.

        Retain the original project descriptor and operation through uncertain
        completion. Scientific values remain in the admitted source, not this context
        record.

        Effects:
            scope: canonical_command
            read_only: false
            idempotent: true
            destructive: false
        """
        return self._host_project_context_append(
            operation,
            project,
            scope,
            source,
            effective_from_us,
            effective_to_us=effective_to_us,
            corrects=corrects,
            deleted=deleted,
        )

    _public_metadata(
        project_context_append,
        ProjectRuntimeGlue,
        "_host_project_context_append",
        "Runtime",
        "pse._workflow",
    )

    def project_create(
        self, operation: str, project: str, title: str
    ) -> ProjectAcknowledgementOutput:
        """Create project organization using an original caller-issued operation.

        Effects:
            scope: canonical_command
            read_only: false
            idempotent: true
            destructive: false
        """
        return self._host_project_create(operation, project, title)

    _public_metadata(
        project_create,
        ProjectRuntimeGlue,
        "_host_project_create",
        "Runtime",
        "pse._workflow",
    )

    def project_record(self, project: str) -> ProjectRecord | None:
        """Read project metadata without hydrating scientific payloads.

        Effects:
            scope: observation
            read_only: true
            idempotent: true
            destructive: false
        """
        return self._host_project_record(project)

    _public_metadata(
        project_record,
        ProjectRuntimeGlue,
        "_host_project_record",
        "Runtime",
        "pse._workflow",
    )

    def project_branch_record(self, branch: str) -> ProjectBranchRecordOutput | None:
        """Read branch ownership; its source problem owns the single head.

        Effects:
            scope: observation
            read_only: true
            idempotent: true
            destructive: false
        """
        return self._host_project_branch_record(branch)

    _public_metadata(
        project_branch_record,
        ProjectRuntimeGlue,
        "_host_project_branch_record",
        "Runtime",
        "pse._workflow",
    )

    def project_branches(
        self, project: str, *, cursor: ProjectCursor | ProjectCursorBody | None = None
    ) -> ProjectBranchPageOutput:
        """Read one ordered page with an authority/generation-fenced continuation.

        Effects:
            scope: observation
            read_only: true
            idempotent: true
            destructive: false
        """
        return self._host_project_branches(project, cursor=cursor)

    _public_metadata(
        project_branches,
        ProjectRuntimeGlue,
        "_host_project_branches",
        "Runtime",
        "pse._workflow",
    )

    def project_fork(
        self,
        project: str,
        branch: str,
        problem: str,
        operation: str,
        source: ModelingPackage,
    ) -> str:
        """Fork an exact admitted source closure into an independent writable branch.

        Effects:
            scope: canonical_command
            read_only: false
            idempotent: true
            destructive: false
        """
        return self._host_project_fork(project, branch, problem, operation, source)

    _public_metadata(
        project_fork,
        ProjectRuntimeGlue,
        "_host_project_fork",
        "Runtime",
        "pse._workflow",
    )

    def project_publication_intent(
        self,
        project: str,
        branch: str,
        expected: str,
        operation: str,
        *,
        draft: str | None = None,
    ) -> ProjectPublicationOutput:
        """Issue the original exact source/draft CAS; retain it through uncertainty.

        Effects:
            scope: canonical_command
            read_only: false
            idempotent: true
            destructive: false
        """
        return self._host_project_publication_intent(
            project, branch, expected, operation, draft=draft
        )

    _public_metadata(
        project_publication_intent,
        ProjectRuntimeGlue,
        "_host_project_publication_intent",
        "Runtime",
        "pse._workflow",
    )

    def project_draft_save_intent(
        self,
        operation: str,
        project: str,
        branch: str,
        key: str,
        base: str,
        content: bytes,
        *,
        expected: int | None = None,
    ) -> ProjectDraftSaveIntentOutput:
        """Read-only issue of raw custody; content may be malformed or empty.

        Keep the original intent until artifact and project acknowledgements settle.
        Scientific publication separately requires the ProjectDraftDocuments codec.

        Effects:
            scope: canonical_command
            read_only: false
            idempotent: true
            destructive: false
        """
        return self._host_project_draft_save_intent(
            operation, project, branch, key, base, content, expected=expected
        )

    _public_metadata(
        project_draft_save_intent,
        ProjectRuntimeGlue,
        "_host_project_draft_save_intent",
        "Runtime",
        "pse._workflow",
    )

    def project_draft_save(
        self,
        intent: ProjectDraftSaveIntent | ProjectDraftSaveIntentOutput,
        content: bytes,
    ) -> ProjectAcknowledgementOutput:
        """Save original bytes while retaining artifact and operation identity.

        Effects:
            scope: canonical_command
            read_only: false
            idempotent: true
            destructive: false
        """
        return self._host_project_draft_save(intent, content)

    _public_metadata(
        project_draft_save,
        ProjectRuntimeGlue,
        "_host_project_draft_save",
        "Runtime",
        "pse._workflow",
    )

    def project_draft_record(self, key: str) -> ProjectDraftRecordOutput | None:
        """Read raw custody metadata without granting a live file read.

        Effects:
            scope: observation
            read_only: true
            idempotent: true
            destructive: false
        """
        return self._host_project_draft_record(key)

    _public_metadata(
        project_draft_record,
        ProjectRuntimeGlue,
        "_host_project_draft_record",
        "Runtime",
        "pse._workflow",
    )

    def project_draft_bytes(self, key: str) -> bytes:
        """Read this exact raw body through its current protection owner.

        Effects:
            scope: observation
            read_only: true
            idempotent: true
            destructive: false
        """
        return self._host_project_draft_bytes(key)

    _public_metadata(
        project_draft_bytes,
        ProjectRuntimeGlue,
        "_host_project_draft_bytes",
        "Runtime",
        "pse._workflow",
    )

    def project_select(
        self, request: ProjectSelectRequest, physical: PhysicalContext
    ) -> ProjectSelection:
        """Freeze source and physical identities for a branch or bookmark.

        Effects:
            scope: source_admission
            read_only: false
            idempotent: true
            destructive: false
        """
        return self._host_project_select(request, physical)

    _public_metadata(
        project_select,
        ProjectRuntimeGlue,
        "_host_project_select",
        "Runtime",
        "pse._workflow",
    )

    def project_open_selection(
        self, selection: ProjectSelection, physical: PhysicalContext
    ) -> ModelingPackage:
        """Reopen retained science after labels or the branch head have changed.

        Effects:
            scope: source_admission
            read_only: false
            idempotent: true
            destructive: false
        """
        return self._host_project_open_selection(selection, physical)

    _public_metadata(
        project_open_selection,
        ProjectRuntimeGlue,
        "_host_project_open_selection",
        "Runtime",
        "pse._workflow",
    )

    def project_navigate(
        self, operation: str, project: ProjectRecord, request: ProjectNavigationRequest
    ) -> ProjectAcknowledgementOutput:
        """Apply an explicit name, bookmark, savepoint or visibility operation.

        Keep the original project generation and request when settling a retry.

        Effects:
            scope: canonical_command
            read_only: false
            idempotent: true
            destructive: false
        """
        return self._host_project_navigate(operation, project, request)

    _public_metadata(
        project_navigate,
        ProjectRuntimeGlue,
        "_host_project_navigate",
        "Runtime",
        "pse._workflow",
    )

    def project_metadata(
        self, request: ProjectMetadataRequest | ProjectMetadataRequestOutput
    ) -> ProjectMetadataPageOutput:
        """Read bounded headers separately from raw or scientific payloads.

        Effects:
            scope: observation
            read_only: true
            idempotent: true
            destructive: false
        """
        return self._host_project_metadata(request)

    _public_metadata(
        project_metadata,
        ProjectRuntimeGlue,
        "_host_project_metadata",
        "Runtime",
        "pse._workflow",
    )

    def project_draft_version(self, key: str) -> ProjectDraftVersionOutput | None:
        """Select an exact immutable raw version after later head changes.

        Effects:
            scope: observation
            read_only: true
            idempotent: true
            destructive: false
        """
        return self._host_project_draft_version(key)

    _public_metadata(
        project_draft_version,
        ProjectRuntimeGlue,
        "_host_project_draft_version",
        "Runtime",
        "pse._workflow",
    )

    def project_draft_version_bytes(self, key: str) -> bytes:
        """Read original raw custody without moving a current draft head.

        Effects:
            scope: observation
            read_only: true
            idempotent: true
            destructive: false
        """
        return self._host_project_draft_version_bytes(key)

    _public_metadata(
        project_draft_version_bytes,
        ProjectRuntimeGlue,
        "_host_project_draft_version_bytes",
        "Runtime",
        "pse._workflow",
    )

    def project_draft_delta(
        self, source: ModelingPackage, delta: SourceDelta | SourceDeltaOutput
    ) -> ProjectDraftDeltaOutput:
        """Capture typed edits against the exact source and physical context.

        Effects:
            scope: observation
            read_only: true
            idempotent: true
            destructive: false
        """
        return self._host_project_draft_delta(source, delta)

    _public_metadata(
        project_draft_delta,
        ProjectRuntimeGlue,
        "_host_project_draft_delta",
        "Runtime",
        "pse._workflow",
    )

    def project_admit_draft(
        self, operation: str, key: str, physical: PhysicalContext
    ) -> ModelingPackage:
        """Admit locally and retain diagnostics; the branch head stays unchanged.

        Effects:
            scope: canonical_command
            read_only: false
            idempotent: true
            destructive: false
        """
        return self._host_project_admit_draft(operation, key, physical)

    _public_metadata(
        project_admit_draft,
        ProjectRuntimeGlue,
        "_host_project_admit_draft",
        "Runtime",
        "pse._workflow",
    )

    def project_acknowledgement(
        self, operation: str
    ) -> ProjectAcknowledgementOutput | None:
        """Observe the original immutable receipt; absence does not prove rollback.

        Effects:
            scope: observation
            read_only: true
            idempotent: true
            destructive: false
        """
        return self._host_project_acknowledgement(operation)

    _public_metadata(
        project_acknowledgement,
        ProjectRuntimeGlue,
        "_host_project_acknowledgement",
        "Runtime",
        "pse._workflow",
    )

    def project_publish_draft(
        self,
        intent: ProjectPublication | ProjectPublicationOutput,
        physical: PhysicalContext,
    ) -> ModelingPackage:
        """Admit saved documents and publish their exact generation.

        Effects:
            scope: canonical_command
            read_only: false
            idempotent: true
            destructive: false
        """
        return self._host_project_publish_draft(intent, physical)

    _public_metadata(
        project_publish_draft,
        ProjectRuntimeGlue,
        "_host_project_publish_draft",
        "Runtime",
        "pse._workflow",
    )

    def project_merge_preview(
        self,
        base: ModelingPackage,
        target: ModelingPackage,
        proposal: ModelingPackage,
        *,
        resolutions: Sequence[ProjectMergeResolution],
    ) -> ProjectMergePreview:
        """Inspect three-way logical conflicts before issuing a publication decision.

        Effects:
            scope: observation
            read_only: true
            idempotent: true
            destructive: false
        """
        return self._host_project_merge_preview(
            base, target, proposal, resolutions=resolutions
        )

    _public_metadata(
        project_merge_preview,
        ProjectRuntimeGlue,
        "_host_project_merge_preview",
        "Runtime",
        "pse._workflow",
    )

    def project_merge_intent(
        self,
        publication: ProjectPublication | ProjectPublicationOutput,
        base: ModelingPackage,
        target: ModelingPackage,
        proposal: ModelingPackage,
        *,
        resolutions: Sequence[ProjectMergeResolution],
    ) -> ProjectMergeIntentOutput:
        """Retain exact admitted revision selections and explicit conflict choices.

        Effects:
            scope: canonical_command
            read_only: false
            idempotent: true
            destructive: false
        """
        return self._host_project_merge_intent(
            publication, base, target, proposal, resolutions=resolutions
        )

    _public_metadata(
        project_merge_intent,
        ProjectRuntimeGlue,
        "_host_project_merge_intent",
        "Runtime",
        "pse._workflow",
    )

    def project_merge_draft_intent(
        self,
        publication: ProjectPublication | ProjectPublicationOutput,
        base: ModelingPackage,
        target: ModelingPackage,
        version: ProjectDraftVersion | ProjectDraftVersionOutput,
        *,
        resolutions: Sequence[ProjectMergeResolution],
    ) -> ProjectMergeIntentOutput:
        """Merge a saved version without a temporary source publication.

        Effects:
            scope: canonical_command
            read_only: false
            idempotent: true
            destructive: false
        """
        return self._host_project_merge_draft_intent(
            publication, base, target, version, resolutions=resolutions
        )

    _public_metadata(
        project_merge_draft_intent,
        ProjectRuntimeGlue,
        "_host_project_merge_draft_intent",
        "Runtime",
        "pse._workflow",
    )

    def project_undo_intent(
        self,
        publication: ProjectPublication | ProjectPublicationOutput,
        target: ModelingPackage,
        restore: ModelingPackage,
    ) -> ProjectMergeIntentOutput:
        """Retain an ordinary compensating source publication, never a head rewind.

        Effects:
            scope: canonical_command
            read_only: false
            idempotent: true
            destructive: false
        """
        return self._host_project_undo_intent(publication, target, restore)

    _public_metadata(
        project_undo_intent,
        ProjectRuntimeGlue,
        "_host_project_undo_intent",
        "Runtime",
        "pse._workflow",
    )

    def project_publish_merge(
        self,
        intent: ProjectMergeIntent | ProjectMergeIntentOutput,
        physical: PhysicalContext,
    ) -> ModelingPackage:
        """Publish or settle the original immutable merge/undo decision.

        Effects:
            scope: canonical_command
            read_only: false
            idempotent: true
            destructive: false
        """
        return self._host_project_publish_merge(intent, physical)

    _public_metadata(
        project_publish_merge,
        ProjectRuntimeGlue,
        "_host_project_publish_merge",
        "Runtime",
        "pse._workflow",
    )


class ResearchRuntime(ResearchRuntimeGlue):
    """Typed facade composed into the ordinary Runtime; no separate engine."""

    def research_dataset_create(
        self, request: ResearchDatasetRequest
    ) -> ResearchDatasetOutput:
        """Admit original typed attribute references into an immutable version.

        Effects:
            scope: canonical_command
            read_only: false
            idempotent: true
            destructive: false
        """
        return self._host_research_dataset_create(request)

    _public_metadata(
        research_dataset_create,
        ResearchRuntimeGlue,
        "_host_research_dataset_create",
        "ResearchRuntime",
        "pse._research",
    )

    def research_selection_create(
        self, request: ResearchSelectionRequest
    ) -> ResearchSelection:
        """Freeze complete original observations, exclusions and calibration context.

        Effects:
            scope: canonical_command
            read_only: false
            idempotent: true
            destructive: false
        """
        return self._host_research_selection_create(request)

    _public_metadata(
        research_selection_create,
        ResearchRuntimeGlue,
        "_host_research_selection_create",
        "ResearchRuntime",
        "pse._research",
    )

    def research_read(self, request: ResearchReadRequest) -> ResearchReadResultOutput:
        """Read bounded metadata without granting measurement or result lifetime.

        Effects:
            scope: observation
            read_only: true
            idempotent: true
            destructive: false
        """
        return self._host_research_read(request)

    _public_metadata(
        research_read,
        ResearchRuntimeGlue,
        "_host_research_read",
        "ResearchRuntime",
        "pse._research",
    )

    def research_dataset_open(
        self, key: str, context: ContextSelection | ContextSelectionOutput
    ) -> ModelingPackage:
        """Reopen selected original typed data through source protections.

        Effects:
            scope: source_admission
            read_only: false
            idempotent: true
            destructive: false
        """
        return self._host_research_dataset_open(key, context)

    _public_metadata(
        research_dataset_open,
        ResearchRuntimeGlue,
        "_host_research_dataset_open",
        "ResearchRuntime",
        "pse._research",
    )

    def research_selection_open(self, key: str) -> ModelingPackage:
        """Reopen exact fitting inputs carrying their original research ancestry.

        Effects:
            scope: source_admission
            read_only: false
            idempotent: true
            destructive: false
        """
        return self._host_research_selection_open(key)

    _public_metadata(
        research_selection_open,
        ResearchRuntimeGlue,
        "_host_research_selection_open",
        "ResearchRuntime",
        "pse._research",
    )

    def research_publication_preview(
        self, request: ResearchPublicationRequest
    ) -> ResearchPublicationPreviewOutput:
        """Inspect an admitted authored delta from original qualified fitted cells.

        Effects:
            scope: observation
            read_only: true
            idempotent: true
            destructive: false
        """
        return self._host_research_publication_preview(request)

    _public_metadata(
        research_publication_preview,
        ResearchRuntimeGlue,
        "_host_research_publication_preview",
        "ResearchRuntime",
        "pse._research",
    )

    def research_publish(
        self, intent: ResearchPublicationIntent | ResearchPublicationIntentOutput
    ) -> ResearchPublication:
        """Settle or execute one retained exact source publication intent.

        Effects:
            scope: canonical_command
            read_only: false
            idempotent: true
            destructive: false
        """
        return self._host_research_publish(intent)

    _public_metadata(
        research_publish,
        ResearchRuntimeGlue,
        "_host_research_publish",
        "ResearchRuntime",
        "pse._research",
    )

    def research_publication_open(self, key: str) -> ModelingPackage:
        """Open the admitted parameter source with ancestry for downstream studies.

        Effects:
            scope: source_admission
            read_only: false
            idempotent: true
            destructive: false
        """
        return self._host_research_publication_open(key)

    _public_metadata(
        research_publication_open,
        ResearchRuntimeGlue,
        "_host_research_publication_open",
        "ResearchRuntime",
        "pse._research",
    )


class StudyWorkbenchMixin(StudyWorkbenchGlue):
    """Public runtime composition; the native owner performs selection and admission."""

    def study_workbench_page(
        self, request: StudyWorkbenchRequest
    ) -> StudyWorkbenchPageOutput:
        """Browse bounded compact metadata, including terminal studies.

        Effects:
            scope: observation
            read_only: true
            idempotent: true
            destructive: false
        """
        return self._host_study_workbench_page(request)

    _public_metadata(
        study_workbench_page,
        StudyWorkbenchGlue,
        "_host_study_workbench_page",
        "StudyWorkbenchMixin",
        "pse._study_workbench",
    )

    def freeze_study_cohort(self, request: StudyCohortRequest) -> FrozenStudyCohort:
        """Freeze a complete bounded cohort before protected analytical reads.

        Effects:
            scope: capture
            read_only: false
            idempotent: false
            destructive: false
        """
        return self._host_freeze_study_cohort(request)

    _public_metadata(
        freeze_study_cohort,
        StudyWorkbenchGlue,
        "_host_freeze_study_cohort",
        "StudyWorkbenchMixin",
        "pse._study_workbench",
    )

    def observe_study(
        self,
        request: StudyObservationRequest,
        *,
        cursor: StudyObservationCursor | StudyObservationCursorOutput | None = None,
    ) -> StudyObserver:
        """Open or reconnect passive observation of one exact selected study.

        Effects:
            scope: observation
            read_only: true
            idempotent: true
            destructive: false
        """
        return self._host_observe_study(request, cursor=cursor)

    _public_metadata(
        observe_study,
        StudyWorkbenchGlue,
        "_host_observe_study",
        "StudyWorkbenchMixin",
        "pse._study_workbench",
    )


__all__ = ("ProjectRuntimeOperations", "ResearchRuntime", "StudyWorkbenchMixin")
