# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Generated defaults and boundary conversion share the leaf identity contract."""

from typing import Annotated

import msgspec
import pytest

from pse import codec
from pse.contracts.values import ContentHash, SemanticId, _convert_document


class _IdentityValue(msgspec.Struct, frozen=True, forbid_unknown_fields=True):
    count: Annotated[int, msgspec.Meta(ge=1)]
    identity: SemanticId
    digest: ContentHash


_VALUE: dict[str, object] = {
    "count": 2,
    "identity": "11" * 16,
    "digest": "blake3:" + "22" * 32,
}


class _DefaultDocument(msgspec.Struct, frozen=True, forbid_unknown_fields=True):
    value: _IdentityValue = msgspec.field(
        default_factory=lambda: _convert_document(_VALUE, _IdentityValue)
    )


@pytest.mark.unit
def test_document_defaults_and_codec_share_typed_identity_values() -> None:
    expected = _IdentityValue(
        count=2,
        identity=SemanticId(bytes.fromhex("11" * 16)),
        digest=ContentHash(bytes.fromhex("22" * 32)),
    )
    defaulted = _DefaultDocument()
    assert defaulted.value == expected
    assert codec.convert_document({"value": _VALUE}, _DefaultDocument) == defaulted
    assert (
        codec.decode_json(codec.encode_json(defaulted), _DefaultDocument) == defaulted
    )
    assert codec.document_builtins(expected) == _VALUE


@pytest.mark.unit
@pytest.mark.parametrize(
    "change",
    [
        {"count": 0},
        {"count": True},
        {"identity": "invalid"},
        {"identity": b"\x11" * 16},
        {"digest": "invalid"},
        {"extra": 1},
    ],
)
def test_shared_document_value_conversion_preserves_refusals(
    change: dict[str, object],
) -> None:
    value = _VALUE | change
    with pytest.raises(msgspec.ValidationError):
        _convert_document(value, _IdentityValue)
    with pytest.raises(msgspec.ValidationError):
        codec.convert_document(value, _IdentityValue)
