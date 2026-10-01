# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""An output whose observable a slot supplies (`observable_from_set`, `output_observables`) and a
form's status, which names no qualification: what the loader refuses, how the declaration is
reified in `meta`, and what the canonical writer checks of the observable a set names."""

from __future__ import annotations

import uuid
from pathlib import Path

import pytest

from declaration_support import NO_PHYSICAL, copy_full, full_declaration
from mapping_support import extended_declaration, origin, writer
from thermo_knowledge.build import build_database
from thermo_knowledge.canonical.writer import CanonicalWriter, ValidationError
from thermo_knowledge.declaration import Declaration, Diagnostic, load_declaration
from thermo_knowledge.testing import TestDatabase

# -- the loader --------------------------------------------------------------------------------

MODULE = """
module = "zz_os"
uses = ["physical", "identity", "framework", "vocab"]
doc = "An output observable under test."

[contracts.curve]
doc = "d"
arguments.T = {{ type = "Temperature", doc = "d" }}
outputs.p = {{ type = "Pressure", {output}, doc = "d" }}
outputs.q = {{ type = "Pressure", observable = "vapor_pressure", doc = "d" }}

[forms.f]
doc = "d"
implements = "curve"
completeness = "opaque_bundle"
status = "{status}"
{declared}

[forms.f.slot_groups.pure]
doc = "d"
subject.i = {{ type = "species_form", doc = "d" }}

[forms.f.slot_groups.pure.slots]
{slots}

[forms.f.slot_groups.pure.families.band]
doc = "d"
index.n = {{ type = "Integer", doc = "d" }}

[forms.f.slot_groups.pure.families.band.slots]
inner = {{ type = "observable", doc = "d" }}
"""
GOOD_SLOTS = 'quantity = { type = "observable", doc = "d" }\nA = { type = "Scalar", doc = "d" }'
DECLARED = '[forms.f.output_observables]\np = "pure.quantity"'
FROM_SET = "observable_from_set = true"


def load(
    tmp_path: Path,
    *,
    output: str = FROM_SET,
    declared: str = DECLARED,
    slots: str = GOOD_SLOTS,
    status: str = "catalogued",
) -> list[Diagnostic]:
    tree = copy_full(tmp_path)
    (tree / "forms" / "zz_os.toml").write_text(
        MODULE.format(output=output, declared=declared, slots=slots, status=status),
        encoding="utf-8",
    )
    return list(load_declaration(tree / "model", tree / "forms", contract=None).diagnostics)


def refused(tmp_path: Path, **kwargs: str) -> list[Diagnostic]:
    found = load(tmp_path, **kwargs)
    assert found, "the declaration was accepted"
    assert all(d.code == "bad-output-observable" for d in found), found
    return found


def test_an_output_that_takes_its_observable_from_a_slot_is_accepted(tmp_path: Path) -> None:
    assert load(tmp_path) == []
    decl = full_declaration()
    output = next(o for o in decl.contracts["saturation_pressure_of_set"].outputs if o.name == "p")
    assert output.observable is None and output.observable_from_set
    (supplied,) = decl.forms["saturation_curve"].output_observables
    assert (supplied.output, supplied.slot_group, supplied.slot) == (
        "p",
        "saturation_curve.pure",
        "quantity",
    )
    assert supplied.qualified == "saturation_curve.pure.quantity"
    assert decl.forms["critical"].output_observables == ()


def test_an_output_names_an_observable_or_takes_it_from_a_set_not_both(tmp_path: Path) -> None:
    (found,) = refused(tmp_path, output='observable = "vapor_pressure", observable_from_set = true')
    assert found.construct == "contracts.curve.outputs.p.observable_from_set"
    assert "not both" in found.message


def test_a_form_declares_the_slot_for_every_output_taken_from_a_set(tmp_path: Path) -> None:
    (found,) = refused(tmp_path, declared="")
    assert found.construct == "forms.f.output_observables.p"
    assert (
        "takes its observable from a set" in found.message
        and "`output_observables`" in found.message
    )


def test_a_form_declares_a_slot_only_for_such_an_output(tmp_path: Path) -> None:
    both = '[forms.f.output_observables]\np = "pure.quantity"\nq = "pure.quantity"'
    (found,) = refused(tmp_path / "named", declared=both)
    assert found.construct == "forms.f.output_observables.q"
    assert (
        "`q` is not an output of contract `curve` that takes its observable from a set"
        in found.message
    )
    (found,) = refused(
        tmp_path / "fixed",
        output='observable = "vapor_pressure"',
        declared='[forms.f.output_observables]\np = "pure.quantity"',
    )
    assert (
        "`p` is not an output of contract `curve` that takes its observable from a set"
        in found.message
    )


@pytest.mark.parametrize(
    ("path", "slots", "fragment"),
    [
        ("nowhere.quantity", GOOD_SLOTS, "`nowhere` is not a slot group of form `f`"),
        ("pure.nothing", GOOD_SLOTS, "`nothing` is not a slot of `f.pure`"),
        ("pure.band.inner", GOOD_SLOTS, "a slot inside a family cannot supply an observable"),
        ("quantity", GOOD_SLOTS, "name a slot as `<slot group>.<slot>`"),
        ("pure.A", GOOD_SLOTS, "the slot is Scalar, not a reference to an observable"),
        (
            "pure.quantity",
            'quantity = { type = "observable", presence = "stateful", doc = "d" }',
            "the slot is stateful",
        ),
        (
            "pure.quantity",
            'quantity = { type = "species", doc = "d" }',
            "the slot is species, not a reference to an observable",
        ),
    ],
)
def test_the_slot_is_a_required_observable_reference_on_a_slot_group(
    tmp_path: Path, path: str, slots: str, fragment: str
) -> None:
    (found,) = refused(
        tmp_path, declared=f'[forms.f.output_observables]\np = "{path}"', slots=slots
    )
    assert found.construct == "forms.f.output_observables.p"
    assert fragment in found.message and f"`{path}`" in found.message


def test_the_observable_role_is_needed(tmp_path: Path) -> None:
    tree = copy_full(tmp_path)
    (tree / "forms" / "zz_os.toml").write_text(
        """
