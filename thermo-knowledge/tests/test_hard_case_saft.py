# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Hard case (c), plan 24 packet TK2: SAFT association and SAFT-gamma Mie groups.

PC-SAFT water under two parameterisations with different site schemes (2B and 4C): the association
sites of each scheme with their multiplicities, the bond of each scheme's two site types as a set
of a pair of sites (asserted for one order, read in either), a cross-association pair set between
the sites of two species and a site with no parameters of its own (induced association: its
self-association slots are not applicable). The pairs a scheme does not list (a site with itself)
take the association volume a selection policy states for unasserted pairs, so the association
matrix the expression reads is complete. SAFT-gamma Mie: groups with segment counts and shape
factors, the group-pair exponents and depth of the pairs a source states, a combining rule declared
as a form that a policy names for the pairs it leaves unasserted, association sites on groups and
the bonds between the groups of a molecule.

The numbers are synthetic. The independent calculation of the association is a successive
substitution written here in numpy, and that of the combining rule is the published mean and
geometric-mean expressions.
"""

from __future__ import annotations

import uuid
from collections.abc import Iterator
from dataclasses import dataclass
from pathlib import Path

import numpy as np
import psycopg
import pytest
from hard_case_support import at, build, failing
from mapping_support import carrier, real_declaration, writer

from thermo_knowledge import db, identity
from thermo_knowledge.canonical.values import NotApplicable, Quantity
from thermo_knowledge.canonical.writer import CanonicalWriter, ValidationError
from thermo_knowledge.declaration import Declaration
from thermo_knowledge.expression.compiled import CompileCache
from thermo_knowledge.expression.evaluate import EvaluationRefusal, bind
from thermo_knowledge.qualify.source import DatabaseSource
from thermo_knowledge.testing import TestDatabase


CACHE = CompileCache()
N_A = 6.02214076e23
CARRIER = identity.identifier("source", [carrier("src", "a.json").key])
PAIR = "pcsaft_association.pair"
ANGSTROM = 1e-10

# -- the synthetic fluids -----------------------------------------------------------------------

WATER = {  # scheme: segment parameters, the bond of the scheme's two site types, sites (label: multiplicity)
    "2B": dict(m=1.05, sigma=3.0, eps=280.0, kappa=0.035, eps_ab=2400.0, sites={"H": 1, "e": 1}),
    "4C": dict(m=1.9, sigma=2.6, eps=200.0, kappa=0.04, eps_ab=1800.0, sites={"H": 2, "e": 2}),
}
POINTS = [(T, rho) for T in (280.0, 320.0, 400.0) for rho in (2_000.0, 15_000.0, 35_000.0, 48_000.0)]

# -- SAFT-gamma Mie -----------------------------------------------------------------------------

GROUPS = {  # code: (label, nu*, S, sigma in angstrom, epsilon/k in K, lambda_r, lambda_a)
    "1": ("CH3", 1, 0.55, 4.1, 250.0, 15.0, 6.0),
    "2": ("CH2", 1, 0.45, 4.6, 300.0, 19.0, 6.0),
    "3": ("OH", 1, 0.40, 3.3, 500.0, 24.0, 6.0),
}
UNLIKE = {("1", "3"): (3.7, 16.5, 6.0, 420.0)}  # stated: (sigma in angstrom, lambda_r, lambda_a, epsilon/k in K)
MOLECULES = {  # name: (group counts, bonds between groups)
    "ethanol": ({"1": 1, "2": 1, "3": 1}, {("1", "2"): 1, ("2", "3"): 1}),
    "hexane": ({"1": 2, "2": 4}, {("1", "2"): 2, ("2", "2"): 3}),  # a bond of a group with itself
}
OH_SITES = {"H": 1, "e": 2}


# -- the independent calculation ----------------------------------------------------------------


def association(T: float, rho: float, p: dict[str, object]) -> tuple[float, float]:
    """a_assoc and the mean unbonded fraction of a pure fluid by successive substitution over the
    two site types of the scheme bonded only with each other."""
    m, sigma, eps = p["m"], p["sigma"] * ANGSTROM, p["eps"]  # type: ignore[operator]
    d = sigma * (1 - 0.12 * np.exp(-3 * eps / T))  # type: ignore[operator]
    rho_n = N_A * rho
    zeta2 = np.pi / 6 * rho_n * m * d**2  # type: ignore[operator]
    zeta3 = np.pi / 6 * rho_n * m * d**3  # type: ignore[operator]
    g = 1 / (1 - zeta3) + 1.5 * d * zeta2 / (1 - zeta3) ** 2 + 0.5 * (d * zeta2) ** 2 / (1 - zeta3) ** 3
    delta = d**3 * g * p["kappa"] * (np.exp(p["eps_ab"] / T) - 1)  # type: ignore[operator]
    n = p["sites"]  # type: ignore[assignment]
    labels = list(n)  # type: ignore[arg-type]
    other = {labels[0]: labels[1], labels[1]: labels[0]}
    X = {a: 0.5 for a in labels}
    for _ in range(20_000):
        new = {a: 1 / (1 + rho_n * n[other[a]] * X[other[a]] * delta) for a in labels}  # type: ignore[index]
        if max(abs(new[a] - X[a]) for a in labels) < 1e-16:
            X = new
            break
        X = {a: 0.5 * X[a] + 0.5 * new[a] for a in labels}
    a_assoc = sum(n[a] * (np.log(X[a]) - X[a] / 2 + 0.5) for a in labels)  # type: ignore[index]
    unbonded = sum(n[a] * X[a] for a in labels) / sum(n.values())  # type: ignore[union-attr,index]
    return float(a_assoc), float(unbonded)


def mie_combining(k: str, l: str) -> tuple[float, float, float, float]:
    """sigma, lambda_r, lambda_a and epsilon of an unlike pair of groups, in SI and kelvin."""
    _, _, _, s_k, e_k, r_k, a_k = GROUPS[k]
    _, _, _, s_l, e_l, r_l, a_l = GROUPS[l]
    sigma = (s_k + s_l) / 2
    epsilon = np.sqrt(s_k**3 * s_l**3) / sigma**3 * np.sqrt(e_k * e_l)
    return (
        sigma * ANGSTROM,
        3 + np.sqrt((r_k - 3) * (r_l - 3)),
        3 + np.sqrt((a_k - 3) * (a_l - 3)),
        float(epsilon),
    )


# -- the fixture --------------------------------------------------------------------------------


@dataclass
class World:
    decl: Declaration
    database: TestDatabase
    ids: dict[str, uuid.UUID]


def conventions(w: CanonicalWriter, key: str) -> uuid.UUID:
    return w.kind(
        "convention_set",
        {
            "key": key,
            "revision": "1",
            "temperature_scale": "its_90",
            "avogadro_constant": Quantity(N_A, "1/mol"),
        },
        origins=at(f"conventions-{key}"),
    )


def write_water(w: CanonicalWriter, ids: dict[str, uuid.UUID], water: uuid.UUID, scheme_key: str) -> None:
    p = WATER[scheme_key]
    scheme = w.kind("site_scheme", {"key": scheme_key}, origins=at(f"scheme-{scheme_key}"))
    parameterization = w.kind(
        "parameterization",
        {
            "key": f"pcsaft-water-{scheme_key}",
            "revision": "1",
            "title": f"PC-SAFT water, scheme {scheme_key}",
            "coherence": "independent_records",
            "convention_set": conventions(w, f"pcsaft-water-{scheme_key}"),
            "site_scheme": scheme,
        },
        origins=at(f"parameterization-{scheme_key}"),
    )
    ids[f"water_{scheme_key}"] = parameterization
    w.parameter_set(
        parameterization=parameterization,
        slot_group="pcsaft_association.pure",
        subjects=[water],
        slots={"m": p["m"], "sigma": Quantity(p["sigma"], "angstrom"), "epsilon_over_k": Quantity(p["eps"], "K")},
        origins=at(f"pure-{scheme_key}"),
    )
    sites = {
        label: w.kind(
            "association_site",
            {
                "scheme": scheme,
                "label": label,
                "on_entity": water,
                "multiplicity": count,
            },
            origins=at(f"site-{scheme_key}-{label}"),
        )
        for label, count in p["sites"].items()
    }
    ids.update({f"site_{scheme_key}_{label}": site for label, site in sites.items()})
    # the source lists the bond of the two site types once, in the order e then H for the scheme 4C
    first, second = ("H", "e") if scheme_key == "2B" else ("e", "H")
    ids[f"bond_{scheme_key}"] = w.parameter_set(
        parameterization=parameterization,
        slot_group=PAIR,
        subjects=[sites[first], sites[second]],
        slots={"kappa": p["kappa"], "epsilon_over_k": Quantity(p["eps_ab"], "K")},
        origins=at(f"bond-{scheme_key}"),
    )
    ids["policy"] = ids.get("policy") or w.kind(
        "selection_policy",
        {
            "key": "unlisted-pairs-do-not-associate",
            "revision": "1",
            "unasserted": "stated_default",
            "scope_slot_group": PAIR,
        },
        origins=at("policy-unlisted"),
    )


def write_world(w: CanonicalWriter, ids: dict[str, uuid.UUID]) -> None:
    water, co2 = (w.kind("species", {"canonical_key": n, "label": n}, origins=at(f"s-{n}")) for n in ("water", "CO2"))
    ids.update(water=water, co2=co2)
    write_water(w, ids, water, "2B")
    write_water(w, ids, water, "4C")
    for slot, value in (("kappa", 0.0), ("epsilon_over_k", Quantity(0.0, "K"))):
        w.relation(
            "policy_default",
            {"policy": ids["policy"], "slot": f"{PAIR}.{slot}"},
            {"state": "known", "value": value},
            at="a.json#/default",
        )
    # cross association of water with CO2 (induced): CO2 has one acceptor site with no parameters of its own
    cross = w.kind(
        "parameterization",
        {
            "key": "pcsaft-cross",
            "revision": "1",
            "title": "cross association of water and CO2",
            "coherence": "independent_records",
            "convention_set": conventions(w, "pcsaft-cross"),
        },
        origins=at("parameterization-cross"),
    )
    ids["cross"] = cross
    scheme = w.kind("site_scheme", {"key": "induced"}, origins=at("scheme-induced"))
    site = w.kind(
        "association_site",
        {"scheme": scheme, "label": "e", "on_entity": co2, "multiplicity": 1},
        origins=at("site-co2"),
    )
    ids["site_co2"] = site
    w.parameter_set(
        parameterization=cross,
        slot_group="pcsaft_association.pure",
        subjects=[co2],
        slots={"m": 2.0, "sigma": Quantity(3.0, "angstrom"), "epsilon_over_k": Quantity(150.0, "K")},
        origins=at("pure-co2"),
    )
    ids["parameterless"] = w.parameter_set(
        parameterization=cross,
        slot_group=PAIR,
        subjects=[site, site],
        slots={"kappa": NotApplicable(), "epsilon_over_k": NotApplicable()},
        origins=at("parameterless"),
    )
    ids["cross_pair"] = w.parameter_set(
        parameterization=cross,
        slot_group=PAIR,
        subjects=[ids["site_2B_H"], site],  # the sites of two species, under two schemes
        slots={"kappa": 0.02, "epsilon_over_k": Quantity(1200.0, "K")},
        origins=at("cross"),
    )
    write_saftgamma(w, ids)


def write_saftgamma(w: CanonicalWriter, ids: dict[str, uuid.UUID]) -> None:
    scheme = w.kind("group_scheme", {"key": "saftgamma-mie", "revision": "1", "role": "equation_of_state"}, origins=at("gscheme"))
    groups = {
        code: w.kind(
            "group", {"scheme": scheme, "code": code, "label": label, "role": "group"}, origins=at(f"group-{code}")
        )
        for code, (label, *_rest) in GROUPS.items()
    }
    ids.update({f"group_{code}": g for code, g in groups.items()})
    p = w.kind(
        "parameterization",
        {
            "key": "saftgamma-mie",
            "revision": "1",
            "title": "SAFT-gamma Mie groups",
            "coherence": "jointly_fitted",
            "group_scheme": scheme,
        },
        origins=at("parameterization-saftgamma"),
    )
    ids["saftgamma"] = p
    for code, (_, nu_star, S, sigma, eps, lam_r, lam_a) in GROUPS.items():
        w.parameter_set(
            parameterization=p,
            slot_group="saftgamma_mie.group",
            subjects=[groups[code]],
            slots={
                "nu_star": nu_star,
                "S": S,
                "sigma": Quantity(sigma, "angstrom"),
                "epsilon_over_k": Quantity(eps, "K"),
                "lambda_r": lam_r,
                "lambda_a": lam_a,
            },
            origins=at(f"like-{code}"),
        )
        w.parameter_set(  # the combining rule reads the like-group values from its own set
            parameterization=p,
            slot_group="mie_group_combining.group",
            subjects=[groups[code]],
            slots={
                "sigma": Quantity(sigma, "angstrom"),
                "lambda_r": lam_r,
                "lambda_a": lam_a,
                "epsilon": Quantity(eps, "K"),
            },
            origins=at(f"combining-{code}"),
        )
    for (k, l), (sigma, lam_r, lam_a, eps) in UNLIKE.items():
        ids[f"unlike_{k}{l}"] = w.parameter_set(
            parameterization=p,
            slot_group="mie_group_pair_table.pair",
            subjects=[groups[k], groups[l]],
            slots={
                "sigma": Quantity(sigma, "angstrom"),
                "lambda_r": lam_r,
                "lambda_a": lam_a,
                "epsilon": Quantity(eps, "K"),
            },
            origins=at(f"unlike-{k}{l}"),
        )
    ids["rule_policy"] = w.kind(
        "selection_policy",
        {
            "key": "mie-combine-unasserted-group-pairs",
            "revision": "1",
            "unasserted": "named_rule",
            "rule_form": "mie_group_combining",
            "scope_slot_group": "mie_group_pair_table.pair",
        },
        origins=at("policy-combining"),
    )
    w.relation(
        "policy_precedence", {"policy": ids["rule_policy"], "parameterization": p}, {"value": 1}, at="a.json#/rank"
    )
    # association on the OH group, bonded between its hydrogen and its two electron-donor sites
    sites_scheme = w.kind("site_scheme", {"key": "saftgamma-oh"}, origins=at("scheme-oh"))
    oh = {
        label: w.kind(
            "association_site",
            {
                "scheme": sites_scheme,
                "label": label,
                "on_group": groups["3"],
                "multiplicity": count,
            },
            origins=at(f"site-oh-{label}"),
        )
        for label, count in OH_SITES.items()
    }
    ids.update({f"site_oh_{label}": s for label, s in oh.items()})
    ids["oh_bond"] = w.parameter_set(
        parameterization=p,
        slot_group="saftgamma_mie.association",
        subjects=[oh["H"], oh["e"]],
        slots={"epsilon_hb_over_k": Quantity(1800.0, "K"), "bonding_volume": Quantity(120.0, "angstrom**3")},
        origins=at("oh-bond"),
    )
    for name, (counts, bonds) in MOLECULES.items():
        species = w.kind("species", {"canonical_key": name, "label": name}, origins=at(f"s-{name}"))
        ids[f"species_{name}"] = species
        assignment = w.kind(
            "group_assignment",
            {"entity": species, "scheme": scheme, "asserted_by": CARRIER, "origin": "published"},
            origins=at(f"assignment-{name}"),
        )
        ids[f"assignment_{name}"] = assignment
        for code, count in counts.items():
            w.relation("group_count", {"assignment": assignment, "group": groups[code]}, {"value": count}, at="a.json#/count")
        for (k, l), count in bonds.items():
            w.relation(
                "group_bond_count",
                {"assignment": assignment, "first": groups[k], "second": groups[l]},
                {"value": count},
                at="a.json#/bond",
            )


@pytest.fixture(scope="module")
def decl() -> Declaration:
    return real_declaration()


@pytest.fixture(scope="module")
def world(decl: Declaration, tmp_path_factory: pytest.TempPathFactory) -> Iterator[World]:
    ids: dict[str, uuid.UUID] = {}
    database = build(tmp_path_factory.mktemp("saft"), lambda w: write_world(w, ids), decl)
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


def multiplicity(conn: psycopg.Connection, site: uuid.UUID) -> int:
    return int(scalar(conn, "SELECT multiplicity FROM tk.association_site WHERE id = %s", site))  # type: ignore[call-overload]


def water_bound(world: World, conn: psycopg.Connection, scheme_key: str, *, policy: bool = True):  # noqa: ANN201
    source = DatabaseSource(
        conn, world.decl, [world.ids[f"water_{scheme_key}"]], policy=world.ids["policy"] if policy else None
    )
    sites = [str(world.ids[f"site_{scheme_key}_{label}"]) for label in WATER[scheme_key]["sites"]]  # type: ignore[attr-defined]
    return bind(
        world.decl, "pcsaft_association", source=source, roles={"i": str(world.ids["water"])}, sets={"sites": sites}, cache=CACHE
    ), sites


def association_outputs(world: World, conn: psycopg.Connection, scheme_key: str) -> dict[str, np.ndarray]:
    found, sites = water_bound(world, conn, scheme_key)
    T, rho = (np.array(v) for v in zip(*POINTS, strict=True))
    n = {s: np.full(T.shape, float(multiplicity(conn, uuid.UUID(s)))) for s in sites}
    return {o: np.asarray(found.evaluate(o, T=T, rho=rho, n=n), dtype=float).reshape(-1) for o in ("a_assoc", "unbonded")}


# -- the structure ------------------------------------------------------------------------------


def test_the_fixture_satisfies_every_invariant(conn: psycopg.Connection) -> None:
    assert failing(conn) == {}


def test_water_has_two_parameterisations_with_different_site_schemes_and_multiplicities(
    world: World, conn: psycopg.Connection
) -> None:
    rows = conn.execute(
        "SELECT p.key, s.key, a.label, a.multiplicity FROM tk.parameterization p "
        "JOIN tk.site_scheme s ON s.id = p.site_scheme JOIN tk.association_site a ON a.scheme = s.id "
        "WHERE p.key LIKE 'pcsaft-water-%'"
    ).fetchall()
    assert sorted(rows) == [
        ("pcsaft-water-2B", "2B", "H", 1),
        ("pcsaft-water-2B", "2B", "e", 1),
        ("pcsaft-water-4C", "4C", "H", 2),
        ("pcsaft-water-4C", "4C", "e", 2),
    ]
    # the same species and the same labels: a site is told apart by its scheme
    assert scalar(conn, "SELECT count(DISTINCT on_entity) FROM tk.association_site WHERE scheme IN (SELECT id FROM tk.site_scheme WHERE key IN ('2B', '4C'))") == 1
    assert world.ids["site_2B_H"] != world.ids["site_4C_H"]


def test_the_bond_of_a_scheme_is_one_set_of_a_pair_of_sites_stored_for_the_order_asserted(
    world: World, conn: psycopg.Connection
) -> None:
    """Symmetric: one canonical orientation and no arrangement, however the source ordered the pair."""
    for scheme_key in WATER:
        a, b, kappa, eps = conn.execute(
            'SELECT a, b, kappa, epsilon_over_k FROM param."pcsaft_association__pair" WHERE id = %s',
            (world.ids[f"bond_{scheme_key}"],),
        ).fetchone()  # type: ignore[misc]
        sites = {world.ids[f"site_{scheme_key}_H"], world.ids[f"site_{scheme_key}_e"]}
        assert {a, b} == sites and str(a) <= str(b)
        assert (kappa, eps) == (WATER[scheme_key]["kappa"], WATER[scheme_key]["eps_ab"])
    columns = {
        r[0] for r in conn.execute("SELECT column_name FROM information_schema.columns WHERE table_name = 'pcsaft_association__pair'")
    }
    assert "arrangement" not in columns


def test_a_cross_association_pair_joins_the_sites_of_two_species(world: World, conn: psycopg.Connection) -> None:
    a, b = conn.execute(
        'SELECT a, b FROM param."pcsaft_association__pair" WHERE id = %s', (world.ids["cross_pair"],)
    ).fetchone()  # type: ignore[misc]
    carriers = {
        scalar(conn, "SELECT on_entity FROM tk.association_site WHERE id = %s", site) for site in (a, b)
    }
    assert carriers == {world.ids["water"], world.ids["co2"]}


def test_a_site_without_parameters_of_its_own_holds_a_set_whose_slots_are_not_applicable(
    world: World, conn: psycopg.Connection
) -> None:
    row = conn.execute(
        'SELECT kappa, kappa__state::text, epsilon_over_k__state::text FROM param."pcsaft_association__pair" WHERE id = %s',
        (world.ids["parameterless"],),
    ).fetchone()
    assert row == (None, "not_applicable", "not_applicable")


# -- the association of water against numpy -----------------------------------------------------


@pytest.mark.parametrize("scheme_key", list(WATER))
def test_the_site_fractions_of_the_scheme_match_a_successive_substitution(
    world: World, conn: psycopg.Connection, scheme_key: str
) -> None:
    found = association_outputs(world, conn, scheme_key)
    want = np.array([association(T, rho, WATER[scheme_key]) for T, rho in POINTS])
    np.testing.assert_allclose(found["a_assoc"], want[:, 0], rtol=1e-9)
    np.testing.assert_allclose(found["unbonded"], want[:, 1], rtol=1e-9)
    assert np.all(found["unbonded"] < 1.0) and np.all(found["unbonded"] > 0.0)


def test_the_closed_form_of_the_two_site_scheme_agrees() -> None:
    """The control of the reference: two site types of one site each bonded only with each other
    have X = (sqrt(1 + 4 rho Delta) - 1) / (2 rho Delta)."""
    p = WATER["2B"]
    T, rho = 320.0, 15_000.0
    _, unbonded = association(T, rho, p)
    d = p["sigma"] * ANGSTROM * (1 - 0.12 * np.exp(-3 * p["eps"] / T))  # type: ignore[operator]
    rho_n = N_A * rho
    z2, z3 = (np.pi / 6 * rho_n * p["m"] * d**k for k in (2, 3))  # type: ignore[operator]
    g = 1 / (1 - z3) + 1.5 * d * z2 / (1 - z3) ** 2 + 0.5 * (d * z2) ** 2 / (1 - z3) ** 3
    delta = d**3 * g * p["kappa"] * (np.exp(p["eps_ab"] / T) - 1)  # type: ignore[operator]
    closed = (np.sqrt(1 + 4 * rho_n * delta) - 1) / (2 * rho_n * delta)
    assert unbonded == pytest.approx(closed, rel=1e-12)


def test_the_two_schemes_give_different_fractions_for_the_same_species(world: World, conn: psycopg.Connection) -> None:
    two, four = association_outputs(world, conn, "2B"), association_outputs(world, conn, "4C")
    assert np.all(np.abs(two["unbonded"] - four["unbonded"]) > 1e-3)


def test_the_pair_is_read_in_either_order_of_its_sites(world: World, conn: psycopg.Connection) -> None:
    """The scheme 4C bond was asserted as (e, H) and stored once; the sites given as [H, e] or as
    [e, H] read the same set."""
    T, rho = np.array([320.0]), np.array([15_000.0])
    results = []
    for scheme_key in ("4C",):
        found, sites = water_bound(world, conn, scheme_key)
        for order in (sites, sites[::-1]):
            source = DatabaseSource(conn, world.decl, [world.ids[f"water_{scheme_key}"]], policy=world.ids["policy"])
            bound = bind(
                world.decl, "pcsaft_association", source=source, roles={"i": str(world.ids["water"])}, sets={"sites": order}
            )
            n = {s: np.array([float(multiplicity(conn, uuid.UUID(s)))]) for s in order}
            results.append(float(bound.evaluate("a_assoc", T=T, rho=rho, n=n)[0]))
    assert results[0] == pytest.approx(results[1], rel=1e-12)


def test_the_unlisted_pairs_take_the_association_volume_the_policy_states(world: World, conn: psycopg.Connection) -> None:
    """A site with itself is listed by no source: under the policy it does not associate, and
    without a policy the evaluation refuses, naming the slot group."""
    found, sites = water_bound(world, conn, "2B", policy=False)
    T, rho = np.array([320.0]), np.array([15_000.0])
    with pytest.raises(EvaluationRefusal, match="pcsaft_association.pair"):
        found.evaluate("a_assoc", T=T, rho=rho, n={s: np.array([1.0]) for s in sites})
    source = DatabaseSource(conn, world.decl, [world.ids["water_2B"]], policy=world.ids["policy"])
    subjects = (str(world.ids["site_2B_H"]), str(world.ids["site_2B_H"]))
    assert source.slot_values(PAIR, subjects) is None
    assert source.default_slot_values(PAIR, subjects) == {"kappa": 0.0, "epsilon_over_k": 0.0}


def test_a_fluid_whose_only_site_has_no_parameters_cannot_be_evaluated_pure(world: World, conn: psycopg.Connection) -> None:
    """Induced association needs the other species: the site's own pair holds no value, and the
    policy's defaults are for pairs the source does not list, not for a pair it lists as not applicable."""
    source = DatabaseSource(conn, world.decl, [world.ids["cross"]], policy=world.ids["policy"])
    site = str(world.ids["site_co2"])
    found = bind(world.decl, "pcsaft_association", source=source, roles={"i": str(world.ids["co2"])}, sets={"sites": [site]})
    with pytest.raises(EvaluationRefusal, match="holds no value for slot"):
        found.evaluate("a_assoc", T=np.array([320.0]), rho=np.array([15_000.0]), n={site: np.array([1.0])})


