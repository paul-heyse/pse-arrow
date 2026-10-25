# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Supported fixed host protocol, owner admission and queued/observed distinctions."""

from __future__ import annotations

import asyncio
import json
import os
import socket
import tempfile
import unittest
from contextlib import asynccontextmanager
from pathlib import Path
from typing import TYPE_CHECKING, cast
from unittest.mock import patch

import pytest

from scripts import codex_mcp

if TYPE_CHECKING:
    from collections.abc import AsyncGenerator

    from websockets.asyncio.client import ClientConnection

THREAD = "12345678-1234-4234-8234-123456789abc"


class FixedConnection:
    def __init__(
        self,
        *,
        fresh: bool = True,
        refuse: str | None = None,
        timeout: str | None = None,
        cwd: str | None = None,
        loaded: bool = True,
        checkout_header: bool = True,
        reported_checkout: str | None = None,
    ) -> None:
        self.fresh = fresh
        self.refuse = refuse
        self.timeout = timeout
        self.requests: list[dict] = []
        self.current: dict = {}
        self.reloaded = False
        self.cwd = (
            cwd
            if cwd is not None
            else str(Path(codex_mcp.__file__).resolve().parents[1])
        )
        self.loaded = loaded
        self.checkout_header = checkout_header
        self.reported_checkout = reported_checkout

    async def send(self, message: str) -> None:
        self.current = json.loads(message)
        self.requests.append(self.current)

    async def recv(self) -> str:
        method = self.current["method"]
        if method == self.timeout:
            await asyncio.sleep(10)
        if method == self.refuse:
            return json.dumps(
                {
                    "id": self.current["id"],
                    "error": {"code": -32601, "message": "SECRET RAW REFUSAL"},
                }
            )
        result: dict = {}
        if method == "thread/read":
            result = {
                "thread": {
                    "cwd": self.cwd,
                    "status": {"type": "active" if self.loaded else "notLoaded"},
                }
            }
        elif method == "mcpServerStatus/list":
            result = {
                "data": [
                    {
                        "name": "pse-capabilities",
                        "tools": {"capabilities_domains": {}},
                        "toolsError": None,
                    }
                ]
            }
        elif method == "config/mcpServer/reload":
            assert self.current["params"] is None
            self.reloaded = True
        elif method == "mcpServer/tool/call":
            interface: dict[str, str | bool] = {
                "restart_required": not (self.reloaded and self.fresh),
                "revision": ("b" if self.reloaded else "a") * 64,
            }
            if self.checkout_header:
                interface["checkout"] = self.reported_checkout or str(
                    Path(codex_mcp.__file__).resolve().parents[1]
                )
            result = {"content": [], "structuredContent": {"interface": interface}}
        return json.dumps({"id": self.current["id"], "result": result})


