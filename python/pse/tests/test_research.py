# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Public research commands retain original admitted scientific owners."""
import msgspec
import pytest

import pse
from pse import codec
from pse.contracts import documents as d
from pse.tests.canonical_fixture import CanonicalFixture
from pse.tests.physical_input_fixture import primitive_quantity
from pse.tests.test_modeling_kernel import physical
from pse.tests.test_native_workflow import _measurement_fit_package, identity


def _body(value: object) -> dict[str, msgspec.Raw]:
    body = codec.decode_json(codec.encode_json(value), dict[str, msgspec.Raw])
    body.pop("document_version", None)
    return body


@pytest.mark.component
def test_research_dataset_discovery_exact_selection_and_protected_reopen(
    inspection_settings: pse.EngineSettings, canonical_substrate: CanonicalFixture
) -> None:
    runtime = canonical_substrate.runtime(inspection_settings)
    context = physical(runtime)
    source = _measurement_fit_package(runtime, context, fixed=True)
    runtime.project_create("create-research", "research", "Original typed measurements")
    revision = runtime.project_fork(
        "research", "research-main", "research-source", "fork-research", source
    )
    source = runtime.modeling_revision(revision, context)
    project = runtime.project_record("research")
    assert project is not None
    runtime.project_context_append("research-context", project, "measurements", source, 0)
    cut = runtime.project_context_cut("research")
    selected = runtime.project_context_select(cut, "measurements", 1)
    assert selected is not None
    case = next(row.declaration_id for row in source.declarations() if row.name == "Measurement")
    quantity, _unit = primitive_quantity("Length")
    request = codec.decode_json(
        codec.encode_json({
            "document_version": 1,
            "key": "measurements-v1",
            "project": _body(project),
            "context": _body(selected),
            "parent": None,
            "role": "raw",
            "members": [{
                "root": case,
                "record": identity(157),
                "attribute": "value",
                "standard_deviation_attribute": "sigma",
                "quantity": quantity,
                "acquisition_clock": "independently authored known-length fixture",
                "parent_member": None,
                "calibration": [],
            }],
        }),
        d.ResearchDatasetRequest,
    )
    created = runtime.research_dataset_create(request)
    assert runtime.research_dataset_create(request) == created
    page = runtime.research_read(codec.decode_json(
        b'{"document_version":1,"kind":"datasets","project":"research","after":"","limit":1}',
        d.ResearchReadRequest,
    ))
    assert tuple(row.key for row in page.rows) == (created.key,)
    assert runtime.research_dataset_open(created.key, selected).canonical_revision == revision
    wrong_quantity, _ = primitive_quantity("Time")
    wrong_payload = codec.decode_json(codec.encode_json(request), dict[str, msgspec.Raw])
    members = codec.decode_json(bytes(wrong_payload["members"]), list[dict[str, msgspec.Raw]])
    members[0]["quantity"] = msgspec.Raw(codec.encode_json(wrong_quantity))
    wrong_payload["members"] = msgspec.Raw(codec.encode_json(members))
    wrong_payload["key"] = msgspec.Raw(b'"wrong-quantity"')
    with pytest.raises(pse.InspectionError):
        runtime.research_dataset_create(codec.decode_json(codec.encode_json(wrong_payload), d.ResearchDatasetRequest))
    selection = codec.decode_json(codec.encode_json({
        "document_version": 1,
        "key": "observations-v1",
        "project": _body(project),
        "context": _body(selected),
        "dataset": created.key,
        "fit": identity(158),
        "observations": [{"observation": identity(157), "member": 0, "reason": "included known length and authored standard uncertainty"}],
    }), d.ResearchSelectionRequest)
    frozen = runtime.research_selection_create(selection)
    assert frozen.dataset == created.key
    reopened = runtime.research_selection_open(frozen.key)
    assert reopened.canonical_revision == revision
    exact = runtime.research_read(codec.decode_json(
        b'{"document_version":1,"kind":"selection","key":"observations-v1"}',
        d.ResearchReadRequest,
    ))
    assert exact.row == frozen
    # A later context correction does not retarget the old selection.
    runtime.project_context_append("later-context", project, "measurements", source, 0, corrects="research-context")
    assert runtime.research_selection_open(frozen.key).canonical_revision == revision