# -- SAFT-gamma Mie -----------------------------------------------------------------------------


def test_the_groups_carry_segment_counts_shape_factors_and_the_mie_potential(world: World, conn: psycopg.Connection) -> None:
    rows = conn.execute(
        'SELECT g.code, l.nu_star, l."S", l.sigma, l.epsilon_over_k, l.lambda_r, l.lambda_a '
        'FROM param."saftgamma_mie__group" l JOIN tk."group" g ON g.id = l.k ORDER BY g.code'
    ).fetchall()
    assert rows == [
        (code, nu, S, pytest.approx(sigma * ANGSTROM), eps, r, a) for code, (_, nu, S, sigma, eps, r, a) in GROUPS.items()
    ]


def test_the_stated_unlike_pair_is_one_symmetric_set_and_the_other_pairs_are_unasserted(
    world: World, conn: psycopg.Connection
) -> None:
    (sigma, lam_r, lam_a, eps) = UNLIKE[("1", "3")]
    row = conn.execute(
        'SELECT sigma, lambda_r, lambda_a, epsilon FROM param."mie_group_pair_table__pair" WHERE id = %s',
        (world.ids["unlike_13"],),
    ).fetchone()
    assert row == (pytest.approx(sigma * ANGSTROM), lam_r, lam_a, eps)
    assert scalar(conn, 'SELECT count(*) FROM param."mie_group_pair_table__pair"') == 1
    assert scalar(conn, "SELECT count(*) FROM param.\"mie_group_combining__group\"") == len(GROUPS)


