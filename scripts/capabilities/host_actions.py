# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Canonical host actions use typed owner kernels and their admitted child lifetime."""

from __future__ import annotations

import asyncio
import json
import subprocess
import tempfile
from pathlib import Path
from typing import TypeVar

from fastmcp.exceptions import ToolError
from fastmcp.tools import Tool
from mcp.types import ToolAnnotations
from pydantic import BaseModel, ConfigDict, Field, JsonValue, TypeAdapter

from scripts.active_diagnostics import ActionRequest
from scripts.capabilities.model import Capability, Contract
from scripts.capabilities.operations import bounded
from scripts.test_archives import ArchiveRequest, ReplayRequest

JSON_OBJECT = TypeAdapter(dict[str, JsonValue])
T = TypeVar("T")


class ArchiveInspectRequest(Contract):
    model_config = ConfigDict(extra="forbid", strict=True, frozen=True, validate_default=True)

    archive: str = Field(min_length=1)
    offset: int = Field(default=0, ge=0)
    limit: int = Field(default=50, ge=1, le=100)
    manifest_sha256: str | None = Field(default=None, pattern=r"^[a-f0-9]{64}$")


class ActionReadRequest(Contract):
    model_config = ConfigDict(extra="forbid", strict=True, frozen=True, validate_default=True)

    output: str = Field(min_length=1)
    tail: int = Field(default=20, ge=0, le=1000)
    summary_only: bool = False
    action_sha256: str | None = Field(default=None, pattern=r"^[a-f0-9]{64}$")


class HostResult(Contract):
    """Projection completeness is independent from the native result's status."""

    operation: str
    result: dict[str, JsonValue]
    detail_complete: bool
    omissions: list[str]


def project(value: object, operation: str) -> HostResult:
    detail = bounded(value, operation)
    return HostResult(
        operation=operation, result=detail.payload,
        detail_complete=detail.complete, omissions=detail.omissions,
    )


def failure(operation: str, error: Exception) -> ToolError:
    return ToolError(json.dumps({
        "code": "pse.host." + operation + ".refused",
        "cause": type(error).__name__, "message": str(error),
    }))


async def settled(task: asyncio.Task[T]) -> T:
    """Repeated transport cancellation cannot abandon an already owned helper."""
    while True:
        try:
            return await asyncio.shield(task)
        except asyncio.CancelledError:
            if task.done():
                return task.result()


async def run_worker(root: Path, operation: str, request: BaseModel) -> HostResult:
    """Run one closed kernel on its own main thread, preserving native signal/drain ownership.

    No timeout or disconnect establishes rollback. On cancellation, signal only
    the unreaped direct helper; the environment/native owners forward and drain
    their own children. An attached diagnostic subject is outside this ownership.
    """
    if operation not in {"archive", "replay", "diagnostic"}:
        raise ValueError("unsupported fixed host operation")
    with tempfile.TemporaryDirectory(prefix="pse-capability-host-") as temporary:
        directory = Path(temporary)
        source, destination = directory / "request.json", directory / "response.json"
        source.write_text(request.model_dump_json(exclude_unset=True))
        source.chmod(0o600)
        with tempfile.TemporaryFile() as log:
            spawn = asyncio.create_task(asyncio.create_subprocess_exec(
                str(root / "scripts/pse-env"), "--",
                str(root / ".venv/bin/python"), "-m", "scripts.capabilities.host_worker",
                operation, str(source), str(destination),
                cwd=root, stdout=log, stderr=log,
            ))
            process = None
            waiting = None
            try:
                process = await asyncio.shield(spawn)
                waiting = asyncio.create_task(process.wait())
                code = await asyncio.shield(waiting)
            except asyncio.CancelledError:
                # Shield spawn as well as wait: cancellation can arrive after OS
                # creation but before create_subprocess_exec returns its handle.
                process = await settled(spawn)
                if process.returncode is None:
                    try:
                        process.terminate()
                    except ProcessLookupError:
                        pass
                if waiting is None:
                    waiting = asyncio.create_task(process.wait())
                await settled(waiting)
                raise
            if code != 0 or not destination.is_file():
                log.seek(0)
                diagnostic = log.read(16384).decode(errors="replace")
                raise ToolError(json.dumps({
                    "code": "pse.host.helper_unsettled",
                    "operation": operation, "helper_exit_code": code,
                    "message": diagnostic,
                    "output": getattr(request, "output", None),
                    "settlement": "inspect native owner output; helper exit alone is not settlement",
                }))
            response = JSON_OBJECT.validate_json(destination.read_bytes())
            if "error" in response:
                raise ToolError(json.dumps(response["error"]))
            return project(response["result"], operation)


