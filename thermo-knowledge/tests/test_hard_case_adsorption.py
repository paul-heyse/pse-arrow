# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Hard case (j), plan 24 packet TK2: adsorption with IAST and ISODB-style data.

The subject of an isotherm is an adsorbate (a species) and an adsorbent (a material): two roles of
different kinds. The Langmuir, dual-site Langmuir and Toth forms give the loading per mass of host as
a function of pressure; the kind of amount they give (absolute, excess or not stated) is named by
the set, so the form's output takes its observable from the data, and the dimension of the loading
is its basis: a set that names an observable of another basis is refused. The spreading pressure of
ideal adsorbed solution theory is the definite integral of the isotherm over the logarithm of the
pressure; the form that holds it has a sub-form slot for the isotherm and integrates in closed form
(Langmuir) or by quadrature (Toth). A pore-size kernel is a tabulated function on two axes.

ISODB-style datasets: an isotherm whose source does not say whether its amounts are absolute or
excess, with amounts that include a negative one (`ExcessLoading`, a signed type), an excess
isotherm whose loading falls below zero at high pressure, an absolute one, and isotherms stated per
volume, per surface area and per unit cell, each a column of the observable of its basis.

The numbers are synthetic. The independent calculations are the isotherm equations, the closed form of
the Langmuir integral and a composite Gauss-Legendre quadrature of the Toth integral, written in numpy.
"""

from __future__ import annotations

import uuid
from collections.abc import Callable, Iterator
from dataclasses import dataclass
from pathlib import Path

import numpy as np
import psycopg
import pytest
from hard_case_support import CHECKS, at, build, failing, observable_id
from mapping_support import carrier, real_declaration, writer

from thermo_knowledge import db, identity
from thermo_knowledge.canonical.values import Quantity, QuantityArray
from thermo_knowledge.canonical.writer import (
    CanonicalWriter,
    TabulatedAxis,
    TabulatedFunction,
    TabulatedSeries,
    ValidationError,
)
from thermo_knowledge.declaration import Declaration
from thermo_knowledge.expression.compiled import CompileCache
from thermo_knowledge.expression.evaluate import EvaluationRefusal, bind
from thermo_knowledge.expression.validity import Membership
from thermo_knowledge.qualify.source import DatabaseSource, SubformBinding
from thermo_knowledge.testing import TestDatabase
from thermo_knowledge.verify.run import run_check

CACHE = CompileCache()
CARRIER = identity.identifier("source", [carrier("src", "a.json").key])
BAR = 1.0e5

# -- the synthetic coefficients ------------------------------------------------------------------

LANGMUIR = dict(n_m=4.5, K=2.2e-5)  # mol/kg, 1/Pa
DUAL_SITE = dict(n_m1=3.0, K1=8.0e-5, n_m2=2.0, K2=4.0e-6)
TOTH = dict(n_m=5.2, K=1.5e-5, t=0.62)
PRESSURES = np.array([1.0e2, 1.0e3, 1.0e4, 1.0e5, 1.0e6])  # Pa
FITTED_TEMPERATURE = 298.0  # K
PRESSURE_RANGE = (10.0, 2.0e6)  # Pa

# -- the isotherms of the datasets, as a digitised library states them -----------------------------

ISOTHERM_PRESSURE_BAR = (0.01, 0.1, 0.5, 1.0, 5.0, 10.0)
NOT_STATED_MMOL_G = (-0.03, 0.6, 1.9, 2.5, 2.9, 3.0)  # a negative amount at the lowest pressure
EXCESS_MMOL_G = (0.05, 0.9, 2.3, 2.6, 1.4, -0.15)  # the excess falls below zero at high pressure
ABSOLUTE_MMOL_G = (0.1, 1.0, 2.4, 2.9, 3.6, 3.8)

BASES = {  # observable key: (source unit, stored factor into the storage unit, three amounts, some negative)
    "adsorbed_amount_not_stated_per_volume": ("mmol/cm**3", 1.0e3, (-0.2, 0.4, 1.3)),
    "adsorbed_amount_not_stated_per_area": ("micromol/m**2", 1.0e-6, (0.1, 2.0, -0.4)),
    "adsorbed_amount_not_stated_per_cell": ("dimensionless", 1.0, (0.2, 1.6, 3.1)),
}

# -- the pore-size kernel: loading of an ideal isotherm in a pore ----------------------------------

WIDTHS_NM = np.array([0.4, 0.6, 0.9, 1.4, 2.2])
RELATIVE_PRESSURE = np.array([1.0e-4, 1.0e-3, 1.0e-2, 1.0e-1, 0.5])
KERNEL_TEMPERATURE = 77.0


def kernel_mmol_g() -> np.ndarray:
    """Synthetic loading in mmol/g on the grid: widths down the rows, relative pressures across."""
    w, x = np.meshgrid(WIDTHS_NM, RELATIVE_PRESSURE, indexing="ij")
    return (2.0 + 6.0 * w) * (300.0 * x) / (1.0 + 300.0 * x)


# -- the independent calculation -------------------------------------------------------------------


def langmuir(p: np.ndarray, n_m: float, K: float) -> np.ndarray:
    return n_m * K * p / (1.0 + K * p)


def dual_site(p: np.ndarray) -> np.ndarray:
    return langmuir(p, DUAL_SITE["n_m1"], DUAL_SITE["K1"]) + langmuir(p, DUAL_SITE["n_m2"], DUAL_SITE["K2"])


def toth(p: np.ndarray) -> np.ndarray:
    n_m, K, t = TOTH["n_m"], TOTH["K"], TOTH["t"]
    return n_m * K * p / (1.0 + (K * p) ** t) ** (1.0 / t)


NODES, WEIGHTS = np.polynomial.legendre.leggauss(16)


def integrate(f: Callable[[np.ndarray], np.ndarray], upper: float) -> float:
    """The integral of f over (0, upper) by Gauss-Legendre panels that halve toward the origin, where the
    Toth integrand is not analytic: sixty panels and the remainder near zero."""
    edges = upper * 0.5 ** np.arange(0, 61)
    total = 0.0
    for high, low in zip(edges[:-1], edges[1:], strict=True):
        x = 0.5 * (high - low) * NODES + 0.5 * (high + low)
        total += 0.5 * (high - low) * float(np.sum(WEIGHTS * f(x)))
    x = 0.5 * edges[-1] * (NODES + 1.0)
    return total + 0.5 * edges[-1] * float(np.sum(WEIGHTS * f(x)))


def spreading(isotherm: Callable[[np.ndarray], np.ndarray], P: float) -> float:
    """psi(P) = integral of n(p) / p dp over (0, P), the integrand taken as n(p) / p."""
    return integrate(lambda p: isotherm(p) / p, P)


# -- the fixture -------------------------------------------------------------------------------------


@dataclass
class World:
    decl: Declaration
    database: TestDatabase
    ids: dict[str, uuid.UUID]


def isotherm_dataset(
    w: CanonicalWriter,
    decl: Declaration,
    ids: dict[str, uuid.UUID],
    key: str,
    observable: str,
    unit: str,
    values: tuple[float, ...],
    pressures_bar: tuple[float, ...],
    *,
    kind: str = "measured",
) -> uuid.UUID:
    dataset = w.kind(
        "dataset",
        {"carrier": CARRIER, "local_key": key, "kind": kind, "title": f"isotherm {key}"},
        origins=at(key, "measured"),
    )
    ids[key] = dataset
    w.kind(
        "dataset_component",
        {"dataset": dataset, "ordinal": 1, "entity": ids["host"], "function": "adsorbent"},
        at="a.json#/components",
    )
    guest = w.kind(
        "dataset_component",
        {"dataset": dataset, "ordinal": 2, "entity": ids["adsorbate"], "function": "adsorbate"},
        at="a.json#/components",
    )
    w.kind(
        "dataset_column",
        {"dataset": dataset, "ordinal": 1, "role": "constraint", "observable": observable_id(decl, "temperature"), "constant": Quantity(FITTED_TEMPERATURE, "K")},
        at="a.json#/columns",
    )
    pressure = w.kind(
        "dataset_column",
        {"dataset": dataset, "ordinal": 2, "role": "variable", "observable": observable_id(decl, "pressure")},
        at="a.json#/columns",
    )
    loading = w.kind(
        "dataset_column",
        {"dataset": dataset, "ordinal": 3, "role": "property", "observable": observable_id(decl, observable), "component": guest},
        at="a.json#/columns",
    )
    for index, (p, n) in enumerate(zip(pressures_bar, values, strict=True), start=1):
        point = w.kind("data_point", {"dataset": dataset, "index": index}, at="a.json#/points")
        w.relation("datum", {"point": point, "column": pressure}, {"state": "known", "value": Quantity(p, "bar")}, at="a.json#/datum")
        w.relation("datum", {"point": point, "column": loading}, {"state": "known", "value": Quantity(n, unit)}, at="a.json#/datum")
    return dataset


def write_world(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    ids["adsorbate"] = w.kind("species", {"canonical_key": "adsorbate-a", "label": "a gas"}, origins=at("adsorbate"))
    ids["probe"] = w.kind("species", {"canonical_key": "probe", "label": "a probe gas"}, origins=at("probe"))
    # an adsorbent is a material identified by a registry or a name: no formula, no structure
    ids["host"] = w.kind(
        "material", {"canonical_key": "host-zeolite", "label": "a zeolite", "registry_key": "NIST-MATDB-synthetic"}, origins=at("host")
    )
    ids["carbon"] = w.kind("material", {"canonical_key": "host-carbon", "label": "a carbon"}, origins=at("carbon"))
    fits = w.kind(
        "parameterization",
        {"key": "isotherm-fits", "revision": "1", "title": "fitted isotherms", "coherence": "independent_records"},
        origins=at("fits"),
    )
    ids["fits"] = fits
    subjects = [ids["adsorbate"], ids["host"]]
    ids["langmuir"] = w.parameter_set(
        parameterization=fits,
        slot_group="langmuir_isotherm.pure",
        subjects=subjects,
        slots={
            "quantity": observable_id(decl, "adsorbed_amount_absolute"),
            "n_m": Quantity(LANGMUIR["n_m"], "mol/kg"),
            "K": Quantity(LANGMUIR["K"], "1/Pa"),
        },
        origins=at("langmuir", "published"),
    )
    ids["dual_site"] = w.parameter_set(
        parameterization=fits,
        slot_group="dual_site_langmuir_isotherm.pure",
        subjects=subjects,
        slots={
            "quantity": observable_id(decl, "adsorbed_amount_absolute"),
            "n_m1": Quantity(DUAL_SITE["n_m1"], "mol/kg"),
            "K1": Quantity(DUAL_SITE["K1"], "1/Pa"),
            "n_m2": Quantity(DUAL_SITE["n_m2"], "mol/kg"),
            "K2": Quantity(DUAL_SITE["K2"], "1/Pa"),
        },
        origins=at("dual-site", "published"),
    )
    ids["toth"] = w.parameter_set(
        parameterization=fits,
        slot_group="toth_isotherm.pure",
        subjects=subjects,
        slots={
            "quantity": observable_id(decl, "adsorbed_amount_excess"),
            "n_m": Quantity(TOTH["n_m"], "mol/kg"),
            "K": Quantity(TOTH["K"], "1/Pa"),
            "t": TOTH["t"],
        },
        origins=at("toth", "published"),
    )
    # the temperature and the pressure range a fit holds for are its fitted range, not arguments
    w.validity_region(
        ids["langmuir"],
        {"kind": "fitted_range"},
        [
            {"observable": observable_id(decl, "temperature"), "lower": Quantity(FITTED_TEMPERATURE, "K"), "upper": Quantity(FITTED_TEMPERATURE, "K")},
            {"observable": observable_id(decl, "pressure"), "lower": Quantity(PRESSURE_RANGE[0], "Pa"), "upper": Quantity(PRESSURE_RANGE[1], "Pa")},
        ],
        origins=at("langmuir-range"),
    )
    # the kernel: a table on the grid of pore widths and relative pressures
    kernels = w.kind(
        "parameterization",
        {"key": "kernels", "revision": "1", "title": "a pore-size kernel", "coherence": "independent_records"},
        origins=at("kernels"),
    )
    ids["kernel"] = w.parameter_set(
        parameterization=kernels,
        slot_group="pore_size_kernel_table.pure",
        subjects=[ids["probe"], ids["carbon"]],
        slots={
            "temperature": Quantity(KERNEL_TEMPERATURE, "K"),
            "kernel": TabulatedFunction(
                "linear",
                axes=[
                    TabulatedAxis("Length", QuantityArray(list(WIDTHS_NM), "nm")),
                    TabulatedAxis("Scalar", QuantityArray(list(RELATIVE_PRESSURE), "dimensionless")),
                ],
                series=[TabulatedSeries("loading", "Loading", QuantityArray(list(kernel_mmol_g().reshape(-1)), "mmol/g"))],
            ),
        },
        origins=at("kernel", "computed"),
    )
    # the isotherms of the library
    isotherm_dataset(w, decl, ids, "not-stated", "adsorbed_amount_not_stated", "mmol/g", NOT_STATED_MMOL_G, ISOTHERM_PRESSURE_BAR)
    isotherm_dataset(w, decl, ids, "excess", "adsorbed_amount_excess", "mmol/g", EXCESS_MMOL_G, ISOTHERM_PRESSURE_BAR)
    isotherm_dataset(w, decl, ids, "absolute", "adsorbed_amount_absolute", "mmol/g", ABSOLUTE_MMOL_G, ISOTHERM_PRESSURE_BAR)
    for observable, (unit, _, values) in BASES.items():
        isotherm_dataset(w, decl, ids, observable, observable, unit, values, ISOTHERM_PRESSURE_BAR[: len(values)])


@pytest.fixture(scope="module")
def decl() -> Declaration:
    return real_declaration()


@pytest.fixture(scope="module")
def world(decl: Declaration, tmp_path_factory: pytest.TempPathFactory) -> Iterator[World]:
    ids: dict[str, uuid.UUID] = {}
    database = build(tmp_path_factory.mktemp("adsorption"), lambda w: write_world(w, decl, ids), decl)
    try:
        yield World(decl, database, ids)
    finally:
        database.remove()


@pytest.fixture
def conn(world: World) -> Iterator[psycopg.Connection]:
    connection = db.connect(world.database.url)
    connection.execute("SELECT 1")
    try:
        yield connection
    finally:
        connection.rollback()
        connection.close()


def scalar(conn: psycopg.Connection, query: str, *params: object) -> object:
    row = conn.execute(query, params).fetchone()  # type: ignore[arg-type]
    assert row is not None
    return row[0]


def floats(value: object) -> np.ndarray:
    if isinstance(value, str):
        return np.array([float(v) for v in value.strip("{}").split(",") if v])
    return np.asarray(value, dtype=float)


def isotherm(world: World, conn: psycopg.Connection, form: str) -> np.ndarray:
    bound = bind(
        world.decl,
        form,
        source=DatabaseSource(conn, world.decl, [world.ids["fits"]]),
        roles={"adsorbate": str(world.ids["adsorbate"]), "host": str(world.ids["host"])},
        cache=CACHE,
    )
    return np.asarray(bound.evaluate("n", p=PRESSURES), dtype=float).reshape(-1)


def test_the_fixture_satisfies_every_invariant(conn: psycopg.Connection) -> None:
    assert failing(conn) == {}


# -- the subject -------------------------------------------------------------------------------------


def test_an_isotherm_is_about_an_adsorbate_and_a_host_of_different_kinds_and_is_not_transposable(world: World, conn: psycopg.Connection) -> None:
    for form in ("langmuir_isotherm", "dual_site_langmuir_isotherm", "toth_isotherm"):
        (group,) = world.decl.forms[form].slot_groups
        assert [(s.name, s.type.text) for s in group.subjects] == [("adsorbate", "species"), ("host", "material")]
        assert group.transposition is None
    row = conn.execute(
        'SELECT adsorbate, host FROM param."langmuir_isotherm__pure" WHERE id = %s', (world.ids["langmuir"],)
    ).fetchone()
    assert row == (world.ids["adsorbate"], world.ids["host"])
    assert scalar(conn, "SELECT registry_key FROM tk.material WHERE id = %s", world.ids["host"]) == "NIST-MATDB-synthetic"
    assert scalar(conn, "SELECT count(*) FROM tk.composition WHERE entity = %s", world.ids["host"]) == 0  # no formula, no structure


def test_the_host_cannot_be_a_species_and_the_adsorbate_cannot_be_a_material(decl: Declaration) -> None:
    w = writer(decl)
    species = w.kind("species", {"canonical_key": "s", "label": "s"}, origins=at("s"))
    material = w.kind("material", {"canonical_key": "m", "label": "m"}, origins=at("m"))
    p = w.kind("parameterization", {"key": "p", "revision": "1", "title": "p", "coherence": "independent_records"}, origins=at("p"))
    slots = {"quantity": observable_id(decl, "adsorbed_amount_absolute"), "n_m": Quantity(1.0, "mol/kg"), "K": Quantity(1e-5, "1/Pa")}
    with pytest.raises(ValidationError, match="is a species, and the role of `langmuir_isotherm.pure` is of kind `material`"):
        w.parameter_set(parameterization=p, slot_group="langmuir_isotherm.pure", subjects=[species, species], slots=slots, origins=at("a", "published"))
    with pytest.raises(ValidationError, match="is a material, and the role of `langmuir_isotherm.pure` is of kind `species`"):
        w.parameter_set(parameterization=p, slot_group="langmuir_isotherm.pure", subjects=[material, material], slots=slots, origins=at("b", "published"))


# -- the isotherm forms ------------------------------------------------------------------------------


def test_the_isotherm_forms_match_numpy(world: World, conn: psycopg.Connection) -> None:
    np.testing.assert_allclose(isotherm(world, conn, "langmuir_isotherm"), langmuir(PRESSURES, **LANGMUIR), rtol=1e-12)
    np.testing.assert_allclose(isotherm(world, conn, "dual_site_langmuir_isotherm"), dual_site(PRESSURES), rtol=1e-12)
    np.testing.assert_allclose(isotherm(world, conn, "toth_isotherm"), toth(PRESSURES), rtol=1e-12)


def test_the_kind_of_amount_an_isotherm_gives_is_the_observable_its_set_names(world: World, conn: psycopg.Connection) -> None:
    names = dict(
        conn.execute(
            "SELECT s.id, o.key FROM tk.parameter_set s "
            'JOIN param."langmuir_isotherm__pure" l ON l.id = s.id JOIN tk.observable o ON o.id = l.quantity'
        ).fetchall()
    )
    assert names == {world.ids["langmuir"]: "adsorbed_amount_absolute"}
    excess = scalar(
        conn,
        'SELECT o.key FROM param."toth_isotherm__pure" t JOIN tk.observable o ON o.id = t.quantity WHERE t.id = %s',
        world.ids["toth"],
    )
    assert excess == "adsorbed_amount_excess"
    # one equation gives both: the form has no observable of its own
    (supplier,) = world.decl.forms["toth_isotherm"].output_observables
    assert (supplier.output, supplier.qualified) == ("n", "toth_isotherm.pure.quantity")


def test_a_set_that_names_an_observable_of_another_loading_basis_is_refused(decl: Declaration) -> None:
    w = writer(decl)
    a = w.kind("species", {"canonical_key": "s", "label": "s"}, origins=at("s"))
    m = w.kind("material", {"canonical_key": "m", "label": "m"}, origins=at("m"))
    p = w.kind("parameterization", {"key": "p", "revision": "1", "title": "p", "coherence": "independent_records"}, origins=at("p"))
    for key in ("adsorbed_amount_absolute_per_area", "adsorbed_amount_excess_per_volume", "adsorbed_amount_not_stated_per_cell", "temperature"):
        with pytest.raises(ValidationError, match="dimension"):
            w.parameter_set(
                parameterization=p,
                slot_group="langmuir_isotherm.pure",
                subjects=[a, m],
                slots={"quantity": observable_id(decl, key), "n_m": Quantity(1.0, "mol/kg"), "K": Quantity(1e-5, "1/Pa")},
                origins=at(f"wrong-{key}", "published"),
            )
    for key in ("adsorbed_amount_absolute", "adsorbed_amount_excess", "adsorbed_amount_not_stated"):  # the bases of a mass loading
        w.parameter_set(
            parameterization=p,
            slot_group="langmuir_isotherm.pure",
            subjects=[a, m],
            slots={"quantity": observable_id(decl, key), "n_m": Quantity(1.0, "mol/kg"), "K": Quantity(1e-5, "1/Pa")},
            origins=at(f"right-{key}", "published"),
            occurrence=1 + ["adsorbed_amount_absolute", "adsorbed_amount_excess", "adsorbed_amount_not_stated"].index(key),
        )


def test_the_temperature_and_pressure_range_of_a_fit_are_its_validity_not_arguments(world: World, conn: psycopg.Connection) -> None:
    bound = bind(
        world.decl,
        "langmuir_isotherm",
        source=DatabaseSource(conn, world.decl, [world.ids["fits"]]),
        roles={"adsorbate": str(world.ids["adsorbate"]), "host": str(world.ids["host"])},
        cache=CACHE,
    )
    assert [a.name for a in world.decl.contracts["adsorption_isotherm"].arguments] == ["p"]
    inside = bound.validity("n", "fitted_range", p=np.array([1.0e2, 1.0e5, 5.0e6]))
    # the pressure clause is decidable and the temperature clause is not (no argument names it): a point
    # in the pressure range is undetermined, one beyond it is outside
    assert list(inside) == [Membership.UNDETERMINED, Membership.UNDETERMINED, Membership.OUTSIDE]


# -- the spreading pressure of IAST --------------------------------------------------------------------


def spreading_bound(world: World, conn: psycopg.Connection, form: str):  # noqa: ANN201
    return bind(
        world.decl,
        "isotherm_spreading_pressure",
        source=DatabaseSource(
            conn,
            world.decl,
            [],
            subforms={"isotherm_spreading_pressure.isotherm": [SubformBinding(form, (world.ids["fits"],))]},
        ),
        roles={"adsorbate": str(world.ids["adsorbate"]), "host": str(world.ids["host"])},
        cache=CACHE,
    )


def test_the_spreading_pressure_of_a_langmuir_isotherm_is_the_closed_form(world: World, conn: psycopg.Connection) -> None:
    found = np.asarray(spreading_bound(world, conn, "langmuir_isotherm").evaluate("psi", P=PRESSURES), dtype=float).reshape(-1)
    np.testing.assert_allclose(found, LANGMUIR["n_m"] * np.log1p(LANGMUIR["K"] * PRESSURES), rtol=1e-12)
    # and the quadrature written here agrees with the closed form, which is what makes it a check of the Toth case
    np.testing.assert_allclose([spreading(lambda p: langmuir(p, **LANGMUIR), float(P)) for P in PRESSURES], found, rtol=1e-9)


def test_the_spreading_pressure_of_a_dual_site_isotherm_is_the_sum_of_its_sites(world: World, conn: psycopg.Connection) -> None:
    found = np.asarray(spreading_bound(world, conn, "dual_site_langmuir_isotherm").evaluate("psi", P=PRESSURES), dtype=float).reshape(-1)
    want = DUAL_SITE["n_m1"] * np.log1p(DUAL_SITE["K1"] * PRESSURES) + DUAL_SITE["n_m2"] * np.log1p(DUAL_SITE["K2"] * PRESSURES)
    np.testing.assert_allclose(found, want, rtol=1e-12)


def test_the_spreading_pressure_of_a_toth_isotherm_is_integrated_by_quadrature(world: World, conn: psycopg.Connection) -> None:
    found = np.asarray(spreading_bound(world, conn, "toth_isotherm").evaluate("psi", P=PRESSURES), dtype=float).reshape(-1)
    np.testing.assert_allclose(found, [spreading(toth, float(P)) for P in PRESSURES], rtol=1e-9)
    assert np.all(np.diff(found) > 0)  # the spreading pressure increases with pressure


def test_the_spreading_pressure_differentiates_to_the_loading_over_the_logarithm_of_the_pressure(world: World, conn: psycopg.Connection) -> None:
    eps = 1.0e-3
    bound = spreading_bound(world, conn, "toth_isotherm")
    upper = np.asarray(bound.evaluate("psi", P=PRESSURES * (1 + eps)), dtype=float).reshape(-1)
    lower = np.asarray(bound.evaluate("psi", P=PRESSURES * (1 - eps)), dtype=float).reshape(-1)
    slope = (upper - lower) / np.log((1 + eps) / (1 - eps))  # d psi / d ln P
    np.testing.assert_allclose(slope, toth(PRESSURES), rtol=1e-5)


def test_a_spreading_pressure_without_an_isotherm_for_the_subject_is_refused(world: World, conn: psycopg.Connection) -> None:
    bound = bind(
        world.decl,
        "isotherm_spreading_pressure",
        source=DatabaseSource(
            conn, world.decl, [world.ids["fits"]],
            subforms={"isotherm_spreading_pressure.isotherm": [SubformBinding("langmuir_isotherm", (world.ids["fits"],))]},
        ),
        roles={"adsorbate": str(world.ids["probe"]), "host": str(world.ids["carbon"])},
        cache=CACHE,
    )
    with pytest.raises(EvaluationRefusal, match="langmuir_isotherm.pure"):
        bound.evaluate("psi", P=np.array([1.0e4]))


# -- the datasets of a digitised library --------------------------------------------------------------


def column_values(conn: psycopg.Connection, dataset: uuid.UUID, ordinal: int) -> np.ndarray:
    return np.array(
        [
            r[0]
            for r in conn.execute(
                "SELECT d.value FROM ev.datum d JOIN ev.data_point p ON p.id = d.point JOIN ev.dataset_column c ON c.id = d.column "
                "WHERE p.dataset = %s AND c.ordinal = %s ORDER BY p.index",
                (dataset, ordinal),
            ).fetchall()
        ]
    )


@pytest.mark.parametrize(
    ("key", "observable", "adsorption", "quantity", "values"),
    [
        ("not-stated", "adsorbed_amount_not_stated", "not_stated", "ExcessLoading", NOT_STATED_MMOL_G),
        ("excess", "adsorbed_amount_excess", "excess", "ExcessLoading", EXCESS_MMOL_G),
        ("absolute", "adsorbed_amount_absolute", "absolute", "Loading", ABSOLUTE_MMOL_G),
    ],
)
def test_an_isotherm_column_is_typed_by_the_observable_of_its_adsorption_kind(
    world: World, conn: psycopg.Connection, key: str, observable: str, adsorption: str, quantity: str, values: tuple[float, ...]
) -> None:
    row = conn.execute(
        "SELECT o.key, o.adsorption::text, q.name, o.basis::text FROM ev.dataset_column c JOIN tk.observable o ON o.id = c.observable "
        "JOIN meta.quantity_type q ON q.id = o.quantity WHERE c.dataset = %s AND c.ordinal = 3",
        (world.ids[key],),
    ).fetchone()
    assert row == (observable, adsorption, quantity, "per_host_mass")
    np.testing.assert_allclose(column_values(conn, world.ids[key], 3), np.array(values) * 1.0, rtol=1e-12)  # mmol/g is mol/kg
    np.testing.assert_allclose(column_values(conn, world.ids[key], 2), np.array(ISOTHERM_PRESSURE_BAR) * BAR, rtol=1e-12)


def test_the_amounts_that_are_negative_are_held_by_a_signed_type_and_the_absolute_type_holds_none(world: World, conn: psycopg.Connection) -> None:
    assert column_values(conn, world.ids["not-stated"], 3).min() < 0.0  # a not-stated amount may be negative
    assert column_values(conn, world.ids["excess"], 3)[-1] < 0.0  # an excess may fall below zero at high pressure
    assert column_values(conn, world.ids["absolute"], 3).min() > 0.0
    scales = dict(conn.execute("SELECT name, scale::text FROM meta.quantity_type WHERE name LIKE '%%Loading'").fetchall())
    assert scales["Loading"] == "absolute" and scales["ExcessLoading"] == "difference"
    assert scales["CellLoading"] == "count" and scales["ExcessCellLoading"] == "difference"


def test_a_source_that_does_not_say_which_amount_it_reports_is_stated_as_such_on_every_basis(world: World, conn: psycopg.Connection) -> None:
    rows = conn.execute(
        "SELECT o.key, o.adsorption::text FROM tk.observable o WHERE o.adsorption::text = 'not_stated' ORDER BY o.key"
    ).fetchall()
    assert [r[0] for r in rows] == [
        "adsorbed_amount_not_stated",
        "adsorbed_amount_not_stated_per_area",
        "adsorbed_amount_not_stated_per_cell",
        "adsorbed_amount_not_stated_per_volume",
    ]
    bases = dict(conn.execute("SELECT key, basis::text FROM tk.observable WHERE key LIKE 'adsorbed_amount_%%'").fetchall())
    assert {bases[k] for k in bases if k.endswith("per_area")} == {"per_host_area"}
    assert {bases[k] for k in bases if k.endswith("per_volume")} == {"per_host_volume"}
    assert {bases[k] for k in bases if k.endswith("per_cell")} == {"per_unit_cell"}


@pytest.mark.parametrize("observable", list(BASES))
def test_an_isotherm_stated_per_volume_area_or_cell_is_a_column_of_the_observable_of_its_basis(
    world: World, conn: psycopg.Connection, observable: str
) -> None:
    unit, factor, values = BASES[observable]
    np.testing.assert_allclose(column_values(conn, world.ids[observable], 3), np.array(values) * factor, rtol=1e-12)
    assert scalar(
        conn,
        "SELECT o.key FROM ev.dataset_column c JOIN tk.observable o ON o.id = c.observable WHERE c.dataset = %s AND c.ordinal = 3",
        world.ids[observable],
    ) == observable


def test_a_unit_that_cannot_be_read_and_a_unit_of_another_basis_are_refused_by_the_writer(decl: Declaration) -> None:
    """The writer takes no guess: a loading unit that is not a unit, or one of another basis than its
    column's observable, is refused, which is why a mapping holds the row (`unit_not_parseable`)."""
    w = writer(decl)
    species = w.kind("species", {"canonical_key": "s", "label": "s"}, origins=at("s"))
    host = w.kind("material", {"canonical_key": "m", "label": "m"}, origins=at("m"))
    ids = {"adsorbate": species, "host": host}
    dataset_args = (w, decl, ids)
    with pytest.raises(ValidationError, match="`cm3.STP./g` is not a unit pint can parse"):
        isotherm_dataset(*dataset_args, "bad-unit", "adsorbed_amount_not_stated", "cm3(STP)/g", (1.0,), (1.0,))
    with pytest.raises(ValidationError, match="cannot be converted"):
        isotherm_dataset(*dataset_args, "wrong-basis", "adsorbed_amount_not_stated", "micromol/m**2", (1.0,), (1.0,))
    with pytest.raises(ValidationError, match="not a member of enum `adsorption_kind`"):
        w.kind("observable", {"key": "x", "quantity": "Loading", "subject": "adsorbate_on_host", "basis": "per_host_mass", "relation": "absolute", "path": "isothermal", "adsorption": "true"}, origins=at("x"))
    assert "unit_not_parseable" in {m.name for m in decl.enums["held_reason"].members}


