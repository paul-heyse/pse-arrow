# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Isolated native boundary admission; no publication, compiler or store fixture."""

from pathlib import Path
from typing import cast

import pytest

import pse
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
    assert report.code is not None
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