def test_a_policy_names_the_combining_rule_for_the_pairs_the_source_leaves_unasserted(
    world: World, conn: psycopg.Connection
) -> None:
    row = conn.execute(
        "SELECT p.unasserted::text, f.name, sg.qualified_name FROM tk.selection_policy p "
        "JOIN meta.form f ON f.id = p.rule_form JOIN meta.slot_group sg ON sg.id = p.scope_slot_group WHERE p.id = %s",
        (world.ids["rule_policy"],),
    ).fetchone()
    assert row == ("named_rule", "mie_group_combining", "mie_group_pair_table.pair")


@pytest.mark.parametrize("pair", [("1", "2"), ("2", "3"), ("2", "1"), ("3", "1")])
def test_the_combining_rule_gives_the_published_mean_exponents_and_depth(
    world: World, conn: psycopg.Connection, pair: tuple[str, str]
) -> None:
    k, l = pair
    source = DatabaseSource(conn, world.decl, [world.ids["saftgamma"]])
    bound = bind(
        world.decl,
        "mie_group_combining",
        source=source,
        sets={"groups": [str(world.ids[f"group_{k}"]), str(world.ids[f"group_{l}"])]},
    )
    found = [float(np.asarray(bound.evaluate(o)).reshape(-1)[0]) for o in ("sigma", "lambda_r", "lambda_a", "epsilon")]
    assert found == pytest.approx(mie_combining(k, l), rel=1e-12)


