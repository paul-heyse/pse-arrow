# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit complete-generation fields for controlled server lifecycle fixtures."""
from __future__ import annotations

from pathlib import Path
from typing import TypedDict
import unittest
from unittest.mock import patch

from scripts import surreal_source, installation_context
import os
import tomllib


class ServerReceipt(TypedDict):
    schema: str
    effective_key: str
    inputs: dict[str, object]
    raw_provenance: dict[str, object]
    lockfile_sha256: str
    surrealmx_version: str


def fixture_receipt(tool_root: Path | None = None) -> ServerReceipt:
    selected = surreal_source.inputs(tool_root)
    return {"schema": surreal_source.RECEIPT_SCHEMA, "effective_key": surreal_source.effective_key(selected), "inputs": selected, "raw_provenance": {"fixture": True}, "lockfile_sha256": "1" * 64, "surrealmx_version": surreal_source.ENGINE}


def fixture_build_context(case: unittest.TestCase) -> None:
    """Lifecycle controls supply an explicit compiler fixture, never host claims."""
    selected = patch.object(surreal_source, "build_context", return_value=(fixture_context(), {"fixture": True}, {}))
    selected.start()
    case.addCleanup(selected.stop)


def fixture_context() -> dict:
    configs, _, _ = installation_context.configuration(surreal_source.ROOT, {"HOME": os.environ.get("HOME", str(Path.home())), **({"CARGO_HOME": os.environ["CARGO_HOME"]} if "CARGO_HOME" in os.environ else {})})
    return {"schema": installation_context.SCHEMA, "configuration": configs, "toolchain_declaration": installation_context.canonical(tomllib.loads((surreal_source.ROOT / "rust-toolchain.toml").read_text())), "tools": {"fixture": True}, "toolchain": {"fixture": True}, "native": [{"fixture": True}], "owners": {"fixture": True}}
