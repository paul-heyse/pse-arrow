# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Finite optional query execution; the declared SQL and schema are owned in Rust."""

from __future__ import annotations

import contextlib
import csv
import os
import signal
import subprocess
import sys
import time
from pathlib import Path
from typing import TYPE_CHECKING

from msgspec.structs import replace

from pse import codec
from pse._build import _diagnostic_query_specification
from pse.contracts.documents import (
    ReviewQueryDocumentOutput,
    ReviewQueryRequestOutput,
    ReviewQueryStatus,
)

if TYPE_CHECKING:
    from collections.abc import Callable

    from pse.contracts.documents import (
        ReviewQueryRequest,
        ReviewQuerySpecificationOutput,
    )


def _bounded_child(
    argv: list[str],
    rows: Path,
    errors: Path,
    allowance: int,
    timeout: float,
    cancelled: Callable[[], bool],
) -> tuple[int, str | None]:
    with rows.open("xb") as output, errors.open("xb") as stderr:
        if cancelled():
            return -1, "cancelled"
        if timeout <= 0:
            return 124, "timed_out"
        child = subprocess.Popen(  # noqa: S603 -- pinned interpreter and explicit tool argv; shell=False
            [
                sys.executable,
                "-I",
                str(Path(__file__).with_name("_diagnostic_query_child.py").resolve()),
                str(allowance),
                *argv,
            ],
            shell=False,
            stdin=subprocess.DEVNULL,
            stdout=output,
            stderr=stderr,
            start_new_session=True,
        )
        reason: str | None = None
        end = time.monotonic() + timeout
        try:
            while child.poll() is None:
                if cancelled():
                    reason = "cancelled"
                    break
                if time.monotonic() >= end:
                    reason = "timed_out"
                    break
                time.sleep(min(0.02, max(0, end - time.monotonic())))
        except BaseException:
            os.killpg(child.pid, signal.SIGKILL)
            child.wait()
            raise
        if reason is not None:
            with contextlib.suppress(ProcessLookupError):
                os.killpg(child.pid, signal.SIGKILL)
        status = child.wait()
        # A normally exited tool can leave inherited output descriptors in descendants.
        # Stop the exact owned group before observing final prefixes, independently of
        # pipes.
        try:
            os.killpg(child.pid, signal.SIGKILL)
            if reason is None:
                reason = "descendants_stopped"
        except ProcessLookupError:
            pass
        end_drain = time.monotonic() + 2
        while True:
            try:
                os.killpg(child.pid, 0)
            except ProcessLookupError:
                break
            if time.monotonic() >= end_drain:
                reason = f"{reason or 'failed'}:drain_unavailable"
                break
            time.sleep(0.01)
    return status, reason


def _validate_csv(path: Path, specification: ReviewQuerySpecificationOutput) -> int:
    """Validate one result schema without collecting rows or inferring types."""
    with path.open(newline="", encoding="utf-8", errors="strict") as source:
        reader = csv.reader(source, strict=True)
        if next(reader, None) != [column.name for column in specification.columns]:
            message = "query did not return the declared CSV header"
            raise ValueError(message)
        count = 0
        for row in reader:
            if len(row) != len(specification.columns):
                message = "malformed or multiple CSV result sets"
                raise ValueError(message)
            for value, column in zip(row, specification.columns, strict=True):
                if value in {"[NULL]", "NULL", ""}:
                    continue
                if column.kind == "integer":
                    integer = int(value)
                    if not -(1 << 63) <= integer < (1 << 63):
                        message = (
                            "SQL integer is outside signed int64; use native UInt64"
                        )
                        raise ValueError(message)
                elif column.kind == "number":
                    float(
                        value
                    )  # Raw text retains nonfinite categories; no arithmetic here.
            count += 1
    return count


def _validate_inputs(binary: Path, trace: Path) -> None:
    if not binary.is_file() or not trace.is_file():
        message = (
            "selected trace or optional Trace Processor unavailable; "
            "no installation attempted"
        )
        raise FileNotFoundError(message)


