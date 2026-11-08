# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Isolated IDAES/PETSc PI reference and exact linear closed-loop response.

Native candidates arrive only as explicit case/results. This script retains the
independent PIDController/PETSc model and matrix-exponential oracle from the old
PID parity test. It never imports the candidate product or its fixtures.
"""

from __future__ import annotations

import contextlib
import hashlib
import json
import math
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING, Protocol, TextIO, cast
from unittest.mock import patch

from google.protobuf import message_factory
from proto_exchange import encode_reference, reference_type

from .ordinary_exchange import (
    _bounded_read,
    _identity,
    _message,
    _read_artifact,
    _validate_physical,
    bits_float,
    float_bits,
)
from .reference_environment import ReferenceEnvironment

if TYPE_CHECKING:
    from google.protobuf.message import Message
    import pyomo.dae as dae
    import pyomo.environ as pyo
    from idaes.models.control.controller import ControllerType
    from pyomo.core.base.var import IndexedVar, VarData


class Derivatives(Protocol):
    """Scalar derivative entries at the declared flowsheet times."""

    def __getitem__(self, time: float, /) -> VarData:
        """Return the original derivative variable at one physical time."""
        ...


class Controller(Protocol):
    """Components supplied by IDAES's dynamically declared PID block."""

    gain_p: IndexedVar
    gain_i: IndexedVar
    setpoint: IndexedVar
    mv_ref: IndexedVar
    mv_integral_component: IndexedVar


class Flowsheet(Protocol):
    """The exact dynamic flowsheet components constructed by this oracle."""

    time: dae.ContinuousSet
    y: IndexedVar
    u: IndexedVar
    dy: Derivatives
    ctrl: Controller
    plant: pyo.Constraint


class ControllerFactory(Protocol):
    """Constructor exported dynamically by IDAES's process-block decorator."""

    def __call__(
        self,
        *,
        process_var: pyo.Var,
        manipulated_var: pyo.Var,
        controller_type: ControllerType,
        calculate_initial_integral: bool,
    ) -> Controller:
        """Construct the selected native IDAES controller component."""
        ...


@dataclass(frozen=True)
class LoopCase:
    """Time is in seconds; process, manipulated variable and Kp are Scalar."""

    id: str
    tau_seconds: float
    kp: float
    ti_seconds: float
    bias: float
    setpoint: float
    initial_process: float
    initial_integral_seconds: float
    derivative_seconds: float
    lower: float
    upper: float
    samples_seconds: list[float]
    engineering_fraction: float
    initial_step_seconds: float


def require(condition: bool, reason: str) -> None:
    """Fail even when the interpreter is invoked with optimized assertions."""
    if not condition:
        raise ValueError(reason)


def validate_case(case: LoopCase) -> None:
    """Bind the reference to the retained original scientific case."""
    require(case.id == "closed-loop-pi-first-order-v1", "unknown PI reference case")
    require(
        (
            case.tau_seconds,
            case.kp,
            case.ti_seconds,
            case.bias,
            case.setpoint,
            case.initial_process,
            case.initial_integral_seconds,
            case.derivative_seconds,
            case.lower,
            case.upper,
            case.initial_step_seconds,
        )
        == (5.0, 0.5, 2.0, 0.5, 1.0, 0.5, 0.0, 0.0, -1000.0, 1000.0, 1e-4),
        "PI case differs from the original gains, conventions or initial conditions",
    )
    require(
        case.samples_seconds == [0.0, 1.0, 2.0, 5.0, 10.0, 20.0],
        "original sample times changed",
    )
    # The value comes from the native policy owner, not a second Python default.
    require(
        math.isfinite(case.engineering_fraction)
        and 0.0 < case.engineering_fraction < 0.01,
        "invalid native engineering allowance",
    )


