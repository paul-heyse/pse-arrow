# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Prepared-artifact fixture for independently scoped executable consumer controls."""

from __future__ import annotations

import hashlib
import os
from typing import TYPE_CHECKING

from scripts import evidence_store, python_extension

if TYPE_CHECKING:
    from pathlib import Path


def executable(
    root: Path,
    tool: Path,
    *,
    files: tuple[Path, ...] = (),
    consumed_environment: tuple[str, ...] = (),
) -> None:
    environment = evidence_store.compiler_environment(os.environ)
    selection: dict[str, object] = {
        "version": 1,
        "scope": "ordinary-cargo-inputs",
        "files": [str(path) for path in files],
        "directories": [],
        "environment": list(consumed_environment),
        "generations": {},
    }
    inputs = python_extension.producing_inputs(
        selection,
        environment,
        mode="dev",
        profile="dev",
        validated=True,
        stage="evidence-executable",
    )
    python_extension.publish_receipt(
        python_extension.stage_receipt("evidence-executable", environment, root=root),
        {
            "inputs": inputs,
            "artifact": {
                "binary": str(tool),
                "sha256": hashlib.sha256(tool.read_bytes()).hexdigest(),
                "features": [],
            },
            "producing": {
                "selection": selection,
                "association": {"kind": "actual-cargo-executable-exact-bytes"},
            },
        },
    )
