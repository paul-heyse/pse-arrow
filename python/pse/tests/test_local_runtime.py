# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Local composition exercises only the native model's actual dependencies."""
from pathlib import Path

import pytest

import pse
from pse.contracts.documents import SourceDelta
from pse.tests.test_native_workflow import package_documents


def physical_documents() -> dict[str, str]:
    root = Path(__file__).resolve().parents[3] / "tests/fixtures/packages/physical-primitives"
    return {str(path.relative_to(root)): path.read_text() for path in root.rglob("*") if path.is_file()}


@pytest.mark.unit
def test_local_runtime_owns_models_and_exact_edits_without_a_substrate(
    inspection_settings: pse.EngineSettings,
) -> None:
    runtime = pse.Runtime(inspection_settings, ephemeral=True)
    assert not runtime.durable
    physical = runtime.physical_from_documents(physical_documents())
    sources = package_documents("package local {param a:Scalar=1;}")
    package = runtime.modeling_from_documents([sources], physical)
    assert package.revision_id
    assert package.canonical_revision is None
    assert package.canonical_problem is None
    assert package.with_delta(SourceDelta(parent=package.revision_id)).revision_id == package.revision_id
    original = package.declarations()
    changed = package.reload_from_parent([package_documents("package local {param a:Scalar=2;}")])
    assert changed.revision_id != package.revision_id
    assert changed.canonical_revision is None
    assert package.declarations() == original
    with pytest.raises(pse.InspectionError):
        changed.with_delta(SourceDelta(parent=package.revision_id))
    with pytest.raises(pse.InspectionError):
        runtime.run_record("unavailable")
    with pytest.raises(pse.InspectionError):
        runtime.source_settlement("original-problem", "original-operation")
    runtime.close()
    with pytest.raises(pse.InspectionError):
        package.declarations()


@pytest.mark.unit
def test_configured_ephemeral_runtime_does_not_open_its_substrate(
    inspection_settings: pse.EngineSettings, tmp_path: Path,
) -> None:
    missing = tmp_path / "never-opened-state"
    runtime = pse.Runtime(inspection_settings, ephemeral=True, substrate=str(missing))
    physical = runtime.physical_from_documents(physical_documents())
    package = runtime.modeling_from_documents([package_documents("package local {param a:Scalar=1;}")], physical)
    assert package.canonical_revision is None
    assert not missing.exists()
    runtime.close()
    with pytest.raises(pse.InspectionError):
        runtime.run_record("unavailable")
    assert not missing.exists(), "closed caller cannot start lazy connection"
    with pytest.raises(pse.InspectionError):
        runtime.physical_from_documents(physical_documents())
    with pytest.raises(pse.InspectionError):
        runtime.modeling_from_documents([package_documents("package closed {param a:Scalar=1;}")], physical)


@pytest.mark.unit
@pytest.mark.parametrize("ephemeral,database", [(False, None), (True, "orphan")])
def test_local_composition_requires_explicit_ephemeral_and_valid_sink_selection(
    inspection_settings: pse.EngineSettings, ephemeral: bool, database: str | None,
) -> None:
    with pytest.raises(pse.InspectionError):
        pse.Runtime(inspection_settings, ephemeral=ephemeral, database=database)
