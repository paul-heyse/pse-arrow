# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Passive diagnostic readers and explicitly owned capture sessions.

Standalone reads consume published checkpoints without constructing a Runtime or
connecting to scientific storage. Rust owns explanations and capture interpretation.
"""

# ruff: noqa: T201 -- the CLI renders requested diagnostic documents and errors.

from __future__ import annotations

import argparse
import math
import shlex
import sys
from pathlib import Path
from typing import TYPE_CHECKING, Self, TypeVar

import attrs
import pyarrow as pa
from msgspec import convert
from msgspec import json as msgspec_json
from msgspec.structs import replace

import pse
from pse import codec
from pse._build import (
    InspectionError,
    _diagnostic_analyze,
    _diagnostic_capture,
    _diagnostic_capture_arrow,
    _diagnostic_compare_invocations,
    _diagnostic_detail_arrow,
    _diagnostic_detail_records,
    _diagnostic_explain_invocation,
    _diagnostic_export_members,
    _diagnostic_export_pack,
    _diagnostic_extract_reproducer,
    _diagnostic_inspect,
    _diagnostic_inspect_frozen,
    _diagnostic_inspect_image,
    _diagnostic_inventory_from_evidence,
    _diagnostic_inventory_page,
    _diagnostic_launch_capture,
    _diagnostic_list,
    _diagnostic_operation_id,
    _diagnostic_operation_text,
    _diagnostic_pack_analysis,
    _diagnostic_pack_evidence,
    _diagnostic_pack_member,
    _diagnostic_read_checkpoint,
    _diagnostic_read_context,
    _diagnostic_read_detail,
    _diagnostic_read_evidence,
    _diagnostic_read_handoff,
    _diagnostic_read_invocation,
    _diagnostic_render_analysis,
    _diagnostic_review_arrow,
    _diagnostic_review_pack,
    _NativeCaptureSession,
    _NativeDetailSession,
    _NativeDiagnostics,
    _NativeLaunchedCommand,
    _NativeOperationObservations,
)
from pse._diagnostic_launch import record_launch as launch
from pse._diagnostic_query import run_query
from pse.contracts.documents import (
    AnalysisDocument,
    AnalysisDocumentOutput,
    AnalysisRequest,
    CaptureBudget,
    CaptureDocument,
    CaptureDocumentOutput,
    CaptureRequest,
    CheckpointIdentity,
    ClosedDuration,
    ComparisonPolicy,
    DetailDocumentOutput,
    DetailRecordPageOutput,
    DetailRequest,
    DiagnosticCommandRequest,
    DiagnosticListRequest,
    DiagnosticProcessPageOutput,
    DiagnosticRuntimeSettings,
    FrozenEvidenceSet,
    FrozenEvidenceSetOutput,
    FrozenInspectionDocumentOutput,
    FrozenInspectionRequest,
    InspectionDocument,
    InspectionDocumentOutput,
    InspectionRequest,
    InvocationComparisonDocumentOutput,
    InvocationEvidenceDocument,
    InvocationEvidenceDocumentOutput,
    LaunchCaptureRequest,
    NumericalHistoryPolicy,
    OperationId,
    OperationIdBody,
    OperationPageOutput,
    OperationSnapshotOutput,
    PackExportRequest,
    PackIndexOutput,
    ProcessCheckpoint,
    ProcessCheckpointOutput,
    ProcessLocatorOutput,
    ReductionCommandRequest,
    ReproducerCommandRequest,
    ReproducerCommandRequestOutput,
    ReproducerExtractionRequest,
    ReviewContextDocument,
    ReviewContextDocumentOutput,
    ReviewExportDeployment,
    ReviewHandoffReceiptOutput,
    ReviewInventoryPageOutput,
    ReviewInventoryRequest,
    ReviewInventorySelection,
    ReviewInventorySelectionOutput,
    ReviewQueryDocumentOutput,
    ReviewQueryRequest,
    ReviewQueryRequestOutput,
    ReviewTableDocument,
    ReviewTableRequest,
    SourceRecordingAcknowledgementOutput,
    SourceRecordingLaunchDocumentOutput,
    SourceRecordingLaunchRequest,
)
from pse.contracts.enums import CaptureKind, CaptureStatus

if TYPE_CHECKING:
    from collections.abc import Sequence
    from types import TracebackType

    from pse._runs import RunResult


@attrs.frozen
class OperationObservations:
    """Exact operation reader with no scientific cancellation or result lease."""

    _handle: _NativeOperationObservations

    @property
    def id(self) -> OperationId:
        """Process instance and sequence, independent from scientific identity."""
        return self._handle.id

    def snapshot(self, *, window: float = 60.0) -> OperationSnapshotOutput:
        """Read copied facts for the requested preceding seconds."""
        return self._handle.snapshot(window)

    def inspect(self, *, window: float = 60.0) -> InspectionDocumentOutput:
        """Explain copied facts; unavailable related owners remain explicit."""
        return self._handle.inspect(window)


@attrs.frozen
class Diagnostics:
    """Collection reader that retains no runtime caller or publisher registration."""

    _handle: _NativeDiagnostics

    @property
    def checkpoint(self) -> Path:
        """Exact publisher checkpoint locator; reading does not publish it."""
        return Path(self._handle.checkpoint)

    def list(self, *, offset: int = 0, limit: int = 16) -> OperationPageOutput:
        """Read a bounded page of known preparation, execution and terminal work."""
        return self._handle.list(offset=offset, limit=limit)

    def reader(self, operation: OperationId) -> OperationObservations | None:
        """Select an independent operation lease, or retain explicit absence."""
        reader = self._handle.reader(operation)
        return None if reader is None else OperationObservations(reader)

    def inspect(self, request: InspectionRequest) -> InspectionDocumentOutput:
        """Read the Rust-owned interpretation without invoking occupied owners."""
        return self._handle.inspect(request)

    def record_from_environment(
        self, supported_families: Sequence[str]
    ) -> DetailedCollection | None:
        """Select source recording before construction; retain activation separately."""
        session = self._handle.record_from_environment(tuple(supported_families))
        return None if session is None else DetailedCollection(session)

    def collect(self, request: DetailRequest) -> DetailedCollection:
        """Select finite detail before constructing work inside the returned context."""
        return DetailedCollection(self._handle.collect(request))


@attrs.frozen
class CaptureSession:
    """Own only optional capture workers; stopping leaves target science running."""

    _handle: _NativeCaptureSession

    def snapshot(self) -> CaptureDocumentOutput:
        """Observe current capture status, losses and flushed readable prefixes."""
        return self._handle.snapshot()

    def request_stop(self) -> None:
        """Request exporter stop without waiting or cancelling the target."""
        self._handle.request_stop()

    def wait(self) -> CaptureDocumentOutput:
        """Release Python execution while waiting and joining capture workers."""
        return self._handle.wait()

    def stop(self) -> CaptureDocumentOutput:
        """Stop and join only this capture, retaining its output files."""
        return self._handle.stop()

    def __enter__(self) -> Self:
        """Retain this explicitly owned capture until context exit."""
        return self

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc: BaseException | None,
        traceback: TracebackType | None,
    ) -> None:
        """Stop and join this capture when its context exits."""
        self.stop()


@attrs.frozen
class LaunchedCommand:
    """Own the launched command independently of its optional CPU sampler.

    Dropping or closing this owner cancels/reaps the command. Retain it while
    science should continue; a CaptureSession never retains this ownership.
    """

    _handle: _NativeLaunchedCommand

    @property
    def locator(self) -> ProcessLocatorOutput:
        """Exact process/start identity captured before scientific exec."""
        return self._handle.locator

    def try_wait(self) -> int | None:
        """Return the exit status when ready; otherwise leave execution running."""
        return self._handle.try_wait()

    def wait(self) -> int:
        """Wait/reap the command; Python interrupts explicitly cancel this owner."""
        return self._handle.wait()

    def cancel(self) -> int:
        """Cancel and drain only this owned command/process group."""
        return self._handle.cancel()

    def close(self) -> int:
        """Release command ownership, cancelling and reaping if still active."""
        return self._handle.close()

    def __enter__(self) -> Self:
        """Retain independent command ownership until context exit."""
        return self

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc: BaseException | None,
        traceback: TracebackType | None,
    ) -> None:
        """Drain the command when its owner context ends."""
        self.close()


def launch_capture(
    request: LaunchCaptureRequest,
) -> tuple[LaunchedCommand, CaptureSession]:
    """Enable sampling before exec; sampler failure still releases the command.

    The command owner must be retained independently. Capture duration, stop,
    drop and failures never cancel scientific execution.
    """
    command, session = _diagnostic_launch_capture(request)
    return LaunchedCommand(command), CaptureSession(session)


@attrs.frozen
class DetailedCollection:
    """Own a finite source collector; scientific work and cancellation stay separate.

    The entered construction scope is synchronous. Call ``leave()`` after starting
    work and before yielding or awaiting; constructed native/async owners retain
    their selected context until this collection finishes.
    """

    _handle: _NativeDetailSession

    @property
    def acknowledgement(self) -> SourceRecordingAcknowledgementOutput | None:
        """Actual cooperating admission, independent of terminal family coverage."""
        return self._handle.acknowledgement

    def retain_result(self, result: RunResult) -> ReviewHandoffReceiptOutput | None:
        """Publish requested joined context with failure separate from science."""
        return self._handle.retain_result(result._handle)  # noqa: SLF001 -- same native boundary

    def retain_context(
        self, context: ReviewContextDocument | ReviewContextDocumentOutput
    ) -> ReviewHandoffReceiptOutput | None:
        """Publish retained pre-handle facts; never reconstruct or run science."""
        return self._handle.retain_context(context)

    def snapshot(self) -> DetailDocumentOutput:
        """Read coverage independently from scientific completion."""
        return self._handle.snapshot()

    def finish(self) -> DetailDocumentOutput:
        """End the caller-owned interval and join only its writer."""
        return self._handle.finish()

    def stop(self) -> DetailDocumentOutput:
        """Stop without asserting complete interval coverage."""
        return self._handle.stop()

    def leave(self) -> None:
        """Leave before an await; retain this writer and already started work."""
        self._handle.leave()

    def __enter__(self) -> Self:
        """Propagate the selected context while constructing the actual invocation."""
        self._handle.enter()
        return self

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc: BaseException | None,
        traceback: TracebackType | None,
    ) -> None:
        """Finish the selected block interval; failures retain incomplete coverage."""
        if exc_type is None:
            self.finish()
        else:
            self.stop()


def read_detail(
    path: str | Path, *, maximum_bytes: int = 256 << 20
) -> DetailDocumentOutput:
    """Read a finite versioned manifest; an unterminated prefix stays incomplete."""
    return _diagnostic_read_detail(Path(path), maximum_bytes)


def detail_arrow(
    records: Path,
    *,
    offset: int = 0,
    limit: int = 16,
    maximum_bytes: int = 16 << 20,
    memory_bytes: int = 16 << 20,
) -> pa.RecordBatch:
    """Project one finite raw page with source event/field keys and leased buffers.

    Use detail_records for the next cursor and original producer declarations.
    The view does not imply a complete history or scientific permission.
    """
    return pa.record_batch(
        _diagnostic_detail_arrow(records, offset, limit, maximum_bytes, memory_bytes)
    )


def capture_arrow(
    document: CaptureDocument | CaptureDocumentOutput, *, memory_bytes: int = 16 << 20
) -> pa.RecordBatch:
    """Link the original optional capture document to its exact operation."""
    return pa.record_batch(_diagnostic_capture_arrow(document, memory_bytes))


def detail_records(
    path: str | Path,
    *,
    offset: int = 0,
    limit: int = 128,
    maximum_bytes: int = 16 << 20,
) -> DetailRecordPageOutput:
    """Read original coherent records using the returned byte cursor."""
    return _diagnostic_detail_records(Path(path), offset, limit, maximum_bytes)


def operation_id(text: str) -> OperationId:
    """Parse Rust's exact 32-hex-process:decimal-sequence diagnostic locator."""
    return _diagnostic_operation_id(text)