def test_the_stated_unlike_pair_differs_from_what_the_rule_would_give(world: World, conn: psycopg.Connection) -> None:
    sigma, lam_r, lam_a, eps = UNLIKE[("1", "3")]
    ruled = mie_combining("1", "3")
    assert abs(eps - ruled[3]) > 1.0 and abs(lam_r - ruled[1]) > 0.1
    source = DatabaseSource(conn, world.decl, [world.ids["saftgamma"]])
    subjects = (str(world.ids["group_3"]), str(world.ids["group_1"]))  # asked in the other order
    stated = source.slot_values("mie_group_pair_table.pair", subjects)
    assert stated == pytest.approx({"sigma": sigma * ANGSTROM, "lambda_r": lam_r, "lambda_a": lam_a, "epsilon": eps})
    unasserted = (str(world.ids["group_1"]), str(world.ids["group_2"]))
    assert source.slot_values("mie_group_pair_table.pair", unasserted) is None
    assert source.default_slot_values("mie_group_pair_table.pair", unasserted) is None  # the rule is applied by a selection, not the source


def test_association_sites_sit_on_a_group_with_their_multiplicities_and_bond_parameters(
    world: World, conn: psycopg.Connection
) -> None:
    rows = conn.execute(
        "SELECT a.label, a.multiplicity, g.label FROM tk.association_site a JOIN tk.\"group\" g ON g.id = a.on_group "
    ).fetchall()
    assert sorted(rows) == [("H", 1, "OH"), ("e", 2, "OH")]
    row = conn.execute(
        'SELECT epsilon_hb_over_k, bonding_volume FROM param."saftgamma_mie__association" WHERE id = %s', (world.ids["oh_bond"],)
    ).fetchone()
    assert row == (1800.0, pytest.approx(120.0 * ANGSTROM**3))


