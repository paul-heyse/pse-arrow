# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Hard case (b), plan 24 packet TK2: a cubic equation of state assembled from parts.

A Peng-Robinson core with sub-form slots for the alpha function (Twu), the mixing rule (a
Huron-Vidal rule at infinite pressure, which embeds NRTL through a sub-form slot of its own) and
the volume translation (Peneloux). The parts have their own parameterisations: the critical
constants, the Twu parameters, the NRTL pair sets and the translations. The NRTL pair set is one
record of a pair with its interaction exponent left at the stated default, which the selecting
policy supplies, and it carries a `dependency` on the parameterisation of the alpha function it was
fitted with. The assembly names the form of each slot by path (`mixing/excess` for the NRTL model
below the mixing rule).

The numbers are synthetic. The independent calculation is written in numpy: the Twu alpha, the
published NRTL sums over ordered pairs, the Huron-Vidal constant and the roots of the cubic from
`numpy.roots`.
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
from mapping_support import real_declaration, writer
from thermo_knowledge import db
from thermo_knowledge.canonical.values import Quantity, StatedDefault
from thermo_knowledge.canonical.writer import CanonicalWriter, ValidationError
from thermo_knowledge.declaration import Declaration
from thermo_knowledge.expression.compiled import CompileCache
from thermo_knowledge.expression.evaluate import EvaluationRefusal, bind
from thermo_knowledge.qualify.source import DatabaseSource, SubformBinding
from thermo_knowledge.testing import TestDatabase

CACHE = CompileCache()
R = 8.314462618
ROOT = "peng_robinson_core"

# -- the synthetic fluids -----------------------------------------------------------------------

CRITICAL = {"A": (470.0, 3.4e6), "B": (620.0, 8.1e6)}  # (T_c, P_c)
TWU = {"A": (0.12, 0.87, 2.1), "B": (0.35, 0.90, 1.3)}  # (L, M, N)
TRANSLATION = {"A": 3.0e-6, "B": -1.5e-6}  # m^3/mol
# the pair as its source asserted it: B first, so tau_BA = a12 + b12 / T and tau_AB = a21 + b21 / T
NRTL_ORDER = ("B", "A")
NRTL = dict(a12=0.35, a21=-0.22, b12=120.0, b21=80.0)  # b in K
ALPHA_DEFAULT = 0.3
ALPHA_OTHER = 0.5

POINTS = [(T, P) for T in (300.0, 380.0, 450.0) for P in (1.0e5, 2.0e6, 2.0e7)]
COMPOSITIONS = [{"A": 0.3, "B": 0.7}, {"A": 0.8, "B": 0.2}]

CHOICES = {  # path: (slot, form), as the assembly holds them
    "alpha": ("peng_robinson_core.alpha", "twu_alpha"),
    "mixing": ("peng_robinson_core.mixing", "huron_vidal_pr_mixing"),
    "mixing/excess": ("huron_vidal_pr_mixing.excess", "nrtl_excess_gibbs"),
    "translation": ("peng_robinson_core.translation", "peneloux_translation"),
}
PARAMETERIZATION_OF_FORM = {  # which parameterisation holds the sets of each part
    "peng_robinson_core": "core",
    "twu_alpha": "twu",
    "nrtl_excess_gibbs": "nrtl",
    "peneloux_translation": "peneloux",
}


# -- the independent calculation ----------------------------------------------------------------


def twu_alpha(name: str, T: float) -> float:
    L, M, N = TWU[name]
    Tr = T / CRITICAL[name][0]
    return Tr ** (N * (M - 1)) * np.exp(L * (1 - Tr ** (N * M)))


def tau(first: str, second: str, T: float) -> float:
    """tau_{first,second} of the NRTL pair as the source asserted it."""
    if first == second:
        return 0.0
    if (first, second) == NRTL_ORDER:
        return NRTL["a12"] + NRTL["b12"] / T
    return NRTL["a21"] + NRTL["b21"] / T


