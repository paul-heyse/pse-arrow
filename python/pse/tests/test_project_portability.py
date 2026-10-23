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
from pse.contracts.values import ContentHash, SemanticId
from pse.contracts.documents import (
    ProjectExportRequest,
    ProjectReferenceBranch,
    ProjectSelectRequest,
    ProjectSelection,
    ProjectSelectionBody,
    SourceDelta,
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


def assert_origin(actual: ProjectSelectionBody, expected: ProjectSelection) -> None:
    # Nested bodies intentionally omit the root document header.
    assert actual.issuing_store == expected.issuing_store
    assert actual.project == expected.project
    assert actual.reference == expected.reference
    assert actual.problem == expected.problem
    assert actual.revision == expected.revision
    assert actual.source_identity == expected.source_identity
    assert actual.physical_context == expected.physical_context


def export_request(
    selection: ProjectSelection, *, descriptive_only: bool = False
) -> ProjectExportRequest:
    return ProjectExportRequest(
        selection=ProjectSelectionBody(
            issuing_store=selection.issuing_store, project=selection.project,
            reference=selection.reference, problem=selection.problem,
            revision=selection.revision, source_identity=selection.source_identity,
            physical_context=selection.physical_context,
        ),
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
    assert_origin(closure.manifest.origin, selection)
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
    assert_origin(manifest.origin, selection)
    assert manifest == published
    selected = next(s for s in manifest.sources if s.revision == selection.revision)
    assert selected.source_identity == selection.source_identity.hex()
    assert selected.physical_context == selection.physical_context.hex()
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
    expected_origin = msgspec.json.decode(codec.encode_json(selection), type=dict[str, object])
    expected_origin.pop("document_version")
    assert result["origin"] == expected_origin
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
    assert_origin(imported.origin, origin)
    assert imported.bundle_digest == bundle.digest
    assert imported.project != origin.project
    assert imported.problem != origin.problem
    assert imported.revision != origin.revision
    assert imported.branch != "portable-branch"
    assert imported.evidence_artifacts
    assert receiver.project_import(bundle, intent) == imported
    assert receiver.project_import_status(intent) == imported
    assert imported.artifact_intent is not None
    recovered = receiver.project_import_recover(intent, imported.artifact_intent)
    assert recovered.state == "project_complete"
    assert recovered.import_.project == imported.project
    assert recovered.import_.revision == imported.revision
    assert recovered.import_.project_complete
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
    # Relation rows carry Arrow column names; the draft consumes generated JSON
    # document rows. Use the declared converter rather than placing attrs rows in
    # a msgspec constructor whose annotations are not runtime validation.
    rows = codec.converter()
    rows.register_unstructure_hook(SemanticId, SemanticId.to_hex)
    rows.register_unstructure_hook(ContentHash, ContentHash.to_prefixed)
    edit = codec.convert_document(
        {
            "parent": reopened.revision_id,
            "replacements": [
                rows.unstructure(row)
                for row, prior in zip(changed, original, strict=True)
                if row != prior
            ],
        },
        SourceDelta,
    )
    delta = receiver.project_draft_delta(reopened, edit)
    body = codec.encode_json(delta)
    save = receiver.project_draft_save_intent("import-edit-save", imported.project, imported.branch, "import-edit", imported.revision, body)
    receiver.project_draft_save(save, body)
    edited = receiver.project_admit_draft("import-edit-admit", "import-edit", reopened.physical_context)
    assert edited.canonical_revision is None
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
    runtime.project_export(destination, export_request(selection, descriptive_only=True))
    bundle = pse.ProjectBundle(destination)
    assert bundle.manifest.offline_complete
    assert bundle.manifest.descriptive_only
    assert not bundle.manifest.members
    assert bundle.manifest.source_revisions
    assert any(omission.required_for_execution for omission in bundle.manifest.omissions)
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
