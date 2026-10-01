# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The loader: a valid declaration loads clean and resolves as section 3 and 4 specify."""

from __future__ import annotations

import uuid
from pathlib import Path

import pytest

from declaration_support import FULL, empty_declaration, full_declaration
from thermo_knowledge import config, identity
from thermo_knowledge.declaration import (
    DeclarationError,
    default_forms_dir,
    default_model_dir,
    load_declaration,
)


def test_fixture_loads_without_diagnostics() -> None:
    result = load_declaration(FULL / "model", FULL / "forms", contract=None)
    assert result.diagnostics == ()
    assert result.declaration is not None


def test_defaults_are_the_trees_directories() -> None:
    assert default_model_dir() == config.TREE_DIR / "model"
    assert default_forms_dir() == config.TREE_DIR / "forms"


def test_the_real_declaration_loads() -> None:
    result = load_declaration()
    assert result.diagnostics == ()
    decl = result.require()
    assert decl.kinds and decl.entities
    assert decl.framework["observable"] == "observable"
    assert set(decl.framework) >= {"parameter_set", "parameterization", "tabulated_function"}


def test_an_empty_declaration_loads(tmp_path: Path) -> None:
    decl = empty_declaration(tmp_path)
    assert decl.kinds == {} and decl.forms == {} and decl.entities == ()


def test_require_raises_with_every_diagnostic(tmp_path: Path) -> None:
    (tmp_path / "model").mkdir()
    result = load_declaration(tmp_path / "model", tmp_path / "forms", contract=None)
    assert [d.code for d in result.diagnostics] == ["missing-manifest"]
    with pytest.raises(DeclarationError) as raised:
        result.require()
    assert raised.value.diagnostics == result.diagnostics


def test_every_construct_of_the_specification_is_present() -> None:
    decl = full_declaration()
    assert set(decl.enums) == {"value_state", "phase_class", "calibration"}
    assert [facet.name for facet in decl.enums["phase_class"].facets] == ["compressible"]
    assert decl.enum_members_with("phase_class", "compressible") == ("gas",)
    assert {r.name for r in decl.relations["reading"].requires} == {
        "value_only_when_compressible",
        "value_in_range",
        "value_not_below",
        "readings_distinct",
    }
    assert set(decl.schemes) == {"inchikey"}
    assert {"Temperature", "MolarCp"} <= set(decl.quantity_types)
    assert decl.kinds["material_entity"].abstract
    assert decl.kinds["species"].extends == "material_entity"
    assert decl.kinds["ion"].root == "material_entity"
    assert decl.kinds["material_entity"].identity == ("canonical_key",)
    assert decl.kinds["species"].identity == ()
    assert decl.kinds["species"].provenance.mode == "own"
    assert decl.kinds["slot_uncertainty"].provenance.attribute == "parameter_set"
    assert {"formula", "interaction", "class_weight", "rank_weight", "reading"} <= set(
        decl.relations
    )
    assert {r.absence for r in decl.relations.values()} == {"required", "optional", "default"}
    assert set(decl.contracts) == {
        "pure_vapor_pressure",
        "alpha_function",
        "heat_capacity",
        "binary_parameter",
        "mixture_heat_capacity",
        "site_balance",
        "saturation_pressure_of_set",
    }
    assert {
        "antoine",
        "critical",
        "nasa7",
        "kij",
        "ratio",
        "cubic",
        "soave",
        "mixture_linear",
        "site_balance_form",
        "margules",
        "saturation_curve",
    } == set(decl.forms)
    assert [e.name for e in decl.entities if e.kind == "element"] == ["H", "He"]


def test_types_resolve_to_their_containers() -> None:
    attributes = {a.name: a.type for a in decl_kind("species").attributes}
    assert attributes["elements"].container == "set"
    assert attributes["reference_temperatures"].container == "array"
    assert attributes["validity"].container == "range"
    assert attributes["source"].element_kind == "record"
    assert attributes["unit_of"].element_kind == "meta"
    assert attributes["unit_of"].element == "quantity_type"
    assert attributes["uncertainty"].element_kind == "real"
    assert attributes["inchikey"].element_kind == "identifier"
    assert attributes["phase"].element_kind == "enum"
    assert attributes["charge"].element_kind == "quantity"
    assert not attributes["tag"].is_floating and attributes["uncertainty"].is_floating


def decl_kind(name: str):  # noqa: ANN201
    return full_declaration().kinds[name]


