# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Project custody and publication exercise the real typed Python boundary."""

from pathlib import Path

import msgspec
import pytest

import pse
from pse import codec
from pse.contracts.documents import ProjectDraftDocuments
from pse.tests.canonical_fixture import CanonicalFixture
from pse.tests.test_modeling_kernel import on_primitives, physical


@pytest.mark.component
def test_project_saved_draft_admission_publish_and_reopen(
    inspection_settings: pse.EngineSettings, canonical_substrate: CanonicalFixture
) -> None:
    runtime = canonical_substrate.runtime(inspection_settings)
    root = Path(__file__).resolve().parents[3]
    manifest = on_primitives(
        (root / "tests/fixtures/packages/minimal_explicit/package.toml")
        .read_text()
        .replace('id_policy = "explicit"', 'id_policy = "named"')
    )
    original = {
        "package.toml": manifest,
        "models/model.pse": "package p {param value:Scalar=1;}",
    }
    context = physical(runtime)
    source = runtime.modeling_from_documents([original], context)
    runtime.project_create("create", "project", "Engineering study")
    base = runtime.project_fork("project", "main", "branch-problem", "fork", source)
    branch = runtime.project_branch_record("main")
    assert branch is not None
    assert branch.problem == "branch-problem"

    # Raw custody accepts invalid and empty bodies without source admission.
    empty = runtime.project_draft_save_intent(
        "save-empty", "project", "main", "draft", base, b""
    )
    runtime.project_draft_save(empty, b"")
    assert runtime.project_draft_bytes("draft") == b""
    malformed = runtime.project_draft_save_intent(
        "save-malformed", "project", "main", "draft", base, b"invalid [[", expected=1
    )
    runtime.project_draft_save(malformed, b"invalid [[")
    malformed_publish = runtime.project_publication_intent(
        "project", "main", base, "publish-invalid", draft="draft"
    )
    with pytest.raises(pse.InspectionError):
        runtime.project_publish_draft(malformed_publish, context)
    assert runtime.project_acknowledgement("publish-invalid") is None

    modified = {**original, "models/model.pse": "package p {param value:Scalar=2;}"}
    content = codec.encode_json(
        ProjectDraftDocuments(
            packages=({path: tuple(text.encode()) for path, text in modified.items()},),
            version=1,
        )
    )
    save = runtime.project_draft_save_intent(
        "save-valid", "project", "main", "draft", base, content, expected=2
    )
    runtime.project_draft_save(save, content)
    intent = runtime.project_publication_intent(
        "project", "main", base, "publish-valid", draft="draft"
    )
    published = runtime.project_publish_draft(intent, context)
    assert published.canonical_problem == "branch-problem"
    assert published.canonical_revision == "publish-valid"
    acknowledgement = runtime.project_acknowledgement("publish-valid")
    assert acknowledgement is not None
    assert acknowledgement.revision == "publish-valid"
    reopened = runtime.modeling_revision("publish-valid", context)
    assert reopened.canonical_problem == "branch-problem"
    assert [row.name for row in reopened.declarations()] == [
        row.name for row in published.declarations()
    ]

    # An immutable accepted publication settles before any later mutable draft read.
    advanced = runtime.project_draft_save_intent(
        "advance-draft",
        "project",
        "main",
        "draft",
        "publish-valid",
        b"later invalid body",
        expected=3,
    )
    runtime.project_draft_save(advanced, b"later invalid body")
    replayed = runtime.project_publish_draft(intent, context)
    assert replayed.canonical_revision == "publish-valid"
    for field, value in (
        ("draft", "other-draft"),
        ("project", "other-project"),
        ("expected", "other-base"),
        ("kind", "fork"),
    ):
        forged = msgspec.structs.replace(intent, **{field: value})
        with pytest.raises(pse.InspectionError):
            runtime.project_publish_draft(forged, context)

    # Larger malformed custody uses the existing opaque sealed artifact path, and
    # replay retains the original issued physical occurrence rather than minting another.
    large = b"[malformed raw draft" * 10_000
    large_intent = runtime.project_draft_save_intent(
        "save-large", "project", "main", "large-draft", "publish-valid", large
    )
    first = runtime.project_draft_save(large_intent, large)
    second = runtime.project_draft_save(large_intent, large)
    assert first.key == second.key
    large_record = runtime.project_draft_record("large-draft")
    assert large_record is not None
    assert large_record.version.artifact is not None
    assert runtime.project_draft_bytes("large-draft") == large

    changed = b"different malformed body" * 10_000
    changed_issue = runtime.project_draft_save_intent(
        "other-issue", "project", "main", "large-draft", "publish-valid", changed
    )
    forged = msgspec.structs.replace(
        large_intent, digest=changed_issue.digest, bytes=changed_issue.bytes
    )
    with pytest.raises(pse.InspectionError):
        runtime.project_draft_save(forged, changed)
    assert runtime.project_draft_bytes("large-draft") == large