def excess_gibbs(T: float, x: dict[str, float], alpha: float) -> float:
    names = list(x)
    total = 0.0
    for i in names:
        numerator = sum(x[j] * tau(j, i, T) * np.exp(-alpha * tau(j, i, T)) for j in names)
        denominator = sum(x[k] * np.exp(-alpha * tau(k, i, T)) for k in names)
        total += x[i] * numerator / denominator
    return R * T * total


def reference(
    T: float, P: float, x: dict[str, float], alpha: float = ALPHA_DEFAULT
) -> dict[str, float]:
    """a, b, the largest and smallest roots above the covolume and their translated volumes."""
    a_pure = {n: 0.45724 * R**2 * CRITICAL[n][0] ** 2 / CRITICAL[n][1] * twu_alpha(n, T) for n in x}
    b_pure = {n: 0.07780 * R * CRITICAL[n][0] / CRITICAL[n][1] for n in x}
    b = sum(x[n] * b_pure[n] for n in x)
    C = np.log((2 + np.sqrt(2)) / (2 - np.sqrt(2))) / (2 * np.sqrt(2))
    a = b * (sum(x[n] * a_pure[n] / b_pure[n] for n in x) - excess_gibbs(T, x, alpha) / C)
    A, B = a * P / (R * T) ** 2, b * P / (R * T)
    roots = np.roots([1.0, -(1 - B), A - 3 * B**2 - 2 * B, -(A * B - B**2 - B**3)])
    real = sorted(
        float(r.real) for r in roots if abs(r.imag) < 1e-9 * max(1.0, abs(r.real)) and r.real > B
    )
    c = sum(x[n] * TRANSLATION[n] for n in x)
    z_vapor, z_liquid = real[-1], real[0]
    return {
        "a": a, "b": b, "Z_vapor": z_vapor, "Z_liquid": z_liquid,
        "v_vapor": z_vapor * R * T / P - c, "v_liquid": z_liquid * R * T / P - c,
    }  # fmt: skip


# -- the fixture --------------------------------------------------------------------------------


@dataclass
class World:
    decl: Declaration
    database: TestDatabase
    ids: dict[str, uuid.UUID]


def parameterization(w: CanonicalWriter, key: str, **attributes: object) -> uuid.UUID:
    conventions = w.kind(
        "convention_set",
        {
            "key": key,
            "revision": "1",
            "temperature_scale": "its_90",
            "gas_constant": Quantity(R, "J/(mol*K)"),
        },
        origins=at(f"conventions-{key}"),
    )
    return w.kind(
        "parameterization",
        {
            "key": key,
            "revision": "1",
            "title": key,
            "coherence": "independent_records",
            "convention_set": conventions,
            **attributes,
        },
        origins=at(f"parameterization-{key}"),
    )


def write_policy(w: CanonicalWriter, key: str, over: uuid.UUID, alpha: float | None) -> uuid.UUID:
    policy = w.kind(
        "selection_policy",
        {"key": key, "revision": "1", "unasserted": "refuse"},
        origins=at(f"policy-{key}"),
    )
    w.relation(
        "policy_precedence",
        {"policy": policy, "parameterization": over},
        {"value": 1},
        at="a.json#/rank",
    )
    if alpha is not None:
        w.relation(
            "policy_default",
            {"policy": policy, "slot": "nrtl_excess_gibbs.pair.alpha"},
            {"state": "known", "value": alpha},
            at="a.json#/default",
        )
    return policy


def write_nrtl_pair(
    w: CanonicalWriter, p: uuid.UUID, species: dict[str, uuid.UUID], tag: str
) -> uuid.UUID:
    first, second = NRTL_ORDER
    return w.parameter_set(
        parameterization=p,
        slot_group="nrtl_excess_gibbs.pair",
        subjects=[species[first], species[second]],  # the order the source asserts
        slots={
            "a12": NRTL["a12"],
            "a21": NRTL["a21"],
            "b12": Quantity(NRTL["b12"], "K"),
            "b21": Quantity(NRTL["b21"], "K"),
            "alpha": StatedDefault(),
        },
        origins=at(f"nrtl-{tag}"),
    )