def test_quantity_expression_resolves_by_unit_algebra() -> None:
    decl = full_declaration()
    cp_curvature = next(a for a in decl_kind("species").attributes if a.name == "cp_curvature")
    assert cp_curvature.type.element_kind == "expression"
    assert cp_curvature.type.element == "MolarCp / Temperature^2"
    unit = decl.expressions["MolarCp / Temperature^2"]
    assert unit == "J / K ** 3 / mol"
    assert dict(decl.units[unit].dimensions) == {
        "length": 2,
        "mass": 1,
        "substance": -1,
        "temperature": -3,
        "time": -2,
    }
    assert decl.quantity_types["Temperature"].unit == "K"
    assert decl.quantity_types["MolarMass"].unit == "kg / mol"
    assert decl.units["dimensionless"].dimensions == ()


def test_slot_groups_expand_to_kinds_refining_parameter_set() -> None:
    decl = full_declaration()
    antoine = decl.kinds["antoine__pure"]
    assert antoine.extends == "parameter_set"
    assert antoine.root == "parameter_set"
    assert antoine.origin == "slot_group"
    assert antoine.schema == "param"
    assert antoine.identity == ()
    assert [a.name for a in antoine.attributes] == ["i", "A", "B", "C"]
    assert antoine.attributes[0].type.element == "species_form"
    assert antoine.provenance == decl.kinds["parameter_set"].provenance
    assert decl.kinds["critical__global"].attributes[0].name == "T_c"
    assert decl.is_a("antoine__pure", "parameter_set")
    assert [k.name for k in decl.chain("antoine__pure")] == ["parameter_set", "antoine__pure"]


def test_slot_shapes_and_presence() -> None:
    decl = full_declaration()
    slots = {s.name: s for s in decl.forms["critical"].slot_groups[0].slots}
    assert slots["T_c"].presence == "stateful"
    assert slots["T_c"].observable == "critical_temperature"
    assert slots["kind"].shape == "enum"
    assert slots["gas_table"].shape == "tabulated_function"
    assert slots["aggregation"].shape == "reference"
    cubic = {s.name: s for s in decl.forms["cubic"].slot_groups[0].slots}
    assert cubic["alpha_coefficients"].shape == "nested_set"
    assert cubic["alpha_coefficients"].accepts == "alpha_function"
    assert cubic["a_c"].shape == "quantity"
    assert decl.forms["critical"].slot_groups[0].subjects == ()


def test_families_subforms_and_transposition() -> None:
    decl = full_declaration()
    (group,) = decl.forms["nasa7"].slot_groups
    (family,) = group.families
    assert family.id == "nasa7__pure__piece"
    assert family.interval == ("T_low", "T_high")
    assert family.indices[0].minimum == 1
    (subform,) = decl.forms["cubic"].subforms
    assert (subform.accepts, subform.multiplicity, subform.per) == (
        "alpha_function",
        "one",
        "subject",
    )
    assert decl.forms["kij"].slot_groups[0].transposition.rule == "symmetric"  # type: ignore[union-attr]
    assert decl.forms["ratio"].slot_groups[0].transposition.slots == ("r",)  # type: ignore[union-attr]
    triple = decl.relations["triple"].transposition
    assert triple is not None and triple.rule == "permutation_group"
    assert decl.relations["signed_interaction"].transposition.by == "order"  # type: ignore[union-attr]


def test_relation_key_types_and_value_columns() -> None:
    decl = full_declaration()
    assert decl.relations["class_weight"].keys[0].type.element_kind == "enum"
    assert decl.relations["rank_weight"].keys[0].type.text == "Integer"
    assert decl.relations["alias"].keys[0].type.element_kind == "identifier"
    assert decl.relations["alias"].values == ()
    measurement = decl.relations["measurement"]
    assert [k.type.element_kind for k in measurement.keys] == ["record", "meta", "primitive"]
    assert [v.name for v in measurement.values] == ["value", "flag", "note"]
    assert measurement.values[0].type.element_kind == "real"
    assert measurement.values[1].default is False
    assert measurement.values[2].optional
    assert decl.relations["formula"].absence_default == 0.0
    assert decl.relations["formula"].provenance.attribute == "j"


def test_declared_entities_resolve_references_and_marks() -> None:
    decl = full_declaration()
    by_name = {(e.kind, e.name): e for e in decl.entities}
    gas = by_name[("aggregation", "gas")]
    vapor = by_name[("phase", "vapor")]
    assert vapor.values["aggregation"] == gas.id
    assert gas.id == identity.identifier("aggregation", ["gas"])
    # the key is the value of an attribute called `name`
    assert gas.values["name"] == "gas"
    observable = by_name[("observable", "vapor_pressure")]
    assert observable.values["quantity"] == identity.meta_identifier("quantity_type", "Pressure")
    assert observable.traces == ("IC-20",)
    group = by_name[("element_group", "light")]
    assert group.values["temperature_range"] == (200.0, 400.5)
    assert group.values["levels"] == (1, 2)
    members = group.values["members"]
    assert set(members) == {by_name[("element", "H")].id, by_name[("element", "He")].id}
    assert all(isinstance(m, uuid.UUID) for m in members)  # type: ignore[attr-defined]
    assert by_name[("element", "H")].values["molar_mass"] == 0.001008