def exact(case: LoopCase) -> tuple[list[float], float, float]:
    """Solve d(y,I)/dt=A(y,I)+c by eigendecomposition, as in the original oracle."""
    import numpy as np

    a = np.array(
        [
            [
                -(1 + case.kp) / case.tau_seconds,
                case.kp / case.ti_seconds / case.tau_seconds,
            ],
            [-1.0, 0.0],
        ]
    )
    c = np.array(
        [(case.bias + case.kp * case.setpoint) / case.tau_seconds, case.setpoint]
    )
    steady = np.linalg.solve(a, -c)
    start = np.array([case.initial_process, case.initial_integral_seconds])
    eigenvalues, vectors = np.linalg.eig(a)
    offset = np.linalg.solve(vectors, start - steady)
    response = [
        float(np.real(steady[0] + vectors[0] @ (np.exp(eigenvalues * t) * offset)))
        for t in case.samples_seconds
    ]
    # Steady process is the setpoint, with u=y and error=0. Integral is in seconds,
    # whereas IDAES's mv_integral_component is Kp*I/Ti in manipulated units.
    require(
        abs(float(steady[0]) - case.setpoint) <= case.engineering_fraction,
        "exact equilibrium process differs",
    )
    expected_integral = case.ti_seconds * (case.setpoint - case.bias) / case.kp
    require(
        abs(float(steady[1]) - expected_integral) <= case.engineering_fraction,
        "exact equilibrium integral differs",
    )
    require(all(math.isfinite(value) for value in response), "nonfinite exact response")
    return response, float(steady[0]), float(steady[1])


def petsc_response(case: LoopCase, environment: ReferenceEnvironment) -> list[float]:
    """Run the original independent IDAES PIDController on the first-order plant."""
    import pyomo.dae as dae
    import pyomo.environ as pyo
    from idaes.core import FlowsheetBlock
    from idaes.core.solvers import petsc
    from idaes.models.control import controller
    from idaes.models.control.controller import ControllerType

    model = pyo.ConcreteModel()
    end = case.samples_seconds[-1]
    fs = cast(
        "Flowsheet",
        FlowsheetBlock(dynamic=True, time_set=[0, end], time_units=pyo.units.s),
    )
    model.fs = fs
    fs.y = cast("IndexedVar", pyo.Var(fs.time, initialize=case.initial_process))
    fs.u = cast("IndexedVar", pyo.Var(fs.time, initialize=case.bias))
    fs.dy = cast("Derivatives", dae.DerivativeVar(fs.y, wrt=fs.time))

    def plant(block: Flowsheet, time: float) -> object:
        """Retain the original first-order plant equation at every time."""
        return case.tau_seconds * block.dy[time] == block.u[time] - block.y[time]

    fs.plant = pyo.Constraint(fs.time, rule=plant)

    # IDAES exports this class through declare_process_block_class at import time.
    pid_controller = cast("ControllerFactory", vars(controller)["PIDController"])
    fs.ctrl = pid_controller(
        process_var=fs.y,
        manipulated_var=fs.u,
        controller_type=ControllerType.PI,
        calculate_initial_integral=False,
    )
    transformation = pyo.TransformationFactory("dae.finite_difference")
    if transformation is None:
        message = "Pyomo finite-difference transformation is unavailable"
        raise ValueError(message)
    transformation.apply_to(fs, nfe=4, wrt=fs.time, scheme="BACKWARD")
    fs.ctrl.gain_p.fix(case.kp)
    fs.ctrl.gain_i.fix(case.kp / case.ti_seconds)
    fs.ctrl.setpoint.fix(case.setpoint)
    fs.ctrl.mv_ref.fix(case.bias)
    fs.ctrl.mv_integral_component[0].fix(
        case.kp * case.initial_integral_seconds / case.ti_seconds
    )
    fs.y[0].fix(case.initial_process)
    reader = petsc.petsc_binary_io()
    require(reader is not None, "PETSc trajectory reader is unavailable")
    handles: list[TextIO] = []

    def reader_open(name: str) -> TextIO:
        if handles:
            handles.pop().close()
        handle = Path(name).open()  # noqa: SIM115 -- explicit ExitStack owns the upstream reader handles
        handles.append(handle)
        return handle

    with contextlib.ExitStack() as ownership:
        ownership.callback(lambda: [handle.close() for handle in handles])
        ownership.enter_context(patch.object(reader, "open", reader_open, create=True))
        initializer = "pse_reference_petsc_snes"
        ownership.enter_context(
            environment.registration(
                "petsc_snes",
                "pid.initial_conditions",
                alias=initializer,
                executable="petsc",
            )
        )
        ownership.enter_context(
            environment.registration(
                "petsc_ts", "pid.integration", alias="petsc_ts", executable="petsc"
            )
        )
        result = petsc.petsc_dae_by_time_element(
            model,
            time=fs.time,
            initial_solver=initializer,
            ts_options={
                "--ts_type": "bdf",
                "--ts_bdf_order": 3,
                "--ts_adapt_type": "basic",
                # Retain the independent reference's finer integration, including
                # its saved-trajectory interpolation accuracy, from the original.
                "--ts_rtol": 1e-10,
                "--ts_atol": 1e-12,
                "--ts_dt": 1e-3,
                "--ts_save_trajectory": 1,
            },
        )
        saved = result.trajectory
        if saved is None:
            message = "PETSc did not retain the requested trajectory"
            raise ValueError(message)
        trajectory = saved.interpolate(case.samples_seconds)
        values = [float(value) for value in trajectory.get_vec(fs.y[end])]
    environment.require_solves("pid.initial_conditions", 1)
    environment.require_solves("pid.integration", 4)
    require(
        len(values) == len(case.samples_seconds), "PETSc sample coverage incomplete"
    )
    require(all(math.isfinite(value) for value in values), "nonfinite PETSc response")
    return values


