# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Receiving controls for the independent reference environment."""

from __future__ import annotations

import io
import os
import sys
import tempfile
import unittest
from contextlib import redirect_stderr
from pathlib import Path
from unittest.mock import patch

from oracles.reference_environment import (
    SOLVER_LOG_BYTES,
    Executable,
    ReferenceAdmissionError,
    ReferenceEnvironment,
    locked_pins,
)


class ReferenceEnvironmentTests(unittest.TestCase):
    """Environment admission controls without a scientific solver."""

    def test_interpreter_and_package_mismatch_refuse(self) -> None:
        role = "PSE_ORDINARY_REFERENCE_PYTHON"
        with patch.dict(os.environ, {role: ""}):
            with self.assertRaisesRegex(ReferenceAdmissionError, "interpreter.*absent"):
                ReferenceEnvironment.from_process()
        with patch.dict(os.environ, {role: "/different/venv/bin/python"}):
            with self.assertRaisesRegex(
                ReferenceAdmissionError, "interpreter.*observed"
            ):
                ReferenceEnvironment.from_process()
        with self.assertRaisesRegex(
            ReferenceAdmissionError, "package.*idaes-pse.*observed"
        ):
            ReferenceEnvironment(sys.executable, {"idaes-pse": "0.0.0"})
        self.assertEqual(
            ReferenceEnvironment(sys.executable, locked_pins(("idaes-pse",))).evidence[
                "python.executable"
            ],
            sys.executable,
        )

    def test_missing_prepared_solver_does_not_search_path(self) -> None:
        environment = ReferenceEnvironment(sys.executable, {})
        with (
            patch.dict(os.environ, {"IPOPT_DIR": ""}),
            self.assertRaisesRegex(ReferenceAdmissionError, "case.stage.*IPOPT_DIR"),
        ):
            environment.solver("ipopt", "case.stage")


class ReferenceSolverEnvironmentTests(unittest.TestCase):
    """Check the original shell boundary after instance construction."""

    def test_executable_changed_after_binding_refuses_the_solve(self) -> None:
        from pyomo.environ import ConcreteModel, Objective, Var

        environment = ReferenceEnvironment(sys.executable, {})
        solver = environment.solver("ipopt", "test.direct")
        model = ConcreteModel()
        model.x = Var(initialize=1.0)
        model.cost = Objective(expr=model.x**2)
        with tempfile.TemporaryDirectory() as temporary:
            substitute = Path(temporary) / "ipopt"
            substitute.write_text("#!/bin/sh\nexit 99\n")
            substitute.chmod(0o700)
            solver.set_executable(str(substitute))
            with self.assertRaisesRegex(
                ReferenceAdmissionError, "test.direct.*substituted executable"
            ):
                solver.solve(model)
        self.assertNotIn("test.direct.solves", environment.evidence)

    def test_performing_solver_refuses_excess_output_before_pyomo_writes_a_log(
        self,
    ) -> None:
        from pyomo.common.collections import Bunch
        from pyomo.environ import SolverFactory

        environment = ReferenceEnvironment(sys.executable, {})
        with tempfile.TemporaryDirectory() as temporary:
            substitute = Path(temporary) / "ipopt"
            substitute.write_text(
                f"#!/bin/sh\nprintf 'actual solver output\\n' >&2\n"
                f"head -c {SOLVER_LOG_BYTES + 1} /dev/zero\n"
            )
            substitute.chmod(0o700)
            solver = environment._bind(
                SolverFactory("ipopt"),
                Executable.select(substitute, "test.overflow"),
                "test.overflow",
            )
            solver._timelimit = None
            diagnostics = io.StringIO()
            with (
                redirect_stderr(diagnostics),
                self.assertRaisesRegex(
                    ReferenceAdmissionError, "test.overflow.*diagnostics exceeded"
                ),
            ):
                solver._execute_command(Bunch(cmd=[str(substitute)], env=None))
            self.assertTrue(diagnostics.getvalue().startswith("actual solver output"))
            self.assertLessEqual(len(diagnostics.getvalue()), SOLVER_LOG_BYTES + 1)
            self.assertNotIn("test.overflow.solves", environment.evidence)
