# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Typed declarations and owned handles over the single native computation pipeline."""

from collections.abc import Mapping, Sequence
from datetime import timedelta
from pathlib import Path

import attrs
import pyarrow as pa

from pse import codec
from pse._build import (
    EngineSettings,
    OperationalStore,
    _NativePhysicalContext,
    _NativeRuntime,
)
from pse._inspection import Publication, TableStream
from pse._modeling import ModelingPackage
from pse._runs import (
    PreparedOperation,
    ProgressStream,
    PublicationTicket,
    RunHandle,
    RunResult,
    StudyHandle,
    Workspace,
)
from pse._strategies import (
    PreparedStrategy,
)
from pse.contracts import runtime as result_contracts
from pse.contracts.documents import (
    ConicRequest,
    ExportReceipt,
    InventoryControls,
    ProgressControls,
    PublicationSettlement,
    ResourceReport,
    RunControls,
    SolveSettings,
)
from pse.contracts.enums import AttemptState, JobState, StudyState
from pse.contracts.identities import (
    AttemptId,
    PublicationId,
    RunId,
    StudyId,
    WorkspaceId,
)
from pse.contracts.values import ContentHash, SemanticId

SolverCapability = result_contracts.RuntimeSolverCapabilitiesRow
OperationalAttempt = result_contracts.RuntimeOperationalAttemptsRow
OperationalJob = result_contracts.RuntimeOperationalJobsRow
OperationalStudy = result_contracts.RuntimeOperationalStudiesRow


