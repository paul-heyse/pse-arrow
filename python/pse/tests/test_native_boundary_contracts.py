# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Isolated native boundary admission; no publication, compiler or store fixture."""

from pathlib import Path
from typing import cast

import msgspec
import pytest

import pse
from pse import codec
from pse.contracts import documents, values
from pse.contracts.enums import (
    AttemptState,
    DiagnosticCode,
    FailureClass,
    NativeBoundaryClass,
    NativeCandidateKind,
    NativeQualification,
    NativeTermination,
    StudyAvailability,
    StudyEffectState,
    StudyLifecycle,
    TearPolicy,
    TrajectoryTermination,
)
from pse.contracts.values import ContentHash, SemanticId


def _mutate(instance: object, field: str, value: object) -> None:
    setattr(instance, field, value)


@pytest.mark.unit
@pytest.mark.parametrize("value", [True, False, 1.5, -1, 1 << 65])
def test_native_cache_integer_boundary_rejects_coercion(value: object) -> None:
    with pytest.raises((TypeError, OverflowError)):
        pse.CacheSettings(working_bytes=cast("int", value))


@pytest.mark.unit
@pytest.mark.parametrize("value", [True, 1.5, -1, 1 << 64])
def test_unsigned_millisecond_boundary_rejects_coercion(value: object) -> None:
    with pytest.raises((TypeError, OverflowError)):
        pse.CacheSettings(working_bytes=1024, listing_ttl_ms=cast("int", value))


@pytest.mark.unit
@pytest.mark.parametrize("value", [None, 0, 1, (1 << 64) - 1])
def test_milliseconds_round_trip_without_truncation(value: int | None) -> None:
    settings = pse.CacheSettings(working_bytes=1024, listing_ttl_ms=value)
    assert settings.listing_ttl_ms == value
    with pytest.raises(AttributeError):
        _mutate(settings, "listing_ttl_ms", 2)


@pytest.mark.unit
def test_generated_settings_admit_native_defaults_and_expose_named_diagnostics(
    tmp_path: Path,
) -> None:
    settings = pse.EngineSettings(
        memory_limit_bytes=256 << 20,
        threads=2,
        spill_dir=str(tmp_path),
        max_spill_bytes=1 << 30,
        batch_size=128,
    )
    assert settings.target_partitions == settings.threads == 2
    assert settings.batch_size == 128
    with pytest.raises(AttributeError):
        _mutate(settings, "batch_size", 8)
    with pytest.raises(pse.InspectionError) as failure:
        pse.EngineSettings(
            memory_limit_bytes=0,
            threads=2,
            spill_dir=str(tmp_path),
            max_spill_bytes=1 << 30,
            batch_size=128,
        )
    report = failure.value.report
    assert isinstance(report, pse.DiagnosticReport)
    assert isinstance(report.code, DiagnosticCode)
    assert isinstance(report.failure_class, FailureClass)
    assert isinstance(report.boundary_class, NativeBoundaryClass)
    assert isinstance(report.envelope, documents.BoundaryDiagnostic)
    assert isinstance(report.observations, dict)
    assert isinstance(report.causes, tuple)
    assert isinstance(report.related, tuple)
    assert isinstance(report.contexts, tuple)
    with pytest.raises(AttributeError):
        _mutate(report, "code", "changed")


@pytest.mark.unit
def test_identity_text_uses_native_case_normalization_and_exact_width() -> None:
    semantic = SemanticId.from_hex("AB" * 16)
    digest = ContentHash.from_prefixed("blake3:" + "CD" * 32)
    assert semantic == bytes([0xAB]) * 16
    assert semantic.to_hex() == "ab" * 16
    assert digest.to_prefixed() == "blake3:" + "cd" * 32
    for invalid in ["a" * 31, "a" * 33, "zz" * 16, " a" * 16, "é" * 32]:
        with pytest.raises(ValueError):
            SemanticId.from_hex(invalid)
    for invalid in ["cd" * 32, "BLAKE3:" + "cd" * 32, "blake3:" + "cd" * 31]:
        with pytest.raises(ValueError):
            ContentHash.from_prefixed(invalid)