# -- the kernel on two axes ----------------------------------------------------------------------------


def test_the_kernel_is_one_function_of_two_axes_with_a_series_in_row_major_order(world: World, conn: psycopg.Connection) -> None:
    function = scalar(conn, 'SELECT kernel FROM param."pore_size_kernel_table__pure" WHERE id = %s', world.ids["kernel"])
    axes = conn.execute("SELECT a.ordinal, q.name, a.points FROM tk.tabulated_axis a JOIN meta.quantity_type q ON q.id = a.axis_type WHERE a.function = %s ORDER BY a.ordinal", (function,)).fetchall()
    assert [a[:2] for a in axes] == [(1, "Length"), (2, "Scalar")]
    np.testing.assert_allclose(floats(axes[0][2]), WIDTHS_NM * 1e-9, rtol=1e-12)  # stored in metres
    np.testing.assert_allclose(floats(axes[1][2]), RELATIVE_PRESSURE, rtol=1e-12)
    ((name, values),) = conn.execute("SELECT name, values FROM tk.tabulated_series WHERE function = %s", (function,)).fetchall()
    assert name == "loading"
    grid = floats(values).reshape(len(WIDTHS_NM), len(RELATIVE_PRESSURE))  # the last axis varies fastest
    np.testing.assert_allclose(grid, kernel_mmol_g(), rtol=1e-12)  # mmol/g is mol/kg
    assert grid[3, 2] == pytest.approx((2.0 + 6.0 * 1.4) * 3.0 / 4.0, rel=1e-12)  # width 1.4 nm, relative pressure 0.01
    assert scalar(conn, 'SELECT temperature FROM param."pore_size_kernel_table__pure" WHERE id = %s', world.ids["kernel"]) == KERNEL_TEMPERATURE


