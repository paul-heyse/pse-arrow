# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit cooperating command ownership, independent from source recorder lifetime."""

from __future__ import annotations

import contextlib
import os
import select
import signal
import subprocess
import threading
import time
from pathlib import Path
from typing import TYPE_CHECKING

from msgspec.structs import replace

from pse import codec
from pse.contracts.documents import (
    SourceRecordingAcknowledgement,
    SourceRecordingLaunchDocument,
    SourceRecordingRequest,
)

if TYPE_CHECKING:
    from collections.abc import Callable
    from typing import IO

    from pse.contracts.documents import ClosedDuration, SourceRecordingLaunchRequest


def _seconds(duration: ClosedDuration) -> float:
    # Shared generated duration, without accepting arbitrary floats as a deadline.
    return duration.secs + duration.nanos / 1_000_000_000


def record_launch(
    request: SourceRecordingLaunchRequest,
    *,
    cancelled: Callable[[], bool] = lambda: False,
) -> SourceRecordingLaunchDocument:
    """Launch literal argv with its own source construction acknowledgement."""
    if request.version != 1:
        message = "unsupported source launch request version"
        raise ValueError(message)
    if not request.argv or not request.argv[0] or request.output_bytes < 2:
        message = "record-launch needs literal argv and finite output allowance"
        raise ValueError(message)
    timeout, ack_timeout = (
        _seconds(request.command_timeout),
        _seconds(request.acknowledgement_timeout),
    )
    if timeout <= 0 or ack_timeout <= 0 or request.output_bytes > 1 << 30:
        message = (
            "record-launch requires finite positive independent clocks "
            "and bounded output"
        )
        raise ValueError(message)
    request_file = Path(request.request_file).resolve()
    with request_file.open("rb") as source:
        encoded = source.read((64 << 10) + 1)
    if len(encoded) > 64 << 10:
        message = "source request exceeds protocol allowance"
        raise ValueError(message)
    selection = codec.decode_json(encoded, SourceRecordingRequest)
    ack_path = Path(selection.acknowledgement)
    if not ack_path.is_absolute() or not Path(selection.detail.output).is_absolute():
        message = (
            "launch source output and acknowledgement must be absolute explicit paths"
        )
        raise ValueError(message)
    claim_path = ack_path.with_name(ack_path.name + ".launch")
    descriptor = os.open(claim_path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    identity = None
    handle_created = False
    try:
        identity = os.fstat(descriptor)
        claim = os.fdopen(descriptor, "w")
        handle_created = True
        with claim:
            claim.write(f"launch owner pid={os.getpid()}\n")
            claim.flush()
            return _launch_owned(request, selection, cancelled)
    finally:
        try:
            if not handle_created:
                os.close(descriptor)
        finally:
            try:
                current = claim_path.stat(follow_symlinks=False)
                if identity is None or (current.st_dev, current.st_ino) == (
                    identity.st_dev,
                    identity.st_ino,
                ):
                    claim_path.unlink()
            except FileNotFoundError:
                pass


def _read_acknowledgement(
    ack_path: Path, child_pid: int, selection: SourceRecordingRequest
) -> SourceRecordingAcknowledgement:
    """Validate the bounded source response against this exact owned launch."""
    with ack_path.open("rb") as source:
        ack_bytes = source.read((64 << 10) + 1)
    if len(ack_bytes) > 64 << 10:
        message = "source acknowledgement exceeds protocol allowance"
        raise ValueError(message)
    acknowledgement = codec.decode_json(ack_bytes, SourceRecordingAcknowledgement)
    # Exact direct receiver remains attributable after its process has exited;
    # a live descendant must still belong to this owned launch group.
    belongs = acknowledgement.pid == child_pid
    if not belongs:
        with contextlib.suppress(ProcessLookupError):
            belongs = os.getpgid(acknowledgement.pid) == child_pid
    admitted = replace(
        acknowledgement.selection,
        expiry_monotonic_ns=selection.detail.expiry_monotonic_ns,
    )
    review_matches = (selection.review is None and acknowledgement.review is None) or (
        selection.review is not None
        and acknowledgement.review is not None
        and selection.review == acknowledgement.review.request
    )
    if not belongs or admitted != selection.detail or not review_matches:
        message = (
            "source acknowledgement receiver/selection does not belong to this launch"
        )
        raise ValueError(message)
    return acknowledgement


def _output_pipe(pipe: IO[bytes] | None) -> IO[bytes]:
    if pipe is None:
        message = "owned command output pipe unavailable"
        raise RuntimeError(message)
    return pipe


def _launch_owned(
    request: SourceRecordingLaunchRequest,
    selection: SourceRecordingRequest,
    cancelled: Callable[[], bool],
) -> SourceRecordingLaunchDocument:
    timeout, ack_timeout = (
        _seconds(request.command_timeout),
        _seconds(request.acknowledgement_timeout),
    )
    ack_path = Path(selection.acknowledgement)
    if not ack_path.is_absolute() or not Path(selection.detail.output).is_absolute():
        message = (
            "launch source output and acknowledgement must be absolute explicit paths"
        )
        raise ValueError(message)
    if ack_path.exists():
        message = "source acknowledgement already exists; refusing stale substitution"
        raise FileExistsError(message)
    output = Path(request.output)
    output.mkdir(parents=True, exist_ok=False)
    result = SourceRecordingLaunchDocument(
        version=2,
        request=request,
        acknowledgement=None,
        command_status=None,
        timed_out=False,
        cancelled=False,
        drained=False,
        stdout_prefix_bytes=0,
        stderr_prefix_bytes=0,
        logs_truncated=False,
        limitations=(
            (
                "Selection acknowledgement is not final family/managed-child "
                "coverage; inspect the original detail stream."
            ),
        ),
    )
    counts = [0, 0]
    truncated = [False, False]
    failures: list[str] = []
    stop_draining = threading.Event()

    def drain(pipe: IO[bytes], destination: Path, index: int) -> None:
        descriptor = pipe.fileno()
        os.set_blocking(descriptor, False)
        try:
            with destination.open("xb") as sink:
                while not stop_draining.is_set():
                    if not select.select([descriptor], [], [], 0.05)[0]:
                        continue
                    block = os.read(descriptor, 65536)
                    if not block:
                        return
                    keep = min(
                        len(block), max(0, request.output_bytes // 2 - counts[index])
                    )
                    if keep:
                        sink.write(block[:keep])
                        counts[index] += keep
                    truncated[index] |= keep < len(block)
                truncated[index] = True
        except OSError as error:
            failures.append(str(error))
            # A failed log file must not deadlock a scientific command's pipe.
            while not stop_draining.is_set():
                if select.select([descriptor], [], [], 0.05)[0] and not os.read(
                    descriptor, 65536
                ):
                    break
        finally:
            pipe.close()

    child: subprocess.Popen[bytes] | None = None
    threads: list[threading.Thread] = []
    interrupted: BaseException | None = None
    start = time.monotonic()
    ack_failed = False
    try:
        environment = os.environ.copy()
        environment["PSE_DIAGNOSTIC_RECORD_REQUEST"] = str(
            Path(request.request_file).resolve()
        )
        child = subprocess.Popen(  # noqa: S603 -- caller explicitly selects literal argv; shell=False
            request.argv,
            cwd=request.cwd,
            env=environment,
            shell=False,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            start_new_session=True,
        )
        for index, (pipe, name) in enumerate(
            ((child.stdout, "stdout.txt"), (child.stderr, "stderr.txt"))
        ):
            owned_pipe = _output_pipe(pipe)
            thread = threading.Thread(
                target=drain, args=(owned_pipe, output / name, index), daemon=False
            )
            thread.start()
            threads.append(thread)
        while True:
            elapsed = time.monotonic() - start
            if (
                not ack_failed
                and result.acknowledgement is None
                and elapsed <= ack_timeout
                and ack_path.is_file()
            ):
                try:
                    acknowledgement = _read_acknowledgement(
                        ack_path, child.pid, selection
                    )
                    result = replace(result, acknowledgement=acknowledgement)
                except (OSError, ValueError, OverflowError) as error:
                    # Protocol/export failure is independent of the scientific command.
                    ack_failed = True
                    failures.append(f"Source acknowledgement unavailable: {error}")
            if child.poll() is not None:
                break
            if cancelled() or elapsed >= timeout:
                result = replace(
                    result, cancelled=cancelled(), timed_out=elapsed >= timeout
                )
                break
            time.sleep(0.02)
    except BaseException as error:
        interrupted = error
        result = replace(
            result, cancelled=True, limitations=(*result.limitations, str(error))
        )
    finally:
        if child is not None:
            # Join the exact owned process group, including output-pipe holders.
            # Recorder stop alone
            # never reaches this path; command completion/timeout/cancellation owns this
            # boundary.
            with contextlib.suppress(ProcessLookupError):
                os.killpg(child.pid, signal.SIGTERM)
            try:
                child.wait(timeout=1)
            except subprocess.TimeoutExpired:
                os.killpg(child.pid, signal.SIGKILL)
                child.wait()
            # Escalate even if the leader exited and a descendant closed its output
            # pipes.
            with contextlib.suppress(ProcessLookupError):
                os.killpg(child.pid, signal.SIGKILL)
            end_drain = time.monotonic() + 2
            drained = False
            while time.monotonic() < end_drain:
                try:
                    os.killpg(child.pid, 0)
                except ProcessLookupError:
                    drained = True
                    break
                time.sleep(0.01)
            for thread in threads:
                thread.join(timeout=0.2)
            stop_draining.set()
            for thread in threads:
                thread.join()
            if not drained:
                failures.append(
                    "Owned process-group drain unavailable after "
                    "stop; no complete lifecycle claim."
                )
            result = replace(result, drained=drained)
            result = replace(result, command_status=child.returncode)
        if result.acknowledgement is None:
            failures.append(
                "No preconstruction acknowledgement observed "
                "inside the requested window; source recording is "
                "unavailable, independent of command success."
            )
        for index, name in enumerate(("stdout.txt", "stderr.txt")):
            try:
                retained = (output / name).stat().st_size
            except OSError as error:
                retained = 0
                failures.append(f"Command log prefix unavailable: {error}")
            truncated[index] |= retained < counts[index]
            counts[index] = min(counts[index], retained)
        result = replace(
            result,
            stdout_prefix_bytes=counts[0],
            stderr_prefix_bytes=counts[1],
            logs_truncated=any(truncated),
            limitations=(*result.limitations, *failures),
        )
        (output / "launch-result.json").write_bytes(codec.encode_json(result))
    if interrupted is not None:
        raise interrupted
    return result