@pytest.mark.unit
def test_named_report_types_have_no_legacy_tuple_surface() -> None:
    assert not hasattr(pse, "ResourceUsage")
    assert hasattr(pse.ResourceReport, "pool_reserved_now")
    assert hasattr(pse.ResourceReport, "top_consumers")
    assert hasattr(pse.ResourceReport, "caches")
    assert hasattr(pse.TableName, "catalog")
    assert hasattr(pse.TableName, "schema")
    assert hasattr(pse.TableName, "table")
    assert hasattr(pse.ResourceConsumer, "reserved_bytes")


@pytest.mark.unit
def test_owned_flow_document_is_a_shape_not_native_domain_admission() -> None:
    # Native admission rejects duplicate occurrences and negative costs.
    document = codec.decode_json(
        b'{"nodes":[],"connections":[{"connection":"01010101010101010101010101010101","group":"02020202020202020202020202020202","cost":-1.0,"policy":"mandatory"}]}',
        documents.FlowSelectionDocument,
    )
    assert document.connections[0].policy is TearPolicy.MANDATORY
    assert document.connections[0].cost == -1.0
    with pytest.raises(msgspec.ValidationError):
        codec.decode_json(
            b'{"nodes":[],"connections":[],"inference":true}',
            documents.FlowSelectionDocument,
        )
    with pytest.raises(msgspec.ValidationError):
        codec.decode_json(
            codec.encode_json(document).replace(b'"mandatory"', b'"automatic"'),
            documents.FlowSelectionDocument,
        )


@pytest.mark.unit
def test_profile_document_retains_worker_failure_and_absent_scientific_bound() -> None:
    document = codec.decode_json(
        b'{"status":"available","chains":[{"worker_failure":{"kind":"sched'
        b'uling","detail":"pool unavailable"},"scheduling_failures":[],"ac'
        b'tual_parallelism":0,"parameter":"0707070707070707070707070707070'
        b'7","end":"lower","estimate":2.0,"bound":{"value":null,"outcome":'
        b'"stopped"},"detail":"worker did not begin","points":[]}]}',
        documents.FitProfileDocument,
    )
    assert isinstance(document, documents.FitProfileDocumentAvailable)
    assert isinstance(
        document.chains[0].worker_failure, documents.ProfileWorkerFailureScheduling
    )
    assert document.chains[0].actual_parallelism == 0
    assert document.chains[0].worker_failure.detail == "pool unavailable"
    assert document.chains[0].bound.value is None
    with pytest.raises(msgspec.ValidationError):
        codec.decode_json(
            b'{"status":"not_requested","chains":[]}', documents.FitProfileDocument
        )


@pytest.mark.unit
def test_strategy_document_constructor_and_decoder_refuse_duplicate_selected_ids() -> (
    None
):
    identity = "01" * 16
    with pytest.raises(ValueError, match="unique"):
        documents.RecycleRequest(
            tears=(identity, identity), units=(), anderson=0, damping=1.0
        )
    with pytest.raises(msgspec.ValidationError, match="unique"):
        codec.decode_json(
            codec.encode_json(
                {
                    "tears": [identity, identity],
                    "units": [],
                    "anderson": 0,
                    "damping": 1.0,
                }
            ),
            documents.RecycleRequest,
        )


@pytest.mark.unit
def test_generated_document_keys_preserve_mapping_and_signed_zero_equality() -> None:
    first = documents.LinearDiagnosticControls(
        relaxation=(1.0, 1.0, 1.0), lower_penalties={"01" * 16: -0.0, "02" * 16: 2.0}
    )
    second = documents.LinearDiagnosticControls(
        relaxation=(1.0, 1.0, 1.0), lower_penalties={"02" * 16: 2.0, "01" * 16: 0.0}
    )
    assert first == second
    assert values.record_key(first) == values.record_key(second)
    with pytest.raises(ValueError, match="unique"):
        values.unique(values.record_key)(None, None, (first, second))