def operation_text(operation: OperationId | OperationIdBody) -> str:
    """Render the diagnostic locator using its Rust owner."""
    if isinstance(operation, OperationIdBody):
        operation = OperationId(
            process_instance=operation.process_instance, sequence=operation.sequence
        )
    return _diagnostic_operation_text(operation)


def list_processes(
    request: DiagnosticListRequest | None = None,
    *,
    directory: str | Path | None = None,
) -> DiagnosticProcessPageOutput:
    """Read one bounded process page; initialize neither runtime nor storage."""
    if request is None:
        request = DiagnosticListRequest(after=None, limit=16)
    return _diagnostic_list(request, None if directory is None else Path(directory))


def inspect_checkpoint(
    checkpoint: str | Path, request: InspectionRequest
) -> InspectionDocumentOutput:
    """Explain an exact published prefix through the same Rust inspection owner."""
    return _diagnostic_inspect(Path(checkpoint), request)


def read_checkpoint(
    checkpoint: str | Path, *, expected: CheckpointIdentity | None = None
) -> ProcessCheckpointOutput:
    """Decode one bounded prefix and validate its exact registered association."""
    return _diagnostic_read_checkpoint(Path(checkpoint), expected)


def inspect_image(
    image: ProcessCheckpoint | ProcessCheckpointOutput,
    request: InspectionRequest,
    *,
    checkpoint: str | Path,
) -> InspectionDocumentOutput:
    """Inspect the already validated image, avoiding a second path read."""
    return _diagnostic_inspect_image(Path(checkpoint), image, request)


