# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The canonical writer: identifiers, refinements, validation with locators, provenance,
transposition and atomicity."""

from __future__ import annotations

import itertools
import json
import uuid
from pathlib import Path

import pyarrow as pa
import pytest

from mapping_support import (
    SATURATION,
    carrier,
    extended_declaration,
    origin,
    real_declaration,
    writer,
)
from thermo_knowledge.canonical import invariants, store
from thermo_knowledge.canonical.provenance import Carriers
from thermo_knowledge.canonical.values import NotApplicable, Quantity, QuantityArray, Redirect
from thermo_knowledge.canonical.writer import (
    CanonicalWriter,
    CompetingAssertion,
    FamilyRow,
    MissingOrigin,
    TabulatedAxis,
    TabulatedFunction,
    TabulatedSeries,
    ValidationError,
    WriteError,
)
from thermo_knowledge.declaration import Declaration, load_declaration

ROOT = uuid.UUID("5ad3ac16-681d-4c3c-878f-2408aa1bef6f")


def independent(root_name: str, values: list[object]) -> uuid.UUID:
    """An identifier computed from the meta-model's definition (section 5), not with the
    tree's own helper."""
    text = json.dumps(values, ensure_ascii=False, separators=(",", ":"))
    return uuid.uuid5(uuid.uuid5(ROOT, root_name), text)


@pytest.fixture(scope="module")
def extended(tmp_path_factory: pytest.TempPathFactory) -> Declaration:
    return extended_declaration(tmp_path_factory.mktemp("extended"))


def species(w: CanonicalWriter, key: str, locator: str = "a.json#/0") -> uuid.UUID:
    return w.kind("species", {"canonical_key": key, "label": key}, origins=[origin(locator)])


def parameterization(w: CanonicalWriter, locator: str = "a.json#/0") -> uuid.UUID:
    return w.kind(
        "parameterization",
        {"key": "k", "revision": "r", "title": "t", "coherence": "independent_records"},
        origins=[origin(locator, "fitted")],
    )


# -- identifiers and rows ---------------------------------------------------------------------


def test_identifiers_equal_an_independent_computation() -> None:
    decl = real_declaration()
    w = writer(decl)
    sp = species(w, "ATUOYWHBWRKTHZ-UHFFFAOYSA-N")
    assert sp == independent("material_entity", ["ATUOYWHBWRKTHZ-UHFFFAOYSA-N"])
    param = parameterization(w)
    assert param == independent("parameterization", ["k", "r"])
    ps = w.parameter_set(
        parameterization=param,
        slot_group=SATURATION,
        subjects=[sp],
        slots={"T_r": Quantity(400.0, "K"), "p_r": Quantity(4.0, "MPa")},
        origins=[origin("a.json#/1", "fitted")],
    )
    marker = independent("meta:slot_group", [SATURATION])
    subject_key = json.dumps([str(sp)], separators=(",", ":"))
    assert ps == independent("parameter_set", [str(param), str(marker), subject_key, 1])
    tables = w.tables()
    (row,) = tables["tk.parameter_set"].to_pylist()
    assert row["subject_key"] == subject_key
    assert row["slot_group"] == marker
    (group_row,) = tables["param.vapor_pressure_exp_series_tau__pure"].to_pylist()
    assert group_row["p_r"] == pytest.approx(4.0e6)
    assert group_row["slot_group"] == marker
    source = independent("source", ["src@0123456789ab"])
    (artifact,) = (r for r in tables["prov.artifact"].to_pylist() if r["path"] == "a.json")
    assert artifact["id"] == independent("artifact", [str(source), "a.json"])
    assert artifact["carrier"] == source
    import_record = independent("import_record", [str(artifact["id"]), "a.json#/1"])
    record = independent("record_origin", [str(ps), str(import_record)])
    assert record in {r["id"] for r in tables["prov.record_origin"].to_pylist()}


def test_occurrence_distinguishes_repeated_assertions_and_defaults_to_one() -> None:
    w = writer(real_declaration())
    sp = species(w, "K")
    param = parameterization(w)

    def write(occurrence: int | None, p_r: float, locator: str) -> uuid.UUID:
        return w.parameter_set(
            parameterization=param,
            slot_group=SATURATION,
            subjects=[sp],
            slots={"T_r": Quantity(400.0, "K"), "p_r": Quantity(p_r, "MPa")},
            origins=[origin(locator, "fitted")],
            occurrence=occurrence,
        )

    marker = independent("meta:slot_group", [SATURATION])
    subject_key = json.dumps([str(sp)], separators=(",", ":"))

    def expected(occurrence: int) -> uuid.UUID:
        return independent("parameter_set", [str(param), str(marker), subject_key, occurrence])

    default, explicit_one, second, third = (
        write(None, 4.0, "a.json#/1"),
        write(1, 4.0, "a.json#/1"),
        write(2, 5.0, "a.json#/2"),
        write(3, 6.0, "a.json#/3"),
    )
    assert default == explicit_one == expected(1)
    assert (second, third) == (expected(2), expected(3))
    rows = {r["id"]: r for r in w.tables()["tk.parameter_set"].to_pylist()}
    assert {i: r["occurrence"] for i, r in rows.items()} == {default: 1, second: 2, third: 3}
    assert all(r["subject_key"] == subject_key for r in rows.values())
    # the same subject written twice as the same occurrence with other values is a competing assertion
    with pytest.raises(CompetingAssertion):
        write(2, 7.0, "a.json#/4")
    for bad in (0, -1, True, 1.5):
        assert "not a whole number from one" in refused(
            w, lambda bad=bad: write(bad, 4.0, "a.json#/5")
        )  # type: ignore[misc]