def run_query(
    request: ReviewQueryRequest | ReviewQueryRequestOutput,
    trace: Path,
    output: Path,
    binary: Path,
    *,
    cancelled: Callable[[], bool] = lambda: False,
) -> ReviewQueryDocumentOutput:
    """Retain query context and failure prefixes without executing science."""
    spec = _diagnostic_query_specification(request)
    end = time.monotonic() + request.timeout_seconds
    output.mkdir(parents=True, exist_ok=False)
    sql = output / "query.sql"
    sql.write_text(spec.sql, encoding="utf-8")
    rows, errors = output / "rows.csv", output / "stderr.txt"
    result = ReviewQueryDocumentOutput(
        version=1,
        request=(
            request
            if isinstance(request, ReviewQueryRequestOutput)
            else ReviewQueryRequestOutput(
                version=request.version,
                clock_scope=request.clock_scope,
                input_member=request.input_member,
                interval=request.interval,
                output_bytes=request.output_bytes,
                question=request.question,
                timeout_seconds=request.timeout_seconds,
            )
        ),
        query_version=spec.query_version,
        sql=spec.sql,
        tool_version=None,
        status="unavailable",
        exit_code=None,
        columns=spec.columns,
        covered_interval=None,
        output="rows.csv",
        output_prefix_bytes=0,
        error_output="stderr.txt",
        error_prefix_bytes=0,
        rows=None,
        limitations=spec.limitations,
    )
    try:
        _validate_inputs(binary, trace)
        version, version_error = (
            output / "tool-version.txt",
            output / "tool-version-error.txt",
        )
        code, reason = _bounded_child(
            [str(binary.resolve()), "--version"],
            version,
            version_error,
            4096,
            min(2.0, end - time.monotonic()),
            cancelled,
        )
        if code == 0 and reason is None:
            result = replace(
                result, tool_version=version.read_text(errors="replace").strip()[:4096]
            )
        else:
            result = replace(
                result,
                limitations=(
                    *result.limitations,
                    (
                        "Actual tool version unavailable; "
                        "declared version was not substituted."
                    ),
                ),
            )
        status, reason = _bounded_child(
            [
                str(binary.resolve()),
                "query",
                "-f",
                str(sql.resolve()),
                str(trace.resolve()),
            ],
            rows,
            errors,
            request.output_bytes // 2,
            end - time.monotonic(),
            cancelled,
        )
        state: ReviewQueryStatus = "completed"
        if reason is not None and reason.startswith("timed_out"):
            state = "timed_out"
        elif reason is not None and reason.startswith("cancelled"):
            state = "cancelled"
        elif reason is not None:
            state = "failed"
            result = replace(
                result,
                limitations=(
                    *result.limitations,
                    f"Owned query group cleanup: {reason}. Output is a failure prefix.",
                ),
            )
        elif (
            rows.stat().st_size >= request.output_bytes // 2
            or errors.stat().st_size >= request.output_bytes // 2
        ):
            state = "output_limit"
        elif status != 0:
            state = "failed"
        result = replace(result, status=state, exit_code=status)
        if state == "completed":
            try:
                result = replace(result, rows=_validate_csv(rows, spec))
            except (OSError, UnicodeError, ValueError, csv.Error) as error:
                result = replace(
                    result,
                    status="malformed",
                    limitations=(*result.limitations, str(error)),
                )
    except (OSError, ValueError) as error:
        result = replace(result, limitations=(*result.limitations, str(error)))
    except BaseException:
        result = replace(
            result,
            status="cancelled",
            limitations=(
                *result.limitations,
                "Caller interrupted; owned query child joined.",
            ),
        )
        raise
    finally:
        result = replace(
            result,
            output_prefix_bytes=rows.stat().st_size if rows.exists() else 0,
            error_prefix_bytes=errors.stat().st_size if errors.exists() else 0,
        )
        (output / "query-result.json").write_bytes(codec.encode_json(result))
    return result
