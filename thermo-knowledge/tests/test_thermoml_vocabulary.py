# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The ThermoML observable vocabulary (`model/observables_thermoml.toml`): every `ePropName`
value of the staged ThermoML schema has exactly one alias of carrier scope `thermoml`, each alias's
unit text has the dimension of its observable's quantity type (or is a listed loss), and two names
share an observable only where their meanings are equal."""

from __future__ import annotations

import re
import uuid
from collections import defaultdict
from pathlib import Path

import pytest

from r3_reader_support import stage_real, staged_table
from thermo_knowledge import config
from thermo_knowledge.acquire import store
from thermo_knowledge.declaration import Declaration, load_declaration
from thermo_knowledge.declaration.model import Entity
from thermo_knowledge.declaration.types import registry

PIN = "5c9945ce07c2"
TREE = store.pin_dir(config.raw_dir(), "thermoml_schema", PIN) / store.TREE_DIR_NAME
SCOPE = "thermoml"

real = pytest.mark.skipif(
    not TREE.is_dir(),
    reason=f"the acquired ThermoML schema {TREE} is absent (run `tk acquire thermoml_schema`)",
)

# The property groups of the schema and the number of names each enumerates.
GROUP_SIZES = {
    "Criticals": 10,
    "VaporPBoilingTAzeotropTandP": 5,
    "PhaseTransition": 19,
    "CompositionAtPhaseEquilibrium": 31,
    "ActivityFugacityOsmoticProp": 8,
    "VolumetricProp": 24,
    "HeatCapacityAndDerivedProp": 20,
    "ExcessPartialApparentEnergyProp": 23,
    "TransportProp": 12,
    "RefractionSurfaceTensionSoundSpeed": 14,
    "BioProperties": 5,
    "ReactionStateChangeProp": 7,
    "ReactionEquilibriumProp": 15,
}

# Names whose unit has an exponent `n` that follows from the data: not a fixed dimension, so each
# is aliased with that loss stated and stored as a dimensionless number on its scale.
DATA_DEPENDENT_UNITS = {
    "Mean ionic activity, (mol/dm3)^n",
    "Equilibrium constant in terms of molality, (mol/kg)^n",
    "Equilibrium constant in terms of amount concentration (molarity), (mol/dm3)^n",
    "Equilibrium constant in terms of partial pressure, kPa^n",
    "Natural logarithm of equilibrium constant in terms of molality, (mol/kg)^n",
    "Natural logarithm of equilibrium constant in terms of amount concentration (molarity), (mol/dm3)^n",
    "Natural logarithm of equilibrium constant in terms of partial pressure, kPa^n",
    "Decadic logarithm of equilibrium constant in terms of molality, (mol/kg)^n",
    "Decadic logarithm of equilibrium constant in terms of amount concentration (molarity), (mol/dm3)^n",
    "Decadic logarithm of equilibrium constant in terms of partial pressure, kPa^n",
}

# Observables that more than one ThermoML name denotes, with the reason their meanings are equal.
# A name that differs in any facet is another observable, so a group is added here only for a real
# equality of meaning.
INTENDED_SHARED: dict[str, tuple[frozenset[str], str]] = {}


@pytest.fixture(scope="module")
def model() -> Declaration:
    result = load_declaration()
    assert result.diagnostics == (), "\n".join(map(str, result.diagnostics))
    return result.require()


@pytest.fixture(scope="module")
def schema_names(tmp_path_factory: pytest.TempPathFactory) -> dict[str, list[str]]:
    """The `ePropName` values of the real schema, staged afresh, by property group."""
    staged: Path = stage_real("thermoml_schema", tmp_path_factory.mktemp("staged"))
    table = staged_table(staged, "schema_enumerations")
    names: dict[str, list[str]] = defaultdict(list)
    for row in table.to_pylist():
        if row["element_name"] == "ePropName":
            names[row["property_group"]].append(row["value"])
    return dict(names)


def aliases_of(model: Declaration) -> dict[str, Entity]:
    found = [
        e
        for e in model.entities
        if e.kind == "observable_alias" and e.values["carrier_scope"] == SCOPE
    ]
    by_name = {str(e.values["name"]): e for e in found}
    assert len(by_name) == len(found), "an alias name is declared twice"
    return by_name


def observables_by_id(model: Declaration) -> dict[uuid.UUID, Entity]:
    return {e.id: e for e in model.entities if e.kind == "observable"}


def quantity_name_by_id(model: Declaration) -> dict[uuid.UUID, str]:
    return {identifier: name for name, identifier in model.meta_ids()["quantity_type"].items()}


def unit_text(name: str) -> str | None:
    """The unit ThermoML writes after the last comma of a property name, `None` for a name with
    none (a dimensionless quantity)."""
    head, separator, tail = name.rpartition(", ")
    return tail if separator else None


def parse_unit(text: str):  # type: ignore[no-untyped-def]
    """`pint` has no unit called `m3`: ThermoML writes an exponent as trailing digits, which this
    restates with `**`. A symbolic exponent is refused."""
    if re.search(r"\^[A-Za-z]", text):
        raise ValueError(f"{text!r} has a symbolic exponent")
    return registry().parse_units(re.sub(r"(?<=[A-Za-z])(\d+)", r"**\1", text))


@real
def test_the_schema_has_the_property_groups_the_vocabulary_was_written_for(
    schema_names: dict[str, list[str]],
) -> None:
    assert {group: len(names) for group, names in schema_names.items()} == GROUP_SIZES
    assert sum(GROUP_SIZES.values()) == 193


@real
def test_every_property_name_has_exactly_one_alias_and_no_alias_is_stale(
    model: Declaration, schema_names: dict[str, list[str]]
) -> None:
    aliases = aliases_of(model)
    every = [name for names in schema_names.values() for name in names]
    assert len(set(every)) == len(every), "a property name is in two groups"
    for group, names in schema_names.items():
        missing = [name for name in names if name not in aliases]
        assert not missing, f"{group}: names without an alias: {missing}"
        assert sum(name in aliases for name in names) == len(names) == GROUP_SIZES[group]
    assert set(aliases) == set(every), sorted(set(aliases) - set(every))
    assert len(aliases) == sum(GROUP_SIZES.values())


@real
def test_each_alias_unit_has_the_dimension_of_its_observables_quantity_type(
    model: Declaration, schema_names: dict[str, list[str]]
) -> None:
    observables = observables_by_id(model)
    quantities = quantity_name_by_id(model)
    ureg = registry()
    dependent: set[str] = set()
    for name, alias in aliases_of(model).items():
        observable = observables[alias.values["observable"]]  # type: ignore[index]
        quantity = model.quantity_types[quantities[observable.values["quantity"]]]  # type: ignore[index]
        text = unit_text(name)
        if text is not None and re.search(r"\^[A-Za-z]", text):
            dependent.add(name)
            continue
        expected = ureg.parse_units(quantity.unit).dimensionality
        stated = ureg.parse_units("dimensionless") if text is None else parse_unit(text)
        stated_dimension = getattr(stated, "dimensionality", stated)
        assert stated_dimension == expected, (
            f"{name!r}: {text!r} has dimension {stated_dimension}, but "
            f"{observable.name} is stored as {quantity.name} in {quantity.unit}"
        )
    assert dependent == DATA_DEPENDENT_UNITS


@real
def test_a_unit_that_depends_on_the_data_is_an_alias_with_its_loss_stated(
    model: Declaration,
) -> None:
    aliases = aliases_of(model)
    for name in DATA_DEPENDENT_UNITS:
        alias = aliases[name]
        text = unit_text(name)
        assert text is not None
        assert alias.values["precision"] != "exact", name
        assert text in str(alias.values.get("loss", "")), name


@real
def test_an_alias_states_a_loss_exactly_when_it_is_not_exact(model: Declaration) -> None:
    for name, alias in aliases_of(model).items():
        exact = alias.values["precision"] == "exact"
        loss = alias.values.get("loss")
        assert exact == (loss is None), name
        assert loss is None or str(loss).strip(), name


@real
def test_only_the_intended_observables_have_several_thermoml_names(model: Declaration) -> None:
    observables = observables_by_id(model)
    by_observable: dict[str, set[str]] = defaultdict(set)
    for name, alias in aliases_of(model).items():
        by_observable[observables[alias.values["observable"]].name].add(name)  # type: ignore[index]
    shared = {key: names for key, names in by_observable.items() if len(names) > 1}
    assert {key: frozenset(names) for key, names in shared.items()} == {
        key: names for key, (names, _reason) in INTENDED_SHARED.items()
    }
    for _key, (names, reason) in INTENDED_SHARED.items():
        assert len(names) > 1 and reason.strip()
