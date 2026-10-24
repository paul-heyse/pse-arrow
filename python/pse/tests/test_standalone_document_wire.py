# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Standalone envelopes preserve domain bodies and original version headers."""

import msgspec
import pytest

import pse
from pse import codec
from pse.contracts import documents


@pytest.mark.unit
def test_standalone_build_info_preserves_its_domain_version() -> None:
    value = pse.build_info()
    assert value.document_version == 1
    assert isinstance(value.version, str)
    assert codec.decode_json(codec.encode_json(value), documents.BuildInfo) == value


@pytest.mark.unit
def test_standalone_history_envelope_keeps_nested_domain_bodies() -> None:
    value = codec.decode_json(
        b'{"document_version":1,"key":"obligation",'
        b'"owner":{"kind":"run","key":"original-owner"},'
        b'"purpose":"audit","attribution":"caller"}',
        documents.HistoryObligation,
    )
    wire = codec.decode_json(codec.encode_json(value), dict[str, msgspec.Raw])
    assert bytes(wire["purpose"]) == b'"audit"'
    owner = codec.decode_json(wire["owner"], dict[str, msgspec.Raw])
    assert "document_version" not in owner
    # The same owner exported independently has a required standalone header.
    with pytest.raises(msgspec.ValidationError, match="version"):
        codec.decode_json(wire["owner"], documents.HistoryOwner)
    standalone = codec.decode_json(
        b'{"document_version":1,"kind":"run","key":"original-owner"}',
        documents.HistoryOwner,
    )
    assert standalone.key == value.owner.key


@pytest.mark.unit
@pytest.mark.parametrize(
    "document",
    [documents.CanonicalGraphRequest, documents.CanonicalGraphDocument],
)
def test_graph_documents_refuse_missing_or_future_header_before_body(
    document: type,
) -> None:
    for payload in (b"{}", b'{"version":2,"anchor":null}'):
        with pytest.raises(msgspec.ValidationError, match="version"):
            codec.decode_json(payload, document)


@pytest.mark.unit
def test_graph_request_roundtrips_its_standalone_envelope() -> None:
    value = codec.decode_json(
        b'{"version":1,"anchor":{"kind":"artifact",'
        b'"artifact":"artifact-a","revision":"revision-a"},'
        b'"scope":"retained_consumers","limit":1,"continuation":null}',
        documents.CanonicalGraphRequest,
    )
    assert value.version == 1
    assert (
        codec.decode_json(codec.encode_json(value), documents.CanonicalGraphRequest)
        == value
    )


@pytest.mark.unit
@pytest.mark.parametrize(
    ("document", "field"),
    [
        (documents.NumericalPolicy, "version"),
        (documents.PreparationSettings, "document_version"),
    ],
)
@pytest.mark.parametrize(("first", "last"), [(1, 1), (1, 2), (2, 1)])
def test_standalone_duplicate_headers_refuse_original_root_and_nested_bytes(
    document: type,
    field: str,
    first: int,
    last: int,
) -> None:
    payload = f'{{"{field}":{first},"{field}":{last}}}'.encode()
    with pytest.raises(msgspec.ValidationError, match="version"):
        codec.decode_json(payload, document)
    with pytest.raises(msgspec.ValidationError, match="version"):
        codec.decode_json(b"[" + payload + b"]", list[document])


@pytest.mark.unit
def test_standalone_native_byte_ingress_uses_the_same_envelope() -> None:
    policy = codec.convert_document(
        {
            "profile": "synthetic",
            "maximum_findings": 50,
            "near_bound_absolute": 1e-4,
            "near_bound_relative": 1e-4,
            "bound_violation": 0.0,
            "residual": 1e-5,
            "variable_small": 1e-4,
            "variable_large": 1e4,
            "variable_zero": 1e-8,
            "jacobian_small": 1e-4,
            "jacobian_large": 1e4,
            "matrix": {
                "dense_entries": 100,
                "findings": 50,
                "parallel_tolerance": 1e-8,
                "rank_absolute": 1e-12,
                "rank_relative": 1e-8,
                "singular_vector": 0.1,
            },
            "terms": {
                "zero": 1e-10,
                "mismatch": 1e6,
                "cancellation": 1e-4,
                "maximum_terms": 5,
                "combinations": 100,
                "findings": 50,
            },
        },
        documents.ModelingDiagnosticPolicy,
    )
    source = codec.encode_json(policy).decode()
    settings = pse.ModelingDiagnosticSettings.from_json(source)
    assert (
        codec.decode_json(settings.to_json(), documents.ModelingDiagnosticPolicy)
        == policy
    )
    for payload in (
        "{}",
        '{"document_version":1,"document_version":1}',
        '{"document_version":2,"private":"header-control-secret"}',
    ):
        with pytest.raises(pse.InspectionError) as failure:
            pse.ModelingDiagnosticSettings.from_json(payload)
        report = failure.value.report
        assert report.reported_code == "validation.invariant"
        location = report.decode_location
        assert isinstance(location, documents.DecodeLocation)
        assert location.stage == "header"
        assert location.category == "data"
        assert location.line == 1
        assert location.column > 0
        assert "header-control-secret" not in str(failure.value)


@pytest.mark.unit
def test_standalone_current_operation_can_be_projected_and_promoted_explicitly() -> (
    None
):
    original = documents.OperationId(process_instance=(1, 2), sequence=3)
    body = msgspec.convert(
        original, type=documents.OperationIdBody, from_attributes=True
    )
    assert body.process_instance == original.process_instance
    assert body.sequence == original.sequence
    promoted = codec.convert_document(
        codec.document_builtins(body), documents.OperationId
    )
    assert promoted == original
    assert (
        pse.diagnostics.operation_text(promoted) == "00000000000000010000000000000002:3"
    )
    with pytest.raises(msgspec.ValidationError, match="version"):
        codec.decode_json(codec.encode_json(body), documents.OperationId)


@pytest.mark.unit
@pytest.mark.parametrize(
    "payload",
    [
        b"{}",
        b'{"version":999,"secret":"credential-sentinel"}',
        b'{"version":1,"version":1}',
    ],
)
def test_version_refusal_preserves_native_structural_report(payload: bytes) -> None:
    with pytest.raises(msgspec.ValidationError) as caught:
        codec.decode_json(payload, documents.CanonicalGraphRequest)
    original = caught.value.__cause__
    assert original is not None
    report = getattr(original, "report", None)
    assert report is not None
    assert report.decode_location.stage == "header"
    assert report.decode_location.category == "data"
    assert report.source_locations == ()
    assert "credential-sentinel" not in report.message