def close(left: float, right: float, allowance: float, reason: str) -> None:
    """Scalar canonical floor is one; comparisons retain zero relative tolerance."""
    require(
        math.isfinite(left) and math.isfinite(right) and abs(left - right) <= allowance,
        reason,
    )


_ARTIFACTS = (
    "packages/reference/methods/package.toml",
    "packages/reference/physical/package.toml",
    "packages/reference/process/models/control.pse",
    "packages/reference/process/package.toml",
)
_FIXED = {
    "tau_seconds": "Time",
    "kp": "Scalar",
    "ti_seconds": "Time",
    "bias": "Scalar",
    "setpoint": "Scalar",
    "initial_process": "Scalar",
    "initial_integral_seconds": "Time",
    "derivative_seconds": "Time",
    "lower": "Scalar",
    "upper": "Scalar",
    "engineering_fraction": "Scalar",
    "initial_step_seconds": "Time",
}


def _sample_key(role: str, time: float) -> str:
    return f"{role}:{float_bits(time):016x}"


def _artifacts(contract: Message) -> None:
    require(
        tuple(item.path for item in contract.artifacts) == _ARTIFACTS,
        "PID source artifact population differs",
    )
    for item in contract.artifacts:
        content = _read_artifact(item.path, 1 << 20)
        require(
            bytes(item.sha256) == hashlib.sha256(content).digest(),
            "PID source artifact differs from the selected original",
        )


def _case(contract: Message) -> LoopCase:
    require(_message(contract, "version").value == 1, "unsupported PID case version")
    fixed = list(contract.specification)
    grid = [0.0, 1.0, 2.0, 5.0, 10.0, 20.0]
    roles = _FIXED | {_sample_key("time", time): "Time" for time in grid}
    require(
        tuple(item.key for item in fixed) == tuple(roles),
        "PID input population differs",
    )
    values = {}
    for item in fixed:
        _validate_physical(_message(item, "physical"), roles[item.key])
        # The enclosing fixed input is explicit; fixed64 zero uses proto3's default.
        values[item.key] = bits_float(int(item.value))
    for time in grid:
        require(
            float_bits(values[_sample_key("time", time)]) == float_bits(time),
            "original sample times changed",
        )
    case = LoopCase(
        id=contract.case_name,
        samples_seconds=grid,
        tau_seconds=values["tau_seconds"],
        kp=values["kp"],
        ti_seconds=values["ti_seconds"],
        bias=values["bias"],
        setpoint=values["setpoint"],
        initial_process=values["initial_process"],
        initial_integral_seconds=values["initial_integral_seconds"],
        derivative_seconds=values["derivative_seconds"],
        lower=values["lower"],
        upper=values["upper"],
        engineering_fraction=values["engineering_fraction"],
        initial_step_seconds=values["initial_step_seconds"],
    )
    validate_case(case)
    return case


