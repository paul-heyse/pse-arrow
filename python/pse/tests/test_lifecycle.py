# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Public lifecycle explanations preserve current original-owner release predicates."""

from pathlib import Path

import msgspec
import pytest

import pse
from pse.contracts.documents import (
    LifecycleKind,
    LifecycleTarget,
    ProjectNavigationProjectVisibility,
    ProjectNavigationRequest,
)
from pse.tests.canonical_fixture import CanonicalFixture
from pse.tests.test_modeling_kernel import on_primitives, physical


def project_source(runtime: pse.Runtime) -> str:
    root = Path(__file__).resolve().parents[3]
    context = physical(runtime)
    source = runtime.modeling_from_documents(
        [
            {
                "package.toml": on_primitives(
                    (
                        root / "tests/fixtures/packages/minimal_named/package.toml"
                    ).read_text()
                ),
                "models/root.pse": "package minimal_named {param value:Scalar=1;}",
            }
        ],
        context,
    )
    runtime.project_create(
        "lifecycle-create", "lifecycle-project", "Current owner release"
    )
    return runtime.project_fork(
        "lifecycle-project",
        "lifecycle-branch",
        "lifecycle-source",
        "lifecycle-fork",
        source,
    )


@pytest.mark.component
def test_referenced_source_explanation_is_not_a_foreign_or_stale_release_grant(
    inspection_settings: pse.EngineSettings,
    canonical_substrate: CanonicalFixture,
) -> None:
    runtime = canonical_substrate.runtime(inspection_settings)
    revision = project_source(runtime)
    selected = runtime.lifecycle_select(
        LifecycleTarget(kind=LifecycleKind.SOURCE_HISTORY, key=revision)
    )
    explanation = runtime.lifecycle_explain(selected)
    assert explanation.selection.issuing_store == selected.issuing_store
    assert explanation.selection.target == selected.target
    assert explanation.selection.revision == selected.revision
    assert explanation.selection.generation == selected.generation
    assert explanation.selection.project_generation == selected.project_generation
    assert explanation.selection.draft_generation == selected.draft_generation
    assert any(
        blocker.kind == "retained_source_owner" for blocker in explanation.blockers
    )
    assert not explanation.blockers_complete
    assert explanation.bytes.physical is None
    assert explanation.bytes.reclaimed is None
    with pytest.raises(pse.InspectionError):
        runtime.lifecycle_release(selected)
    foreign = msgspec.structs.replace(selected, issuing_store="foreign-installation")
    with pytest.raises(pse.InspectionError):
        runtime.lifecycle_release(foreign)
    stale = msgspec.structs.replace(selected, generation=selected.generation + 1)
    with pytest.raises(pse.InspectionError):
        runtime.lifecycle_explain(stale)
    with pytest.raises(pse.InspectionError):
        runtime.lifecycle_release(stale)
    assert runtime.project_branch_record("lifecycle-branch") is not None
    assert (
        runtime.modeling_revision(revision, physical(runtime)).canonical_revision
        == revision
    )


@pytest.mark.component
def test_historical_draft_release_rechecks_head_generation_and_keeps_summary(
    inspection_settings: pse.EngineSettings,
    canonical_substrate: CanonicalFixture,
) -> None:
    runtime = canonical_substrate.runtime(inspection_settings)
    revision = project_source(runtime)
    first_bytes = b"original malformed [[ draft"
    first = runtime.project_draft_save_intent(
        "first-raw",
        "lifecycle-project",
        "lifecycle-branch",
        "raw",
        revision,
        first_bytes,
    )
    runtime.project_draft_save(first, first_bytes)
    target = LifecycleTarget(kind=LifecycleKind.DRAFT_VERSION, key="first-raw")
    current = runtime.lifecycle_select(target)
    assert any(
        b.kind == "current_draft_head"
        for b in runtime.lifecycle_explain(current).blockers
    )
    with pytest.raises(pse.InspectionError):
        runtime.lifecycle_release(current)
    newer_bytes = b"newer malformed [[ draft"
    newer = runtime.project_draft_save_intent(
        "second-raw",
        "lifecycle-project",
        "lifecycle-branch",
        "raw",
        revision,
        newer_bytes,
        expected=1,
    )
    runtime.project_draft_save(newer, newer_bytes)
    with pytest.raises(pse.InspectionError):
        runtime.lifecycle_release(current)
    historical = runtime.lifecycle_select(target)
    project = runtime.project_record("lifecycle-project")
    assert project is not None
    runtime.project_navigate(
        "hide-project",
        project,
        ProjectNavigationRequest(
            action=ProjectNavigationProjectVisibility(deleted=True)
        ),
    )
    # Visibility alone releases nothing; the saved explanation is now stale.
    assert runtime.project_draft_version_bytes("first-raw") == first_bytes
    with pytest.raises(pse.InspectionError):
        runtime.lifecycle_release(historical)
    selected = runtime.lifecycle_select(target)
    receipt = runtime.lifecycle_release(selected)
    assert receipt.selection.issuing_store == selected.issuing_store
    assert receipt.selection.target == selected.target
    assert receipt.selection.revision == selected.revision
    assert receipt.selection.generation == selected.generation
    assert receipt.selection.project_generation == selected.project_generation
    assert receipt.selection.draft_generation == selected.draft_generation
    assert receipt.complete
    assert receipt.physical_reclaimed_bytes is None
    summary = runtime.project_draft_version("first-raw")
    assert summary is not None
    assert summary.key == "first-raw"
    assert summary.base_revision == revision
    assert summary.retired
    assert not summary.content
    released = runtime.lifecycle_explain(selected)
    assert released.detail_released
    assert released.bytes.physical is None
    assert released.bytes.reclaimed is None
    with pytest.raises(pse.InspectionError):
        runtime.project_draft_version_bytes("first-raw")
    assert runtime.project_draft_bytes("raw") == newer_bytes
