# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Typed declarations and owned handles over the single native computation pipeline."""

from collections.abc import Mapping, Sequence
from pathlib import Path

import attrs

from pse import codec
from pse._analyses import Analysis
from pse._artifacts import (
    ArtifactOwnerKind,
    ArtifactSettlement,
    ScientificArtifact,
    ScientificArtifactKind,
)
from pse._build import (
    EngineSettings,
    _NativePhysicalContext,
    _NativeRuntime,
)
from pse._inspection import TableStream
from pse._modeling import LocalStudy, ModelingPackage, StudyReport
from pse._runs import (
    PreparedOperation,
    ProgressStream,
    RunHandle,
    StudyHandle,
)
from pse._strategies import (
    PreparedStrategy,
)
from pse.contracts import runtime as result_contracts
from pse.contracts.documents import (
    AnalysisControls,
    ConicRequest,
    HistoryObligation,
    HistoryOwner,
    HistoryReclamationPage,
    ResourceReport,
    ResultReclamationPage,
    RunControls,
    SolveSettings,
    SourceSettlement,
    StudyDefinition,
    StudyRunControls,
)
from pse.contracts.identities import (
    SolutionId,
    StudyId,
)
from pse.contracts.values import ContentHash, SemanticId
from pse.diagnostics import Diagnostics

SolverCapability = result_contracts.RuntimeSolverCapabilitiesRow


