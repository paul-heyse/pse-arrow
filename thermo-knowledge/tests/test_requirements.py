# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The enforcement boundary of constraints (meta-model sections 3.1, 3.3, 3.4 and 3.5):
invariants of relations, the check forms `present_iff` and `within`, enum facets, `absolute`
quantities that accept zero, and the text keys the writer computes from references."""

from __future__ import annotations

import uuid
from pathlib import Path

import pytest

from declaration_support import copy_full, full_declaration
from mapping_support import carrier, origin, real_declaration, writer
from thermo_knowledge.canonical.invariants import ddl_violation
from thermo_knowledge.canonical.provenance import Carriers
from thermo_knowledge.canonical.values import Quantity, ValueRefused, scalar
from thermo_knowledge.canonical.writer import (
    CanonicalWriter,
    FamilyRow,
    ValidationError,
    WriteError,
)
from thermo_knowledge.declaration import Declaration, Diagnostic, load_declaration
from thermo_knowledge.declaration.types import TypeRef

MODULE = """
module = "zz_probe"
schema = "tk"
uses = ["physical", "identity"]
doc = "A probe relation with one requirement."

[relations.probe]
doc = "A probe."
provenance = "none"
absence = "optional"

[relations.probe.keys]
n = {{ type = "Integer", doc = "Position." }}

[relations.probe.columns]
state = {{ type = "phase_class", doc = "A state." }}
value = {{ type = "Scalar", optional = true, doc = "A value." }}
label = {{ type = "Text", optional = true, doc = "A label." }}
strict = {{ type = "Scalar", doc = "Always present." }}

[[relations.probe.requires]]
name = "probe_rule"
doc = "A rule."
enforced = "{enforced}"
{check}
"""


def load(tmp_path: Path, check: str, enforced: str = "ddl") -> list[Diagnostic]:
    tree = copy_full(tmp_path)
    (tree / "model" / "zz_probe.toml").write_text(MODULE.format(enforced=enforced, check=check))
    return list(load_declaration(tree / "model", tree / "forms", contract=None).diagnostics)


def refused(tmp_path: Path, check: str, message: str) -> None:
    found = load(tmp_path, check)
    assert [d.code for d in found] == ["bad-check"], found
    assert message in found[0].message
    assert found[0].construct == "relations.probe.requires[0].check"
    assert found[0].document == "model/zz_probe.toml"


# -- the loader ---------------------------------------------------------------------------------


def test_a_relation_declares_requirements_at_each_enforcement_point(tmp_path: Path) -> None:
    assert load(tmp_path, 'check = { within = { column = "value", lower = 0 } }') == []
    assert load(tmp_path / "v", "", "verify") == []
    assert load(tmp_path / "l", "", "load") == []


@pytest.mark.parametrize(
    ("check", "message"),
    [
        (
            'check = { present_iff = { column = "strict", when = "state", in = ["gas"] } }',
            "`present_iff` governs an optional column, and `strict` is always present",
        ),
        (
            'check = { present_iff = { column = "value", when = "label", in = ["gas"] } }',
            "`when` names an enum column, and `label` is not one",
        ),
        (
            'check = { present_iff = { column = "value", when = "state", in = ["plasma"] } }',
            "`plasma` is not a member of enum `phase_class`",
        ),
        (
            'check = { present_iff = { column = "value", when = "state", in = [] } }',
            "`in` lists at least one member",
        ),
        (
            'check = { present_iff = { column = "value", when = "state", in = ["gas", "gas"] } }',
            "`in` names a member twice",
        ),
        (
            'check = { present_iff = { column = "nothing", when = "state", in = ["gas"] } }',
            "`nothing` is not a column of this relation's own table",
        ),
        (
            'check = { within = { column = "value" } }',
            "`within` states `lower`, `upper` or both",
        ),
        (
            'check = { within = { column = "value", lower = 2, upper = 1 } }',
            "`within` has `lower` above `upper`",
        ),
        (
            'check = { within = { column = "label", lower = 0 } }',
            "`within` applies to numeric attributes",
        ),
        (
            'check = { nonempty = "label", within = { column = "value", lower = 0 } }',
            "a `check` has exactly one of",
        ),
    ],
)
def test_the_new_check_forms_are_validated(tmp_path: Path, check: str, message: str) -> None:
    refused(tmp_path, check, message)


def test_a_check_belongs_to_a_ddl_requirement(tmp_path: Path) -> None:
    found = load(tmp_path, 'check = { within = { column = "value", lower = 0 } }', "verify")
    assert [d.code for d in found] == ["bad-check"]
    assert "belongs to invariants enforced by `ddl`" in found[0].message


def test_a_requirement_is_reified_with_its_owner_and_its_check() -> None:
    decl = full_declaration()
    (rule,) = [r for r in decl.relations["reading"].requires if r.name == "value_in_range"]
    assert (rule.enforced, rule.rule, rule.attributes, rule.lower, rule.upper) == (
        "ddl",
        "within",
        ("value",),
        -1.0,
        1.0,
    )
    (only,) = [
        r for r in decl.relations["reading"].requires if r.name == "value_only_when_compressible"
    ]
    assert (only.rule, only.attributes, only.members) == (
        "present_iff",
        ("value", "state"),
        ("gas",),
    )


# -- the writer ---------------------------------------------------------------------------------


MINIMAL = """
module = "zz_minimal"
schema = "tk"
doc = "The smallest declaration with a relation that has requirements."

