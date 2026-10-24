# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Launch Codex with explicit checkout and existing selected-service capabilities."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import sys
from pathlib import Path

from scripts import diagnostic_trace, pse_env

ROOT = Path(__file__).resolve().parents[1]


def session_description(
    *,
    state: Path | None,
    database: bool,
    rust: bool,
    require_database: bool,
    require_rust: bool,
    arguments: list[str],
    python: bool = True,
    require_python: bool = False,
) -> tuple[list[str], dict[str, object]]:
    """Select one source-only attachment; specialist readiness stays demand-driven.

    Required options check local preparation, never connect, start a service, or
    acquire workload admission. Live readiness is established by actual selected
    calls, and cannot honestly be claimed by constructing a launch command.
    """
    executable = shutil.which("codex")
    if executable is None:
        raise ValueError("Codex is unavailable; prepare the workstation Codex installation")
    interpreter = ROOT / ".venv/bin/python"
    if not interpreter.is_file():
        raise ValueError("selected checkout .venv/bin/python is unavailable; prepare this checkout's environment")
    bridge = diagnostic_trace.binary_path("mcpls").is_file()
    rust_available = rust and bridge and shutil.which("rust-analyzer") is not None
    python_available = python and bridge and (ROOT / ".venv/bin/pyrefly").is_file()
    if require_rust and not rust_available:
        raise ValueError("Required rust-analyzer navigation is unavailable")
    if require_python and not python_available:
        raise ValueError("Required Pyrefly navigation is unavailable")
    selected_state = state or Path(pse_env.base_environment(ROOT, os.environ)["PSE_SURREAL_STATE"])
    # Local source-only preparation, not a claim that a service is live/attached.
    database_prepared = database and selected_state.is_dir()
    if require_database and not database_prepared:
        raise ValueError("Required selected database state is unavailable; prepare it explicitly")
    languages = ",".join(name for name, enabled in (("python", python_available), ("rust", rust_available)) if enabled)
    settings = {
        "mcp_servers.pse-capabilities.command": str(interpreter),
        "mcp_servers.pse-capabilities.args": ["-m", "scripts.capabilities"],
        "mcp_servers.pse-capabilities.cwd": str(ROOT),
        "mcp_servers.pse-capabilities.enabled": True,
        "mcp_servers.pse-capabilities.required": True,
        "mcp_servers.pse-capabilities.env": {
            "PSE_CAPABILITIES_LANGUAGES": languages,
            "PSE_CAPABILITIES_DATABASE": "1" if database else "0",
            "PSE_CAPABILITIES_DATABASE_STATE": str(selected_state.absolute()),
        },
        # Disable inherited specialist attachments; all useful operations now
        # share the canonical endpoint and its owned lazy transport lifetime.
        "mcp_servers.rust-analyzer.enabled": False,
        "mcp_servers.pse-python.enabled": False,
        "mcp_servers.pse-surreal.enabled": False,
    }
    command = [executable, "-C", str(ROOT)]
    for key, value in settings.items():
        if isinstance(value, dict):
            # Codex -c consumes TOML, whose inline tables use '=' rather than ':'.
            encoded = "{" + ",".join(json.dumps(k) + "=" + json.dumps(v) for k, v in value.items()) + "}"
        else:
            encoded = json.dumps(value)
        command.extend(["-c", key + "=" + encoded])
    command += arguments
    return command, {
        "schema": "codex-selected-session-v2",
        "checkout": str(ROOT),
        "entrypoint": "pse-capabilities",
        "capabilities": {
            "rust": {"requested": rust, "prepared": rust_available,
                     "required": require_rust, "attached": "not observed"},
            "python": {"requested": python, "prepared": python_available,
                       "required": require_python, "attached": "not observed"},
            "database": {"requested": database, "prepared": database_prepared,
                         "required": require_database, "state": str(selected_state.absolute()),
                         "attached": "not observed", "identity": "validated lazily by database_select"},
        },
        "caller_overrides": bool(arguments),
        "attachment": "requires a real client interaction; process spawn is not attachment",
        "readiness": "required flags check local preparation; actual selected calls establish live readiness",
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--state", type=Path)
    parser.add_argument("--without-database", action="store_true")
    parser.add_argument("--without-rust", action="store_true")
    parser.add_argument("--without-python", action="store_true")
    parser.add_argument("--require-python", action="store_true")
    parser.add_argument("--require-database", action="store_true")
    parser.add_argument("--require-rust", action="store_true")
    parser.add_argument("--explain", action="store_true")
    parser.add_argument("--json", action="store_true")
    parser.add_argument(
        "arguments", nargs=argparse.REMAINDER, help="literal Codex arguments after --"
    )
    args = parser.parse_args(argv)
    if (
        (args.require_database and args.without_database)
        or (args.require_rust and args.without_rust)
        or (args.require_python and args.without_python)
    ):
        parser.error("a required capability cannot also be disabled")
    trailing = args.arguments[1:] if args.arguments[:1] == ["--"] else args.arguments
    try:
        command, description = session_description(
            state=args.state,
            database=not args.without_database,
            rust=not args.without_rust,
            require_database=args.require_database,
            require_rust=args.require_rust,
            arguments=trailing,
            python=not args.without_python,
            require_python=args.require_python,
        )
    except (OSError, ValueError) as error:
        print(f"codex-session: {error}", file=sys.stderr)
        return 125
    if args.explain:
        print(json.dumps(description, indent=2) if args.json else description)
        return 0
    print(json.dumps(description), file=sys.stderr)
    return pse_env.execute(command, dict(os.environ))


if __name__ == "__main__":
    raise SystemExit(main())
