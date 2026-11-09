# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Isolated sIPOPT oracle for explicit native original-unit sensitivity results.

sIPOPT predicts the perturbed state independently. The analytic KKT branch
supplies separate nominal, prediction and derivative observations.
"""

from __future__ import annotations

import contextlib
import json
import math
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING, cast

from .reference_environment import ReferenceEnvironment
from .scalar_case import ScalarCase, require

if TYPE_CHECKING:
    from pyomo.core.base.param import ScalarParam
    from pyomo.core.base.var import ScalarVar


@dataclass(frozen=True)
class SensitivityCase:
    """One dimensionless quadratic optimization and its parameter perturbation."""

    id: str
    nominal: dict[str, float]
    perturbed: dict[str, float]
    variables: list[str]
    unit: str
    initial: float
    z_bounds: tuple[float, float]
    coupling: tuple[float, float]
    objective: tuple[float, float, float, float, float]


_VARIABLES = ("x", "y", "z")
_PARAMETERS = ("a", "b")
_STAGES = ("sipopt:nominal", "sipopt:prediction", "exact:nominal", "exact:prediction")
_INPUTS = (
    "nominal:a",
    "nominal:b",
    "perturbed:a",
    "perturbed:b",
    "initial",
    "z:lower",
    "z:upper",
    "coupling:x",
    "coupling:y",
    "objective:x*x",
    "objective:y*y",
    "objective:-b*x",
    "objective:z*z",
    "objective:a*z",
)
SIPOPT_VALUE_ABSOLUTE_ERROR = 5e-7


def validate_case(case: SensitivityCase) -> None:
    """Keep the original independent quadratic and its active-bound branch."""
    require(case.id == "active-bound-quadratic-response", "unknown sensitivity case")
    require(case.nominal == {"a": 1.0, "b": 2.0}, "unexpected nominal parameters")
    require(case.perturbed == {"a": 1.1, "b": 2.1}, "unexpected perturbed parameters")
    require(case.variables == list(_VARIABLES), "unexpected variables")
    require(case.unit == "dimensionless Scalar", "original Scalar units required")
    require(
        case.initial == 0.5 and case.z_bounds == (0.0, 10.0), "changed start/bounds"
    )
    require(case.coupling == (1.0, 1.0), "changed equality")
    require(
        case.objective == (0.5, 1.0, 1.0, 0.5, 1.0),
        "changed quadratic objective",
    )
    require(case.nominal["a"] > 0 and case.perturbed["a"] > 0, "active branch changed")


def sipopt_prediction(
    case: SensitivityCase,
    environment: ReferenceEnvironment,
) -> tuple[dict[str, float], dict[str, float]]:
    """Solve the original independent Pyomo model through its sIPOPT interface."""
    import pyomo.environ as pyo
    from pyomo.contrib.sensitivity_toolbox.sens import sensitivity_calculation

    model = pyo.ConcreteModel()
    a = cast("ScalarParam", pyo.Param(initialize=case.nominal["a"], mutable=True))
    b = cast("ScalarParam", pyo.Param(initialize=case.nominal["b"], mutable=True))
    x = cast("ScalarVar", pyo.Var(initialize=case.initial))
    y = cast("ScalarVar", pyo.Var(initialize=case.initial))
    z = cast("ScalarVar", pyo.Var(bounds=case.z_bounds, initialize=case.initial))
    model.a, model.b = a, b
    model.x, model.y, model.z = x, y, z
    model.coupling = pyo.Constraint(expr=x + y == a)
    model.cost = pyo.Objective(expr=0.5 * x**2 + y**2 - b * x + 0.5 * z**2 + a * z)
    # Pyomo's sIPOPT path constructs its own AMPL shell plugin and performs
    # nominal and perturbed calculation in that one invocation.
    with environment.registration(
        "ipopt_sens", "sensitivity.sipopt", alias="ipopt_sens"
    ):
        solved = sensitivity_calculation(
            "sipopt", model, [a, b], [case.perturbed["a"], case.perturbed["b"]]
        )
    environment.require_solves("sensitivity.sipopt", 1)
    prediction = {
        name: float(
            cast("float", pyo.value(solved.sens_sol_state_1[getattr(solved, name)]))
        )
        for name in case.variables
    }
    require(
        all(math.isfinite(value) for value in prediction.values()),
        "nonfinite sIPOPT response",
    )
    nominal = {
        name: float(cast("float", pyo.value(getattr(solved, name))))
        for name in case.variables
    }
    return nominal, prediction


def exact(parameters: dict[str, float]) -> dict[str, float]:
    """The regular active-bound branch has linear KKT conditions in a and b."""
    a, b = parameters["a"], parameters["b"]
    return {"x": (b + 2 * a) / 3, "y": (a - b) / 3, "z": 0.0}


def close(actual: float, expected: float, allowance: float, message: str) -> None:
    """Check a case-owned absolute error in the original physical coordinate."""
    require(
        math.isfinite(actual)
        and math.isfinite(expected)
        and math.isfinite(allowance)
        and allowance >= 0
        and abs(actual - expected) <= allowance,
        message,
    )


def check_sipopt(
    case: SensitivityCase, nominal: dict[str, float], prediction: dict[str, float]
) -> None:
    """Check the external solver before consulting the native candidate population."""
    require(
        set(nominal) == set(prediction) == set(case.variables),
        "sIPOPT response population differs",
    )
    for stage, observed, parameters in (
        ("nominal", nominal, case.nominal),
        ("prediction", prediction, case.perturbed),
    ):
        expected = exact(parameters)
        for name in case.variables:
            close(
                observed[name],
                expected[name],
                SIPOPT_VALUE_ABSOLUTE_ERROR,
                f"sIPOPT {stage} differs for {name}",
            )


def _case(original: ScalarCase) -> SensitivityCase:
    values = original.inputs(_INPUTS)
    case = SensitivityCase(
        id=original.contract.case_name,
        nominal={name: values[f"nominal:{name}"] for name in _PARAMETERS},
        perturbed={name: values[f"perturbed:{name}"] for name in _PARAMETERS},
        variables=list(_VARIABLES),
        unit="dimensionless Scalar",
        initial=values["initial"],
        z_bounds=(values["z:lower"], values["z:upper"]),
        coupling=(values["coupling:x"], values["coupling:y"]),
        objective=(
            values["objective:x*x"],
            values["objective:y*y"],
            values["objective:-b*x"],
            values["objective:z*z"],
            values["objective:a*z"],
        ),
    )
    validate_case(case)
    return case


def main() -> None:
    """Check original physical inputs, run sIPOPT and publish typed observations."""
    require(len(sys.argv) == 4, "expected descriptor, contract and response paths")
    require(sys.flags.isolated == 1, "oracle requires the interpreter's -I option")
    original = ScalarCase.read(Path(sys.argv[1]), Path(sys.argv[2]))
    case = _case(original)
    original.outputs(
        [f"{stage}:{name}" for stage in _STAGES for name in _VARIABLES]
        + [
            f"derivative:{name}:{parameter}"
            for name in _VARIABLES
            for parameter in _PARAMETERS
        ]
    )
    with contextlib.redirect_stdout(sys.stderr):
        environment = ReferenceEnvironment.from_process(
            ("idaes-pse", "pyomo", "protobuf", "pint"),
            role="PSE_SENSITIVITY_REFERENCE_PYTHON",
        )
        nominal, predicted = sipopt_prediction(case, environment)
        check_sipopt(case, nominal, predicted)
    require(
        not any(name == "pse" or name.startswith("pse.") for name in sys.modules),
        "reference execution imported the candidate product",
    )
    jacobian = {
        "x": {"a": 2.0 / 3.0, "b": 1.0 / 3.0},
        "y": {"a": 1.0 / 3.0, "b": -1.0 / 3.0},
        "z": {"a": 0.0, "b": 0.0},
    }
    solutions = (nominal, predicted, exact(case.nominal), exact(case.perturbed))
    original.publish(
        Path(sys.argv[3]),
        [solution[name] for solution in solutions for name in _VARIABLES]
        + [
            jacobian[name][parameter]
            for name in _VARIABLES
            for parameter in _PARAMETERS
        ],
    )
    sys.stderr.write(
        "sensitivity_reference_environment "
        + json.dumps(environment.evidence, sort_keys=True)
        + "\n"
    )
