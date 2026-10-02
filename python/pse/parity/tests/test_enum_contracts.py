# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Exercise admitted package vocabularies against the pinned parity runtime."""

from enum import Enum
from importlib import import_module
from pathlib import Path

import msgspec
import pytest
from pyomo.dae import ContinuousSet, DerivativeVar
from pyomo.environ import ConcreteModel, TransformationFactory, Var

import pse
from pse.conformance import _documents
from pse.contracts import enums


class UpstreamBinding(msgspec.Struct, frozen=True, forbid_unknown_fields=True):
    module: str
    aliases: dict[str, str] = msgspec.field(default_factory=dict)


class Bindings(msgspec.Struct, frozen=True, forbid_unknown_fields=True):
    enums: dict[str, UpstreamBinding]


BINDINGS = msgspec.toml.decode(
    (Path(__file__).parents[1] / "enum-bindings.toml").read_bytes(), type=Bindings
).enums

# These scientific declarations have local meaning and no IDAES parity target.
LOCAL_ENUMS = {
    "CompositionKnowledge": ("Unknown", "Complete"),
    "ExtentNormalization": ("AsAuthored",),
}


@pytest.fixture(scope="module")
def admitted_names(
    runtime: pse.Runtime,
) -> dict[str, tuple[str, dict[str, str]]]:
    root = Path(__file__).resolve().parents[4]
    documents = _documents(root / "packages/reference/physical")
    physical = runtime.physical_from_documents(documents)
    package = runtime.modeling_from_documents([documents], physical)
    declared = {
        row.name: tuple(member.name for member in row.value.enumeration.members)
        for row in package.declarations()
        if row.value.enumeration is not None
    }
    assert not (set(BINDINGS) & set(LOCAL_ENUMS))
    assert set(declared) == set(BINDINGS) | set(LOCAL_ENUMS)
    assert {name: declared[name] for name in LOCAL_ENUMS} == LOCAL_ENUMS
    names = dict(enums.IDAES_NAMES)
    assert set(names) == {"ConstraintScalingScheme"}
    for name, binding in BINDINGS.items():
        members = declared[name]
        assert set(binding.aliases) <= set(members)
        names[name] = (
            binding.module,
            {member: binding.aliases.get(member, member) for member in members},
        )
    return names


@pytest.mark.unit
@pytest.mark.parity
@pytest.mark.parametrize(
    "name",
    sorted(
        (set(BINDINGS) | {"ConstraintScalingScheme"})
        - {"ComponentType", "DiscretizationScheme"}
    ),
)
def test_declared_enum_names_match_upstream(
    name: str,
    admitted_names: dict[str, tuple[str, dict[str, str]]],
) -> None:
    module, bindings = admitted_names[name]
    upstream = getattr(import_module(module), name)
    assert isinstance(upstream, type), name
    assert issubclass(upstream, Enum), name
    # Blueprint section 6.14 selects the four physical Henry definitions.
    # IDAES also exposes Dummy solely to exercise its own error handling.
    upstream_only = {"HenryType": {"Dummy"}}.get(name, set())
    assert upstream_only <= set(upstream.__members__), name
    assert set(upstream.__members__) - upstream_only == set(bindings.values()), name
    assert not (upstream_only & set(bindings)), name


@pytest.mark.unit
@pytest.mark.parity
def test_component_type_names_resolve_to_actual_upstream_classes(
    admitted_names: dict[str, tuple[str, dict[str, str]]],
) -> None:
    module, bindings = admitted_names["ComponentType"]
    upstream = import_module(module)
    for target in bindings.values():
        component = getattr(upstream, target)
        assert isinstance(component, type), target
        assert component.__name__ == target


@pytest.mark.component
@pytest.mark.parity
def test_discretization_names_are_accepted_by_actual_pyomo_transforms(
    admitted_names: dict[str, tuple[str, dict[str, str]]],
) -> None:
    _, bindings = admitted_names["DiscretizationScheme"]
    for spelling in bindings.values():
        method = (
            "dae.collocation"
            if spelling.startswith("LAGRANGE-")
            else "dae.finite_difference"
        )
        model = ConcreteModel()
        model.time = ContinuousSet(bounds=(0, 1))
        model.state = Var(model.time)
        model.rate = DerivativeVar(model.state, wrt=model.time)
        transform = TransformationFactory(method)
        transform.apply_to(model, wrt=model.time, nfe=2, scheme=spelling)
        assert model.rate.is_fully_discretized()
        assert len(model.rate_disc_eq) > 0
        assert model.time.get_discretization_info()["scheme"].startswith(spelling)