def capture(request: CaptureRequest) -> CaptureSession:
    """Start explicitly selected capture; omission uses Rust's request defaults."""
    return CaptureSession(_diagnostic_capture(request))


def _seconds(value: str) -> float:
    parsed = float(value)
    if not math.isfinite(parsed) or parsed < 0:
        message = "seconds must be finite and nonnegative"
        raise argparse.ArgumentTypeError(message)
    return parsed


def _bounded_count(value: str) -> int:
    parsed = int(value)
    if not 1 <= parsed <= 32:
        message = "count must be between 1 and 32"
        raise argparse.ArgumentTypeError(message)
    return parsed


def _positive(value: str) -> int:
    parsed = int(value)
    if not 1 <= parsed <= (1 << 63) - 1:
        message = "allowance must be a positive signed 64-bit integer"
        raise argparse.ArgumentTypeError(message)
    return parsed


def _duration(seconds: float) -> ClosedDuration:
    total = round(seconds * 1_000_000_000)
    secs, nanos = divmod(total, 1_000_000_000)
    return ClosedDuration(secs=secs, nanos=nanos)


def _render_inspection(document: InspectionDocument | InspectionDocumentOutput) -> None:
    print(f"operation {operation_text(document.request.operation)}")
    operation = document.operation
    if operation is not None:
        print(f"  kind={operation.kind} outcome={operation.outcome}")
        print(
            f"  read_at={operation.read_at!r} "
            f"observed_at={operation.observed_at!r} "
            f"stale_after={operation.stale_after!r}"
        )
        print(
            f"  requested_interval={operation.requested_interval!r} "
            f"actual_interval={operation.interval_start!r}..{operation.interval_end!r} "
            f"interval_partial={operation.interval_partial}"
        )
        print(
            f"  coverage={operation.coverage!r} "
            f"detail_partial={operation.detail_partial}"
        )
        for phase in operation.phases:
            print(
                f"  phase token={phase.token} kind={phase.kind} "
                f"started_at={phase.started_at!r}"
            )
    print(f"transport {codec.encode_json(document.transport).decode()}")
    for statement in document.statements:
        print(f"{statement.kind} [{statement.code}] {statement.message}")
        for evidence in statement.evidence:
            print(f"  evidence {codec.encode_json(evidence).decode()}")
        if statement.next:
            print(f"  next {', '.join(statement.next)}")
    if operation is not None:
        for metric in operation.metrics:
            print(
                f"metric {metric.name} unit={metric.unit} "
                f"observed_at={metric.observed_at!r}"
            )
            print(
                f"  latest={codec.encode_json(metric.latest).decode()} "
                f"cumulative={codec.encode_json(metric.cumulative).decode()} "
                f"recent={codec.encode_json(metric.recent).decode()}"
            )
            print(
                f"  generation={metric.generation} resets={metric.resets} "
                f"cumulative_partial={metric.cumulative_partial} "
                f"recent_partial={metric.recent_partial} "
                f"history_partial={metric.history_partial} "
                f"history_dropped={metric.history_dropped}"
            )
    if document.operation is not None:
        for producer in document.operation.producers:
            print(
                f"producer {producer.descriptor.id.sequence} "
                f"role={producer.descriptor.role} "
                f"coverage={codec.encode_json(producer.coverage).decode()}"
            )
            if producer.current is not None:
                print(f"  current={codec.encode_json(producer.current).decode()}")
    for related in document.related:
        print(
            f"related {operation_text(related.id)} kind={related.kind} "
            f"outcome={related.outcome}"
        )
        for phase in related.phases:
            print(
                f"  phase token={phase.token} kind={phase.kind} "
                f"started_at={phase.started_at!r}"
            )
    print(f"omitted {document.omitted}")