def write_world(w: CanonicalWriter, ids: dict[str, uuid.UUID]) -> None:
    species = {
        n: w.kind("species", {"canonical_key": n, "label": n}, origins=at(f"s-{n}"))
        for n in CRITICAL
    }
    ids.update({f"species_{n}": s for n, s in species.items()})
    core = parameterization(w, "pr-core")
    twu = parameterization(w, "twu-alpha")
    nrtl = parameterization(w, "nrtl-fitted-with-twu", coherence="conditional")
    peneloux = parameterization(w, "peneloux")
    ids.update(core=core, twu=twu, nrtl=nrtl, peneloux=peneloux)
    for n, (T_c, P_c) in CRITICAL.items():
        w.parameter_set(
            parameterization=core,
            slot_group="peng_robinson_core.pure",
            subjects=[species[n]],
            slots={"Tc": Quantity(T_c, "K"), "Pc": Quantity(P_c, "Pa")},
            origins=at(f"core-{n}"),
        )
        L, M, N = TWU[n]
        w.parameter_set(
            parameterization=twu,
            slot_group="twu_alpha.pure",
            subjects=[species[n]],
            slots={"L": L, "M": M, "N": N},
            origins=at(f"twu-{n}"),
        )
        w.parameter_set(
            parameterization=peneloux,
            slot_group="peneloux_translation.pure",
            subjects=[species[n]],
            slots={"c": Quantity(TRANSLATION[n], "m^3/mol")},
            origins=at(f"peneloux-{n}"),
        )
    ids["pair"] = write_nrtl_pair(w, nrtl, species, "pair")
    # the pair was fitted with the Twu parameterisation in place: it is valid only with it
    w.relation(
        "dependency",
        {"dependent": ids["pair"], "prerequisite": twu},
        {"kind": "fitted_given"},
        at="a.json#/dependency",
    )
    ids["policy"] = write_policy(w, "nrtl-default-alpha", nrtl, ALPHA_DEFAULT)
    ids["policy_other"] = write_policy(w, "nrtl-other-alpha", nrtl, ALPHA_OTHER)
    ids["policy_silent"] = write_policy(w, "core-only", core, None)  # ranks no set at its default
    assembly = w.kind(
        "model_assembly",
        {
            "key": "pr-twu-hv-nrtl-peneloux",
            "revision": "1",
            "title": "PR, Twu, Huron-Vidal with NRTL, Peneloux",
            "root": ROOT,
        },
        origins=at("assembly"),
    )
    ids["assembly"] = assembly
    for path, (slot, form) in CHOICES.items():
        w.kind(
            "assembly_choice",
            {"assembly": assembly, "path": path, "ordinal": 1, "slot": slot, "form": form},
            at="a.json#/choice",
        )


@pytest.fixture(scope="module")
def decl() -> Declaration:
    return real_declaration()


@pytest.fixture(scope="module")
def world(decl: Declaration, tmp_path_factory: pytest.TempPathFactory) -> Iterator[World]:
    ids: dict[str, uuid.UUID] = {}
    database = build(tmp_path_factory.mktemp("cubic"), lambda w: write_world(w, ids), decl)
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


def bindings(world: World, conn: psycopg.Connection) -> dict[str, list[SubformBinding]]:
    """The forms the assembly puts in each sub-form slot, read back out of the database, each with
    the parameterisation that holds its sets."""
    rows = conn.execute(
        "SELECT sl.qualified_name, f.name FROM tk.assembly_choice c "
        "JOIN meta.subform_slot sl ON sl.id = c.slot JOIN meta.form f ON f.id = c.form "
        "WHERE c.assembly = %s ORDER BY c.path, c.ordinal",
        (world.ids["assembly"],),
    ).fetchall()
    found: dict[str, list[SubformBinding]] = {}
    for slot, form in rows:
        held_by = PARAMETERIZATION_OF_FORM.get(form)
        parameterizations = () if held_by is None else (world.ids[held_by],)
        found.setdefault(slot, []).append(SubformBinding(form, parameterizations))
    return found


def cubic(
    world: World, conn: psycopg.Connection, order: tuple[str, ...], policy: str | None = "policy"
):  # noqa: ANN201
    source = DatabaseSource(
        conn,
        world.decl,
        [world.ids["core"]],
        subforms=bindings(world, conn),
        policy=None if policy is None else world.ids[policy],
    )
    members = [str(world.ids[f"species_{n}"]) for n in order]
    return bind(world.decl, ROOT, source=source, sets={"components": members}, cache=CACHE)