def test_an_identity_attribute_with_a_default_takes_it_before_the_identifier_is_computed(
    tmp_path: Path,
) -> None:
    """The rule is general: any identity attribute with a declared default, not only `occurrence`."""
    (tmp_path / "model").mkdir()
    (tmp_path / "model" / "manifest.toml").write_text('doc = "A declaration."\n\n[framework]\n')
    (tmp_path / "model" / "physical.toml").write_text(
        'module = "physical"\nschema = "tk"\ndoc = "d"\n\n[quantity_types.Count]\n'
        'doc = "d"\nunit = "dimensionless"\nscale = "count"\n'
    )
    (tmp_path / "model" / "thing.toml").write_text(
        'module = "thing"\nschema = "tk"\nuses = ["physical"]\ndoc = "d"\n\n[kinds.thing]\n'
        'doc = "d"\nidentity = ["name", "revision"]\nprovenance = "none"\n\n[kinds.thing.attributes]\n'
        'name = { type = "Text", doc = "d" }\nrevision = { type = "Integer", default = 1, doc = "d" }\n'
        'note = { type = "Text", optional = true, doc = "d" }\n'
    )
    decl = load_declaration(tmp_path / "model", tmp_path / "forms", contract=None).require()
    w = CanonicalWriter(decl)
    plain = w.kind("thing", {"name": "x"})
    assert plain == independent("thing", ["x", 1])
    assert w.kind("thing", {"name": "x", "revision": 1}) == plain
    assert w.kind("thing", {"name": "x", "revision": 2}) == independent("thing", ["x", 2])
    (row, _) = sorted(w.tables()["tk.thing"].to_pylist(), key=lambda r: r["revision"])
    assert row["revision"] == 1


def test_a_nested_set_keeps_occurrence_one(extended: Declaration) -> None:
    from thermo_knowledge.canonical.writer import NestedSet

    w = writer(extended)
    low, high, param = nested_parent(w)
    w.parameter_set(
        parameterization=param,
        slot_group=PAIR,
        subjects=[low, high],
        occurrence=2,
        slots={"k": 1.0, "f": NestedSet("fixture_constant.global", {"c": 5.0})},
        origins=[origin("a.json#/2", "fitted")],
    )
    parent, child = sorted(
        w.tables()["tk.parameter_set"].to_pylist(), key=lambda r: r["parent"] is not None
    )
    assert (parent["occurrence"], child["occurrence"]) == (2, 1)
    assert child["subject_key"] == json.dumps(
        [str(parent["id"]), str(independent("meta:slot", [f"{PAIR}.f"])), ""], separators=(",", ":")
    )


def test_a_refinement_emits_parent_and_child_rows_and_one_record() -> None:
    w = writer(real_declaration())
    sp = species(w, "K")
    tables = w.tables()
    parent, child = tables["tk.material_entity"].to_pylist(), tables["tk.species"].to_pylist()
    assert [r["id"] for r in parent] == [r["id"] for r in child] == [sp]
    assert parent[0]["canonical_key"] == "K" and parent[0]["provisional"] is False
    assert child[0]["charge"] == 0.0 and child[0]["inchikey"] is None
    assert tables["prov.record"].to_pylist() == [{"id": sp, "kind": "species"}]
    (origin_row,) = tables["prov.record_origin"].to_pylist()
    assert origin_row["record"] == sp and origin_row["role"] == "published"


def test_provenance_rows_are_created_from_the_carrier_and_the_origin() -> None:
    w = writer(real_declaration())
    species(w, "K", "a.json#/0")
    species(w, "L", "a.json#/1")
    tables = w.tables()
    assert tables["prov.carrier"].num_rows == tables["prov.source"].num_rows == 1
    (carrier_row,) = tables["prov.carrier"].to_pylist()
    assert carrier_row["manifest_id"] == "src" and carrier_row["resolved_pin"] == "0123456789ab"
    assert tables["prov.artifact"].num_rows == 1
    assert tables["prov.import_record"].num_rows == 2
    (licence,) = tables["prov.licence"].to_pylist()
    (source,) = tables["prov.source"].to_pylist()
    assert source["key"] == "src@0123456789ab"
    (rights,) = tables["prov.rights_determination"].to_pylist()
    assert rights["ordinal"] == 1 and rights["licence"] == licence["id"]
    assert rights["basis"] == "licence_grant" and rights["attribution_required"] is True


def test_rights_without_a_licence_and_competing_statements_keep_their_order() -> None:
    info = carrier("src", "a.json", spdx=None)
    info.rights.append(info.rights[0])
    w = writer(real_declaration(), info)
    species(w, "K")
    tables = w.tables()
    assert "prov.licence" not in tables
    assert sorted(r["ordinal"] for r in tables["prov.rights_determination"].to_pylist()) == [1, 2]


# -- validation, each refusal reported with the source locator --------------------------------


def refused(w: CanonicalWriter, call: object) -> str:
    before = w.counts()
    with pytest.raises(ValidationError) as caught:
        call()  # type: ignore[operator]
    assert w.counts() == before, "nothing of a refused record is written"
    return str(caught.value)


def test_a_missing_required_value_is_reported_with_the_locator() -> None:
    w = writer(real_declaration())
    message = refused(
        w,
        lambda: w.kind(
            "parameterization",
            {"key": "k", "revision": "r", "coherence": "independent_records"},
            origins=[origin("a.json#/7")],
        ),
    )
    assert message.startswith("a.json#/7: ") and "title: a required value is missing" in message


def test_a_bad_enum_member_is_reported_with_the_locator() -> None:
    w = writer(real_declaration())
    message = refused(
        w,
        lambda: w.kind(
            "parameterization",
            {"key": "k", "revision": "r", "title": "t", "coherence": "sometimes"},
            origins=[origin("a.json#/8")],
        ),
    )
    assert "a.json#/8" in message and "'sometimes' is not a member of enum `coherence`" in message


def test_an_unknown_attribute_and_a_wrong_type_are_reported_together() -> None:
    w = writer(real_declaration())
    message = refused(
        w,
        lambda: w.kind(
            "species",
            {"canonical_key": 3, "label": "x", "colour": "red"},
            origins=[origin("a.json#/9")],
        ),
    )
    assert "colour: is not an attribute" in message and "canonical_key: 3 is not a Text" in message


