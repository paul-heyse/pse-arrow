# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""The import-time contract-type lint (blueprint §21.5, §24.1)."""

from typing import Any

import attrs
import pytest

from pse import governance
from pse.governance import ContractTypeError


@attrs.frozen
class _Good:
    name: str
    ordinals: tuple[int, ...]
    labels: dict[str, str]


@attrs.frozen
class _AnyField:
    payload: Any


@attrs.frozen
class _BareDict:
    payload: dict


@attrs.frozen
class _BareList:
    payload: list


@pytest.mark.unit
def test_specific_types_pass() -> None:
    governance.check_class(_Good)


@pytest.mark.unit
@pytest.mark.parametrize(
    ("cls", "reason"),
    [
        (_AnyField, "typing.Any"),
        (_BareDict, "a bare dict"),
        (_BareList, "a bare list"),
    ],
)
def test_unconstrained_types_fail(cls: type, reason: str) -> None:
    with pytest.raises(ContractTypeError) as caught:
        governance.check_class(cls)
    assert caught.value.reason == reason
    assert "payload" in str(caught.value)


@pytest.mark.unit
def test_generated_contracts_pass_the_lint() -> None:
    governance.check()
