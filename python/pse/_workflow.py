# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Typed declarations and owned handles over the single native computation pipeline."""

from collections.abc import Mapping, Sequence
from pathlib import Path

import attrs

from pse import codec
from pse._analyses import Analysis
from pse._build import (
    EngineSettings,
    _NativePhysicalContext,
    _NativeRuntime,
)
from pse._inspection import TableStream
from pse._modeling import ModelingPackage
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
    ResourceReport,
    ResultReclamationPage,
    RunControls,
    SolveSettings,
)
from pse.contracts.identities import (
    StudyId,
)
from pse.contracts.values import ContentHash, SemanticId

SolverCapability = result_contracts.RuntimeSolverCapabilitiesRow


@attrs.frozen(init=False)
class Runtime:
    """One native deployment budget over canonical scientific storage."""

    _handle: _NativeRuntime

    def __init__(
        self,
        settings: EngineSettings,
        *,
        substrate: str,
        producer: str | None = None,
        ephemeral: bool = False,
    ) -> None:
        """Attach to the shared deployment under an explicit durability class.

        Args:
            settings: The deployment budget shared by compilation, native work
                and reads.
            substrate: Supervisor state directory for canonical scientific storage.
            producer: Optional qualified mathematical producer receipt path.
            ephemeral: Explicitly retain numerical outcomes only in this process.
                Application execution durably records outcomes in canonical storage.
        """
        object.__setattr__(
            self,
            "_handle",
            _NativeRuntime(
                settings, substrate=substrate, producer=producer, ephemeral=ephemeral
            ),
        )

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

    def forget_study_results(self, study: str) -> None:
        """Withdraw a terminal study's explicit retention obligation."""
        self._handle.forget_study_results(study)

    def forget_analysis_results(self, analysis: str) -> None:
        """Withdraw derived analysis retention, preserving its lineage receipts."""
        self._handle.forget_analysis_results(analysis)

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
