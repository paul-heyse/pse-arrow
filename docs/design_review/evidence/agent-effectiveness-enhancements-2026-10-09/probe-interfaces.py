# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Read-only command/MCP observations and isolated admission fault control.

Run from the checkout: scripts/pse-env --resource-class light -- .venv/bin/python
docs/design_review/evidence/agent-effectiveness-enhancements-2026-10-09/probe-interfaces.py
No codegen, compilation, tests, service control, configuration or data writes.
"""

from __future__ import annotations

import base64
import contextlib
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import urllib.error
import urllib.request
from unittest.mock import patch
from types import SimpleNamespace

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT))


def command(argv: list[str], *, env: dict[str, str] | None = None) -> dict:
    started = time.monotonic()
    result = subprocess.run(
        argv, cwd=ROOT, env=env, capture_output=True, text=True, timeout=30, check=False
    )
    return {
        "argv": argv,
        "exit": result.returncode,
        "elapsed_seconds": round(time.monotonic() - started, 4),
        "stdout": result.stdout,
        "stderr": result.stderr,
    }


def admission_control() -> dict:
    from scripts import host_admission, pse_env

    environment = dict(os.environ)
    for key in (
        host_admission.MARKER,
        "PSE_RESOURCE_CLASS",
        "PSE_MEMORY_MAX",
        "PSE_REQUIRE_STORE",
    ):
        environment.pop(key, None)
    out, err = io.StringIO(), io.StringIO()
    # No pressure, scopes or real ledger changes: force only the admission result.
    with (
        patch.dict(os.environ, environment, clear=True),
        patch.object(host_admission, "inherit", return_value=None),
        patch.object(pse_env.operation, "scope_owner", return_value=None),
        patch.object(pse_env, "manager_available", return_value=True),
        patch.object(pse_env, "manager_environment", return_value={}),
        patch.object(
            host_admission,
            "acquire",
            side_effect=host_admission.AdmissionError("synthetic capacity unavailable"),
        ) as acquire,
        patch.object(pse_env, "execute") as execute,
        contextlib.redirect_stdout(out),
        contextlib.redirect_stderr(err),
    ):
        code = pse_env.main(
            ["--", str(ROOT / ".venv/bin/python"), "scripts/activity.py", "--json"]
        )
    return {
        "exit": code,
        "admission_called": acquire.call_count,
        "observer_executed": execute.call_count,
        "stdout": out.getvalue(),
        "stderr": err.getvalue(),
        "scope": "synthetic unavailable capacity; no live capacity changed",
    }


def unavailable_activity_control() -> dict:
    from scripts import activity

    out = io.StringIO()
    failed = SimpleNamespace(
        returncode=1, stdout="", stderr="synthetic user manager unavailable"
    )
    with (
        patch.object(activity.subprocess, "run", return_value=failed),
        patch.object(activity, "build_processes", return_value=[]),
        contextlib.redirect_stdout(out),
    ):
        code = activity.main(["--json"])
    return {
        "exit": code,
        "observation": json.loads(out.getvalue()),
        "scope": "synthetic failed systemctl; no service state changed",
    }


def mcp_observation() -> dict:
    state = Path.home() / ".local/state/pse-arrow/surreal-functional-v2"
    config = json.loads((state / "config.json").read_text())
    credentials = json.loads((state / "credentials.json").read_text())
    port = int(config["port"])
    url = f"http://127.0.0.1:{port}/mcp"
    ns, db = str(config["namespace"]), str(config["database"])
    username, password = (
        credentials["selection_username"],
        credentials["selection_password"],
    )
    basic = base64.b64encode(f"{username}:{password}".encode()).decode()
    headers = {
        "Content-Type": "application/json",
        "Accept": "application/json, text/event-stream",
        "Authorization": "Basic " + basic,
        "Surreal-NS": ns,
        "Surreal-DB": db,
    }
    observed: dict = {
        "endpoint": url,
        "principal": "existing root VIEWER selection account",
        "namespace": ns,
        "database": db,
        "requests": [],
    }

    def request(method: str, params: dict, ident: int | None) -> dict | None:
        payload: dict = {"jsonrpc": "2.0", "method": method, "params": params}
        if ident is not None:
            payload["id"] = ident
        req = urllib.request.Request(
            url, data=json.dumps(payload).encode(), headers=headers
        )
        try:
            with urllib.request.urlopen(req, timeout=10) as response:
                raw = response.read(1 << 20).decode()
                if session := response.headers.get("MCP-Session-Id"):
                    headers["MCP-Session-Id"] = session
                result = None
                if raw.strip():
                    messages = [
                        line[5:].strip()
                        for line in raw.splitlines()
                        if line.startswith("data:")
                    ]
                    result = json.loads(messages[-1] if messages else raw)
                observed["requests"].append(
                    {
                        "method": method,
                        "http_status": response.status,
                        "rpc_error": result.get("error") if result else None,
                    }
                )
                return result
        except urllib.error.HTTPError as error:
            observed["requests"].append(
                {
                    "method": method,
                    "http_status": error.code,
                    "body": error.read(4096).decode(errors="replace"),
                }
            )
            return None

    initialized = request(
        "initialize",
        {
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": {"name": "pse-bounded-design-review", "version": "1"},
        },
        1,
    )
    if not initialized or "result" not in initialized:
        return observed
    version = initialized["result"]["protocolVersion"]
    headers["MCP-Protocol-Version"] = version
    observed["protocol_version"] = version
    observed["server_info"] = initialized["result"].get("serverInfo")
    request("notifications/initialized", {}, None)
    listed = request("tools/list", {}, 2)
    if listed and "result" in listed:
        observed["tools"] = sorted(
            tool["name"] for tool in listed["result"].get("tools", [])
        )
    for ident, name, args in (
        (3, "query", {"namespace": ns, "database": db, "query": "RETURN 1;"}),
        (4, "list", {"namespace": ns, "database": db, "kind": "tables"}),
    ):
        reply = request("tools/call", {"name": name, "arguments": args}, ident)
        if reply:
            # Schema names and a constant scalar only; no scientific record payloads.
            observed.setdefault("read_calls", {})[name] = reply
    return observed


def main() -> None:
    python = str(ROOT / ".venv/bin/python")
    facts: dict = {
        "baseline": "d0f2c41818a34539a910654dfea4760771603f45",
        "utc_observed": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "mode": "read-only interfaces plus synthetic admission control",
        "commands": {},
    }
    for name, argv in {
        "assessment_usage": ["just", "--usage", "assessment"],
        "assessment_flag_shape": ["just", "assessment", "--group", "turn-end"],
        "assessment_list": ["just", "assessment-list", "--group", "turn-end"],
        "codegen_check_dry_run": ["just", "--dry-run", "codegen", "--check"],
        "select_direct_help": [python, "scripts/select.py", "--help"],
        "select_module_help": [python, "-m", "scripts.select", "--help"],
        "surreal_quoted_state_dry_run": [
            "just",
            "--dry-run",
            "surreal",
            "status",
            "--state",
            "/tmp/pse review absent-state",
        ],
        "surreal_quoted_state": [
            "just",
            "surreal",
            "status",
            "--state",
            "/tmp/pse review absent-state",
        ],
    }.items():
        facts["commands"][name] = command(argv)
    from scripts import host_admission

    facts["classification"] = {
        "direct_cargo_check": host_admission.classify(
            ["cargo", "check", "-p", "pse-ids"]
        ),
        "just_shell_cargo_check": host_admission.classify(
            ["bash", "-euo", "pipefail", "-c", "cargo check -p pse-ids"]
        ),
        "direct_activity": host_admission.classify(
            [python, "scripts/activity.py", "--json"]
        ),
    }
    facts["admission_fault_control"] = admission_control()
    facts["activity_transport_fault_control"] = unavailable_activity_control()
    facts["control_development_note"] = (
        "The first admission control cleared all PSE variables and stopped at the compiler-cache prerequisite before admission. The corrected control preserves unrelated environment prerequisites and removes only allocation/selection variables. That earlier attempt established no admission claim."
    )
    facts["http_mcp"] = mcp_observation()
    mcp = command(["codex", "mcp", "list", "--json"])
    try:
        servers = json.loads(mcp["stdout"])
        facts["effective_cli_mcp"] = [
            {
                "name": server["name"],
                "enabled": server.get("enabled"),
                "transport": server.get("transport", {}).get("type"),
            }
            for server in servers
        ]
    except (TypeError, ValueError, KeyError):
        facts["effective_cli_mcp"] = {"exit": mcp["exit"], "parse": "unavailable"}
    expected = {
        "assessment_usage": 0,
        "assessment_flag_shape": 2,
        "assessment_list": 0,
        "codegen_check_dry_run": 0,
        "select_direct_help": 1,
        "select_module_help": 2,
        "surreal_quoted_state_dry_run": 0,
        "surreal_quoted_state": 2,
    }
    facts["negative_control_assertions"] = {
        "expected_command_exits": all(
            facts["commands"][name]["exit"] == code for name, code in expected.items()
        ),
        "select_import_error": "ModuleNotFoundError"
        in facts["commands"]["select_direct_help"]["stderr"],
        "observer_not_admitted": facts["admission_fault_control"]["admission_called"]
        == 1
        and facts["admission_fault_control"]["observer_executed"] == 0
        and facts["admission_fault_control"]["exit"] == 125,
        "unavailable_is_reported_empty": facts["activity_transport_fault_control"][
            "exit"
        ]
        == 0
        and facts["activity_transport_fault_control"]["observation"]["units"] == [],
    }
    output = Path(__file__).with_name("interface-observations.json")
    output.write_text(json.dumps(facts, indent=2) + "\n")
    print(
        json.dumps(
            {
                "evidence": str(output),
                "command_exits": {
                    name: item["exit"] for name, item in facts["commands"].items()
                },
                "mcp_requests": facts["http_mcp"]["requests"],
                "admission_fault": facts["admission_fault_control"],
                "negative_control_assertions": facts["negative_control_assertions"],
            },
            indent=2,
        )
    )
    assert all(facts["negative_control_assertions"].values()), (
        "A negative reproduction did not match its expected observation"
    )


if __name__ == "__main__":
    main()
