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
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PATCH_HEADER = re.compile(
    r"^\*\*\* (?:Add File|Update File|Delete File|Move to): (.+)$", re.MULTILINE
)


def edit_paths(payload: dict) -> list[str]:
    if not isinstance(payload, dict):
        raise TypeError("edit payload must be an object")
    inputs = payload.get("tool_input", {})
    if not isinstance(inputs, dict):
        raise TypeError("tool_input must be an object")
    if "file_path" in inputs:
        if not isinstance(inputs["file_path"], str) or not inputs["file_path"]:
            raise ValueError("file_path must be a nonempty string")
        return [inputs["file_path"]]
    command = inputs.get("command", "")
    if isinstance(command, str) and command.startswith("*** Begin Patch\n"):
        paths = PATCH_HEADER.findall(command)
        if paths and "*** End Patch" in command:
            return paths
    raise ValueError("unrecognized edit payload; refusing an unchecked file edit")


def protected(root: Path, path: str, *, design_edit: bool = False) -> str | None:
    target = (root / path).resolve()
    try:
        relative = target.relative_to(root.resolve())
    except ValueError:
        return "edit is outside the repository"
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
        if protected(root, path, design_edit=os.environ.get("PSE_DESIGN_EDIT") == "1"):
            continue
        target = (root / path).resolve()
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
            command = [str(bindir / "ruff"), "format", str(target)]
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
        print(
            json.dumps(
                {
                    "hookSpecificOutput": {
                        "hookEventName": "SessionStart",
                        "additionalContext": result.stdout + result.stderr,
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
