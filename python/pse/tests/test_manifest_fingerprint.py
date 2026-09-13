# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Manifests are strict documents (blueprint §21.5, §24.1)."""

import msgspec
import pytest

from pse.codec import Manifest, decode_json, encode_json

_MANIFEST = Manifest(
    schema_version=1,
    artifact_id="0f8c1c2a",
    content_hash="b3" * 32,
    lockfile_hash="b3" * 32,
)


@pytest.mark.unit
def test_manifest_round_trips() -> None:
    assert decode_json(encode_json(_MANIFEST), Manifest) == _MANIFEST


@pytest.mark.unit
def test_unknown_key_is_refused() -> None:
    document = b'{"schema_version":1,"artifact_id":"0f8c1c2a","content_hash":"b3",'
    document += b'"lockfile_hash":"b3","plan_fingerprint":"nope"}'
    with pytest.raises(msgspec.ValidationError, match="plan_fingerprint"):
        decode_json(document, Manifest)


@pytest.mark.unit
def test_missing_key_is_refused() -> None:
    document = b'{"schema_version":1,"artifact_id":"a","content_hash":"b"}'
    with pytest.raises(msgspec.ValidationError, match="lockfile_hash"):
        decode_json(document, Manifest)


@pytest.mark.unit
def test_wrong_type_is_refused() -> None:
    document = b'{"schema_version":"1","artifact_id":"a","content_hash":"b",'
    document += b'"lockfile_hash":"c"}'
    with pytest.raises(msgspec.ValidationError, match="schema_version"):
        decode_json(document, Manifest)


@pytest.mark.unit
def test_manifest_is_frozen() -> None:
    with pytest.raises(AttributeError):
        # pyrefly: ignore[read-only]  -- refusing this is what is under test
        _MANIFEST.schema_version = 2
