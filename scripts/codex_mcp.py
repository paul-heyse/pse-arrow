# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit supported Codex host reload; independent of serving and generated sources.

Linux discovery observes only the current ancestor chain and its owned Unix listener.
The official reload method affects loaded threads globally. An acknowledgement is
queued work, while a benign read on the selected thread establishes observed freshness.
"""

from __future__ import annotations

import asyncio
import json
import os
import re
import socket
import stat
import struct
from contextlib import asynccontextmanager
from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING
from uuid import UUID

from pydantic import JsonValue, TypeAdapter

if TYPE_CHECKING:
    from collections.abc import AsyncGenerator

    from websockets.asyncio.client import ClientConnection

PROC = Path("/proc")
SERVER = "pse-capabilities"
JSON_OBJECT = TypeAdapter(dict[str, JsonValue])
MAX_MESSAGE_BYTES = 2 << 20


class HostUnavailableError(ValueError):
    """A redacted host-discovery, identity or protocol refusal."""


class RpcRefusedError(HostUnavailableError):
    def __init__(self, method: str, code: int | None) -> None:
        super().__init__("Codex refused the fixed host request; message redacted.")
        self.method = method
        self.code = code


@dataclass(frozen=True)
class Process:
    pid: int
    parent: int
    start: int
    app_server: bool


def _read(path: Path, maximum: int) -> bytes:
    with path.open("rb") as stream:
        value = stream.read(maximum + 1)
    if len(value) > maximum:
        raise HostUnavailableError("Host discovery exceeds its bounded source extent.")
    return value


def process(pid: int) -> Process:
    """Read same-UID identity, without environment values or unrelated process scans."""
    directory = PROC / str(pid)
    if directory.stat().st_uid != os.getuid():
        raise HostUnavailableError("The host process belongs to another user.")
    fields = _read(directory / "stat", 8192).decode().rsplit(")", 1)[1].split()
    arguments = _read(directory / "cmdline", 32768).split(b"\0")
    app_server = (
        len(arguments) > 1
        and Path(os.fsdecode(arguments[0])).name == "codex"
        and arguments[1] == b"app-server"
        and (directory / "exe").resolve().name == "codex"
    )
    return Process(pid, int(fields[1]), int(fields[19]), app_server)


def socket_stamp(path: Path) -> tuple[int, int, int]:
    """Reject missing, symlink, nonsocket and foreign-user endpoints before connection."""
    info = path.lstat()
    if not stat.S_ISSOCK(info.st_mode) or info.st_uid != os.getuid():
        raise HostUnavailableError("Select an owned Unix socket, without a symlink.")
    return info.st_dev, info.st_ino, info.st_uid


def discover_socket() -> Path:
    """Find one actual ancestor app-server's authenticated listening socket inode."""
    pid = os.getpid()
    seen: set[int] = set()
    for _ in range(64):
        if pid <= 1 or pid in seen:
            break
        seen.add(pid)
        owner = process(pid)
        if owner.app_server:
            directory = PROC / str(pid)
            descriptors = list((directory / "fd").iterdir())
            if len(descriptors) > 4096:
                raise HostUnavailableError(
                    "Host descriptor discovery exceeds its bound."
                )
            inodes = set()
            for descriptor in descriptors:
                try:
                    target = str(descriptor.readlink())
                except OSError:
                    continue
                if target.startswith("socket:[") and target.endswith("]"):
                    inodes.add(target[8:-1])
            paths = set()
            for line in (
                _read(directory / "net/unix", 2 << 20).decode().splitlines()[1:]
            ):
                columns = line.split(maxsplit=7)
                if (
                    len(columns) == 8
                    and columns[3] == "00010000"
                    and columns[4] == "0001"
                    and columns[6] in inodes
                    and columns[7].startswith("/")
                ):
                    candidate = Path(columns[7])
                    socket_stamp(candidate)
                    paths.add(candidate)
            if process(pid) != owner:
                raise HostUnavailableError(
                    "Host process identity changed during discovery."
                )
            if len(paths) != 1:
                raise HostUnavailableError(
                    "The ancestor host has no unique owned Unix listener."
                )
            return paths.pop()
        pid = owner.parent
    raise HostUnavailableError(
        "No supported ancestor Codex app-server Unix listener was found."
    )


