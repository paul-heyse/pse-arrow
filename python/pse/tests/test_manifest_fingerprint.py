# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""The full generated manifest is a strict document, independent of hash claims."""

import msgspec
import msgspec.json
import pytest

from pse.codec import Manifest, decode_json, encode_json
from pse.contracts import REGISTRY_FINGERPRINT


def manifest_document() -> dict[str, object]:
    return {
        "manifest_version": "pse.manifest.v2",
        "snapshot_kind": "model",
        "snapshot_id": "blake3:" + "00" * 32,
        "membership_profile": "pse.snapshot.v2",
        "created_at": "2026-09-14T12:00:00Z",
        "schema_registry_fingerprint": REGISTRY_FINGERPRINT,
        "relations": [],
        "packages": [],
        "compiler": {"version": "0.0.1", "passes": []},
        "engine_profile": None,
        "numerical_policy": None,
        "toolchain": {
            "lockfile_hash": "blake3:" + "01" * 32,
            "canonicalization": "pse.canon.v2",
        },
        "kernels": [],
        "semantic_parents": [],
        "evidence": [],
    }


@pytest.mark.unit
def test_manifest_round_trips() -> None:
    manifest = decode_json(msgspec.json.encode(manifest_document()), Manifest)
    assert decode_json(encode_json(manifest), Manifest) == manifest


@pytest.mark.unit
def test_unknown_key_is_refused() -> None:
    document = manifest_document()
    document["plan_fingerprint"] = "nope"
    with pytest.raises(msgspec.ValidationError, match="plan_fingerprint"):
        decode_json(msgspec.json.encode(document), Manifest)


@pytest.mark.unit
def test_missing_key_is_refused() -> None:
    document = manifest_document()
    del document["toolchain"]
    with pytest.raises(msgspec.ValidationError, match="toolchain"):
        decode_json(msgspec.json.encode(document), Manifest)


@pytest.mark.unit
@pytest.mark.parametrize("bad", [1, "unknown", "blake3:" + "ff" * 31])
def test_wrong_hash_wire_type_or_width_is_refused(bad: object) -> None:
    document = manifest_document()
    document["snapshot_id"] = bad
    with pytest.raises(msgspec.ValidationError, match="snapshot_id"):
        decode_json(msgspec.json.encode(document), Manifest)


@pytest.mark.unit
def test_matching_registry_hash_does_not_admit_unknown_nested_fields() -> None:
    document = manifest_document()
    document["compiler"] = {"version": "0.0.1", "passes": [], "assume_valid": True}
    with pytest.raises(msgspec.ValidationError, match="assume_valid"):
        decode_json(msgspec.json.encode(document), Manifest)


@pytest.mark.unit
def test_manifest_is_frozen() -> None:
    manifest = decode_json(msgspec.json.encode(manifest_document()), Manifest)
    with pytest.raises(AttributeError):
        # pyrefly: ignore[read-only]  -- refusing mutation is the behavior under test
        manifest.snapshot_kind = "case"


@pytest.mark.unit
@pytest.mark.parametrize(
    "value",
    [
        "2026-02-30T12:00:00Z",
        "0000-01-01T00:00:00Z",
        "2026-09-14T12:00:60Z",
        "2026-09-14T12:00:00+00:00",
        "2026-09-14T12:00:00.1234567890Z",
        "yesterday",
    ],
)
def test_manifest_timestamp_requires_actual_utc_calendar_value(value: str) -> None:
    document = manifest_document()
    document["created_at"] = value
    with pytest.raises(msgspec.ValidationError):
        decode_json(msgspec.json.encode(document), Manifest)


@pytest.mark.unit
@pytest.mark.parametrize(
    "value", ["2024-02-29T23:59:59Z", "2026-09-14T12:00:00.123456789Z"]
)
def test_manifest_preserves_valid_nanosecond_timestamp_text(value: str) -> None:
    document = manifest_document()
    document["created_at"] = value
    decoded = decode_json(msgspec.json.encode(document), Manifest)
    assert decoded.created_at == value