def main(argv: Sequence[str] | None = None) -> int:
    """Run bounded diagnostic reads/capture against already published targets."""
    parser = argparse.ArgumentParser(prog="just diagnostics")
    commands = parser.add_subparsers(dest="command", required=True)
    listing = commands.add_parser("list", help="discover published process prefixes")
    listing.add_argument("--directory", "--dir", type=Path)
    listing.add_argument("--after")
    listing.add_argument("--limit", type=_bounded_count, default=16)
    listing.add_argument("--json", action="store_true")
    inspecting = commands.add_parser(
        "inspect", help="explain an exact published operation"
    )
    inspecting.add_argument("checkpoint", type=Path)
    inspecting.add_argument("operation")
    inspecting.add_argument("--window", type=_seconds, default=60.0)
    inspecting.add_argument("--related-limit", type=_bounded_count, default=16)
    inspecting.add_argument("--no-related", action="store_true")
    inspecting.add_argument("--json", action="store_true")
    inspecting.add_argument(
        "--numerical-policy",
        type=Path,
        help="explicit versioned analytical policy JSON",
    )
    capturing = commands.add_parser(
        "capture", help="start explicit bounded optional capture"
    )
    capturing.add_argument("checkpoint", type=Path)
    capturing.add_argument("operation")
    capturing.add_argument("--kind", choices=("cpu", "timeline"), default="timeline")
    capturing.add_argument("--output", type=Path, required=True)
    capturing.add_argument("--duration", type=_seconds, default=10.0)
    capturing.add_argument("--memory-bytes", type=_positive, default=16 << 20)
    capturing.add_argument("--output-bytes", type=_positive, default=256 << 20)
    capturing.add_argument("--json", action="store_true")
    launching = commands.add_parser(
        "capture-launch", help="sample an owned command before exec"
    )
    launching.add_argument("--output", type=Path, required=True)
    launching.add_argument("--cwd", type=Path)
    launching.add_argument("--duration", type=_seconds, default=10.0)
    launching.add_argument("--startup-timeout", type=_seconds, default=10.0)
    launching.add_argument("--memory-bytes", type=_positive, default=16 << 20)
    launching.add_argument("--output-bytes", type=_positive, default=256 << 20)
    launching.add_argument("--json", action="store_true")
    launching.add_argument("argv", nargs=argparse.REMAINDER)
    detail = commands.add_parser(
        "read-detail", help="read selected source coverage and original records"
    )
    detail.add_argument("records", type=Path)
    detail.add_argument("--offset", type=int, default=0)
    detail.add_argument("--limit", type=_bounded_count, default=16)
    detail.add_argument("--maximum-bytes", type=_positive, default=16 << 20)
    detail.add_argument("--json", action="store_true")
    _add_review_commands(commands)
    args = parser.parse_args(argv)
    try:
        if args.command in _REVIEW_COMMANDS:
            return _review_command(args)
        if args.command == "read-detail":
            page = detail_records(
                args.records,
                offset=args.offset,
                limit=args.limit,
                maximum_bytes=args.maximum_bytes,
            )
            print(codec.encode_json(page).decode())
            return 0
        if args.command == "list":
            page = list_processes(
                DiagnosticListRequest(after=args.after, limit=args.limit),
                directory=args.directory,
            )
            if args.json:
                print(codec.encode_json(page).decode())
            else:
                for process in page.processes:
                    print(f"checkpoint {process.checkpoint} partial={process.partial}")
                    if process.failure:
                        print(f"  unavailable {process.failure}")
                    for operation in process.operations:
                        print(
                            f"  {operation_text(operation.id)} {operation.kind} "
                            f"outcome={operation.outcome}"
                        )
                if page.next is not None:
                    print(f"next {page.next}")
            return 0
        if args.command == "capture-launch":
            command_argv = args.argv[1:] if args.argv[:1] == ["--"] else args.argv
            if not command_argv:
                parser.error("capture-launch requires -- executable [arguments]")
            request = LaunchCaptureRequest(
                version=1,
                argv=tuple(command_argv),
                cwd=None if args.cwd is None else str(args.cwd),
                output=str(args.output),
                duration=_duration(args.duration),
                startup_timeout=_duration(args.startup_timeout),
                budget=CaptureBudget(
                    memory_bytes=args.memory_bytes, output_bytes=args.output_bytes
                ),
            )
            command, session = launch_capture(request)
            try:
                # Retain the command even if capture finishes early or never starts.
                status = command.wait()
                result = session.stop()
            except BaseException:
                try:
                    command.cancel()
                finally:
                    session.stop()
                raise
            finally:
                command.close()
            print(codec.encode_json(result).decode())
            return status or (1 if result.status == CaptureStatus.FAILED else 0)
        selected = operation_id(args.operation)
        if args.command == "inspect":
            document = inspect_checkpoint(
                args.checkpoint,
                InspectionRequest(
                    version=2,
                    operation=convert(
                        selected, type=OperationIdBody, from_attributes=True
                    ),
                    window=_duration(args.window),
                    include_related=not args.no_related,
                    related_limit=args.related_limit,
                    numerical_history_policy=None
                    if args.numerical_policy is None
                    else codec.decode_json(
                        args.numerical_policy.read_bytes(), NumericalHistoryPolicy
                    ),
                ),
            )
            if args.json:
                print(codec.encode_json(document).decode())
            else:
                _render_inspection(document)
            return 0
        if args.duration == 0:
            parser.error("capture duration must be positive")
        if args.memory_bytes < 4 << 20:
            parser.error("capture needs at least 4194304 memory bytes")
        request = CaptureRequest(
            version=1,
            checkpoint=str(args.checkpoint),
            operation=convert(selected, type=OperationIdBody, from_attributes=True),
            kind=CaptureKind.CPU if args.kind == "cpu" else CaptureKind.TIMELINE,
            output=str(args.output),
            duration=_duration(args.duration),
            budget=CaptureBudget(
                memory_bytes=args.memory_bytes, output_bytes=args.output_bytes
            ),
        )
        with capture(request) as session:
            result = session.wait()
        print(codec.encode_json(result).decode())
    except (InspectionError, OSError, ValueError, OverflowError) as error:
        print(f"diagnostics unavailable: {error}", file=sys.stderr)
        return 1
    else:
        return 1 if result.status == CaptureStatus.FAILED else 0


