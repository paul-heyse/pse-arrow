# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Fixed native MCP credential reads and disposable VIEWER controls."""
# ruff: noqa: PT009, PT027

from __future__ import annotations

import base64
import contextlib
import copy
import io
import json
import os
import re
import subprocess
import tempfile
import time
import unittest
import urllib.error
import urllib.request
import uuid
from pathlib import Path
from typing import TYPE_CHECKING
from unittest.mock import MagicMock, patch

if TYPE_CHECKING:
    from collections.abc import Generator

from scripts import host_admission as host
from scripts import pse_env
from scripts import surreal_server as server
from scripts.tests.test_surrealism_module import fixture_module


def fixture_mcp_controls(
    state: Path, directory: Path, *, attached: bool = False
) -> dict[str, object]:
    """Own a unique disposable namespace; retain its declared identity on failure."""
    deadline = time.monotonic() + 30
    config = server.config_for(state)
    if not (directory / "fixture-ownership.json").is_file() or not server.owns_listener(
        state, config, deadline=deadline
    ):
        raise server.SupervisorError(
            "MCP write control requires its disposable fixture owner"
        )
    lease = directory / "mcp-fixture.json"
    resumed = lease.exists()
    if resumed:
        declaration = server.read_json(lease)
        if (
            declaration.get("service_state") != str(state)
            or declaration.get("service_generation") != config["instance_id"]
            or declaration.get("phase") != "created"
            or declaration.get("database") != "toy"
            or not re.fullmatch(
                r"pse_mcp_fixture_[a-f0-9]{32}", str(declaration.get("namespace", ""))
            )
            or not re.fullmatch(
                r"[a-f0-9]{32}", str(declaration.get("owner_identity", ""))
            )
        ):
            raise server.SupervisorError("Retained MCP fixture identity differs")
        namespace, database, owner_identity = (
            declaration["namespace"],
            declaration["database"],
            declaration["owner_identity"],
        )
    else:
        namespace, database = "pse_mcp_fixture_" + uuid.uuid4().hex, "toy"
        owner_identity = uuid.uuid4().hex
        declaration = {
            "service_state": str(state),
            "service_generation": config["instance_id"],
            "namespace": namespace,
            "database": database,
            "owner_identity": owner_identity,
            "phase": "declared",
            "attached": attached,
        }
        server.write_json(lease, declaration)
    prefix = f"USE NS {namespace}; USE DB {database}; "
    credentials, _ = server.mcp_private_record(state / "credentials.json", deadline)
    admin_token = base64.b64encode(
        f"{credentials['username']}:{credentials['password']}".encode()
    ).decode()

    def admin(query: str) -> list[dict]:
        operation = urllib.request.Request(
            f"http://127.0.0.1:{config['port']}/sql",
            data=query.encode(),
            headers={
                "Authorization": "Basic " + admin_token,
                "Accept": "application/json",
            },
        )
        with urllib.request.urlopen(  # noqa: S310 -- verified owned loopback endpoint
            operation, timeout=server.remaining(deadline)
        ) as response:
            raw = response.read((1 << 20) + 1)
        server.remaining(deadline)
        if len(raw) > 1 << 20:
            raise server.SupervisorError(
                "MCP administrative fixture response exceeds its bound"
            )
        result = json.loads(raw)
        if not isinstance(result, list) or any(
            not isinstance(row, dict) or row.get("status") != "OK" for row in result
        ):
            raise server.SupervisorError("MCP administrative fixture query failed")
        return result

    def namespaces() -> dict:
        result = admin("INFO FOR ROOT;")[0]["result"]["namespaces"]
        if not isinstance(result, dict):
            raise server.SupervisorError(
                "MCP namespace ownership observation unavailable"
            )
        return result

    if resumed:
        if namespace not in namespaces() or admin(
            prefix + "SELECT `value`, owner_identity FROM probe:seed;"  # noqa: S608 -- validated owned namespace identifier
        )[-1]["result"] != [{"value": "known", "owner_identity": owner_identity}]:
            raise server.SupervisorError(
                "Retained MCP namespace owner readback differs"
            )
    else:
        if namespace in namespaces():
            raise server.SupervisorError("MCP fixture refuses a preexisting namespace")
        admin(
            f"DEFINE NAMESPACE {namespace}; USE NS {namespace}; DEFINE DATABASE {database}; USE DB {database}; DEFINE TABLE probe SCHEMALESS; CREATE probe:seed SET value='known', owner_identity={json.dumps(owner_identity)};"
        )
        server.write_json(lease, {**declaration, "phase": "created"})
    token = base64.b64encode(
        f"{credentials['selection_username']}:{credentials['selection_password']}".encode()
    ).decode()
    endpoint = f"http://127.0.0.1:{config['port']}/mcp"
    headers = {
        "Authorization": "Basic " + token,
        "Content-Type": "application/json",
        "Accept": "application/json, text/event-stream",
    }

    def request(
        method: str, params: dict, identity: int | None, *, bad_auth: bool = False
    ) -> dict | None:
        payload: dict[str, object] = {
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        }
        if identity is not None:
            payload["id"] = identity
        selected_headers = (
            {
                **headers,
                "Authorization": "Basic "
                + base64.b64encode(b"pse-selection:deliberately-wrong").decode(),
            }
            if bad_auth
            else headers
        )
        operation = urllib.request.Request(
            endpoint, data=json.dumps(payload).encode(), headers=selected_headers
        )
        with urllib.request.urlopen(  # noqa: S310 -- verified owned loopback endpoint
            operation, timeout=server.remaining(deadline)
        ) as response:
            if session := response.headers.get("MCP-Session-Id"):
                headers["MCP-Session-Id"] = session
            raw = response.read((1 << 20) + 1).decode()
        server.remaining(deadline)
        if len(raw) > 1 << 20:
            raise server.SupervisorError("MCP fixture response exceeds its bound")
        if not raw.strip():
            return None
        messages = [
            line[5:].strip() for line in raw.splitlines() if line.startswith("data:")
        ]
        reply = json.loads(messages[-1] if messages else raw)
        if "error" in reply:
            raise server.SupervisorError("MCP fixture RPC failed")
        return reply["result"]

    initialized = request(
        "initialize",
        {
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": {"name": "pse-disposable-mcp-control", "version": "1"},
        },
        1,
    )
    if initialized is None:
        raise server.SupervisorError("MCP fixture initialization unavailable")
    headers["MCP-Protocol-Version"] = initialized["protocolVersion"]
    request("notifications/initialized", {}, None)
    try:
        request("tools/list", {}, 99, bad_auth=True)
    except urllib.error.HTTPError as error:
        if error.code not in {401, 403}:
            raise server.SupervisorError(
                "MCP invalid authentication failed unexpectedly"
            ) from None
        error.close()
    else:
        raise server.SupervisorError("MCP invalid VIEWER authentication was accepted")
    scoped = {"namespace": namespace, "database": database}
    read = request(
        "tools/call",
        {
            "name": "query",
            "arguments": {**scoped, "query": "SELECT `value` FROM probe:seed;"},
        },
        2,
    )
    if (
        read is None
        or read.get("isError")
        or read.get("structuredContent", {}).get("value") != [{"value": "known"}]
    ):
        raise server.SupervisorError("MCP contextual fixture read differs")
    write = request(
        "tools/call",
        {
            "name": "query",
            "arguments": {
                **scoped,
                "query": "CREATE probe:forbidden SET value='must-not-write';",
            },
        },
        3,
    )
    missing = request(
        "tools/call",
        {
            "name": "list",
            "arguments": {**scoped, "database": "missing_context", "kind": "tables"},
        },
        4,
    )
    if missing is None or not missing.get("isError"):
        raise server.SupervisorError("MCP missing fixture context was not refused")
    # A VIEWER may silently return [] on CREATE: only independent privileged
    # readback establishes that no write happened.
    readback = admin(
        prefix  # noqa: S608 -- validated owned namespace identifier
        + "SELECT `value` FROM probe:forbidden; SELECT `value`, owner_identity FROM probe:seed;"
    )
    if [record["result"] for record in readback[-2:]] != [
        [],
        [{"value": "known", "owner_identity": owner_identity}],
    ]:
        raise server.SupervisorError("MCP VIEWER write-denial readback differs")
    if server.config_for(state) != config or not server.owns_listener(
        state, config, deadline=deadline
    ):
        raise server.SupervisorError(
            "MCP service ownership changed before fixture retirement"
        )
    # Every client response is closed before the new, positively identified
    # namespace is dropped. No existing namespace/database is a cleanup target.
    admin(f"REMOVE NAMESPACE {namespace};")
    if namespace in namespaces():
        raise server.SupervisorError("MCP owned namespace remains after retirement")
    server.remaining(deadline)
    server.write_json(
        directory / "mcp-fixture.json",
        {**declaration, "phase": "retired", "namespace_absent": True},
    )
    return {
        "endpoint": endpoint,
        "namespace": namespace,
        "database": database,
        "principal": "existing root VIEWER",
        "contextual_read": "passed",
        "write_denial_readback": "passed",
        "write_tool_reported_error": bool(write and write.get("isError")),
        "authentication_failure": "refused",
        "missing_context": "refused",
        "namespace_absent": True,
        "clients_closed": True,
        "service_generation": config["instance_id"],
        "attached_state": str(state) if attached else None,
        "scope": "disposable native MCP controls; no retained database writes",
    }


class SurrealMcpTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.state = self.root / "state"
        self.state.mkdir(mode=0o700)
        self.secret = "private-viewer-password"  # noqa: S105 -- synthetic fixture credential
        self.authorization = (
            "Basic "
            + base64.b64encode(f"pse-selection:{self.secret}".encode()).decode()
        )
        self.config: dict[str, object] = {
            "port": 18240,
            "endpoint": "grpc://127.0.0.1:18240",
            "instance_id": "server-instance",
            "credentials_file": str(self.state / "credentials.json"),
            "server": {"binary_sha256": "1" * 64},
        }
        study_module = fixture_module(self.root / "modules")
        self.config["study_module"] = study_module
        self.credentials = {
            "username": "root",
            "password": "private-root-password",
            "selection_username": "pse-selection",
            "selection_password": self.secret,
        }
        self.write("credentials.json", self.credentials)
        self.proof = {
            "schema": "native-grpc-readiness-v1",
            "instance_id": "server-instance",
            "invocation": "b" * 32,
            "binary_sha256": "1" * 64,
            "credentials_sha256": server.file_digest(self.state / "credentials.json"),
            "study_module_sha256": study_module["sha256"],
            "study_module_qualification_sha256": None,
        }
        self.write("protocol-readiness.json", self.proof)
        self.endpoint = "http://127.0.0.1:18240/mcp"
        self.deadline = time.monotonic() + 8

    def write(self, name: str, data: dict) -> None:
        server.write_json(self.state / name, data)

    @contextlib.contextmanager
    def proof_controls(self) -> Generator[MagicMock, None, None]:
        with (
            patch.object(server, "config_for", return_value=self.config),
            patch.object(server, "owns_listener", return_value=True) as listener,
            patch.object(
                server,
                "storage_unit_observation",
                return_value={"InvocationID": "b" * 32},
            ),
        ):
            yield listener

    def invoke(self, child: bool = False) -> tuple[int, str, str]:
        out, err = io.StringIO(), io.StringIO()
        argv = [
            "_mcp-headers-child" if child else "mcp-headers",
            "--state",
            str(self.state),
            "--expected-endpoint",
            self.endpoint,
        ]
        if child:
            argv += ["--deadline", str(self.deadline)]
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            code = server.main(argv)
        return code, out.getvalue(), err.getvalue()

    def test_private_viewer_only_header_and_no_file_changes(self) -> None:
        before = {
            path: (path.read_bytes(), path.stat().st_mtime_ns)
            for path in self.state.iterdir()
        }
        with self.proof_controls() as listener:
            headers = server.mcp_authorization(self.state, self.endpoint, self.deadline)
        self.assertEqual(headers, {"Authorization": self.authorization})
        self.assertEqual(listener.call_count, 2)
        for call in listener.call_args_list:
            self.assertEqual(call.kwargs["deadline"], self.deadline)
        after = {
            path: (path.read_bytes(), path.stat().st_mtime_ns)
            for path in self.state.iterdir()
        }
        self.assertEqual(before, after)
        self.assertNotIn("private-root-password", json.dumps(headers))

    def test_selected_service_identity_redaction_and_changed_invocation(self) -> None:
        with self.proof_controls():
            selected = server.selected_service(self.state, self.deadline)
            self.assertEqual(selected.identity["invocation"], "b" * 32)
            self.assertNotIn(self.secret, repr(selected))
            self.assertNotIn(self.secret, json.dumps(selected.identity))
            with (
                patch.object(
                    server,
                    "storage_unit_observation",
                    return_value={"InvocationID": "c" * 32},
                ),
                self.assertRaisesRegex(server.SupervisorError, "changed"),
            ):
                selected.recheck(self.deadline)

    def test_selection_fence_refuses_same_port_restart_and_credential_rotation(
        self,
    ) -> None:
        with self.proof_controls():
            selected = server.selected_service(self.state, self.deadline)
            fence = selected.selection_fence
            self.assertRegex(fence, r"^[a-f0-9]{64}$")
            self.assertNotIn(self.secret, fence)
            self.assertEqual(
                server.mcp_authorization(
                    self.state, self.endpoint, self.deadline, expected_selection=fence
                ),
                {"Authorization": self.authorization},
            )
            changed = {**self.proof, "invocation": "c" * 32}
            self.write("protocol-readiness.json", changed)
            with (
                patch.object(
                    server,
                    "storage_unit_observation",
                    return_value={"InvocationID": "c" * 32},
                ),
                self.assertRaisesRegex(server.SupervisorError, "generation changed"),
            ):
                server.mcp_authorization(
                    self.state, self.endpoint, self.deadline, expected_selection=fence
                )
            self.write(
                "credentials.json",
                {**self.credentials, "selection_password": "rotated-private-viewer"},
            )
            self.write(
                "protocol-readiness.json",
                {
                    **self.proof,
                    "credentials_sha256": server.file_digest(
                        self.state / "credentials.json"
                    ),
                },
            )
            with self.assertRaisesRegex(server.SupervisorError, "generation changed"):
                server.mcp_authorization(
                    self.state, self.endpoint, self.deadline, expected_selection=fence
                )

    def test_fixed_helper_forwards_nonsecret_selection_fence(self) -> None:
        fence = "a" * 64
        with (
            patch.object(pse_env, "control_environment", return_value={}),
            patch.object(
                pse_env,
                "run_light_control",
                return_value=subprocess.CompletedProcess(
                    [], 0, json.dumps({"Authorization": self.authorization}), ""
                ),
            ) as run,
            contextlib.redirect_stdout(io.StringIO()),
        ):
            self.assertEqual(
                server.main(
                    [
                        "mcp-headers",
                        "--state",
                        str(self.state),
                        "--expected-endpoint",
                        self.endpoint,
                        "--expected-selection",
                        fence,
                    ]
                ),
                0,
            )
        command = run.call_args.args[0]
        self.assertEqual(command[command.index("--expected-selection") + 1], fence)

    def test_selected_service_rechecks_protocol_proof_after_observation(self) -> None:
        with self.proof_controls():
            selected = server.selected_service(self.state, self.deadline)
            self.write(
                "protocol-readiness.json", {**self.proof, "invocation": "changed"}
            )
            with self.assertRaisesRegex(server.SupervisorError, "changed"):
                selected.recheck(self.deadline)

    def test_metrics_preserve_zero_normal_absence_and_exporter_metadata(self) -> None:
        prefix = server.MX_METRIC_PREFIX
        observed = server.selected_metric_values(
            f"# HELP {prefix}active_readers Native reader count\n# TYPE {prefix}active_readers gauge\n{prefix}active_readers 0\n# TYPE {prefix}oldest_pinned_version gauge\n"
        )["families"]
        observed = server.object_mapping(observed)
        self.assertEqual(
            server.object_mapping(observed[prefix + "active_readers"])["samples"],
            [{"labels": "", "value": 0}],
        )
        self.assertEqual(
            server.object_mapping(observed[prefix + "oldest_pinned_version"])[
                "availability"
            ],
            "registered_no_sample",
        )
        self.assertEqual(
            server.object_mapping(observed[prefix + "poisoned"])["availability"],
            "unregistered",
        )
        with self.assertRaises(server.SupervisorError):
            server.selected_metric_values(prefix + "active_readers not-a-number")

    def test_metrics_request_is_fixed_and_changed_identity_refuses_output(self) -> None:
        reply = MagicMock(status=200)
        reply.read1.side_effect = [
            (server.MX_METRIC_PREFIX + "active_readers 0\n").encode(),
            b"",
            b"",
        ]
        connection = MagicMock()
        connection.getresponse.return_value = reply
        with (
            self.proof_controls(),
            patch.object(server.http.client, "HTTPConnection", return_value=connection),
        ):
            snapshot = server.metric_snapshot(self.state, self.deadline)
            connection.request.assert_called_once_with(
                "GET", "/metrics", headers={"Authorization": self.authorization}
            )
            self.assertEqual(
                server.object_mapping(snapshot["identity"])["instance_id"],
                "server-instance",
            )
            self.assertNotIn(self.secret, json.dumps(snapshot))
        with (
            self.proof_controls(),
            patch.object(server.http.client, "HTTPConnection", return_value=connection),
        ):
            connection.getresponse.side_effect = lambda: (
                self.write(
                    "protocol-readiness.json", {**self.proof, "invocation": "changed"}
                ),
                reply,
            )[1]
            with self.assertRaisesRegex(server.SupervisorError, "changed"):
                server.metric_snapshot(self.state, self.deadline)
        connection.close.assert_called()

    def test_metrics_dripped_response_cannot_renew_absolute_clock(self) -> None:
        reply = MagicMock(status=200)
        connection = MagicMock()
        connection.getresponse.return_value = reply
        clock = [100.0]

        def drip(_size: int) -> bytes:
            clock[0] += 0.4
            return b"# harmless comment\n"

        reply.read1.side_effect = drip
        with (
            self.proof_controls(),
            patch.object(server.time, "monotonic", side_effect=lambda: clock[0]),
            patch.object(server.http.client, "HTTPConnection", return_value=connection),
            self.assertRaises(server.DeadlineExpiredError),
        ):
            server.metric_snapshot(self.state, 101.0)
        self.assertEqual(reply.read1.call_count, 3)
        connection.close.assert_called_once()

    def test_metric_comparison_refuses_restart_and_preserves_gauge_meaning(
        self,
    ) -> None:
        before = {
            "identity": {"invocation": "first"},
            **server.selected_metric_values(
                server.MX_METRIC_PREFIX + "active_readers 0"
            ),
        }
        after = {
            "identity": before["identity"],
            **server.selected_metric_values(
                server.MX_METRIC_PREFIX + "active_readers 1"
            ),
        }
        comparison = server.compare_metric_snapshots(before, after)
        self.assertIn("gauge", str(comparison["scope"]))
        after["identity"] = {"invocation": "second"}
        with self.assertRaisesRegex(server.SupervisorError, "changed"):
            server.compare_metric_snapshots(before, after)

    def test_fixed_route_bypasses_mutating_dispatch_and_compiler_setup(self) -> None:
        result = subprocess.CompletedProcess(
            [],
            0,
            json.dumps({"Authorization": self.authorization}),
            "must not be forwarded",
        )
        with (
            patch.object(server, "dispatch", side_effect=AssertionError("lifecycle")),
            patch.object(
                server, "checked_directory", side_effect=AssertionError("repair")
            ),
            patch.object(
                pse_env.build_environment,
                "configure",
                side_effect=AssertionError("compiler"),
            ),
            patch.object(
                pse_env, "control_environment", return_value={}
            ) as environment,
            patch.object(pse_env, "run_light_control", return_value=result) as launch,
        ):
            code, output, error = self.invoke()
        self.assertEqual((code, error), (0, ""))
        self.assertEqual(json.loads(output), {"Authorization": self.authorization})
        command = launch.call_args.args[0]
        self.assertEqual(
            command[:3],
            [server.sys.executable, str(server.SCRIPT), "_mcp-headers-child"],
        )
        clock = environment.call_args.kwargs["deadline"]
        self.assertEqual(launch.call_args.kwargs["deadline"], clock)
        self.assertEqual(float(command[-1]), clock)
        self.assertNotIn(self.secret, " ".join(command))
        self.assertNotIn(self.authorization, " ".join(command))

    def test_all_launch_failures_and_child_outputs_are_redacted(self) -> None:
        failures = (
            subprocess.CompletedProcess([], 125, self.secret, self.authorization),
            subprocess.CompletedProcess([], 0, self.secret, self.authorization),
            subprocess.CompletedProcess(
                [],
                0,
                json.dumps({"Authorization": self.authorization, "extra": self.secret}),
                "",
            ),
            pse_env.BoundaryError(self.secret + self.authorization),
            subprocess.TimeoutExpired([self.secret], 8, output=self.authorization),
        )
        for failure in failures:
            with (
                self.subTest(failure=type(failure).__name__),
                patch.object(pse_env, "control_environment", return_value={}),
                patch.object(
                    pse_env,
                    "run_light_control",
                    side_effect=failure if isinstance(failure, Exception) else None,
                    return_value=failure,
                ),
            ):
                code, output, error = self.invoke()
            self.assertEqual((code, output), (125, ""))
            self.assertIn("pse-env:", error)
            self.assertNotIn(self.secret, error)
            self.assertNotIn(self.authorization, error)

    def test_child_requires_actual_ownership_before_credential_read(self) -> None:
        with (
            patch.object(host, "inherit", return_value=None),
            patch.object(
                server,
                "mcp_authorization",
                side_effect=AssertionError("read credentials"),
            ) as read,
        ):
            code, output, _ = self.invoke(child=True)
        self.assertEqual((code, output), (125, ""))
        read.assert_not_called()
        with (
            patch.object(host, "inherit", side_effect=host.AdmissionError("unbound")),
            patch.object(
                host, "verify_light_control_child", return_value=MagicMock()
            ) as verify,
            patch.object(
                server,
                "mcp_authorization",
                return_value={"Authorization": self.authorization},
            ),
        ):
            code, output, error = self.invoke(child=True)
        self.assertEqual((code, error), (0, ""))
        self.assertEqual(json.loads(output), {"Authorization": self.authorization})
        self.assertEqual(verify.call_args.kwargs["deadline"], self.deadline)

    def test_viewer_readiness_changes_are_refused(self) -> None:
        for field in (
            "schema",
            "instance_id",
            "invocation",
            "binary_sha256",
            "credentials_sha256",
            "study_module_sha256",
            "study_module_qualification_sha256",
        ):
            with self.subTest(field=field), self.proof_controls():
                self.write("protocol-readiness.json", {**self.proof, field: "changed"})
                with self.assertRaises(server.SupervisorError):
                    server.mcp_authorization(self.state, self.endpoint, self.deadline)
        self.write("protocol-readiness.json", self.proof)
        with self.proof_controls(), self.assertRaises(server.SupervisorError):
            server.mcp_authorization(
                self.state, "http://127.0.0.1:18241/mcp", self.deadline
            )
        with self.proof_controls() as listener:
            listener.return_value = False
            with self.assertRaises(server.SupervisorError):
                server.mcp_authorization(self.state, self.endpoint, self.deadline)

    def test_private_records_reject_permissions_links_fifo_and_missing(self) -> None:
        credentials = self.state / "credentials.json"
        for invalid in ("permissions", "link", "fifo", "missing"):
            with self.subTest(invalid=invalid):
                credentials.unlink()
                if invalid == "permissions":
                    self.write("credentials.json", self.credentials)
                    credentials.chmod(0o644)
                elif invalid == "link":
                    credentials.symlink_to(self.state / "protocol-readiness.json")
                elif invalid == "fifo":
                    os.mkfifo(credentials, mode=0o600)
                with self.assertRaises((server.SupervisorError, OSError)):
                    server.mcp_private_record(credentials, self.deadline)
                credentials.unlink(missing_ok=True)
                self.write("credentials.json", self.credentials)

    def test_fixed_owner_reads_reject_fifo_and_wrong_mode_before_credentials(
        self,
    ) -> None:
        for name in ("config.json", "server-process.json", "service-launch.json"):
            for invalid in ("fifo", "permissions"):
                with self.subTest(record=name, invalid=invalid):
                    path = self.state / name
                    path.unlink(missing_ok=True)
                    if invalid == "fifo":
                        os.mkfifo(path, mode=0o600)
                    else:
                        self.write(name, {})
                        path.chmod(0o644)
                    started = time.monotonic()
                    with self.assertRaises((server.SupervisorError, OSError)):
                        server.read_json(path, deadline=started + 0.2)
                    self.assertLess(time.monotonic() - started, 0.2)
                    path.unlink()

        os.mkfifo(self.state / "config.json", mode=0o600)
        with patch.object(host, "inherit", return_value=MagicMock()):
            code, output, error = self.invoke(child=True)
        self.assertEqual((code, output), (125, ""))
        self.assertNotIn(self.secret, error)
        (self.state / "config.json").unlink()
        self.write("config.json", self.config)
        process = {
            "pid": os.getpid(),
            "start": server.native_operation.start_identity(os.getpid()),
            "instance_id": self.config["instance_id"],
        }
        for name in ("server-process.json", "service-launch.json"):
            for invalid in ("fifo", "permissions"):
                with self.subTest(fixed_route=name, invalid=invalid):
                    self.write("server-process.json", process)
                    path = self.state / name
                    path.unlink(missing_ok=True)
                    if invalid == "fifo":
                        os.mkfifo(path, mode=0o600)
                    else:
                        self.write(name, {})
                        path.chmod(0o644)
                    with (
                        patch.object(server, "config_for", return_value=self.config),
                        patch.object(
                            server,
                            "mcp_private_record",
                            wraps=server.mcp_private_record,
                        ) as reads,
                        self.assertRaises(server.SupervisorError),
                    ):
                        server.mcp_authorization(
                            self.state, self.endpoint, self.deadline
                        )
                    self.assertNotIn(
                        self.state / "credentials.json",
                        [call.args[0] for call in reads.call_args_list],
                    )
                    path.unlink()

    def test_lifecycle_inventory_read_preserves_mcp_record_size_policy(self) -> None:
        path = self.state / "large-lifecycle.json"
        inventory = {
            "derived_rebuild_pending": {"payload_inventory": ["x" * 1024] * 80}
        }
        server.write_json(path, inventory)
        self.assertGreater(path.stat().st_size, 64 * 1024)
        self.assertEqual(server.read_json(path, deadline=self.deadline), inventory)
        with self.assertRaisesRegex(server.SupervisorError, "exceeds its bound"):
            server.mcp_private_record(path, self.deadline)

    def test_lifecycle_inventory_read_keeps_original_deadline_between_chunks(
        self,
    ) -> None:
        path = self.state / "large-lifecycle.json"
        server.write_json(path, {"inventory": "x" * (128 * 1024)})
        with (
            patch.object(server.time, "monotonic", side_effect=[1.0, 1.1, 2.1]),
            self.assertRaises(server.DeadlineExpiredError),
        ):
            server.read_json(path, deadline=2.0)

    def test_fixed_config_process_and_launch_propagate_read_deadline(self) -> None:
        with patch.object(
            server, "read_json", side_effect=server.SupervisorError("bounded record")
        ) as read:
            with self.assertRaises(server.SupervisorError):
                server.config_for(self.state, deadline=self.deadline)
            read.assert_called_once_with(
                self.state / "config.json", deadline=self.deadline
            )
        with (
            patch.object(server, "service_directory", return_value=self.state),
            patch.object(
                server,
                "read_json",
                side_effect=server.SupervisorError("bounded record"),
            ) as read,
        ):
            self.assertFalse(
                server.owns_listener(self.state, self.config, deadline=self.deadline)
            )
            read.assert_called_once_with(
                self.state / "server-process.json", deadline=self.deadline
            )
        with patch.object(
            server, "read_json", side_effect=server.SupervisorError("bounded record")
        ) as read:
            with self.assertRaises(server.SupervisorError):
                server.storage_launch(self.state, self.config, deadline=self.deadline)
            read.assert_called_once_with(
                self.state / "service-launch.json", deadline=self.deadline
            )

    def test_deadline_is_not_renewed_and_usage_is_two(self) -> None:
        with self.proof_controls(), self.assertRaises(server.SupervisorError):
            server.mcp_authorization(self.state, self.endpoint, time.monotonic() - 1)
        for arguments in (
            ["mcp-headers"],
            [
                "mcp-headers",
                "--state",
                str(self.state),
                "--expected-endpoint",
                "https://remote/mcp",
            ],
            [
                "mcp-headers",
                "--state",
                str(self.state),
                "--expected-endpoint",
                self.endpoint,
                "--worker-command",
                "anything",
            ],
        ):
            with (
                self.subTest(arguments=arguments),
                contextlib.redirect_stderr(io.StringIO()),
                self.assertRaises(SystemExit) as raised,
            ):
                server.main(arguments)
            self.assertEqual(raised.exception.code, 2)

    def test_storage_owner_uses_detached_snapshot_without_ledger_writes(self) -> None:
        profile = host.select("store-functional", str(server.GIB))
        path = self.root / "admission" / ("a" * 32)
        binding = {
            "group": "/pse.slice/server.service",
            "invocation": "b" * 32,
            "inode": 91,
        }
        launch = {"allocation": str(path), "binding": binding}
        owner: dict[str, object] = {
            "boot": host.boot(),
            "service": str(self.state),
            "class": profile.name,
            "memory": profile.memory,
            "lane": profile.lane,
            "slots": profile.slots,
            "cores": list(profile.cores),
            "exclusive": profile.exclusive,
            "deadline": self.deadline,
            "units": {server.unit_name(self.state): binding},
        }
        ledger = {"owners": {path.name: owner}}
        before = copy.deepcopy(ledger)
        with (
            patch.object(server, "storage_launch", return_value=launch),
            patch.object(host, "readonly_snapshot", return_value=ledger) as read,
            patch.object(
                host,
                "allocation_metadata",
                side_effect=AssertionError("mutating ledger"),
            ),
        ):
            actual, observed = server.recorded_storage_owner(
                self.state, self.config, deadline=self.deadline
            )
        self.assertEqual(observed, launch)
        self.assertEqual(actual.nonce, path.name)
        read.assert_called_once_with(path.parent, deadline=self.deadline)
        self.assertEqual(ledger, before)
        for field, value in (
            ("boot", "different"),
            ("released", True),
            ("service", "unowned"),
        ):
            bad = copy.deepcopy(ledger)
            bad["owners"][path.name][field] = value
            with (
                self.subTest(field=field),
                patch.object(server, "storage_launch", return_value=launch),
                patch.object(host, "readonly_snapshot", return_value=bad),
                self.assertRaises(server.SupervisorError),
            ):
                server.recorded_storage_owner(
                    self.state, self.config, deadline=self.deadline
                )

    def test_listener_requires_pid_start_generation_allocation_cgroup_and_socket(
        self,
    ) -> None:
        profile = host.select("store-functional", str(server.GIB))
        logical_cpus = host.cpu_set(profile.cores)
        owner = host.Allocation(
            self.root / "admission", "a" * 32, profile, self.deadline
        )
        group = "/pse.slice/server.service"
        binding = {"group": group, "invocation": "b" * 32, "inode": 91}
        launch = {"allocation": str(owner.directory / owner.nonce), "binding": binding}
        process: dict[str, object] = {
            "pid": 98765,
            "start": "actual-start",
            "instance_id": self.config["instance_id"],
            "allocation": launch["allocation"],
        }
        observed: dict[str, str] = {
            "ActiveState": "active",
            "InvocationID": "b" * 32,
            "ControlGroup": group,
        }
        config: dict[str, object] = {
            **self.config,
            "resources": {"server_memory_bytes": server.GIB},
        }
        texts = {
            "/proc/98765/cgroup": "0::" + group,
            "/sys/fs/cgroup" + group + "/memory.max": str(server.GIB),
            "/sys/fs/cgroup"
            + group
            + "/cpu.max": f"{len(logical_cpus) * 100000} 100000",
            "/proc/net/tcp": "header\n0: 0100007F:4740 remote 0A z z z z z 777",
        }
        descriptor = MagicMock()
        descriptor.readlink.return_value = Path("socket:[777]")
        original_read = Path.read_text

        def read(path: Path) -> str:
            return texts[str(path)] if str(path) in texts else original_read(path)

        with (
            patch.object(server, "service_directory", return_value=self.state),
            patch.object(server, "read_json", return_value=process),
            patch.object(
                server.native_operation, "start_identity", return_value="actual-start"
            ),
            patch.object(
                server, "recorded_storage_owner", return_value=(owner, launch)
            ),
            patch.object(server, "storage_unit_observation", return_value=observed),
            patch.object(host, "group_identity", return_value=91),
            patch.object(
                server,
                "effective_limits",
                return_value=(server.GIB, len(logical_cpus)),
            ),
            patch.object(
                server, "observed_role_affinity", return_value=True
            ) as affinity,
            patch.object(
                server, "group_for_slice", return_value=Path("/sys/fs/cgroup/pse.slice")
            ),
            patch.object(Path, "read_text", read),
            patch.object(Path, "iterdir", return_value=iter([descriptor])),
        ):
            self.assertTrue(
                server.owns_listener(self.state, config, deadline=self.deadline)
            )
            affinity.assert_called_once_with(
                process["pid"], {"cpu_threads": len(logical_cpus)}, list(logical_cpus)
            )
            affinity.return_value = False
            self.assertFalse(
                server.owns_listener(self.state, config, deadline=self.deadline)
            )
            affinity.return_value = True
            for field in ("start", "instance_id", "allocation"):
                previous = process[field]
                process[field] = "different"
                self.assertFalse(
                    server.owns_listener(self.state, config, deadline=self.deadline)
                )
                process[field] = previous
            for field in ("ActiveState", "InvocationID", "ControlGroup"):
                previous = observed[field]
                observed[field] = "different"
                self.assertFalse(
                    server.owns_listener(self.state, config, deadline=self.deadline)
                )
                observed[field] = previous
            descriptor.readlink.return_value = Path("socket:[unowned]")
            with patch.object(Path, "iterdir", return_value=iter([descriptor])):
                self.assertFalse(
                    server.owns_listener(self.state, config, deadline=self.deadline)
                )


if __name__ == "__main__":
    unittest.main()