def main() -> None:
    """Receive the exact case, perform independent solvers, and publish original bytes."""
    require(len(sys.argv) == 4, "expected descriptor, contract and response paths")
    require(sys.flags.isolated == 1, "oracle requires the interpreter's -I option")
    envelope_type = reference_type(_bounded_read(Path(sys.argv[1])))
    pool = envelope_type.DESCRIPTOR.file.pool
    contract_type = message_factory.GetMessageClass(
        pool.FindMessageTypeByName("pse.reference.v1.ReferenceCaseContract")
    )
    contract = contract_type.FromString(_bounded_read(Path(sys.argv[2])))
    case = _case(contract)
    _artifacts(contract)
    meaning = _message(contract, "meaning")
    for field, size in (
        ("case", 16),
        ("source", 32),
        ("formulation", 32),
        ("physical", 32),
        ("configuration", 32),
    ):
        original = bytes(getattr(meaning, field))
        require(
            len(original) == size and original != bytes(size),
            "PID case meaning is incomplete",
        )
    roles = {
        _sample_key(role, time): "Scalar"
        for role in ("exact_process", "petsc_process")
        for time in case.samples_seconds
    } | {"steady_process": "Scalar", "steady_integral_seconds": "Time"}
    observables = list(contract.observables)
    require(
        tuple(item.key for item in observables) == tuple(roles),
        "PID output population differs",
    )
    output_ids = [bytes(item.output) for item in observables]
    require(
        len(set(output_ids)) == len(output_ids)
        and all(
            len(identity) == 16 and identity != bytes(16) for identity in output_ids
        ),
        "PID output identities are invalid or repeated",
    )
    for item in observables:
        _validate_physical(_message(item, "physical"), roles[item.key])
    with contextlib.redirect_stdout(sys.stderr):
        environment = ReferenceEnvironment.from_process(
            ("idaes-pse", "pyomo", "protobuf", "numpy", "pint", "scipy"),
            role="PSE_PID_REFERENCE_PYTHON",
        )
        values, steady_process, steady_integral = exact(case)
        integrated = petsc_response(case, environment)
    for time, want, idaes in zip(case.samples_seconds, values, integrated, strict=True):
        close(
            idaes,
            want,
            case.engineering_fraction,
            f"PETSc versus exact at {time}s",
        )
    require(
        not any(name == "pse" or name.startswith("pse.") for name in sys.modules),
        "reference imported the candidate product",
    )
    _artifacts(contract)
    envelope = envelope_type(version=1)
    envelope.meaning.CopyFrom(meaning)
    envelope.required.extend(output_ids)
    for coordinate, value in zip(
        observables,
        (*values, *integrated, steady_process, steady_integral),
        strict=True,
    ):
        require(math.isfinite(value), "PID reference returned nonfinite evidence")
        physical = coordinate.physical
        envelope.outcome.completed.outputs.add(
            id=coordinate.output,
            quantity=_identity(physical.quantity),
            unit=_identity(physical.representation_unit),
            value=float_bits(value),
        )
    with Path(sys.argv[3]).open("xb") as stream:
        stream.write(encode_reference(envelope))
    sys.stderr.write(
        "pid_reference_environment "
        + json.dumps(environment.evidence, sort_keys=True)
        + "\n"
    )
