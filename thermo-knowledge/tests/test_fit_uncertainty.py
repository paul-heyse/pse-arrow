# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""A fit's uncertainty is held as typed standard uncertainties of its free parameters and the
correlations between them; there is no covariance array whose unit nothing states."""

from __future__ import annotations

import uuid
from collections.abc import Iterator

import psycopg
import pytest
from psycopg import errors

from mapping_support import SATURATION, origin, real_declaration, writer
from thermo_knowledge.build import build_database
from thermo_knowledge.canonical.invariants import LOAD_INVARIANTS
from thermo_knowledge.canonical.values import Quantity
from thermo_knowledge.canonical.writer import CanonicalWriter, ValidationError
from thermo_knowledge.testing import TestDatabase


def fit(w: CanonicalWriter) -> uuid.UUID:
    return w.kind(
        "fit",
        {"key": "f", "kind": "fit", "outcome": "converged"},
        origins=[origin("a.json#/0", "published")],
    )


def test_a_fit_has_no_covariance_attribute() -> None:
    decl = real_declaration()
    assert "covariance" not in {a.name for a in decl.attributes_of("fit")}
    free = decl.relations["fit_free_parameter"]
    column = next(c for c in free.values if c.name == "standard_uncertainty")
    assert (column.type.text, column.optional, column.unit_from) == ("Real", True, "slot")


def free_parameter(
    w: CanonicalWriter, found: uuid.UUID, uncertainty: object, *, slot: bool = True
) -> uuid.UUID:
    values: dict[str, object] = {"record": uuid.uuid4(), "standard_uncertainty": uncertainty}
    if slot:
        values["slot"] = f"{SATURATION}.T_r"
    return w.relation("fit_free_parameter", {"fit": found, "ordinal": 1}, values)


def test_a_standard_uncertainty_is_converted_to_the_storage_unit_of_its_slot() -> None:
    w = writer(real_declaration())
    free_parameter(w, fit(w), Quantity(250.0, "mK"))
    (row,) = w.tables()["prov.fit_free_parameter"].to_pylist()
    assert row["standard_uncertainty"] == pytest.approx(0.25)


def test_a_standard_uncertainty_needs_the_slot_that_fixes_its_unit() -> None:
    w = writer(real_declaration())
    with pytest.raises(ValidationError, match="its unit comes from `slot`"):
        free_parameter(w, fit(w), Quantity(1.0, "K"), slot=False)
    rule = LOAD_INVARIANTS[("fit_free_parameter", "uncertainty_needs_slot")]
    assert rule({"standard_uncertainty": 1.0, "slot": None}) is not None
    assert rule({"standard_uncertainty": 1.0, "slot": uuid.uuid4()}) is None
    assert rule({"standard_uncertainty": None, "slot": None}) is None


def test_a_parameter_without_an_uncertainty_is_still_a_parameter() -> None:
    w = writer(real_declaration())
    w.relation("fit_free_parameter", {"fit": fit(w), "ordinal": 1}, {"record": uuid.uuid4()})
    (row,) = w.tables()["prov.fit_free_parameter"].to_pylist()
    assert row["standard_uncertainty"] is None and row["slot"] is None


def correlate(w: CanonicalWriter, found: uuid.UUID, a: int, b: int, coefficient: float) -> None:
    w.relation("fit_correlation", {"fit": found, "a": a, "b": b}, {"coefficient": coefficient})


def test_a_correlation_is_stored_once_in_the_canonical_orientation() -> None:
    w = writer(real_declaration())
    found = fit(w)
    correlate(w, found, 2, 1, 0.5)
    correlate(w, found, 1, 3, -1.0)
    rows = {
        (r["a"], r["b"]): r["coefficient"] for r in w.tables()["prov.fit_correlation"].to_pylist()
    }
    assert rows == {(1, 2): 0.5, (1, 3): -1.0}


def test_the_diagonal_and_a_coefficient_out_of_range_are_refused_by_the_writer() -> None:
    w = writer(real_declaration())
    found = fit(w)
    with pytest.raises(ValidationError, match="forbids the diagonal"):
        correlate(w, found, 2, 2, 1.0)
    with pytest.raises(ValidationError, match="coefficient 1.5 is above 1.0"):
        correlate(w, found, 1, 2, 1.5)
    with pytest.raises(ValidationError, match="coefficient -1.5 is below -1.0"):
        correlate(w, found, 1, 2, -1.5)
    assert w.rows("prov.fit_correlation") == 0


@pytest.fixture(scope="module")
def conn() -> Iterator[psycopg.Connection]:
    with TestDatabase() as database:
        build_database(database.url, real_declaration())
        connection = psycopg.connect(database.url)
        try:
            yield connection
        finally:
            connection.close()


INSERT = "INSERT INTO prov.fit_correlation (id, fit, a, b, coefficient) VALUES (%s, %s, %s, %s, %s)"


def attempt(conn: psycopg.Connection, a: int, b: int, coefficient: float) -> None:
    with conn.transaction(force_rollback=True):
        conn.execute(INSERT, (uuid.uuid4(), uuid.uuid4(), a, b, coefficient))


def test_the_database_refuses_a_correlation_out_of_range(conn: psycopg.Connection) -> None:
    for coefficient in (-1.0, 0.0, 1.0):
        attempt(conn, 1, 2, coefficient)
    for coefficient in (1.0000001, -1.5):
        with pytest.raises(errors.CheckViolation) as raised:
            attempt(conn, 1, 2, coefficient)
        assert raised.value.diag.constraint_name == "fit_correlation__ck__coefficient_in_range"
    with pytest.raises(errors.CheckViolation) as raised:
        attempt(conn, 2, 1, 0.5)
    assert raised.value.diag.constraint_name == "fit_correlation__ck__canonical"
    with pytest.raises(errors.CheckViolation) as raised:
        attempt(conn, 2, 2, 0.5)
    assert raised.value.diag.constraint_name in {
        "fit_correlation__ck__diagonal",
        "fit_correlation__ck__canonical",
    }