def inspect_frozen(
    evidence: FrozenEvidenceSet | FrozenEvidenceSetOutput,
    request: FrozenInspectionRequest,
) -> FrozenInspectionDocumentOutput:
    """Interpret only supplied immutable members at recorded source time."""
    return _diagnostic_inspect_frozen(evidence, request)


def read_evidence(path: str | Path) -> FrozenEvidenceSetOutput:
    """Admit a bounded saved set; embedded paths never authorize lookup."""
    return _diagnostic_read_evidence(Path(path))


def read_invocation(path: str | Path) -> InvocationEvidenceDocumentOutput:
    """Read descriptive invocation facts without a live result handle."""
    return _diagnostic_read_invocation(Path(path))


def read_context(path: str | Path) -> ReviewContextDocumentOutput:
    """Read original descriptive context without Replay admission."""
    return _diagnostic_read_context(Path(path))


def explain_invocation(
    source: InvocationEvidenceDocument | InvocationEvidenceDocumentOutput,
    *,
    policy: ComparisonPolicy | None = None,
) -> InvocationComparisonDocumentOutput:
    """Explain saved facts using the same pure owner as live invocation readback."""
    return _diagnostic_explain_invocation(source, policy)


def compare_invocations(
    left: InvocationEvidenceDocument | InvocationEvidenceDocumentOutput,
    right: InvocationEvidenceDocument | InvocationEvidenceDocumentOutput,
    *,
    policy: ComparisonPolicy | None = None,
) -> InvocationComparisonDocumentOutput:
    """Compare supplied facts without rerunning science or choosing a winner."""
    return _diagnostic_compare_invocations(left, right, policy)


def inventory_page(
    selection: ReviewInventorySelection | ReviewInventorySelectionOutput,
    *,
    offset: int = 0,
    limit: int = 64,
) -> ReviewInventoryPageOutput:
    """Page an immutable selection without consulting current occurrence pointers."""
    return _diagnostic_inventory_page(selection, offset, limit)


def inventory_from_evidence(
    evidence: FrozenEvidenceSet | FrozenEvidenceSetOutput,
    request: ReviewInventoryRequest,
    *,
    scope: str = "supplied.operations",
) -> ReviewInventorySelectionOutput:
    """Project actual copied operation/producer relationships without live discovery."""
    return _diagnostic_inventory_from_evidence(evidence, request, scope)


_REVIEW_COMMANDS = {
    "inspect-recorded",
    "analyze",
    "compare",
    "context",
    "inventory",
    "query",
    "record-launch",
    "export",
    "review",
    "diagnose",
    "extract-reproducer",
    "reproduce",
    "reduce",
}