[enums.phase_class]
doc = "Coarse classification of a phase."
members.gas = { doc = "A gas." }
members.liquid = { doc = "A liquid." }

[quantity_types.Scalar]
doc = "A dimensionless number."
unit = "dimensionless"
scale = "dimensionless"

[relations.reading]
doc = "A reading: a value that only a gas has."
provenance = "none"
absence = "optional"

[relations.reading.keys]
n = { type = "Integer", doc = "Position." }

[relations.reading.columns]
state = { type = "phase_class", doc = "A state." }
value = { type = "Scalar", optional = true, doc = "A value." }

[[relations.reading.requires]]
name = "value_only_when_gas"
doc = "A value is present exactly when the state is gas."
enforced = "ddl"
check = { present_iff = { column = "value", when = "state", in = ["gas"] } }

[[relations.reading.requires]]
name = "value_in_range"
doc = "A value lies between minus one and one."
enforced = "ddl"
check = { within = { column = "value", lower = -1, upper = 1 } }

[[relations.reading.requires]]
name = "value_not_below"
doc = "A value is not below minus one half."
enforced = "ddl"
check = { within = { column = "value", lower = -0.5 } }
"""


def minimal(tmp_path: Path) -> Declaration:
    (tmp_path / "model").mkdir(parents=True)
    (tmp_path / "model" / "manifest.toml").write_text('doc = "d"\n\n[framework]\n')
    (tmp_path / "model" / "zz_minimal.toml").write_text(MINIMAL)
    return load_declaration(tmp_path / "model", tmp_path / "forms", contract=None).require()


def reading(w: CanonicalWriter, n: int, state: str, value: float | None) -> uuid.UUID:
    values: dict[str, object] = {"state": state}
    if value is not None:
        values["value"] = value
    return w.relation("reading", {"n": n}, values)


def test_the_writer_enforces_the_ddl_requirements_of_a_relation(tmp_path: Path) -> None:
    w = CanonicalWriter(minimal(tmp_path))
    reading(w, 1, "gas", 0.5)
    reading(w, 2, "gas", 1.0)
    reading(w, 3, "liquid", None)
    for n, state, value, message in (
        (4, "gas", None, "value is absent but state is `gas`"),
        (5, "liquid", 0.5, "value is present but state is `liquid`"),
        (6, "gas", 1.5, "value 1.5 is above 1.0"),
        (7, "gas", -0.75, "value -0.75 is below -0.5"),
    ):
        with pytest.raises(ValidationError) as raised:
            reading(w, n, state, value)
        assert message in str(raised.value)
    assert w.rows("tk.reading") == 3


def test_the_writer_names_a_conversion_exactly_when_the_level_is_equal_under_conversion() -> None:
    w = writer(real_declaration())
    conversion = uuid.uuid4()

    def assess(level: str, named: uuid.UUID | None) -> uuid.UUID:
        return w.relation(
            "equivalence_assessment",
            {"a": uuid.uuid4(), "b": uuid.uuid4()},
            {"level": level, "conversion": named},
        )

    assess("exact", None)
    assess("equal_under_conversion", conversion)
    for level, named, message in (
        ("exact", conversion, "conversion is present but level is `exact`"),
        ("conflicting", conversion, "conversion is present but level is `conflicting`"),
        (
            "equal_under_conversion",
            None,
            "conversion is absent but level is `equal_under_conversion`",
        ),
    ):
        with pytest.raises(ValidationError) as raised:
            assess(level, named)
        assert "conversion_iff_under_conversion" in str(raised.value) and message in str(
            raised.value
        )
    assert w.rows("prov.equivalence_assessment") == 2


def test_a_relation_load_invariant_needs_an_evaluator(tmp_path: Path) -> None:
    tree = copy_full(tmp_path)
    (tree / "model" / "zz_probe.toml").write_text(MODULE.format(enforced="load", check=""))
    decl = load_declaration(tree / "model", tree / "forms", contract=None).require()
    with pytest.raises(WriteError, match=r"probe\.probe_rule"):
        CanonicalWriter(decl)


@pytest.mark.parametrize(
    ("state", "value", "violation"),
    [
        ("gas", 0.25, None),
        ("gas", None, "absent"),
        ("liquid", 0.25, "present"),
        ("liquid", None, None),
        (None, 0.25, None),  # a null condition satisfies the check, as SQL's CHECK does
    ],
)
def test_present_iff_follows_the_semantics_of_a_sql_check(
    state: str | None, value: float | None, violation: str | None
) -> None:
    (rule,) = [
        r for r in full_declaration().relations["reading"].requires if r.rule == "present_iff"
    ]
    found = ddl_violation(rule, {"state": state, "value": value})
    assert (found is None) == (violation is None)
    if violation is not None:
        assert violation in found  # type: ignore[operator]


# -- enum facets --------------------------------------------------------------------------------

FACETS = """
[enums.origin_role.facets]
extra = {{ doc = "Another property." }}
"""


def test_the_facets_of_origin_role_are_declared_on_its_members() -> None:
    decl = real_declaration()
    assert [f.name for f in decl.enums["origin_role"].facets] == [
        "requires_derivation",
        "requires_fit",
        "not_knowledge",
    ]
    assert decl.enum_members_with("origin_role", "requires_derivation") == (
        "fitted",
        "estimated",
        "derived",
    )
    assert decl.enum_members_with("origin_role", "requires_fit") == ("fitted",)
    assert decl.enum_members_with("origin_role", "not_knowledge") == ("synthetic", "oracle_input")


def test_a_member_may_only_name_facets_its_enum_declares(tmp_path: Path) -> None:
    tree = copy_full(tmp_path)
    path = tree / "model" / "physical.toml"
    path.write_text(
        path.read_text().replace(
            'members.liquid = { doc = "A liquid." }',
            'members.liquid = { doc = "A liquid.", facets = ["compressible", "invented"] }',
        )
    )
    found = load_declaration(tree / "model", tree / "forms", contract=None).diagnostics
    assert [(d.code, d.construct) for d in found] == [
        ("unknown-name", "enums.phase_class.members.liquid.facets")
    ]
    assert "`invented` is not a facet of enum `phase_class`" in found[0].message
    assert "facets: compressible" in found[0].message


def test_a_facet_is_named_once_by_a_member(tmp_path: Path) -> None:
    tree = copy_full(tmp_path)
    path = tree / "model" / "physical.toml"
    path.write_text(
        path.read_text().replace(
            'facets = ["compressible"]', 'facets = ["compressible", "compressible"]'
        )
    )
    found = load_declaration(tree / "model", tree / "forms", contract=None).diagnostics
    assert [d.code for d in found] == ["bad-attribute"]


# -- absolute means non-negative ---------------------------------------------------------------


def quantity(decl: Declaration, name: str) -> TypeRef:
    return TypeRef(container="scalar", element_kind="quantity", element=name, text=name)


@pytest.mark.parametrize(
    ("type_name", "unit"),
    [("DipoleMoment", "C*m"), ("Temperature", "K"), ("Loading", "mol/kg"), ("Fraction", "")],
)
def test_an_absolute_quantity_accepts_zero_and_refuses_a_negative(
    type_name: str, unit: str
) -> None:
    decl = real_declaration()
    assert decl.quantity_types[type_name].scale == "absolute"
    type_ = quantity(decl, type_name)
    raw = Quantity(0.0, unit) if unit else 0.0
    assert scalar(decl, type_, raw) == 0.0
    negative = Quantity(-1.0, unit) if unit else -1.0
    with pytest.raises(ValueRefused, match="is negative, as an absolute quantity must not be"):
        scalar(decl, type_, negative)


def piece_rows(low: float) -> dict[str, list[FamilyRow]]:
    values: dict[str, object] = {
        "T_low": Quantity(low, "K"),
        "T_high": Quantity(1000.0, "K"),
        "a1": 0.0,
        "a2": Quantity(0.0, "1/K"),
        "a3": Quantity(0.0, "1/K**2"),
        "a4": Quantity(0.0, "1/K**3"),
        "a5": Quantity(0.0, "1/K**4"),
        "a6": Quantity(0.0, "K"),
        "a7": 0.0,
    }
    return {"piece": [FamilyRow({"n": 1}, values)]}


def test_a_temperature_piece_may_start_at_zero_kelvin_and_a_negative_bound_is_refused() -> None:
    decl = real_declaration()
    w = writer(decl)
    param = w.kind(
        "parameterization",
        {"key": "k", "revision": "r", "title": "t", "coherence": "independent_records"},
        origins=[origin("a.json#/0", "published")],
    )
    species = w.kind("species", {"canonical_key": "K", "label": "K"}, origins=[origin("a.json#/1")])
    gas = next(e.id for e in decl.entities if e.kind == "aggregation" and e.name == "gas")
    form = w.kind(
        "species_form",
        {"canonical_key": "K gas", "label": "K gas", "species": species, "aggregation": gas},
        origins=[origin("a.json#/2")],
    )

    def write(low: float, locator: str) -> uuid.UUID:
        return w.parameter_set(
            parameterization=param,
            slot_group="nasa7.pure",
            subjects=[form],
            slots={},
            families=piece_rows(low),
            origins=[origin(locator, "fitted")],
        )

    write(0.0, "a.json#/3")
    assert w.rows("param.nasa7__pure__piece") == 1
    with pytest.raises(ValidationError, match="a.json#/4: .*T_low: -1.0 is negative"):
        write(-1.0, "a.json#/4")
    assert w.rows("param.nasa7__pure__piece") == 1


# -- keys that restate references ---------------------------------------------------------------


def test_the_writer_computes_the_host_key_of_a_site_class_from_its_reference() -> None:
    decl = real_declaration()
    w = CanonicalWriter(decl, carriers_of("a.json"))
    phase = uuid.uuid4()
    site = w.kind(
        "site_class",
        {"index": 1, "phase": phase, "ratio_kind": "composition_dependent"},
        origins=[origin("a.json#/0")],
    )
    (row,) = w.tables()["tk.site_class"].to_pylist()
    assert row["id"] == site and row["host_key"] == str(phase)
    material = uuid.uuid4()
    w.kind(
        "site_class",
        {"index": 2, "material": material, "ratio_kind": "composition_dependent"},
        origins=[origin("a.json#/1")],
    )
    keys = {r["index"]: r["host_key"] for r in w.tables()["tk.site_class"].to_pylist()}
    assert keys == {1: str(phase), 2: str(material)}


def test_a_mapping_supplies_the_reference_and_never_the_key() -> None:
    w = CanonicalWriter(real_declaration(), carriers_of("a.json"))
    with pytest.raises(ValidationError, match="host_key: is computed by the writer"):
        w.kind(
            "site_class",
            {
                "index": 1,
                "phase": uuid.uuid4(),
                "host_key": "a name",
                "ratio_kind": "composition_dependent",
            },
            origins=[origin("a.json#/0")],
        )
    with pytest.raises(ValidationError, match="the one of phase, material that is present, and 0"):
        w.kind(
            "site_class",
            {"index": 1, "ratio_kind": "composition_dependent"},
            origins=[origin("a.json#/0")],
        )
    with pytest.raises(ValidationError, match="the one of phase, material that is present, and 2"):
        w.kind(
            "site_class",
            {
                "index": 1,
                "phase": uuid.uuid4(),
                "material": uuid.uuid4(),
                "ratio_kind": "composition_dependent",
            },
            origins=[origin("a.json#/0")],
        )


def test_the_writer_computes_the_carrier_key_of_an_association_site() -> None:
    w = CanonicalWriter(real_declaration(), carriers_of("a.json"))
    group = uuid.uuid4()
    scheme = w.kind("site_scheme", {"key": "2B"}, origins=[origin("a.json#/0")])
    w.kind(
        "association_site",
        {"scheme": scheme, "label": "H", "on_group": group, "multiplicity": 2.0},
        origins=[origin("a.json#/1")],
    )
    (row,) = w.tables()["tk.association_site"].to_pylist()
    assert row["carrier_key"] == str(group) and row["on_entity"] is None


def carriers_of(*artifacts: str) -> Carriers:
    registry = Carriers()
    registry.add(carrier("src", *artifacts))
    return registry
