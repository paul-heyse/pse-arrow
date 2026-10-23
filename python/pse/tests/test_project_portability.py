# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Selected project exchange uses ordinary admission and independent copied custody."""

import os
import shutil
import subprocess
from pathlib import Path

import attrs
import msgspec
import pytest

import pse
from pse import codec
from pse.contracts.documents import (
    ProjectExportRequest,
    ProjectReferenceBranch,
    ProjectSelectRequest,
    ProjectSelection,
)
from pse.tests.canonical_fixture import CanonicalFixture
from pse.tests.test_modeling_kernel import on_primitives, physical

ROOT = Path(__file__).resolve().parents[3]
SOURCE = "package minimal_named {param value:Scalar=1;}"


def selected_project(
    runtime: pse.Runtime, context: pse.PhysicalContext
) -> ProjectSelection:
    documents = {
        "package.toml": on_primitives(
            (ROOT / "tests/fixtures/packages/minimal_named/package.toml").read_text()
        ),
        "models/root.pse": SOURCE,
    }
    source = runtime.modeling_from_documents([documents], context)
    runtime.project_create("portable-create", "portable-project", "Portable source")
    runtime.project_fork(
        "portable-project", "portable-branch", "portable-problem", "portable-fork", source
    )
    return runtime.project_select(
        ProjectSelectRequest(reference=ProjectReferenceBranch(key="portable-branch")),
        context,
    )


def export_request(
    selection: ProjectSelection, *, descriptive_only: bool = False
) -> ProjectExportRequest:
    return ProjectExportRequest(
        selection=selection,
        research_selections=(),
        publications=(),
        studies=(),
        results=(),
        artifacts=(),
        reproducers=(),
        descriptive_only=descriptive_only,
        omissions=(),
        maximum_output_bytes=32 << 20,
        maximum_document_bytes=16 << 20,
    )


@pytest.mark.component
def test_selected_pack_offline_python_and_actual_cli_preserve_original_science(
    inspection_settings: pse.EngineSettings,
    canonical_substrate: CanonicalFixture,
    tmp_path: Path,
) -> None:
    runtime = canonical_substrate.runtime(inspection_settings)
    context = physical(runtime)
    selection = selected_project(runtime, context)
    request = export_request(selection)
    closure = runtime.project_closure(request, context)
    assert closure.manifest.origin == selection
    assert closure.known_source_bytes > 0
    assert not closure.result_bytes_known
    destination = tmp_path / "selected-pack"
    published = runtime.project_export(destination, request, context)
    assert published.offline_complete
    assert published.executable_complete
    runtime.close()

    # Both readers operate on copied membership after closing the original runtime.
    bundle = pse.ProjectBundle(destination)
    manifest = bundle.manifest
    assert manifest.origin == selection
    assert manifest == published
    selected = next(s for s in manifest.sources if s.revision == selection.revision)
    assert selected.source_identity == f"blake3:{selection.source_identity.hex()}"
    assert selected.physical_context == f"blake3:{selection.physical_context.hex()}"
    document = next(d for d in selected.documents if d.path == "models/root.pse")
    assert bundle.member_bytes(document.member, maximum_bytes=1 << 20) == SOURCE.encode()
    with pytest.raises(pse.InspectionError):
        bundle.member_bytes(document.member, maximum_bytes=1)
    with pytest.raises(pse.InspectionError):
        bundle.member_bytes("unselected-member", maximum_bytes=1 << 20)

    absent_state = tmp_path / "no-service-here"
    environment = {
        key: value
        for key, value in os.environ.items()
        if not key.startswith(("SURREAL_", "DATABASE_"))
        and key not in {"PSE_TOOLING_ACTIVITY_FD", "PSE_TOOLING_SERVICE_INSTANCE_ID"}
    }
    environment["PSE_SURREAL_STATE"] = str(absent_state)
    completed = subprocess.run(
        [str(ROOT / "target/debug/pse-project"), "project", "inspect-pack", "--input", str(destination)],
        cwd=ROOT,
        env=environment,
        capture_output=True,
        text=True,
        check=False,
        timeout=90,
    )
    assert completed.returncode == 0, completed.stderr
    envelope = msgspec.json.decode(completed.stdout, type=dict[str, object])
    assert envelope["version"] == 1
    result = envelope["result"]
    assert isinstance(result, dict)
    assert result["origin"] == msgspec.json.decode(
        codec.encode_json(selection), type=dict[str, object]
    )
    assert result["offline_complete"] is True
    assert result["executable_complete"] is True
    assert not absent_state.exists()