def _add_review_commands(commands: argparse._SubParsersAction) -> None:
    recorded = commands.add_parser(
        "inspect-recorded",
        help="READ supplied file/pack at recorded source time; no live lookup",
    )
    recorded.add_argument("subject", type=Path)
    recorded.add_argument(
        "--request", required=True, type=Path, help="FrozenInspectionRequest JSON"
    )
    recorded.add_argument("--json", action="store_true")
    recorded.add_argument(
        "--source", choices=("recorded", "live", "handoff"), default="recorded"
    )
    recorded.add_argument("--handoff", type=Path)
    analysis = commands.add_parser(
        "analyze", help="READ ten source-linked questions over supplied file/pack"
    )
    analysis.add_argument("subject", type=Path)
    analysis.add_argument(
        "--request", required=True, type=Path, help="AnalysisRequest JSON"
    )
    analysis.add_argument("--json", action="store_true")
    analysis.add_argument(
        "--source", choices=("recorded", "live", "handoff"), default="recorded"
    )
    analysis.add_argument("--handoff", type=Path)
    analysis.add_argument("--memory-bytes", type=_positive, default=64 << 20)
    comparison = commands.add_parser(
        "compare", help="READ and compare two saved invocation documents"
    )
    comparison.add_argument("left", type=Path)
    comparison.add_argument("right", type=Path)
    comparison.add_argument("--policy", type=Path)
    comparison.add_argument("--json", action="store_true")
    for name in ("context", "inventory"):
        selected = commands.add_parser(
            name, help=f"READ selected {name} from supplied file/pack only"
        )
        selected.add_argument("subject", type=Path)
        selected.add_argument("--index", type=int, default=0)
        selected.add_argument("--offset", type=int, default=0)
        selected.add_argument("--limit", type=_bounded_count, default=16)
        selected.add_argument("--json", action="store_true")
        selected.add_argument(
            "--source", choices=("recorded", "live", "handoff"), default="recorded"
        )
        selected.add_argument("--handoff", type=Path)
    querying = commands.add_parser(
        "query",
        help=(
            "READ a supplied trace with a declared bounded optional "
            "tool query; never install"
        ),
    )
    querying.add_argument("trace", type=Path)
    querying.add_argument(
        "--request", required=True, type=Path, help="ReviewQueryRequest JSON"
    )
    querying.add_argument(
        "--binary",
        type=Path,
        help="explicit optional trace_processor_shell for trace queries",
    )
    querying.add_argument(
        "--table",
        action="store_true",
        help=(
            "READ one registry table from supplied frozen file/pack; "
            "request is ReviewTableRequest"
        ),
    )
    querying.add_argument("--output", required=True, type=Path)
    querying.add_argument("--json", action="store_true")
    launch = commands.add_parser(
        "record-launch",
        help=(
            "RECORD explicit literal argv with preconstruction "
            "request/ACK; command ownership is separate"
        ),
    )
    launch.add_argument(
        "--request",
        required=True,
        type=Path,
        help=(
            "SourceRecordingLaunchRequest JSON (includes literal argv "
            "and finite clocks)"
        ),
    )
    launch.add_argument("--json", action="store_true")
    exporting = commands.add_parser(
        "export",
        help=(
            "EXPORT selected immutable facts and explicit readable "
            "file prefixes to a new offline pack"
        ),
    )
    exporting.add_argument("subject", type=Path)
    exporting.add_argument(
        "--analysis",
        required=True,
        type=Path,
        help="AnalysisRequest JSON evaluated by the shared owner",
    )
    exporting.add_argument(
        "--request", required=True, type=Path, help="PackExportRequest JSON"
    )
    exporting.add_argument("--output", required=True, type=Path)
    exporting.add_argument("--json", action="store_true")
    exporting.add_argument(
        "--source", choices=("recorded", "live", "handoff"), default="recorded"
    )
    exporting.add_argument("--handoff", type=Path)
    exporting.add_argument(
        "--deployment",
        type=Path,
        help=(
            "Explicit existing ReviewExportDeployment; required for "
            "scientific selectors"
        ),
    )
    exporting.add_argument("--memory-bytes", type=_positive, default=64 << 20)
    reviewing = commands.add_parser(
        "review", help="READ only indexed contained members of an offline pack"
    )
    reviewing.add_argument("subject", type=Path)
    reviewing.add_argument("--json", action="store_true")
    extracting = commands.add_parser(
        "extract-reproducer",
        help="OFFLINE transformation of recorded text or frozen source codecs",
    )
    extracting.add_argument(
        "--request",
        required=True,
        type=Path,
        help="ReproducerExtractionRequest JSON; no Runtime is constructed",
    )
    extracting.add_argument(
        "--output",
        required=True,
        type=Path,
        help="New readable ReproducerCommandRequest driver",
    )
    reproducing = commands.add_parser(
        "reproduce",
        help="NEW WORK: reconstruct, admit and execute an extracted reproducer",
    )
    reproducing.add_argument(
        "--request",
        required=True,
        type=Path,
        help="Extracted ReproducerCommandRequest JSON",
    )
    reproducing.add_argument("--json", action="store_true")
    reducing = commands.add_parser(
        "reduce", help="NEW WORK: reduce a separately selected scientific issue"
    )
    reducing.add_argument(
        "--request", required=True, type=Path, help="ReductionCommandRequest JSON"
    )
    reducing.add_argument("--json", action="store_true")
    diagnosing = commands.add_parser(
        "diagnose",
        help=(
            "NEW NUMERICAL WORK from explicit original model/physical "
            "documents under one finite deadline"
        ),
    )
    diagnosing.add_argument(
        "--request",
        required=True,
        type=Path,
        help=(
            "DiagnosticCommandRequest JSON; descriptive context cannot "
            "reconstruct a model"
        ),
    )
    diagnosing.add_argument("--json", action="store_true")


_Request = TypeVar("_Request")


def _read_review_request(path: Path, kind: type[_Request]) -> _Request:
    with path.open("rb") as source:
        encoded = source.read((64 << 20) + 1)
    if len(encoded) > 64 << 20:
        message = "review document exceeds 64 MiB"
        raise ValueError(message)
    return codec.decode_json(encoded, kind)


