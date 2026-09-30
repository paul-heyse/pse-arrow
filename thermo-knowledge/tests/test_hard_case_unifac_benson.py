# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Hard case (d), plan 24 packet TK2: UNIFAC matrices and RMG Benson trees.

UNIFAC: two schemes (original and Dortmund), each with subgroups identified by the scheme's own
code, main groups as partition classes the subgroups point at, R and Q for each subgroup, the
interaction of main groups as ordered pairs (one constant in the original scheme, a0, a1 and a2 in
Dortmund) and the group assignments of molecules. The Dortmund scheme has two subgroups that share a
label.

Benson trees: a hierarchical scheme with parents and sibling positions, a node that holds no data
(an instruction to the estimator, which the scheme's `dataless_rule` names), a node whose value is a
pointer to another node, and group values that one fit produced, with the standard uncertainties of
its free parameters and the correlations between them. Two schemes show the two rules for a node
without data: the nearest ancestor with data, and the average of the children.

The numbers are synthetic. The independent calculations are the published combinatorial terms of
original and Dortmund UNIFAC, the exponential interaction factors and the sums over groups, written
in numpy.
"""

from __future__ import annotations

import uuid
from collections.abc import Iterator
from dataclasses import dataclass
from pathlib import Path

import numpy as np
import psycopg
import pytest
from hard_case_support import CHECKS, at, build, failing
from mapping_support import carrier, real_declaration, writer

from thermo_knowledge import db, identity
from thermo_knowledge.canonical.values import Quantity, Redirect
from thermo_knowledge.canonical.writer import CanonicalWriter, FamilyRow, ValidationError
from thermo_knowledge.declaration import Declaration
from thermo_knowledge.expression.compiled import CompileCache
from thermo_knowledge.expression.evaluate import bind
from thermo_knowledge.qualify.source import DatabaseSource
from thermo_knowledge.testing import TestDatabase
from thermo_knowledge.verify.run import run_check

CACHE = CompileCache()
CARRIER = identity.identifier("source", [carrier("src", "a.json").key])

# -- UNIFAC -------------------------------------------------------------------------------------

# (code, label, main group code, R, Q); a main group has the code "M<number>" and is its own class
ORIGINAL = {
    "mains": {"M1": "CH2", "M5": "OH"},
    "subgroups": [("1", "CH3", "M1", 0.91, 0.86), ("2", "CH2", "M1", 0.66, 0.52), ("14", "OH", "M5", 1.05, 1.15)],
    "a": {("M1", "M5"): 986.5, ("M5", "M1"): 156.4},
    "molecules": {"ethanol": {"1": 1, "2": 1, "14": 1}, "propan-1-ol": {"1": 1, "2": 2, "14": 1}},
}
DORTMUND = {
    "mains": {"M1": "CH2", "M5": "OH", "M21": "CHO-a", "M22": "CHO-b"},
    "subgroups": [
        ("1", "CH3", "M1", 0.89, 0.83),
        ("2", "CH2", "M1", 0.64, 0.50),
        ("14", "OH", "M5", 1.02, 1.12),
        ("40", "CHO", "M21", 0.97, 0.94),  # two subgroups of two main groups that share one label
        ("41", "CHO", "M22", 1.20, 0.98),
    ],
    "a": {  # (a0 in K, a1, a2 in 1/K)
        ("M1", "M5"): (700.0, -0.80, 0.0012),
        ("M5", "M1"): (250.0, 0.35, -0.0004),
        ("M1", "M1"): (0.0, 0.0, 0.0),  # the diagonal is allowed and stated here
        ("M1", "M21"): (90.0, 0.10, 0.0001),
    },
    "molecules": {"ethanol": {"1": 1, "2": 1, "14": 1}, "acetaldehyde": {"1": 1, "40": 1}},
}
MIXTURE = {"ethanol": 0.35, "propan-1-ol": 0.65}
MIXTURE_DORTMUND = {"ethanol": 0.6, "acetaldehyde": 0.4}


def sizes(scheme: dict, molecule: str) -> tuple[float, float]:  # type: ignore[type-arg]
    counts = scheme["molecules"][molecule]
    by_code = {code: (r, q) for code, _, _, r, q in scheme["subgroups"]}
    return (
        sum(n * by_code[c][0] for c, n in counts.items()),
        sum(n * by_code[c][1] for c, n in counts.items()),
    )


def combinatorial(x: np.ndarray, r: np.ndarray, q: np.ndarray, *, dortmund: bool) -> float:
    """The mole-fraction sum of ln gamma^C, written from the published expressions."""
    V = r / np.sum(x * r)
    F = q / np.sum(x * q)
    V_first = r**0.75 / np.sum(x * r**0.75) if dortmund else V
    ln_gamma = 1 - V_first + np.log(V_first) - 5 * q * (1 - V / F + np.log(V / F))
    return float(np.sum(x * ln_gamma))


# -- Benson trees -------------------------------------------------------------------------------

# code: (parent, position, pattern, value) with value None (no data), a pointer (a code) or (H298 in J/mol, S298 in J/(mol K))
GROUP_TREE = {
    "C": (None, None, "C", (-1000.0, 150.0)),
    "Cs": ("C", 1, "Cs", None),
    "Cs-HHH": ("Cs", 1, "H H H", (-42_000.0, 127.0)),
    "Cs-CsHH": ("Cs", 2, "H H", "Cs-HHH"),  # a pointer to the values of another node
    "Cs-CsCsH": ("Cs", 3, "H", (-7_500.0, -50.0)),
    "Cd": ("C", 2, "Cd", (26_000.0, 33.0)),
}
RING_TREE = {
    "Ring": (None, None, "ring", None),
    "Cyclopropane": ("Ring", 1, "r3", (115_000.0, 130.0)),
    "Cyclobutane": ("Ring", 2, "r4", (110_000.0, 120.0)),
}
CP_GRID = (300.0, 500.0, 1000.0)
CP = {"C": (8.0, 10.0, 12.0), "Cs-HHH": (25.0, 32.0, 43.0), "Cs-CsCsH": (4.0, 6.5, 9.0), "Cd": (20.0, 26.0, 33.0)}
# the parameter vector of the fit: (node, slot, standard uncertainty); correlations by position
FIT_PARAMETERS = [
    ("Cs-HHH", "H298", 400.0),
    ("Cs-HHH", "S298", 1.5),
    ("Cs-CsCsH", "H298", 520.0),
    ("Cs-CsCsH", "S298", 2.0),
    ("Cd", "H298", 610.0),
]
CORRELATIONS = {(1, 2): 0.85, (1, 3): -0.2, (3, 4): 0.6, (2, 5): 0.1}
MOLECULE = {"Cs-HHH": 2, "Cs-CsHH": 1, "Cd": 1}


@dataclass
class World:
    decl: Declaration
    database: TestDatabase
    ids: dict[str, uuid.UUID]


def write_unifac(w: CanonicalWriter, ids: dict[str, uuid.UUID], key: str, scheme_data: dict, name: str) -> None:  # type: ignore[type-arg]
    scheme = w.kind(
        "group_scheme", {"key": name, "revision": "1", "role": "activity"}, origins=at(f"scheme-{name}")
    )
    ids[f"scheme_{key}"] = scheme
    mains = {
        code: w.kind(
            "group",
            {"scheme": scheme, "code": code, "label": label, "role": "partition_class"},
            origins=at(f"{key}-main-{code}"),
        )
        for code, label in scheme_data["mains"].items()
    }
    groups = {
        code: w.kind(
            "group",
            {"scheme": scheme, "code": code, "label": label, "role": "group", "partition_class": mains[main]},
            origins=at(f"{key}-sub-{code}"),
        )
        for code, label, main, _, _ in scheme_data["subgroups"]
    }
    ids.update({f"{key}_main_{c}": g for c, g in mains.items()})
    ids.update({f"{key}_group_{c}": g for c, g in groups.items()})
    p = w.kind(
        "parameterization",
        {
            "key": f"{name}-parameters",
            "revision": "1",
            "title": name,
            "coherence": "jointly_fitted",
            "group_scheme": scheme,
            "convention_set": w.kind(
                "convention_set",
                {"key": f"{name}-conventions", "revision": "1", "temperature_scale": "its_90"},
                origins=at(f"{key}-conventions"),
            ),
        },
        origins=at(f"{key}-parameterization"),
    )
    ids[f"parameterization_{key}"] = p
    form = "unifac_group_sizes"
    for code, _, _, r, q in scheme_data["subgroups"]:
        w.parameter_set(
            parameterization=p,
            slot_group=f"{form}.group",
            subjects=[groups[code]],
            slots={"R": r, "Q": q},
            origins=at(f"{key}-sizes-{code}"),
        )
    for (m, n), value in scheme_data["a"].items():
        if key == "original":
            slots = {"a": Quantity(value, "K")}
            group = "unifac_interaction_original.main"
        else:
            a0, a1, a2 = value
            slots = {"a0": Quantity(a0, "K"), "a1": a1, "a2": Quantity(a2, "1/K")}
            group = "unifac_interaction_dortmund.main"
        ids[f"{key}_a_{m}_{n}"] = w.parameter_set(
            parameterization=p,
            slot_group=group,
            subjects=[mains[m], mains[n]],
            slots=slots,
            origins=at(f"{key}-a-{m}-{n}"),
        )
    for molecule, counts in scheme_data["molecules"].items():
        species = ids.get(f"species_{molecule}") or w.kind(
            "species", {"canonical_key": molecule, "label": molecule}, origins=at(f"s-{molecule}")
        )
        ids[f"species_{molecule}"] = species
        assignment = w.kind(
            "group_assignment",
            {"entity": species, "scheme": scheme, "asserted_by": CARRIER, "origin": "published"},
            origins=at(f"{key}-assignment-{molecule}"),
        )
        ids[f"{key}_assignment_{molecule}"] = assignment
        for code, count in counts.items():
            w.relation("group_count", {"assignment": assignment, "group": groups[code]}, {"value": count}, at="a.json#/count")


def write_tree(
    w: CanonicalWriter, ids: dict[str, uuid.UUID], key: str, tree: dict, role: str, rule: str, p: uuid.UUID | None  # type: ignore[type-arg]
) -> tuple[uuid.UUID, uuid.UUID]:
    scheme = w.kind(
        "group_scheme",
        {"key": f"rmg-{key}", "revision": "1", "role": role, "dataless_rule": rule},
        origins=at(f"scheme-{key}"),
    )
    ids[f"scheme_{key}"] = scheme
    parameterization = w.kind(
        "parameterization",
        {
            "key": f"rmg-{key}-values",
            "revision": "1",
            "title": key,
            "coherence": "independent_records",
            "group_scheme": scheme,
        },
        origins=at(f"parameterization-{key}"),
    )
    ids[f"parameterization_{key}"] = parameterization
    nodes: dict[str, uuid.UUID] = {}
    for code, (parent, position, pattern, _) in tree.items():  # parents come before their children
        nodes[code] = w.kind(
            "group",
            {
                "scheme": scheme,
                "code": code,
                "label": code,
                "role": "group",
                "pattern": pattern,
                **({} if parent is None else {"parent": nodes[parent]}),
                **({} if position is None else {"position": position}),
            },
            origins=at(f"{key}-node-{code}"),
        )
    ids.update({f"{key}_node_{c}": n for c, n in nodes.items()})
    return scheme, parameterization


def write_values(
    w: CanonicalWriter, ids: dict[str, uuid.UUID], key: str, tree: dict, parameterization: uuid.UUID, fit: uuid.UUID | None  # type: ignore[type-arg]
) -> None:
    sets: dict[str, uuid.UUID] = {}
    data = [c for c, (*_, v) in tree.items() if isinstance(v, tuple)]
    for code in data + [c for c, (*_, v) in tree.items() if isinstance(v, str)]:
        value = tree[code][3]
        if isinstance(value, str):
            slots = {"H298": Redirect(sets[value]), "S298": Redirect(sets[value])}
            families = None
        else:
            slots = {"H298": Quantity(value[0], "J/mol"), "S298": Quantity(value[1], "J/(mol*K)")}
            families = (
                {"cp": [FamilyRow({"n": n}, {"T": Quantity(T, "K"), "value": Quantity(c, "J/(mol*K)")}) for n, (T, c) in enumerate(zip(CP_GRID, CP[code], strict=True), 1)]}
                if code in CP
                else None
            )
        fitted = fit is not None and any(node == code for node, _, _ in FIT_PARAMETERS)
        sets[code] = w.parameter_set(
            parameterization=parameterization,
            slot_group="benson_group_additivity.node",
            subjects=[ids[f"{key}_node_{code}"]],
            slots=slots,
            families=families,
            origins=at(f"{key}-values-{code}", "fitted" if fitted else "published"),
        )
        if fitted:
            assert fit is not None
            w.relation("derivation_output", {"derivation": fit, "record": sets[code]}, at="a.json#/output")
    ids.update({f"{key}_set_{c}": s for c, s in sets.items()})


def write_world(w: CanonicalWriter, ids: dict[str, uuid.UUID]) -> None:
    write_unifac(w, ids, "original", ORIGINAL, "unifac-original")
    write_unifac(w, ids, "dortmund", DORTMUND, "unifac-dortmund")
    _, group_p = write_tree(w, ids, "group", GROUP_TREE, "additive_increment", "nearest_ancestor", None)
    _, ring_p = write_tree(w, ids, "ring", RING_TREE, "correction_tree", "average_of_children", None)
    fit = w.kind("fit", {"key": "benson-fit", "kind": "fit", "outcome": "converged"}, origins=at("fit", "derived"))
    ids["fit"] = fit
    write_values(w, ids, "group", GROUP_TREE, group_p, fit)
    write_values(w, ids, "ring", RING_TREE, ring_p, None)
    for ordinal, (node, slot, uncertainty) in enumerate(FIT_PARAMETERS, 1):
        unit = "J/mol" if slot == "H298" else "J/(mol*K)"
        w.relation(
            "fit_free_parameter",
            {"fit": fit, "ordinal": ordinal},
            {
                "record": ids[f"group_set_{node}"],
                "slot": f"benson_group_additivity.node.{slot}",
                "standard_uncertainty": Quantity(uncertainty, unit),
            },
        )
    for (a, b), coefficient in CORRELATIONS.items():
        w.relation("fit_correlation", {"fit": fit, "a": a, "b": b}, {"coefficient": coefficient})
    species = w.kind("species", {"canonical_key": "benson-molecule", "label": "molecule"}, origins=at("s-benson"))
    ids["benson_molecule"] = species


@pytest.fixture(scope="module")
def decl() -> Declaration:
    return real_declaration()


@pytest.fixture(scope="module")
def world(decl: Declaration, tmp_path_factory: pytest.TempPathFactory) -> Iterator[World]:
    ids: dict[str, uuid.UUID] = {}
    database = build(tmp_path_factory.mktemp("unifac-benson"), lambda w: write_world(w, ids), decl)
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


def test_the_fixture_satisfies_every_invariant(conn: psycopg.Connection) -> None:
    assert failing(conn) == {}


# -- UNIFAC: identity, partition classes, assignments --------------------------------------------


def test_subgroups_are_identified_by_code_with_main_groups_as_the_classes_they_point_at(
    world: World, conn: psycopg.Connection
) -> None:
    rows = conn.execute(
        'SELECT s.code, s.label, m.code, m.label FROM tk."group" s JOIN tk."group" m ON m.id = s.partition_class '
        "WHERE s.scheme = %s ORDER BY s.code",
        (world.ids["scheme_original"],),
    ).fetchall()
    assert rows == [
        ("1", "CH3", "M1", "CH2"),
        ("14", "OH", "M5", "OH"),
        ("2", "CH2", "M1", "CH2"),
    ]
    roles = dict(conn.execute('SELECT code, role::text FROM tk."group" WHERE scheme = %s', (world.ids["scheme_original"],)).fetchall())
    assert {roles[c] for c in ORIGINAL["mains"]} == {"partition_class"} and roles["1"] == "group"


def test_two_subgroups_of_one_scheme_may_share_a_label_and_are_two_groups(world: World, conn: psycopg.Connection) -> None:
    rows = conn.execute(
        'SELECT id, code FROM tk."group" WHERE scheme = %s AND label = %s ORDER BY code',
        (world.ids["scheme_dortmund"], "CHO"),
    ).fetchall()
    assert rows == [(world.ids["dortmund_group_40"], "40"), (world.ids["dortmund_group_41"], "41")]
    # the same code in the same scheme is the same group: the identity is (scheme, code)
    w = writer(world.decl)
    scheme = w.kind("group_scheme", {"key": "s", "revision": "1", "role": "activity"}, origins=at("s"))
    one = w.kind("group", {"scheme": scheme, "code": "40", "label": "CHO", "role": "group"}, origins=at("a"))
    again = w.kind("group", {"scheme": scheme, "code": "40", "label": "CHO", "role": "group"}, origins=at("a"))
    other = w.kind("group", {"scheme": scheme, "code": "41", "label": "CHO", "role": "group"}, origins=at("b"))
    assert one == again and one != other


def test_the_same_molecule_has_an_assignment_in_each_scheme(world: World, conn: psycopg.Connection) -> None:
    rows = conn.execute(
        "SELECT s.key, count(*) FROM tk.group_assignment a JOIN tk.group_scheme s ON s.id = a.scheme "
        "WHERE a.entity = %s GROUP BY s.key",
        (world.ids["species_ethanol"],),
    ).fetchall()
    assert sorted(rows) == [("unifac-dortmund", 1), ("unifac-original", 1)]


def test_the_ordered_pairs_of_main_groups_are_two_facts_and_the_diagonal_is_held(
    world: World, conn: psycopg.Connection
) -> None:
    rows = conn.execute(
        'SELECT m.code, n.code, a.a0, a.a1, a.a2 FROM param."unifac_interaction_dortmund__main" a '
        'JOIN tk."group" m ON m.id = a.m JOIN tk."group" n ON n.id = a.n ORDER BY m.code, n.code'
    ).fetchall()
    expected = sorted((m, n, *v) for (m, n), v in DORTMUND["a"].items())
    assert rows == expected
    assert ("M1", "M1", 0.0, 0.0, 0.0) in rows  # the pair of a main group with itself
    columns = {
        r[0] for r in conn.execute("SELECT column_name FROM information_schema.columns WHERE table_name = 'unifac_interaction_dortmund__main'")
    }
    assert "arrangement" not in columns


# -- UNIFAC against numpy -----------------------------------------------------------------------


def size_bound(world: World, conn: psycopg.Connection, key: str, molecule: str, order: list[str]):  # noqa: ANN201
    """The size form bound to the groups of the scheme taken in `order`, and the counts of the
    molecule's assignment for them (a group it has no count for occurs zero times)."""
    counts = dict(
        conn.execute(
            'SELECT g.code, c.value FROM tk.group_count c JOIN tk."group" g ON g.id = c."group" WHERE c.assignment = %s',
            (world.ids[f"{key}_assignment_{molecule}"],),
        ).fetchall()
    )
    members = {code: str(world.ids[f"{key}_group_{code}"]) for code in order}
    source = DatabaseSource(conn, world.decl, [world.ids[f"parameterization_{key}"]])
    bound = bind(
        world.decl,
        "unifac_group_sizes",
        source=source,
        roles={"i": str(world.ids[f"species_{molecule}"])},
        sets={"groups": list(members.values())},
        cache=CACHE,
    )
    return bound, {members[code]: np.array([float(counts.get(code, 0))]) for code in order}


@pytest.mark.parametrize(
    "key, molecule",
    [("original", "ethanol"), ("original", "propan-1-ol"), ("dortmund", "ethanol"), ("dortmund", "acetaldehyde")],
)
def test_r_and_q_are_the_sums_of_the_group_counts_times_the_subgroup_parameters(
    world: World, conn: psycopg.Connection, key: str, molecule: str
) -> None:
    scheme = ORIGINAL if key == "original" else DORTMUND
    order = [code for code, *_ in scheme["subgroups"]]
    want_r, want_q = sizes(scheme, molecule)
    for arrangement in (order, order[::-1]):
        bound, nu = size_bound(world, conn, key, molecule, arrangement)
        assert float(bound.evaluate("r", nu=nu)[0]) == pytest.approx(want_r, rel=1e-13)
        assert float(bound.evaluate("q", nu=nu)[0]) == pytest.approx(want_q, rel=1e-13)


@pytest.mark.parametrize("form, dortmund", [("unifac_combinatorial_original", False), ("unifac_combinatorial_dortmund", True)])
def test_the_combinatorial_term_matches_the_published_expression(
    world: World, conn: psycopg.Connection, form: str, dortmund: bool
) -> None:
    scheme, key, composition = (DORTMUND, "dortmund", MIXTURE_DORTMUND) if dortmund else (ORIGINAL, "original", MIXTURE)
    names = list(composition)
    x = np.array([composition[n] for n in names])
    size = [sizes(scheme, n) for n in names]
    r, q = np.array([s[0] for s in size]), np.array([s[1] for s in size])
    want = combinatorial(x, r, q, dortmund=dortmund)
    source = DatabaseSource(conn, world.decl, [world.ids[f"parameterization_{key}"]])
    bound = bind(
        world.decl,
        form,
        source=source,
        sets={"components": [str(world.ids[f"species_{n}"]) for n in names]},
        cache=CACHE,
    )
    ids = [str(world.ids[f"species_{n}"]) for n in names]
    found = bound.evaluate(
        "gE_over_RT",
        x={i: np.array([v]) for i, v in zip(ids, x, strict=True)},
        r={i: np.array([v]) for i, v in zip(ids, r, strict=True)},
        q={i: np.array([v]) for i, v in zip(ids, q, strict=True)},
    )
    assert float(np.asarray(found).reshape(-1)[0]) == pytest.approx(want, rel=1e-12)


def test_the_two_combinatorial_terms_differ_for_the_same_sizes() -> None:
    x = np.array([0.3, 0.7])
    r, q = np.array([2.6, 4.1]), np.array([2.3, 3.4])
    assert abs(combinatorial(x, r, q, dortmund=True) - combinatorial(x, r, q, dortmund=False)) > 1e-4


def test_the_interaction_factors_of_the_ordered_pairs_match_numpy_and_differ_between_the_orders(
    world: World, conn: psycopg.Connection
) -> None:
    T = np.array([283.15, 313.15, 353.15])
    source = DatabaseSource(conn, world.decl, [world.ids["parameterization_original"]])

    def original(m: str, n: str) -> np.ndarray:
        bound = bind(
            world.decl,
            "unifac_interaction_original",
            source=source,
            roles={"m": str(world.ids[f"original_main_{m}"]), "n": str(world.ids[f"original_main_{n}"])},
        )
        return np.asarray(bound.evaluate("psi", T=T), dtype=float).reshape(-1)

    np.testing.assert_allclose(original("M1", "M5"), np.exp(-986.5 / T), rtol=1e-14)
    np.testing.assert_allclose(original("M5", "M1"), np.exp(-156.4 / T), rtol=1e-14)
    source = DatabaseSource(conn, world.decl, [world.ids["parameterization_dortmund"]])
    for (m, n), (a0, a1, a2) in DORTMUND["a"].items():
        bound = bind(
            world.decl,
            "unifac_interaction_dortmund",
            source=source,
            roles={"m": str(world.ids[f"dortmund_main_{m}"]), "n": str(world.ids[f"dortmund_main_{n}"])},
        )
        np.testing.assert_allclose(
            np.asarray(bound.evaluate("psi", T=T), dtype=float).reshape(-1), np.exp(-(a0 + a1 * T + a2 * T**2) / T), rtol=1e-13
        )


# -- Benson trees -------------------------------------------------------------------------------


def children(conn: psycopg.Connection, parent: uuid.UUID) -> list[tuple[str, str]]:
    """The children of a node in the order the scheme gives them: (code, pattern)."""
    return [
        (code, pattern)
        for code, pattern in conn.execute(
            'SELECT code, pattern FROM tk."group" WHERE parent = %s ORDER BY position', (parent,)
        ).fetchall()
    ]


def descend(conn: psycopg.Connection, root: uuid.UUID, environment: set[str]) -> str:
    """The node a fragment matches: from the root, the first child (by position) whose pattern
    needs only what the fragment has, repeatedly."""
    current = root
    code = scalar(conn, 'SELECT code FROM tk."group" WHERE id = %s', root)
    while True:
        for child_code, pattern in children(conn, current):
            if set(str(pattern).split()) <= environment:
                code = child_code
                current = scalar(conn, 'SELECT id FROM tk."group" WHERE parent = %s AND code = %s', current, child_code)  # type: ignore[assignment]
                break
        else:
            return str(code)


def test_the_tree_has_parents_and_sibling_positions_from_one(world: World, conn: psycopg.Connection) -> None:
    rows = conn.execute(
        'SELECT c.code, p.code, c."position" FROM tk."group" c LEFT JOIN tk."group" p ON p.id = c.parent '
        "WHERE c.scheme = %s ORDER BY c.code",
        (world.ids["scheme_group"],),
    ).fetchall()
    assert rows == sorted((code, parent, position) for code, (parent, position, *_rest) in GROUP_TREE.items())
    assert [c for c, _ in children(conn, world.ids["group_node_Cs"])] == ["Cs-HHH", "Cs-CsHH", "Cs-CsCsH"]


def test_sibling_order_decides_which_node_a_fragment_matches(world: World, conn: psycopg.Connection) -> None:
    """A fragment of a saturated carbon with hydrogens fits every child of `Cs`; it matches the
    first by position. With the positions of the first two children exchanged it matches the
    other: the stored order is part of the scheme."""
    fragment = {"Cs", "H"}
    assert descend(conn, world.ids["group_node_C"], fragment) == "Cs-HHH"
    conn.execute(
        'UPDATE tk."group" SET "position" = CASE code WHEN \'Cs-HHH\' THEN 2 ELSE 1 END '
        "WHERE scheme = %s AND parent = %s AND code IN ('Cs-HHH', 'Cs-CsHH')",
        (world.ids["scheme_group"], world.ids["group_node_Cs"]),
    )
    assert descend(conn, world.ids["group_node_C"], fragment) == "Cs-CsHH"  # the connection rolls the exchange back


@pytest.mark.parametrize("key, rule", [("group", "nearest_ancestor"), ("ring", "average_of_children")])
def test_a_scheme_states_what_an_estimator_does_at_a_node_without_data(
    world: World, conn: psycopg.Connection, key: str, rule: str
) -> None:
    assert scalar(conn, "SELECT dataless_rule::text FROM tk.group_scheme WHERE id = %s", world.ids[f"scheme_{key}"]) == rule


def value_of(conn: psycopg.Connection, world: World, key: str, code: str, rule: str) -> float:
    """H298 of a node under the scheme's rule: its own (or its pointer's) value, else what the rule says."""
    found = conn.execute(
        'SELECT h."H298" FROM param."benson_group_additivity__node" h JOIN tk.parameter_set s ON s.id = h.id '
        "WHERE h.k = %s",
        (world.ids[f"{key}_node_{code}"],),
    ).fetchone()
    if found is not None and found[0] is not None:
        return float(found[0])
    redirected = conn.execute(
        'SELECT t."H298" FROM param."benson_group_additivity__node" h JOIN param."benson_group_additivity__node" t '
        'ON t.id = h."H298__redirect" WHERE h.k = %s',
        (world.ids[f"{key}_node_{code}"],),
    ).fetchone()
    if redirected is not None:
        return float(redirected[0])
    if rule == "nearest_ancestor":
        parent = scalar(conn, 'SELECT p.code FROM tk."group" c JOIN tk."group" p ON p.id = c.parent WHERE c.id = %s', world.ids[f"{key}_node_{code}"])
        return value_of(conn, world, key, str(parent), rule)
    kids = [c for c, _ in children(conn, world.ids[f"{key}_node_{code}"])]
    return float(np.mean([value_of(conn, world, key, c, rule) for c in kids]))


def test_a_node_without_data_holds_no_set_and_the_rule_of_its_scheme_gives_its_value(
    world: World, conn: psycopg.Connection
) -> None:
    for key, node, expected in (
        ("group", "Cs", GROUP_TREE["C"][3][0]),  # the nearest ancestor with data
        ("ring", "Ring", np.mean([RING_TREE["Cyclopropane"][3][0], RING_TREE["Cyclobutane"][3][0]])),  # the children
    ):
        assert scalar(
            conn,
            'SELECT count(*) FROM param."benson_group_additivity__node" WHERE k = %s',
            world.ids[f"{key}_node_{node}"],
        ) == 0
        rule = str(scalar(conn, "SELECT dataless_rule::text FROM tk.group_scheme WHERE id = %s", world.ids[f"scheme_{key}"]))
        assert value_of(conn, world, key, node, rule) == pytest.approx(float(expected), rel=1e-14)


def test_a_pointer_is_a_redirect_to_the_values_of_another_node(world: World, conn: psycopg.Connection) -> None:
    row = conn.execute(
        'SELECT "H298", "H298__state"::text, "H298__redirect" FROM param."benson_group_additivity__node" WHERE id = %s',
        (world.ids["group_set_Cs-CsHH"],),
    ).fetchone()
    assert row == (None, "redirect", world.ids["group_set_Cs-HHH"])


def test_the_additivity_sum_follows_a_pointer_and_matches_numpy(world: World, conn: psycopg.Connection) -> None:
    order = list(MOLECULE)
    members = [str(world.ids[f"group_node_{c}"]) for c in order]
    source = DatabaseSource(conn, world.decl, [world.ids["parameterization_group"]])
    bound = bind(
        world.decl,
        "benson_group_additivity",
        source=source,
        roles={"i": str(world.ids["benson_molecule"])},
        sets={"groups": members},
        cache=CACHE,
    )
    nu = {m: np.array([float(MOLECULE[c])]) for m, c in zip(members, order, strict=True)}
    resolved = {"Cs-HHH": "Cs-HHH", "Cs-CsHH": "Cs-HHH", "Cd": "Cd"}
    h = sum(MOLECULE[c] * GROUP_TREE[resolved[c]][3][0] for c in order)
    s = sum(MOLECULE[c] * GROUP_TREE[resolved[c]][3][1] for c in order)
    assert float(bound.evaluate("H298", nu=nu)[0]) == pytest.approx(h, rel=1e-13)
    assert float(bound.evaluate("S298", nu=nu)[0]) == pytest.approx(s, rel=1e-13)


def test_the_heat_capacity_table_of_a_node_is_a_family_of_temperature_and_value(world: World, conn: psycopg.Connection) -> None:
    rows = conn.execute(
        'SELECT n, "T", value FROM param."benson_group_additivity__node__cp" WHERE set_id = %s ORDER BY n',
        (world.ids["group_set_Cs-HHH"],),
    ).fetchall()
    assert rows == [(n, T, c) for n, (T, c) in enumerate(zip(CP_GRID, CP["Cs-HHH"], strict=True), 1)]


def test_one_fit_produced_the_group_values_with_its_uncertainties_and_correlations(world: World, conn: psycopg.Connection) -> None:
    produced = {
        scalar(conn, 'SELECT k FROM param."benson_group_additivity__node" WHERE id = %s', record) for (record,) in conn.execute(
            "SELECT record FROM prov.derivation_output WHERE derivation = %s", (world.ids["fit"],)
        ).fetchall()
    }
    assert produced == {world.ids[f"group_node_{node}"] for node, _, _ in FIT_PARAMETERS}
    rows = conn.execute(
        "SELECT f.ordinal, sl.qualified_name, f.standard_uncertainty, f.record FROM prov.fit_free_parameter f "
        "JOIN meta.slot sl ON sl.id = f.slot WHERE f.fit = %s ORDER BY f.ordinal",
        (world.ids["fit"],),
    ).fetchall()
    assert rows == [
        (n, f"benson_group_additivity.node.{slot}", u, world.ids[f"group_set_{node}"])
        for n, (node, slot, u) in enumerate(FIT_PARAMETERS, 1)
    ]
    correlations = {
        (a, b): c for a, b, c in conn.execute("SELECT a, b, coefficient FROM prov.fit_correlation WHERE fit = %s", (world.ids["fit"],)).fetchall()
    }
    assert correlations == CORRELATIONS


def test_the_covariance_of_the_fit_is_recovered_from_uncertainties_and_correlations(world: World, conn: psycopg.Connection) -> None:
    sd = np.array([u for *_, u in FIT_PARAMETERS])
    correlation = np.eye(len(sd))
    for (a, b), coefficient in CORRELATIONS.items():
        correlation[a - 1, b - 1] = correlation[b - 1, a - 1] = coefficient
    covariance = np.outer(sd, sd) * correlation
    assert np.allclose(covariance, covariance.T) and np.all(np.linalg.eigvalsh(covariance) > 0)


# -- what the model refuses ---------------------------------------------------------------------


def test_a_sibling_position_below_one_and_a_rule_outside_the_vocabulary_are_refused(decl: Declaration) -> None:
    w = writer(decl)
    scheme = w.kind("group_scheme", {"key": "s", "revision": "1", "role": "correction_tree"}, origins=at("s"))
    root = w.kind("group", {"scheme": scheme, "code": "r", "label": "r", "role": "group"}, origins=at("r"))
    with pytest.raises(ValidationError, match="position_from_one"):
        w.kind("group", {"scheme": scheme, "code": "c", "label": "c", "role": "group", "parent": root, "position": 0}, origins=at("c"))
    with pytest.raises(ValidationError, match="not a member of enum `dataless_node_rule`"):
        w.kind(
            "group_scheme",
            {"key": "t", "revision": "1", "role": "correction_tree", "dataless_rule": "nearest"},
            origins=at("t"),
        )


def test_the_verify_checks_flag_a_parent_or_class_of_another_scheme_a_cycle_and_a_count_of_another_scheme(
    decl: Declaration, tmp_path: Path
) -> None:
    ids: dict[str, uuid.UUID] = {}

    def emit(w: CanonicalWriter) -> None:
        one = w.kind("group_scheme", {"key": "one", "revision": "1", "role": "activity"}, origins=at("one"))
        two = w.kind("group_scheme", {"key": "two", "revision": "1", "role": "activity"}, origins=at("two"))
        own = w.kind("group", {"scheme": one, "code": "a", "label": "a", "role": "group"}, origins=at("a"))
        foreign = w.kind("group", {"scheme": two, "code": "b", "label": "b", "role": "group"}, origins=at("b"))
        ids["foreign_parent"] = w.kind(
            "group", {"scheme": one, "code": "c", "label": "c", "role": "group", "parent": foreign}, origins=at("c")
        )
        ids["foreign_class"] = w.kind(
            "group", {"scheme": one, "code": "d", "label": "d", "role": "group", "partition_class": foreign}, origins=at("d")
        )
        ids["first"] = w.kind("group", {"scheme": one, "code": "e", "label": "e", "role": "group", "parent": own}, origins=at("e"))
        ids["second"] = w.kind(
            "group", {"scheme": one, "code": "f", "label": "f", "role": "group", "parent": ids["first"]}, origins=at("f")
        )
        species = w.kind("species", {"canonical_key": "s", "label": "s"}, origins=at("s"))
        assignment = w.kind(
            "group_assignment",
            {"entity": species, "scheme": one, "asserted_by": CARRIER, "origin": "published"},
            origins=at("assignment"),
        )
        w.relation("group_count", {"assignment": assignment, "group": foreign}, {"value": 1}, at="a.json#/count")

    database = build(tmp_path, emit, decl)
    try:
        with psycopg.connect(database.url, autocommit=True) as connection:
            assert failing(connection) == {
                "group.parent_acyclic_same_scheme": 2,
                "group_count.group_in_assignment_scheme": 1,
            }
            flagged = run_check(connection, CHECKS["group.parent_acyclic_same_scheme"], shown=10)
            assert sorted(flagged.ids) == sorted(str(ids[k]) for k in ("foreign_parent", "foreign_class"))
            # a cycle among the parents of one scheme, made in the row: the writer cannot point a node at a descendant
            connection.execute('UPDATE tk."group" SET parent = %s WHERE id = %s', (ids["second"], ids["first"]))
            assert failing(connection)["group.parent_acyclic_same_scheme"] > 2
    finally:
        database.remove()