def test_the_kernel_is_catalogued_with_its_equation_external_and_a_grid_of_the_wrong_size_is_refused(world: World, decl: Declaration) -> None:
    form = world.decl.forms["pore_size_kernel_table"]
    assert (form.status, form.completeness) == ("catalogued", "structure_declared_equation_external")
    w = writer(decl)
    probe = w.kind("species", {"canonical_key": "s", "label": "s"}, origins=at("s"))
    carbon = w.kind("material", {"canonical_key": "m", "label": "m"}, origins=at("m"))
    p = w.kind("parameterization", {"key": "p", "revision": "1", "title": "p", "coherence": "independent_records"}, origins=at("p"))
    with pytest.raises(ValidationError, match=r"has 5 values, the grid of the axes \(25\) has 25 points|the grid of the axes"):
        w.parameter_set(
            parameterization=p,
            slot_group="pore_size_kernel_table.pure",
            subjects=[probe, carbon],
            slots={
                "temperature": Quantity(77.0, "K"),
                "kernel": TabulatedFunction(
                    "linear",
                    [TabulatedAxis("Length", QuantityArray(list(WIDTHS_NM), "nm")), TabulatedAxis("Scalar", QuantityArray(list(RELATIVE_PRESSURE), "dimensionless"))],
                    [TabulatedSeries("loading", "Loading", QuantityArray([1.0] * 5, "mmol/g"))],
                ),
            },
            origins=at("short", "computed"),
        )


