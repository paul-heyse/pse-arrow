# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Hard case (k), plan 24 packet TK2: sigma profiles and polymer models.

A COSMO-SAC sigma profile is one tabulated function with three named series (non-hydrogen-bonding,
hydrogen-bond donor and other hydrogen-bonding area) on one screening-charge-density grid, typed by
the `sigma_profile` distributed attribute, stored with the molecule's cavity volume and dispersion
class; the parameterisation of the profiles has a `dependency` on the parameterisation of the model
constants they were generated for. A polymer is a polymer type with repeat units and a Flory-Huggins
parameter set; its molar-mass distribution is a tabulated function held on a sample, never on the type,
and the number of segments of a chain, which the Flory-Huggins expression needs, is read from it.

The numbers are synthetic. The profile's area is the sum of the three series; it is compared with
numpy on the stored values (the expression language cannot read the values of a tabulated function, so
the form is catalogued). The Flory-Huggins excess Gibbs energy is an expressed form, evaluated and
compared with the published expression written in numpy.
"""

from __future__ import annotations

import uuid
from collections.abc import Iterator
from dataclasses import dataclass
from pathlib import Path

import numpy as np
import psycopg
import pytest
from hard_case_support import at, build, entity_id, failing
from mapping_support import carrier, real_declaration, writer

from thermo_knowledge import db, identity
from thermo_knowledge.build.database import DatabaseRefusedError
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
from thermo_knowledge.expression.evaluate import bind
from thermo_knowledge.qualify.source import DatabaseSource
from thermo_knowledge.testing import TestDatabase

CACHE = CompileCache()
CARRIER = identity.identifier("source", [carrier("src", "a.json").key])
R = 8.314462618
ELEMENTARY_CHARGE = 1.602176634e-19  # C, defined
ANGSTROM = 1e-10

# -- the synthetic molecules --------------------------------------------------------------------

SIGMA = np.round(np.linspace(-0.025, 0.025, 51), 6)  # e / angstrom^2
MOLECULES = {  # name: (dispersion class, cavity volume in angstrom^3, dispersion energy over k in K, series centres and heights)
    "methanol-like": ("hb_donor_acceptor", 52.0, 340.0, dict(nhb=(0.002, 0.010, 14.0), oh=(0.016, 0.006, 6.0), ot=(-0.015, 0.006, 5.0))),
    "hexane-like": ("nhb", 140.0, 210.0, dict(nhb=(0.000, 0.012, 95.0), oh=(0.016, 0.006, 0.0), ot=(-0.015, 0.006, 0.0))),
}
CONSTANTS = dict(  # in the units the source states, before conversion: kcal A^4 / (mol e^2) and K
    effective_area=7.25, c_oh_oh=4013.78, c_oh_ot=932.31, c_ot_ot=3016.43, a_es=6525.69, b_es=1.4859e8, q0=79.53, r0=66.69, z=10.0
)
KCAL = 4184.0


def series_of(shape: tuple[float, float, float]) -> np.ndarray:
    centre, width, height = shape
    return height * np.exp(-(((SIGMA - centre) / width) ** 2))


# -- the polymer --------------------------------------------------------------------------------

REPEAT_UNITS = {"repeat-a": 0.3, "repeat-b": 0.7}  # a copolymer: fractions of its units
M_REPEAT = 0.0281  # kg/mol, from the formulas of the repeat units: synthetic
BINS = np.array([5_000.0, 10_000.0, 20_000.0, 40_000.0, 80_000.0]) * 1e-3  # kg/mol
WEIGHTS = np.array([0.10, 0.30, 0.35, 0.20, 0.05])  # mass fractions of the bins
V_SEGMENT = {"solvent": 9.0e-5, "polymer": 3.1e-5}  # m^3/mol
CHI = (0.12, 95.0)  # chi0, chi1 in K
V_REF = 5.0e-5


def number_average(masses: np.ndarray, weights: np.ndarray) -> float:
    return float(np.sum(weights) / np.sum(weights / masses))


def flory_huggins(T: float, x: dict[str, float], n_seg: dict[str, float]) -> float:
    """gE of the binary from the published expression, per mole of molecules."""
    names = list(x)
    V = {k: n_seg[k] * V_SEGMENT[k] for k in names}
    V_mean = sum(x[k] * V[k] for k in names)
    phi = {k: x[k] * V[k] / V_mean for k in names}
    chi = CHI[0] + CHI[1] / T
    solvent, polymer = names
    athermal = sum(x[k] * np.log(phi[k] / x[k]) for k in names)
    return R * T * (athermal + V_mean / V_REF * chi * phi[solvent] * phi[polymer])


# -- the fixture --------------------------------------------------------------------------------


@dataclass
class World:
    decl: Declaration
    database: TestDatabase
    ids: dict[str, uuid.UUID]


def sigma_function(decl: Declaration, shapes: dict[str, tuple[float, float, float]]) -> TabulatedFunction:
    return TabulatedFunction(
        "bin_weights",
        axes=[TabulatedAxis("SurfaceChargeDensity", QuantityArray(list(SIGMA), "e/angstrom**2"))],
        series=[
            TabulatedSeries(name, "Area", QuantityArray(list(series_of(shape)), "angstrom**2"))
            for name, shape in shapes.items()
        ],
        distribution=entity_id(decl, "distributed_attribute", "sigma_profile"),
    )


def write_world(w: CanonicalWriter, decl: Declaration, ids: dict[str, uuid.UUID]) -> None:
    # the model constants, and two profile parameterisations generated for two sets of them
    constants = {}
    for key, scale in (("hsieh-constants", 1.0), ("refit-constants", 1.07)):
        constants[key] = w.kind(
            "parameterization",
            {"key": key, "revision": "1", "title": key, "coherence": "independent_records"},
            origins=at(f"parameterization-{key}"),
        )
        c = CONSTANTS
        w.parameter_set(
            parameterization=constants[key],
            slot_group="cosmosac_2010.constants",
            subjects=[],
            slots={
                "effective_area": Quantity(c["effective_area"], "angstrom**2"),
                "c_oh_oh": Quantity(scale * c["c_oh_oh"], "kcal/mol * angstrom**4 / e**2"),
                "c_oh_ot": Quantity(scale * c["c_oh_ot"], "kcal/mol * angstrom**4 / e**2"),
                "c_ot_ot": Quantity(scale * c["c_ot_ot"], "kcal/mol * angstrom**4 / e**2"),
                "a_es": Quantity(scale * c["a_es"], "kcal/mol * angstrom**4 / e**2"),
                "b_es": Quantity(scale * c["b_es"], "kcal/mol * kelvin**2 * angstrom**4 / e**2"),
                "q0": Quantity(c["q0"], "angstrom**2"),
                "r0": Quantity(c["r0"], "angstrom**3"),
                "z_coordination": c["z"],
            },
            origins=at(f"constants-{key}"),
        )
    ids.update({key.replace("-", "_"): p for key, p in constants.items()})
    profiles = {}
    for key, dependency in (("profiles-hsieh-averaging", "hsieh-constants"), ("profiles-refit-averaging", "refit-constants")):
        profiles[key] = w.kind(
            "parameterization",
            {"key": key, "revision": "1", "title": key, "coherence": "independent_records"},
            origins=at(f"parameterization-{key}"),
        )
        w.relation(
            "dependency",
            {"dependent": profiles[key], "prerequisite": constants[dependency]},
            {"kind": "consistent_with"},
            at=f"a.json#/dependency-{key}",
        )
    ids.update({key.replace("-", "_"): p for key, p in profiles.items()})
    for name, (dispersion_class, volume, energy, shapes) in MOLECULES.items():
        species = w.kind("species", {"canonical_key": name, "label": name}, origins=at(f"s-{name}"))
        ids[f"species_{name}"] = species
        ids[f"profile_{name}"] = w.parameter_set(
            parameterization=profiles["profiles-hsieh-averaging"],
            slot_group="cosmosac_2010.molecule",
            subjects=[species],
            slots={
                "profile": sigma_function(decl, shapes),
                "cavity_volume": Quantity(volume, "angstrom**3"),
                "dispersion_class": dispersion_class,
                "dispersion_energy_over_k": Quantity(energy, "K"),
            },
            origins=at(f"profile-{name}", "computed"),
        )
    # a polymer: repeat units, a Flory-Huggins set with its solvent, and the distribution of a sample
    units = {n: w.kind("species", {"canonical_key": n, "label": n}, origins=at(f"s-{n}")) for n in REPEAT_UNITS}
    solvent = w.kind("species", {"canonical_key": "solvent", "label": "solvent"}, origins=at("s-solvent"))
    polymer = w.kind(
        "polymer_type",
        {"canonical_key": "copolymer-ab", "label": "a synthetic copolymer", "architecture": "random, linear"},
        origins=at("polymer"),
    )
    ids.update(solvent=solvent, polymer=polymer)
    for name, fraction in REPEAT_UNITS.items():
        w.relation("repeat_unit", {"polymer": polymer, "unit": units[name]}, {"value": fraction}, at="a.json#/repeat")
    fh = w.kind(
        "parameterization",
        {
            "key": "flory-huggins-solvent-copolymer",
            "revision": "1",
            "title": "Flory-Huggins",
            "coherence": "independent_records",
            "convention_set": w.kind(
                "convention_set",
                {"key": "fh-conventions", "revision": "1", "temperature_scale": "its_90", "gas_constant": Quantity(R, "J/(mol*K)")},
                origins=at("fh-conventions"),
            ),
        },
        origins=at("parameterization-fh"),
    )
    ids["flory_huggins"] = fh
    for name, entity in (("solvent", solvent), ("polymer", polymer)):
        w.parameter_set(
            parameterization=fh,
            slot_group="flory_huggins_excess_gibbs.component",
            subjects=[entity],
            slots={"segment_volume": Quantity(V_SEGMENT[name], "m^3/mol")},
            origins=at(f"fh-segment-{name}"),
        )
    w.parameter_set(
        parameterization=fh,
        slot_group="flory_huggins_excess_gibbs.pair",
        subjects=[polymer, solvent],
        slots={"chi0": CHI[0], "chi1": Quantity(CHI[1], "K")},
        origins=at("fh-pair"),
    )
    w.parameter_set(
        parameterization=fh,
        slot_group="flory_huggins_excess_gibbs.lattice",
        subjects=[],
        slots={"reference_volume": Quantity(V_REF, "m^3/mol")},
        origins=at("fh-lattice"),
    )
    sample = w.kind(
        "sample",
        {
            "carrier": CARRIER,
            "local_key": "batch-1",
            "entity": polymer,
            "source": "synthesized_by_authors",
            "status": "described",
        },
        origins=at("sample"),
    )
    ids["sample"] = sample
    distributions = w.kind(
        "parameterization",
        {"key": "sample-distributions", "revision": "1", "title": "distributions", "coherence": "independent_records"},
        origins=at("parameterization-distributions"),
    )
    ids["distributions"] = w.parameter_set(
        parameterization=distributions,
        slot_group="molar_mass_distribution.sample",
        subjects=[sample],
        slots={
            "distribution": TabulatedFunction(
                "bin_weights",
                axes=[TabulatedAxis("MolarMass", QuantityArray(list(BINS * 1e3), "g/mol"))],
                series=[TabulatedSeries("weight", "Scalar", QuantityArray(list(WEIGHTS), "dimensionless"))],
                distribution=entity_id(decl, "distributed_attribute", "molar_mass_distribution_mass"),
            )
        },
        origins=at("distribution", "measured"),
    )


@pytest.fixture(scope="module")
def decl() -> Declaration:
    return real_declaration()


@pytest.fixture(scope="module")
def world(decl: Declaration, tmp_path_factory: pytest.TempPathFactory) -> Iterator[World]:
    ids: dict[str, uuid.UUID] = {}
    database = build(tmp_path_factory.mktemp("sigma-polymer"), lambda w: write_world(w, decl, ids), decl)
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
    """An array column of the database: a list, or the text of a PostgreSQL array of a domain."""
    if isinstance(value, str):
        return np.array([float(v) for v in value.strip("{}").split(",") if v])
    return np.asarray(value, dtype=float)


def function_of(conn: psycopg.Connection, world: World, name: str, slot_group: str = "cosmosac_2010__molecule") -> uuid.UUID:
    return scalar(conn, f'SELECT profile FROM param."{slot_group}" WHERE id = %s', world.ids[f"profile_{name}"])  # type: ignore[return-value]


def test_the_fixture_satisfies_every_invariant(conn: psycopg.Connection) -> None:
    assert failing(conn) == {}


# -- the sigma profile --------------------------------------------------------------------------


@pytest.mark.parametrize("name", list(MOLECULES))
def test_the_profile_is_one_function_with_three_named_series_on_one_grid(
    world: World, conn: psycopg.Connection, name: str
) -> None:
    function = function_of(conn, world, name)
    row = conn.execute(
        "SELECT interpolation::text, d.key FROM tk.tabulated_function f LEFT JOIN tk.distributed_attribute d ON d.id = f.distribution WHERE f.id = %s",
        (function,),
    ).fetchone()
    assert row == ("bin_weights", "sigma_profile")
    (axis,) = conn.execute(
        "SELECT ordinal, points FROM tk.tabulated_axis WHERE function = %s", (function,)
    ).fetchall()
    assert axis[0] == 1 and len(floats(axis[1])) == 51
    np.testing.assert_allclose(floats(axis[1]), SIGMA * ELEMENTARY_CHARGE / ANGSTROM**2, rtol=1e-12)  # e / A^2 in C / m^2
    series = {
        n: floats(v)
        for n, v in conn.execute("SELECT name, values FROM tk.tabulated_series WHERE function = %s", (function,)).fetchall()
    }
    assert sorted(series) == ["nhb", "oh", "ot"] and {len(v) for v in series.values()} == {51}
    for series_name, shape in MOLECULES[name][3].items():
        np.testing.assert_allclose(series[series_name], series_of(shape) * ANGSTROM**2, rtol=1e-12, atol=0.0)


@pytest.mark.parametrize("name", list(MOLECULES))
def test_the_area_of_the_profile_is_the_sum_of_its_series(world: World, conn: psycopg.Connection, name: str) -> None:
    function = function_of(conn, world, name)
    stored = float(
        scalar(
            conn,
            "SELECT sum(v) FROM tk.tabulated_series s, unnest(s.values) AS v WHERE s.function = %s",
            function,
        )  # type: ignore[arg-type]
    )
    want = sum(float(np.sum(series_of(shape))) for shape in MOLECULES[name][3].values()) * ANGSTROM**2
    assert stored == pytest.approx(want, rel=1e-12)
    per_series = dict(
        conn.execute(
            "SELECT s.name, (SELECT sum(v) FROM unnest(s.values) AS v) FROM tk.tabulated_series s WHERE s.function = %s",
            (function,),
        ).fetchall()
    )
    assert sum(per_series.values()) == pytest.approx(want, rel=1e-12)


def test_a_molecule_without_hydrogen_bonding_has_empty_donor_and_acceptor_series(world: World, conn: psycopg.Connection) -> None:
    series = {
        n: floats(v)
        for n, v in conn.execute(
            "SELECT name, values FROM tk.tabulated_series WHERE function = %s", (function_of(conn, world, "hexane-like"),)
        ).fetchall()
    }
    assert sum(series["oh"]) == 0.0 and sum(series["ot"]) == 0.0 and sum(series["nhb"]) > 0.0


def test_the_molecule_has_a_cavity_volume_and_a_dispersion_class_and_energy(world: World, conn: psycopg.Connection) -> None:
    for name, (dispersion_class, volume, energy, _) in MOLECULES.items():
        row = conn.execute(
            'SELECT cavity_volume, dispersion_class::text, dispersion_energy_over_k FROM param."cosmosac_2010__molecule" WHERE id = %s',
            (world.ids[f"profile_{name}"],),
        ).fetchone()
        assert row == (pytest.approx(volume * ANGSTROM**3), dispersion_class, energy)


def test_the_profile_parameterisation_depends_on_the_constants_it_was_generated_for(world: World, conn: psycopg.Connection) -> None:
    rows = dict(
        conn.execute(
            "SELECT dependent, prerequisite FROM tk.dependency WHERE kind = 'consistent_with'"
        ).fetchall()
    )
    assert rows == {
        world.ids["profiles_hsieh_averaging"]: world.ids["hsieh_constants"],
        world.ids["profiles_refit_averaging"]: world.ids["refit_constants"],
    }
    # the profiles of the molecules belong to the first, so they are valid with the first constants only
    owner = scalar(conn, "SELECT parameterization FROM tk.parameter_set WHERE id = %s", world.ids["profile_methanol-like"])
    assert rows[owner] == world.ids["hsieh_constants"]  # type: ignore[index]


def test_the_constants_are_one_global_set_in_the_storage_units(world: World, conn: psycopg.Connection) -> None:
    row = conn.execute(
        'SELECT c_oh_oh, b_es, effective_area, z_coordination FROM param."cosmosac_2010__constants" c '
        "JOIN tk.parameter_set s ON s.id = c.id WHERE s.parameterization = %s",
        (world.ids["hsieh_constants"],),
    ).fetchone()
    per_e2 = KCAL * ANGSTROM**4 / ELEMENTARY_CHARGE**2  # kcal A^4 / (mol e^2) in J m^4 / (mol C^2)
    assert row is not None
    assert row[0] == pytest.approx(CONSTANTS["c_oh_oh"] * per_e2, rel=1e-9)
    assert row[1] == pytest.approx(CONSTANTS["b_es"] * per_e2, rel=1e-9)  # with K^2 in the unit
    assert row[2] == pytest.approx(CONSTANTS["effective_area"] * ANGSTROM**2, rel=1e-12)
    assert row[3] == CONSTANTS["z"]
    assert scalar(conn, 'SELECT count(*) FROM param."cosmosac_2010__constants"') == 2  # one set for each constants parameterisation


# -- the polymer --------------------------------------------------------------------------------


def test_a_polymer_type_has_repeat_units_and_no_distribution(world: World, conn: psycopg.Connection, decl: Declaration) -> None:
    rows = dict(
        conn.execute(
            "SELECT e.canonical_key, r.value FROM tk.repeat_unit r JOIN tk.material_entity e ON e.id = r.unit WHERE r.polymer = %s",
            (world.ids["polymer"],),
        ).fetchall()
    )
    assert rows == REPEAT_UNITS
    assert sorted(a.name for a in decl.attributes_of("polymer_type")) == ["architecture", "canonical_key", "label", "provisional"]


def test_the_molar_mass_distribution_is_held_on_a_sample_of_the_polymer(world: World, conn: psycopg.Connection, decl: Declaration) -> None:
    (group,) = decl.forms["molar_mass_distribution"].slot_groups
    assert [(s.name, s.type.text) for s in group.subjects] == [("s", "sample")]
    row = conn.execute(
        'SELECT s, distribution FROM param."molar_mass_distribution__sample" WHERE id = %s', (world.ids["distributions"],)
    ).fetchone()
    assert row is not None and row[0] == world.ids["sample"]
    assert scalar(conn, "SELECT entity FROM tk.sample WHERE id = %s", world.ids["sample"]) == world.ids["polymer"]
    kind = scalar(
        conn,
        "SELECT d.key FROM tk.tabulated_function f JOIN tk.distributed_attribute d ON d.id = f.distribution WHERE f.id = %s",
        row[1],
    )
    assert kind == "molar_mass_distribution_mass"
    points, values = (
        conn.execute("SELECT a.points, s.values FROM tk.tabulated_axis a JOIN tk.tabulated_series s ON s.function = a.function WHERE a.function = %s", (row[1],)).fetchone()  # type: ignore[misc]
    )
    np.testing.assert_allclose(floats(points), BINS, rtol=1e-12)
    np.testing.assert_allclose(floats(values), WEIGHTS, rtol=1e-12)


def test_the_distribution_cannot_be_held_on_the_polymer_type(decl: Declaration, tmp_path: Path) -> None:
    """The writer does not know the kind of an entity; the database does: the subject of the set is
    a foreign key to the samples, and a polymer type is no sample, so the build is refused."""

    def emit(w: CanonicalWriter) -> None:
        polymer = w.kind("polymer_type", {"canonical_key": "p", "label": "p"}, origins=at("p"))
        p = w.kind(
            "parameterization",
            {"key": "d", "revision": "1", "title": "d", "coherence": "independent_records"},
            origins=at("d"),
        )
        w.parameter_set(
            parameterization=p,
            slot_group="molar_mass_distribution.sample",
            subjects=[polymer],
            slots={
                "distribution": TabulatedFunction(
                    "bin_weights",
                    axes=[TabulatedAxis("MolarMass", QuantityArray([1.0, 2.0], "kg/mol"))],
                    series=[TabulatedSeries("weight", "Scalar", QuantityArray([0.5, 0.5], "dimensionless"))],
                )
            },
            origins=at("on-type"),
        )

    with pytest.raises(DatabaseRefusedError, match="molar_mass_distribution__sample__fk__s"):
        build(tmp_path, emit, decl)


def test_flory_huggins_with_the_chain_length_of_the_sample_matches_numpy(world: World, conn: psycopg.Connection) -> None:
    m_n = number_average(BINS, WEIGHTS)
    n_polymer = m_n / M_REPEAT
    source = DatabaseSource(conn, world.decl, [world.ids["flory_huggins"]])
    members = [str(world.ids["solvent"]), str(world.ids["polymer"])]
    for order in (members, members[::-1]):  # the pair is symmetric: the order of the components does not matter
        bound = bind(world.decl, "flory_huggins_excess_gibbs", source=source, sets={"components": order}, cache=CACHE)
        for T in (300.0, 350.0, 420.0):
            for x_solvent in (0.6, 0.9, 0.995):
                x = {"solvent": x_solvent, "polymer": 1 - x_solvent}
                n_seg = {"solvent": 1.0, "polymer": n_polymer}
                by_id = {str(world.ids[k]): v for k, v in x.items()}
                segments = {str(world.ids[k]): np.array([v]) for k, v in n_seg.items()}
                found = float(
                    np.asarray(
                        bound.evaluate(
                            "gE",
                            T=np.array([T]),
                            x={k: np.array([v]) for k, v in by_id.items()},
                            n_seg=segments,
                        )
                    ).reshape(-1)[0]
                )
                assert found == pytest.approx(flory_huggins(T, x, n_seg), rel=1e-11)


def test_the_number_of_segments_of_a_chain_depends_on_the_sample(world: World, conn: psycopg.Connection) -> None:
    """The same polymer type with another sample distribution gives another excess Gibbs energy: the
    chain length is a fact of the sample, which is why the type holds none."""
    longer = number_average(BINS * 3, WEIGHTS)
    shorter = number_average(BINS, WEIGHTS)
    x = {"solvent": 0.9, "polymer": 0.1}
    a = flory_huggins(330.0, x, {"solvent": 1.0, "polymer": shorter / M_REPEAT})
    b = flory_huggins(330.0, x, {"solvent": 1.0, "polymer": longer / M_REPEAT})
    assert abs(a - b) / abs(a) > 1e-3


# -- what the model refuses ---------------------------------------------------------------------


def test_a_profile_whose_series_does_not_cover_the_grid_or_has_the_wrong_dimension_is_refused(decl: Declaration) -> None:
    w = writer(decl)
    species = w.kind("species", {"canonical_key": "s", "label": "s"}, origins=at("s"))
    p = w.kind("parameterization", {"key": "p", "revision": "1", "title": "p", "coherence": "independent_records"}, origins=at("p"))

    def write(function: TabulatedFunction) -> uuid.UUID:
        return w.parameter_set(
            parameterization=p,
            slot_group="cosmosac_2010.molecule",
            subjects=[species],
            slots={
                "profile": function,
                "cavity_volume": Quantity(50.0, "angstrom**3"),
                "dispersion_class": "nhb",
                "dispersion_energy_over_k": Quantity(200.0, "K"),
            },
            origins=at("molecule", "computed"),
        )

    axis = TabulatedAxis("SurfaceChargeDensity", QuantityArray(list(SIGMA), "e/angstrom**2"))
    good = [TabulatedSeries(n, "Area", QuantityArray([1.0] * 51, "angstrom**2")) for n in ("nhb", "oh", "ot")]
    with pytest.raises(ValidationError, match=r"has 50 values, the grid of the axes \(51\) has 51 points"):
        write(TabulatedFunction("bin_weights", [axis], [TabulatedSeries("nhb", "Area", QuantityArray([1.0] * 50, "angstrom**2")), *good[1:]]))
    with pytest.raises(ValidationError, match="cannot be converted"):
        write(TabulatedFunction("bin_weights", [axis], [TabulatedSeries("nhb", "Area", QuantityArray([1.0] * 51, "kg")), *good[1:]]))
    with pytest.raises(ValidationError, match="the series name is repeated"):
        write(TabulatedFunction("bin_weights", [axis], [good[0], good[0], good[2]]))
    with pytest.raises(ValidationError, match="not strictly ascending"):
        write(
            TabulatedFunction(
                "bin_weights",
                [TabulatedAxis("SurfaceChargeDensity", QuantityArray([0.0, 0.0] + list(SIGMA[2:]), "e/angstrom**2"))],
                good,
            )
        )
    with pytest.raises(ValidationError, match="not a member of enum `dispersion_class`"):
        w.parameter_set(
            parameterization=p,
            slot_group="cosmosac_2010.molecule",
            subjects=[species],
            slots={
                "profile": TabulatedFunction("bin_weights", [axis], good),
                "cavity_volume": Quantity(50.0, "angstrom**3"),
                "dispersion_class": "polar",
                "dispersion_energy_over_k": Quantity(200.0, "K"),
            },
            origins=at("molecule-bad-class", "computed"),
        )
    write(TabulatedFunction("bin_weights", [axis], good))  # the conforming function is accepted
