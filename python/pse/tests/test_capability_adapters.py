# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Actual capability consumers preserve native admission, identities and lifetimes."""

from __future__ import annotations

import asyncio
from pathlib import Path
from unittest.mock import patch

import pytest
from fastmcp import Client

import pse
from pse import codec
from pse.contracts import documents
from pse.tests.canonical_fixture import CanonicalFixture
from scripts.capabilities import native_worker
from scripts.capabilities.native_schema import JSON_OBJECT
from scripts.tests.test_capability_native import preparation_request, source_request

ROOT = Path(__file__).resolve().parents[3]


def invoke(operation: str, arguments: dict[str, object]) -> dict[str, object]:
    from scripts.capabilities.server import build

    async def consume() -> dict[str, object]:
        runtime = build(ROOT)
        capability = "native_" + operation
        tool = runtime.catalogue.tools[capability]
        assert tool.annotations is not None
        invoker = "capabilities_invoke_read" if tool.annotations.read_only_hint else "capabilities_invoke_action"
        async with Client(runtime.server) as client:
            await client.call_tool("capabilities_select", {"capabilities": [capability]})
            result = await client.call_tool(invoker, {"capability": capability, "arguments": arguments})
            assert isinstance(result.structured_content, dict)
            payload = result.structured_content["payload"]
            assert isinstance(payload, dict)
            return payload

    return asyncio.run(consume())


