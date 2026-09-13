# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""The codec refuses unknown keys (blueprint §21.5, §24.1)."""

import attrs
import cattrs
import pytest

from pse.codec import converter, render_errors, structure_rows


@attrs.frozen
class _Row:
    name: str
    ordinal: int


@pytest.mark.unit
def test_known_keys_structure() -> None:
    assert structure_rows([{"name": "a", "ordinal": 1}], _Row) == [_Row("a", 1)]


@pytest.mark.unit
def test_extra_key_is_refused() -> None:
    with pytest.raises(cattrs.BaseValidationError):
        structure_rows([{"name": "a", "ordinal": 1, "renamed": 2}], _Row)


@pytest.mark.unit
def test_missing_key_is_refused() -> None:
    with pytest.raises(cattrs.BaseValidationError):
        structure_rows([{"name": "a"}], _Row)


@pytest.mark.unit
def test_errors_render_one_finding_per_field() -> None:
    with pytest.raises(cattrs.BaseValidationError) as caught:
        converter().structure({"name": "a", "ordinal": 1, "renamed": 2}, _Row)
    findings = render_errors(caught.value)
    assert findings, "a validation error must render at least one finding"
    assert any("renamed" in finding for finding in findings), findings
