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
    original = {"package.toml": manifest, "models/model.pse": "package p {param value:Scalar=1;}"}
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
    content = codec.encode_json(ProjectDraftDocuments(
        packages=({path: tuple(text.encode()) for path, text in modified.items()},), version=1
    ))
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
        "advance-draft", "project", "main", "draft", "publish-valid", b"later invalid body", expected=3
    )
    runtime.project_draft_save(advanced, b"later invalid body")
    replayed = runtime.project_publish_draft(intent, context)
    assert replayed.canonical_revision == "publish-valid"
    for field, value in (("draft", "other-draft"), ("project", "other-project"),
                         ("expected", "other-base"), ("kind", "fork")):
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
    assert large_record.artifact is not None
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
