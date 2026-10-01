# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Complex constants held as real pairs (plan 24, packet TK2g, proof fixture 5; the IAPWS ice Ih
Gibbs energy the dispositions assume).

The complex part of the IAPWS Gibbs energy of ice Ih,
`g = T_t Re sum_k r_k [(t_k - tau) ln(t_k - tau) + (t_k + tau) ln(t_k + tau) - 2 t_k ln t_k - tau^2 / t_k]`,
has complex `t_k` and `r_k`. Each is a pair of real slots in a family row, the expression is written
in real arithmetic, and a sub-form takes the principal logarithm of a complex number (the argument
in (-pi, pi]). The evaluation is compared with Python's `cmath` on the same stored numbers.
"""

from __future__ import annotations

import cmath
import uuid
from collections.abc import Iterator
from dataclasses import dataclass

import numpy as np
import psycopg
import pytest

from build_support import fingerprint, inputs_of, write_source
from mapping_support import origin
from mechanisms_support import mechanism_declaration
from thermo_knowledge import config, db
from thermo_knowledge.build import build_database
from thermo_knowledge.canonical.values import Quantity
from thermo_knowledge.canonical.writer import CanonicalWriter, FamilyRow
from thermo_knowledge.declaration import Declaration
from thermo_knowledge.expression.evaluate import bind
from thermo_knowledge.qualify.source import DatabaseSource, SubformBinding
from thermo_knowledge.testing import TestDatabase
from thermo_knowledge.verify.checks import VERIFY_DIR, load_checks
from thermo_knowledge.verify.run import run_checks

CHECKS = {c.target: c for c in load_checks(config.TREE_DIR / VERIFY_DIR)[0]}
FORM = "ice_complex_fixture"
T_T = 273.16
# the complex constants of IAPWS R10-06(2009) as used here: t_k and r_k (J/(kg K))
T_K = (3.68017112855051e-2 + 5.10878114959572e-2j, 0.337315741065416 + 0.335449415919309j)
R_K = (44.7050716285388 + 65.6876847463481j, -72.597457432922 - 78.100842711287j)


@dataclass
class World:
    decl: Declaration
    database: TestDatabase
    ids: dict[str, uuid.UUID]


def at(tag: str) -> list:  # noqa: ANN401
    return [origin(f"a.json#/{tag}", "published")]


def write_world(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    ice = w.kind("species", {"canonical_key": "ice", "label": "ice"}, origins=at("ice"))
    ids["ice"] = ice
    p = w.kind(
        "parameterization",
        {"key": "ice-ih", "revision": "1", "title": "t", "coherence": "independent_records"},
        origins=at("p"),
    )
    ids["parameterization"] = p
    w.parameter_set(
        parameterization=p,
        slot_group=f"{FORM}.pure",
        subjects=[ice],
        slots={"T_t": Quantity(T_T, "K")},
        families={
            "term": [
                FamilyRow(
                    {"k": k},
                    {
                        "t_re": t.real,
                        "t_im": t.imag,
                        "r_re": Quantity(r.real, "J/(kg*K)"),
                        "r_im": Quantity(r.imag, "J/(kg*K)"),
                    },
                )
                for k, (t, r) in enumerate(zip(T_K, R_K), start=1)
            ]
        },
        origins=at("ice-terms"),
    )


@pytest.fixture(scope="module")
def world(tmp_path_factory: pytest.TempPathFactory) -> Iterator[World]:
    decl = mechanism_declaration(tmp_path_factory.mktemp("complex-declaration"))
    canonical = tmp_path_factory.mktemp("complex-canonical")
    ids: dict[str, uuid.UUID] = {}
    write_source(
        canonical,
        "src",
        lambda w: write_world(w, decl, ids),
        decl=decl,
        declaration=fingerprint(decl),
    )
    with TestDatabase() as database:
        build_database(database.url, decl, inputs_of(canonical))
        yield World(decl, database, ids)


@pytest.fixture
def conn(world: World) -> Iterator[psycopg.Connection]:
    connection = db.connect(world.database.url)
    connection.execute("SELECT 1")
    try:
        yield connection
    finally:
        connection.rollback()
        connection.close()


def reference(temperature: float) -> float:
    """The same sum in complex arithmetic, with the principal logarithm of `cmath`."""
    tau = temperature / T_T
    total = 0j
    for t, r in zip(T_K, R_K):
        total += r * (
            (t - tau) * cmath.log(t - tau)
            + (t + tau) * cmath.log(t + tau)
            - 2 * t * cmath.log(t)
            - tau**2 / t
        )
    return T_T * total.real


def bound(world: World, conn: psycopg.Connection):  # noqa: ANN201
    p = world.ids["parameterization"]
    source = DatabaseSource(
        conn,
        world.decl,
        [p],
        subforms={f"{FORM}.zlogz": [SubformBinding("principal_zlogz_fixture", (p,))]},
    )
    return bind(world.decl, FORM, source=source, roles={"i": str(world.ids["ice"])})


def test_every_fixture_satisfies_every_invariant(world: World, conn: psycopg.Connection) -> None:
    results = run_checks(conn, list(CHECKS.values()))
    assert [(r.check.target, r.violations, r.error) for r in results if not r.passed] == []


@pytest.mark.parametrize("temperature", [5.0, 50.0, 150.0, 240.0, 270.0, 273.0, 273.16])
def test_the_real_arithmetic_equals_cmath_with_the_principal_logarithm(
    world: World, conn: psycopg.Connection, temperature: float
) -> None:
    found = float(
        np.asarray(bound(world, conn).evaluate("g", T=np.array([temperature]))).reshape(-1)[0]
    )
    assert found == pytest.approx(reference(temperature), rel=1e-11, abs=1e-9)


def test_an_array_of_temperatures_is_evaluated_together(
    world: World, conn: psycopg.Connection
) -> None:
    temperatures = np.array([20.0, 100.0, 200.0, 260.0])
    found = np.asarray(bound(world, conn).evaluate("g", T=temperatures))
    np.testing.assert_allclose(found, [reference(float(T)) for T in temperatures], rtol=1e-11)


def test_the_temperature_where_the_real_part_of_t_k_minus_tau_changes_sign_is_crossed(
    world: World, conn: psycopg.Connection
) -> None:
    """The first constant has Re t_1 = 0.0368, so t_1 - tau has a negative real part above about
    10 K: the argument of the logarithm crosses from the first to the second quadrant, where a
    naive arctangent is wrong by pi. Both sides agree with cmath."""
    below, above = 9.0, 11.0
    assert (T_K[0] - below / T_T).real > 0 > (T_K[0] - above / T_T).real
    found = np.asarray(bound(world, conn).evaluate("g", T=np.array([below, above])))
    np.testing.assert_allclose(found, [reference(below), reference(above)], rtol=1e-11)


def test_the_complex_constants_are_held_as_pairs_of_real_numbers(
    world: World, conn: psycopg.Connection
) -> None:
    rows = conn.execute(
        'SELECT k, t_re, t_im, r_re, r_im FROM param."ice_complex_fixture__pure__term" ORDER BY k'
    ).fetchall()
    assert rows == [
        (k, t.real, t.imag, r.real, r.imag) for k, (t, r) in enumerate(zip(T_K, R_K), start=1)
    ]