@pytest.mark.unit
def test_rust_owned_outcome_bytes_preserve_partial_cancelled_and_native_branches() -> (
    None
):
    fixture = (
        Path(__file__).parent
        / "fixtures"
        / "generated-native-boundaries"
        / "products.json"
    )
    products = codec.decode_json(fixture.read_bytes(), dict[str, msgspec.Raw])
    partial = codec.decode_json(bytes(products["partial"]), documents.Conclusion)
    cancelled = codec.decode_json(
        bytes(products["cancelled_partial"]), documents.Conclusion
    )
    assert partial.availability is cancelled.availability is StudyAvailability.PARTIAL
    assert partial.lifecycle is StudyLifecycle.TERMINAL
    assert cancelled.lifecycle is StudyLifecycle.CANCELLED

    refusal = codec.decode_json(
        bytes(products["pre_result_refusal"]), documents.BoundaryDiagnostic
    )
    failed = codec.decode_json(
        bytes(products["failed_attempt"]), documents.TerminationDetail
    )
    stopped = codec.decode_json(
        bytes(products["cancelled_attempt"]), documents.TerminationDetail
    )
    assert failed.point is not None
    assert stopped.point is not None
    assert failed.point.lifecycle is AttemptState.FAILED
    assert stopped.point.lifecycle is AttemptState.CANCELLED
    assert failed.point.diagnostic == refusal
    assert failed.point.start is stopped.point.start is None
    assert not failed.point.scientific.usable
    assert not stopped.point.scientific.usable
    assert failed.effect is stopped.effect is StudyEffectState.ABSENT
    assert stopped.point.diagnostic is not None
    assert stopped.point.diagnostic.code != refusal.code

    progress = codec.decode_json(
        bytes(products["incumbent"]), documents.ProgressEventDocument
    )
    assert progress.incumbent is not None
    assert progress.incumbent.nodes == (1 << 53) + 1
    assert progress.incumbent.objective == 12.0
    assert progress.incumbent.dual_bound == 11.5
    assert progress.incumbent.gap == 0.04
    assert progress.incumbent.solution_id is None

    incumbent = codec.decode_json(
        bytes(products["qualified_incumbent"]), documents.Completion
    )
    assert incumbent.computation is None
    assert len(incumbent.solves) == 1
    assert incumbent.solves[0].termination is NativeTermination.NODE_LIMIT
    assert incumbent.solves[0].qualification is NativeQualification.GAP_QUALIFIED
    assert incumbent.solves[0].candidate_kind is NativeCandidateKind.FEASIBLE_POINT
    assert incumbent.solves[0].feasible is True
    assert incumbent.assessments == ()
    trajectory = codec.decode_json(
        bytes(products["event_ended_trajectory"]), documents.Completion
    )
    assert trajectory.computation is not None
    assert trajectory.computation.trajectory_termination is TrajectoryTermination.EVENT
    assert trajectory.computation.completed_time == 0.125
    assert trajectory.computation.completed_samples == 1
    assert trajectory.computation.termination is None
    assert trajectory.assessments == ()


@pytest.mark.unit
def test_generated_duration_retains_standard_fields_and_refuses_unknown_fields() -> (
    None
):
    overrides = codec.decode_json(
        b'{"time_limit":{"secs":7,"nanos":123}}', documents.InitializationOverrides
    )
    assert overrides.time_limit is not None
    assert overrides.time_limit.secs == 7
    assert overrides.time_limit.nanos == 123
    with pytest.raises(msgspec.ValidationError):
        codec.decode_json(
            b'{"time_limit":{"secs":7,"nanos":123,"seconds":8}}',
            documents.InitializationOverrides,
        )
    with pytest.raises(msgspec.ValidationError):
        codec.decode_json(
            b'{"time_limit":{"secs":-1,"nanos":0}}', documents.InitializationOverrides
        )
