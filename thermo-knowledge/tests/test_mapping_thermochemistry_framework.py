# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The mapping framework's support for the thermochemical sources: the Hill formula, list elements
of a column, partitions that test a joined table or exclude values and override field rules,
decodings, structure checks and text values that carry their unit."""

from __future__ import annotations

import pytest

from mapping_support import real_declaration
from thermo_knowledge.mapping import spec as spec_module
from thermo_knowledge.mapping.formula import hill
from thermo_knowledge.mapping.spec import (
    DecodeRule,
    Decoding,
    FieldRule,
    Partition,
    TableRule,
    element_of,
    validate,
)
from thermo_knowledge.mapping.staged import MappingError, SourceRow, _matches

# -- the Hill formula ------------------------------------------------------------------------


@pytest.mark.parametrize(
    ("counts", "formula"),
    [
        ({"H": 2, "O": 1}, "H2O"),
        ({"O": 1, "H": 4, "C": 1}, "CH4O"),
        ({"C": 2, "O": 1, "H": 6}, "C2H6O"),
        ({"Cl": 1, "H": 1}, "ClH"),
        ({"Na": 1, "Cl": 1}, "ClNa"),
        ({"O": 2, "Si": 1}, "O2Si"),
        ({"C": 0.98, "Nb": 1}, "C0.98Nb"),
        ({"C": 1, "Al": 2}, "CAl2"),
        ({"H": 1}, "H"),
        ({}, ""),
        ({"H": 0, "O": 2}, "O2"),
    ],
)
def test_the_hill_formula_orders_carbon_and_hydrogen_first_only_with_carbon(
    counts: dict[str, float], formula: str
) -> None:
    assert hill(counts) == formula


def test_a_count_of_one_is_left_out_and_a_negative_count_is_refused() -> None:
    assert hill({"H": 2.0, "O": 1.0}) == "H2O"
    with pytest.raises(ValueError, match="negative"):
        hill({"H": -1})


# -- list elements, partitions and field rules --------------------------------------------------


def test_a_column_key_with_an_index_names_an_element_of_the_column() -> None:
    assert element_of("coefficients[3]") == ("coefficients", 3)
    assert element_of("coefficients") == ("coefficients", None)
    assert element_of("derived:formula") == ("derived:formula", None)


def test_an_element_of_a_list_column_is_absent_beyond_the_list_and_refused_on_a_scalar() -> None:
    row = SourceRow("t", "a", "a#1", {"coefficients": [1.0, 2.0], "x": 3, "none": None})
    assert row["coefficients[1]"] == 2.0
    assert row["coefficients[2]"] is None
    assert row["none[0]"] is None
    with pytest.raises(MappingError, match="holds no list"):
        row["x[0]"]


def test_a_partition_condition_can_exclude_values_and_a_missing_joined_column_never_matches() -> (
    None
):
    assert _matches({"element": {"not": "E"}}, {"element": "H"})
    assert not _matches({"element": {"not": "E"}}, {"element": "E"})
    assert not _matches({"element": {"not": ["E", "D"]}}, {"element": "D"})
    assert not _matches({"thermo.model": "NASA7"}, {"element": "H"})
    assert _matches({"thermo.model": ["NASA7", "NASA9"]}, {"thermo.model": "NASA9"})


def test_a_partition_rule_replaces_the_tables_rules_of_that_column_and_its_elements() -> None:
    structure = FieldRule(role="structure", reason="x")
    table = TableRule(
        disposition="deferred",
        reason="r",
        wave=1,
        fields={
            "a": structure,
            "coefficients[0]": structure,
            "coefficients[1]": structure,
            "b": structure,
        },
        partitions=[
            Partition(
                name="p",
                where={"b": 1},
                disposition="mapped",
                origin_role="fitted",
                fields={"coefficients": FieldRule(disposition="out_of_scope", reason="y")},
            )
        ],
    )
    assert set(table.fields_of(None)) == {"a", "coefficients[0]", "coefficients[1]", "b"}
    assert set(table.fields_of("p")) == {"a", "b", "coefficients"}


# -- the declaration checks the new rules --------------------------------------------------------


def _problems(edit) -> list[str]:  # noqa: ANN001
    import pyarrow as pa

    decl = real_declaration()
    schema = pa.schema(
        [
            pa.field("_artifact", pa.string()),
            pa.field("_locator", pa.string()),
            pa.field("phase_flag", pa.int64()),
            pa.field("values", pa.list_(pa.float64())),
        ]
    )
    base = spec_module.MappingSpec(
        source="s",
        doc="d",
        tables={
            "t": TableRule(
                disposition="mapped",
                origin_role="published",
                fields={
                    "phase_flag": FieldRule(role="structure", reason="a"),
                    "values[0]": FieldRule(
                        target="nasa7.pure.piece.a1",
                        unit="dimensionless",
                        precision="exact",
                        loss="assumed",
                    ),
                },
            )
        },
    )
    return validate(edit(base), decl, {"t": schema}, source="s")


def test_a_column_is_covered_by_the_rules_of_its_elements() -> None:
    assert _problems(lambda spec: spec) == []


def test_a_list_column_ruled_by_elements_has_no_rule_of_its_own() -> None:
    def edit(spec):  # noqa: ANN001, ANN202
        spec.tables["t"].fields["values"] = FieldRule(role="structure", reason="a")
        return spec

    assert any("ruled by its elements" in problem for problem in _problems(edit))


def test_an_element_rule_needs_a_list_column() -> None:
    def edit(spec):  # noqa: ANN001, ANN202
        spec.tables["t"].fields["phase_flag[0]"] = FieldRule(role="structure", reason="a")
        return spec

    assert any("holds no list" in problem for problem in _problems(edit))


def test_a_decoding_names_an_entity_the_target_declares() -> None:
    def edit(spec):  # noqa: ANN001, ANN202
        spec.decodings["d"] = Decoding(
            doc="d", rules=[DecodeRule(where={"phase_flag": 0}, to="plasma", reason="r")]
        )
        spec.tables["t"].fields["phase_flag"] = FieldRule(
            target="source_entity.aggregation", decode="d", precision="close"
        )
        return spec

    assert any(
        "`plasma`, which is not a declared entity of kind `aggregation`" in p
        for p in _problems(edit)
    )


def test_a_default_is_an_assumption_the_rule_states() -> None:
    def edit(spec):  # noqa: ANN001, ANN202
        spec.tables["t"].fields["values[0]"] = FieldRule(
            target="nasa7.pure.piece.a1",
            unit="dimensionless",
            precision="exact",
            loss="x",
            default=0.0,
        )
        spec.tables["t"].fields["values[1]"] = FieldRule(
            target="nasa7.pure.piece.a2", unit="1/K", precision="exact", default=0.0, loss=None
        )
        return spec

    assert any("a `default` is an assumption" in problem for problem in _problems(edit))


def test_holds_belongs_to_a_column_the_mapping_does_not_map() -> None:
    def edit(spec):  # noqa: ANN001, ANN202
        spec.tables["t"].fields["values[1]"] = FieldRule(
            target="nasa7.pure.piece.a2", unit="1/K", precision="exact", loss="x", holds=True
        )
        return spec

    assert any("`holds` belongs to a column the mapping does not map" in p for p in _problems(edit))
