# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Passive diagnostic readers and explicitly owned capture sessions.

Standalone reads consume published checkpoints without constructing a Runtime or
connecting to scientific storage. Rust owns explanations and capture interpretation.
"""

from __future__ import annotations

# ruff: noqa: T201 -- the CLI renders requested diagnostic documents and errors.
import argparse
import math
import sys
from pathlib import Path
from typing import TYPE_CHECKING, Self

import attrs
import pyarrow as pa

from pse import codec
from pse._build import (
    InspectionError,
    _diagnostic_capture,
    _diagnostic_capture_arrow,
    _diagnostic_detail_arrow,
    _diagnostic_detail_records,
    _diagnostic_inspect,
    _diagnostic_inspect_image,
    _diagnostic_launch_capture,
    _diagnostic_list,
    _diagnostic_operation_id,
    _diagnostic_operation_text,
    _diagnostic_read_checkpoint,
    _diagnostic_read_detail,
    _NativeCaptureSession,
    _NativeDetailSession,
    _NativeDiagnostics,
    _NativeLaunchedCommand,
    _NativeOperationObservations,
)
from pse.contracts.documents import (
    CaptureBudget,
    CaptureDocument,
    CaptureRequest,
    CheckpointIdentity,
    ClosedDuration,
    DetailDocument,
    DetailRecordPage,
    DetailRequest,
    DiagnosticListRequest,
    DiagnosticProcessPage,
    InspectionDocument,
    InspectionRequest,
    LaunchCaptureRequest,
    NumericalHistoryPolicy,
    OperationId,
    OperationPage,
    OperationSnapshot,
    ProcessCheckpoint,
    ProcessLocator,
)
from pse.contracts.enums import CaptureKind, CaptureStatus

if TYPE_CHECKING:
    from collections.abc import Sequence
    from types import TracebackType


@attrs.frozen
class OperationObservations:
    """Exact operation reader with no scientific cancellation or result lease."""

    _handle: _NativeOperationObservations

    @property
    def id(self) -> OperationId:
        """Process instance and sequence, independent from scientific identity."""
        return self._handle.id

    def snapshot(self, *, window: float = 60.0) -> OperationSnapshot:
        """Read copied facts for the requested preceding seconds."""
        return self._handle.snapshot(window)

    def inspect(self, *, window: float = 60.0) -> InspectionDocument:
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

    def list(self, *, offset: int = 0, limit: int = 16) -> OperationPage:
        """Read a bounded page of known preparation, execution and terminal work."""
        return self._handle.list(offset=offset, limit=limit)

    def reader(self, operation: OperationId) -> OperationObservations | None:
        """Select an independent operation lease, or retain explicit absence."""
        reader = self._handle.reader(operation)
        return None if reader is None else OperationObservations(reader)

    def inspect(self, request: InspectionRequest) -> InspectionDocument:
        """Read the Rust-owned interpretation without invoking occupied owners."""
        return self._handle.inspect(request)

    def collect(self, request: DetailRequest) -> DetailedCollection:
        """Select finite detail before constructing work inside the returned context."""
        return DetailedCollection(self._handle.collect(request))


@attrs.frozen
class CaptureSession:
    """Own only optional capture workers; stopping leaves target science running."""

    _handle: _NativeCaptureSession

    def snapshot(self) -> CaptureDocument:
        """Observe current capture status, losses and flushed readable prefixes."""
        return self._handle.snapshot()

    def request_stop(self) -> None:
        """Request exporter stop without waiting or cancelling the target."""
        self._handle.request_stop()

    def wait(self) -> CaptureDocument:
        """Release Python execution while waiting and joining capture workers."""
        return self._handle.wait()

    def stop(self) -> CaptureDocument:
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
    def locator(self) -> ProcessLocator:
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

    def snapshot(self) -> DetailDocument:
        """Read coverage independently from scientific completion."""
        return self._handle.snapshot()

    def finish(self) -> DetailDocument:
        """End the caller-owned interval and join only its writer."""
        return self._handle.finish()

    def stop(self) -> DetailDocument:
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


def read_detail(path: str | Path, *, maximum_bytes: int = 256 << 20) -> DetailDocument:
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
    document: CaptureDocument, *, memory_bytes: int = 16 << 20
) -> pa.RecordBatch:
    """Link the original optional capture document to its exact operation."""
    return pa.record_batch(_diagnostic_capture_arrow(document, memory_bytes))


def detail_records(
    path: str | Path,
    *,
    offset: int = 0,
    limit: int = 128,
    maximum_bytes: int = 16 << 20,
) -> DetailRecordPage:
    """Read original coherent records using the returned byte cursor."""
    return _diagnostic_detail_records(Path(path), offset, limit, maximum_bytes)


def operation_id(text: str) -> OperationId:
    """Parse Rust's exact 32-hex-process:decimal-sequence diagnostic locator."""
    return _diagnostic_operation_id(text)


def operation_text(operation: OperationId) -> str:
    """Render the diagnostic locator using its Rust owner."""
    return _diagnostic_operation_text(operation)


def list_processes(
    request: DiagnosticListRequest | None = None,
    *,
    directory: str | Path | None = None,
) -> DiagnosticProcessPage:
    """Read one bounded process page; initialize neither runtime nor storage."""
    if request is None:
        request = DiagnosticListRequest(after=None, limit=16)
    return _diagnostic_list(request, None if directory is None else Path(directory))


def inspect_checkpoint(
    checkpoint: str | Path, request: InspectionRequest
) -> InspectionDocument:
    """Explain an exact published prefix through the same Rust inspection owner."""
    return _diagnostic_inspect(Path(checkpoint), request)


def read_checkpoint(
    checkpoint: str | Path, *, expected: CheckpointIdentity | None = None
) -> ProcessCheckpoint:
    """Decode one bounded prefix and validate its exact registered association."""
    return _diagnostic_read_checkpoint(Path(checkpoint), expected)


def inspect_image(
    image: ProcessCheckpoint, request: InspectionRequest, *, checkpoint: str | Path
) -> InspectionDocument:
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


def _render_inspection(document: InspectionDocument) -> None:
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
    args = parser.parse_args(argv)
    try:
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
                argv=command_argv,
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
                    operation=selected,
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
            operation=selected,
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


if __name__ == "__main__":
    raise SystemExit(main())
