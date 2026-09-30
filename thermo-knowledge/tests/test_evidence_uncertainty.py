# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Evidence with several typed uncertainty assessments, two-sided magnitudes, censored limits
with a side, a constant with an uncertainty and a column presented against a reference state:
written through the canonical writer, loaded into a database and checked by `tk verify`."""

from __future__ import annotations

import uuid
from collections.abc import Callable
from pathlib import Path

import psycopg
import pytest
from build_support import fingerprint, inputs_of, write_source
from mapping_support import carrier, origin, real_declaration, writer

from thermo_knowledge import config, identity
from thermo_knowledge.build import build_database
from thermo_knowledge.canonical.values import Quantity
from thermo_knowledge.canonical.writer import CanonicalWriter, ValidationError
from thermo_knowledge.declaration import Declaration
from thermo_knowledge.testing import TestDatabase
from thermo_knowledge.verify.checks import VERIFY_DIR, load_checks
from thermo_knowledge.verify.run import run_checks

MEASURED = origin("a.json#/0", "measured")


class Evidence:
    """One dataset with a temperature variable, a pressure constraint and a property column,
    written through a canonical writer."""

    def __init__(self, w: CanonicalWriter, decl: Declaration) -> None:
        self.w = w
        seen = {e.name: e.id for e in decl.entities if e.kind == "observable"}
        self.observables = seen
        self.dataset = w.kind(
            "dataset",
            {
                "carrier": identity.identifier("source", [carrier("src", "a.json").key]),
                "local_key": "d",
                "kind": "measured",
            },
            origins=[MEASURED],
        )
        self.temperature = self.column(1, "variable", "temperature")
        self.pressure = self.column(
            2,
            "constraint",
            "pressure",
            constant=Quantity(2.0, "bar"),
            constant_digits=3,
        )
        self.property = self.column(3, "property", "vapor_pressure")
        self.point = w.kind("data_point", {"dataset": self.dataset, "index": 1}, at="a.json#/0")

    def column(self, ordinal: int, role: str, observable: str, **extra: object) -> uuid.UUID:
        return self.w.kind(
            "dataset_column",
            {
                "dataset": self.dataset,
                "ordinal": ordinal,
                "role": role,
                "observable": self.observables[observable],
                **extra,
            },
            at="a.json#/0",
        )

    def assessment(
        self, column: uuid.UUID, ordinal: int, kind: str, **extra: object
    ) -> uuid.UUID:
        return self.w.kind(
            "uncertainty_assessment",
            {"column": column, "ordinal": ordinal, "kind": kind, **extra},
            at="a.json#/0",
        )

    def datum(
        self,
        column: uuid.UUID,
        state: str = "known",
        value: Quantity | None = None,
        *,
        point: uuid.UUID | None = None,
    ) -> uuid.UUID:
        return self.w.relation(
            "datum",
            {"point": point or self.point, "column": column},
            {"state": state, "value": value, "digits": 4},
            at="a.json#/1",
        )

    def uncertainty(self, assessment: uuid.UUID, **magnitudes: object) -> uuid.UUID:
        return self.w.relation(
            "datum_uncertainty",
            {"point": self.point, "assessment": assessment},
            magnitudes,
            at="a.json#/2",
        )


def loaded(tmp_path: Path, fill: Callable[[Evidence], None]) -> TestDatabase:
    """A database built from the evidence `fill` writes (the caller removes it)."""
    decl = real_declaration()

    def emit(w: CanonicalWriter) -> None:
        fill(Evidence(w, decl))

    write_source(tmp_path, "src", emit, decl=decl, declaration=fingerprint(decl))
    database = TestDatabase.create()
    try:
        build_database(database.url, decl, inputs_of(tmp_path))
    except BaseException:
        database.remove()
        raise
    return database


def violations(database: TestDatabase) -> dict[str, int]:
    checks, problems = load_checks(config.TREE_DIR / VERIFY_DIR)
    assert problems == []
    with psycopg.connect(database.url) as conn:
        results = run_checks(conn, checks)
    assert [r.error for r in results if r.error] == []
    return {r.check.target: r.violations for r in results if r.violations}


def test_a_value_with_a_standard_an_expanded_and_a_repeatability_uncertainty_loads_as_three_typed_rows(
    tmp_path: Path,
) -> None:
    def fill(e: Evidence) -> None:
        e.datum(e.property, value=Quantity(1.5, "bar"))
        standard = e.assessment(e.property, 1, "standard", evaluator="the experimenters")
        expanded = e.assessment(
            e.property,
            2,
            "expanded",
            coverage_factor=2.0,
            confidence_level=0.95,
            evaluator="the experimenters",
            method="propagation of the standard uncertainties",
        )
        repeatability = e.assessment(e.property, 3, "repeatability_single_unbiased")
        e.uncertainty(standard, minus=Quantity(0.01, "bar"), plus=Quantity(0.01, "bar"))
        e.uncertainty(expanded, minus=Quantity(0.02, "bar"), plus=Quantity(0.02, "bar"))
        e.uncertainty(repeatability, minus=Quantity(0.005, "bar"), plus=Quantity(0.005, "bar"))

    database = loaded(tmp_path, fill)
    try:
        with psycopg.connect(database.url) as conn:
            rows = conn.execute(
                "SELECT a.ordinal, a.kind::text, a.coverage_factor, a.confidence_level, a.evaluator, "
                "a.method, u.minus, u.plus, u.relative_minus, u.relative_plus "
                "FROM ev.datum_uncertainty u JOIN ev.uncertainty_assessment a ON a.id = u.assessment "
                "ORDER BY a.ordinal"
            ).fetchall()
            value = conn.execute("SELECT state::text, value, digits FROM ev.datum").fetchall()
        assert value == [("known", pytest.approx(1.5e5), 4)], "the value is in pascal"
        assert [(r[0], r[1]) for r in rows] == [
            (1, "standard"),
            (2, "expanded"),
            (3, "repeatability_single_unbiased"),
        ]
        assert [(r[6], r[7]) for r in rows] == [
            (pytest.approx(1e3), pytest.approx(1e3)),
            (pytest.approx(2e3), pytest.approx(2e3)),
            (pytest.approx(500.0), pytest.approx(500.0)),
        ], "each magnitude takes the unit of the column's observable"
        assert rows[1][2:6] == (2.0, 0.95, "the experimenters", "propagation of the standard uncertainties")
        assert rows[0][2:4] == (None, None) and rows[2][4:6] == (None, None)
        assert all(r[8] is None and r[9] is None for r in rows)
        assert violations(database) == {}
    finally:
        database.remove()


def test_an_uncertainty_may_be_asymmetric(tmp_path: Path) -> None:
    def fill(e: Evidence) -> None:
        e.datum(e.property, value=Quantity(1.5, "bar"))
        combined = e.assessment(e.property, 1, "combined_standard")
        e.uncertainty(combined, minus=Quantity(0.01, "bar"), plus=Quantity(0.03, "bar"))

    database = loaded(tmp_path, fill)
    try:
        with psycopg.connect(database.url) as conn:
            row = conn.execute("SELECT minus, plus FROM ev.datum_uncertainty").fetchone()
        assert row == (pytest.approx(1e3), pytest.approx(3e3))
        assert violations(database) == {}
    finally:
        database.remove()


def test_a_relative_uncertainty_states_relative_sides(tmp_path: Path) -> None:
    def fill(e: Evidence) -> None:
        e.datum(e.property, value=Quantity(1.5, "bar"))
        e.uncertainty(e.assessment(e.property, 1, "relative"), relative_minus=0.01, relative_plus=0.02)

    database = loaded(tmp_path, fill)
    try:
        assert violations(database) == {}
    finally:
        database.remove()


def test_a_censored_limit_has_a_side_and_a_value_and_a_missing_value_has_neither(
    tmp_path: Path,
) -> None:
    def fill(e: Evidence) -> None:
        for index, (state, value) in enumerate(
            (
                ("censored_below", Quantity(1.0, "bar")),
                ("censored_above", Quantity(3.0, "bar")),
                ("not_measured", None),
                ("known", Quantity(2.0, "bar")),
            ),
            start=2,
        ):
            point = e.w.kind("data_point", {"dataset": e.dataset, "index": index}, at="a.json#/0")
            e.datum(e.property, state, value, point=point)

    database = loaded(tmp_path, fill)
    try:
        with psycopg.connect(database.url) as conn:
            rows = conn.execute(
                "SELECT p.index, d.state::text, d.value FROM ev.datum d "
                "JOIN ev.data_point p ON p.id = d.point ORDER BY p.index"
            ).fetchall()
        assert rows == [
            (2, "censored_below", pytest.approx(1e5)),
            (3, "censored_above", pytest.approx(3e5)),
            (4, "not_measured", None),
            (5, "known", pytest.approx(2e5)),
        ]
    finally:
        database.remove()


def test_the_value_of_a_datum_is_present_exactly_when_the_state_carries_one(tmp_path: Path) -> None:
    w = writer(real_declaration())
    e = Evidence(w, real_declaration())
    for state in ("known", "censored_below", "censored_above"):
        with pytest.raises(ValidationError, match="value_matches_state"):
            e.datum(e.property, state, None)
    with pytest.raises(ValidationError, match="value_matches_state"):
        e.datum(e.property, "not_measured", Quantity(1.0, "bar"))
    with pytest.raises(ValidationError, match="'censored' is not a member of enum `datum_state`"):
        e.datum(e.property, "censored", Quantity(1.0, "bar"))


def test_a_constraint_column_has_an_uncertainty_on_its_constant(tmp_path: Path) -> None:
    def fill(e: Evidence) -> None:
        standard = e.assessment(e.pressure, 1, "standard", evaluator="the supplier")
        relative = e.assessment(e.pressure, 2, "relative")
        e.w.relation(
            "column_uncertainty",
            {"assessment": standard},
            {"minus": Quantity(0.01, "bar"), "plus": Quantity(0.01, "bar")},
            at="a.json#/2",
        )
        e.w.relation(
            "column_uncertainty",
            {"assessment": relative},
            {"relative_minus": 0.005, "relative_plus": 0.005},
            at="a.json#/2",
        )

    database = loaded(tmp_path, fill)
    try:
        with psycopg.connect(database.url) as conn:
            rows = conn.execute(
                "SELECT a.kind::text, u.minus, u.relative_minus FROM ev.column_uncertainty u "
                "JOIN ev.uncertainty_assessment a ON a.id = u.assessment ORDER BY a.ordinal"
            ).fetchall()
            column = conn.execute(
                "SELECT constant, constant_digits FROM ev.dataset_column WHERE role::text = 'constraint'"
            ).fetchone()
        assert rows == [("standard", pytest.approx(1e3), None), ("relative", None, 0.005)]
        assert column == (pytest.approx(2e5), 3)
        assert violations(database) == {}
    finally:
        database.remove()


def test_a_relative_magnitude_on_a_kind_that_is_not_relative_is_flagged(tmp_path: Path) -> None:
    def fill(e: Evidence) -> None:
        e.datum(e.property, value=Quantity(1.5, "bar"))
        e.uncertainty(
            e.assessment(e.property, 1, "standard"), relative_minus=0.01, relative_plus=0.01
        )
        e.uncertainty(
            e.assessment(e.property, 2, "relative"),
            minus=Quantity(0.01, "bar"),
            plus=Quantity(0.01, "bar"),
        )

    database = loaded(tmp_path, fill)
    try:
        assert violations(database) == {"datum_uncertainty.magnitude_matches_kind": 2}
    finally:
        database.remove()


def test_a_column_presented_against_a_reference_state_carries_it(tmp_path: Path) -> None:
    def fill(e: Evidence) -> None:
        liquid = e.w.kind(
            "dataset_phase",
            {"dataset": e.dataset, "ordinal": 1, "aggregation": next(
                x.id for x in real_declaration().entities if x.kind == "aggregation" and x.name == "liquid"
            )},
            at="a.json#/0",
        )
        e.column(
            4,
            "property",
            "vapor_pressure",
            presentation="difference_from_reference",
            reference_state_kind="reference_phase_fixed_tp",
            reference_temperature=Quantity(25.0, "degC"),
            reference_pressure=Quantity(1.0, "bar"),
            reference_phase=liquid,
        )

    database = loaded(tmp_path, fill)
    try:
        with psycopg.connect(database.url) as conn:
            row = conn.execute(
                "SELECT presentation::text, reference_state_kind::text, reference_temperature, "
                "reference_pressure, reference_phase IS NOT NULL FROM ev.dataset_column "
                "WHERE ordinal = 4"
            ).fetchone()
            direct = conn.execute(
                "SELECT DISTINCT presentation::text FROM ev.dataset_column WHERE ordinal < 4"
            ).fetchall()
        assert row == (
            "difference_from_reference",
            "reference_phase_fixed_tp",
            pytest.approx(298.15),
            pytest.approx(1e5),
            True,
        )
        assert direct == [("direct",)], "a column states direct unless it says otherwise"
        assert violations(database) == {}
    finally:
        database.remove()


def test_a_presentation_against_a_reference_state_without_its_kind_is_flagged(
    tmp_path: Path,
) -> None:
    def fill(e: Evidence) -> None:
        e.column(4, "property", "vapor_pressure", presentation="ratio_to_reference")
        # an interval presentation has no reference state to state
        e.column(5, "property", "vapor_pressure", presentation="difference_between_temperatures")

    database = loaded(tmp_path, fill)
    try:
        assert violations(database) == {"dataset_column.reference_state_kind_when_relative": 1}
    finally:
        database.remove()
