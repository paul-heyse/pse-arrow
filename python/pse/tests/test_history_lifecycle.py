# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Typed history adapter controls; canonical ownership is tested on its native store."""

from typing import TYPE_CHECKING, cast

import msgspec
import pytest

import pse
from pse import codec
from pse.contracts.documents import (
    HistoryObligation,
    HistoryOwner,
    HistoryReclamationPage,
)

if TYPE_CHECKING:
    from pse._build import _NativeRuntime


class HistoryBoundary:
    """Observe document transport without pretending to settle canonical effects."""

    def __init__(self) -> None:
        self.calls: list[tuple[str, bytes]] = []

    def retain_history_obligation(self, document: bytes) -> None:
        self.calls.append(("retain", document))

    def release_history_obligation(self, document: bytes) -> None:
        self.calls.append(("release_obligation", document))

    def release_history(self, document: bytes) -> None:
        self.calls.append(("release_history", document))

    def reclaim_history_page(self, document: bytes) -> HistoryReclamationPage:
        self.calls.append(("collect", document))
        return HistoryReclamationPage(complete=False, removed=64, compacted=0)


@pytest.mark.unit
@pytest.mark.parametrize("kind", ["run", "study"])
def test_history_adapters_keep_explicit_owner_and_obligation_documents(
    kind: str,
) -> None:
    owner_body = b'{"kind":"' + kind.encode() + b'","key":"original-owner"}'
    owner = codec.decode_json(b'{"document_version":1,' + owner_body[1:], HistoryOwner)
    obligation = codec.decode_json(
        b'{"document_version":1,"key":"audit-1","owner":'
        + owner_body
        + b',"purpose":"audit","attribution":"scientific investigation"}',
        HistoryObligation,
    )
    boundary = HistoryBoundary()
    runtime = object.__new__(pse.Runtime)
    object.__setattr__(runtime, "_handle", cast("_NativeRuntime", boundary))
    runtime.retain_history_obligation(obligation)
    runtime.release_history_obligation(obligation)
    runtime.release_history(owner)
    page = runtime.reclaim_history_page(owner)
    assert page == HistoryReclamationPage(complete=False, removed=64, compacted=0)
    assert boundary.calls == [
        ("retain", codec.encode_json(obligation)),
        ("release_obligation", codec.encode_json(obligation)),
        ("release_history", codec.encode_json(owner)),
        ("collect", codec.encode_json(owner)),
    ]


@pytest.mark.unit
@pytest.mark.parametrize(
    "document",
    [
        b'{"document_version":1,"kind":"run","key":"owner","extra":true}',
        b'{"document_version":1,"kind":"other","key":"owner"}',
    ],
)
def test_generated_history_owner_refuses_unknown_fields_and_variants(
    document: bytes,
) -> None:
    with pytest.raises(msgspec.ValidationError):
        codec.decode_json(document, HistoryOwner)