@pytest.mark.component
def test_project_versions_local_admission_and_retained_selection(
    inspection_settings: pse.EngineSettings, canonical_substrate: CanonicalFixture
) -> None:
    import json

    from pse.contracts.documents import (
        ProjectMetadataRequest,
        ProjectNavigationRequest,
        ProjectSelectRequest,
        SourceDelta,
    )

    runtime = canonical_substrate.runtime(inspection_settings)
    root = Path(__file__).resolve().parents[3]
    manifest = on_primitives(
        (root / "tests/fixtures/packages/minimal_explicit/package.toml")
        .read_text()
        .replace('id_policy = "explicit"', 'id_policy = "named"')
    )
    original = {"package.toml": manifest, "models/model.pse": "package p {param value:Scalar=1;}"}
    context = physical(runtime)
    source = runtime.modeling_from_documents([original], context)
    runtime.project_create("create-versioned", "versioned", "Draft editing")
    base = runtime.project_fork("versioned", "branch-stable", "version-owner", "fork-versioned", source)
    selected = runtime.project_select(
        codec.decode_json('{"version":1,"reference":{"kind":"branch","key":"branch-stable"}}', ProjectSelectRequest),
        context,
    )
    assert selected.project == "versioned"
    assert selected.revision == base

    def navigate(operation: str, action: dict[str, object]) -> None:
        project = runtime.project_record("versioned")
        assert project is not None
        request = codec.decode_json(json.dumps({"version": 1, "action": action}), ProjectNavigationRequest)
        runtime.project_navigate(operation, project, request)

    malformed = runtime.project_draft_save_intent("raw-one", "versioned", "branch-stable", "raw", base, b"bad [[")
    runtime.project_draft_save(malformed, b"bad [[")
    with pytest.raises(pse.InspectionError):
        runtime.project_admit_draft("admit-invalid", "raw", context)
    diagnostics = runtime.project_metadata(ProjectMetadataRequest(kind="diagnostics", owner="raw-one", cursor=None))
    assert len(diagnostics.diagnostics) == 1
    assert not diagnostics.diagnostics[0].admitted
    navigate("point", {"kind": "savepoint", "key": "point-stable", "name": "before fix", "draft": "raw", "version": "raw-one", "generation": 1})
    modified = {**original, "models/model.pse": "package p {param value:Scalar=2;}"}
    body = codec.encode_json(ProjectDraftDocuments(version=1, packages=({k: tuple(v.encode()) for k, v in modified.items()},)))
    save = runtime.project_draft_save_intent("raw-two", "versioned", "branch-stable", "raw", base, body, expected=1)
    runtime.project_draft_save(save, body)
    admitted = runtime.project_admit_draft("admit-valid", "raw", context)
    assert admitted.canonical_revision is None
    assert runtime.project_open_selection(selected, context).canonical_revision == base
    before = runtime.modeling_revision(base, context)
    typed = runtime.project_draft_delta(before, SourceDelta(parent=before.revision_id))
    typed_body = codec.encode_json(typed)
    intent = runtime.project_publication_intent("versioned", "branch-stable", base, "publish-versioned", draft="raw")
    published = runtime.project_publish_draft(intent, context)
    assert published.canonical_revision == "publish-versioned"
    # A retained selection still opens its exact original source after head movement.
    assert runtime.project_open_selection(selected, context).canonical_revision == base
    wrong_base = runtime.project_draft_save_intent("wrong-base", "versioned", "branch-stable", "wrong", "publish-versioned", typed_body)
    runtime.project_draft_save(wrong_base, typed_body)
    with pytest.raises(pse.InspectionError):
        runtime.project_admit_draft("wrong-parent-admit", "wrong", context)
    assert runtime.project_draft_bytes("wrong") == typed_body
    navigate("restore-point", {"kind": "restore", "key": "point-stable", "draft": "raw", "generation": 2})
    restored = runtime.project_draft_record("raw")
    assert restored is not None
    assert restored.head.generation == 3
    assert restored.version.key == "raw-one"
    assert runtime.project_draft_bytes("raw") == b"bad [["
    assert runtime.project_draft_version("raw-two") is not None
    assert runtime.project_draft_version_bytes("raw-two") == body
    navigate("rename", {"kind": "branch_name", "key": "branch-stable", "name": "renamed"})
    navigate("bookmark", {"kind": "bookmark", "key": "baseline-bookmark", "name": "baseline", "revision": base})
    bookmark = runtime.project_select(codec.decode_json('{"version":1,"reference":{"kind":"bookmark","key":"baseline-bookmark"}}', ProjectSelectRequest), context)
    assert runtime.project_open_selection(bookmark, context).canonical_revision == base
    runtime.close()
