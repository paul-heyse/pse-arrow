# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Internal fixed action transport; native operation owners retain physical drain.

Worker/module identities are declared here, never supplied by an MCP request.
Detached observation adapters have a different cancellation policy and do not use this.
"""

from __future__ import annotations

import asyncio
import json
import tempfile
from contextlib import suppress
from dataclasses import dataclass
from pathlib import Path
from typing import TypeVar

from fastmcp.exceptions import ToolError
from pydantic import BaseModel, JsonValue, TypeAdapter

JSON_OBJECT = TypeAdapter(dict[str, JsonValue])
T = TypeVar("T")


@dataclass(frozen=True)
class Worker:
    module: str
    operations: frozenset[str]
    environment: tuple[str, ...] = ()
    redact_diagnostics: bool = False


WORKERS = {
    "host": Worker(
        "scripts.capabilities.host_worker",
        frozenset({"archive", "replay", "diagnostic"}),
    ),
    "qualification": Worker(
        "scripts.capabilities.qualification_worker",
        frozenset(
            {
                "assessment",
                "compilation",
                "contracts-check",
                "contracts-generate",
                "documentation",
            }
        ),
    ),
    "rust-tests": Worker(
        "scripts.capabilities.qualification_worker",
        frozenset({"rust-test"}),
        ("--recipe-default-class", "exclusive", "--native="),
    ),
    "native": Worker(
        "scripts.capabilities.native_worker",
        frozenset({"source_admit", "preparation_discover"}),
        ("--native=solver,klu,isolation,uno,petsc",),
        redact_diagnostics=True,
    ),
}


async def settled(task: asyncio.Task[T]) -> T:
    """Repeated transport cancellation cannot abandon an already owned helper."""
    while True:
        try:
            return await asyncio.shield(task)
        except asyncio.CancelledError:
            if task.done():
                return task.result()


async def run_worker(
    root: Path, worker: str, operation: str, request: BaseModel | dict[str, JsonValue]
) -> dict[str, JsonValue]:
    """Run one closed kernel on its own main thread, preserving native signal/drain ownership.

    No timeout or disconnect establishes rollback. On cancellation, signal only
    the unreaped direct helper; the environment/native owners forward and drain
    their own children. An attached diagnostic subject is outside this ownership.
    """
    if worker not in WORKERS or operation not in WORKERS[worker].operations:
        raise ValueError("unsupported fixed worker operation")
    declaration = WORKERS[worker]
    with tempfile.TemporaryDirectory(prefix="pse-capability-host-") as temporary:
        directory = Path(temporary)
        source, destination = directory / "request.json", directory / "response.json"
        value = (
            request.model_dump(mode="json", exclude_unset=True)
            if isinstance(request, BaseModel)
            else JSON_OBJECT.validate_python(request)
        )
        source.write_text(json.dumps(value, allow_nan=False))
        source.chmod(0o600)
        with tempfile.TemporaryFile() as log:
            spawn = asyncio.create_task(
                asyncio.create_subprocess_exec(
                    str(root / "scripts/pse-env"),
                    *declaration.environment,
                    "--",
                    str(root / ".venv/bin/python"),
                    "-m",
                    declaration.module,
                    operation,
                    str(source),
                    str(destination),
                    cwd=root,
                    stdout=log,
                    stderr=log,
                )
            )
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
                    with suppress(ProcessLookupError):
                        process.terminate()
                if waiting is None:
                    waiting = asyncio.create_task(process.wait())
                await settled(waiting)
                raise
            if code != 0 or not destination.is_file():
                log.seek(0)
                diagnostic = (
                    "Native helper did not return structured output; source-bearing diagnostics withheld."
                    if declaration.redact_diagnostics
                    else log.read(16384).decode(errors="replace")
                )
                raise ToolError(
                    json.dumps(
                        {
                            "code": "pse.host.helper_unsettled",
                            "operation": operation,
                            "helper_exit_code": code,
                            "message": diagnostic,
                            "output": getattr(request, "output", None),
                            "settlement": "inspect native owner output; helper exit alone is not settlement",
                        }
                    )
                )
            response = JSON_OBJECT.validate_json(destination.read_bytes())
            if "error" in response:
                raise ToolError(json.dumps(response["error"]))
            result = response.get("result")
            if not isinstance(result, dict):
                raise ToolError(
                    "pse.host.invalid_response: fixed worker must return an object"
                )
            return result