@asynccontextmanager
async def host_connection(path: Path) -> AsyncGenerator[tuple[ClientConnection, int]]:
    """Admit the actual Unix peer before sending an app-server handshake."""
    # Independent offline help requires no transport import.
    from websockets.asyncio.client import unix_connect  # noqa: PLC0415

    before = socket_stamp(path)
    transport = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    transport.setblocking(False)
    try:
        await asyncio.get_running_loop().sock_connect(transport, str(path))
        pid, uid, _ = struct.unpack(
            "3i",
            transport.getsockopt(
                socket.SOL_SOCKET, socket.SO_PEERCRED, struct.calcsize("3i")
            ),
        )
        if uid != os.getuid():
            raise HostUnavailableError("The connected host belongs to another user.")
        owner = process(pid)
        if not owner.app_server or socket_stamp(path) != before:
            raise HostUnavailableError(
                "The selected endpoint is not the witnessed Codex app-server."
            )
        async with unix_connect(
            sock=transport,
            uri="ws://localhost/",
            open_timeout=5,
            close_timeout=1,
            max_size=MAX_MESSAGE_BYTES,
            max_queue=16,
            compression=None,
            proxy=None,
        ) as connection:
            if process(pid) != owner:
                raise HostUnavailableError(
                    "The connected host process identity changed."
                )
            yield connection, pid
    finally:
        transport.close()


class HostClient:
    """Only fixed official requests; bounded reply consumption and no request replay."""

    def __init__(self, connection: ClientConnection) -> None:
        self.connection = connection
        self.identity = 0

    async def request(self, method: str, params: JsonValue) -> dict[str, JsonValue]:
        self.identity += 1
        identity = self.identity
        await self.connection.send(
            json.dumps(
                {"id": identity, "method": method, "params": params}, allow_nan=False
            )
        )
        for _ in range(128):
            raw = await self.connection.recv()
            if len(raw) > MAX_MESSAGE_BYTES:
                raise HostUnavailableError("Host response exceeds its bounded extent.")
            value = JSON_OBJECT.validate_json(raw, strict=True)
            if value.get("id") != identity:
                continue
            error = value.get("error")
            if isinstance(error, dict):
                code = error.get("code")
                raise RpcRefusedError(method, code if isinstance(code, int) else None)
            result = value.get("result")
            if not isinstance(result, dict):
                raise HostUnavailableError(
                    "The fixed host response has no object result."
                )
            return result
        raise HostUnavailableError(
            "The matching host response was not observed within its message bound."
        )

    async def initialize(self) -> None:
        await self.request(
            "initialize",
            {
                "clientInfo": {"name": "pse-capability-reconnect", "version": "1"},
                "capabilities": {"experimentalApi": True},
            },
        )
        await self.connection.send(json.dumps({"method": "initialized"}))

    async def guard(self, thread: str) -> None:
        metadata = await self.request(
            "thread/read", {"threadId": thread, "includeTurns": False}
        )
        selected_thread = metadata.get("thread")
        if not isinstance(selected_thread, dict) or selected_thread.get("cwd") != str(
            Path(__file__).resolve().parents[1]
        ):
            raise HostUnavailableError(
                "Select a loaded thread belonging to this checkout."
            )
        runtime_status = selected_thread.get("status")
        if not isinstance(runtime_status, dict) or runtime_status.get("type") not in {
            "active",
            "idle",
            "systemError",
        }:
            raise HostUnavailableError(
                "The selected thread is not loaded in this host."
            )
        status = await self.request(
            "mcpServerStatus/list",
            {
                "threadId": thread,
                "serverName": SERVER,
                "detail": "toolsAndAuthOnly",
                "limit": 1,
            },
        )
        data = status.get("data")
        if (
            not isinstance(data, list)
            or len(data) != 1
            or not isinstance(data[0], dict)
        ):
            raise HostUnavailableError(
                "The selected thread has no unique configured capability server."
            )
        selected = data[0]
        tools = selected.get("tools")
        if (
            selected.get("name") != SERVER
            or selected.get("toolsError") is not None
            or not isinstance(tools, dict)
            or "capabilities_domains" not in tools
        ):
            raise HostUnavailableError(
                "The selected thread's canonical server is unavailable or has a different interface."
            )

    async def bootstrap(self, thread: str) -> dict[str, JsonValue]:
        result = await self.request(
            "mcpServer/tool/call",
            {
                "threadId": thread,
                "server": SERVER,
                "tool": "capabilities_domains",
                "arguments": {"limit": 1},
            },
        )
        if result.get("isError"):
            raise HostUnavailableError(
                "The selected thread's benign bootstrap read was refused."
            )
        value = result.get("structuredContent")
        if isinstance(value, dict) and set(value) == {"result"}:
            value = value["result"]
        interface = value.get("interface") if isinstance(value, dict) else None
        if not isinstance(interface, dict) or not isinstance(
            interface.get("restart_required"), bool
        ):
            raise HostUnavailableError(
                "The selected thread returned no explicit interface freshness."
            )
        revision = interface.get("revision")
        if (
            not isinstance(revision, str)
            or re.fullmatch(r"[0-9a-f]{64}", revision) is None
        ):
            raise HostUnavailableError(
                "The selected thread returned no bounded interface revision."
            )
        checkout = interface.get("checkout")
        if checkout is not None and checkout != str(
            Path(__file__).resolve().parents[1]
        ):
            raise HostUnavailableError(
                "The selected thread's capability server belongs to another checkout."
            )
        # An older loaded interface may not report its checkout. Missing provenance
        # is unavailable, not evidence of a different checkout. The independent
        # thread/host guard still authorizes the explicit global queue request.
        return {
            "restart_required": interface["restart_required"],
            "revision": revision,
            "checkout": checkout,
            "checkout_observation": "unavailable" if checkout is None else "matches",
        }