def evaluate(
    world: World,
    conn: psycopg.Connection,
    order: tuple[str, ...],
    composition: dict[str, float],
    output: str,
    policy: str | None = "policy",
) -> np.ndarray:
    found = cubic(world, conn, order, policy)
    T, P = (np.array(v) for v in zip(*POINTS, strict=True))
    x = {str(world.ids[f"species_{n}"]): np.full(T.shape, composition[n]) for n in order}
    return np.asarray(found.evaluate(output, T=T, P=P, x=x), dtype=float).reshape(-1)


# -- the structure ------------------------------------------------------------------------------


def test_the_fixture_satisfies_every_invariant(conn: psycopg.Connection) -> None:
    assert failing(conn) == {}


def test_the_core_has_three_sub_form_slots_and_the_mixing_rule_has_one_below_it(
    decl: Declaration,
) -> None:
    core = decl.forms[ROOT]
    assert {(s.name, s.accepts, s.multiplicity, s.per) for s in core.subforms} == {
        ("alpha", "cubic_alpha_function", "one", "model"),
        ("mixing", "cubic_mixing_rule", "one", "model"),
        ("translation", "cubic_volume_translation", "one", "model"),
    }
    (excess,) = decl.forms["huron_vidal_pr_mixing"].subforms
    assert (excess.name, excess.accepts) == ("excess", "molar_excess_gibbs_mixture")
    assert all(
        decl.forms[f].status == "expressed"
        for f in ("twu_alpha", "nrtl_excess_gibbs", "peneloux_translation", ROOT)
    )


def test_the_assembly_names_each_slot_by_its_path_and_the_nested_one_below_the_mixing_rule(
    world: World, conn: psycopg.Connection
) -> None:
    rows = conn.execute(
        "SELECT c.path, sl.qualified_name, f.name FROM tk.assembly_choice c "
        "JOIN meta.subform_slot sl ON sl.id = c.slot JOIN meta.form f ON f.id = c.form "
        "WHERE c.assembly = %s ORDER BY c.path",
        (world.ids["assembly"],),
    ).fetchall()
    assert rows == sorted((path, slot, form) for path, (slot, form) in CHOICES.items())
    assert (
        scalar(
            conn,
            "SELECT f.name FROM tk.model_assembly a JOIN meta.form f ON f.id = a.root WHERE a.id = %s",
            world.ids["assembly"],
        )
        == ROOT
    )


def test_the_nrtl_pair_is_one_record_stored_as_asserted_with_its_alpha_at_the_default(
    world: World, conn: psycopg.Connection
) -> None:
    i, j, arrangement, a12, a21, b12, b21, alpha, state = conn.execute(
        'SELECT i, j, arrangement, a12, a21, b12, b21, alpha, alpha__state::text FROM param."nrtl_excess_gibbs__pair" WHERE id = %s',
        (world.ids["pair"],),
    ).fetchone()  # type: ignore[misc]
    first, second = (world.ids[f"species_{n}"] for n in NRTL_ORDER)
    assert (i, j) == tuple(sorted((first, second), key=str))
    assert arrangement == (0 if first < second else 1)
    assert (a12, a21, b12, b21) == (
        NRTL["a12"],
        NRTL["a21"],
        NRTL["b12"],
        NRTL["b21"],
    )  # never rewritten
    assert (alpha, state) == (None, "stated_default")
    assert scalar(conn, 'SELECT count(*) FROM param."nrtl_excess_gibbs__pair"') == 1


def test_the_pair_depends_on_the_alpha_function_parameterisation_it_was_fitted_with(
    world: World, conn: psycopg.Connection
) -> None:
    row = conn.execute(
        "SELECT prerequisite, kind::text FROM tk.dependency WHERE dependent = %s",
        (world.ids["pair"],),
    ).fetchone()
    assert row == (world.ids["twu"], "fitted_given")
    assert (
        scalar(
            conn, "SELECT coherence::text FROM tk.parameterization WHERE id = %s", world.ids["nrtl"]
        )
        == "conditional"
    )


