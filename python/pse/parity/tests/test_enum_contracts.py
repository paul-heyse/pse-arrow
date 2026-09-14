# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Exercise the generated name bindings against the pinned parity runtime."""

from enum import Enum
from importlib import import_module

import pytest
from pyomo.dae import ContinuousSet, DerivativeVar
from pyomo.environ import ConcreteModel, TransformationFactory, Var

from pse.contracts import enums


@pytest.mark.unit
@pytest.mark.parity
@pytest.mark.parametrize(
    "name",
    sorted(set(enums.IDAES_NAMES) - {"ComponentType", "DiscretizationScheme"}),
)
def test_declared_enum_names_match_upstream(name: str) -> None:
    module, bindings = enums.IDAES_NAMES[name]
    upstream = getattr(import_module(module), name)
    assert isinstance(upstream, type), name
    assert issubclass(upstream, Enum), name
    # Blueprint section 6.14 selects the four physical Henry definitions.
    # IDAES also exposes Dummy solely to exercise its own error handling.
    upstream_only = {"HenryType": {"Dummy"}}.get(name, set())
    assert upstream_only <= set(upstream.__members__), name
    assert set(upstream.__members__) - upstream_only == set(bindings.values()), name
    declared = getattr(enums, name)
    assert {member.value for member in declared} == set(bindings), name
    for unsupported in upstream_only:
        with pytest.raises(ValueError):
            declared(unsupported)


@pytest.mark.unit
@pytest.mark.parity
def test_component_type_names_resolve_to_actual_upstream_classes() -> None:
    # Blueprint section 6.14 preserves these class names as a closed dictionary.
    module, bindings = enums.IDAES_NAMES["ComponentType"]
    upstream = import_module(module)
    assert {member.value for member in enums.ComponentType} == set(bindings)
    for target in bindings.values():
        component = getattr(upstream, target)
        assert isinstance(component, type), target
        assert component.__name__ == target


@pytest.mark.component
@pytest.mark.parity
@pytest.mark.parametrize("scheme", list(enums.DiscretizationScheme))
def test_discretization_name_is_accepted_by_actual_pyomo_transform(
    scheme: enums.DiscretizationScheme,
) -> None:
    _, bindings = enums.IDAES_NAMES["DiscretizationScheme"]
    spelling = bindings[scheme.value]
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