module = "zz_os"
uses = ["physical", "identity", "framework", "vocab"]
doc = "d"

[contracts.curve]
doc = "d"
arguments.T = { type = "Temperature", doc = "d" }
outputs.p = { type = "Pressure", observable_from_set = true, doc = "d" }
""",
        encoding="utf-8",
    )
    manifest = tree / "model" / "manifest.toml"
    manifest.write_text(manifest.read_text().replace('observable = "observable"\n', ""))
    found = load_declaration(tree / "model", tree / "forms", contract=None).diagnostics
    assert any(
        d.code == "framework-role"
        and d.construct == "contracts.curve.outputs.p.observable_from_set"
        for d in found
    ), found


def test_a_form_is_not_declared_qualified(tmp_path: Path) -> None:
    (found,) = [d for d in load(tmp_path, status="qualified") if d.construct == "forms.f.status"]
    assert found.code == "invalid-value"
    assert "qual.form_qualification" in found.message and "catalogued" in found.message
    (other,) = [
        d for d in load(tmp_path / "other", status="proven") if d.construct == "forms.f.status"
    ]
    assert other.construct == "forms.f.status" and "found 'proven'" in other.message


# -- meta --------------------------------------------------------------------------------------


def test_the_declaration_is_reified_in_meta() -> None:
    decl = full_declaration()
    with TestDatabase() as database:
        build_database(database.url, decl, tree=NO_PHYSICAL)
        import psycopg

        with psycopg.connect(database.url) as conn:
            assert conn.execute(
                "SELECT observable IS NULL, observable_from_set FROM meta.contract_output "
                "WHERE contract = 'saturation_pressure_of_set'"
            ).fetchall() == [(True, True)]
            assert conn.execute(
                "SELECT count(*) FROM meta.contract_output WHERE observable_from_set"
            ).fetchone() == (1,)
            assert conn.execute(
                "SELECT form, contract, output, slot FROM meta.form_output_observable"
            ).fetchall() == [
                (
                    "saturation_curve",
                    "saturation_pressure_of_set",
                    "p",
                    "saturation_curve.pure.quantity",
                )
            ]
            assert conn.execute(
                "SELECT s.shape, s.element FROM meta.form_output_observable o "
                "JOIN meta.slot s ON s.qualified_name = o.slot"
            ).fetchall() == [("reference", "observable")]
            # an output names its observable or takes it from a set, never both
            with pytest.raises(psycopg.errors.CheckViolation), conn.transaction():
                conn.execute(
                    "UPDATE meta.contract_output SET observable = %s "
                    "WHERE contract = 'saturation_pressure_of_set'",
                    (uuid.uuid4(),),
                )


# -- the canonical writer ----------------------------------------------------------------------

PURE = "fixture_curve.pure"


@pytest.fixture(scope="module")
def extended(tmp_path_factory: pytest.TempPathFactory) -> Declaration:
    return extended_declaration(tmp_path_factory.mktemp("extended"))


def observable(decl: Declaration, name: str) -> uuid.UUID:
    entity = decl.observable_entity(name)
    assert entity is not None
    return entity.id


def curve(w: CanonicalWriter, named: uuid.UUID, locator: str = "a.json#/2") -> uuid.UUID:
    species = w.kind("species", {"canonical_key": "K", "label": "K"}, origins=[origin("a.json#/0")])
    param = w.kind(
        "parameterization",
        {"key": "k", "revision": "r", "title": "t", "coherence": "independent_records"},
        origins=[origin("a.json#/0", "fitted")],
    )
    return w.parameter_set(
        parameterization=param,
        slot_group=PURE,
        subjects=[species],
        slots={"quantity": named, "A": 1.0},
        origins=[origin(locator, "fitted")],
    )


def test_a_set_names_an_observable_of_the_outputs_dimension(extended: Declaration) -> None:
    w = writer(extended)
    for name in ("vapor_pressure", "pressure"):
        w = writer(extended)
        written = curve(w, observable(extended, name))
        (row,) = w.tables()["param.fixture_curve__pure"].to_pylist()
        assert row["id"] == written and row["quantity"] == observable(extended, name)


def test_an_observable_of_another_dimension_is_refused_with_the_locator(
    extended: Declaration,
) -> None:
    w = writer(extended)
    species = w.kind("species", {"canonical_key": "K", "label": "K"}, origins=[origin("a.json#/0")])
    param = w.kind(
        "parameterization",
        {"key": "k", "revision": "r", "title": "t", "coherence": "independent_records"},
        origins=[origin("a.json#/0", "fitted")],
    )
    before = w.counts()
    with pytest.raises(ValidationError) as caught:
        w.parameter_set(
            parameterization=param,
            slot_group=PURE,
            subjects=[species],
            slots={"quantity": observable(extended, "temperature"), "A": 1.0},
            origins=[origin("a.json#/7", "fitted")],
        )
    message = str(caught.value)
    assert message.startswith("a.json#/7: fixture_curve.pure.quantity: ")
    assert (
        "observable `temperature` is a Temperature quantity of dimension [temperature]" in message
    )
    assert (
        "output `p` of form `fixture_curve` is Pressure, of dimension [mass] / [length] / [time] ** 2"
        in message
    )
    assert w.counts() == before, "nothing of the refused set is written"


def test_a_reference_that_is_no_declared_observable_is_refused(extended: Declaration) -> None:
    w = writer(extended)
    with pytest.raises(ValidationError, match="is not a declared observable"):
        curve(w, uuid.uuid4())
    element = next(e for e in extended.entities if e.kind == "element")
    w = writer(extended)
    with pytest.raises(ValidationError, match="is not a declared observable"):
        curve(w, element.id)
