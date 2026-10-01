# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Validity regions (meta-model section 3.4 of the declaration, expressions.md section 5): a
contract argument names its observable; a region of several clauses, a clause about a component
and several alternative regions load and verify; the evaluator reports, per point, whether it lies
inside, outside, undetermined, or that no region is stated, and never refuses because of one."""

from __future__ import annotations

import uuid
from pathlib import Path

import numpy as np
import psycopg
import pytest

from build_support import fingerprint, inputs_of, write_source
from declaration_support import copy_full
from mapping_support import SATURATION, origin, real_declaration, writer
from qualify_support import fixture_declaration
from thermo_knowledge import config
from thermo_knowledge.build import build_database
from thermo_knowledge.canonical.values import Quantity
from thermo_knowledge.canonical.writer import (
    CanonicalWriter,
    CompetingAssertion,
    FamilyRow,
    ValidationError,
)
from thermo_knowledge.declaration import Declaration, load_declaration
from thermo_knowledge.expression.evaluate import EvaluationRefusal, bind
from thermo_knowledge.expression.parameters import (
    InMemorySource,
    RecordValidity,
    RegionClause,
    ValidityRegion,
)
from thermo_knowledge.expression.validity import Membership, counts
from thermo_knowledge.testing import TestDatabase
from thermo_knowledge.verify.checks import VERIFY_DIR, load_checks
from thermo_knowledge.verify.run import run_checks

FORM = "vapor_pressure_exp_series_tau"
INSIDE, OUTSIDE = int(Membership.INSIDE), int(Membership.OUTSIDE)
UNDETERMINED, NOT_STATED = int(Membership.UNDETERMINED), int(Membership.NOT_STATED)

# -- the contract argument names its observable -----------------------------------------------


def test_the_contracts_of_the_committed_forms_name_the_observable_of_their_temperature() -> None:
    decl = real_declaration()
    for name in ("pure_vapor_pressure", "standard_state_thermo"):
        (argument,) = decl.contracts[name].arguments
        assert (argument.name, argument.observable) == ("T", "temperature")


def test_an_argument_observable_is_reified_as_the_entity_it_names() -> None:
    decl = real_declaration()
    temperature = decl.observable_entity("temperature")
    assert temperature is not None
    with TestDatabase() as database:
        build_database(database.url, decl)
        with psycopg.connect(database.url) as conn:
            row = conn.execute(
                "SELECT observable FROM meta.contract_argument "
                "WHERE contract = 'pure_vapor_pressure' AND name = 'T'"
            ).fetchone()
            unnamed = conn.execute(
                "SELECT count(*) FROM meta.contract_argument WHERE name = 'T' AND observable IS DISTINCT FROM %s",
                (temperature.id,),
            ).fetchone()
            target = conn.execute(
                "SELECT count(*) FROM pg_constraint WHERE conname = 'contract_argument__fk__observable'"
            ).fetchone()
    assert row == (temperature.id,)
    assert unnamed is not None and unnamed[0] == 0, (
        "every committed argument named T is the temperature"
    )
    assert target == (1,), "the column references the observable kind"


MODULE = """
module = "zz_av"
uses = ["physical", "identity", "framework", "vocab"]
doc = "An argument observable under test."

[contracts.curve]
doc = "d"
arguments.T = {{ type = "Temperature", {observable}, doc = "d" }}
outputs.p = {{ type = "Pressure", observable = "vapor_pressure", doc = "d" }}
"""


def load_argument(tmp_path: Path, observable: str, *, role: bool = True) -> list[str]:
    tree = copy_full(tmp_path)
    (tree / "forms" / "zz_av.toml").write_text(
        MODULE.format(observable=observable), encoding="utf-8"
    )
    if not role:
        manifest = tree / "model" / "manifest.toml"
        manifest.write_text(manifest.read_text().replace('observable = "observable"\n', ""))
    found = load_declaration(tree / "model", tree / "forms", contract=None).diagnostics
    return [f"{d.code} {d.construct}: {d.message}" for d in found]


def test_an_argument_may_name_a_declared_observable(tmp_path: Path) -> None:
    assert load_argument(tmp_path, 'observable = "critical_temperature"') == []


def test_an_argument_naming_an_undeclared_observable_is_refused(tmp_path: Path) -> None:
    (found,) = load_argument(tmp_path, 'observable = "no_such_observable"')
    assert found.startswith("unknown-name contracts.curve.arguments.T.observable: ")
    assert "`no_such_observable` is not a declared entity of kind `observable`" in found


def test_an_argument_observable_needs_the_observable_role(tmp_path: Path) -> None:
    found = load_argument(tmp_path, 'observable = "critical_temperature"', role=False)
    assert any(
        item.startswith("framework-role contracts.curve.arguments.T.observable") for item in found
    ), found


DIMENSIONS = """
module = "zz_dim"
uses = ["physical", "identity", "framework", "vocab"]
doc = "An observable whose dimension the argument or output must have."

