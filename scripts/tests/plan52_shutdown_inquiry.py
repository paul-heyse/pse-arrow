# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit I14 selected cleanup-repair controls; no provider process or network.

Invoke this driver deliberately. The controls use actual Connection/Specialists,
the actual language context manager and the actual endpoint lifespan callback.
Transport exit exceptions are injected; no natural incident frequency is claimed.
"""

from __future__ import annotations

import argparse
import asyncio
import json
import tempfile
from pathlib import Path
from unittest.mock import patch

import httpx2
from fastmcp import Client, FastMCP
from fastmcp.client.transports import FastMCPTransport
from pydantic import BaseModel

from scripts import semantic_navigation
from scripts.capabilities import server as canonical
from scripts.capabilities import specialists
from scripts.telemetry_investigation import BackendClient

ROOT = Path(__file__).resolve().parents[2]


class Observation(BaseModel):
    case: str
    injected_boundary: str
    close_raised: str | None
    closed_flags: dict[str, bool]
    owner_tasks_done: dict[str, bool]
    exit_attempts: dict[str, int]
    retained_owner_error: bool
    fixture_settled: bool


def backend() -> FastMCP:
    service = FastMCP("i14-in-process-transport-control")

    @service.tool
    def probe() -> int:
        return 1

    return service


async def ping(client: Client, _tools: specialists.ClientTools) -> dict:
    result = await client.call_tool("probe", {})
    if result.is_error:
        raise AssertionError("in-process probe failed")
    return {"observed": True}


async def connection_control(root: Path) -> Observation:
    """Actual language factory exit error remains visible after siblings settle."""
    service = backend()
    first = FastMCPTransport(service)
    sibling = FastMCPTransport(service)
    exits = {"first": 0, "sibling": 0}

    async def first_exit() -> None:
        exits["first"] += 1
        raise RuntimeError("injected language transport exit")

    async def sibling_exit() -> None:
        exits["sibling"] += 1

    owner = specialists.Specialists(root)
    owner.languages["python"] = specialists.Connection(lambda: semantic_navigation.connection(root, "python", fixture=True))
    owner.languages["rust"] = specialists.Connection(lambda: semantic_navigation.connection(root, "rust", fixture=True))
    connections = dict(owner.languages)
    raised = None
    with (
        patch.object(semantic_navigation, "configuration", return_value=""),
        patch.object(semantic_navigation.diagnostic_trace, "binary_path", return_value=Path("/unexecuted/mcpls")),
        patch("fastmcp.client.transports.StdioTransport", side_effect=[first, sibling]),
        patch.object(first, "close", side_effect=first_exit),
        patch.object(sibling, "close", side_effect=sibling_exit),
    ):
        try:
            for connection in connections.values():
                await connection.invoke(ping)
            try:
                await owner.close()
            except Exception as error:
                raised = type(error).__name__
            done = {name: item.task is not None and item.task.done() for name, item in connections.items()}
            flags = {name: item.closed for name, item in connections.items()}
            retained = any(item.task is not None and not item.task.cancelled() and item.task.exception() is not None for item in connections.values())
        finally:
            if not owner.closed:
                await owner.close()
    return Observation(case="actual-language-context-exit", injected_boundary="semantic_navigation.connection finally -> transport.close", close_raised=raised, closed_flags=flags, owner_tasks_done=done, exit_attempts=exits, retained_owner_error=retained, fixture_settled=all(item.task is not None and item.task.done() for item in connections.values()))


async def endpoint_control(root: Path) -> Observation:
    """Real BackendClient/HTTP-client exit failure cannot skip the lifespan sibling."""
    attempts = {"telemetry": 0}

    class ExitFailure(httpx2.AsyncBaseTransport):
        async def aclose(self) -> None:
            attempts["telemetry"] += 1
            raise RuntimeError("injected HTTP transport exit")

    telemetry = BackendClient()
    telemetry._client = httpx2.AsyncClient(transport=ExitFailure(), trust_env=False)
    with patch.object(canonical, "BackendClient", return_value=telemetry):
        surface = canonical.build(ROOT)
    connection = specialists.Connection(lambda: Client(backend()))
    surface.specialists.languages["python"] = connection
    raised = None
    try:
        await connection.invoke(ping)
        try:
            async with surface.server._lifespan(surface.server):
                pass
        except Exception as error:
            raised = type(error).__name__
        done = {"language": connection.task is not None and connection.task.done()}
        flags = {"specialists": surface.specialists.closed, "language": connection.closed, "telemetry_reference_cleared": telemetry._client is None}
    finally:
        await surface.specialists.close()
    return Observation(case="actual-endpoint-telemetry-exit", injected_boundary="server.build lifetime -> BackendClient.close -> httpx2.AsyncClient.aclose -> transport.aclose", close_raised=raised, closed_flags=flags, owner_tasks_done=done, exit_attempts=attempts, retained_owner_error=raised is not None, fixture_settled=connection.task is not None and connection.task.done())


async def run() -> dict:
    with tempfile.TemporaryDirectory(prefix="pse-i14-inquiry-") as directory:
        async with asyncio.timeout(20):
            observations = [await connection_control(Path(directory)), await endpoint_control(Path(directory))]
    if any(not item.fixture_settled or not item.close_raised or not item.retained_owner_error or not all(item.owner_tasks_done.values()) for item in observations):
        raise AssertionError("selected cleanup repair did not settle owners and retain failure")
    return {
        "schema": "plan52-i14-inquiry-v1",
        "observations": [item.model_dump(mode="json") for item in observations],
        "scope": "Injected exception controls with actual owners and in-process transports; no language/provider subprocess, network, external process settlement or host campaign.",
        "recommendation": "Selected bounded repair verified: independent owners settle despite sibling failures, Connection context-exit exceptions remain visible, and failures are retained. Cancellation controls are exercised by test_capability_cleanup.",
        "production_changed": True,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    result = asyncio.run(run())
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open("x") as destination:
        json.dump(result, destination, indent=2)
        destination.write("\n")
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
