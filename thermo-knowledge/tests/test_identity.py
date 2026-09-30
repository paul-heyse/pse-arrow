# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Deterministic identifiers: fixed expected values pin the encoding rules of section 5."""

from __future__ import annotations

import hashlib
import unicodedata
import uuid
from datetime import UTC, date, datetime, timedelta, timezone

import pytest

from thermo_knowledge import identity

REFERENCE = uuid.UUID("12345678-1234-5678-1234-567812345678")
OTHER = uuid.UUID("87654321-4321-8765-4321-876543218765")


def test_root_namespace_is_declared_once() -> None:
    assert identity.ROOT_NAMESPACE == uuid.UUID("5ad3ac16-681d-4c3c-878f-2408aa1bef6f")
    assert identity.namespace("species") == uuid.UUID("6cc5192d-8611-52eb-aa5f-cd3390b197db")
    assert identity.namespace("species") == uuid.uuid5(identity.ROOT_NAMESPACE, "species")
    assert identity.namespace("species") != identity.namespace("element")


def test_text_key() -> None:
    assert identity.identifier("species", ["H2O"]) == uuid.UUID(
        "056a4853-4883-5b9e-9754-857bb5a13c16"
    )


def test_text_key_matches_an_independent_computation() -> None:
    # version-5 UUID: SHA-1 of the namespace bytes and the name, truncated and stamped.
    namespace = uuid.uuid5(uuid.UUID("5ad3ac16-681d-4c3c-878f-2408aa1bef6f"), "species")
    digest = hashlib.sha1(namespace.bytes + b'["H2O"]').digest()
    assert identity.identifier("species", ["H2O"]) == uuid.UUID(bytes=digest[:16], version=5)


def test_integer_key() -> None:
    assert identity.identifier("rank_weight", [7]) == uuid.UUID(
        "647a1b67-5278-5727-891d-5be0bd178c3c"
    )
    assert identity.canonical_encoding([7]) == "[7]"
    assert identity.identifier("rank_weight", [7]) != identity.identifier("rank_weight", ["7"])


def test_reference_key() -> None:
    assert identity.canonical_encoding([REFERENCE]) == '["12345678-1234-5678-1234-567812345678"]'
    assert identity.identifier("species_form", [REFERENCE]) != identity.identifier(
        "species_form", [OTHER]
    )


def test_multi_key_tuple() -> None:
    assert identity.identifier("formula", [REFERENCE, OTHER]) == uuid.UUID(
        "a8bdc756-7d38-56d6-8ecf-4792cebb45e0"
    )
    assert identity.identifier("species_form", [REFERENCE, "gas", 3, True]) == uuid.UUID(
        "0ee09855-37b1-5060-892b-634bc8986a82"
    )
    # order matters
    assert identity.identifier("formula", [REFERENCE, OTHER]) != identity.identifier(
        "formula", [OTHER, REFERENCE]
    )


def test_every_encoding_rule() -> None:
    encoded = identity.canonical_encoding(
        [
            REFERENCE,
            "gas",
            3,
            True,
            False,
            date(2020, 1, 2),
            datetime(2020, 1, 2, 3, 4, 5, tzinfo=UTC),
            bytes(range(32)),
            "é",
        ]
    )
    assert encoded == (
        '["12345678-1234-5678-1234-567812345678","gas",3,true,false,"2020-01-02",'
        '"2020-01-02T03:04:05.000000Z",'
        '"000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f","é"]'
    )


def test_timestamps_are_encoded_in_utc() -> None:
    plus_two = timezone(timedelta(hours=2))
    assert identity.canonical_encoding(
        [datetime(2020, 1, 2, 5, 4, 5, tzinfo=plus_two)]
    ) == identity.canonical_encoding([datetime(2020, 1, 2, 3, 4, 5, tzinfo=UTC)])
    with pytest.raises(identity.IdentityError, match="time zone"):
        identity.canonical_encoding([datetime(2020, 1, 2, 3, 4, 5)])


def test_nfc_normalisation_makes_differently_composed_strings_equal() -> None:
    composed, decomposed = "é", "é"
    assert composed != decomposed
    assert unicodedata.normalize("NFC", decomposed) == composed
    assert identity.canonical_encoding([composed]) == identity.canonical_encoding([decomposed])
    assert identity.identifier("species", [composed]) == identity.identifier(
        "species", [decomposed]
    )
    assert identity.namespace(composed) == identity.namespace(decomposed)


def test_text_is_not_case_folded_or_trimmed() -> None:
    assert identity.identifier("species", ["H2O"]) != identity.identifier("species", ["h2o"])
    assert identity.identifier("species", ["H2O"]) != identity.identifier("species", [" H2O"])


def test_floating_point_and_other_types_are_refused() -> None:
    with pytest.raises(identity.IdentityError, match="floating"):
        identity.identifier("species", [1.5])  # type: ignore[list-item]
    with pytest.raises(identity.IdentityError):
        identity.identifier("species", [None])  # type: ignore[list-item]
    with pytest.raises(identity.IdentityError):
        identity.identifier("species", [["a"]])  # type: ignore[list-item]


def test_namespaces_separate_kinds() -> None:
    assert identity.identifier("species", ["x"]) != identity.identifier("element", ["x"])


def test_meta_identifier_is_derived_from_the_qualified_name() -> None:
    assert identity.meta_identifier("slot_group", "antoine.pure") == uuid.UUID(
        "2460c2b2-eead-54d0-aa02-b24495a4acc3"
    )
    assert identity.meta_identifier("slot_group", "antoine.pure") == identity.identifier(
        "meta:slot_group", ["antoine.pure"]
    )
    assert identity.meta_identifier("slot", "antoine.pure") != identity.meta_identifier(
        "slot_group", "antoine.pure"
    )
