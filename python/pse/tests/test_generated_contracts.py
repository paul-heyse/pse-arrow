# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Actual generated contract boundaries, including nested values and metadata."""

from typing import get_type_hints

import attrs
import cattrs
import pyarrow as pa
import pytest

from pse.codec import converter, structure_rows
from pse.contracts import values as contract_values
from pse.contracts.authored import AuthoredPackagesRow
from pse.contracts.compiled import CompiledSolvePlansRow
from pse.contracts.enums import EquationFamily, EquationRole, SolvePlanClass
from pse.contracts.extension_types import PseEnum, PseOrdinalRef
from pse.contracts.reference import ReferenceUnitsRow
from pse.contracts.values import FIELD_NAME_METADATA, ContentHash, SemanticId


@pytest.mark.unit
def test_validator_annotations_evaluate_on_supported_python_versions() -> None:
    # Python 3.14 defers annotations that Python 3.13 evaluates during import.
    # Force evaluation here so the host also catches parity import regressions.
    for validator in (
        contract_values.exact_type,
        contract_values.integer_range,
        contract_values.exact_type(bool),
        contract_values.integer_range(0, 7),
        contract_values.finite_float,
        contract_values.utc_timestamp,
    ):
        assert get_type_hints(validator)


def package_row() -> dict[str, object]:
    return {
        "package_id": "01" * 16,
        "name": "example",
        "version": "1.0.0",
        "kind": "model",
        "id_policy": "explicit",
        "dependencies": [{"package_id": "02" * 16, "version_req": "=1.0.0"}],
        "content_hash": "blake3:" + "03" * 32,
        "doc": "nested fixture",
    }


@pytest.mark.unit
def test_nested_generated_row_is_typed_and_frozen() -> None:
    row = structure_rows([package_row()], AuthoredPackagesRow)[0]
    assert row.package_id == SemanticId(bytes([1] * 16))
    assert row.dependencies[0].package_id == SemanticId(bytes([2] * 16))
    assert row.content_hash == ContentHash(bytes([3] * 32))
    assert isinstance(row.dependencies, tuple)
    with pytest.raises(attrs.exceptions.FrozenInstanceError):
        # pyrefly: ignore[read-only] -- refusal is under test
        row.name = "changed"


@pytest.mark.unit
def test_nested_unknown_column_is_not_ignored() -> None:
    document = package_row()
    document["dependencies"] = [
        {"package_id": "02" * 16, "version_req": "=1.0.0", "unchecked": True},
    ]
    with pytest.raises(cattrs.BaseValidationError):
        structure_rows([document], AuthoredPackagesRow)


@pytest.mark.unit
@pytest.mark.parametrize("value", [True, "1", 1.5])
def test_integer_codec_does_not_coerce_other_value_kinds(value: object) -> None:
    with pytest.raises(ValueError):
        converter().structure(value, int)


@pytest.mark.unit
def test_fixed_dimension_width_is_enforced_without_a_fingerprint() -> None:
    values = {
        "unit_id": "01" * 16,
        "symbol": "x",
        "name": "x",
        "dimension": [{"num": 0, "den": 1}] * 7,
        "scale_to_canonical": 1.0,
        "offset_to_canonical": 0.0,
        "is_affine": False,
        "reference_state_id": None,
        "system": "SI",
        "doc": "",
    }
    with pytest.raises(cattrs.BaseValidationError):
        structure_rows([values], ReferenceUnitsRow)


@pytest.mark.unit
def test_parameterized_metadata_preserves_actual_binding() -> None:
    binding = "01" * 16
    enum = PseEnum(binding)
    assert (
        enum.__arrow_ext_serialize__() == (f'{{"v":1,"enum_id":"{binding}"}}').encode()
    )
    assert (
        PseEnum.__arrow_ext_deserialize__(
            pa.dictionary(pa.int32(), pa.utf8()),
            enum.__arrow_ext_serialize__(),
        ).binding_id
        == binding
    )
    ordinal = PseOrdinalRef(binding)
    assert (
        PseOrdinalRef.__arrow_ext_deserialize__(
            pa.uint64(),
            ordinal.__arrow_ext_serialize__(),
        ).binding_id
        == binding
    )
    with pytest.raises(ValueError):
        PseEnum.__arrow_ext_deserialize__(
            pa.dictionary(pa.int32(), pa.utf8()),
            (f'{{"v":1,"enum_id":"{binding}","enum_id":"{binding}"}}').encode(),
        )
    with pytest.raises(ValueError):
        PseOrdinalRef.__arrow_ext_deserialize__(pa.uint64(), b'{"v":1}')


@pytest.mark.unit
def test_equation_taxonomy_has_conservative_unclassified_and_producer_roles() -> None:
    assert EquationFamily.UNCLASSIFIED.value == "UNCLASSIFIED"
    assert EquationFamily.GENERAL_NONLINEAR.value == "GENERAL_NONLINEAR"
    assert {member.value for member in EquationRole} == {
        "HARD_FEASIBILITY",
        "DEFINITION",
        "LINKING",
        "DOMAIN_GUARD",
        "REPORTING",
        "APPROXIMATION",
    }


@pytest.mark.unit
def test_keyword_attribute_round_trips_exact_declared_wire_name() -> None:
    values: dict[str, object] = {
        "plan_id": bytes([1] * 16),
        "problem_id": bytes([2] * 16),
        "class": "SQUARE_NLE",
        "justification": "square smooth equations with no objective",
        "modifiers": ["continuation on failure"],
    }
    codec = converter()
    row = codec.structure(values, CompiledSolvePlansRow)
    assert row.class_ is SolvePlanClass.SQUARE_NLE
    encoded = codec.unstructure(row)
    assert encoded["class"] == values["class"]
    assert "class_" not in encoded
    assert codec.structure(encoded, CompiledSolvePlansRow) == row
    for unknown in ("class_", "undeclared"):
        with pytest.raises(cattrs.BaseValidationError):
            codec.structure({**values, unknown: "NLP_LOCAL"}, CompiledSolvePlansRow)
    alias_only = {key: value for key, value in values.items() if key != "class"}
    alias_only["class_"] = "SQUARE_NLE"
    with pytest.raises(cattrs.BaseValidationError):
        codec.structure(alias_only, CompiledSolvePlansRow)


@attrs.frozen(kw_only=True)
class _KeywordCollision:
    class__: str = attrs.field(metadata={FIELD_NAME_METADATA: "class"})
    class_: str


@pytest.mark.unit
def test_declared_keyword_and_existing_suffix_remain_distinct() -> None:
    codec = converter()
    values = {"class": "keyword value", "class_": "literal suffixed field"}
    row = codec.structure(values, _KeywordCollision)
    assert row.class__ == values["class"]
    assert row.class_ == values["class_"]
    assert codec.unstructure(row) == values
    with pytest.raises(cattrs.BaseValidationError):
        codec.structure({**values, "class__": "not a wire field"}, _KeywordCollision)


class _DeclaredText(str):
    """A declared text constructor must remain active at the codec boundary."""


@pytest.mark.unit
def test_text_hook_preserves_actual_constructor_and_refuses_invalid_enum() -> None:
    codec = converter()
    assert type(codec.structure("plain", str)) is str
    text = codec.structure("declared", _DeclaredText)
    assert type(text) is _DeclaredText
    assert text == "declared"
    assert codec.structure("SQUARE_NLE", SolvePlanClass) is SolvePlanClass.SQUARE_NLE
    with pytest.raises(ValueError):
        codec.structure("undeclared", SolvePlanClass)
    with pytest.raises(ValueError):
        codec.structure(123, _DeclaredText)
