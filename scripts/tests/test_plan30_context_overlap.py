# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit cross-language registered database overlap; no scientific execution."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import time
from pathlib import Path
from typing import TYPE_CHECKING

import pytest

from pse.tests import conftest as fixture_plugin

if TYPE_CHECKING:
    from pse.tests.canonical_fixture import CanonicalFixture

from scripts import (
    native_operation,
    producer_deployment,
    surreal_server,
    test_resources,
)

canonical_substrate = fixture_plugin.canonical_substrate
inspection_settings = fixture_plugin.inspection_settings


def query(state: Path, database: str, sql: str) -> list[object]:
    config = surreal_server.config_for(state)
    credentials = surreal_server.read_json(Path(str(config["credentials_file"])))
    environment = {
        key: value
        for key, value in os.environ.items()
        if not key.startswith("SURREAL_")
    }
    environment.update(
        SURREAL_USER=str(credentials["username"]),
        SURREAL_PASS=str(credentials["password"]),
    )
    server = config["server"]
    if not isinstance(server, dict):
        raise TypeError("Invalid control server configuration")
    result = subprocess.run(
        [
            str(server["binary"]),
            "sql",
            "--endpoint",
            str(config["endpoint"]),
            "--namespace",
            str(config["namespace"]),
            "--database",
            database,
            "--json",
            "--hide-welcome",
        ],
        input=sql,
        env=environment,
        capture_output=True,
        text=True,
        timeout=30,
        check=True,
    )
    if result.stderr.strip():
        raise AssertionError("native CLI emitted diagnostics")
    parsed = test_resources.cli_json(
        result.stdout, namespace=str(config["namespace"]), database=database
    )
    if not (isinstance(parsed, list)):
        raise TypeError("complete native CLI statement list required")
    return parsed


@pytest.mark.component
def test_rust_python_fixture_context_overlap(
    canonical_substrate: CanonicalFixture,
) -> None:
    directory = Path(os.environ["PSE_PLAN30_C1_CONTROL"])
    native_path = sys.modules["pse._native"].__file__
    if native_path is None:
        raise TypeError("Native extension has no artifact path")
    imported_native = producer_deployment.file_observation(Path(native_path))
    native_operation.write_json(
        directory / "python-native-imported.json", imported_native
    )
    if not (
        imported_native == json.loads((directory / "python-native.json").read_text())
    ):
        raise AssertionError("actual Python child imported a different native artifact")
    state = Path(canonical_substrate.state)
    marker = "python:" + canonical_substrate.database
    installed = query(
        state,
        canonical_substrate.database,
        "DEFINE TABLE c1_overlap_problem SCHEMALESS; CREATE c1_overlap_problem:same_authored_name SET marker = "
        + json.dumps(marker)
        + ";\n",
    )
    if not (
        len(installed) == 2 and installed[0] is None and isinstance(installed[1], list)
    ):
        raise AssertionError("Bounded overlap assertion failed")
    native_operation.write_json(
        directory / "python-ready.json",
        {
            "database": canonical_substrate.database,
            "resource": canonical_substrate.resource,
            "marker": marker,
        },
    )
    deadline = time.monotonic() + 60
    while not (directory / "rust-ready.json").exists():
        if not (time.monotonic() < deadline):
            raise AssertionError(
                "Rust overlap peer did not arrive within the control deadline"
            )
        time.sleep(0.02)
    peer = json.loads((directory / "rust-ready.json").read_text())
    if not (peer["database"] != canonical_substrate.database):
        raise AssertionError("Bounded overlap assertion failed")
    observed = query(
        state,
        canonical_substrate.database,
        "SELECT VALUE marker FROM c1_overlap_problem:same_authored_name;\n",
    )
    if not (observed == [[marker]]):
        raise AssertionError("Bounded overlap assertion failed")
    native_operation.write_json(
        directory / "python-observed.json", {"values": observed}
    )
    while not (directory / "rust-observed.json").exists():
        if not (time.monotonic() < deadline):
            raise AssertionError(
                "Rust overlap proof did not complete within the control deadline"
            )
        time.sleep(0.02)
