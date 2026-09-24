#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Shared edit policy with Claude file inputs and Codex patch inputs."""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PATCH_HEADER = re.compile(
    r"^\*\*\* (?:Add File|Update File|Delete File|Move to): (.+)$", re.MULTILINE
)
# Claude sends one edited file under one of these keys; Codex sends a patch.
FILE_KEYS = ("file_path", "notebook_path")
WRITABLE_ENV = "PSE_AGENT_WRITABLE"


def edit_paths(payload: dict) -> list[str]:
    if not isinstance(payload, dict):
        raise TypeError("edit payload must be an object")
    inputs = payload.get("tool_input", {})
    if not isinstance(inputs, dict):
        raise TypeError("tool_input must be an object")
    for key in FILE_KEYS:
        if key in inputs:
            if not isinstance(inputs[key], str) or not inputs[key]:
                raise ValueError(f"{key} must be a nonempty string")
            return [inputs[key]]
    command = inputs.get("command", "")
    if isinstance(command, str) and command.startswith("*** Begin Patch\n"):
        paths = PATCH_HEADER.findall(command)
        if paths and "*** End Patch" in command:
            return paths
    raise ValueError("unrecognized edit payload; refusing an unchecked file edit")


def agent_areas() -> list[Path]:
    """Directories an agent runtime owns: its config, memory and scratch space.

    These hold runtime state, not project state. The guard protects the working
    copy and is not a shell sandbox, so refusing them would only add friction to
    the sanctioned tool path while leaving shell writes untouched.
    """
    home = Path.home()
    areas = [
        Path(os.environ.get("CLAUDE_CONFIG_DIR") or home / ".claude"),
        home / ".codex",
        Path(tempfile.gettempdir()),
    ]
    areas += [
        Path(entry)
        for entry in os.environ.get(WRITABLE_ENV, "").split(os.pathsep)
        if entry
    ]
    return [area.expanduser().resolve() for area in areas]


def within(target: Path, base: Path) -> bool:
    return target == base or base in target.parents


def protected(root: Path, path: str, *, design_edit: bool = False) -> str | None:
    target = (root / path).resolve()
    try:
        relative = target.relative_to(root.resolve())
    except ValueError:
        if any(within(target, area) for area in agent_areas()):
            return None
        return "edit is outside the working copy and the agent's own directories"
    parts = relative.parts
    if not parts or ".git" in parts or parts[0] in {"target", "build", "external"}:
        return "VCS state, build output and reading copies are protected"
    name = relative.as_posix()
    if (
        name.startswith(("docs/generated/", "python/pse/contracts/"))
        or (parts[0] == "crates" and "generated" in parts)
        or name == "crates/pse-ipopt-sys/src/bindings.rs"
    ):
        return "generator output is protected; fix the generator"
    if not design_edit:
        if name.startswith("docs/authoritative_design/"):
            return "blueprint edits require the authorized design workflow (PSE_DESIGN_EDIT=1)"
        if re.fullmatch(r"docs/adr/\d{4}-.+\.md", name) and target.is_file():
            text = target.read_text()
            front = text.split("---", 2)[1] if text.startswith("---\n") else ""
            match = re.search(r"^status:\s*(\S+)", front, re.MULTILINE)
            if not match or match[1] != "proposed":
                return "accepted ADRs are immutable; use the ADR supersession workflow"
    return None


def format_paths(root: Path, paths: list[str]) -> None:
    venv = Path(os.environ.get("UV_PROJECT_ENVIRONMENT", ".venv"))
    bindir = root / venv / ("Scripts" if os.name == "nt" else "bin")
    for path in dict.fromkeys(paths):
        target = (root / path).resolve()
        # Repo formatters carry repo configuration. An agent's own scratch space
        # and runtime config are not the working copy and are left untouched.
        if not within(target, root.resolve()):
            continue
        # Match canonical bundles, symlink aliases and Windows materialized copies.
        if any(
            within(target, (root / runtime / "skills").resolve())
            for runtime in (".codex", ".claude", ".agents")
        ):
            continue
        if protected(root, path, design_edit=os.environ.get("PSE_DESIGN_EDIT") == "1"):
            continue
        if not target.is_file():
            continue
        command = None
        if target.suffix == ".rs":
            command = [
                "rustfmt",
                "--edition",
                "2024",
                "--config",
                "skip_children=true",
                str(target),
            ]
        elif target.suffix == ".py":
            command = [str(bindir / "ruff"), "format", "--force-exclude", str(target)]
        elif target.suffix == ".toml":
            command = [str(bindir / "taplo"), "fmt", str(target)]
        if command:
            try:
                subprocess.run(
                    command, cwd=root, check=True, capture_output=True, timeout=60
                )
            except (OSError, subprocess.SubprocessError) as exc:
                print(f"formatter did not finish for {path}: {exc}", file=sys.stderr)


def main() -> int:
    action = sys.argv[1]
    if action == "session":
        result = subprocess.run(
            [sys.executable, str(ROOT / "scripts/doctor.py")],
            capture_output=True,
            text=True,
            check=False,
            timeout=45,
        )
        # A clean report is one line: a full PASS list primes agents to re-verify
        # the environment instead of working on the change (AGENTS.md).
        report = result.stdout + result.stderr
        if result.returncode == 0 and not re.search(r"\[(FAIL|WARN)\]", report):
            report = "doctor: environment ready (all checks pass); do not rerun it."
        print(
            json.dumps(
                {
                    "hookSpecificOutput": {
                        "hookEventName": "SessionStart",
                        "additionalContext": report,
                    }
                }
            )
        )
        return 0
    try:
        payload = json.load(sys.stdin)
        paths = edit_paths(payload)
        if action == "guard":
            for path in paths:
                reason = protected(
                    ROOT, path, design_edit=os.environ.get("PSE_DESIGN_EDIT") == "1"
                )
                if reason:
                    print(f"BLOCKED: {path}: {reason}", file=sys.stderr)
                    return 2
        elif action == "format":
            format_paths(ROOT, paths)
        else:
            print(f"unknown action: {action}", file=sys.stderr)
            return 2
    except (ValueError, OSError, TypeError, KeyError) as exc:
        print(f"BLOCKED: {exc}", file=sys.stderr)
        return 2 if action == "guard" else 0
    return 0


if __name__ == "__main__":
    sys.exit(main())
