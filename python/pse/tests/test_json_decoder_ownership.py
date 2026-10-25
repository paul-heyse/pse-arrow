# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Ordinary document decoding retains strict admission and independent thread state."""

from concurrent.futures import ThreadPoolExecutor
from threading import Barrier

import msgspec
import pytest

from pse import codec
from pse.contracts import documents


@pytest.mark.unit
def test_thread_owned_decoders_preserve_values_and_refusal_isolation() -> None:
    barrier = Barrier(2)

    def read(key: str) -> tuple[int, str]:
        raw = ('{"document_version":1,"kind":"run","key":"' + key + '"}').encode()
        first = codec.decode_json(raw, documents.HistoryOwner)
        decoder = codec._json_decoder(documents.HistoryOwner)[2]
        barrier.wait(timeout=10)
        with pytest.raises(msgspec.ValidationError):
            codec.decode_json(
                b'{"document_version":1,"kind":"run","key":"x","extra":1}',
                documents.HistoryOwner,
            )
        with pytest.raises(msgspec.ValidationError, match="version"):
            codec.decode_json(b'{"kind":"run","key":"x"}', documents.HistoryOwner)
        again = codec.decode_json(raw, documents.HistoryOwner)
        assert first == again
        assert codec._json_decoder(documents.HistoryOwner)[2] is decoder
        return id(decoder), again.key

    with ThreadPoolExecutor(max_workers=2) as workers:
        first, second = [
            future.result(timeout=15)
            for future in (
                workers.submit(read, "first"),
                workers.submit(read, "second"),
            )
        ]
    assert first[0] != second[0]
    assert (first[1], second[1]) == ("first", "second")