# -- the sign rule of the scale ------------------------------------------------------------------------


def test_the_verify_checks_flag_a_negative_value_of_an_absolute_observable_and_no_signed_one(decl: Declaration, tmp_path: Path) -> None:
    """The column type of a value carries its unit and not its sign, so the scale of the quantity type
    decides: a negative absolute amount, pressure or constant is flagged, and a negative excess or
    not-stated amount (a signed type) is not."""
    ids: dict[str, uuid.UUID] = {}

    def emit(w: CanonicalWriter) -> None:
        ids["adsorbate"] = w.kind("species", {"canonical_key": "s", "label": "s"}, origins=at("s"))
        ids["host"] = w.kind("material", {"canonical_key": "m", "label": "m"}, origins=at("m"))
        # a negative absolute amount and a negative pressure: two values flagged
        ids["absolute"] = isotherm_dataset(w, decl, ids, "absolute", "adsorbed_amount_absolute", "mmol/g", (-0.5, 1.0), (-1.0, 2.0))
        # signed types: nothing flagged
        isotherm_dataset(w, decl, ids, "excess", "adsorbed_amount_excess", "mmol/g", (-0.5, 1.0), (1.0, 2.0))
        isotherm_dataset(w, decl, ids, "not-stated", "adsorbed_amount_not_stated", "mmol/g", (-0.5, 1.0), (1.0, 2.0))
        # a negative constant of an absolute observable
        ids["cold"] = w.kind("dataset", {"carrier": CARRIER, "local_key": "cold", "kind": "measured"}, origins=at("cold", "measured"))
        w.kind(
            "dataset_column",
            {"dataset": ids["cold"], "ordinal": 1, "role": "constraint", "observable": observable_id(decl, "temperature"), "constant": Quantity(-200.0, "K")},
            at="a.json#/columns",
        )

    database = build(tmp_path, emit, decl)
    try:
        with psycopg.connect(database.url, autocommit=True) as connection:
            assert failing(connection) == {"datum.value_respects_scale": 2, "dataset_column.constant_respects_scale": 1}
            flagged = run_check(connection, CHECKS["datum.value_respects_scale"], shown=10)
            position = flagged.columns.index("observable")
            assert {row[position] for row in flagged.rows} == {"adsorbed_amount_absolute", "pressure"}
    finally:
        database.remove()