def _subject(
    path: Path, *, source: str = "recorded", handoff: Path | None = None
) -> FrozenEvidenceSetOutput:
    if source == "handoff":
        evidence = read_handoff(path)
    elif source == "live":
        evidence = FrozenEvidenceSetOutput(
            version=3, checkpoints=(read_checkpoint(path),)
        )
    else:
        evidence = (
            _diagnostic_pack_evidence(path)
            if path.is_dir()
            else _diagnostic_read_evidence(path)
        )
    if handoff is None:
        return evidence

    retained = read_handoff(handoff)
    selected = {operation.id for operation in evidence.operations}
    selected.update(
        operation.id for image in evidence.checkpoints for operation in image.operations
    )
    # The explicit primary snapshot determines current facts. Retained context never
    # substitutes another snapshot for that chosen live publication.
    return replace(
        evidence,
        operations=(
            *evidence.operations,
            *(
                operation
                for operation in retained.operations
                if operation.id not in selected
            ),
        ),
        invocations=(*evidence.invocations, *retained.invocations),
        contexts=(*evidence.contexts, *retained.contexts),
        inventories=(*evidence.inventories, *retained.inventories),
        engine_observations=(
            *evidence.engine_observations,
            *retained.engine_observations,
        ),
    )


def analyze_evidence(
    evidence: FrozenEvidenceSet | FrozenEvidenceSetOutput,
    request: AnalysisRequest,
    *,
    memory_bytes: int = 64 << 20,
) -> AnalysisDocumentOutput:
    """Interpret immutable facts under one cancellable finite working allocation."""
    return _diagnostic_analyze(evidence, request, memory_bytes=memory_bytes)


def read_handoff(path: str | Path) -> FrozenEvidenceSetOutput:
    """Read an explicit receipt's finalized contained context members."""
    return _diagnostic_read_handoff(Path(path))


def export_review_pack(
    destination: str | Path,
    evidence: FrozenEvidenceSet | FrozenEvidenceSetOutput,
    analysis: AnalysisDocument | AnalysisDocumentOutput,
    request: PackExportRequest,
) -> PackIndexOutput:
    """Publish selected facts and prefixes from explicit original input paths."""
    return _diagnostic_export_pack(Path(destination), evidence, analysis, request)


def review_pack(directory: str | Path) -> PackIndexOutput:
    """Validate an indexed local directory without following original paths."""
    return _diagnostic_review_pack(Path(directory))


def export_member_pack(
    destination: str | Path, request: PackExportRequest
) -> PackIndexOutput:
    """Copy supplied non-scientific members through native finite pack custody."""
    return _diagnostic_export_members(Path(destination), request)


def pack_member(
    directory: str | Path, member: str, *, maximum_bytes: int = 1 << 20
) -> bytes:
    """Read only the original indexed acknowledged member under a finite allowance."""
    return _diagnostic_pack_member(Path(directory), member, maximum_bytes)


def review_arrow(
    evidence: FrozenEvidenceSet | FrozenEvidenceSetOutput, request: ReviewTableRequest
) -> pa.RecordBatch:
    """Select one registry table and retain its finite buffer ownership."""
    return pa.record_batch(_diagnostic_review_arrow(evidence, request))


def query_trace(
    request: ReviewQueryRequest | ReviewQueryRequestOutput,
    trace: Path,
    output: Path,
    binary: Path,
) -> ReviewQueryDocumentOutput:
    """Run a declared optional query, retaining failure prefixes and context."""
    return run_query(request, trace, output, binary)


def record_launch(
    request: SourceRecordingLaunchRequest,
) -> SourceRecordingLaunchDocumentOutput:
    """Own one literal command and report actual cooperating-source acknowledgement."""
    return launch(request)


def extract_reproducer(
    request: ReproducerExtractionRequest,
) -> ReproducerCommandRequestOutput:
    """Transform recorded codecs without constructing a runtime or running science."""
    return _diagnostic_extract_reproducer(request)


def _diagnostic_runtime(settings: DiagnosticRuntimeSettings) -> pse.Runtime:
    return pse.Runtime(
        pse.EngineSettings(
            memory_limit_bytes=settings.memory_limit_bytes,
            threads=settings.threads,
            spill_dir=settings.spill_dir,
            max_spill_bytes=settings.max_spill_bytes,
            batch_size=settings.batch_size,
        ),
        ephemeral=True,
    )