@attrs.frozen(init=False)
class Runtime:
    """One native deployment budget with explicit local or durable composition."""

    _handle: _NativeRuntime

    def __init__(
        self,
        settings: EngineSettings,
        *,
        substrate: str | None = None,
        database: str | None = None,
        producer: str | None = None,
        ephemeral: bool = False,
    ) -> None:
        """Attach to the shared deployment under an explicit durability class.

        Args:
            settings: The deployment budget shared by compilation, native work
                and reads.
            substrate: Supervisor state directory for canonical scientific storage;
                optional for explicit ephemeral computation.
            database: Optional explicitly selected existing canonical database.
            producer: Optional qualified mathematical producer receipt path.
            ephemeral: Explicitly admit models and retain outcomes locally, without
                connecting during construction.
                Application execution durably records outcomes in canonical storage.
        """
        object.__setattr__(
            self,
            "_handle",
            _NativeRuntime(
                settings,
                substrate=substrate,
                database=database,
                producer=producer,
                ephemeral=ephemeral,
            ),
        )

    @staticmethod
    def initialize_database(
        settings: EngineSettings, *, substrate: str, database: str
    ) -> None:
        """Explicitly provision a selected database before opening its runtime."""
        _NativeRuntime.initialize_database(
            settings, substrate=substrate, database=database
        )

    def close(self) -> None:
        """Release this caller and its aliases.

        Admitted work continues through drain.
        """
        self._handle.close()

    def observations(self) -> Diagnostics:
        """Read preparation and execution diagnostics independently of this caller."""
        return Diagnostics(self._handle.observations())

    @property
    def durable(self) -> bool:
        """Whether execution records canonical runs and terminal observations."""
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

    def source_settlement(self, problem: str, operation: str) -> SourceSettlement:
        """Observe the original source acknowledgement without replaying the edit.

        An absent acknowledgement leaves the original outcome unproved.
        """
        return self._handle.source_settlement(problem, operation)

    def resource_usage(self) -> ResourceReport:
        """Observe the shared deployment's accounted pool and process memory."""
        return self._handle.resource_usage()

    def run_record(self, run: str) -> TableStream:
        """Read the exact registry canonical run row without result hydration."""
        return TableStream(self._handle.run_record(run))

    def attempt_record(self, attempt: str) -> TableStream:
        """Read one actual attempt's lifecycle and recorded terminal class."""
        return TableStream(self._handle.attempt_record(attempt))

    def result_manifest(self, attempt: str) -> TableStream:
        """Read the exact admitted closed descriptor through its Arrow schema."""
        return TableStream(self._handle.result_manifest(attempt))

    def recover_artifact_intent(self, intent: bytes) -> ArtifactSettlement:
        """Recover exact error/interrupt bytes for settlement; never replay creation."""
        return ArtifactSettlement(self._handle.recover_artifact_intent(intent))

    def scientific_artifact(self, key: str) -> ScientificArtifact:
        """Reopen one active independent artifact after producer retirement."""
        return ScientificArtifact(self._handle.scientific_artifact(key))

    def promote_seed_artifact(
        self, solution: SolutionId, *, owner_kind: ArtifactOwnerKind, owner: str
    ) -> ScientificArtifact:
        """Copy one freshly qualified original start and its reuse assessment."""
        return ScientificArtifact(
            self._handle.promote_seed_artifact(solution.to_hex(), owner_kind, owner)
        )

    def promote_scientific_artifact(
        self,
        run: str,
        attempt: str,
        kind: ScientificArtifactKind,
        *,
        owner_kind: ArtifactOwnerKind,
        owner: str,
    ) -> ScientificArtifact:
        """Preserve an interpreted scientific product; insufficient context refuses."""
        return ScientificArtifact(
            self._handle.promote_scientific_artifact(
                run, attempt, kind, owner_kind, owner
            )
        )

    def analysis(self, key: str) -> Analysis:
        """Reopen one exact active scientific graph without hydrating results."""
        return Analysis(self._handle.analysis(key))

    def result_analysis(
        self, run: str, attempt: str, controls: AnalysisControls
    ) -> Analysis:
        """Persist reported sensitivity evidence and exact result provenance."""
        return Analysis(
            self._handle.result_analysis(run, attempt, codec.encode_json(controls))
        )

    def retain_history_obligation(self, obligation: HistoryObligation) -> None:
        """Protect exact audit or unsettled-effect detail independently of payloads."""
        self._handle.retain_history_obligation(codec.encode_json(obligation))

    def release_history_obligation(self, obligation: HistoryObligation) -> None:
        """Release this exact attributed obligation; other owners still protect it."""
        self._handle.release_history_obligation(codec.encode_json(obligation))

    def release_history(self, owner: HistoryOwner) -> None:
        """Irreversibly release settled detail separately from payload retirement."""
        self._handle.release_history(codec.encode_json(owner))

    def reclaim_history_page(self, owner: HistoryOwner) -> HistoryReclamationPage:
        """Collect one bounded history page while preserving permanent replay fences."""
        return codec.decode_json(
            self._handle.reclaim_history_page(codec.encode_json(owner)),
            HistoryReclamationPage,
        )

    def forget_study_results(self, study: str) -> None:
        """Withdraw a terminal study's explicit retention obligation."""
        self._handle.forget_study_results(study)

    def forget_analysis_results(self, analysis: str) -> bool:
        """Remove a bounded page; repeat until creation settles and returns True."""
        return self._handle.forget_analysis_results(analysis)

    def reclaim_run_results(self, run: str) -> ResultReclamationPage:
        """Retire a terminal run and reclaim one bounded page of scientific data."""
        return codec.decode_json(
            self._handle.reclaim_run_results(run), ResultReclamationPage
        )

    def latest_results(
        self,
        problem: str,
        relation: str,
        *,
        classes: Sequence[str] = (),
        start: int = 0,
        end: int = 18446744073709551615,
    ) -> TableStream:
        """Select the newest admitted semantic run in explicit terminal classes.

        Empty classes include every recorded terminal outcome. A requested class
        never changes the scientific meaning of a failed or partial result.
        """
        return TableStream(
            self._handle.latest_results(
                problem, relation, classes=list(classes), start=start, end=end
            )
        )

    def results(
        self,
        run: str,
        attempt: str,
        relation: str,
        *,
        start: int = 0,
        end: int = 18446744073709551615,
    ) -> TableStream:
        """Stream one exact admitted run/attempt's registry result relation.

        The identities survive restart. ``start..end`` selects half-open recorded
        row coordinates; partial or failed outcomes keep their own available
        observations. Missing coverage does not select another attempt's values.
        The stream retains source/result protection until its unread work closes.
        Values preserve their exact IEEE payloads, units and registry metadata.
        """
        return TableStream(
            self._handle.results(run, attempt, relation, start=start, end=end)
        )

    def output_results(
        self,
        run: str,
        attempt: str,
        relation: str,
        output: SemanticId,
        field: str,
        *,
        partition: str = "0",
        start: int = 0,
        end: int = (1 << 64) - 1,
        minimum: float | None = None,
        maximum: float | None = None,
        missing: bool | None = None,
    ) -> TableStream:
        """Read one declared output's original scientific rows.

        Scalar bounds filter finite values; trajectory bounds filter time.
        Missing scalar values require an explicit missing predicate. Units and
        quality remain in the original rows; numeric indexes never replace bits.
        """
        return TableStream(
            self._handle.output_results(
                run,
                attempt,
                relation,
                output.to_hex(),
                field,
                partition=partition,
                start=start,
                end=end,
                minimum=minimum,
                maximum=maximum,
                missing=missing,
            )
        )

    def export_results(
        self,
        run: str,
        attempt: str,
        relation: str,
        destination: str | Path,
        *,
        start: int = 0,
        end: int = 18446744073709551615,
    ) -> None:
        """Export exact results as IPC with recorded source and terminal lineage.

        The final path appears after successful completion. An interrupted export
        retains a sibling ``.incomplete`` file. Existing destinations are refused.
        """
        self._handle.export_results(
            run, attempt, relation, str(destination), start=start, end=end
        )

    def progress(self, run: str, attempt: str) -> ProgressStream:
        """Read the exact terminal attempt's bounded recorded event history."""
        return ProgressStream(self._handle.progress(run, attempt))

    def record_study(self, local: LocalStudy) -> StudyDefinition:
        """Publish exact retained inputs and return a portable recorded definition."""
        return codec.decode_json(self._handle.record_study(local._handle), StudyDefinition)

    def execute_study(
        self, definition: StudyDefinition, *, controls: StudyRunControls | None = None
    ) -> StudyReport:
        """Join recorded durable execution and retain its original result report."""
        return StudyReport(self._handle.execute_study(
            codec.encode_json(definition),
            controls=None if controls is None else codec.encode_json(controls),
        ))

    def start_study(self, definition: StudyDefinition) -> StudyHandle:
        """Start durable workers from a genuinely recorded portable definition."""
        return StudyHandle(self._handle.start_study(codec.encode_json(definition)))

    def study(self, study_id: StudyId) -> StudyHandle:
        """Return a handle on a durable study of this runtime's store."""
        return StudyHandle(self._handle.study(study_id.to_hex()))

    def work(self, *, maximum_actions: int | None = None) -> int:
        """Serve native ready points and finalization until idle or the action limit.

        Args:
            maximum_actions: Stop after this many native worker actions.

        Returns:
            The number of native worker actions processed.
        """
        return self._handle.work(maximum_actions=maximum_actions)

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