def test_a_molecule_has_group_counts_and_bonds_between_its_groups_including_a_group_with_itself(
    world: World, conn: psycopg.Connection
) -> None:
    for name, (counts, bonds) in MOLECULES.items():
        held = dict(
            conn.execute(
                'SELECT g.code, c.value FROM tk.group_count c JOIN tk."group" g ON g.id = c."group" WHERE c.assignment = %s',
                (world.ids[f"assignment_{name}"],),
            ).fetchall()
        )
        assert held == counts
        rows = conn.execute(
            'SELECT g1.code, g2.code, b.value FROM tk.group_bond_count b JOIN tk."group" g1 ON g1.id = b."first" '
            'JOIN tk."group" g2 ON g2.id = b."second" WHERE b.assignment = %s',
            (world.ids[f"assignment_{name}"],),
        ).fetchall()
        assert sorted((tuple(sorted((a, b))), v) for a, b, v in rows) == sorted(bonds.items())


# -- what the model refuses ---------------------------------------------------------------------


def fresh(decl: Declaration) -> CanonicalWriter:
    return writer(decl)


def test_a_site_on_an_entity_and_a_group_at_once_or_on_neither_is_refused(decl: Declaration) -> None:
    w = fresh(decl)
    species = w.kind("species", {"canonical_key": "s", "label": "s"}, origins=at("s"))
    scheme = w.kind("group_scheme", {"key": "g", "revision": "1", "role": "equation_of_state"}, origins=at("gs"))
    group = w.kind("group", {"scheme": scheme, "code": "1", "label": "g", "role": "group"}, origins=at("g"))
    site_scheme = w.kind("site_scheme", {"key": "k"}, origins=at("k"))
    base = {"scheme": site_scheme, "label": "H", "multiplicity": 1}
    with pytest.raises(ValidationError, match="the one of on_entity, on_group that is present, and 2 are"):
        w.kind("association_site", {**base, "on_entity": species, "on_group": group}, origins=at("both"))
    with pytest.raises(ValidationError, match="the one of on_entity, on_group that is present, and 0 are"):
        w.kind("association_site", base, origins=at("neither"))
    with pytest.raises(ValidationError, match="multiplicity"):
        w.kind("association_site", {**base, "on_entity": species, "multiplicity": -1}, origins=at("negative"))


