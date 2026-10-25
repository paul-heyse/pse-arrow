# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Internal fixed action transport; native operation owners retain physical drain.

Worker/module identities come from the shared fixed declaration, never an MCP request.
Detached observation adapters have a different cancellation policy and do not use this.
"""

from __future__ import annotations

import asyncio
import json
import tempfile
from contextlib import suppress
from pathlib import Path
from typing import TYPE_CHECKING, TypeVar

from fastmcp.exceptions import ToolError
from pydantic import BaseModel, JsonValue, TypeAdapter

from scripts.capabilities import execution_basis
from scripts.capabilities.worker_progress import ProgressObserver
from scripts.telemetry_process import helper_environment

if TYPE_CHECKING:
    from scripts.capabilities.workers import WorkerCall

JSON_OBJECT = TypeAdapter(dict[str, JsonValue])
T = TypeVar("T")


async def settled(task: asyncio.Task[T]) -> T:
    """Repeated transport cancellation cannot abandon an already owned helper."""
    while True:
        try:
            return await asyncio.shield(task)
        except asyncio.CancelledError:
            if task.done():
                return task.result()


async def run_worker(
    root: Path, call: WorkerCall, request: BaseModel | dict[str, JsonValue]
) -> dict[str, JsonValue]:
    """Run one closed kernel on its own main thread, preserving native signal/drain ownership.

    No timeout or disconnect establishes rollback. On cancellation, signal only
    the unreaped direct helper; the environment/native owners forward and drain
    their own children. An attached diagnostic subject is outside this ownership.
    """
    declaration = call.declaration
    operation = call.operation
    if call.lifetime != "attached" or operation not in declaration.operations:
        raise ValueError("unsupported fixed worker operation")
    with tempfile.TemporaryDirectory(prefix="pse-capability-host-") as temporary:
        directory = Path(temporary)
        observer = ProgressObserver(directory / "progress.json", operation)
        source, destination = directory / "request.json", directory / "response.json"
        value = (
            request.model_dump(mode="json", exclude_unset=True)
            if isinstance(request, BaseModel)
            else JSON_OBJECT.validate_python(request)
        )
        document = json.dumps(value, allow_nan=False).encode()
        source.write_bytes(document)
        admitted = execution_basis.publish(
            directory, root, call.worker, operation, document
        )
        source.chmod(0o600)
        command = [str(root / ".venv/bin/python"), "-m", declaration.module]
        if declaration.launch == "environment":
            command = [
                str(root / "scripts/pse-env"),
                *declaration.environment,
                "--",
                *command,
            ]
        with tempfile.TemporaryFile() as log:
            spawn = asyncio.create_task(
                asyncio.create_subprocess_exec(
                    *command,
                    operation,
                    str(source),
                    str(destination),
                    cwd=root,
                    env={**helper_environment(), **admitted},
                    stdout=log,
                    stderr=log,
                )
            )
            process = None
            waiting = None
            observing = None
            try:
                process = await asyncio.shield(spawn)
                waiting = asyncio.create_task(process.wait())
                observing = asyncio.create_task(observer.until(waiting))
                code = await asyncio.shield(waiting)
                await asyncio.shield(observing)
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
                if observing is not None:
                    await settled(observing)
                raise
            if code != 0 or not destination.is_file():
                diagnostic = "Native helper did not return structured output; source-bearing diagnostics withheld."
                raise ToolError(
                    json.dumps(
                        {
                            "code": "pse.host.helper_unsettled",
                            "operation": operation,
                            "helper_exit_code": code,
                            "message": diagnostic,
                            "stage": "helper_transport",
                            "remedy": "Inspect the original operation's output and native settlement reader before deciding further action. Do not repeat work based on helper exit alone.",
                            "requested_output": value.get("output"),
                            "output_admission": "unconfirmed",
                            "settlement": "inspect native owner output; helper exit alone is not settlement",
                            "observation": observer.recovery(),
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