def test_a_failed_ddl_check_is_reported_before_the_database_sees_it() -> None:
    w = writer(real_declaration())
    sp = species(w, "K")
    ps = w.parameter_set(
        parameterization=parameterization(w),
        slot_group=SATURATION,
        subjects=[sp],
        slots={"T_r": Quantity(400.0, "K"), "p_r": Quantity(1e6, "Pa")},
        origins=[origin("a.json#/1", "fitted")],
    )
    temperature = next(
        e for e in w.decl.entities if e.kind == "observable" and e.name == "temperature"
    )
    message = refused(
        w,
        lambda: w.validity_region(
            ps,
            {"kind": "fitted_range"},
            [
                {
                    "observable": temperature.id,
                    "lower": Quantity(500.0, "K"),
                    "upper": Quantity(300.0, "K"),
                }
            ],
            origins=[origin("a.json#/1", "fitted")],
            at="a.json#/1",
        ),
    )
    assert message.startswith("a.json#/1: ") and "`bounds_ordered`" in message
    message = refused(
        w,
        lambda: w.kind(
            "artifact",
            {"carrier": uuid.uuid4(), "path": "x", "sha256": "ab" * 32, "size": -1},
            at="a.json#/2",
        ),
    )
    assert "`size_nonnegative`" in message


def test_a_failed_unit_conversion_is_reported_with_the_locator() -> None:
    w = writer(real_declaration())
    sp = species(w, "K")
    param = parameterization(w)

    def write(slots: dict[str, object]) -> uuid.UUID:
        return w.parameter_set(
            parameterization=param,
            slot_group=SATURATION,
            subjects=[sp],
            slots=slots,
            origins=[origin("a.json#/3", "fitted")],
        )

    message = refused(w, lambda: write({"T_r": Quantity(400.0, "Pa"), "p_r": Quantity(1.0, "Pa")}))
    assert (
        message.startswith("a.json#/3: ")
        and "T_r: a value in `Pa` cannot be converted to `K`" in message
    )
    message = refused(
        w, lambda: write({"T_r": Quantity(1.0, "no_such_unit"), "p_r": Quantity(1.0, "Pa")})
    )
    assert "`no_such_unit` is not a unit pint can parse" in message
    message = refused(w, lambda: write({"T_r": 400.0, "p_r": Quantity(1.0, "Pa")}))
    assert "give the value with its source unit" in message
    message = refused(w, lambda: write({"T_r": Quantity(-5.0, "K"), "p_r": Quantity(1.0, "Pa")}))
    assert "is negative" in message
    message = refused(
        w, lambda: write({"T_r": Quantity(float("nan"), "K"), "p_r": Quantity(1.0, "Pa")})
    )
    assert "is not finite" in message


def test_units_convert_through_pint_to_the_storage_unit() -> None:
    w = writer(real_declaration())
    sp = species(w, "K")
    w.parameter_set(
        parameterization=parameterization(w),
        slot_group=SATURATION,
        subjects=[sp],
        slots={"T_r": Quantity(26.85, "degC"), "p_r": Quantity(2.0, "bar")},
        families={"term": [FamilyRow({"k": 1}, {"n": 1.5, "t": 2.0})]},
        origins=[origin("a.json#/1", "fitted")],
    )
    (row,) = w.tables()["param.vapor_pressure_exp_series_tau__pure"].to_pylist()
    assert row["T_r"] == pytest.approx(300.0) and row["p_r"] == pytest.approx(2.0e5)


def test_family_rows_are_validated() -> None:
    w = writer(real_declaration())
    sp = species(w, "K")
    param = parameterization(w)

    def write(rows: list[FamilyRow]) -> uuid.UUID:
        return w.parameter_set(
            parameterization=param,
            slot_group=SATURATION,
            subjects=[sp],
            slots={"T_r": Quantity(400.0, "K"), "p_r": Quantity(1e6, "Pa")},
            families={"term": rows},
            origins=[origin("a.json#/4", "fitted")],
        )

    assert "k: 0 is below the minimum 1" in refused(
        w, lambda: write([FamilyRow({"k": 0}, {"n": 1.0, "t": 1.0})])
    )
    assert "the index (1,) is repeated" in refused(
        w, lambda: write([FamilyRow({"k": 1}, {"n": 1.0, "t": 1.0})] * 2)
    )
    assert "slot `t` is missing" in refused(w, lambda: write([FamilyRow({"k": 1}, {"n": 1.0})]))
    assert "x: is not a family" in refused(
        w,
        lambda: w.parameter_set(
            parameterization=param,
            slot_group=SATURATION,
            subjects=[sp],
            slots={"T_r": Quantity(400.0, "K"), "p_r": Quantity(1e6, "Pa")},
            families={"x": []},
            origins=[origin("a.json#/4", "fitted")],
        ),
    )