class HostProtocolTests(unittest.IsolatedAsyncioTestCase):
    async def run_fixture(self, client: FixedConnection, *, timeout: float = 2) -> dict:
        @asynccontextmanager
        async def connected(
            _path: Path,
        ) -> AsyncGenerator[tuple[ClientConnection, int]]:
            yield cast("ClientConnection", client), 10

        with (
            patch.object(codex_mcp, "socket_stamp", return_value=(1, 2, os.getuid())),
            patch.object(codex_mcp, "host_connection", connected),
        ):
            return await codex_mcp.reconnect(
                socket_path=Path("/fixture/owned.sock"),
                thread_id=THREAD,
                timeout=timeout,
            )

    async def test_one_global_reload_and_fixed_thread_read_verify_freshness(
        self,
    ) -> None:
        client = FixedConnection()
        report = await self.run_fixture(client)
        assert report["reload"] == "queued"
        assert report["freshness"] == "fresh"
        methods = [request["method"] for request in client.requests]
        assert methods == [
            "initialize",
            "initialized",
            "thread/read",
            "mcpServerStatus/list",
            "mcpServer/tool/call",
            "config/mcpServer/reload",
            "mcpServer/tool/call",
        ]
        status = client.requests[3]["params"]
        assert status["threadId"] == THREAD
        assert status["serverName"] == "pse-capabilities"
        tool = client.requests[-1]["params"]
        assert tool == {
            "threadId": THREAD,
            "server": "pse-capabilities",
            "tool": "capabilities_domains",
            "arguments": {"limit": 1},
        }
        assert "all loaded" in str(report["effect"])

    async def test_acknowledged_queue_remains_distinct_from_stale_connection(
        self,
    ) -> None:
        client = FixedConnection(fresh=False)
        report = await self.run_fixture(client)
        assert report["reload"] == "queued"
        assert report["freshness"] == "stale"
        assert (
            len(
                [
                    request
                    for request in client.requests
                    if request["method"] == "config/mcpServer/reload"
                ]
            )
            == 1
        )

    async def test_missing_thread_or_socket_refuses_before_connection(self) -> None:
        with patch.object(codex_mcp, "host_connection") as connect:
            missing_thread = await codex_mcp.reconnect(thread_id="")
            missing_socket = await codex_mcp.reconnect(
                socket_path=Path("/missing-pse-codex-host/socket"), thread_id=THREAD
            )
        assert missing_thread["reload"] == "not_sent"
        assert missing_socket["reload"] == "not_sent"
        connect.assert_not_called()

    async def test_refusal_is_redacted_and_never_retries_reload(self) -> None:
        client = FixedConnection(refuse="config/mcpServer/reload")
        report = await self.run_fixture(client)
        assert report["reload"] == "refused"
        assert report["rpc_error_code"] == -32601
        assert "SECRET" not in json.dumps(report)
        assert (
            len(
                [
                    request
                    for request in client.requests
                    if request["method"] == "config/mcpServer/reload"
                ]
            )
            == 1
        )

    async def test_thread_guard_refusal_and_timeout_never_send_reload(self) -> None:
        for client in (
            FixedConnection(refuse="mcpServerStatus/list"),
            FixedConnection(timeout="initialize"),
        ):
            report = await self.run_fixture(client, timeout=1)
            assert report["reload"] == "not_sent"
            assert not any(
                request["method"] == "config/mcpServer/reload"
                for request in client.requests
            )

    async def test_checkout_and_loaded_thread_guards_precede_global_reload(
        self,
    ) -> None:
        for client in (
            FixedConnection(cwd="/other/checkout"),
            FixedConnection(loaded=False),
        ):
            report = await self.run_fixture(client)
            assert report["reload"] == "not_sent"
            assert not any(
                request["method"] == "config/mcpServer/reload"
                for request in client.requests
            )

    async def test_missing_legacy_checkout_header_retains_observation_without_false_freshness(
        self,
    ) -> None:
        client = FixedConnection(checkout_header=False)
        report = await self.run_fixture(client)
        assert report["reload"] == "queued"
        assert report["freshness"] == "unavailable"
        assert report["before"] == {
            "restart_required": True,
            "revision": "a" * 64,
            "checkout": None,
            "checkout_observation": "unavailable",
        }
        assert report["after"] == {
            "restart_required": False,
            "revision": "b" * 64,
            "checkout": None,
            "checkout_observation": "unavailable",
        }

    async def test_explicit_foreign_checkout_header_refuses_before_reload(self) -> None:
        client = FixedConnection(reported_checkout="/different/checkout")
        report = await self.run_fixture(client)
        assert report["reload"] == "not_sent"
        assert (
            report["reason"]
            == "The selected thread's capability server belongs to another checkout."
        )
        assert not any(
            request["method"] == "config/mcpServer/reload"
            for request in client.requests
        )

    async def test_host_policy_can_refuse_bootstrap_without_fabricating_freshness(
        self,
    ) -> None:
        client = FixedConnection(refuse="mcpServer/tool/call")
        report = await self.run_fixture(client)
        assert report["reload"] == "queued"
        assert report["freshness"] == "unavailable"
        assert report["before"] == {
            "availability": "host_refused",
            "rpc_error_code": -32601,
        }
        assert report["after"] == report["before"]
        assert "SECRET" not in json.dumps(report)

    async def test_lost_reload_ack_is_uncertain_and_never_replayed(self) -> None:
        client = FixedConnection(timeout="config/mcpServer/reload")
        report = await self.run_fixture(client, timeout=1)
        assert report["reload"] == "uncertain"
        assert report["error_type"] == "TimeoutError"
        assert (
            len(
                [
                    request
                    for request in client.requests
                    if request["method"] == "config/mcpServer/reload"
                ]
            )
            == 1
        )

    async def test_foreign_non_codex_peer_refuses_before_websocket_handshake(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "foreign.sock"
            with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as listener:
                listener.bind(str(path))
                listener.listen()
                with (
                    patch.object(
                        codex_mcp,
                        "process",
                        return_value=codex_mcp.Process(
                            os.getpid(), os.getppid(), 1, False
                        ),
                    ),
                    pytest.raises(
                        codex_mcp.HostUnavailableError, match="not the witnessed Codex"
                    ),
                ):
                    async with codex_mcp.host_connection(path):
                        pytest.fail("foreign endpoint must not be entered")


class HostDiscoveryTests(unittest.TestCase):
    def test_socket_owner_and_symlink_are_checked(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "owned.sock"
            with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as listener:
                listener.bind(str(path))
                assert codex_mcp.socket_stamp(path)[2] == os.getuid()
                link = Path(directory) / "alias.sock"
                link.symlink_to(path)
                with pytest.raises(
                    codex_mcp.HostUnavailableError, match="owned Unix socket"
                ):
                    codex_mcp.socket_stamp(link)
                with (
                    patch.object(codex_mcp.os, "getuid", return_value=os.getuid() + 1),
                    pytest.raises(
                        codex_mcp.HostUnavailableError, match="owned Unix socket"
                    ),
                ):
                    codex_mcp.socket_stamp(path)

    def test_ancestor_listener_requires_actual_owned_descriptor_inode(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            proc = root / "proc"
            owner = proc / "123"
            (owner / "fd").mkdir(parents=True)
            (owner / "net").mkdir()
            executable = root / "bin/codex"
            executable.parent.mkdir()
            executable.touch()
            (owner / "exe").symlink_to(executable)
            (owner / "cmdline").write_bytes(b"codex\0app-server\0--listen\0unix://\0")
            (owner / "stat").write_text(
                "123 (codex) " + " ".join(["S", "1", *(["0"] * 17), "999"])
            )
            (owner / "fd/1").symlink_to("socket:[55]")
            path = root / "actual.sock"
            with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as listener:
                listener.bind(str(path))
                (owner / "net/unix").write_text(
                    "header\n00000000: 00000002 00000000 00010000 0001 01 55 "
                    + str(path)
                    + "\n00000000: 00000002 00000000 00010000 0001 01 99 /foreign/not-owned\n"
                )
                with (
                    patch.object(codex_mcp, "PROC", proc),
                    patch.object(codex_mcp.os, "getpid", return_value=123),
                ):
                    assert codex_mcp.discover_socket() == path


if __name__ == "__main__":
    unittest.main()