def _review_command(args: argparse.Namespace) -> int:
    if args.command == "compare":
        policy = (
            None
            if args.policy is None
            else _read_review_request(args.policy, ComparisonPolicy)
        )
        result = compare_invocations(
            read_invocation(args.left), read_invocation(args.right), policy=policy
        )
    elif args.command == "query":
        if args.table:
            request = _read_review_request(args.request, ReviewTableRequest)
            batch = review_arrow(_subject(args.trace), request)
            args.output.mkdir(parents=True, exist_ok=False)
            with (
                (args.output / "rows.arrow").open("xb") as output,
                pa.ipc.new_stream(output, batch.schema) as writer,
            ):
                writer.write_batch(batch)
            (args.output / "request.json").write_bytes(codec.encode_json(request))
            result = ReviewTableDocument(
                version=1,
                request=request,
                input_member=str(args.trace),
                output="rows.arrow",
                rows=batch.num_rows,
                schema_metadata={
                    key.decode(): value.decode()
                    for key, value in (batch.schema.metadata or {}).items()
                },
            )
            (args.output / "table-result.json").write_bytes(codec.encode_json(result))
            print(codec.encode_json(result).decode())
            return 0
        if args.binary is None:
            message = (
                "trace query requires explicit --binary; no implicit "
                "installation or tool discovery"
            )
            raise ValueError(message)
        result = query_trace(
            _read_review_request(args.request, ReviewQueryRequest),
            args.trace,
            args.output,
            args.binary,
        )
        print(codec.encode_json(result).decode())
        return 0 if result.status == "completed" else 1
    elif args.command == "record-launch":
        result = record_launch(
            _read_review_request(args.request, SourceRecordingLaunchRequest)
        )
        print(codec.encode_json(result).decode())
        return result.command_status or (
            0
            if result.drained
            and result.acknowledgement is not None
            and result.acknowledgement.accepted
            else 1
        )
    elif args.command == "extract-reproducer":
        result = extract_reproducer(
            _read_review_request(args.request, ReproducerExtractionRequest)
        )
        with args.output.open("xb") as destination:
            destination.write(msgspec_json.format(codec.encode_json(result)))
        print(
            f"Extracted {args.output}; required environment: "
            f"{result.reproducer.required_environment}"
        )
        print(
            "Explicit new run: python -m pse.diagnostics reproduce --request "
            f"{shlex.quote(str(args.output))} --json"
        )
        return 0
    elif args.command == "reproduce":
        command = _read_review_request(args.request, ReproducerCommandRequest)
        runtime = _diagnostic_runtime(command.runtime)
        try:
            reproduction = runtime.run_reproducer(command.reproducer)
        finally:
            runtime.close()
        print(codec.encode_json(reproduction).decode())
        reproduced = reproduction.status in (
            "original_failure_reproduced",
            "related_diagnostic_reproduced",
        )
        return 0 if reproduced else 1
    elif args.command == "reduce":
        command = _read_review_request(args.request, ReductionCommandRequest)
        runtime = _diagnostic_runtime(command.runtime)
        try:
            result = runtime.reduce_diagnostic(command.reduction)
        finally:
            runtime.close()
        print(codec.encode_json(result).decode())
        return (
            0
            if result.attempts and result.attempts[0].disposition == "preserved"
            else 1
        )
    elif args.command == "diagnose":
        command = _read_review_request(args.request, DiagnosticCommandRequest)
        runtime = _diagnostic_runtime(command.runtime)
        try:
            result = runtime.diagnose_documents(command.operation)
        finally:
            runtime.close()
        print(codec.encode_json(result).decode())
        return 0 if result.failure is None else 1
    elif args.command == "review":
        result = review_pack(args.subject)
        if not args.json:
            print(
                f"publication={result.publication} "
                f"membership_complete={result.membership_complete} "
                f"self_contained={result.self_contained}"
            )
            print(_diagnostic_render_analysis(_diagnostic_pack_analysis(args.subject)))
            return 0
    else:
        evidence = _subject(args.subject, source=args.source, handoff=args.handoff)
        if args.command == "inspect-recorded":
            result = inspect_frozen(
                evidence, _read_review_request(args.request, FrozenInspectionRequest)
            )
            if not args.json:
                _render_inspection(result.inspection)
                for limitation in result.limitations:
                    print(f"limitation {limitation}")
                return 0
        elif args.command == "analyze":
            result = analyze_evidence(
                evidence,
                _read_review_request(args.request, AnalysisRequest),
                memory_bytes=args.memory_bytes,
            )
            if not args.json:
                print(_diagnostic_render_analysis(result))
                return 0
        elif args.command == "export":
            analysis = analyze_evidence(
                evidence,
                _read_review_request(args.analysis, AnalysisRequest),
                memory_bytes=args.memory_bytes,
            )
            request = _read_review_request(args.request, PackExportRequest)
            if request.scientific_results:
                if args.deployment is None:
                    message = (
                        "scientific export requires an explicit existing deployment"
                    )
                    raise ValueError(message)

                deployment = _read_review_request(
                    args.deployment, ReviewExportDeployment
                )
                settings = deployment.runtime
                runtime = pse.Runtime(
                    pse.EngineSettings(
                        memory_limit_bytes=settings.memory_limit_bytes,
                        threads=settings.threads,
                        spill_dir=settings.spill_dir,
                        max_spill_bytes=settings.max_spill_bytes,
                        batch_size=settings.batch_size,
                    ),
                    substrate=deployment.substrate,
                    database=deployment.database,
                    producer=deployment.producer,
                )
                try:
                    result = runtime.export_review_pack(
                        args.output, evidence, analysis, request
                    )
                finally:
                    runtime.close()
            else:
                result = export_review_pack(args.output, evidence, analysis, request)
        elif args.command == "context":
            if not 0 <= args.index < len(evidence.contexts):
                message = (
                    "selected context is unavailable; no live source lookup attempted"
                )
                raise ValueError(message)
            result = evidence.contexts[args.index]
        else:
            if not 0 <= args.index < len(evidence.inventories):
                message = (
                    "selected frozen membership is unavailable; no "
                    "live inventory substituted"
                )
                raise ValueError(message)
            result = inventory_page(
                evidence.inventories[args.index], offset=args.offset, limit=args.limit
            )
    print(codec.encode_json(result).decode())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

