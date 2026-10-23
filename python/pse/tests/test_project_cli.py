# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""The dedicated executable shares exact project selections with Python."""

import subprocess
from pathlib import Path

import msgspec
import pytest

import pse
from pse import codec
from pse.contracts.documents import ProjectSelectRequest
from pse.tests.canonical_fixture import CanonicalFixture
from pse.tests.test_modeling_kernel import on_primitives, physical


@pytest.mark.component
def test_project_executable_reopens_exact_python_selection(
    inspection_settings: pse.EngineSettings,
    canonical_substrate: CanonicalFixture,
    tmp_path: Path,
) -> None:
    root = Path(__file__).resolve().parents[3]
    runtime = canonical_substrate.runtime(inspection_settings)
    context = physical(runtime)
    manifest = on_primitives(
        (root / "tests/fixtures/packages/minimal_named/package.toml").read_text()
    )
    source = runtime.modeling_from_documents(
        [
            {
                "package.toml": manifest,
                "models/root.pse": "package minimal_named {param k:Scalar=1;}",
            }
        ],
        context,
    )
    runtime.project_create("cli-create", "cli-project", "Executable boundary")
    revision = runtime.project_fork(
        "cli-project", "cli-branch", "cli-source", "cli-fork", source
    )
    executable = root / "target/debug/pse-project"

    def invoke(arguments: list[str]) -> object:
        completed = subprocess.run(
            [
                str(executable),
                "project",
                "--state",
                canonical_substrate.state,
                "--database",
                canonical_substrate.database,
                *arguments,
            ],
            check=False,
            capture_output=True,
            text=True,
            timeout=90,
        )
        assert completed.returncode == 0, completed.stderr
        envelope = msgspec.json.decode(completed.stdout, type=dict[str, object])
        assert envelope["version"] == 1
        return envelope["result"]

    branch = invoke(["branch", "--branch", "cli-branch"])
    assert isinstance(branch, dict)
    assert branch["problem"] == "cli-source"
    physical_root = root / "tests/fixtures/packages/physical-primitives"
    physical_documents = {
        path: list((physical_root / path).read_bytes())
        for path in ("package.toml", "materials/physical.yaml", "materials/time.yaml")
    }
    request = tmp_path / "request.json"
    spill = tmp_path / "spill"
    spill.mkdir()

    def action(value: dict[str, object]) -> object:
        request.write_bytes(
            msgspec.json.encode(
                {
                    "version": 1,
                    "physical": {"version": 1, "packages": [physical_documents]},
                    "action": value,
                }
            )
        )
        return invoke(
            [
                "runtime",
                "--request",
                str(request),
                "--spill-dir",
                str(spill),
                "--threads",
                "2",
            ]
        )

    selected = action(
        {
            "kind": "select",
            "request": {
                "version": 1,
                "reference": {"kind": "branch", "key": "cli-branch"},
            },
        }
    )
    assert isinstance(selected, dict)
    assert selected["revision"] == revision
    python_selection = runtime.project_select(
        codec.decode_json(
            '{"version":1,"reference":{"kind":"branch","key":"cli-branch"}}',
            ProjectSelectRequest,
        ),
        context,
    )
    assert (
        selected["source_identity"]
        == f"blake3:{python_selection.source_identity.hex()}"
    )
    assert (
        selected["physical_context"]
        == f"blake3:{python_selection.physical_context.hex()}"
    )
    reopened = action({"kind": "open_selection", "selection": selected})
    assert isinstance(reopened, dict)
    assert reopened["canonical_revision"] == revision
    assert reopened["canonical_problem"] == "cli-source"
