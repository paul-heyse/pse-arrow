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
from typing import TYPE_CHECKING, cast
from unittest.mock import patch

from pyomo.common.collections import Bunch
from pyomo.environ import ConcreteModel, Objective, SolverFactory, Var

if TYPE_CHECKING:
    from pyomo.core.base.var import ScalarVar
    from pyomo.opt.solver.shellcmd import SystemCallSolver

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
        """Refuse missing or stale interpreter selection and a wrong release."""
        role = "PSE_ORDINARY_REFERENCE_PYTHON"
        with (
            patch.dict(os.environ, {role: ""}),
            self.assertRaisesRegex(ReferenceAdmissionError, "interpreter.*absent"),
        ):
            ReferenceEnvironment.from_process()
        with (
            patch.dict(os.environ, {role: "/different/venv/bin/python"}),
            self.assertRaisesRegex(ReferenceAdmissionError, "interpreter.*observed"),
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
        """A missing prepared capability refuses before solver construction."""
        environment = ReferenceEnvironment(sys.executable, {})
        with (
            patch.dict(os.environ, {"IPOPT_DIR": ""}),
            self.assertRaisesRegex(ReferenceAdmissionError, "case.stage.*IPOPT_DIR"),
        ):
            environment.solver("ipopt", "case.stage")


class ReferenceSolverEnvironmentTests(unittest.TestCase):
    """Check the original shell boundary after instance construction."""

    def test_executable_changed_after_binding_refuses_the_solve(self) -> None:
        """Refuse a substituted executable without recording a performed solve."""
        environment = ReferenceEnvironment(sys.executable, {})
        solver = environment.solver("ipopt", "test.direct")
        model = ConcreteModel()
        x = cast("ScalarVar", Var(initialize=1.0))
        model.x = x
        model.cost = Objective(expr=x**2)
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
        """Keep the actual bounded solver output when the shell boundary refuses."""
        environment = ReferenceEnvironment(sys.executable, {})
        with tempfile.TemporaryDirectory() as temporary:
            substitute = Path(temporary) / "ipopt"
            substitute.write_text(
                f"#!/bin/sh\nprintf 'actual solver output\\n' >&2\n"
                f"head -c {SOLVER_LOG_BYTES + 1} /dev/zero\n"
            )
            substitute.chmod(0o700)
            solver = environment._bind(  # noqa: SLF001 - bind the controlled image
                cast("SystemCallSolver", SolverFactory("ipopt")),
                Executable.select(substitute, "test.overflow"),
                "test.overflow",
            )
            solver._timelimit = None  # noqa: SLF001 - normally set by Pyomo solve
            diagnostics = io.StringIO()
            with (
                redirect_stderr(diagnostics),
                self.assertRaisesRegex(
                    ReferenceAdmissionError, "test.overflow.*diagnostics exceeded"
                ),
            ):
                solver._execute_command(  # noqa: SLF001 - exercise the shell seam directly
                    Bunch(cmd=[str(substitute)], env=None)
                )
            self.assertTrue(diagnostics.getvalue().startswith("actual solver output"))
            self.assertLessEqual(len(diagnostics.getvalue()), SOLVER_LOG_BYTES + 1)
            self.assertNotIn("test.overflow.solves", environment.evidence)