def selected_thread(thread_id: str | None, timeout: float) -> str:
    thread = thread_id if thread_id is not None else os.environ.get("CODEX_THREAD_ID")
    if thread is None or not thread.strip():
        raise HostUnavailableError(
            "Select a loaded thread or run inside Codex with CODEX_THREAD_ID."
        )
    UUID(thread)
    if not 1 <= timeout <= 60:
        raise HostUnavailableError(
            "The total host-control deadline must be within one to sixty seconds."
        )
    return thread


async def reconnect(
    *,
    socket_path: Path | None = None,
    thread_id: str | None = None,
    timeout: float = 15,
) -> dict[str, JsonValue]:
    """Queue the GLOBAL official reload, then observe one thread without replaying work."""
    report: dict[str, JsonValue] = {
        "method": "config/mcpServer/reload",
        "effect": "Reloads configuration and queues MCP refresh for all loaded Codex threads; no action replay or scientific settlement is inferred.",
        "server": SERVER,
        "reload": "not_sent",
        "stage": "selection",
        "freshness": "unobserved",
    }
    try:
        thread = selected_thread(thread_id, timeout)
        path = socket_path.absolute() if socket_path is not None else discover_socket()
        socket_stamp(path)
        report.update(thread=thread, socket=str(path), stage="connection")
        async with asyncio.timeout(timeout):
            async with host_connection(path) as (connection, pid):
                report.update(host_pid=pid, stage="initialization")
                client = HostClient(connection)
                await client.initialize()
                report["stage"] = "thread_guard"
                await client.guard(thread)
                report["guard"] = (
                    "Same-UID Codex peer, exact checkout thread metadata and configured canonical MCP server."
                )
                try:
                    report["before"] = await client.bootstrap(thread)
                except RpcRefusedError as error:
                    # The host may forbid direct MCP tool calls through this
                    # control connection. Keep the official metadata guard and
                    # disclose the observation boundary; do not bypass policy.
                    report["before"] = {
                        "availability": "host_refused",
                        "rpc_error_code": error.code,
                    }
                report.update(stage="reload", reload="uncertain")
                await client.request("config/mcpServer/reload", None)
                report.update(reload="queued", stage="verification")
                # A reload queues refresh. The current turn may retain its old connection.
                for attempt in range(4):
                    if attempt:
                        await asyncio.sleep(0.25 * attempt)
                    try:
                        observed = await client.bootstrap(thread)
                    except RpcRefusedError as error:
                        report["after"] = {
                            "availability": "host_refused",
                            "rpc_error_code": error.code,
                        }
                        report["freshness"] = "unavailable"
                        break
                    report["after"] = observed
                    report["freshness"] = (
                        "unavailable"
                        if observed["checkout_observation"] == "unavailable"
                        else "stale"
                        if observed["restart_required"]
                        else "fresh"
                    )
                    if not observed["restart_required"]:
                        break
                report["stage"] = "observed"
    except Exception as error:
        report["error_type"] = type(error).__name__
        if isinstance(error, HostUnavailableError):
            report["reason"] = str(error)
        if isinstance(error, RpcRefusedError):
            report["rpc_error_code"] = error.code
            report["rpc_method"] = error.method
            if report["stage"] == "reload":
                report["reload"] = "refused"
        report["remedy"] = (
            "Inspect current thread/host connection and original work owners; do not replay effects. A queued refresh may require a turn boundary or frontend reconnect."
        )
    return report
