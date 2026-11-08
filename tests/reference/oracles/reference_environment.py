# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Admission and performed shell-solver evidence for isolated reference cases."""

from __future__ import annotations

import hashlib
import importlib.metadata
import os
import select
import subprocess
import sys
import time
import tomllib
from contextlib import contextmanager
from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING, cast

if TYPE_CHECKING:
    from collections.abc import Generator, Iterable, Mapping

    from pyomo.common.collections import Bunch
    from pyomo.opt.solver.shellcmd import SystemCallSolver

ROOT = Path(__file__).resolve().parents[3]
ORDINARY_PACKAGES = ("idaes-pse", "pyomo", "pint", "pyarrow", "protobuf")
SOLVER_LOG_BYTES = 128 << 10


class ReferenceAdmissionError(ValueError):
    """A missing, stale or substituted prerequisite attributable to its stage."""

    def __init__(self, stage: str, reason: str) -> None:
        """Bind the refusal to the stage that required the selected input."""
        self.stage = stage
        super().__init__(f"reference admission [{stage}]: {reason}")


def locked_pins(packages: Iterable[str]) -> dict[str, str]:
    """Select package records from the parity lock for this interpreter."""
    from packaging.markers import Marker

    with (ROOT / "uv.lock").open("rb") as lock:
        entries = tomllib.load(lock)["package"]
    selected = {}
    for name in sorted(set(packages)):
        versions = {
            entry["version"]
            for entry in entries
            if entry["name"] == name
            and (
                "resolution-markers" not in entry
                or any(
                    Marker(marker).evaluate() for marker in entry["resolution-markers"]
                )
            )
        }
        if len(versions) != 1:
            raise ReferenceAdmissionError(
                "package", f"ambiguous locked version for {name}"
            )
        selected[name] = versions.pop()
    return selected


def _stamp(state: os.stat_result) -> tuple[int, int, int, int, int]:
    return (
        state.st_dev,
        state.st_ino,
        state.st_size,
        state.st_mtime_ns,
        state.st_ctime_ns,
    )


def _file_stamp(path: Path) -> tuple[int, int, int, int, int]:
    return _stamp(path.stat())


