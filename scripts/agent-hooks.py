#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
# ruff: noqa: N999 -- executable hook filename, not an importable module
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
# Claude sends one edited file under one of these keys; Codex sends a patch.
FILE_KEYS = ("file_path", "notebook_path")


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


def protected(root: Path, path: str, *, design_edit: bool = False) -> str | None:
    target = (root / path).resolve()
    try:
        relative = target.relative_to(root.resolve())
    except ValueError:
        # Shared library skills and other personal projects are editable.
        return None
    parts = relative.parts
    if not parts or ".git" in parts or parts[0] in {"target", "build", "external"}:
        return "VCS state, build output and reading copies are protected"
    name = relative.as_posix()
    if (
        name.startswith(
            (
                "docs/generated/",
                "python/pse/contracts/",
                "crates/pse-operations-queries/",
            )
        )
        or (parts[0] == "crates" and "generated" in parts)
        or name in {"crates/pse-ipopt-sys/src/bindings.rs", "python/pse/_native.pyi"}
    ):
        return "generator output is protected; fix the generator"
    if not design_edit:
        if name.startswith("docs/authoritative_design/"):
            return (
                "blueprint edits require the authorized design workflow "
                f"(PSE_DESIGN_EDIT=1 or an untracked {DESIGN_EDIT_MARKER} file)"
            )
        if re.fullmatch(r"docs/adr/\d{4}-.+\.md", name) and target.is_file():
            text = target.read_text()
            front = text.split("---", 2)[1] if text.startswith("---\n") else ""
            match = re.search(r"^status:\s*(\S+)", front, re.MULTILINE)
            if not match or match[1] != "proposed":
                return "accepted ADRs are immutable; use the ADR supersession workflow"
    return None


DESIGN_EDIT_MARKER = ".design-edit"


def design_edit_authorized(root: Path) -> bool:
    """The launch-time variable, or a visible untracked checkout file an agent can create.

    The file is not ignored, so `git status` shows it for as long as it authorizes
    edits; delete it to end the exception.
    """
    if os.environ.get("PSE_DESIGN_EDIT") == "1":
        return True
    if not (root / DESIGN_EDIT_MARKER).is_file():
        return False
    # A committed file would authorize everyone; only an untracked one counts.
    tracked = subprocess.run(
        ["git", "-C", str(root), "ls-files", "--error-unmatch", DESIGN_EDIT_MARKER],
        capture_output=True,
        check=False,
    )
    return tracked.returncode != 0


def session_env(root: Path) -> int:
    """Claude SessionStart: give later Bash commands the checkout environment.

    Appends `scripts/pse_env.py --print` output (ordinary exports, local values as deferred
    reads) to $CLAUDE_ENV_FILE. A failure never blocks the session.
    """
    target = os.environ.get("CLAUDE_ENV_FILE")
    if not target:
        return 0
    try:
        if str(root) not in sys.path:
            sys.path.insert(0, str(root))
        from scripts import pse_env  # noqa: PLC0415 -- the shared environment boundary

        text = pse_env.render(root, dict(os.environ), complete=True)
        with Path(target).open("a") as output:
            output.write(text)
    except Exception as exc:  # noqa: BLE001 -- the session proceeds without it
        print(f"pse-env: session environment unavailable: {exc}", file=sys.stderr)
    return 0


def session_root(default: Path) -> Path:
    """The checkout the session works in: a Claude worktree's hook runs this script from
    the main checkout (CLAUDE_PROJECT_DIR) but its payload carries the worktree's cwd."""
    try:
        payload = json.load(sys.stdin)
    except ValueError:
        return default
    cwd = payload.get("cwd") if isinstance(payload, dict) else None
    if not isinstance(cwd, str) or not cwd:
        return default
    top = subprocess.run(
        ["git", "-C", cwd, "rev-parse", "--show-toplevel"],
        capture_output=True,
        text=True,
        check=False,
    )
    candidate = Path(top.stdout.strip()) if top.returncode == 0 else None
    if candidate is not None and (candidate / "scripts/pse_env.py").is_file():
        return candidate
    return default


def main() -> int:
    action = sys.argv[1]
    if action == "session-env":
        return session_env(session_root(ROOT))
    try:
        payload = json.load(sys.stdin)
        paths = edit_paths(payload)
        if action == "guard":
            for path in paths:
                reason = protected(ROOT, path, design_edit=design_edit_authorized(ROOT))
                if reason:
                    print(f"BLOCKED: {path}: {reason}", file=sys.stderr)
                    return 2
        else:
            print(f"unknown action: {action}", file=sys.stderr)
            return 2
    except (ValueError, OSError, TypeError, KeyError) as exc:
        print(f"BLOCKED: {exc}", file=sys.stderr)
        return 2 if action == "guard" else 0
    return 0


if __name__ == "__main__":
    sys.exit(main())