# -- the assembled model against numpy ----------------------------------------------------------


@pytest.mark.parametrize("composition", COMPOSITIONS)
def test_the_attractive_parameter_with_the_twu_alpha_and_the_excess_gibbs_rule_matches_numpy(
    world: World, conn: psycopg.Connection, composition: dict[str, float]
) -> None:
    want = np.array([reference(T, P, composition)["a"] for T, P in POINTS])
    np.testing.assert_allclose(
        evaluate(world, conn, ("A", "B"), composition, "a"), want, rtol=1e-12
    )
    want_b = np.array([reference(T, P, composition)["b"] for T, P in POINTS])
    np.testing.assert_allclose(
        evaluate(world, conn, ("A", "B"), composition, "b"), want_b, rtol=1e-13
    )


@pytest.mark.parametrize("composition", COMPOSITIONS)
def test_the_compressibility_roots_and_translated_volumes_match_the_roots_of_the_cubic(
    world: World, conn: psycopg.Connection, composition: dict[str, float]
) -> None:
    found = {
        o: evaluate(world, conn, ("A", "B"), composition, o)
        for o in ("Z_vapor", "Z_liquid", "v_vapor", "v_liquid")
    }
    want = [reference(T, P, composition) for T, P in POINTS]
    for output in found:
        np.testing.assert_allclose(
            found[output], [w[output] for w in want], rtol=1e-9, err_msg=output
        )


def test_the_points_include_states_with_one_root_and_with_three(
    world: World, conn: psycopg.Connection
) -> None:
    """The fixture exercises both cases of the block: where the cubic has one real root the two
    selections meet, where it has three they differ."""
    vapor = evaluate(world, conn, ("A", "B"), COMPOSITIONS[0], "Z_vapor")
    liquid = evaluate(world, conn, ("A", "B"), COMPOSITIONS[0], "Z_liquid")
    same = np.isclose(vapor, liquid, rtol=1e-9)
    assert same.any() and (~same).any()
    assert np.all(liquid <= vapor)


def test_the_translation_moves_the_volume_and_not_the_roots(
    world: World, conn: psycopg.Connection
) -> None:
    composition = COMPOSITIONS[0]
    z = evaluate(world, conn, ("A", "B"), composition, "Z_vapor")
    v = evaluate(world, conn, ("A", "B"), composition, "v_vapor")
    T, P = (np.array(x) for x in zip(*POINTS, strict=True))
    c = sum(composition[n] * TRANSLATION[n] for n in composition)
    np.testing.assert_allclose(v, z * R * T / P - c, rtol=1e-12)
    assert abs(c) > 1e-7  # the translation is not negligible against the volumes


def test_the_components_in_either_order_give_the_same_mixture(
    world: World, conn: psycopg.Connection
) -> None:
    """The pair was asserted as (B, A): components as [A, B] read it in the other order, where the
    directed taus change places, and as [B, A] in the order asserted."""
    composition = COMPOSITIONS[0]
    for output in ("a", "Z_vapor", "v_liquid"):
        np.testing.assert_allclose(
            evaluate(world, conn, ("A", "B"), composition, output),
            evaluate(world, conn, ("B", "A"), composition, output),
            rtol=1e-12,
            err_msg=output,
        )


def test_without_the_exchange_of_the_directed_taus_the_attractive_parameter_differs() -> None:
    """The control: reading the pair as asserted in both orders (no exchange) changes gE, so
    the agreement above is the transposition at work."""
    T, x = 350.0, COMPOSITIONS[0]
    swapped = {"a12": NRTL["a21"], "a21": NRTL["a12"], "b12": NRTL["b21"], "b21": NRTL["b12"]}
    kept = dict(NRTL)
    try:
        NRTL.update(swapped)
        other = excess_gibbs(T, x, ALPHA_DEFAULT)
    finally:
        NRTL.update(kept)
    assert abs(other - excess_gibbs(T, x, ALPHA_DEFAULT)) > 1.0


# -- the stated default of the interaction exponent ---------------------------------------------