def _digest(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


@dataclass(frozen=True)
class Executable:
    """A prepared executable with a stable path, image and observed file stamp."""

    path: Path
    stamp: tuple[int, int, int, int, int]
    sha256: str

    @classmethod
    def select(cls, path: Path, stage: str) -> Executable:
        """Require the configured image before constructing the solver instance."""
        if not path.is_absolute():
            raise ReferenceAdmissionError(stage, f"executable is not absolute: {path}")
        try:
            selected = path.resolve(strict=True)
            if selected.is_file() and os.access(selected, os.X_OK):
                stamp = _file_stamp(selected)
                image = cls(selected, stamp, _digest(selected))
                image.verify(selected, stage)
                return image
        except OSError as error:
            raise ReferenceAdmissionError(
                stage, f"unavailable executable {path}: {error}"
            ) from error
        raise ReferenceAdmissionError(
            stage, f"unavailable executable {path}: expected an executable file"
        )

    def verify(self, actual: Path, stage: str) -> None:
        """Refuse a different path or file before crediting a performed command."""
        try:
            if (
                actual.resolve(strict=True) == self.path
                and _file_stamp(actual) == self.stamp
            ):
                return
        except OSError as error:
            raise ReferenceAdmissionError(
                stage, f"executable unavailable during use: {actual}"
            ) from error
        raise ReferenceAdmissionError(
            stage, f"substituted executable {actual}; expected {self.path}"
        )


def prepared_solver(name: str, stage: str) -> Executable:
    """Resolve from the existing provider selection rather than solver PATH."""
    if name == "petsc":
        # The IDAES AMPL extension is a distinct capability from the native
        # PETSc library provider. It is installed in IDAES's selected data tree.
        from idaes import bin_directory

        if not bin_directory:
            raise ReferenceAdmissionError(
                stage, "IDAES PETSc extension directory absent"
            )
        return Executable.select(Path(bin_directory) / name, stage)
    role = "SCIPOPTDIR" if name == "scip" else "IPOPT_DIR"
    selected = os.environ.get(role)
    if not selected:
        raise ReferenceAdmissionError(stage, f"prepared solver role {role} absent")
    return Executable.select(Path(selected) / "bin" / name, stage)


def _execute_solver(
    solver: SystemCallSolver, command: Bunch, image: Executable, stage: str
) -> tuple[int, str]:
    """Give the pinned shell plugin a bounded log for its normal result parser."""
    from pyomo.opt.solver.shellcmd import (
        SUBPROCESS_TIMEOUT_ABS_ADJUST,
        SUBPROCESS_TIMEOUT_REL_ADJUST,
    )

    if "script" in command:
        raise ReferenceAdmissionError(stage, "selected solver unexpectedly needs stdin")
    # Each selected Pyomo shell plugin supplies an argv list in its command Bunch.
    argv = cast("list[str] | None", command.cmd)
    if not argv:
        raise ReferenceAdmissionError(stage, "performing plugin has no command")
    image.verify(Path(argv[0]), stage)
    environment = cast("Mapping[str, str] | None", command.env)
    directory = cast("str | os.PathLike[str] | None", command.cwd)
    began = time.monotonic()
    timeout = solver._timelimit  # noqa: SLF001 - preserve pinned shell plugin timeout
    if timeout is not None:
        timeout += max(
            SUBPROCESS_TIMEOUT_ABS_ADJUST, SUBPROCESS_TIMEOUT_REL_ADJUST * timeout
        )
    deadline = began + (timeout if timeout is not None else 240)
    log = bytearray()
    fd = os.open(image.path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    try:
        if _stamp(os.fstat(fd)) != image.stamp:
            raise ReferenceAdmissionError(stage, "substituted executable before launch")
        # The descriptor selects the observed executable. Keep Pyomo's argv
        # and its normal solver solution-file and result parsing protocols.
        with subprocess.Popen(  # noqa: S603 - the checked descriptor selects the executable
            argv,
            executable=f"/proc/self/fd/{fd}",
            pass_fds=(fd,),
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            env=environment,
            cwd=directory,
        ) as child:
            try:
                if child.stdout is None:
                    raise ReferenceAdmissionError(
                        stage, "solver diagnostic pipe absent"
                    )
                stream = child.stdout.fileno()
                while True:
                    remaining = deadline - time.monotonic()
                    if (
                        remaining <= 0
                        or not select.select([stream], [], [], remaining)[0]
                    ):
                        raise ReferenceAdmissionError(
                            stage, "solver diagnostic deadline expired"
                        )
                    chunk = os.read(stream, 8192)
                    if not chunk:
                        break
                    available = SOLVER_LOG_BYTES - len(log)
                    log.extend(chunk[:available])
                    if len(chunk) > available:
                        sys.stderr.write(log.decode("utf-8", errors="replace") + "\n")
                        raise ReferenceAdmissionError(
                            stage, "solver diagnostics exceeded their byte limit"
                        )
                status = child.wait(timeout=max(0, deadline - time.monotonic()))
            finally:
                if child.poll() is None:
                    child.kill()
        if _stamp(os.fstat(fd)) != image.stamp:
            raise ReferenceAdmissionError(
                stage, "solver image changed during execution"
            )
    finally:
        os.close(fd)
    solver._last_solve_time = time.monotonic() - began  # noqa: SLF001 - Pyomo result accounting
    output = log.decode("utf-8").replace("\r\n", "\n").replace("\r", "\n")
    if solver._tee:  # noqa: SLF001 - preserve pinned shell plugin tee policy
        sys.stdout.write(output)
        sys.stdout.flush()
    return status, output


class ReferenceEnvironment:
    """Case-owned environment and results observed on performing solver instances."""

    def __init__(self, interpreter: str, pins: Mapping[str, str]) -> None:
        """Admit the running interpreter and observe the selected packages."""
        selected, actual = Path(interpreter), Path(sys.executable)
        if (
            not interpreter
            or not selected.is_absolute()
            or selected != actual
            or not selected.is_file()
            or not os.access(selected, os.X_OK)
            or Path(sys.prefix).resolve() != selected.parent.parent.resolve()
            or sys.prefix == sys.base_prefix
        ):
            raise ReferenceAdmissionError(
                "interpreter",
                f"selected virtualenv interpreter {selected}; observed {actual}",
            )
        observed: dict[str, str] = {}
        for name, expected in pins.items():
            try:
                actual_version = importlib.metadata.version(name)
            except importlib.metadata.PackageNotFoundError as error:
                raise ReferenceAdmissionError(
                    "package", f"missing {name}=={expected}"
                ) from error
            if actual_version != expected:
                raise ReferenceAdmissionError(
                    "package", f"{name} expected {expected}; observed {actual_version}"
                )
            observed[name] = actual_version
        running_digest = _digest(Path("/proc/self/exe"))
        if running_digest != _digest(actual):
            raise ReferenceAdmissionError(
                "interpreter", "running executable differs from selected image"
            )
        self.evidence = {
            **observed,
            "python": ".".join(str(part) for part in sys.version_info[:3]),
            "python.executable": str(actual),
            "python.sha256": running_digest,
        }
        self._executables: dict[str, Executable] = {}
        self._counts: dict[str, int] = {}

    @classmethod
    def from_process(
        cls,
        packages: Iterable[str] = ORDINARY_PACKAGES,
        *,
        role: str = "PSE_ORDINARY_REFERENCE_PYTHON",
        declared: Mapping[str, str] | None = None,
    ) -> ReferenceEnvironment:
        """Bind the isolated process to the caller role and checkout lock."""
        interpreter = os.environ.get(role)
        if not interpreter:
            raise ReferenceAdmissionError("interpreter", f"selected role {role} absent")
        pins = locked_pins(packages)
        if declared is not None and declared != {
            name: pins[name] for name in ("idaes-pse", "pyomo")
        }:
            raise ReferenceAdmissionError(
                "package", f"caller pins differ from selected lock: {declared}"
            )
        return cls(interpreter, pins)

    def _selected(self, executable: str, stage: str) -> Executable:
        if executable not in self._executables:
            self._executables[executable] = prepared_solver(executable, stage)
        return self._executables[executable]

    def _bind(
        self, solver: SystemCallSolver, image: Executable, stage: str
    ) -> SystemCallSolver:
        solver.set_executable(str(image.path))

        def execute(command: Bunch) -> list[int | str]:
            executable = solver.executable()
            if not executable:
                raise ReferenceAdmissionError(
                    stage, "performing plugin has no executable"
                )
            image.verify(Path(executable), stage)
            status, output = _execute_solver(solver, command, image, stage)
            image.verify(Path(executable), stage)
            count = self._counts.get(stage, 0)
            if count == 0:
                # Probe the very instance that executed the command. Its bound
                # executable and stable image identify which release was used.
                version = solver.version()
                image.verify(Path(executable), stage)
                if not version or not all(isinstance(part, int) for part in version):
                    raise ReferenceAdmissionError(
                        stage, "performing solver release unavailable"
                    )
                self.evidence[f"{stage}.release"] = ".".join(
                    str(part) for part in version
                )
                self.evidence[f"{stage}.executable"] = str(image.path)
                self.evidence[f"{stage}.sha256"] = image.sha256
            self._counts[stage] = count + 1
            self.evidence[f"{stage}.solves"] = str(count + 1)
            return [status, output]

        solver._execute_command = execute  # noqa: SLF001 - scoped to this performing instance
        return solver

    def solver(self, plugin: str, stage: str) -> SystemCallSolver:
        """Construct and bind the directly called Pyomo shell plugin."""
        from pyomo.environ import SolverFactory

        image = self._selected(plugin, stage)
        solver = cast("SystemCallSolver", SolverFactory(plugin))
        return self._bind(solver, image, stage)

    @contextmanager
    def registration(
        self, plugin: str, stage: str, *, alias: str, executable: str | None = None
    ) -> Generator[None, None, None]:
        """Bind instances created indirectly through a scoped Pyomo factory name."""
        from pyomo.environ import SolverFactory

        image = self._selected(executable or plugin, stage)
        original = SolverFactory.get_class(alias) if alias in SolverFactory else None
        doc = SolverFactory.doc(alias) if alias in SolverFactory else None
        # Unregistered AMPL solvers use the Pyomo ASL shell plugin.
        registered = plugin in SolverFactory
        constructor = SolverFactory.get_class(plugin if registered else "asl")

        @SolverFactory.register(alias, doc=doc)
        def build(**options: object) -> SystemCallSolver:
            solver = cast("SystemCallSolver", constructor(**options))
            if not registered:
                solver.options.solver = plugin
            return self._bind(solver, image, stage)

        try:
            yield
        finally:
            SolverFactory.unregister(alias)
            if original is not None:
                SolverFactory.register(alias, doc=doc)(original)

    def require_solves(self, stage: str, expected: int) -> None:
        """Require the actual selected stage population before publishing results."""
        count = self._counts.get(stage, 0)
        if count != expected:
            raise ReferenceAdmissionError(
                stage, f"expected {expected} performed solves; observed {count}"
            )
