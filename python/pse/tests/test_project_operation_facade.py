# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Public facade metadata and receiver routing; native owner effects are separate."""

from __future__ import annotations

import inspect
from typing import TYPE_CHECKING, cast

import pytest

import pse
from pse._project_bundle import ProjectBundle
from pse.contracts import documents as d
from pse.tests.invalid_call import invalid_call

if TYPE_CHECKING:
    from pse._build import _NativeProjectBundle, _NativeRuntime


class FacadeBoundary:
    """Observe original arguments without pretending to perform native admission."""

    def __init__(self) -> None:
        self.calls: list[tuple[str, object]] = []
        self.import_intent = object()
        self.datasets = d.ResearchReadResultOutputDatasets(rows=())
        self.branches = d.ProjectBranchPageOutput(
            branches=(),
            next=None,
            project=d.ProjectRecordBody(
                deleted=False, generation=3, key="project", title="Original"
            ),
        )

    def project_import_issue(
        self,
        bundle: _NativeProjectBundle,
        *,
        title: str,
        branch_name: str,
    ) -> d.ProjectImportIntentOutput:
        self.calls.append(("import", (bundle, title, branch_name)))
        return cast("d.ProjectImportIntentOutput", self.import_intent)

    def project_branches(
        self, project: str, *, cursor: d.ProjectCursor | None
    ) -> d.ProjectBranchPageOutput:
        self.calls.append(("branches", (project, cursor)))
        return self.branches

    def research_read(
        self, request: d.ResearchReadRequest
    ) -> d.ResearchReadResultOutput:
        self.calls.append(("research", request))
        return self.datasets


def runtime(boundary: FacadeBoundary) -> pse.Runtime:
    result = object.__new__(pse.Runtime)
    object.__setattr__(result, "_handle", cast("_NativeRuntime", boundary))
    return result


@pytest.mark.unit
@pytest.mark.parametrize(
    ("name", "qualname", "module"),
    [
        ("project_import_issue", "Runtime.project_import_issue", "pse._workflow"),
        ("research_read", "ResearchRuntime.research_read", "pse._research"),
    ],
)
def test_projected_public_methods_retain_inspection_identity(
    name: str,
    qualname: str,
    module: str,
) -> None:
    method = getattr(pse.Runtime, name)
    assert method.__name__ == name
    assert method.__qualname__ == qualname
    assert method.__module__ == module


@pytest.mark.unit
def test_public_annotations_preserve_existing_eager_and_postponed_forms() -> None:
    assert pse.Runtime.project_import_issue.__annotations__["title"] is str
    assert (
        pse.Runtime.project_import_issue.__annotations__["bundle"] is pse.ProjectBundle
    )
    assert (
        pse.Runtime.project_import_issue.__annotations__["return"]
        is d.ProjectImportIntentOutput
    )
    assert (
        pse.Runtime.project_context_package.__annotations__["physical"]
        == "PhysicalContext"
    )
    assert pse.Runtime.research_read.__annotations__["request"] is d.ResearchReadRequest


@pytest.mark.unit
def test_project_import_keeps_native_bundle_keyword_binding_and_default() -> None:
    boundary = FacadeBoundary()
    selected = runtime(boundary)
    native_bundle = object()
    bundle = object.__new__(pse.ProjectBundle)
    object.__setattr__(bundle, "_handle", cast("_NativeProjectBundle", native_bundle))
    assert (
        selected.project_import_issue(bundle, title="Copied") is boundary.import_intent
    )
    assert (
        selected.project_import_issue(bundle, title="Other", branch_name="branch")
        is boundary.import_intent
    )
    assert boundary.calls == [
        ("import", (native_bundle, "Copied", "main")),
        ("import", (native_bundle, "Other", "branch")),
    ]
    parameter = inspect.signature(pse.Runtime.project_import_issue).parameters["title"]
    assert parameter.kind is inspect.Parameter.KEYWORD_ONLY
    with pytest.raises(TypeError):
        invalid_call(selected.project_import_issue, bundle, "positional title")


@pytest.mark.unit
def test_project_cursor_body_keeps_its_original_authority_and_generation() -> None:
    boundary = FacadeBoundary()
    cursor = d.ProjectCursorBody(
        after="last", generation=7, issuing_store="original-store", project="project"
    )
    assert (
        runtime(boundary).project_branches("project", cursor=cursor)
        is boundary.branches
    )
    expected_cursor = d.ProjectCursor(
        after="last", generation=7, issuing_store="original-store", project="project"
    )
    assert boundary.calls == [("branches", ("project", expected_cursor))]


@pytest.mark.unit
def test_research_projection_forwards_the_original_typed_request_and_result() -> None:
    boundary = FacadeBoundary()
    request = d.ResearchReadRequestDatasets(
        after="original-key", limit=3, project="project"
    )
    assert runtime(boundary).research_read(request) is boundary.datasets
    assert boundary.calls == [("research", request)]


@pytest.mark.unit
def test_bundle_reexport_and_existing_document_byte_defaults_are_preserved() -> None:
    assert pse.ProjectBundle is ProjectBundle
    constructor_default = (
        inspect.signature(ProjectBundle).parameters["maximum_document_bytes"].default
    )
    member_default = (
        inspect.signature(ProjectBundle.member_bytes)
        .parameters["maximum_bytes"]
        .default
    )
    assert constructor_default == 16 << 20
    assert member_default == 16 << 20
