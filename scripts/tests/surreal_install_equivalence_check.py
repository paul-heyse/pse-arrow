# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Reuse a real installed external server across caller-context changes.

Never builds, executes a server, edits checkout configuration or starts a service.
Existing complete receipts retain their original bytes and build provenance.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from unittest.mock import patch

from scripts import surreal_server as server
from scripts import surreal_source as source


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("tool_root", type=Path)
    args = parser.parse_args()
    tool_root = args.tool_root.resolve()
    receipts = {path: path.read_bytes() for path in tool_root.glob("*/release.json")}
    with (
        patch.object(source, "build_context", side_effect=AssertionError("reuse attempted compiler capture")),
        patch.object(source, "command", side_effect=AssertionError("reuse attempted a compiler, source probe or executable")),
    ):
        before = server.install(None, tool_root)
        binary = Path(str(before["binary"]))
        binary_digest = source.digest(binary)
        observed = []
        variants = {
            "worktree": {},
            "PATH": {"PATH": "/different/caller/bin"},
            "LD_LIBRARY_PATH": {"LD_LIBRARY_PATH": "/different/runtime/libraries"},
            "RUSTC_WRAPPER": {"RUSTC_WRAPPER": "/different/worktree/scripts/sccache"},
            "SCCACHE_CONF": {"SCCACHE_CONF": "/different/worktree/.config/sccache.toml"},
            "CARGO_HOME": {"CARGO_HOME": "/different/cargo/home"},
        }
        for kind, environment in variants.items():
            with (
                patch.dict(source.os.environ, environment),
                patch.object(source, "ROOT", tool_root / "different-caller-worktree"),
                patch.object(source.installation_context, "configuration", side_effect=AssertionError("reuse attempted caller configuration capture")),
            ):
                reused = server.install(None, tool_root)
                require(reused == before, "caller change selected a different artifact")
                require(source.admitted_receipt(reused), "launch admission refused the installed package")
                # Even an explicit installer must reuse a completed package.
                require(server.install(None, tool_root, build=True) == before, "explicit installer did not reuse the completed package")
            observed.append(kind)
    require(source.digest(binary) == binary_digest, "reuse changed the executable")
    require(all(path.read_bytes() == original for path, original in receipts.items()), "reuse rewrote original build provenance")
    result = {
        "tool_root": str(tool_root),
        "binary": str(binary),
        "binary_sha256": binary_digest,
        "original_build_key": before["effective_key"],
        "package_selection": source.installation_name(),
        "caller_context_reused": observed,
        "original_receipts_unchanged": True,
        "source_or_executable_commands": 0,
        "compiler_captures": 0,
        "build_invocations": 0,
        "services_started": 0,
    }
    (tool_root / "equivalence-check.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