def test_alpha_takes_the_default_the_selecting_policy_states(
    world: World, conn: psycopg.Connection
) -> None:
    composition = COMPOSITIONS[0]
    default = evaluate(world, conn, ("A", "B"), composition, "a", "policy")
    other = evaluate(world, conn, ("A", "B"), composition, "a", "policy_other")
    want = np.array([reference(T, P, composition, ALPHA_OTHER)["a"] for T, P in POINTS])
    np.testing.assert_allclose(other, want, rtol=1e-12)
    assert np.all(np.abs(default - other) > 0)


def test_without_a_policy_or_under_a_policy_that_states_no_default_the_evaluation_refuses(
    world: World, conn: psycopg.Connection
) -> None:
    for policy in (None, "policy_silent"):
        with pytest.raises(EvaluationRefusal, match="alpha"):
            evaluate(world, conn, ("A", "B"), COMPOSITIONS[0], "a", policy)


# -- what the model refuses ---------------------------------------------------------------------


def test_a_policy_that_ranks_a_parameterisation_with_a_set_at_its_default_and_states_none_is_flagged(
    decl: Declaration, tmp_path: Path
) -> None:
    def emit(w: CanonicalWriter) -> None:
        species = {
            n: w.kind("species", {"canonical_key": n, "label": n}, origins=at(f"s-{n}"))
            for n in "AB"
        }
        nrtl = parameterization(w, "nrtl")
        write_nrtl_pair(w, nrtl, species, "pair")
        write_policy(w, "silent", nrtl, None)

    database = build(tmp_path, emit, decl)
    try:
        with psycopg.connect(database.url) as connection:
            assert failing(connection) == {"set_default_needs_policy_default": 1}
    finally:
        database.remove()


def fresh(decl: Declaration) -> tuple[CanonicalWriter, dict[str, uuid.UUID], uuid.UUID]:
    w = writer(decl)
    species = {
        n: w.kind("species", {"canonical_key": n, "label": n}, origins=at(f"s-{n}")) for n in "AB"
    }
    p = w.kind(
        "parameterization",
        {"key": "p", "revision": "1", "title": "p", "coherence": "independent_records"},
        origins=at("p"),
    )
    return w, species, p


def test_an_nrtl_pair_of_a_component_with_itself_is_refused(decl: Declaration) -> None:
    w, species, p = fresh(decl)
    with pytest.raises(ValidationError, match="forbids the diagonal"):
        w.parameter_set(
            parameterization=p,
            slot_group="nrtl_excess_gibbs.pair",
            subjects=[species["A"], species["A"]],
            slots={
                "a12": 0.0,
                "a21": 0.0,
                "b12": Quantity(0.0, "K"),
                "b21": Quantity(0.0, "K"),
                "alpha": 0.3,
            },
            origins=at("diagonal"),
        )


def test_a_temperature_coefficient_in_the_wrong_dimension_is_refused(decl: Declaration) -> None:
    w, species, p = fresh(decl)
    with pytest.raises(ValidationError):
        w.parameter_set(
            parameterization=p,
            slot_group="nrtl_excess_gibbs.pair",
            subjects=[species["A"], species["B"]],
            slots={
                "a12": 0.0,
                "a21": 0.0,
                "b12": Quantity(1.0, "Pa"),
                "b21": Quantity(0.0, "K"),
                "alpha": 0.3,
            },
            origins=at("dimension"),
        )


def test_a_component_with_no_translation_is_refused_naming_the_slot_group(
    world: World, conn: psycopg.Connection
) -> None:
    """A missing set is not a zero: a mixture with a component the translation parameterisation
    has no set for is refused, although the core and the other parts have one."""
    stranger = uuid.uuid4()
    members = [str(world.ids["species_A"]), str(stranger)]
    source = DatabaseSource(
        conn,
        world.decl,
        [world.ids["core"]],
        subforms=bindings(world, conn),
        policy=world.ids["policy"],
    )
    found = bind(world.decl, ROOT, source=source, sets={"components": members}, cache=CACHE)
    x = {m: np.array([0.5]) for m in members}
    with pytest.raises(EvaluationRefusal, match="peng_robinson_core.pure"):
        found.evaluate("a", T=np.array([350.0]), P=np.array([1.0e6]), x=x)
