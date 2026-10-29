# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Isolated IDAES/PETSc PI reference and exact linear closed-loop response.

Native candidates arrive only as explicit case/results. This script retains the
independent PIDController/PETSc model and matrix-exponential oracle from the old
PID parity test. It never imports the candidate product or its fixtures.
"""

from __future__ import annotations

import contextlib
import importlib.metadata
import math
import shutil
import sys
from pathlib import Path
from typing import TYPE_CHECKING, Protocol, TextIO, cast
from unittest.mock import patch

import msgspec

if TYPE_CHECKING:
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


class LoopCase(msgspec.Struct, forbid_unknown_fields=True):
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


class CandidateReport(msgspec.Struct, forbid_unknown_fields=True):
    """Observed native completion and both physical Scalar trajectories."""

    complete: bool
    accepted: bool
    termination: str
    fixture_passed: bool
    samples_seconds: list[float]
    process: list[float]
    measurement: list[float]


class Candidate(msgspec.Struct, forbid_unknown_fields=True):
    """A result with explicit identity, including an explicitly absent result."""

    id: str
    report: CandidateReport | None


class Request(msgspec.Struct, forbid_unknown_fields=True):
    """Bounded scientific exchange without a Python candidate adapter."""

    version: int
    case: LoopCase
    pins: dict[str, str]
    candidates: list[Candidate]


class Reference(msgspec.Struct, forbid_unknown_fields=True):
    """Independent trajectory and equilibrium observations."""

    samples_seconds: list[float]
    exact_process: list[float]
    petsc_process: list[float]
    steady_process: float
    steady_integral_seconds: float


class Comparison(msgspec.Struct, forbid_unknown_fields=True):
    """One comparison retained across a mixed population."""

    id: str
    status: str
    reasons: list[str]


class Response(msgspec.Struct, forbid_unknown_fields=True):
    """Independent results plus all original candidate identities."""

    version: int
    case_id: str
    environment: dict[str, str]
    reference: Reference
    comparisons: list[Comparison]


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


def environment(pins: dict[str, str]) -> dict[str, str]:
    """Require the declared reference pins and executable reference environment."""
    require(
        (3, 11) <= sys.version_info[:2] < (3, 15), "reference requires Python 3.11-3.14"
    )
    require(set(pins) == {"idaes-pse", "pyomo"}, "missing or unknown reference pins")
    observed = {name: importlib.metadata.version(name) for name in pins}
    require(
        observed == pins,
        f"reference pin mismatch: expected {pins}, observed {observed}",
    )
    require(
        shutil.which("ipopt") is not None,
        "ipopt is absent; reference execution never skips",
    )
    observed["python"] = ".".join(str(part) for part in sys.version_info[:3])
    observed["numpy"] = importlib.metadata.version("numpy")
    return observed


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


def petsc_response(case: LoopCase) -> list[float]:
    """Run the original independent IDAES PIDController on the first-order plant."""
    import idaes
    import pyomo.dae as dae
    import pyomo.environ as pyo
    from idaes import config
    from idaes.core import FlowsheetBlock
    from idaes.core.solvers import petsc
    from idaes.models.control import controller
    from idaes.models.control.controller import ControllerType

    # Match the original parity environment: retain the explicitly prepared
    # solver prefix ahead of the bundled IDAES (PETSc) fallback.
    idaes.cfg.use_idaes_solvers = False
    config.reconfig(idaes.cfg)
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
        result = petsc.petsc_dae_by_time_element(
            model,
            time=fs.time,
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


def compare(candidate: Candidate, case: LoopCase, reference: Reference) -> Comparison:
    """Retain missing/failing controls beside passing native trajectories."""
    report = candidate.report
    if report is None:
        return Comparison(candidate.id, "missing", ["native result missing"])
    try:
        require(report.complete, "native trajectory incomplete")
        require(report.accepted, "native trajectory not accepted")
        require(report.termination == "completed", "native integration not completed")
        require(report.fixture_passed, "native authored fixture failed")
        require(
            report.samples_seconds == case.samples_seconds, "native sample times differ"
        )
        count = len(case.samples_seconds)
        require(
            len(report.process) == count and len(report.measurement) == count,
            "native sample coverage incomplete",
        )
        for time, ours, measured, want, idaes in zip(
            case.samples_seconds,
            report.process,
            report.measurement,
            reference.exact_process,
            reference.petsc_process,
            strict=True,
        ):
            close(
                ours,
                measured,
                case.engineering_fraction,
                f"plant versus measurement at {time}s",
            )
            close(
                ours, want, case.engineering_fraction, f"native versus exact at {time}s"
            )
            close(
                ours,
                idaes,
                case.engineering_fraction,
                f"native versus PETSc at {time}s",
            )
    except ValueError as error:
        return Comparison(candidate.id, "failed", [str(error)])
    return Comparison(candidate.id, "passed", [])


def main() -> None:
    """Execute the independent oracle before producing any comparison verdict."""
    require(len(sys.argv) == 3, "expected request and response paths")
    require(sys.flags.isolated == 1, "oracle requires the interpreter's -I option")
    request = msgspec.json.decode(Path(sys.argv[1]).read_bytes(), type=Request)
    require(request.version == 1, "unsupported exchange version")
    validate_case(request.case)
    identities = [candidate.id for candidate in request.candidates]
    require(
        bool(identities)
        and all(identities)
        and len(set(identities)) == len(identities),
        "candidate identities must be unique and nonempty",
    )
    with contextlib.redirect_stdout(sys.stderr):
        observed = environment(request.pins)
        values, steady_process, steady_integral = exact(request.case)
        integrated = petsc_response(request.case)
    for time, want, idaes in zip(
        request.case.samples_seconds, values, integrated, strict=True
    ):
        close(
            idaes,
            want,
            request.case.engineering_fraction,
            f"PETSc versus exact at {time}s",
        )
    require(
        not any(name == "pse" or name.startswith("pse.") for name in sys.modules),
        "reference imported the candidate product",
    )
    reference = Reference(
        request.case.samples_seconds,
        values,
        integrated,
        steady_process,
        steady_integral,
    )
    response = Response(
        1,
        request.case.id,
        observed,
        reference,
        [
            compare(candidate, request.case, reference)
            for candidate in request.candidates
        ],
    )
    Path(sys.argv[2]).write_bytes(msgspec.json.encode(response))


if __name__ == "__main__":
    main()