@pytest.mark.unit
def test_capability_original_source_admission_and_same_lifetime_preparation(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    missing = tmp_path / "never-opened-store"
    monkeypatch.setenv("PSE_SURREAL_STATE", str(missing))
    request = source_request()
    admitted = invoke("source_admit", request)
    assert admitted["outcome"] == "admitted", admitted
    assert admitted["durable"] is False
    assert admitted["revision"]["reopenable"] is False
    assert admitted["revision"]["lifetime"] == "completed_child"
    assert admitted["source"]["physical_context"]
    # Explicit authored identity makes independently expected discovery possible.
    root = "bb" * 16
    request["documents"]["packages"][0]["models/fixed.pse"] = list(
        f'package local {{@id("{root}") def Root {{var x:Scalar;eq e:x*x==1;}}}}'.encode()
    )
    request["request"] = preparation_request(root)
    observed = invoke("preparation_discover", request)
    assert observed["outcome"] == "admitted", observed
    assert observed["preparation"]["resident"] == []
    assert observed["preparation"]["residents_truncated"] is False
    assert not missing.exists()


@pytest.mark.unit
def test_capability_native_loader_refuses_schema_shaped_invalid_authored_source() -> None:
    request = source_request()
    request["documents"]["packages"][0]["models/fixed.pse"] = list(b"THIS IS ORIGINAL INVALID SOURCE secret-value")
    refused = invoke("source_admit", request)
    assert refused["outcome"] == "native_refusal", refused
    assert refused["error"]["code"]
    assert refused["error"]["code"] != "config.invalid"
    assert "secret-value" not in str(refused)


@pytest.mark.unit
def test_capability_native_codec_refuses_before_runtime_effect(tmp_path: Path) -> None:
    bad_requests = [
        ("project_select", {"state": "/explicit", "database": "existing", "physical": {"version": 1, "packages": [{}]}, "request": {"version": 1, "reference": {"kind": "unknown", "key": "main"}}}),
        ("source_admit", {"physical": {"version": 1, "packages": [{"a": [256]}]}, "documents": {"version": 1, "packages": []}}),
        ("source_admit", {"physical": {"version": 1, "packages": [], "unknown": 1}, "documents": {"version": 1, "packages": []}}),
        ("source_admit", {"physical": {"version": 2, "packages": [{}]}, "documents": {"version": 1, "packages": []}}),
    ]
    for operation, request in bad_requests:
        request["schema_set_fingerprint"] = pse.SCHEMA_SET_FINGERPRINT
        with patch.object(pse, "Runtime", side_effect=AssertionError("must not create runtime")), pytest.raises((ValueError, TypeError)):
            native_worker.execute(operation, request, scratch=tmp_path)
    with patch.object(pse, "Runtime", side_effect=AssertionError("must not create runtime")):
        stale = native_worker.execute("source_admit", {"schema_set_fingerprint": "blake3:stale"}, scratch=tmp_path)
    assert stale["outcome"] == "stale_interface"


@pytest.mark.component
def test_capability_project_selection_metadata_and_retained_revision(
    inspection_settings: pse.EngineSettings, canonical_substrate: CanonicalFixture,
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch,
) -> None:
    request = source_request()
    runtime = canonical_substrate.runtime(inspection_settings)
    physical = runtime.physical_from_documents(native_worker.source_documents(request["physical"])[0])
    package = runtime.modeling_from_documents(native_worker.source_documents(request["documents"]), physical)
    runtime.project_create("capability-create", "capability-project", "Native capability consumer")
    canonical = runtime.project_fork("capability-project", "capability-main", "capability-problem", "capability-fork", package)
    deployment = {"state": canonical_substrate.state, "database": canonical_substrate.database}
    selected = invoke("project_select", {**deployment, "physical": request["physical"], "request": {"version": 1, "reference": {"kind": "branch", "key": "capability-main"}}})
    assert selected["outcome"] == "selected", selected
    decoded = codec.decode_json(codec.encode_json(selected["document"]), documents.ProjectSelection)
    assert decoded.revision == canonical
    from pse.tests.test_project_portability import export_request

    pack = tmp_path / "selected-pack"
    published = runtime.project_export(pack, export_request(decoded), physical)
    missing = tmp_path / "never-opened-offline-store"
    with monkeypatch.context() as offline:
        offline.setenv("PSE_SURREAL_STATE", str(missing))
        bundle = invoke("project_bundle_inspect", {"path": str(pack)})
    assert bundle["outcome"] == "observed", bundle
    manifest = codec.decode_json(codec.encode_json(bundle["document"]), documents.ProjectManifestOutput)
    assert manifest.origin.revision == published.origin.revision == canonical
    assert bundle["digest"] == pse.ProjectBundle(pack).digest
    assert not missing.exists()
    metadata = invoke("project_metadata", {**deployment, "request": {"document_version": 1, "kind": "projects", "owner": "", "cursor": None}})
    assert metadata["outcome"] == "observed", metadata
    page = codec.decode_json(codec.encode_json(metadata["document"]), documents.ProjectMetadataPageOutput)
    assert any(project.key == "capability-project" for project in page.projects)
    source = invoke("revision_source", {**deployment, "physical": request["physical"], "revision": canonical, "logical": ["models/fixed.pse"]})
    assert source["outcome"] == "observed", source
    assert source["canonical_revision"] == canonical
    stale = invoke("project_select", {**deployment, "physical": request["physical"], "request": {"version": 1, "reference": {"kind": "branch", "key": "not-an-existing-branch"}}})
    assert stale["outcome"] == "native_refusal", stale
    assert stale["error"]["code"]
    wrong_cursor = invoke("project_metadata", {**deployment, "request": {
        "document_version": 1, "kind": "projects", "owner": "",
        "cursor": {"issuing_store": "not-this-installation", "kind": "projects", "owner": "", "after": ""},
    }})
    assert wrong_cursor["outcome"] == "native_refusal", wrong_cursor
    assert wrong_cursor["error"]["code"]


@pytest.mark.integration
def test_capability_stored_result_preserves_native_arrow_window(
    inspection_settings: pse.EngineSettings, canonical_substrate: CanonicalFixture
) -> None:
    import base64

    import pyarrow as pa

    from pse.tests.test_canonical_results import _package, _settings

    runtime = canonical_substrate.runtime(inspection_settings)
    package, case = _package(runtime)
    prepared = package.prepare_solve(case, _settings())
    handle = prepared.start()
    result = handle.wait()
    assert result.usable
    run, attempt = result.canonical_run_key, result.canonical_attempt_key
    assert run is not None and attempt is not None
    relation = "runtime.solve_variables"
    with runtime.results(run, attempt, relation, start=0, end=1) as expected_stream:
        expected = pa.table(expected_stream)
    response = invoke("result_read", {
        "state": canonical_substrate.state, "database": canonical_substrate.database,
        "run": run, "attempt": attempt, "relation": relation,
        "start": 0, "end": 1, "maximum_bytes": 65536,
    })
    assert response["outcome"] == "observed", response
    assert (response["run"], response["attempt"], response["relation"]) == (run, attempt, relation)
    assert response["encoding"] == "arrow-ipc-stream-base64"
    raw = base64.b64decode("".join(response["chunks"]), validate=True)
    with pa.ipc.open_stream(raw) as reader:
        actual = reader.read_all()
    assert actual.schema.equals(expected.schema, check_metadata=True)
    assert actual.equals(expected, check_metadata=True)


@pytest.mark.unit
def test_capability_offline_diagnostic_reader_preserves_original_native_outcome(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    import msgspec

    products = codec.decode_json(
        (Path(__file__).parent / "fixtures/generated-native-boundaries/products.json").read_bytes(),
        dict[str, msgspec.Raw],
    )
    original = codec.decode_json(bytes(products["diagnostic_inspection"]), documents.InspectionDocument)
    assert original.operation is not None
    image = documents.ProcessCheckpoint(
        version=4, process="0000000000000001-0000000000000002", sequence=1,
        published_unix_micros=1, operations=(original.operation,), omitted=0,
        summaries=documents.SummaryCoverage(refused=0, expired=0, pressure_lost=0),
        registry_coverage=original.operation.coverage,
        resources=documents.ProcessResources(resident_bytes=None, lifetime_peak_resident_bytes=None, user_cpu_ticks=None, system_cpu_ticks=None),
        parent=None, locator=documents.ProcessLocatorBody(pid=(1 << 32) - 1, start_ticks=77, executable=None),
        clock=None, threads=(), threads_partial=False,
    )
    checkpoint = tmp_path / "retained.json"
    checkpoint.write_bytes(codec.encode_json(image))
    missing = tmp_path / "never-opened-offline-store"
    monkeypatch.setenv("PSE_SURREAL_STATE", str(missing))
    observed = invoke("diagnostic_inspect", {"checkpoint": str(checkpoint), "request": JSON_OBJECT.validate_json(codec.encode_json(original.request), strict=True)})
    assert observed["outcome"] == "observed", observed
    inspected = codec.decode_json(codec.encode_json(observed["document"]), documents.InspectionDocumentOutput)
    assert inspected.operation is not None
    assert inspected.operation.id == original.operation.id
    assert inspected.operation.outcome == original.operation.outcome
    assert inspected.transport.partial
    assert not missing.exists()
