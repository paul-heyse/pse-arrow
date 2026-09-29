# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""A PI loop against IDAES's PIDController integrated by PETSc (blueprint §13.5).

A first-order plant ``tau dy/dt = u - y`` starts at ``y = 0.5`` under PI control
toward a setpoint of 1. IDAES builds its ``PIDController`` (PI, unbounded) on a dynamic
flowsheet and integrates it with ``petsc_dae_by_time_element``; pse integrates its
control library's ``PID`` with the derivative time zero and output limits far outside
the trajectory, so the smooth saturation and the anti-windup term vanish to rounding.
The gains map as ``gain_p = Kp``, ``gain_i = Kp/Ti`` and ``mv_ref = bias``, with IDAES's
integral contribution equal to ``Kp integral_state/Ti``.

The loop is linear, so the exact response is a matrix exponential. Both integrators
agree with it, and with each other, to their tolerances.
"""

import gc

import numpy as np
import pyomo.dae as dae
import pyomo.environ as pyo
import pytest
from idaes.core import FlowsheetBlock
from idaes.core.solvers import petsc
from idaes.models.control.controller import ControllerType, PIDController

import pse
from pse.contracts.enums import NativeSolveIntent

from . import support

TAU, KP, TI, BIAS, SETPOINT, START = 5.0, 0.5, 2.0, 0.5, 1.0, 0.5
END = 20.0
SAMPLES = (0.0, 1.0, 2.0, 5.0, 10.0, 20.0)
LOOP = f"""package pid_parity {{
 use control @"1.0.0";
 def Loop {{
  domain t:Time from 0{{s}} to {END}{{s}};
  discretize grid on t using integrated(elements=1,order=1);
  child regulator:control.PID=control.PID(timeline=t,first=0{{s}},Kp={KP},Ti={TI}{{s}},
   Td=0{{s}},bias={BIAS},lower=-1000,upper=1000,initial_integral=0{{s}});
  var y[i in t]:Scalar;
  eq plant[i in t]:d(y[i])/di==(regulator.output[i]-y[i])/{TAU}{{s}};
  eq plant_initial:y[0{{s}}]=={START};
  eq measured[i in t]:regulator.measurement[i]==y[i];
  eq target[i in t]:regulator.setpoint[i]=={SETPOINT};
  annotation start y({START});
 }}
 test pi_loop fixture {{dof 0; run integrated;
   integrate samples({",".join(f"{t}{{s}}" for t in SAMPLES)})
   relative(1e-10) normalized_absolute(1e-12) step(1e-4{{s}});}} {{
  child root:Loop=Loop();
 }}
}}"""


def exact() -> list[float]:
    """The closed loop's exact response, ``d/dt (y, I) = A (y, I) + c``.

    ``exp(A t)`` comes from the eigendecomposition of ``A``, whose eigenvalues are a
    distinct complex pair.
    """
    a = np.array([[-(1 + KP) / TAU, KP / TI / TAU], [-1.0, 0.0]])
    c = np.array([(BIAS + KP * SETPOINT) / TAU, SETPOINT])
    steady = np.linalg.solve(a, -c)
    start = np.array([START, 0.0])
    values, vectors = np.linalg.eig(a)
    offset = np.linalg.solve(vectors, start - steady)
    return [
        float(np.real(steady[0] + vectors[0] @ (np.exp(values * t) * offset)))
        for t in SAMPLES
    ]


def petsc_response() -> list[float]:
    """IDAES's PIDController on the plant, integrated by PETSc."""
    m = pyo.ConcreteModel()
    m.fs = FlowsheetBlock(dynamic=True, time_set=[0, END], time_units=pyo.units.s)
    m.fs.y = pyo.Var(m.fs.time, initialize=START)
    m.fs.u = pyo.Var(m.fs.time, initialize=BIAS)
    m.fs.dy = dae.DerivativeVar(m.fs.y, wrt=m.fs.time)

    @m.fs.Constraint(m.fs.time)
    def plant(b: pyo.Block, t: float) -> object:
        return TAU * b.dy[t] == b.u[t] - b.y[t]

    m.fs.ctrl = PIDController(
        process_var=m.fs.y,
        manipulated_var=m.fs.u,
        controller_type=ControllerType.PI,
        calculate_initial_integral=False,
    )
    pyo.TransformationFactory("dae.finite_difference").apply_to(
        m.fs, nfe=4, wrt=m.fs.time, scheme="BACKWARD"
    )
    m.fs.ctrl.gain_p.fix(KP)
    m.fs.ctrl.gain_i.fix(KP / TI)
    m.fs.ctrl.setpoint.fix(SETPOINT)
    m.fs.ctrl.mv_ref.fix(BIAS)
    m.fs.ctrl.mv_integral_component[0].fix(0.0)
    m.fs.y[0].fix(START)
    result = petsc.petsc_dae_by_time_element(
        m,
        time=m.fs.time,
        ts_options={
            "--ts_type": "bdf",
            "--ts_bdf_order": 3,
            "--ts_adapt_type": "basic",
            "--ts_rtol": 1e-10,
            "--ts_atol": 1e-12,
            "--ts_dt": 1e-3,
            "--ts_save_trajectory": 1,
        },
    )
    trajectory = result.trajectory.interpolate(list(SAMPLES))
    response = [float(v) for v in trajectory.get_vec(m.fs.y[END])]
    # Release the trajectory reader's open result files within this test.
    del result, trajectory
    gc.collect()
    return response


def pse_response(runtime: pse.Runtime) -> list[float]:
    """The pse control-library PID on the plant, integrated by the authored fixture."""
    package = support.reference_package(runtime, LOOP)
    fixture = next(
        d.declaration_id for d in package.declarations() if d.name == "pi_loop"
    )
    conformance = package.conform(pse.SolveSettings(intent=NativeSolveIntent.ROOT))
    # The reference libraries' definitions fail package coverage here; the fixture is
    # judged on its own status.
    (status,) = (
        row
        for row in support.rows(conformance.fixture_statuses())
        if row["fixture_id"] == fixture
    )
    assert status["status"] == "passed", status
    trajectory = conformance.trajectory(fixture)
    assert trajectory.accepted, trajectory.validation_error
    assert trajectory.termination == "completed"
    series: dict[bytes, dict[float, float]] = {}
    for row in support.rows(trajectory.table()):
        samples = series.setdefault(support.identity(row["symbol_id"]), {})
        samples[support.real(row["time"])] = support.real(row["value"])
    # The plant state and the controller's measurement are equal by the loop's
    # equation, and they are the only samples that start at y(0).
    (process, measurement) = (
        [values[t] for t in SAMPLES]
        for values in series.values()
        if values[0.0] == START
    )
    assert process == pytest.approx(measurement, abs=1e-12)
    return process


@pytest.mark.integration
@pytest.mark.parity
# IDAES's PETSc trajectory reader leaves its temporary result files open.
@pytest.mark.filterwarnings("ignore::pytest.PytestUnraisableExceptionWarning")
def test_pi_loop_agrees_with_petsc(runtime: pse.Runtime) -> None:
    """The pse and PETSc responses are within 1e-6 of the exact one and each other."""
    reference = exact()
    integrated = petsc_response()
    simulated = pse_response(runtime)
    for t, want, idaes, ours in zip(
        SAMPLES, reference, integrated, simulated, strict=True
    ):
        assert idaes == pytest.approx(want, abs=1e-6), t
        assert ours == pytest.approx(want, abs=1e-6), t
        assert ours == pytest.approx(idaes, abs=1e-6), t