[contracts.curve]
doc = "d"
arguments.T = {{ type = "{argument}", observable = "critical_temperature", doc = "d" }}
outputs.p = {{ type = "{output}", observable = "vapor_pressure", doc = "d" }}
"""


def load_dimensions(tmp_path: Path, argument: str, output: str) -> list[tuple[str, str, str]]:
    tree = copy_full(tmp_path)
    (tree / "forms" / "zz_dim.toml").write_text(
        DIMENSIONS.format(argument=argument, output=output), encoding="utf-8"
    )
    found = load_declaration(tree / "model", tree / "forms", contract=None).diagnostics
    return [(d.code, d.construct, d.message) for d in found]


def test_an_argument_or_output_that_names_an_observable_has_its_dimension(tmp_path: Path) -> None:
    assert load_dimensions(tmp_path / "ok", "Temperature", "Pressure") == []


def test_an_argument_of_another_dimension_than_its_observable_is_a_located_diagnostic(
    tmp_path: Path,
) -> None:
    ((code, construct, message),) = load_dimensions(tmp_path, "Pressure", "Pressure")
    assert (code, construct) == ("observable-dimension", "contracts.curve.arguments.T.observable")
    assert "`critical_temperature` is a `Temperature`" in message and "`T` is a Pressure" in message


def test_an_output_of_another_dimension_than_its_observable_is_a_located_diagnostic(
    tmp_path: Path,
) -> None:
    ((code, construct, message),) = load_dimensions(tmp_path, "Temperature", "Temperature")
    assert (code, construct) == ("observable-dimension", "contracts.curve.outputs.p.observable")
    assert "`vapor_pressure` is a `Pressure`" in message and "`p` is a Temperature" in message


def test_both_a_wrong_argument_and_a_wrong_output_are_reported(tmp_path: Path) -> None:
    found = load_dimensions(tmp_path, "Pressure", "Temperature")
    assert sorted(construct for _, construct, _ in found) == [
        "contracts.curve.arguments.T.observable",
        "contracts.curve.outputs.p.observable",
    ]


# -- a region, its clauses and its coverage load and verify -----------------------------------


def species_and_set(
    w: CanonicalWriter, decl: Declaration
) -> tuple[uuid.UUID, uuid.UUID, uuid.UUID]:
    """A species, a parameterization and one saturation set for it, all published."""
    published = origin("a.json#/0", "published")
    species = w.kind("species", {"canonical_key": "K", "label": "K"}, origins=[published])
    parameterization = w.kind(
        "parameterization",
        {"key": "k", "revision": "r", "title": "t", "coherence": "independent_records"},
        origins=[published],
    )
    made = w.parameter_set(
        parameterization=parameterization,
        slot_group=SATURATION,
        subjects=[species],
        slots={"T_r": Quantity(400.0, "K"), "p_r": Quantity(1e6, "Pa")},
        families={"term": [FamilyRow({"k": 1}, {"n": -7.0, "t": 1.0})]},
        origins=[published],
    )
    return species, parameterization, made


def observables(decl: Declaration) -> dict[str, uuid.UUID]:
    found = {e.name: e.id for e in decl.entities if e.kind == "observable"}
    return found


def test_a_region_of_two_clauses_a_clause_about_a_component_and_alternatives_load_and_verify(
    tmp_path: Path,
) -> None:
    decl = real_declaration()
    seen = observables(decl)
    made: dict[str, uuid.UUID] = {}

    def emit(w: CanonicalWriter) -> None:
        species, parameterization, record = species_and_set(w, decl)
        published = origin("a.json#/1", "published")
        gas = next(e.id for e in decl.entities if e.kind == "aggregation" and e.name == "gas")
        # a temperature and a pressure limit that hold together
        made["coupled"] = w.validity_region(
            record,
            {"kind": "fitted_range"},
            [
                {
                    "observable": seen["temperature"],
                    "lower": Quantity(250.0, "K"),
                    "upper": Quantity(399.0, "K"),
                },
                {
                    "observable": seen["pressure"],
                    "lower": Quantity(1.0, "bar"),
                    "upper": Quantity(40.0, "bar"),
                },
            ],
            origins=[published],
        )
        # a second alternative of the same kind, one clause about a component and one about the
        # gas phase of the set
        made["alternative"] = w.validity_region(
            record,
            {"kind": "fitted_range"},
            [
                {
                    "observable": seen["molar_density"],
                    "component": species,
                    "lower": Quantity(0.0, "mol/m^3"),
                    "upper": Quantity(5000.0, "mol/m^3"),
                },
                {
                    "observable": seen["temperature"],
                    "aggregation": gas,
                    "lower": Quantity(300.0, "K"),
                },
            ],
            origins=[published],
            ordinal=2,
        )
        # validity of another kind, and of the parameterization as a whole
        w.validity_region(
            record,
            {"kind": "recommended_range"},
            [{"observable": seen["temperature"], "upper": Quantity(380.0, "K")}],
            origins=[published],
        )
        w.validity_region(
            parameterization,
            {"kind": "validated_range"},
            [{"observable": seen["pressure"], "lower": Quantity(0.0, "Pa")}],
            origins=[published],
        )
        w.validity_not_stated(record, "validated_range", at="a.json#/2")
        made["record"], made["parameterization"] = record, parameterization

    write_source(tmp_path, "src", emit, decl=decl, declaration=fingerprint(decl))
    with TestDatabase() as database:
        build_database(database.url, decl, inputs_of(tmp_path))
        with psycopg.connect(database.url) as conn:
            clauses = conn.execute(
                "SELECT r.kind::text, r.ordinal, c.ordinal, o.key, c.component IS NOT NULL, "
                "c.aggregation IS NOT NULL, c.lower, c.upper "
                "FROM tk.validity_region r JOIN tk.region_clause c ON c.region = r.id "
                "JOIN tk.observable o ON o.id = c.observable "
                "WHERE r.record = %s ORDER BY r.kind::text, r.ordinal, c.ordinal",
                (made["record"],),
            ).fetchall()
            coverage = conn.execute(
                "SELECT kind::text, value::text FROM tk.validity_coverage WHERE record = %s "
                "ORDER BY kind::text",
                (made["record"],),
            ).fetchall()
            whole = conn.execute(
                "SELECT kind::text, value::text FROM tk.validity_coverage WHERE record = %s",
                (made["parameterization"],),
            ).fetchall()
            checks, problems = load_checks(config.TREE_DIR / VERIFY_DIR)
            results = run_checks(conn, checks)
    assert clauses == [
        ("fitted_range", 1, 1, "temperature", False, False, 250.0, 399.0),
        ("fitted_range", 1, 2, "pressure", False, False, 1e5, 4e6),
        ("fitted_range", 2, 1, "molar_density", True, False, 0.0, 5000.0),
        ("fitted_range", 2, 2, "temperature", False, True, 300.0, None),
        ("recommended_range", 1, 1, "temperature", False, False, None, 380.0),
    ]
    assert coverage == [
        ("fitted_range", "stated"),
        ("recommended_range", "stated"),
        ("validated_range", "not_stated"),
    ]
    assert whole == [("validated_range", "stated")], "a parameterization has regions too"
    assert problems == []
    assert [
        (r.check.target, r.violations, r.error) for r in results if r.violations or r.error
    ] == [], "every constraint and every verify check is satisfied"


def writer_with_set() -> tuple[CanonicalWriter, uuid.UUID, dict[str, uuid.UUID]]:
    decl = real_declaration()
    w = writer(decl)
    _, _, record = species_and_set(w, decl)
    return w, record, observables(decl)


def refused(call: object) -> str:
    with pytest.raises((ValidationError, CompetingAssertion)) as error:
        call()  # type: ignore[operator]
    return str(error.value)


def test_a_region_needs_a_clause_and_a_clause_is_checked_like_any_record() -> None:
    w, record, seen = writer_with_set()
    published = origin("a.json#/1", "published")
    clause = {"observable": seen["temperature"], "lower": Quantity(250.0, "K")}
    before = w.counts()
    assert "a validity region has at least one clause" in refused(
        lambda: w.validity_region(
            record, {"kind": "fitted_range"}, [], origins=[published], at="a.json#/1"
        )
    )
    assert "'nonsense' is not a member of enum `envelope_kind`" in refused(
        lambda: w.validity_region(
            record, {"kind": "nonsense"}, [clause], origins=[published], at="a.json#/1"
        )
    )
    assert "`bounds_ordered`" in refused(
        lambda: w.validity_region(
            record,
            {"kind": "fitted_range"},
            [
                {
                    "observable": seen["temperature"],
                    "lower": Quantity(300.0, "K"),
                    "upper": Quantity(200.0, "K"),
                }
            ],
            origins=[published],
            at="a.json#/1",
        )
    )
    assert "cannot be converted to `K`" in refused(
        lambda: w.validity_region(
            record,
            {"kind": "fitted_range"},
            [{"observable": seen["temperature"], "lower": Quantity(1.0, "Pa")}],
            origins=[published],
            at="a.json#/1",
        )
    )
    assert "is given by the writer" in refused(
        lambda: w.validity_region(
            record,
            {"kind": "fitted_range", "ordinal": 7},
            [clause],
            origins=[published],
            at="a.json#/1",
        )
    )
    assert w.counts() == before, "nothing of a refused region is written"


def test_a_source_states_regions_of_a_kind_or_that_it_states_none_not_both() -> None:
    w, record, seen = writer_with_set()
    published = origin("a.json#/1", "published")
    w.validity_region(
        record,
        {"kind": "fitted_range"},
        [{"observable": seen["temperature"], "lower": Quantity(250.0, "K")}],
        origins=[published],
    )
    # the same region again is the same record; a second ordinal is another alternative
    w.validity_region(
        record,
        {"kind": "fitted_range"},
        [{"observable": seen["temperature"], "lower": Quantity(250.0, "K")}],
        origins=[published],
    )
    assert w.rows("tk.validity_region") == 1 and w.rows("tk.validity_coverage") == 1
    assert "competing assertions" in refused(
        lambda: w.validity_not_stated(record, "fitted_range", at="a.json#/2")
    )
    assert "'nonsense' is not a member of enum `envelope_kind`" in refused(
        lambda: w.validity_not_stated(record, "nonsense", at="a.json#/2")
    )


# -- where a point lies -----------------------------------------------------------------------

SP = str(uuid.UUID(int=1))


def saturation(validities: list[RecordValidity]) -> tuple[Declaration, InMemorySource]:
    decl = real_declaration()
    return decl, InMemorySource(
        slots={(SATURATION, (SP,)): {"T_r": 400.0, "p_r": 1.0e6}},
        families={(SATURATION, "term", (SP,)): {(1,): {"n": -7.0, "t": 1.0}}},
        declaration=decl,
        validities={(SATURATION, (SP,)): validities},
    )


def clause(
    observable: str = "temperature",
    lower: float | None = 250.0,
    upper: float | None = 399.0,
    **extra: object,
) -> RegionClause:
    return RegionClause(observable, lower, upper, **extra)  # type: ignore[arg-type]


def stated(
    *regions: tuple[RegionClause, ...], kind: str = "fitted_range", record: str = "set"
) -> RecordValidity:
    return RecordValidity(record, kind, "stated", tuple(ValidityRegion(r) for r in regions))


TEMPERATURES = np.array([200.0, 250.0, 300.0, 399.0, 399.5])


def membership(validities: list[RecordValidity], kind: str = "fitted_range") -> list[int]:
    decl, source = saturation(validities)
    bound = bind(decl, FORM, source=source, roles={"i": SP})
    return [int(code) for code in bound.validity("p_sat", kind, T=TEMPERATURES)]


def test_a_point_is_inside_or_outside_the_one_region_a_record_states() -> None:
    assert membership([stated((clause(),))]) == [OUTSIDE, INSIDE, INSIDE, INSIDE, OUTSIDE]


def test_a_clause_with_one_bound_limits_one_side() -> None:
    assert membership([stated((clause(lower=300.0, upper=None),))]) == [
        OUTSIDE,
        OUTSIDE,
        INSIDE,
        INSIDE,
        INSIDE,
    ]


def test_several_regions_of_one_kind_are_alternatives_and_a_point_in_any_is_inside() -> None:
    low, high = (clause(lower=200.0, upper=260.0),), (clause(lower=390.0, upper=400.0),)
    assert membership([stated(low, high)]) == [INSIDE, INSIDE, OUTSIDE, INSIDE, INSIDE]


def test_the_clauses_of_a_region_all_hold() -> None:
    decl = real_declaration()
    assert decl.contracts["pure_vapor_pressure"].arguments[0].observable == "temperature"
    both = (clause(lower=200.0, upper=300.0), clause(lower=250.0, upper=399.0))
    assert membership([stated(both)]) == [OUTSIDE, INSIDE, INSIDE, OUTSIDE, OUTSIDE]


def test_a_clause_no_argument_names_leaves_a_point_undetermined_unless_a_decidable_clause_fails() -> (
    None
):
    # the pressure is no argument of the contract: inside the temperature clause the region cannot be
    # decided, and outside it the region is false whatever the pressure is
    coupled = (clause(), clause("pressure", 1e5, 4e6))
    assert membership([stated(coupled)]) == [
        OUTSIDE,
        UNDETERMINED,
        UNDETERMINED,
        UNDETERMINED,
        OUTSIDE,
    ]
    assert membership([stated((clause(component=SP), clause()))]) == [
        OUTSIDE,
        UNDETERMINED,
        UNDETERMINED,
        UNDETERMINED,
        OUTSIDE,
    ]


def test_a_region_whose_only_clause_is_about_a_component_or_a_phase_is_undetermined() -> None:
    assert membership([stated((clause(component=SP),))]) == [UNDETERMINED] * 5
    assert membership([stated((clause(aggregation="gas"),))]) == [UNDETERMINED] * 5


def test_a_region_that_can_be_decided_decides_for_its_record_whatever_the_others_say() -> None:
    decidable, undecided = (clause(),), (clause(component=SP),)
    # inside the decidable alternative: inside, although the other alternative cannot be decided
    assert membership([stated(decidable, undecided)]) == [
        UNDETERMINED,
        INSIDE,
        INSIDE,
        INSIDE,
        UNDETERMINED,
    ]


def test_a_point_is_outside_when_outside_any_record_that_states_regions() -> None:
    whole = stated((clause(),), record="set")
    narrower = stated((clause(lower=280.0, upper=320.0),), record="parameterization")
    assert membership([whole, narrower]) == [OUTSIDE, OUTSIDE, INSIDE, OUTSIDE, OUTSIDE]
    undecided = stated((clause(component=SP),), record="parameterization")
    assert membership([whole, undecided]) == [
        OUTSIDE,
        UNDETERMINED,
        UNDETERMINED,
        UNDETERMINED,
        OUTSIDE,
    ]


def test_no_region_stated_for_the_kind_is_reported_as_such() -> None:
    assert membership([]) == [NOT_STATED] * 5, "a record with no row: not mapped"
    not_stated = RecordValidity("set", "fitted_range", "not_stated")
    assert membership([not_stated]) == [NOT_STATED] * 5, "the source gives no range"
    assert membership([stated((clause(),), kind="recommended_range")]) == [NOT_STATED] * 5, (
        "regions of another kind say nothing about this one"
    )
    # a record that states none does not hide another that does
    assert membership([not_stated, stated((clause(),))]) == [
        OUTSIDE,
        INSIDE,
        INSIDE,
        INSIDE,
        OUTSIDE,
    ]


def test_the_counts_add_up_and_a_scalar_argument_gives_one_membership() -> None:
    decl, source = saturation([stated((clause(),))])
    bound = bind(decl, FORM, source=source, roles={"i": SP})
    codes = bound.validity("p_sat", "fitted_range", T=TEMPERATURES)
    assert counts(codes) == {
        Membership.INSIDE: 3,
        Membership.OUTSIDE: 2,
        Membership.UNDETERMINED: 0,
        Membership.NOT_STATED: 0,
    }
    assert bound.validity("p_sat", "fitted_range", T=300.0).shape == ()
    assert int(bound.validity("p_sat", "fitted_range", T=300.0)) == INSIDE


def test_membership_never_refuses_an_evaluation_and_an_unknown_kind_is_refused() -> None:
    decl, source = saturation([stated((clause(),))])
    bound = bind(decl, FORM, source=source, roles={"i": SP})
    outside = np.array([200.0])
    assert int(bound.validity("p_sat", "fitted_range", T=outside)[0]) == OUTSIDE
    assert np.isfinite(bound.evaluate("p_sat", T=outside)).all(), "evaluating is not refused"
    with pytest.raises(EvaluationRefusal, match="`nonsense` is not a validity region kind"):
        bound.validity("p_sat", "nonsense", T=outside)
    with pytest.raises(EvaluationRefusal, match="`x` is not an argument"):
        bound.validity("p_sat", "fitted_range", x=outside)


@pytest.fixture(scope="module")
def fixtures(tmp_path_factory: pytest.TempPathFactory) -> Declaration:
    return fixture_declaration(tmp_path_factory.mktemp("validity-declaration"))


def state_source(validities: list[RecordValidity]) -> InMemorySource:
    return InMemorySource(
        slots={("qval_state_linear.pure", (SP,)): {"a": 1.0, "b": 0.5, "c": 1e-6}},
        validities={("qval_state_linear.pure", (SP,)): validities},
    )


def test_two_arguments_that_name_observables_each_decide_their_clause(
    fixtures: Declaration,
) -> None:
    both = stated((clause("temperature", 300.0, 400.0), clause("pressure", 1e5, 2e5)))
    bound = bind(fixtures, "qval_state_linear", source=state_source([both]), roles={"i": SP})
    temperature = np.array([290.0, 350.0, 350.0, 350.0, 410.0])
    pressure = np.array([1.5e5, 1.5e5, 0.5e5, 2.5e5, 1.5e5])
    codes = bound.validity("y", "fitted_range", T=temperature, p=pressure)
    assert [int(code) for code in codes] == [OUTSIDE, INSIDE, OUTSIDE, OUTSIDE, OUTSIDE]
    # one argument is enough for the clauses on its observable, and the others are undetermined
    only = bound.validity("y", "fitted_range", T=temperature)
    assert [int(code) for code in only] == [
        OUTSIDE,
        UNDETERMINED,
        UNDETERMINED,
        UNDETERMINED,
        OUTSIDE,
    ], "the pressure clause has no value: undetermined inside the temperature range, outside it"


def test_an_observable_that_two_arguments_name_binds_to_neither(fixtures: Declaration) -> None:
    region = stated((clause("temperature", 300.0, 400.0),))
    source = InMemorySource(
        slots={("qval_twin_linear.pure", (SP,)): {"b": 1.0, "c": 2.0}},
        validities={("qval_twin_linear.pure", (SP,)): [region]},
    )
    bound = bind(fixtures, "qval_twin_linear", source=source, roles={"i": SP})
    codes = bound.validity("y", "fitted_range", T1=np.array([350.0]), T2=np.array([350.0]))
    assert [int(code) for code in codes] == [UNDETERMINED], "which temperature is meant is not said"