def test_the_writer_refuses_to_start_when_a_load_invariant_has_no_evaluator(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    assert invariants.missing_evaluators(real_declaration()) == []
    monkeypatch.delitem(
        invariants.LOAD_INVARIANTS, ("standard_state", "dilution_needs_solvent_and_scale")
    )
    with pytest.raises(WriteError, match="standard_state.dilution_needs_solvent_and_scale"):
        CanonicalWriter(real_declaration(), Carriers())


def test_a_kind_that_cannot_be_instantiated_directly_is_refused() -> None:
    w = writer(real_declaration())
    assert "is abstract" in refused(
        w,
        lambda: w.kind("material_entity", {"canonical_key": "x", "label": "x"}, origins=[origin()]),
    )
    assert "is a slot group" in refused(
        w, lambda: w.kind("vapor_pressure_exp_series_tau__pure", {}, origins=[origin()])
    )
    assert "is not a kind" in refused(w, lambda: w.kind("nothing", {}, at="x"))


# -- provenance --------------------------------------------------------------------------------


def test_a_record_without_an_origin_is_refused() -> None:
    w = writer(real_declaration())
    with pytest.raises(MissingOrigin, match=r"a\.json#/0: the species .* has no origin"):
        w.kind("species", {"canonical_key": "K", "label": "K"}, at="a.json#/0")
    assert w.counts() == {}, "a refused record leaves no rows, not even its provenance"
    sp = species(w, "K")
    with pytest.raises(MissingOrigin):
        w.parameter_set(
            parameterization=parameterization(w),
            slot_group=SATURATION,
            subjects=[sp],
            slots={"T_r": Quantity(400.0, "K"), "p_r": Quantity(1e6, "Pa")},
            origins=[],
            at="a.json#/1",
        )


def test_origins_are_refused_for_a_kind_that_registers_no_record() -> None:
    w = writer(real_declaration())
    message = refused(
        w, lambda: w.kind("licence", {"key": "MIT", "title": "MIT"}, origins=[origin()])
    )
    assert "registers no record" in message


def test_one_record_may_have_several_origins_and_an_artifact_must_be_known() -> None:
    w = writer(real_declaration())
    sp = w.kind(
        "species",
        {"canonical_key": "K", "label": "K"},
        origins=[origin("a.json#/0"), origin("b.json#/0", "evaluated")],
    )
    origin_rows = [r for r in w.tables()["prov.record_origin"].to_pylist() if r["record"] == sp]
    assert sorted(r["role"] for r in origin_rows) == ["evaluated", "published"]
    message = refused(
        w,
        lambda: w.kind(
            "species", {"canonical_key": "L", "label": "L"}, origins=[origin("zzz.json#/0")]
        ),
    )
    assert "zzz.json" in message and "cannot be hashed" in message


def test_competing_assertions_name_both_locators() -> None:
    w = writer(real_declaration())
    species(w, "K", "a.json#/1")
    species(w, "K", "a.json#/1")  # the same content again is one row
    assert w.rows("tk.species") == 1
    with pytest.raises(CompetingAssertion) as caught:
        w.kind(
            "species",
            {"canonical_key": "K", "label": "another label"},
            origins=[origin("b.json#/2")],
        )
    message = str(caught.value)
    assert "a.json#/1" in message and "b.json#/2" in message and "label" in message
    assert w.rows("tk.species") == 1


# -- transposition ----------------------------------------------------------------------------


def ordered_pair(w: CanonicalWriter) -> tuple[uuid.UUID, uuid.UUID]:
    first, second = species(w, "A", "a.json#/0"), species(w, "B", "a.json#/1")
    return (first, second) if first < second else (second, first)


def test_symmetric_subjects_are_stored_in_the_canonical_orientation(extended: Declaration) -> None:
    w = writer(extended)
    low, high = ordered_pair(w)
    param = parameterization(w)

    def write(subjects: list[uuid.UUID], locator: str, k: float = 0.25) -> uuid.UUID:
        return w.parameter_set(
            parameterization=param,
            slot_group="fixture_symmetric.pair",
            subjects=subjects,
            slots={"k": k, "T_ref": Quantity(300.0, "K")},
            origins=[origin(locator, "fitted")],
        )

    forward = write([low, high], "a.json#/2")
    assert write([high, low], "a.json#/3") == forward, "both orientations are one set"
    (row,) = w.tables()["param.fixture_symmetric__pair"].to_pylist()
    assert (row["i"], row["j"]) == (low, high) and row["k"] == 0.25
    (parent,) = w.tables()["tk.parameter_set"].to_pylist()
    assert parent["subject_key"] == json.dumps([str(low), str(high)], separators=(",", ":"))
    with pytest.raises(CompetingAssertion):
        write([high, low], "a.json#/4", k=0.5)
    assert "forbids the diagonal" in refused(w, lambda: write([low, low], "a.json#/5"))


def test_a_reciprocal_pair_asserted_in_the_other_order_is_stored_as_asserted(
    extended: Declaration,
) -> None:
    w = writer(extended)
    low, high = ordered_pair(w)
    param = parameterization(w)
    asserted = w.parameter_set(
        parameterization=param,
        slot_group="fixture_reciprocal.pair",
        subjects=[high, low],
        slots={"r": 0.25, "other": 3.0},
        origins=[origin("a.json#/3", "fitted")],
    )
    (row,) = w.tables()["param.fixture_reciprocal__pair"].to_pylist()
    assert row["id"] == asserted
    assert (row["i"], row["j"]) == (low, high), "the subject columns are canonical"
    assert (row["r"], row["other"]) == (0.25, 3.0), "the values are the asserted numbers"
    assert row["arrangement"] == 1, "they were asserted for the swapped order"
    (parent,) = w.tables()["tk.parameter_set"].to_pylist()
    assert parent["subject_key"] == json.dumps([str(low), str(high)], separators=(",", ":"))


def test_the_same_pair_asserted_in_both_orders_is_two_assertions_of_one_set(
    extended: Declaration,
) -> None:
    w = writer(extended)
    low, high = ordered_pair(w)
    param = parameterization(w)

    def assert_pair(
        subjects: list[uuid.UUID], r: float, locator: str, occurrence: int | None = None
    ):  # noqa: ANN202
        return w.parameter_set(
            parameterization=param,
            slot_group="fixture_reciprocal.pair",
            subjects=subjects,
            slots={"r": r, "other": 3.0},
            origins=[origin(locator, "fitted")],
            occurrence=occurrence,
        )

    direct = assert_pair([low, high], 4.0, "a.json#/2")
    assert assert_pair([low, high], 4.0, "a.json#/4") == direct, "the same assertion twice"
    with pytest.raises(CompetingAssertion):
        assert_pair([high, low], 0.25, "a.json#/3")
    second = assert_pair([high, low], 0.25, "a.json#/5", occurrence=2)
    assert second != direct, "another occurrence is another set"
    rows = {r["id"]: r for r in w.tables()["param.fixture_reciprocal__pair"].to_pylist()}
    assert (rows[direct]["r"], rows[direct]["arrangement"]) == (4.0, 0)
    assert (rows[second]["r"], rows[second]["arrangement"]) == (0.25, 1)


def test_a_reciprocal_slot_holding_zero_is_stored(extended: Declaration) -> None:
    w = writer(extended)
    low, high = ordered_pair(w)
    w.parameter_set(
        parameterization=parameterization(w),
        slot_group="fixture_reciprocal.pair",
        subjects=[high, low],
        slots={"r": 0.0, "other": 3.0},
        origins=[origin("a.json#/3", "fitted")],
    )
    (row,) = w.tables()["param.fixture_reciprocal__pair"].to_pylist()
    assert (row["r"], row["arrangement"]) == (0.0, 1)


def test_parity_family_rows_are_stored_as_asserted_when_the_subjects_are_swapped(
    extended: Declaration,
) -> None:
    w = writer(extended)
    low, high = ordered_pair(w)
    w.parameter_set(
        parameterization=parameterization(w),
        slot_group="fixture_parity.pair",
        subjects=[high, low],
        slots={},
        families={"term": [FamilyRow({"order": n}, {"c": float(n + 1)}) for n in range(4)]},
        origins=[origin("a.json#/2", "fitted")],
    )
    rows = w.tables()["param.fixture_parity__pair__term"].to_pylist()
    assert [(r["order"], r["c"]) for r in rows] == [(0, 1.0), (1, 2.0), (2, 3.0), (3, 4.0)]
    (head,) = w.tables()["param.fixture_parity__pair"].to_pylist()
    assert (head["i"], head["j"], head["arrangement"]) == (low, high, 1)


def test_a_permutation_group_has_one_canonical_representative_and_records_the_arrangement(
    extended: Declaration,
) -> None:
    arrangements: dict[tuple[uuid.UUID, ...], tuple[uuid.UUID, int]] = {}
    for order in itertools.permutations(range(3)):
        w = writer(extended)
        members = sorted(species(w, key, f"a.json#/{n}") for n, key in enumerate("XYZ"))
        asserted = tuple(members[position] for position in order)
        identifier = w.parameter_set(
            parameterization=parameterization(w),
            slot_group="fixture_group.triple",
            subjects=list(asserted),
            slots={"v": 2.0},
            origins=[origin("a.json#/9", "fitted")],
        )
        (row,) = w.tables()["param.fixture_group__triple"].to_pylist()
        assert [row["a"], row["b"], row["c"]] == members, "the canonical representative"
        assert row["v"] == 2.0
        arrangements[asserted] = (identifier, row["arrangement"])
    assert len({identifier for identifier, _ in arrangements.values()}) == 1, "one set"
    assert sorted(number for _, number in arrangements.values()) == list(range(6)), (
        "each of the six arrangements of the group the two generators make has its own number"
    )


def test_stateful_slots_and_piecewise_families(extended: Declaration) -> None:
    w = writer(extended)
    sp = species(w, "K")
    other = species(w, "L", "a.json#/1")
    param = parameterization(w)
    first = w.parameter_set(
        parameterization=param,
        slot_group="fixture_plain.pure",
        subjects=[sp],
        slots={"T_c": NotApplicable()},
        families={
            "piece": [
                FamilyRow({"n": 1}, {"T_low": Quantity(200.0, "K"), "T_high": Quantity(400.0, "K")})
            ]
        },
        origins=[origin("a.json#/2", "fitted")],
    )
    w.parameter_set(
        parameterization=param,
        slot_group="fixture_plain.pure",
        subjects=[other],
        slots={"T_c": Redirect(first)},
        origins=[origin("a.json#/3", "fitted")],
    )
    third = species(w, "M", "a.json#/4")
    w.parameter_set(
        parameterization=param,
        slot_group="fixture_plain.pure",
        subjects=[third],
        slots={"T_c": Quantity(300.0, "K")},
        origins=[origin("a.json#/5", "fitted")],
    )
    rows = {r["i"]: r for r in w.tables()["param.fixture_plain__pure"].to_pylist()}
    assert rows[sp]["T_c"] is None and rows[sp]["T_c__state"] == "not_applicable"
    assert rows[other]["T_c__state"] == "redirect" and rows[other]["T_c__redirect"] == first
    assert rows[third]["T_c"] == 300.0 and rows[third]["T_c__state"] == "known"
    fourth, fifth = species(w, "N", "a.json#/6"), species(w, "P", "a.json#/8")
    assert "a stateful slot states a value or why it has none" in refused(
        w,
        lambda: w.parameter_set(
            parameterization=param,
            slot_group="fixture_plain.pure",
            subjects=[fourth],
            slots={},
            origins=[origin("a.json#/7", "fitted")],
        ),
    )
    overlapping = [
        FamilyRow({"n": 1}, {"T_low": Quantity(200.0, "K"), "T_high": Quantity(400.0, "K")}),
        FamilyRow({"n": 2}, {"T_low": Quantity(300.0, "K"), "T_high": Quantity(500.0, "K")}),
    ]
    assert "pieces overlap" in refused(
        w,
        lambda: w.parameter_set(
            parameterization=param,
            slot_group="fixture_plain.pure",
            subjects=[fifth],
            slots={"T_c": NotApplicable()},
            families={"piece": overlapping},
            origins=[origin("a.json#/9", "fitted")],
        ),
    )


# -- atomicity and order ----------------------------------------------------------------------


def test_a_transaction_withdraws_everything_it_wrote() -> None:
    w = writer(real_declaration())
    species(w, "kept")
    before = w.counts()
    with pytest.raises(ValidationError), w.transaction():
        species(w, "dropped", "a.json#/1")
        w.kind("species", {"canonical_key": "bad"}, origins=[origin("a.json#/2")])
    assert w.counts() == before
    species(w, "dropped", "a.json#/1")
    assert w.rows("tk.species") == 2 and w.rows("prov.import_record") == 2
    with w.transaction(), pytest.raises(WriteError, match="do not nest"), w.transaction():
        pass


def test_the_same_records_in_any_order_are_the_same_tables() -> None:
    def build(order: list[str]) -> dict[str, pa.Table]:
        w = writer(real_declaration())
        for n, key in enumerate(order):
            species(w, key, "a.json#/0" if key == "K1" else f"b.json#/{n}")
        return w.tables()

    forward, backward = build(["K1", "K2", "K3"]), build(["K3", "K2", "K1"])
    # the locators of K2 and K3 differ by position, so compare the tables that do not cite them
    for name in ("tk.species", "tk.material_entity", "prov.record"):
        assert forward[name].equals(backward[name])


def test_parquet_files_of_equal_tables_are_equal_bytes(tmp_path: Path) -> None:
    def build() -> dict[str, pa.Table]:
        w = writer(real_declaration())
        species(w, "K")
        return w.tables()

    first, second = tmp_path / "one", tmp_path / "two"
    first.mkdir(), second.mkdir()
    one, two = store.write_tables(first, build()), store.write_tables(second, build())
    assert one == two
    for name in one:
        assert (first / one[name].file).read_bytes() == (second / two[name].file).read_bytes()


# -- the unit of a Real value comes from the declaration --------------------------------------


def test_the_unit_of_a_real_follows_a_dotted_path_of_references() -> None:
    """`datum.value` takes the unit of the observable of its column: `unit_from = "column.observable"`."""
    decl = real_declaration()
    w = writer(decl)
    entities = {e.name: e.id for e in decl.entities if e.kind == "observable"}
    dataset = w.kind(
        "dataset",
        {
            "carrier": independent("source", ["src@0123456789ab"]),
            "local_key": "d",
            "kind": "measured",
        },
        origins=[origin("a.json#/0", "measured")],
    )
    temperature = w.kind(
        "dataset_column",
        {
            "dataset": dataset,
            "ordinal": 1,
            "role": "variable",
            "observable": entities["temperature"],
        },
        at="a.json#/0",
    )
    pressure = w.kind(
        "dataset_column",
        {
            "dataset": dataset,
            "ordinal": 2,
            "role": "constraint",
            "observable": entities["pressure"],
            "constant": Quantity(2.0, "bar"),
        },
        at="a.json#/0",
    )
    point = w.kind("data_point", {"dataset": dataset, "index": 1}, at="a.json#/0")

    def datum(column: uuid.UUID, value: Quantity | float) -> uuid.UUID:
        return w.relation(
            "datum",
            {"point": point, "column": column},
            {"state": "known", "value": value},
            at="a.json#/1",
        )

    datum(temperature, Quantity(25.0, "degC"))
    (row,) = w.tables()["ev.datum"].to_pylist()
    assert row["value"] == pytest.approx(298.15)
    (constraint,) = [r for r in w.tables()["ev.dataset_column"].to_pylist() if r["id"] == pressure]
    assert constraint["constant"] == pytest.approx(2.0e5), (
        "a column's constant takes its own observable's unit"
    )
    assert "cannot be converted to `K`" in refused(
        w, lambda: datum(temperature, Quantity(1.0, "Pa"))
    )
    unknown = uuid.uuid4()
    assert "neither declared nor written before this record" in refused(
        w,
        lambda: w.relation(
            "datum",
            {"point": point, "column": unknown},
            {"state": "known", "value": Quantity(1.0, "K")},
            at="a.json#/2",
        ),
    )
    assert "give the value with its source unit" in refused(w, lambda: datum(temperature, 3.0))
    assert "a constraint column states its constant" in refused(
        w,
        lambda: w.kind(
            "dataset_column",
            {
                "dataset": dataset,
                "ordinal": 3,
                "role": "constraint",
                "observable": entities["pressure"],
            },
            at="a.json#/3",
        ),
    )


def test_a_slot_reference_fixes_the_unit_of_an_uncertainty() -> None:
    w = writer(real_declaration())
    sp = species(w, "K")
    ps = w.parameter_set(
        parameterization=parameterization(w),
        slot_group=SATURATION,
        subjects=[sp],
        slots={"T_r": Quantity(400.0, "K"), "p_r": Quantity(1e6, "Pa")},
        origins=[origin("a.json#/1", "fitted")],
    )
    slot = independent("meta:slot", [f"{SATURATION}.p_r"])
    w.kind(
        "slot_uncertainty",
        {
            "parameter_set": ps,
            "slot": slot,
            "index_key": "",
            "kind": "standard",
            "magnitude": Quantity(2.0, "bar"),
        },
        at="a.json#/1",
    )
    (row,) = w.tables()["tk.slot_uncertainty"].to_pylist()
    assert row["magnitude"] == pytest.approx(2.0e5)


# -- tabulated functions -----------------------------------------------------------------------

PROFILE = "fixture_profile.pure"


def profile_set(
    w: CanonicalWriter,
    value: object,
    *,
    families: dict[str, list[FamilyRow]] | None = None,
    prepared: tuple[uuid.UUID, uuid.UUID] | None = None,
) -> uuid.UUID:
    sp, param = prepared or (species(w, "K"), parameterization(w))
    return w.parameter_set(
        parameterization=param,
        slot_group=PROFILE,
        subjects=[sp],
        slots={"profile": value},
        families=families,
        origins=[origin("a.json#/2", "fitted")],
    )


def sigma_profile() -> TabulatedFunction:
    return TabulatedFunction(
        "linear",
        axes=[TabulatedAxis("Scalar", QuantityArray([-1.0, 0.0, 1.0], "dimensionless"))],
        series=[
            TabulatedSeries("nhb", "Scalar", QuantityArray([0.1, 0.2, 0.3], "dimensionless")),
            TabulatedSeries("oh", "Scalar", QuantityArray([0.0, 0.5, 0.0], "dimensionless")),
            TabulatedSeries("other", "Scalar", QuantityArray([1.0, 1.0, 1.0], "dimensionless")),
        ],
    )


def test_a_tabulated_slot_writes_the_function_with_its_axes_and_series(
    extended: Declaration,
) -> None:
    w = writer(extended)
    written = profile_set(w, sigma_profile())
    tables = w.tables()
    (function,) = tables["tk.tabulated_function"].to_pylist()
    slot = independent("meta:slot", [f"{PROFILE}.profile"])
    key = json.dumps([str(written), str(slot), ""], separators=(",", ":"))
    assert (
        function["key"] == key
        and function["interpolation"] == "linear"
        and function["distribution"] is None
    )
    assert function["id"] == independent("tabulated_function", [key])
    (held,) = tables["param.fixture_profile__pure"].to_pylist()
    assert held["profile"] == function["id"]
    (axis,) = tables["tk.tabulated_axis"].to_pylist()
    assert (
        axis["function"] == function["id"]
        and axis["ordinal"] == 1
        and axis["points"] == [-1.0, 0.0, 1.0]
    )
    assert axis["axis_type"] == independent("meta:quantity_type", ["Scalar"])
    series = {r["name"]: r for r in tables["tk.tabulated_series"].to_pylist()}
    assert sorted(series) == ["nhb", "oh", "other"] and series["oh"]["values"] == [0.0, 0.5, 0.0]
    assert all(r["function"] == function["id"] for r in series.values())
    records = {r["id"] for r in tables["prov.record"].to_pylist()}
    assert function["id"] in records and axis["id"] not in records, (
        "axes and series share their function's record"
    )
    origins = [r for r in tables["prov.record_origin"].to_pylist() if r["record"] == function["id"]]
    assert len(origins) == 1 and origins[0]["role"] == "fitted"


def test_values_and_points_convert_to_the_unit_of_their_quantity_type(
    extended: Declaration,
) -> None:
    w = writer(extended)
    profile_set(
        w,
        TabulatedFunction(
            "linear",
            axes=[TabulatedAxis("Temperature", QuantityArray([25.0, 50.0], "degC"))],
            series=[TabulatedSeries("p", "Pressure", QuantityArray([1.0, 2.0], "bar"))],
        ),
    )
    (axis,) = w.tables()["tk.tabulated_axis"].to_pylist()
    (series,) = w.tables()["tk.tabulated_series"].to_pylist()
    assert axis["points"] == pytest.approx([298.15, 323.15]) and series["values"] == pytest.approx(
        [1e5, 2e5]
    )
    assert "cannot be converted to `K`" in refused(
        w,
        lambda: profile_set(
            w,
            TabulatedFunction(
                "linear",
                axes=[TabulatedAxis("Temperature", QuantityArray([1.0, 2.0], "Pa"))],
                series=[TabulatedSeries("p", "Pressure", QuantityArray([1.0, 2.0], "Pa"))],
            ),
        ),
    )


def test_a_tabulated_function_on_two_axes_takes_row_major_values(extended: Declaration) -> None:
    w = writer(extended)
    kernel = TabulatedFunction(
        "linear",
        axes=[
            TabulatedAxis("Length", QuantityArray([1.0, 2.0], "nm")),
            TabulatedAxis("Pressure", QuantityArray([1.0, 2.0, 3.0], "bar")),
        ],
        series=[
            TabulatedSeries(
                "uptake", "Loading", QuantityArray([float(n) for n in range(6)], "mol/kg")
            )
        ],
    )
    profile_set(w, sigma_profile(), families={"band": [FamilyRow({"n": 1}, {"kernel": kernel})]})
    axes = sorted(
        w.tables()["tk.tabulated_axis"].to_pylist(),
        key=lambda r: (str(r["function"]), r["ordinal"]),
    )
    two = [r for r in axes if r["ordinal"] == 2]
    assert len(two) == 1 and two[0]["points"] == pytest.approx([1e5, 2e5, 3e5])
    functions = w.tables()["tk.tabulated_function"].to_pylist()
    assert len(functions) == 2
    (band,) = w.tables()["param.fixture_profile__pure__band"].to_pylist()
    held = next(f for f in functions if f["id"] == band["kernel"])
    assert json.loads(held["key"])[2] == json.dumps([1], separators=(",", ":"))


def test_a_tabulated_function_is_refused_with_the_locator_when_its_grid_is_wrong(
    extended: Declaration,
) -> None:
    w = writer(extended)
    prepared = (species(w, "K"), parameterization(w))

    def write(value: object) -> uuid.UUID:
        return profile_set(w, value, prepared=prepared)

    def grid(
        points: list[float], values: list[float], *, repeated: bool = False
    ) -> TabulatedFunction:
        series = [TabulatedSeries("a", "Scalar", QuantityArray(values, "dimensionless"))]
        if repeated:
            series.append(series[0])
        return TabulatedFunction(
            "linear",
            axes=[TabulatedAxis("Scalar", QuantityArray(points, "dimensionless"))],
            series=series,
        )

    profile = sigma_profile()
    message = refused(w, lambda: write(grid([1.0, 2.0, 2.0], [1.0, 2.0, 3.0])))
    assert message.startswith("a.json#/2: ")
    assert (
        "fixture_profile.pure.profile.axes[1]: the axis points are not strictly ascending"
        in message
    )
    message = refused(w, lambda: write(grid([1.0, 2.0, 3.0], [1.0, 2.0])))
    assert (
        "fixture_profile.pure.profile.series[a]: has 2 values, the grid of the axes (3) has 3 points"
        in message
    )
    assert "the series name is repeated" in refused(
        w, lambda: write(grid([1.0, 2.0], [1.0, 2.0], repeated=True))
    )
    assert "at least one axis" in refused(
        w, lambda: write(TabulatedFunction("linear", axes=[], series=profile.series))
    )
    assert "at least one series" in refused(
        w, lambda: write(TabulatedFunction("linear", axes=profile.axes, series=[]))
    )
    assert "a tabulated-function slot takes a TabulatedFunction" in refused(
        w, lambda: write(uuid.uuid4())
    )
    assert "is not a member of enum `interpolation`" in refused(
        w, lambda: write(TabulatedFunction("spline", axes=profile.axes, series=profile.series))
    )


# -- nested sets -------------------------------------------------------------------------------

PAIR = "fixture_nested_pair.pair"
LINEAR = "fixture_linear.global"


def nested_parent(w: CanonicalWriter) -> tuple[uuid.UUID, uuid.UUID, uuid.UUID]:
    low, high = ordered_pair(w)
    return low, high, parameterization(w)


def test_a_slot_can_hold_a_nested_set_of_a_form_implementing_its_contract(
    extended: Declaration,
) -> None:
    from thermo_knowledge.canonical.writer import NestedSet

    w = writer(extended)
    low, high, param = nested_parent(w)
    parent = w.parameter_set(
        parameterization=param,
        slot_group=PAIR,
        subjects=[high, low],
        slots={
            "k": 1.0,
            "f": NestedSet(LINEAR, {"a": 1.0, "b": 2.0, "T_ref": Quantity(25.0, "degC")}),
        },
        origins=[origin("a.json#/2", "fitted")],
    )
    tables = w.tables()
    sets = {r["id"]: r for r in tables["tk.parameter_set"].to_pylist()}
    (child,) = [r for r in sets.values() if r["parent"] is not None]
    slot = independent("meta:slot", [f"{PAIR}.f"])
    assert child["parent"] == parent and child["parent_slot"] == slot
    assert child["subject_key"] == json.dumps([str(parent), str(slot), ""], separators=(",", ":"))
    assert child["id"] == independent(
        "parameter_set",
        [str(param), str(independent("meta:slot_group", [LINEAR])), child["subject_key"], 1],
    )
    assert "role" not in child and child["parameterization"] == param
    (value,) = tables["param.fixture_linear__global"].to_pylist()
    assert value["id"] == child["id"] and value["T_ref"] == pytest.approx(298.15)
    (held,) = tables["param.fixture_nested_pair__pair"].to_pylist()
    assert held["f"] == child["id"] and held["id"] == parent
    assert sets[parent]["parent"] is None
    records = {r["id"] for r in tables["prov.record"].to_pylist()}
    assert {parent, child["id"]} <= records
    origins = [r for r in tables["prov.record_origin"].to_pylist() if r["record"] == child["id"]]
    assert len(origins) == 1 and origins[0]["role"] == "fitted"


def test_a_nested_set_inside_a_family_row_is_keyed_by_its_index(extended: Declaration) -> None:
    from thermo_knowledge.canonical.writer import NestedSet

    w = writer(extended)
    low, high, param = nested_parent(w)
    parent = w.parameter_set(
        parameterization=param,
        slot_group=PAIR,
        subjects=[low, high],
        slots={"k": 1.0, "f": NestedSet("fixture_constant.global", {"c": 5.0})},
        families={
            "term": [
                FamilyRow(
                    {"order": n},
                    {
                        "c": 0.5,
                        "g": NestedSet(
                            LINEAR, {"a": float(n), "b": 1.0, "T_ref": Quantity(300.0, "K")}
                        ),
                    },
                )
                for n in (0, 1)
            ]
        },
        origins=[origin("a.json#/2", "fitted")],
    )
    tables = w.tables()
    sets = {r["id"]: r for r in tables["tk.parameter_set"].to_pylist()}
    children = [r for r in sets.values() if r["parent"] == parent]
    assert len(children) == 3
    slot = independent("meta:slot", ["fixture_nested_pair.pair.term.g"])
    keys = {r["subject_key"] for r in children if r["parent_slot"] == slot}
    assert keys == {
        json.dumps(
            [str(parent), str(slot), json.dumps([n], separators=(",", ":"))], separators=(",", ":")
        )
        for n in (0, 1)
    }
    rows = {r["order"]: r for r in tables["param.fixture_nested_pair__pair__term"].to_pylist()}
    assert {rows[n]["g"] for n in (0, 1)} == {r["id"] for r in children if r["parent_slot"] == slot}
    linear = {r["id"]: r for r in tables["param.fixture_linear__global"].to_pylist()}
    assert sorted(linear[rows[n]["g"]]["a"] for n in (0, 1)) == [0.0, 1.0]


def test_a_nested_set_may_have_subjects_and_nest_again(extended: Declaration) -> None:
    from thermo_knowledge.canonical.writer import NestedSet

    w = writer(extended)
    sp = species(w, "K")
    param = parameterization(w)
    w.parameter_set(
        parameterization=param,
        slot_group="fixture_nested_pure.pure",
        subjects=[sp],
        slots={"h": NestedSet("fixture_species_function.pure", {"a": 3.0}, subjects=[sp])},
        origins=[origin("a.json#/2", "fitted")],
    )
    (row,) = w.tables()["param.fixture_species_function__pure"].to_pylist()
    assert row["i"] == sp and row["a"] == 3.0


def test_a_nested_set_is_refused_when_its_form_does_not_implement_the_accepted_contract(
    extended: Declaration,
) -> None:
    from thermo_knowledge.canonical.writer import NestedSet

    w = writer(extended)
    low, high, param = nested_parent(w)

    def write(value: object, group: str = PAIR) -> uuid.UUID:
        return w.parameter_set(
            parameterization=param,
            slot_group=group,
            subjects=[low, high],
            slots={"k": 1.0, "f": value},
            origins=[origin("a.json#/2", "fitted")],
        )

    message = refused(
        w,
        lambda: write(
            NestedSet(
                "fixture_symmetric.pair",
                {"k": 1.0, "T_ref": Quantity(1.0, "K")},
                subjects=[low, high],
            )
        ),
    )
    assert (
        "accepts contract `fixture_function`" in message and "implements `fixture_pair`" in message
    )
    assert message.startswith("a.json#/2: ")
    assert "a nested-set slot takes a NestedSet" in refused(w, lambda: write(uuid.uuid4()))
    assert "is not a slot group" in refused(w, lambda: write(NestedSet("nothing.here", {})))
    assert "a required value is missing" in refused(
        w,
        lambda: w.parameter_set(
            parameterization=param,
            slot_group=PAIR,
            subjects=[low, high],
            slots={"k": 1.0},
            origins=[origin("a.json#/2", "fitted")],
        ),
    )
    assert "b: a required value is missing" in refused(
        w, lambda: write(NestedSet(LINEAR, {"a": 1.0, "T_ref": Quantity(1.0, "K")}))
    )
    assert "has 1 subject role(s), 2 given" in refused(
        w,
        lambda: w.parameter_set(
            parameterization=param,
            slot_group="fixture_nested_pure.pure",
            subjects=[low],
            slots={
                "h": NestedSet("fixture_species_function.pure", {"a": 1.0}, subjects=[low, high])
            },
            origins=[origin("a.json#/2", "fitted")],
        ),
    )


def test_nested_sets_load_with_every_constraint_satisfied(
    extended: Declaration, tmp_path: Path
) -> None:
    from build_support import fingerprint, inputs_of, write_source
    from thermo_knowledge.build import build_database
    from thermo_knowledge.canonical.writer import NestedSet
    from thermo_knowledge.testing import TestDatabase

    def fill(w: CanonicalWriter) -> None:
        low, high, param = nested_parent(w)
        w.parameter_set(
            parameterization=param,
            slot_group=PAIR,
            subjects=[high, low],
            slots={
                "k": 1.0,
                "f": NestedSet(LINEAR, {"a": 1.0, "b": 2.0, "T_ref": Quantity(300.0, "K")}),
            },
            families={
                "term": [
                    FamilyRow(
                        {"order": 0},
                        {"c": 1.0, "g": NestedSet("fixture_constant.global", {"c": 2.0})},
                    )
                ]
            },
            origins=[origin("a.json#/2", "fitted")],
        )

    write_source(tmp_path, "src", fill, decl=extended, declaration=fingerprint(extended))
    with TestDatabase() as database:
        counts = build_database(database.url, extended, inputs_of(tmp_path)).tables
    assert counts["tk.parameter_set"] == 3 and counts["param.fixture_linear__global"] == 1
