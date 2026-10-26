# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Typed invocation glue for project and source lifecycle operations."""

from collections.abc import Sequence
from pathlib import Path

from pse import codec
from pse._build import _NativeRuntime
from pse._modeling import ModelingPackage
from pse._physical import PhysicalContext
from pse._project_bundle import ProjectBundle
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
    SourceDelta,
    SourceDeltaOutput,
)


class ProjectRuntimeGlue:
    """One native owner supplies each generated public operation."""

    @property
    def _handle(self) -> _NativeRuntime:
        """Read-only native owner supplied by the concrete Runtime facade."""
        raise NotImplementedError

    def _host_modeling_revision(
        self, revision: str, physical: "PhysicalContext"
    ) -> ModelingPackage:
        """Reopen a retained source revision from canonical storage."""
        return ModelingPackage(
            self._handle.modeling_revision(
                revision,
                physical._handle,  # noqa: SLF001 - same native boundary
            )
        )

    def _host_project_context_cut(self, project: str) -> ContextCut:
        """Capture recorded context independently of the source sequence."""
        return self._handle.project_context_cut(project)

    def _host_project_context_select(
        self, cut: ContextCut, scope: str, effective_at_us: int
    ) -> ContextSelectionOutput | None:
        """Select at an explicit UTC microsecond instant and original recorded cut."""
        return self._handle.project_context_select(cut, scope, effective_at_us)

    def _host_project_context_record(self, key: str) -> ProjectContextOutput | None:
        """Read immutable metadata, including hidden or deleted records."""
        return self._handle.project_context_record(key)

    def _host_project_context_reopen(
        self, selection: ContextSelection | ContextSelectionOutput
    ) -> ModelingPackage:
        """Reopen the original persisted source and physical closure."""
        return ModelingPackage(self._handle.project_context_reopen(selection))

    def _host_project_context_package(
        self,
        selection: ContextSelection | ContextSelectionOutput,
        physical: "PhysicalContext",
    ) -> ModelingPackage:
        """Admit this exact source selection without selecting latest."""
        return ModelingPackage(
            self._handle.project_context_package(
                selection,
                physical._handle,  # noqa: SLF001 - same native boundary
            )
        )

    def _host_project_context_append(
        self,
        operation: str,
        project: ProjectRecord,
        scope: str,
        source: ModelingPackage,
        effective_from_us: int,
        *,
        effective_to_us: int | None,
        corrects: str | None,
        deleted: bool,
    ) -> ProjectContextOutput:
        """Append an exact source, effective interval and correction ancestry.

        Retain the original project descriptor and operation through uncertain
        completion. Scientific values remain in the admitted source, not this context
        record.
        """
        return self._handle.project_context_append(
            operation,
            project,
            scope,
            source._handle,  # noqa: SLF001 - same native boundary
            effective_from_us,
            effective_to_us=effective_to_us,
            corrects=corrects,
            deleted=deleted,
        )

    def _host_project_create(
        self, operation: str, project: str, title: str
    ) -> ProjectAcknowledgementOutput:
        """Create project organization using an original caller-issued operation."""
        return self._handle.project_create(operation, project, title)

    def _host_project_record(self, project: str) -> ProjectRecord | None:
        """Read project metadata without hydrating scientific payloads."""
        return self._handle.project_record(project)

    def _host_project_branch_record(
        self, branch: str
    ) -> ProjectBranchRecordOutput | None:
        """Read branch ownership; its source problem owns the single head."""
        return self._handle.project_branch_record(branch)

    def _host_project_branches(
        self, project: str, *, cursor: ProjectCursor | ProjectCursorBody | None
    ) -> ProjectBranchPageOutput:
        """Read one ordered page with an authority/generation-fenced continuation."""
        if isinstance(cursor, ProjectCursorBody):
            cursor = ProjectCursor(
                issuing_store=cursor.issuing_store,
                project=cursor.project,
                generation=cursor.generation,
                after=cursor.after,
            )
        return self._handle.project_branches(project, cursor=cursor)

    def _host_project_fork(
        self,
        project: str,
        branch: str,
        problem: str,
        operation: str,
        source: ModelingPackage,
    ) -> str:
        """Fork an exact admitted source closure into an independent writable branch."""
        return self._handle.project_fork(
            project,
            branch,
            problem,
            operation,
            source._handle,  # noqa: SLF001 - same native boundary
        )

    def _host_project_publication_intent(
        self,
        project: str,
        branch: str,
        expected: str,
        operation: str,
        *,
        draft: str | None,
    ) -> ProjectPublicationOutput:
        """Issue the original exact source/draft CAS; retain it through uncertainty."""
        return self._handle.project_publication_intent(
            project, branch, expected, operation, draft=draft
        )

    def _host_project_draft_save_intent(
        self,
        operation: str,
        project: str,
        branch: str,
        key: str,
        base: str,
        content: bytes,
        *,
        expected: int | None,
    ) -> ProjectDraftSaveIntentOutput:
        """Read-only issue of raw custody; content may be malformed or empty.

        Keep the original intent until artifact and project acknowledgements settle.
        Scientific publication separately requires the ProjectDraftDocuments codec.
        """
        return self._handle.project_draft_save_intent(
            operation, project, branch, key, base, content, expected=expected
        )

    def _host_project_draft_save(
        self,
        intent: ProjectDraftSaveIntent | ProjectDraftSaveIntentOutput,
        content: bytes,
    ) -> ProjectAcknowledgementOutput:
        """Save original bytes while retaining artifact and operation identity."""
        return self._handle.project_draft_save(intent, content)

    def _host_project_draft_record(self, key: str) -> ProjectDraftRecordOutput | None:
        """Read raw custody metadata without granting a live file read."""
        return self._handle.project_draft_record(key)

    def _host_project_draft_bytes(self, key: str) -> bytes:
        """Read this exact raw body through its current protection owner."""
        return self._handle.project_draft_bytes(key)

    def _host_project_closure(
        self, request: ProjectExportRequest, physical: PhysicalContext | None
    ) -> ProjectClosureExplanationOutput:
        """Explain exact selected membership through the original owners."""
        return self._handle.project_closure(
            request,
            None if physical is None else physical._handle,  # noqa: SLF001 -- package-native facade composition
        )

    def _host_project_export(
        self,
        destination: Path,
        request: ProjectExportRequest,
        physical: PhysicalContext | None,
    ) -> ProjectManifestOutput:
        """Copy a protected selected closure for offline inspection and fresh import."""
        return self._handle.project_export(
            destination,
            request,
            None if physical is None else physical._handle,  # noqa: SLF001 -- package-native facade composition
        )

    def _host_project_import_issue(
        self, bundle: ProjectBundle, *, title: str, branch_name: str
    ) -> ProjectImportIntentOutput:
        """Issue a retryable intent bound to this receiving authority and exact pack."""
        return self._handle.project_import_issue(
            bundle._handle,  # noqa: SLF001 -- package-native facade composition
            title=title,
            branch_name=branch_name,
        )

    def _host_project_import_status(
        self, intent: ProjectImportIntent | ProjectImportIntentOutput
    ) -> ProjectImportReportOutput | None:
        """Distinguish admitted source from complete copied project custody."""
        return self._handle.project_import_status(intent)

    def _host_project_import(
        self,
        bundle: ProjectBundle,
        intent: ProjectImportIntent | ProjectImportIntentOutput,
    ) -> ProjectImportReportOutput:
        """Admit fresh source and copied evidence before project availability."""
        return self._handle.project_import(bundle._handle, intent)  # noqa: SLF001

    def _host_project_import_recover(
        self,
        intent: ProjectImportIntent | ProjectImportIntentOutput,
        original: ArtifactIntent | ArtifactIntentBody,
    ) -> ProjectImportRecoveryReportOutput:
        """Settle the original occurrence without renewing its clock."""
        record = (
            original
            if isinstance(original, ArtifactIntent)
            else codec.convert_document(
                codec.document_builtins(original), ArtifactIntent
            )
        )
        return self._handle.project_import_recover(intent, record)

    def _host_project_import_open(
        self, intent: ProjectImportIntent | ProjectImportIntentOutput
    ) -> ModelingPackage:
        """Reopen owned imported science without the originally supplied pack."""
        return ModelingPackage(self._handle.project_import_open(intent))

    def _host_lifecycle_select(
        self, target: LifecycleTarget
    ) -> LifecycleSelectionOutput:
        """Freeze original issuing authority and current owner generations."""
        return self._handle.lifecycle_select(target)

    def _host_lifecycle_explain(
        self, selection: LifecycleSelection | LifecycleSelectionOutput
    ) -> LifecycleExplanationOutput:
        """Observe references and known bytes without granting disposal authority."""
        return self._handle.lifecycle_explain(selection)

    def _host_lifecycle_release(
        self, selection: LifecycleSelection | LifecycleSelectionOutput
    ) -> LifecycleReleaseOutput:
        """Recheck and dispatch release to its original owner."""
        return self._handle.lifecycle_release(selection)

    def _host_project_select(
        self, request: ProjectSelectRequest, physical: "PhysicalContext"
    ) -> ProjectSelection:
        """Freeze source and physical identities for a branch or bookmark."""
        return self._handle.project_select(request, physical._handle)  # noqa: SLF001

    def _host_project_open_selection(
        self, selection: ProjectSelection, physical: "PhysicalContext"
    ) -> ModelingPackage:
        """Reopen retained science after labels or the branch head have changed."""
        return ModelingPackage(
            self._handle.project_open_selection(selection, physical._handle)  # noqa: SLF001
        )

    def _host_project_navigate(
        self,
        operation: str,
        project: ProjectRecord,
        request: ProjectNavigationRequest,
    ) -> ProjectAcknowledgementOutput:
        """Apply an explicit name, bookmark, savepoint or visibility operation.

        Keep the original project generation and request when settling a retry.
        """
        return self._handle.project_navigate(operation, project, request)

    def _host_project_metadata(
        self, request: ProjectMetadataRequest | ProjectMetadataRequestOutput
    ) -> ProjectMetadataPageOutput:
        """Read bounded headers separately from raw or scientific payloads."""
        return self._handle.project_metadata(request)

    def _host_project_draft_version(self, key: str) -> ProjectDraftVersionOutput | None:
        """Select an exact immutable raw version after later head changes."""
        return self._handle.project_draft_version(key)

    def _host_project_draft_version_bytes(self, key: str) -> bytes:
        """Read original raw custody without moving a current draft head."""
        return self._handle.project_draft_version_bytes(key)

    def _host_project_draft_delta(
        self, source: ModelingPackage, delta: SourceDelta | SourceDeltaOutput
    ) -> ProjectDraftDeltaOutput:
        """Capture typed edits against the exact source and physical context."""
        return self._handle.project_draft_delta(source._handle, delta)  # noqa: SLF001

    def _host_project_admit_draft(
        self, operation: str, key: str, physical: "PhysicalContext"
    ) -> ModelingPackage:
        """Admit locally and retain diagnostics; the branch head stays unchanged."""
        return ModelingPackage(
            self._handle.project_admit_draft(operation, key, physical._handle)  # noqa: SLF001
        )

    def _host_project_acknowledgement(
        self, operation: str
    ) -> ProjectAcknowledgementOutput | None:
        """Observe the original immutable receipt; absence does not prove rollback."""
        return self._handle.project_acknowledgement(operation)

    def _host_project_merge_preview(
        self,
        base: ModelingPackage,
        target: ModelingPackage,
        proposal: ModelingPackage,
        *,
        resolutions: Sequence[ProjectMergeResolution],
    ) -> ProjectMergePreview:
        """Inspect three-way logical conflicts before issuing a publication decision."""
        return self._handle.project_merge_preview(
            base._handle,  # noqa: SLF001 -- package-native facade composition
            target._handle,  # noqa: SLF001 -- package-native facade composition
            proposal._handle,  # noqa: SLF001 -- package-native facade composition
            list(resolutions),
        )

    def _host_project_merge_intent(
        self,
        publication: ProjectPublication | ProjectPublicationOutput,
        base: ModelingPackage,
        target: ModelingPackage,
        proposal: ModelingPackage,
        *,
        resolutions: Sequence[ProjectMergeResolution],
    ) -> ProjectMergeIntentOutput:
        """Retain exact admitted revision selections and explicit conflict choices."""
        return self._handle.project_merge_intent(
            publication,
            base._handle,  # noqa: SLF001 -- package-native facade composition
            target._handle,  # noqa: SLF001 -- package-native facade composition
            proposal._handle,  # noqa: SLF001 -- package-native facade composition
            list(resolutions),
        )

    def _host_project_merge_draft_intent(
        self,
        publication: ProjectPublication | ProjectPublicationOutput,
        base: ModelingPackage,
        target: ModelingPackage,
        version: ProjectDraftVersion | ProjectDraftVersionOutput,
        *,
        resolutions: Sequence[ProjectMergeResolution],
    ) -> ProjectMergeIntentOutput:
        """Merge a saved version without a temporary source publication."""
        return self._handle.project_merge_draft_intent(
            publication,
            base._handle,  # noqa: SLF001 -- package-native facade composition
            target._handle,  # noqa: SLF001 -- package-native facade composition
            version,
            list(resolutions),
        )

    def _host_project_undo_intent(
        self,
        publication: ProjectPublication | ProjectPublicationOutput,
        target: ModelingPackage,
        restore: ModelingPackage,
    ) -> ProjectMergeIntentOutput:
        """Retain an ordinary compensating source publication, never a head rewind."""
        return self._handle.project_undo_intent(
            publication,
            target._handle,  # noqa: SLF001 -- package-native facade composition
            restore._handle,  # noqa: SLF001 -- package-native facade composition
        )

    def _host_project_publish_merge(
        self,
        intent: ProjectMergeIntent | ProjectMergeIntentOutput,
        physical: "PhysicalContext",
    ) -> ModelingPackage:
        """Publish or settle the original immutable merge/undo decision."""
        return ModelingPackage(
            self._handle.project_publish_merge(intent, physical._handle)  # noqa: SLF001 -- package-native facade composition
        )

    def _host_project_publish_draft(
        self,
        intent: ProjectPublication | ProjectPublicationOutput,
        physical: "PhysicalContext",
    ) -> ModelingPackage:
        """Admit saved documents and publish their exact generation."""
        return ModelingPackage(
            self._handle.project_publish_draft(
                intent,
                physical._handle,  # noqa: SLF001 - same native boundary
            )
        )
