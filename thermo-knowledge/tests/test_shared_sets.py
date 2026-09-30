# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""A set that several parents use is stored once and referenced (meta-model section 4.2).

A slot that `references` a contract holds an independently identified, top-level parameter set of
a form implementing it, which any number of sets may reference. The writer names the target by
its identity and checks its form; the verify check `referenced_set_implements_contract` states
the same rule over the database; an expression calls a referenced set like a nested one, and the
parameter sources follow the reference. A sub-form slot chosen per subject takes its form from
the relation `subject_subform_choice` when the qualification case states none.
"""

from __future__ import annotations

import uuid
from collections.abc import Iterator
from dataclasses import dataclass
from pathlib import Path

import numpy as np
import psycopg
import pytest
from build_support import fingerprint, inputs_of, write_source
from mapping_support import origin, writer
from qualify_support import fixture_declaration

from thermo_knowledge import config, db
from thermo_knowledge.build import build_database
from thermo_knowledge.canonical.writer import (
    CanonicalWriter,
    FamilyRow,
    NestedSet,
    SetReference,
    ValidationError,
)
from thermo_knowledge.declaration import Declaration
from thermo_knowledge.expression.evaluate import EvaluationRefusal, bind
from thermo_knowledge.expression.parameters import FormChoice, InMemorySource
from thermo_knowledge.qualify.source import DatabaseSource, SubformBinding
from thermo_knowledge.testing import TestDatabase
from thermo_knowledge.verify.checks import VERIFY_DIR, load_checks
from thermo_knowledge.verify.run import run_check

DEPARTURE = "qfix_departure.core"
PAIR = "qfix_pair_departure.pair"
NAMED = "qfix_named_line.core"
TERMS = "qfix_term_sum_form.param"
CHOICE_SLOT = "qfix_pair_choice.model"

CHECKS = {check.target: check for check in load_checks(config.TREE_DIR / VERIFY_DIR)[0]}

# the departure function: alpha_r = sum n delta^d tau^t
DEPARTURE_TERMS = [(0.8, 1.0, 0.25), (-0.35, 2.5, 1.5), (0.06, 4.0, 3.0)]
SCALES = {("A", "B"): 0.95, ("A", "C"): 1.05, ("B", "C"): 0.6}
LINE = (1250.0, -3.4)  # the named function a + b T
T = np.array([250.0, 300.0, 444.4])
DELTA = np.array([0.2, 0.9, 1.4])
TAU = np.array([1.1, 0.8, 0.5])


@dataclass
class World:
    decl: Declaration
    database: TestDatabase
    ids: dict[str, uuid.UUID]


def fill(w: CanonicalWriter, ids: dict[str, uuid.UUID]) -> None:
    fitted = origin("a.json#/9", "published")
    species = {
        name: w.kind(
            "species", {"canonical_key": name, "label": name}, origins=[origin(f"a.json#/{n}")]
        )
        for n, name in enumerate("ABC")
    }
    ids.update({f"species_{name}": found for name, found in species.items()})

    def parameterization(key: str) -> uuid.UUID:
        found = w.kind(
            "parameterization",
            {"key": key, "revision": "1", "title": key, "coherence": "independent_records"},
            origins=[fitted],
        )
        ids[key] = found
        return found

    gerg, calphad, choices, extra = (
        parameterization(key) for key in ("gerg", "calphad", "choices", "extra")
    )
    # one departure function, the subject of a model component, used by three pairs
    function = w.kind(
        "model_component",
        {"parameterization": gerg, "name": "generalised_departure"},
        origins=[fitted],
    )
    ids["departure"] = w.parameter_set(
        parameterization=gerg,
        slot_group=DEPARTURE,
        subjects=[function],
        slots={},
        families={
            "term": [
                FamilyRow({"k": k}, {"n": n, "d": d, "t": t})
                for k, (n, d, t) in enumerate(DEPARTURE_TERMS, 1)
            ]
        },
        origins=[fitted],
    )
    for (first, second), scale in SCALES.items():
        ids[f"pair_{first}{second}"] = w.parameter_set(
            parameterization=gerg,
            slot_group=PAIR,
            subjects=[species[first], species[second]],
            slots={
                "scale": scale,
                "departure": SetReference(gerg, DEPARTURE, [function]),
            },
            origins=[fitted],
        )
    # a named function used by two parameters, each a sum of literal terms and multiples of it
    line = w.kind(
        "model_component", {"parameterization": calphad, "name": "ghser"}, origins=[fitted]
    )
    ids["line"] = w.parameter_set(
        parameterization=calphad,
        slot_group=NAMED,
        subjects=[line],
        slots={"a": LINE[0], "b": LINE[1]},
        origins=[fitted],
    )
    reference = SetReference(calphad, NAMED, [line])
    for name, literal, named in (
        ("A", [(100.0, 0.0), (-3.0, 1.0)], [1.0]),
        ("B", [(50.0, 0.0)], [2.0, -1.0]),
    ):
        ids[f"term_{name}"] = w.parameter_set(
            parameterization=calphad,
            slot_group=TERMS,
            subjects=[species[name]],
            slots={},
            families={
                "literal": [
                    FamilyRow({"n": n}, {"coefficient": c, "power": p})
                    for n, (c, p) in enumerate(literal, 1)
                ],
                "named": [
                    FamilyRow({"n": n}, {"multiplier": m, "function": reference})
                    for n, m in enumerate(named, 1)
                ],
            },
            origins=[fitted],
        )
    # a form chosen for each pair by a relation: one held by another parameterization
    for form, pair, parameterization_of, coefficient in (
        ("qfix_pair_linear", ("A", "B"), extra, 3.0),
        ("qfix_pair_quadratic", ("A", "C"), choices, 0.5),
    ):
        w.parameter_set(
            parameterization=parameterization_of,
            slot_group=f"{form}.pair",
            subjects=[species[pair[0]], species[pair[1]]],
            slots={"k": coefficient},
            origins=[fitted],
        )
        w.subform_choice(
            parameterization=choices,
            slot=CHOICE_SLOT,
            subjects=[species[pair[0]], species[pair[1]]],
            form=form,
            source_parameterization=None if parameterization_of == choices else parameterization_of,
            at="a.json#/9",
        )


@pytest.fixture(scope="module")
def world(tmp_path_factory: pytest.TempPathFactory) -> Iterator[World]:
    decl = fixture_declaration(tmp_path_factory.mktemp("shared-declaration"))
    canonical = tmp_path_factory.mktemp("shared-canonical")
    ids: dict[str, uuid.UUID] = {}
    write_source(canonical, "src", lambda w: fill(w, ids), decl=decl, declaration=fingerprint(decl))
    with TestDatabase() as database:
        build_database(database.url, decl, inputs_of(canonical))
        yield World(decl, database, ids)


@pytest.fixture
def conn(world: World) -> Iterator[psycopg.Connection]:
    connection = db.connect(world.database.url)
    connection.execute("SELECT 1")  # an open transaction: what a test changes is rolled back
    try:
        yield connection
    finally:
        connection.rollback()
        connection.close()


def source(world: World, conn: psycopg.Connection, *keys: str, **options: object) -> DatabaseSource:
    return DatabaseSource(conn, world.decl, [world.ids[key] for key in keys], **options)  # type: ignore[arg-type]


def close(got: np.ndarray, want: np.ndarray) -> None:
    np.testing.assert_allclose(got, want, rtol=1e-13, atol=0.0)


# -- a departure function stored once, referenced by three pairs ------------------------------------


def departure(delta: np.ndarray, tau: np.ndarray) -> np.ndarray:
    return sum(n * delta**d * tau**t for n, d, t in DEPARTURE_TERMS)  # type: ignore[return-value]


def test_the_shared_function_is_stored_once_and_three_pairs_reference_it(
    world: World, conn: psycopg.Connection
) -> None:
    assert conn.execute("SELECT count(*) FROM param.qfix_departure__core").fetchone() == (1,)
    assert conn.execute("SELECT count(*) FROM param.qfix_departure__core__term").fetchone() == (3,)
    assert conn.execute(
        "SELECT count(DISTINCT departure), count(*) FROM param.qfix_pair_departure__pair"
    ).fetchone() == (1, 3)
    assert conn.execute(
        "SELECT DISTINCT departure FROM param.qfix_pair_departure__pair"
    ).fetchall() == [(world.ids["departure"],)]
    assert conn.execute(
        "SELECT parent FROM tk.parameter_set WHERE id = %s", (world.ids["departure"],)
    ).fetchone() == (None,), "the shared set is top-level"


@pytest.mark.parametrize("swap", [False, True], ids=["as-held", "swapped"])
def test_each_pair_is_evaluated_through_the_shared_function_with_its_own_scale(
    world: World, conn: psycopg.Connection, swap: bool
) -> None:
    found = source(world, conn, "gerg")
    for (first, second), scale in SCALES.items():
        i, j = (second, first) if swap else (first, second)
        roles = {
            "i": str(world.ids[f"species_{i}"]),
            "j": str(world.ids[f"species_{j}"]),
        }
        bound = bind(world.decl, "qfix_pair_departure", source=found, roles=roles)
        close(bound.evaluate("alpha_r", delta=DELTA, tau=TAU), scale * departure(DELTA, TAU))


def test_the_in_memory_source_calls_a_referenced_set_like_a_nested_one() -> None:
    decl = _declaration()
    rows = {(k,): {"n": n, "d": d, "t": t} for k, (n, d, t) in enumerate(DEPARTURE_TERMS, 1)}
    shared = FormChoice(
        "qfix_departure",
        InMemorySource(families={(DEPARTURE, "term", ("fn",)): rows}),
        DEPARTURE,
        ("fn",),
    )
    source_ = InMemorySource(
        slots={(PAIR, ("a", "b")): {"scale": 0.95}},
        nested={(PAIR, "departure", ("a", "b")): shared},
        declaration=decl,
    )
    bound = bind(decl, "qfix_pair_departure", source=source_, roles={"i": "b", "j": "a"})
    close(bound.evaluate("alpha_r", delta=DELTA, tau=TAU), 0.95 * departure(DELTA, TAU))


# -- a named function referenced from two parameters' family rows -----------------------------------


def line(temperature: np.ndarray) -> np.ndarray:
    return LINE[0] + LINE[1] * temperature


def test_the_named_function_is_stored_once_and_referenced_from_the_rows_of_two_parameters(
    world: World, conn: psycopg.Connection
) -> None:
    assert conn.execute("SELECT count(*) FROM param.qfix_named_line__core").fetchone() == (1,)
    assert conn.execute(
        "SELECT count(*), count(DISTINCT function) FROM param.qfix_term_sum_form__param__named"
    ).fetchone() == (3, 1)
    assert conn.execute(
        "SELECT count(DISTINCT set_id) FROM param.qfix_term_sum_form__param__named"
    ).fetchone() == (2,)


def test_a_parameter_is_the_sum_of_literal_terms_and_multiples_of_the_named_function(
    world: World, conn: psycopg.Connection
) -> None:
    found = source(world, conn, "calphad")

    def evaluate(name: str) -> np.ndarray:
        roles = {"i": str(world.ids[f"species_{name}"])}
        return bind(world.decl, "qfix_term_sum_form", source=found, roles=roles).evaluate("g", T=T)

    close(evaluate("A"), 100.0 - 3.0 * T + 1.0 * line(T))
    close(evaluate("B"), 50.0 + 2.0 * line(T) - 1.0 * line(T))


def test_the_in_memory_source_follows_a_reference_held_by_a_family_row() -> None:
    decl = _declaration()
    named = FormChoice(
        "qfix_named_line",
        InMemorySource(slots={(NAMED, ("ghser",)): {"a": LINE[0], "b": LINE[1]}}),
        NAMED,
        ("ghser",),
    )
    source_ = InMemorySource(
        families={
            (TERMS, "literal", ("x",)): {(1,): {"coefficient": 7.0, "power": 2.0}},
            (TERMS, "named", ("x",)): {(1,): {"multiplier": 0.5}},
        },
        nested={(TERMS, "named.function", ("x",), (1,)): named},
    )
    bound = bind(decl, "qfix_term_sum_form", source=source_, roles={"i": "x"})
    close(bound.evaluate("g", T=T), 7.0 * T**2 + 0.5 * line(T))


# -- the writer and the check -----------------------------------------------------------------------


def test_the_writer_refuses_a_target_whose_form_implements_another_contract(
    world: World,
) -> None:
    w = writer(world.decl)
    species = w.kind("species", {"canonical_key": "X", "label": "X"}, origins=[origin()])
    param = w.kind(
        "parameterization",
        {"key": "k", "revision": "r", "title": "t", "coherence": "independent_records"},
        origins=[origin("a.json#/0", "fitted")],
    )
    component = w.kind(
        "model_component", {"parameterization": param, "name": "f"}, origins=[origin()]
    )

    partner = _other(w)

    def write(reference: object) -> None:
        w.parameter_set(
            parameterization=param,
            slot_group=PAIR,
            subjects=[species, partner],
            slots={"scale": 1.0, "departure": reference},
            origins=[origin("a.json#/3", "fitted")],
        )

    with pytest.raises(ValidationError, match=r"implements `qfix_named_function`"):
        write(SetReference(param, NAMED, [component]))
    with pytest.raises(ValidationError, match="not a slot group"):
        write(SetReference(param, "nothing.here", []))
    with pytest.raises(ValidationError, match="1 subject role"):
        write(SetReference(param, DEPARTURE, []))
    with pytest.raises(ValidationError, match="a set-reference slot takes a SetReference"):
        write(NestedSet(DEPARTURE, {}, subjects=[component]))
    assert w.rows("param.qfix_pair_departure__pair") == 0


def _other(w: CanonicalWriter) -> uuid.UUID:
    return w.kind("species", {"canonical_key": "Y", "label": "Y"}, origins=[origin("a.json#/1")])


def test_the_check_passes_on_the_built_database(world: World, conn: psycopg.Connection) -> None:
    assert run_check(conn, CHECKS["referenced_set_implements_contract"]).violations == 0


def test_the_check_flags_a_target_of_a_form_that_implements_another_contract(
    world: World, conn: psycopg.Connection
) -> None:
    conn.execute(
        "UPDATE param.qfix_pair_departure__pair SET departure = %s WHERE id = %s",
        (world.ids["line"], world.ids["pair_AB"]),
    )
    result = run_check(conn, CHECKS["referenced_set_implements_contract"])
    assert result.error is None and result.violations == 1
    assert result.ids == [str(world.ids["pair_AB"])]


def test_the_check_flags_a_target_that_is_a_nested_set(
    world: World, conn: psycopg.Connection
) -> None:
    slot = conn.execute(
        "SELECT id FROM meta.slot WHERE qualified_name = 'qfix_pair_departure.pair.departure'"
    ).fetchone()
    assert slot is not None
    conn.execute(
        "UPDATE tk.parameter_set SET parent = %s, parent_slot = %s WHERE id = %s",
        (world.ids["pair_AB"], slot[0], world.ids["departure"]),
    )
    result = run_check(conn, CHECKS["referenced_set_implements_contract"])
    assert result.error is None and result.violations == 3


def test_the_check_flags_a_family_row_that_references_a_set_of_another_contract(
    world: World, conn: psycopg.Connection
) -> None:
    conn.execute(
        "UPDATE param.qfix_term_sum_form__param__named SET function = %s "
        "WHERE set_id = %s AND n = 1",
        (world.ids["departure"], world.ids["term_A"]),
    )
    result = run_check(conn, CHECKS["referenced_set_implements_contract"])
    assert result.error is None and result.violations == 1
    assert result.ids == [str(world.ids["term_A"])]


def test_the_slot_shape_is_reified_in_meta(world: World, conn: psycopg.Connection) -> None:
    assert conn.execute(
        "SELECT shape, references_contract, accepts, element_kind FROM meta.slot "
        "WHERE qualified_name = 'qfix_pair_departure.pair.departure'"
    ).fetchone() == ("set_reference", "qfix_departure_function", None, None)
    assert conn.execute(
        "SELECT shape, references_contract FROM meta.slot "
        "WHERE qualified_name = 'qfix_term_sum_form.param.named.function'"
    ).fetchone() == ("set_reference", "qfix_named_function")
    assert conn.execute(
        "SELECT count(*) FROM meta.slot WHERE shape = 'set_reference'"
    ).fetchone() == (3,)  # the two slots of the fixtures and the departure slot of the committed multifluid pair form


# -- the form of a sub-form slot chosen for each subject ---------------------------------------------


def pair_ids(world: World, first: str, second: str) -> dict[str, str]:
    return {"i": str(world.ids[f"species_{first}"]), "j": str(world.ids[f"species_{second}"])}


def test_a_per_subject_choice_is_read_from_the_relation(
    world: World, conn: psycopg.Connection
) -> None:
    found = source(world, conn, "choices")
    via_relation = bind(
        world.decl, "qfix_pair_choice", source=found, roles=pair_ids(world, "A", "C")
    )
    close(via_relation.evaluate("y", T=T), 0.5 * T**2)
    # the choice names the parameterization that holds the chosen form's sets
    via_source = bind(
        world.decl, "qfix_pair_choice", source=found, roles=pair_ids(world, "A", "B")
    )
    close(via_source.evaluate("y", T=T), 3.0 * T)


def test_a_case_s_explicit_choice_takes_precedence_over_the_relation(
    world: World, conn: psycopg.Connection
) -> None:
    explicit = source(
        world,
        conn,
        "choices",
        subforms={CHOICE_SLOT: [SubformBinding("qfix_pair_quadratic", (world.ids["choices"],))]},
    )
    bound = bind(world.decl, "qfix_pair_choice", source=explicit, roles=pair_ids(world, "A", "C"))
    close(bound.evaluate("y", T=T), 0.5 * T**2)
    # the relation says linear for (A, B); the case's choice is quadratic, and no quadratic set
    # exists for that pair in the parameterization the case reads
    other = bind(world.decl, "qfix_pair_choice", source=explicit, roles=pair_ids(world, "A", "B"))
    with pytest.raises(EvaluationRefusal, match=r"no parameter set of slot group"):
        other.evaluate("y", T=T)


def test_with_no_choice_stated_the_evaluation_is_refused_naming_the_slot_and_subject(
    world: World, conn: psycopg.Connection
) -> None:
    found = source(world, conn, "choices")
    roles = pair_ids(world, "B", "C")
    bound = bind(world.decl, "qfix_pair_choice", source=found, roles=roles)
    with pytest.raises(EvaluationRefusal) as refused:
        bound.evaluate("y", T=T)
    message = str(refused.value)
    assert CHOICE_SLOT in message and roles["i"] in message and roles["j"] in message


def test_the_writer_refuses_a_choice_the_declaration_does_not_allow(world: World) -> None:
    w = writer(world.decl)
    param = w.kind(
        "parameterization",
        {"key": "k", "revision": "r", "title": "t", "coherence": "independent_records"},
        origins=[origin("a.json#/0", "fitted")],
    )
    a, b = _other(w), w.kind(
        "species", {"canonical_key": "Z", "label": "Z"}, origins=[origin("a.json#/2")]
    )

    def choose(**changes: object) -> None:
        w.subform_choice(
            **{
                "parameterization": param,
                "slot": CHOICE_SLOT,
                "subjects": [a, b],
                "form": "qfix_pair_linear",
                **changes,
            },  # type: ignore[arg-type]
            at="a.json#/4",
        )

    choose()
    with pytest.raises(ValidationError, match="accepts contract `qfix_pair`"):
        choose(form="qfix_departure")
    with pytest.raises(ValidationError, match="not a declared form"):
        choose(form="nothing")
    with pytest.raises(ValidationError, match="is not a sub-form slot"):
        choose(slot="qfix_pair_choice.nothing")
    with pytest.raises(ValidationError, match="2 role"):
        choose(subjects=[a])
    with pytest.raises(ValidationError, match="takes one choice"):
        choose(ordinal=2)
    with pytest.raises(ValidationError, match="not a whole number from one"):
        choose(ordinal=0)
    with pytest.raises(ValidationError, match="chosen per model"):
        w.subform_choice(
            parameterization=param,
            slot="qfix_pair_form.shift",
            subjects=[],
            form="qfix_linear",
            at="a.json#/4",
        )


def test_the_checks_flag_a_form_of_another_contract_and_a_gap_in_the_ordinals(
    world: World, conn: psycopg.Connection
) -> None:
    implements = CHECKS["subject_subform_choice.form_implements_slot_contract"]
    contiguous = CHECKS["subject_subform_choice.ordinals_contiguous"]
    assert run_check(conn, implements).violations == 0
    assert run_check(conn, contiguous).violations == 0
    conn.execute(
        "UPDATE tk.subject_subform_choice SET form = (SELECT id FROM meta.form WHERE name = %s) "
        "WHERE parameterization = %s AND subject_key LIKE %s",
        ("qfix_departure", world.ids["choices"], f'%{world.ids["species_B"]}%'),
    )
    flagged = run_check(conn, implements)
    assert flagged.error is None and flagged.violations == 1
    # a second choice for a slot of multiplicity one, and a choice numbered from two
    conn.execute(
        "INSERT INTO tk.subject_subform_choice (id, parameterization, slot, subject_key, ordinal, "
        "form) SELECT gen_random_uuid(), parameterization, slot, subject_key, 2, form "
        "FROM tk.subject_subform_choice LIMIT 1"
    )
    assert run_check(conn, contiguous).violations == 1
    conn.execute("UPDATE tk.subject_subform_choice SET ordinal = 3 WHERE ordinal = 2")
    assert run_check(conn, contiguous).violations == 1


def _declaration() -> Declaration:
    import tempfile

    if not _DECLARATION:
        _DECLARATION.append(
            fixture_declaration(Path(tempfile.mkdtemp(prefix="tk-shared-declaration-")))
        )
    return _DECLARATION[0]


_DECLARATION: list[Declaration] = []