def bindings(root: Path) -> tuple[tuple[Capability, ...], dict[str, Tool]]:
    async def experiment_archive(request: ArchiveRequest) -> HostResult:
        """Archive one selected pure Rust test closure into an explicit fresh build directory. Retains real producer/resource handles; native archival remains unsupported. Cancellation uses the admitted helper owner and does not undo output."""
        return await run_worker(root, "archive", request)

    def experiment_inspect(request: ArchiveInspectRequest) -> HostResult:
        """Inspect retained archive bytes and page historical selected tests. Supply returned manifest_sha256 on continuation to refuse replaced source. Does not build or replay current source."""
        from scripts import test_archives  # noqa: PLC0415

        try:
            return project(test_archives.inspect(
                root / request.archive, offset=request.offset, limit=request.limit,
                manifest_sha256=request.manifest_sha256,
            ), "experiment_inspect")
        except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
            raise failure("experiment_inspect", error) from error

    async def experiment_replay(request: ReplayRequest) -> HostResult:
        """Replay exact retained pure test executables into a fresh explicit build directory with a new authentic invocation. Historical replay never qualifies current source. Cancellation retains native terminal/drain distinctions and partial artifacts."""
        return await run_worker(root, "replay", request)

    async def diagnostic_action(request: ActionRequest) -> HostResult:
        """Run an explicit diagnostic experiment, including exact PID/start attachment or literal owned launch argv. Helper/capture/decoder status and subject outcome remain independent. Cancelling the owned helper cannot cancel its observed subject; no rollback is promised."""
        return await run_worker(root, "diagnostic", request)

    def diagnostic_action_read(request: ActionReadRequest) -> HostResult:
        """Read an exact retained action record and verified bounded text tails. Changed artifacts retain native refusals; reading never launches, cancels or retries a diagnostic subject."""
        from scripts import active_diagnostics  # noqa: PLC0415

        try:
            return project(active_diagnostics.read_action(
                root / request.output, tail=request.tail, summary_only=request.summary_only,
                action_sha256=request.action_sha256,
            ), "diagnostic_action_read")
        except (OSError, ValueError, RuntimeError) as error:
            raise failure("diagnostic_action_read", error) from error

    rows = (
        (experiment_archive, "Archive selected historical tests", "scripts/test_archives.py", "experiments", ("run-experiment",), False),
        (experiment_inspect, "Inspect a retained test archive", "scripts/test_archives.py", "experiments", ("run-experiment", "investigate-evidence"), True),
        (experiment_replay, "Replay retained historical tests", "scripts/test_archives.py", "experiments", ("run-experiment",), False),
        (diagnostic_action, "Run an explicit diagnostic action", "scripts/active_diagnostics.py", "diagnostics", ("diagnose-run",), False),
        (diagnostic_action_read, "Read retained diagnostic output", "scripts/active_diagnostics.py", "diagnostics", ("diagnose-run", "investigate-evidence"), True),
    )
    specs = []
    tools: dict[str, Tool] = {}
    for function, title, source, domain, workflows, readonly in rows:
        tool = Tool.from_function(function, annotations=ToolAnnotations(
            read_only_hint=readonly, destructive_hint=not readonly,
            idempotent_hint=readonly, open_world_hint=not readonly,
        ))
        spec = Capability(
            id=tool.name, title=title, description=tool.description or title,
            domain=domain, workflows=workflows, guides=("root", "tooling"),
            family="experiments-recovery" if domain == "experiments" else "active-diagnostics",
            owner=source, source=source, exposure="tool",
            effects="Read exact retained artifacts; no workload effects." if readonly else
            "Runs one explicit admitted host operation. Creates selected output and may launch or sample processes; native owners retain admission, cancellation and drain. Cancellation does not establish rollback.",
            prerequisites=() if readonly else ("Checkout environment and operation-specific native tools/identity admission",),
        )
        tool.tags.update(spec.tags)
        specs.append(spec)
        tools[spec.id] = tool
    return tuple(specs), tools