@attrs.frozen(init=False)
class Runtime:
    """One native deployment budget shared with publication inspection."""

    _handle: _NativeRuntime

    def __init__(
        self,
        settings: EngineSettings,
        *,
        substrate: str,
        producer: str | None = None,
        store: OperationalStore | None = None,
    ) -> None:
        """Attach to the shared deployment under an explicit durability class.

        Args:
            settings: The deployment budget shared with publication inspection.
            substrate: Supervisor state directory for canonical scientific storage.
            producer: Optional qualified mathematical producer receipt path.
            store: With a store, every run is a durable attempt registered in it
                and may be published; without one, runs are ephemeral and cannot
                publish (ADR-0112 Outcome 16).
        """
        object.__setattr__(
            self,
            "_handle",
            _NativeRuntime(
                settings, substrate=substrate, producer=producer, store=store
            ),
        )

    @property
    def durable(self) -> bool:
        """Whether runs are durable attempts in an operational store."""
        return self._handle.durable

    def modeling_revision(
        self, revision: str, physical: "PhysicalContext"
    ) -> ModelingPackage:
        """Reopen a retained source revision from canonical storage."""
        return ModelingPackage(
            self._handle.modeling_revision(
                revision,
                physical._handle,  # noqa: SLF001 - same native boundary
            )
        )

    def resource_usage(self) -> ResourceReport:
        """Observe the shared deployment's accounted pool and process memory."""
        return self._handle.resource_usage()

    def runs(
        self,
        *,
        run_id: RunId | None = None,
        states: Sequence[AttemptState] = (),
        controls: InventoryControls | None = None,
    ) -> tuple[OperationalAttempt, ...]:
        """List the store's durable attempts, newest first; they survive restarts.

        Args:
            run_id: Only the attempts of this run.
            states: Only attempts in these lifecycle states; every state when empty.
            controls: Rust-owned page limits.

        Returns:
            One registry ``runtime.operational_attempts`` row per attempt.
        """
        stream = TableStream(
            self._handle.runs(
                run_id=None if run_id is None else run_id.to_hex(),
                states=[AttemptState(state).value for state in states],
                controls=None if controls is None else codec.encode_json(controls),
            )
        )
        return tuple(
            codec.structure_rows(pa.table(stream).to_pylist(), OperationalAttempt)
        )

    def studies(
        self,
        *,
        states: Sequence[StudyState] = (),
        controls: InventoryControls | None = None,
    ) -> tuple[OperationalStudy, ...]:
        """List the store's durable studies, newest first.

        Args:
            states: Only studies in these states; every state when empty.
            controls: Rust-owned page limits.

        Returns:
            One registry ``runtime.operational_studies`` row per study.
        """
        stream = TableStream(
            self._handle.studies(
                states=[StudyState(state).value for state in states],
                controls=None if controls is None else codec.encode_json(controls),
            )
        )
        return tuple(
            codec.structure_rows(pa.table(stream).to_pylist(), OperationalStudy)
        )

    def jobs(
        self,
        *,
        states: Sequence[JobState] = (),
        controls: InventoryControls | None = None,
    ) -> tuple[OperationalJob, ...]:
        """List the store's durable jobs, newest first.

        Args:
            states: Only jobs in these states; every state when empty.
            controls: Rust-owned page limits.

        Returns:
            One registry ``runtime.operational_jobs`` row per job.
        """
        stream = TableStream(
            self._handle.jobs(
                states=[JobState(state).value for state in states],
                controls=None if controls is None else codec.encode_json(controls),
            )
        )
        return tuple(codec.structure_rows(pa.table(stream).to_pylist(), OperationalJob))

    def query(
        self,
        sql: str,
        *,
        result: RunResult | None = None,
        publication: Publication | None = None,
    ) -> TableStream:
        """Run SQL over the operational store and, optionally, results.

        A durable runtime's query sees the operational relations as read-only
        tables under ``pse_ops`` (``pse_ops.attempts``, ``pse_ops.jobs``,
        ``pse_ops.progress_events``, ``pse_ops.incumbents``, ``pse_ops.studies``,
        ...), read from the store as the query runs; typed filters on their
        identities, states and times are applied by the store.

        Args:
            sql: One query.
            result: A run whose retained result relations the query may read, as
                ``workspace.<namespace>.<name>`` (``workspace.runtime.solve_runs``).
            publication: An open publication whose members the query may read by
                their catalog, schema and table names.

        Returns:
            The answer as a one-consumption Arrow stream.
        """
        return TableStream(
            self._handle.query(
                sql,
                result=None if result is None else result._handle,  # noqa: SLF001 - same native boundary
                publication=None if publication is None else publication._handle,  # noqa: SLF001 - same native boundary
            )
        )

    def progress(
        self, attempt_id: AttemptId, *, controls: ProgressControls | None = None
    ) -> ProgressStream:
        """Observe stored progress under Rust-owned page and follow controls."""
        return ProgressStream(
            self._handle.progress(
                attempt_id.to_hex(),
                controls=None if controls is None else codec.encode_json(controls),
            )
        )

    def study(self, study_id: StudyId) -> StudyHandle:
        """Return a handle on a durable study of this runtime's store."""
        return StudyHandle(self._handle.study(study_id.to_hex()))

    def work(self, *, jobs: int | None = None) -> int:
        """Serve the durable job queue in this process until it is empty.

        This process claims jobs as ``pse-worker --until-idle`` does: each
        runs under a lease, and a study's points, finalization and publication
        run here like in any worker.

        Args:
            jobs: Stop after this many jobs.

        Returns:
            The number of jobs processed.
        """
        return self._handle.work(jobs=jobs)

    def physical_from_documents(
        self, documents: Mapping[str, str | bytes]
    ) -> "PhysicalContext":
        """Admit actual physical package rows using the native source loader."""
        return PhysicalContext(self._handle.physical_from_documents(dict(documents)))

    def modeling_from_documents(
        self,
        documents: Sequence[Mapping[str, str | bytes]],
        physical: "PhysicalContext",
    ) -> ModelingPackage:
        """Admit an exact package closure with manifest-owned physical type aliases."""
        return ModelingPackage(
            self._handle.modeling_from_documents(
                [dict(bundle) for bundle in documents],
                physical._handle,  # noqa: SLF001 - same native boundary
            )
        )

    def capabilities(self) -> tuple[SolverCapability, ...]:
        """Discover linked native libraries without PATH or optional Python probes."""
        return codec.decode_rows_json(self._handle.capabilities(), SolverCapability)

    def clear_program_cache(self) -> None:
        """Release retained programs while keeping active prepared workers valid."""
        self._handle.clear_program_cache()

    def settle_publication(self, ticket: PublicationTicket) -> PublicationSettlement:
        """Settle a ticket whose commit outcome is unknown by querying the catalog.

        Nothing is prepared, written or solved again.
        """
        return codec.decode_json(
            self._handle.settle_publication(ticket.json), PublicationSettlement
        )

    def register_workspace(self, name: str, root: str | Path) -> Workspace:
        """Register a publication workspace, or return the one of that name and root.

        Args:
            name: The workspace's unique name.
            root: The directory its members are written under (a URI or local path).
                A root holding a former Delta control table is refused.

        Returns:
            The registered workspace.
        """
        uri = root.resolve().as_uri() + "/" if isinstance(root, Path) else root
        return codec.decode_json(self._handle.register_workspace(name, uri), Workspace)

    def workspace(self, name: str) -> Workspace:
        """Return a registered workspace by name."""
        return codec.decode_json(self._handle.workspace(name), Workspace)

    def head(self, workspace_id: WorkspaceId) -> PublicationId | None:
        """Return a workspace's head; ``None`` before its first publication."""
        head = self._handle.head(workspace_id.to_hex())
        return None if head is None else PublicationId(SemanticId.from_hex(head))

    def open(self, publication_id: PublicationId) -> Publication:
        """Open an exact publication under a catalog reader lease.

        The lease is renewed while the publication or a stream of it is open and
        released by ``close``; an ephemeral runtime is refused.
        """
        return Publication(self._handle.open(publication_id.to_hex()))

    def open_head(self, workspace_id: WorkspaceId) -> Publication:
        """Open a workspace's head under a catalog reader lease."""
        return Publication(self._handle.open_head(workspace_id.to_hex()))

    def export_publication(
        self,
        publication_id: PublicationId,
        destination: str | Path,
        *,
        valid_for: timedelta,
    ) -> ExportReceipt:
        """Export a publication for offline readers.

        Args:
            publication_id: The publication.
            destination: A new directory for the one-row manifest.
            valid_for: How long the export's lease protects the members.

        Returns:
            The receipt; release it with ``release_export``.
        """
        uri = (
            destination.resolve().as_uri() + "/"
            if isinstance(destination, Path)
            else destination
        )
        return codec.decode_json(
            self._handle.export_publication(
                publication_id.to_hex(), uri, valid_for.total_seconds()
            ),
            ExportReceipt,
        )

    def release_export(self, receipt: ExportReceipt) -> bool:
        """Release an export's lease; returns whether it was still held."""
        return self._handle.release_export(codec.encode_json(receipt))

    def prepare_conic(
        self,
        request: ConicRequest,
        physical: "PhysicalContext",
        settings: SolveSettings,
    ) -> PreparedStrategy:
        """Admit explicit library cone geometry and an exact quadratic witness."""
        return PreparedStrategy(
            self._handle.prepare_conic(
                codec.encode_json(request),
                physical._handle,  # noqa: SLF001 - same native boundary
                codec.encode_json(settings),
            )
        )

    def start(
        self, cases: Sequence[PreparedOperation], *, controls: RunControls | None = None
    ) -> RunHandle:
        """Run a finite sequence on the existing completion-owned native pipeline."""
        return RunHandle(
            self._handle.start(
                [c._handle for c in cases],  # noqa: SLF001 - same native boundary
                controls=None if controls is None else codec.encode_json(controls),
            )
        )


@attrs.frozen
class PhysicalContext:
    """Admitted quantities, units, prerequisites and original source rows."""

    _handle: _NativePhysicalContext

    @property
    def identity(self) -> ContentHash:
        """Complete native compiler context identity."""
        return ContentHash.from_prefixed(self._handle.identity)