@pytest.mark.component
def test_import_retry_fresh_ownership_and_reopen_after_supplied_pack_is_removed(
    inspection_settings: pse.EngineSettings,
    canonical_substrate: CanonicalFixture,
    tmp_path: Path,
) -> None:
    exporting = canonical_substrate.runtime(inspection_settings)
    context = physical(exporting)
    origin = selected_project(exporting, context)
    destination = tmp_path / "exchange-pack"
    exporting.project_export(destination, export_request(origin), context)
    exporting.close()
    bundle = pse.ProjectBundle(destination)
    receiver = canonical_substrate.runtime(inspection_settings)
    intent = receiver.project_import_issue(bundle, title="Imported project", branch_name="working")
    imported = receiver.project_import(bundle, intent)
    assert imported.source_admitted
    assert imported.project_complete
    assert imported.origin == origin
    assert imported.bundle_digest == bundle.digest
    assert imported.project != origin.project
    assert imported.problem != origin.problem
    assert imported.revision != origin.revision
    assert imported.branch != "portable-branch"
    assert imported.evidence_artifacts
    assert receiver.project_import(bundle, intent) == imported
    project = receiver.project_record(imported.project)
    branch = receiver.project_branch_record(imported.branch)
    assert project is not None
    assert project.title == "Imported project"
    assert branch is not None
    assert branch.problem == imported.problem
    assert len(receiver.project_branches(imported.project).branches) == 1

    # End the supplied reader's custody and remove only this test's exported directory.
    del bundle
    shutil.rmtree(destination)
    assert not destination.exists()
    with pytest.raises(pse.InspectionError):
        pse.ProjectBundle(destination)
    reopened = receiver.project_import_open(intent)
    assert reopened.canonical_problem == imported.problem
    assert reopened.canonical_revision == imported.revision
    assert reopened.operation_source.revision == origin.source_identity
    assert reopened.operation_source.physical_context == origin.physical_context
    original = reopened.declarations()
    assert any(row.name == "value" for row in original)
    changed = tuple(
        attrs.evolve(row, name="renamed_value") if row.name == "value" else row
        for row in original
    )
    edited = reopened.with_declarations(changed)
    assert edited.revision_id != reopened.revision_id
    assert any(row.name == "renamed_value" for row in edited.declarations())
    assert reopened.declarations() == original
    # Reopening is still an explicit original import operation, independent of the removed pack.
    assert receiver.project_import_open(intent).canonical_revision == imported.revision


@pytest.mark.component
def test_descriptive_pack_is_inspectable_but_not_executable_import_authority(
    inspection_settings: pse.EngineSettings,
    canonical_substrate: CanonicalFixture,
    tmp_path: Path,
) -> None:
    runtime = canonical_substrate.runtime(inspection_settings)
    context = physical(runtime)
    selection = selected_project(runtime, context)
    destination = tmp_path / "descriptive-pack"
    runtime.project_export(destination, export_request(selection, descriptive_only=True), context)
    bundle = pse.ProjectBundle(destination)
    assert bundle.manifest.offline_complete
    assert bundle.manifest.descriptive_only
    assert not bundle.manifest.executable_complete
    with pytest.raises(pse.InspectionError):
        runtime.project_import_issue(bundle, title="Must not become executable")


@pytest.mark.component
@pytest.mark.parametrize("damage", ["missing", "changed"])
def test_pack_refuses_missing_or_same_extent_changed_required_member(
    inspection_settings: pse.EngineSettings,
    canonical_substrate: CanonicalFixture,
    tmp_path: Path,
    damage: str,
) -> None:
    runtime = canonical_substrate.runtime(inspection_settings)
    context = physical(runtime)
    selection = selected_project(runtime, context)
    destination = tmp_path / "damaged-pack"
    runtime.project_export(destination, export_request(selection), context)
    bundle = pse.ProjectBundle(destination)
    source = next(s for s in bundle.manifest.sources if s.revision == selection.revision)
    document = next(d for d in source.documents if d.path == "models/root.pse")
    member = next(m for m in bundle.manifest.members if m.id == document.member)
    payload = destination / member.relative_path
    if damage == "missing":
        payload.unlink()
    else:
        before = payload.read_bytes()
        changed = before.replace(b"value:Scalar=1", b"value:Scalar=2")
        assert changed != before and len(changed) == len(before)
        payload.write_bytes(changed)
    # Even an already opened reader rechecks its exact member before returning bytes.
    with pytest.raises(pse.InspectionError):
        bundle.member_bytes(document.member, maximum_bytes=1 << 20)
    with pytest.raises(pse.InspectionError):
        pse.ProjectBundle(destination)