def test_defaults_marks_and_uniqueness_are_kept() -> None:
    species = decl_kind("species")
    charge = next(a for a in species.attributes if a.name == "charge")
    assert charge.default == 0.0
    inchikey = next(a for a in species.attributes if a.name == "inchikey")
    assert inchikey.unique and inchikey.optional and inchikey.traces == ("IC-13",)
    assert species.pse == "gap:species_marks"
    assert [u.attributes for u in species.uniques] == [("tag", "charge")]
    assert {r.enforced for r in species.requires} == {"ddl", "load", "verify"}
    assert {r.rule for r in species.requires if r.enforced == "ddl"} == {
        "nonempty",
        "ordered",
        "nonnegative",
        "one_of_present",
    }


def test_collections_are_ordered_by_name() -> None:
    decl = full_declaration()
    for mapping in (decl.kinds, decl.relations, decl.forms, decl.quantity_types, decl.enums):
        assert list(mapping) == sorted(mapping)


def test_meta_ids_cover_every_reified_construct() -> None:
    ids = full_declaration().meta_ids()
    assert set(ids) == {
        "quantity_type",
        "kind",
        "contract",
        "form",
        "slot_group",
        "slot",
        "family",
        "subform_slot",
    }
    assert "antoine.pure" in ids["slot_group"]
    assert "antoine.pure.A" in ids["slot"]
    assert "nasa7.pure.piece" in ids["family"]
    assert "nasa7.pure.piece.T_low" in ids["slot"]
    assert "cubic.alpha" in ids["subform_slot"]
    assert ids["kind"]["species"] == identity.meta_identifier("kind", "species")


def test_scalar_and_meta_types_resolve() -> None:
    species = {a.name: a.type for a in decl_kind("species").attributes}
    assert (species["curated"].element_kind, species["curated"].element) == ("primitive", "Boolean")
    assert species["curated_on"].element == "Date"
    assert species["curated_at"].element == "Timestamp"
    assert species["checksum"].element == "Hash"
    assert species["note"].element_kind == "source_text"
    probe = {a.name: a.type for a in decl_kind("meta_probe").attributes}
    assert {t.element for name, t in probe.items() if name.startswith("m_")} == {
        "quantity_type",
        "kind",
        "contract",
        "form",
        "slot_group",
        "slot",
        "family",
        "subform_slot",
    }
    assert all(t.element_kind == "meta" for name, t in probe.items() if name.startswith("m_"))
    # Real is floating point and Meta is identity-eligible
    assert species["uncertainty"].is_floating and not species["uncertainty"].identity_eligible
    assert probe["m_slot"].identity_eligible
    parameter_set = {a.name: a.type for a in decl_kind("parameter_set").attributes}
    assert parameter_set["slot_group"].text == "Meta<slot_group>"


def test_release_entity_values_are_normalised() -> None:
    (release,) = [e for e in full_declaration().entities if e.kind == "release"]
    values = release.values
    assert str(values["released_on"]) == "2026-09-30"
    assert values["checksum"] == bytes(range(32))
    assert values["scale"] == 2.0 and values["scale_units"] == (1.0, 2.5)
    assert values["stable"] is True  # the default
    assert release.id == identity.identifier("release", ["r1"])


def test_array_of_real_resolves() -> None:
    samples = next(a for a in decl_kind("species").attributes if a.name == "samples")
    assert (samples.type.container, samples.type.element_kind) == ("array", "real")
    assert samples.type.is_floating
    (release,) = [e for e in full_declaration().entities if e.kind == "release"]
    assert release.values["readings"] == (0.5, -1.0)


def test_declared_entities_of_an_own_kind_are_allowed() -> None:
    decl = full_declaration()
    by_name = {(e.kind, e.name): e for e in decl.entities}
    water = by_name[("species", "water")]  # a refinement of a root with own provenance
    assert decl.kinds["species"].provenance.mode == "own"
    assert water.id == identity.identifier("material_entity", ["O"])
    assert by_name[("parameterization", "default_fit")].id == identity.identifier(
        "parameterization", ["default"]
    )


def test_observable_role_binds_a_kind_whose_entities_are_the_observables() -> None:
    decl = full_declaration()
    assert decl.framework["observable"] == "observable"
    entity = decl.observable_entity("vapor_pressure")
    assert entity is not None and entity.kind == "observable"
    assert decl.observable_entity("gas") is None  # an entity of another kind
    assert decl.observable_entity("nothing") is None
    output = decl.contracts["pure_vapor_pressure"].outputs[0]
    assert output.observable == "vapor_pressure"
