# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Launch Codex with explicit checkout and existing selected-service capabilities."""

from __future__ import annotations

import argparse
import json
import os
import shlex
import shutil
import sys
import time
from pathlib import Path

from scripts import pse_env, surreal_server

ROOT = Path(__file__).resolve().parents[1]


def session_description(
    *, state: Path | None, database: bool, rust: bool,
    require_database: bool, require_rust: bool, arguments: list[str],
) -> tuple[list[str], dict[str, object]]:
    """Observe existing capabilities only; never start, build or reconcile anything."""
    executable = shutil.which("codex")
    if executable is None:
        raise ValueError("Codex is unavailable; prepare the workstation Codex installation")
    command = [executable, "-C", str(ROOT)]
    capabilities: dict[str, object] = {
        "rust": {"requested": rust, "attached": "not observed"},
        "database": {"requested": database, "attached": "not observed"},
    }
    rust_available = False
    if rust:
        analyzer = shutil.which("rust-analyzer-mcp")
        available = analyzer is not None
        rust_available = available
        capabilities["rust"] = {
            "requested": True, "available": available, "attached": "not observed",
            "repair": "prepare the configured rust-analyzer-mcp executable",
        }
        if require_rust and not available:
            raise ValueError("Required rust-analyzer-mcp executable is unavailable")
        if available:
            command += ["-c", "mcp_servers.rust-analyzer.command=" + json.dumps(analyzer)]
    command += ["-c", "mcp_servers.rust-analyzer.enabled=" + str(rust_available).lower()]
    if database:
        selected_state = state or Path(pse_env.base_environment(ROOT, os.environ)["PSE_SURREAL_STATE"])
        deadline = time.monotonic() + 8.0
        try:
            selected = surreal_server.selected_service(selected_state, deadline)
            endpoint = f"http://127.0.0.1:{selected.port}/mcp"
            helper = shlex.join([
                str(ROOT / ".venv/bin/python"), str(ROOT / "scripts/surreal_server.py"),
                "mcp-headers", "--state", str(selected.state), "--expected-endpoint", endpoint,
            ])
            command += ["-c", "mcp_servers.pse-surreal.enabled=true",
                        "-c", "mcp_servers.pse-surreal.url=" + json.dumps(endpoint),
                        "-c", "mcp_servers.pse-surreal.http_headers_helper=" + json.dumps(helper),
                        "-c", 'mcp_servers.pse-surreal.enabled_tools=["info","list","query"]',
                        "-c", "mcp_servers.pse-surreal.required=" + str(require_database).lower()]
            selected.recheck(deadline)
            capabilities["database"] = {"requested": True, "available": True,
                                        "attached": "not observed", "identity": selected.identity}
        except (OSError, ValueError, surreal_server.SupervisorError) as error:
            if require_database:
                raise ValueError(f"Required selected database is unavailable: {error}") from error
            command += ["-c", "mcp_servers.pse-surreal.enabled=false"]
            capabilities["database"] = {"requested": True, "available": False,
                                        "attached": "not observed", "reason": str(error),
                                        "repair": f"inspect the selected state with just surreal status --state {shlex.quote(str(selected_state))}"}
    else:
        command += ["-c", "mcp_servers.pse-surreal.enabled=false"]
    if require_rust:
        command += ["-c", "mcp_servers.rust-analyzer.required=true"]
    # Explicit caller overrides remain last. These strings are argv data, never shell code.
    command += arguments
    return command, {
        "schema": "codex-selected-session-v1", "checkout": str(ROOT),
        "capabilities": capabilities, "caller_overrides": bool(arguments),
        "attachment": "requires a real client interaction; process spawn is not attachment",
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--state", type=Path)
    parser.add_argument("--without-database", action="store_true")
    parser.add_argument("--without-rust", action="store_true")
    parser.add_argument("--require-database", action="store_true")
    parser.add_argument("--require-rust", action="store_true")
    parser.add_argument("--explain", action="store_true")
    parser.add_argument("--json", action="store_true")
    parser.add_argument("arguments", nargs=argparse.REMAINDER, help="literal Codex arguments after --")
    args = parser.parse_args(argv)
    if (args.require_database and args.without_database) or (args.require_rust and args.without_rust):
        parser.error("a required capability cannot also be disabled")
    trailing = args.arguments[1:] if args.arguments[:1] == ["--"] else args.arguments
    try:
        command, description = session_description(
            state=args.state, database=not args.without_database, rust=not args.without_rust,
            require_database=args.require_database, require_rust=args.require_rust, arguments=trailing,
        )
    except (OSError, ValueError) as error:
        print(f"codex-session: {error}", file=sys.stderr)
        return 125
    if args.explain:
        print(json.dumps(description, indent=2) if args.json else description)
        return 0
    print(json.dumps(description), file=sys.stderr)
    os.execv(command[0], command)
    return 127


if __name__ == "__main__":
    raise SystemExit(main())
