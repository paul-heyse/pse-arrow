# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Historical selection traverses the real Python/native source owner boundary."""

from pathlib import Path

import msgspec
import pytest

import pse
from pse.tests.canonical_fixture import CanonicalFixture
from pse.tests.test_modeling_kernel import on_primitives, physical


@pytest.mark.component
def test_project_context_cut_correction_tombstone_and_exact_source_reopen(
    inspection_settings: pse.EngineSettings, canonical_substrate: CanonicalFixture
) -> None:
    runtime = canonical_substrate.runtime(inspection_settings)
    root = Path(__file__).resolve().parents[3]
    manifest = on_primitives(
        (root / "tests/fixtures/packages/minimal_explicit/package.toml")
        .read_text()
        .replace('id_policy = "explicit"', 'id_policy = "named"')
    )
    context = physical(runtime)
    original = runtime.modeling_from_documents(
        [{"package.toml": manifest, "models/value.pse": "package p {param value:Scalar=1;}"}],
        context,
    )
    runtime.project_create("create", "project", "Historical calibration")
    revision = runtime.project_fork("project", "original", "original-source", "fork-original", original)
    original = runtime.modeling_revision(revision, context)
    project = runtime.project_record("project")
    assert project is not None
    first = runtime.project_context_append("first", project, "calibration", original, 0)
    cut = runtime.project_context_cut("project")
    selected = runtime.project_context_select(cut, "calibration", 25)
    assert selected is not None
    assert selected.context.key == first.key
    reopened = runtime.project_context_package(selected, context)
    assert reopened.canonical_revision == revision

    corrected = runtime.modeling_from_documents(
        [{"package.toml": manifest, "models/value.pse": "package p {param value:Scalar=2;}"}],
        context,
    )
    corrected_revision = runtime.project_fork("project", "corrected", "corrected-source", "fork-corrected", corrected)
    corrected = runtime.modeling_revision(corrected_revision, context)
    current_project = runtime.project_record("project")
    assert current_project is not None
    runtime.project_context_append(
        "correction", current_project, "calibration", corrected, 20,
        effective_to_us=40, corrects="first",
    )
    corrected_cut = runtime.project_context_cut("project")
    later = runtime.project_context_select(corrected_cut, "calibration", 25)
    assert later is not None
    assert later.context.key == "correction"
    assert runtime.project_context_package(later, context).canonical_revision == corrected_revision
    # Original operation settles even after unrelated membership generation advancement.
    assert runtime.project_context_append("first", project, "calibration", original, 0).key == first.key
    runtime.project_context_append(
        "deleted", current_project, "calibration", corrected, 24,
        effective_to_us=30, corrects="correction", deleted=True,
    )
    final_cut = runtime.project_context_cut("project")
    assert runtime.project_context_select(final_cut, "calibration", 25) is None
    boundary = runtime.project_context_select(final_cut, "calibration", 30)
    assert boundary is not None
    assert boundary.context.key == "correction"
    assert runtime.project_context_select(cut, "calibration", 25) == selected
    assert runtime.project_context_package(selected, context).canonical_revision == revision
    tombstone = runtime.project_context_record("deleted")
    assert tombstone is not None
    assert tombstone.deleted
    forged_record = msgspec.structs.replace(selected.context, source_identity="forged")
    forged = msgspec.structs.replace(selected, context=forged_record)
    with pytest.raises(pse.InspectionError):
        runtime.project_context_package(forged, context)
    foreign = msgspec.structs.replace(cut, issuing_store="foreign")
    with pytest.raises(pse.InspectionError):
        runtime.project_context_select(foreign, "calibration", 25)