def test_the_verify_checks_flag_a_carrier_key_that_names_another_carrier_and_a_bond_of_an_uncounted_group(
    decl: Declaration, tmp_path: Path
) -> None:
    def emit(w: CanonicalWriter) -> None:
        a, b = (w.kind("species", {"canonical_key": n, "label": n}, origins=at(f"s-{n}")) for n in "ab")
        scheme = w.kind("site_scheme", {"key": "k"}, origins=at("k"))
        w.kind(
            "association_site",
            {"scheme": scheme, "label": "H", "on_entity": a, "multiplicity": 1},
            origins=at("site"),
        )
        gscheme = w.kind("group_scheme", {"key": "g", "revision": "1", "role": "equation_of_state"}, origins=at("gs"))
        first, second = (
            w.kind("group", {"scheme": gscheme, "code": c, "label": c, "role": "group"}, origins=at(f"g-{c}")) for c in "12"
        )
        assignment = w.kind(
            "group_assignment",
            {"entity": a, "scheme": gscheme, "asserted_by": CARRIER, "origin": "published"},
            origins=at("assignment"),
        )
        w.relation("group_count", {"assignment": assignment, "group": first}, {"value": 1}, at="a.json#/count")
        w.relation(
            "group_bond_count", {"assignment": assignment, "first": first, "second": second}, {"value": 1}, at="a.json#/bond"
        )

    database = build(tmp_path, emit, decl)
    try:
        with psycopg.connect(database.url, autocommit=True) as connection:
            assert failing(connection) == {"group_bond_count.groups_in_assignment": 1}
            # the writer computes the carrier key, so the wrong one is put in the row directly
            connection.execute("UPDATE tk.association_site SET carrier_key = 'somebody-else'")
            assert failing(connection) == {
                "association_site.carrier_key_matches_carrier": 1,
                "group_bond_count.groups_in_assignment": 1,
            }
    finally:
        database.remove()
