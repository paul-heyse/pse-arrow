# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""One environment and placement boundary for recipes, bare commands and agent runtimes.

``scripts/pse-env [--native[=caps]] [--no-scope] -- <command>`` runs a command with this
checkout's environment: the caller's values, then ``.envrc.local`` for anything the caller
has not set, then repository defaults (the venv, the local solver stack), then the
compiler/cache configuration of ``build_environment.configure``. Native capabilities are
admitted only on request, under ``native_operation``'s operation owner. A finite host
allocation precedes each owned scope in ``pse.slice``; nested work borrows its actual
owner. ``PSE_MEMORY_MAX`` widens a compatible allocation and its ancestors together.

``--print`` emits shell exports of the ordinary environment (never native setup, never a
local secret's value), for ``.envrc`` and a Claude ``SessionStart`` hook. ``--explain``
reports what a command would get and why, without preparing anything.

Exit status: the command's own; 125 when this boundary fails (with a ``pse-env:`` line);
126/127 when the command cannot be executed.
"""

from __future__ import annotations

import argparse
import contextlib
import functools
import json
import math
import os
import re
import shlex
import shutil
import signal
import subprocess
import sys
import time
import uuid
from pathlib import Path
from typing import TYPE_CHECKING, Literal, TypedDict

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence

ROOT = Path(__file__).resolve().parents[1]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))
from scripts import (  # noqa: E402 -- direct-script path routing
    build_environment,
    host_admission,
    native_operation,
    operation_lifetime,
    surreal_server,
)
from scripts.systemd_environment import (  # noqa: E402 -- same routing
    BoundaryError,
    current_memory_ceiling,
    manager_environment,
    require_unbounded_slice,
    slice_group,
)

host = host_admission
operation = native_operation

FAILURE = 125
LOCAL = ".envrc.local"
DEFAULT_SLICE = "pse.slice"
CARGO_DEFAULT_JOBS = "PSE_CARGO_DEFAULT_JOBS"
NEXTEST_DEFAULT_THREADS = "PSE_NEXTEST_DEFAULT_THREADS"
RESOURCE_CLASSES = (
    "light",
    "compile",
    "build",
    "functional",
    "wide",
    "timing",
    "reference",
    "exclusive",
)
SOLVER_STACK = Path("/opt/pse-solvers")
# Values configure() replaces to keep compiler supervision and checkout isolation.
CONFIGURED = {
    "CARGO_TARGET_DIR": "another checkout's target; choose one with PSE_CARGO_TARGET_DIR",
    "CLANG_PATH": "the selected LLVM installation",
    "LLVM_CONFIG_PATH": "the selected LLVM installation",
    "SCCACHE_CLIENT_SIDE": "compiler children stay in their caller's lifetime",
    "SCCACHE_DIRECT": "direct mode cannot establish recorded absence premises",
    "RUSTC_WRAPPER": "an unsupported sccache configuration disables the wrapper",
}


def local_keys(root: Path, *, deadline: float | None = None) -> dict[str, str]:
    """Exported values of ``.envrc.local``, read in a clean shell and never printed."""
    path = root / LOCAL
    if not path.is_file():
        return {}
    clean = {"HOME": os.environ.get("HOME", "/"), "PATH": "/usr/bin:/bin"}
    script = '. "$1" >/dev/null 2>&1; env -0'
    command = ["bash", "-c", script, "pse-env", str(path)]
    if deadline is None:
        result = subprocess.run(
            command, check=False, capture_output=True, env=clean, timeout=10
        )
        content = result.stdout.decode()
        returncode = result.returncode
    else:
        controlled = host.control_run(command, env=clean, deadline=deadline)
        returncode = controlled.returncode
        content = controlled.stdout
    if returncode:
        raise BoundaryError(f"{LOCAL} failed to load")
    after = dict(item.split("=", 1) for item in content.split("\0") if "=" in item)
    return {
        name: value
        for name, value in after.items()
        if clean.get(name) != value and name not in {"_", "SHLVL", "PWD", "OLDPWD"}
    }


def prepend(path: str, existing: str | None) -> str:
    parts = [
        part for part in (existing or "").split(os.pathsep) if part and part != path
    ]
    return os.pathsep.join([path, *parts])


def venv(root: Path, env: Mapping[str, str]) -> Path:
    selected = Path(env.get("UV_PROJECT_ENVIRONMENT", ".venv"))
    return selected if selected.is_absolute() else root / selected


def base_environment(
    root: Path,
    caller: Mapping[str, str],
    local: Mapping[str, str] | None = None,
    *,
    deadline: float | None = None,
) -> dict[str, str]:
    """The ordinary environment: caller, then local file, then repository defaults."""
    env = dict(caller)
    for name, value in (
        local_keys(root, deadline=deadline) if local is None else local
    ).items():
        env.setdefault(name, value)
    if "PSE_PYTHON" not in env:
        version = root / ".python-version"
        env["PSE_PYTHON"] = (
            version.read_text().strip() if version.is_file() else "3.14.8"
        )
    env.setdefault("UV_PROJECT_ENVIRONMENT", ".venv")
    # The supervisor's own default state; recipes that need the server check it (--store).
    state_home = env.get("XDG_STATE_HOME") or str(
        Path(env.get("HOME", str(Path.home()))) / ".local/state"
    )
    env.setdefault(
        "PSE_SURREAL_STATE", str(Path(state_home) / "pse-arrow/surreal-functional-v2")
    )
    # Cargo resolves the scoped c/t external subcommand from this checkout.
    env["PATH"] = prepend(str(root / "scripts/cargo-bin"), env.get("PATH"))
    env["PATH"] = prepend(str(venv(root, env) / "bin"), env["PATH"])
    if (SOLVER_STACK / "bin").is_dir():
        env["PATH"] = prepend(str(SOLVER_STACK / "bin"), env["PATH"])
        env["PKG_CONFIG_PATH"] = prepend(
            str(SOLVER_STACK / "lib/pkgconfig"), env.get("PKG_CONFIG_PATH")
        )
    from scripts import (  # noqa: PLC0415 -- optional passive selection at the launch boundary.
        telemetry_launch,
    )

    telemetry_launch.configure(root, env)
    return env


def compose(
    root: Path, caller: Mapping[str, str], local: Mapping[str, str] | None = None
) -> dict[str, str]:
    env = base_environment(root, caller, local)
    inherited_threads = env.pop(NEXTEST_DEFAULT_THREADS, None)
    if (
        inherited_threads is not None
        and env.get("NEXTEST_TEST_THREADS") == inherited_threads
    ):
        env.pop("NEXTEST_TEST_THREADS", None)
    inherited_default = env.get(CARGO_DEFAULT_JOBS)
    explicit_jobs = "CARGO_BUILD_JOBS" in env and (
        inherited_default is None or env["CARGO_BUILD_JOBS"] != inherited_default
    )
    if not explicit_jobs:
        # Re-read the declaration on every entry, rather than carrying a previous
        # allocation's narrower default into a newly admitted workload.
        env.pop("CARGO_BUILD_JOBS", None)
    env.pop(CARGO_DEFAULT_JOBS, None)
    try:
        configured = build_environment.configure(root, env)
        if not explicit_jobs and "CARGO_BUILD_JOBS" in configured:
            configured[CARGO_DEFAULT_JOBS] = configured["CARGO_BUILD_JOBS"]
    except ValueError as error:
        raise BoundaryError(str(error)) from error
    return configured


def documentation_environment(root: Path, env: Mapping[str, str]) -> dict[str, str]:
    """Explicit docs selection never exact-syncs the caller's product environment."""
    chosen = root / ".venv-docs"
    product = root / ".venv"
    inherited = venv(root, env)
    if chosen.resolve() == product.resolve() or (
        inherited.resolve() == chosen.resolve() and inherited.name != ".venv-docs"
    ):
        raise BoundaryError(
            ".venv-docs aliases the product environment; choose distinct environment directories before bootstrap"
        )
    result = dict(env)
    result["UV_PROJECT_ENVIRONMENT"] = str(chosen)
    result["PATH"] = prepend(str(chosen / "bin"), result.get("PATH"))
    result["PSE_DOCUMENTATION_ENVIRONMENT"] = "1"
    return result


# Values an agent sets deliberately; the others usually come from a login profile, so
# their replacement is reported by --explain rather than on every command.
ANNOUNCED = ("CARGO_TARGET_DIR", "RUSTC_WRAPPER")


def refusals(
    chosen: Mapping[str, str],
    env: Mapping[str, str],
    names: Sequence[str] = tuple(CONFIGURED),
) -> list[str]:
    """Values the caller or .envrc.local chose that composition replaced."""
    notes = []
    for name in names:
        reason = CONFIGURED[name]
        prior = chosen.get(name)
        if prior is not None and env.get(name) != prior:
            now = env.get(name)
            shown = "unset" if now is None else now
            notes.append(f"pse-env: refused {name}={prior}: {reason}; using {shown}")
    return notes


def deferred(root: Path, name: str) -> str:
    """Read a local value at use, keeping it out of the rendered text."""
    source = shlex.quote(str(root / LOCAL))
    return (
        f'[ -n "${{{name}+x}}" ] || {{ {name}="$(. {source} >/dev/null 2>&1; '
        f'printf %s "${{{name}-}}")" && export {name}; }}'
    )


def required_path(
    root: Path, caller: Mapping[str, str], local: Mapping[str, str]
) -> list[str]:
    """PATH entries composition puts first, independent of the caller's own PATH."""
    bare = compose(root, {"HOME": caller.get("HOME", "/"), "PATH": ""}, local)
    return [entry for entry in bare.get("PATH", "").split(os.pathsep) if entry]


def render(
    root: Path, caller: Mapping[str, str], *, complete: bool = False, docs: bool = False
) -> str:
    """Shell text for ``eval``: ordinary exports only, local values deferred.

    PATH is rendered as idempotent prepends, because the shell that evaluates the text
    (direnv, or a Claude Bash command after its shell snapshot) may not share the
    caller's PATH. ``complete`` also exports values equal to the caller's, for a consumer
    whose environment is not the caller's (a SessionStart hook's env file).
    """
    local = local_keys(root)
    env = compose(root, caller, local)
    if docs:
        env = documentation_environment(root, env)
    touched = set(
        compose(root, {"HOME": caller.get("HOME", "/"), "PATH": ""}, local)
    ) - {"HOME"}
    paths = required_path(root, caller, local)
    if docs:
        paths.insert(0, str(root / ".venv-docs/bin"))
    lines = [
        f'case ":$PATH:" in *:{shlex.quote(entry)}:*) ;; *) PATH={shlex.quote(entry)}"${{PATH:+:$PATH}}" ;; esac'
        for entry in reversed(paths)
    ]
    if docs:
        docs_bin = shlex.quote(str(root / ".venv-docs/bin"))
        lines.append(
            f'case "$PATH" in {docs_bin}|{docs_bin}:*) ;; *) PATH={docs_bin}"${{PATH:+:$PATH}}" ;; esac'
        )
    lines.append("export PATH")
    for name in sorted(env):
        if (
            (name in local and not (docs and name == "UV_PROJECT_ENVIRONMENT"))
            or name == "PATH"
            or name.startswith("PSE_NATIVE_")
        ):
            continue
        if caller.get(name) != env[name] or (complete and name in touched):
            lines.append(f"export {name}={shlex.quote(env[name])}")
    lines.extend(
        f"unset {name}" for name in sorted(caller.keys() - env.keys()) if name != "PATH"
    )
    lines.extend(
        deferred(root, name)
        for name in sorted(local)
        if name not in caller and not (docs and name == "UV_PROJECT_ENVIRONMENT")
    )
    return "\n".join(lines) + "\n"


def parse_bytes(value: str) -> int | None:
    match = re.fullmatch(r"(\d+)([KMGT]?)", value.strip().upper())
    if not match:
        return None
    scale = {"": 1, "K": 1 << 10, "M": 1 << 20, "G": 1 << 30, "T": 1 << 40}
    return int(match.group(1)) * scale[match.group(2)]


def slice_name(env: Mapping[str, str]) -> str | None:
    name = env.get("PSE_SLICE", DEFAULT_SLICE)
    if name in {"", "none"}:
        return None
    if not SLICE_VALUE.fullmatch(name):
        raise BoundaryError(
            f"PSE_SLICE={name!r} is not a slice unit name (e.g. pse.slice) or none"
        )
    return name


def selected_resource_class(
    command: Sequence[str],
    env: Mapping[str, str],
    *,
    native: bool,
    explicit: str | None = None,
    default: str | None = None,
) -> str:
    """Caller choices win; local unbounded mode changes ordinary recipe defaults only."""
    if explicit is not None:
        return explicit
    selected = env.get("PSE_RESOURCE_CLASS")
    if selected is not None:
        return selected
    mode = env.get("PSE_BUILD_MODE", "bounded")
    if mode not in {"bounded", "unbounded"}:
        raise BoundaryError("PSE_BUILD_MODE must be bounded or unbounded")
    selected = default or host.classify(command, native)
    if (
        mode == "unbounded"
        and selected in {"compile", "functional", "wide", "exclusive"}
        and "PSE_MEMORY_MAX" not in env
        and operation.BUILD_TIMING not in env
        and "--managed-primary-route" not in command
    ):
        return "build"
    return selected


def require_store(env: Mapping[str, str], *, deadline: float | None = None) -> None:
    """Fail early, with the fix, when the selected canonical server is not serving."""
    state = Path(env["PSE_SURREAL_STATE"])
    config_path = state / "config.json"
    if not config_path.is_file():
        raise BoundaryError(
            f"canonical store {state} is not set up; run: just surreal setup --state {state}"
            f" && just surreal start --state {state} && just canonical-init {state}"
            " (or choose another with PSE_SURREAL_STATE)"
        )
    from scripts import surreal_server  # noqa: PLC0415 -- supervisor owns readiness

    clock = (
        contextlib.nullcontext()
        if deadline is None
        else surreal_server.lifecycle_deadline(deadline)
    )
    with clock as effective:
        deadline = effective if deadline is not None else None
        offered = {} if deadline is None else {"deadline": deadline}
        config = (
            json.loads(config_path.read_text())
            if deadline is None
            else surreal_server.config_for(state, deadline=deadline)
        )
        owner = (
            host.inherit(env)
            if deadline is None
            else host.inherit(env, deadline=deadline)
        )
        if deadline is not None:
            surreal_server.remaining(deadline)
        if owner is None:
            if config.get("admission") not in {"open", "quiesced"} or (
                config.get("admission") == "quiesced" and not config.get("parked")
            ):
                raise BoundaryError("Selected canonical service is not admitted")
            return  # Demand readiness inside the registered actual allocation.
        if config.get("parked"):
            surreal_server.unpark_service(state, **offered)
            config = surreal_server.config_for(state, **offered)

        def serving() -> bool:
            if deadline is None:
                return surreal_server.ready(state, config)
            observed = surreal_server.service_readiness(
                state, config, deadline=deadline
            )
            surreal_server.remaining(deadline)
            return observed.phase is surreal_server.Readiness.READY

        if config.get("admission") == "open" and not serving():
            if deadline is not None:
                surreal_server.remaining(deadline)
            if deadline is None:
                surreal_server.start(state, config)
            else:
                surreal_server.start(state, config, deadline=deadline)
        if config.get("admission") != "open" or not serving():
            raise BoundaryError(
                f"canonical server for {state} is not ready; run: just surreal start --state {state}"
            )


SLICE_VALUE = re.compile(r"[A-Za-z0-9_:.-]+\.slice")


@functools.cache
def manager_available() -> bool:
    """A user manager that actually answers, not just a socket path."""
    if (
        shutil.which("systemd-run") is None
        or not Path(f"/run/user/{os.getuid()}/bus").is_socket()
    ):
        return False
    try:
        probe = subprocess.run(
            ["systemctl", "--user", "show-environment"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            env={**os.environ, **manager_environment()},
            timeout=5,
            check=False,
        )
    except (OSError, subprocess.SubprocessError):
        return False
    return probe.returncode == 0


def cpu_ceiling(group: Path, *, root: Path = Path("/sys/fs/cgroup")) -> float | None:
    """Smallest enforced quota in the actual cgroup ancestry."""
    ceilings = []
    while group.is_relative_to(root):
        path = group / "cpu.max"
        if path.is_file():
            quota, period = path.read_text().split()
            if int(period) <= 0 or (quota != "max" and int(quota) <= 0):
                raise BoundaryError("Invalid enclosing CPU quota")
            if quota != "max":
                ceilings.append(int(quota) / int(period))
        if group == root:
            break
        group = group.parent
    return min(ceilings) if ceilings else None


def cpu_widths(
    allocation: host.Allocation, *, pending: bool = False
) -> tuple[int, int]:
    """Physical runner and logical compiler widths within actual CPU ceilings."""
    cpus = set(host.cpu_set(allocation.profile.cores))
    if pending:
        # Before launch, observe the enforced destination, not the coordinator's
        # unrelated lane. The child binds and re-observes its actual affinity.
        selected = host.allocation_slice(allocation)
        if selected is None:
            raise BoundaryError("Cannot observe admitted CPU placement")
        group = slice_group(selected)
    else:
        cpus.intersection_update(os.sched_getaffinity(0))
        membership = operation_lifetime.process_group(os.getpid())
        if not membership:
            raise BoundaryError("Cannot observe actual enclosing CPU placement")
        group = Path("/sys/fs/cgroup") / membership.lstrip("/")
    quota = cpu_ceiling(group)
    if quota is None or not math.isfinite(quota) or quota <= 0:
        raise BoundaryError("Admitted CPU allocation has no enforced finite quota")
    logical = min(len(cpus), max(1, math.floor(quota)))
    if logical < 1:
        raise BoundaryError("Admitted CPU allocation has no usable lane")
    return min(len(set(allocation.profile.cores)), logical), logical


def runner_width(allocation: host.Allocation, *, pending: bool = False) -> int:
    """Intersect physical runner width with usable affinity and finite quota."""
    return cpu_widths(allocation, pending=pending)[0]


def default_test_width(environment: Mapping[str, str]) -> int:
    allocation = host.inherit(environment)
    if allocation is None:
        raise BoundaryError("Default test width requires an admitted enclosing owner")
    return runner_width(allocation)


def admitted_cargo_jobs(
    env: dict[str, str], allocation: host.Allocation, *, pending: bool = False
) -> None:
    """Use physical runner defaults and logical compiler capacity; preserve overrides."""
    width, compiler_width = cpu_widths(allocation, pending=pending)
    previous = env.get(NEXTEST_DEFAULT_THREADS)
    if "NEXTEST_TEST_THREADS" not in env or env["NEXTEST_TEST_THREADS"] == previous:
        env["NEXTEST_TEST_THREADS"] = str(width)
        env[NEXTEST_DEFAULT_THREADS] = str(width)
    declared = env.get(CARGO_DEFAULT_JOBS)
    if declared is None or env.get("CARGO_BUILD_JOBS") != declared:
        return
    jobs = str(min(int(declared), compiler_width))
    env["CARGO_BUILD_JOBS"] = jobs
    env[CARGO_DEFAULT_JOBS] = jobs


def placement(env: dict[str, str], *, native: bool) -> list[str]:
    """systemd-run arguments that give the command its own scope, or nothing."""
    allocation = host.inherit(env)
    if allocation is not None:
        admitted_cargo_jobs(env, allocation)
        if allocation.profile.unbounded:
            require_unbounded_slice(host.allocation_slice(allocation))
            if env.get("PSE_MEMORY_MAX") is not None:
                raise BoundaryError(
                    "A finite request cannot inherit an unbounded build allocation"
                )
            desired = env.get("PSE_RESOURCE_CLASS", "build")
            if desired not in {"build", "light", "compile"}:
                raise BoundaryError(
                    "A scientific role requires its own finite allocation"
                )
            return []
        requested = env.get("PSE_MEMORY_MAX")
        if (
            requested is not None
            and host.finite_bytes(requested) > allocation.profile.memory
        ):
            raise BoundaryError(
                "Nested memory request exceeds the admitted enclosing allocation; start this work in a widened allocation"
            )
        actual = current_memory_ceiling()
        if (
            requested is not None
            and actual is not None
            and host.finite_bytes(requested) > actual
        ):
            raise BoundaryError(
                "Nested memory request exceeds the actual enclosing role/ancestor cap; enter a widened role before execution"
            )
        requested_class = env.get("PSE_RESOURCE_CLASS")
        if requested_class and requested_class != allocation.profile.name:
            desired = host.select(requested_class, requested)
            # Light and compile classify nested work, not a demand to move its
            # inherited CPU lane. Explicit scientific lanes keep that refusal.
            changes_lane = requested_class not in {"light", "compile"} and not set(
                desired.cores
            ).issubset(host.cpu_set(allocation.profile.cores))
            if (
                (desired.exclusive and not allocation.profile.exclusive)
                or desired.memory > allocation.profile.memory
                or changes_lane
            ):
                raise BoundaryError(
                    "Nested resource class exceeds the admitted enclosing allocation; start a new matching allocation"
                )
        return []
    if operation_lifetime.scope_owner() is not None:
        raise BoundaryError("Native operation has no verified host allocation")
    slice_name(env)  # Validate an explicit caller selection before admission.
    if not manager_available():
        raise BoundaryError(
            "Coordinated command placement requires the systemd user manager"
        )
    for name, value in manager_environment().items():
        env.setdefault(name, value)
    kind = "native" if native else "cmd"
    profile = host.select(
        env.get("PSE_RESOURCE_CLASS", "functional"), env.get("PSE_MEMORY_MAX")
    )
    from scripts import surreal_server  # noqa: PLC0415 -- command/service composition

    allocation = host.acquire(
        profile,
        directory=host.root_path(env),
        services=surreal_server.ServiceAdmission(),
    )
    env.update(allocation.environment())
    try:
        host.enforce_parent(profile, env)
        host.enforce_allocation(allocation, env)
        admitted_cargo_jobs(env, allocation, pending=True)
    except Exception:
        allocation.release()
        raise
    selected = host.allocation_slice(allocation)
    cap = None if profile.unbounded else str(profile.memory)
    unit = f"pse-{kind}-{uuid.uuid4().hex}.scope"
    allocation.register(unit)
    command = [
        "systemd-run",
        "--user",
        "--scope",
        "--quiet",
        "--collect",
        f"--unit={unit}",
    ]
    if selected is not None:
        command.append(f"--slice={selected}")
    if cap is not None:
        command += ["-p", f"MemoryMax={cap}", "-p", "MemorySwapMax=0"]
    command += [
        "-p",
        "AllowedCPUs=" + " ".join(map(str, host.cpu_set(profile.cores))),
        "-p",
        f"CPUQuota={len(host.cpu_set(profile.cores)) * 100}%",
    ]
    return [
        *command,
        "--",
        setup_python(ROOT, env),
        str(ROOT / "scripts/pse_env.py"),
        "--bind-allocation",
        unit,
        "--no-scope",
        "--",
    ]


def control_environment(
    root: Path, caller: Mapping[str, str], *, deadline: float
) -> dict[str, str]:
    """Ordinary checkout defaults for a fixed helper, without compiler preparation."""
    host.remaining(deadline, 1)
    return base_environment(root, caller, deadline=deadline)


def run_light_control(
    command: Sequence[str], env: Mapping[str, str], *, deadline: float
) -> subprocess.CompletedProcess[str]:
    """Internal fixed-control launch; no native setup or service recovery effects.

    The caller establishes its absolute deadline before environment composition and
    passes only its fixed control child. This is deliberately not a pse-env CLI mode.
    """
    selected_env = dict(env)
    allocation = host.inherit(selected_env, deadline=deadline)
    owned = allocation is None
    prefix = []
    if owned:
        selected_env.update(manager_environment())
        probe = host.control_run(
            ["systemctl", "--user", "show-environment"],
            env=selected_env,
            deadline=deadline,
            maximum=5,
        )
        if probe.returncode:
            raise BoundaryError("Control placement requires the systemd user manager")
        allocation = host.acquire_light_control(
            directory=host.root_path(selected_env), deadline=deadline
        )
        selected_env.update(allocation.environment())
        try:
            host.enforce_parent(allocation.profile, selected_env, deadline=deadline)
            host.enforce_allocation(allocation, selected_env, deadline=deadline)
            unit = f"pse-control-{uuid.uuid4().hex}.scope"
            allocation.register(unit, deadline=deadline)
            prefix = [
                "systemd-run",
                "--user",
                "--scope",
                "--quiet",
                "--collect",
                f"--unit={unit}",
                f"--slice={host.allocation_slice(allocation)}",
                "-p",
                f"MemoryMax={allocation.profile.memory}",
                "-p",
                "MemorySwapMax=0",
                "-p",
                "AllowedCPUs="
                + " ".join(map(str, host.cpu_set(allocation.profile.cores))),
                "-p",
                f"CPUQuota={len(host.cpu_set(allocation.profile.cores)) * 100}%",
                "--",
            ]
        except BaseException:
            host.release_light_control(allocation, deadline=deadline)
            raise
    try:
        return host.control_run(
            [*prefix, *command], env=selected_env, deadline=deadline
        )
    finally:
        if (
            owned
            and allocation is not None
            and not host.release_light_control(allocation, deadline=deadline)
        ):
            raise BoundaryError("Control allocation retained until verified drain")


def setup_python(root: Path, env: Mapping[str, str]) -> str:
    chosen = env.get("PSE_NATIVE_SETUP_PYTHON")
    if chosen:
        return chosen
    candidate = venv(root, env) / "bin/python"
    return str(candidate) if candidate.exists() else sys.executable


def unexecutable(command: Sequence[str], env: Mapping[str, str]) -> int | None:
    """126/127 for a command that cannot run, decided before any scope wraps it."""
    name = command[0]
    path = name if "/" in name else shutil.which(name, path=env.get("PATH"))
    if path is None or not Path(path).exists():
        print(f"pse-env: command not found: {name}", file=sys.stderr)
        return 127
    if Path(path).is_dir() or not os.access(path, os.X_OK):
        print(f"pse-env: command not executable: {name}", file=sys.stderr)
        return 126
    return None


def execute(command: Sequence[str], env: Mapping[str, str]) -> int:
    try:
        os.execvpe(command[0], list(command), dict(env))  # noqa: S606 -- the boundary becomes the command
    except FileNotFoundError:
        print(f"pse-env: command not found: {command[0]}", file=sys.stderr)
        return 127
    except PermissionError:
        print(f"pse-env: command not executable: {command[0]}", file=sys.stderr)
        return 126


def owned_scope(command: Sequence[str], env: dict[str, str]) -> int:
    """Run a newly placed scope and settle only its acquired allocation.

    Call only with a nonempty placement prefix. Inherited placement returns no
    prefix and never enters this terminal owner. Release refusal is a boundary
    failure; diagnostics retain the independent raw and normalized child status.
    """
    allocation = host.inherit(env, services=surreal_server.ServiceAdmission())
    if allocation is None:
        raise BoundaryError("New scope has no acquired allocation to settle")
    raw_status: int | None = None
    child_status: int | None = None
    cancellation_deadline: float | None = None
    try:
        units = [
            value.removeprefix("--unit=")
            for value in command
            if value.startswith("--unit=")
        ]
        if (
            command[0] == "systemd-run"
            and len(units) == 1
            and re.fullmatch(r"pse-native-[a-f0-9]{32}\.scope", units[0])
        ):
            # The admission owner already registered this exact fresh placement.
            # Its external observer survives the child's mandatory whole-scope
            # kill and carries cancellation through the existing bound handoff.
            with host.allocation_metadata(allocation.directory) as ledger:
                registered = ledger["owners"].get(allocation.nonce)
                if (
                    not isinstance(registered, dict)
                    or registered["units"].get(units[0]) != {}
                ):
                    raise BoundaryError(
                        "Native placement lost its fresh allocation registration"
                    )
            from scripts import (  # noqa: PLC0415 -- cache and environment owners import each other
                native_cache as cache,
            )

            observation: dict[str, object] = {}
            with operation.Operation(cache.cache_root(env)) as observer:
                handoff = operation.prepare_handoff(units[0], foreground=True)
                if handoff is None:
                    raise BoundaryError(
                        "Native placement lacks its exact foreground handoff"
                    )
                child_env = dict(env)
                child_env.pop(operation.MARKER, None)
                child_env["PSE_NATIVE_HANDOFF"] = str(handoff)
                normal_deadline: float | None = None

                def selected_cancellation_clock(
                    child: Mapping[str, object],
                ) -> float | None:
                    clocks = [
                        cancellation_deadline,
                        child.get("cancel_deadline"),
                        observation.get("cancellation_deadline"),
                        operation._record(observer.path).get("cancel_deadline"),  # noqa: SLF001 -- exact enclosing cancellation owner
                    ]
                    selected_clocks: list[float] = []
                    for value in clocks:
                        if value is None:
                            continue
                        try:
                            selected_clocks.append(
                                operation.original_cancellation_clock(value)
                            )
                        except ValueError as error:
                            raise BoundaryError(
                                "Invalid original cancellation clock"
                            ) from error
                    return min(selected_clocks) if selected_clocks else None

                def settle_scope() -> None:
                    nonlocal normal_deadline, cancellation_deadline
                    # Reaping can precede the manager's terminal transition.
                    # Join while run still owns its non-raising signal handlers,
                    # under one cleanup clock and any earlier cancellation clock.
                    normal_deadline = time.monotonic() + 10
                    previous_mask: set[int | signal.Signals] | None = None
                    published_deadline: float | None = None
                    try:
                        while True:
                            child = operation._record(handoff)  # noqa: SLF001 -- authenticated foreground handoff protocol
                            cancellation_deadline = selected_cancellation_clock(child)
                            deadline = (
                                min(normal_deadline, cancellation_deadline)
                                if cancellation_deadline is not None
                                else normal_deadline
                            )
                            if cancellation_deadline is not None:
                                if previous_mask is None:
                                    previous_mask = signal.pthread_sigmask(
                                        signal.SIG_BLOCK,
                                        {signal.SIGINT, signal.SIGTERM},
                                    )
                                if published_deadline != cancellation_deadline:
                                    with operation.record_lock(
                                        observer.path, deadline=deadline
                                    ):
                                        registered = operation._record(observer.path)  # noqa: SLF001 -- publish the original clock on its authenticated owner
                                        if (
                                            registered.get("cancel_deadline")
                                            is not None
                                        ):
                                            cancellation_deadline = min(
                                                cancellation_deadline,
                                                operation.original_cancellation_clock(
                                                    registered["cancel_deadline"]
                                                ),
                                            )
                                            deadline = min(
                                                deadline, cancellation_deadline
                                            )
                                        registered["cancel_deadline"] = (
                                            cancellation_deadline
                                        )
                                        operation_lifetime.write_json(
                                            observer.path, registered
                                        )
                                    published_deadline = cancellation_deadline
                            completion = operation_lifetime.drained(
                                child, deadline=deadline
                            )
                            if completion.complete:
                                return
                            if (
                                cancellation_deadline is None
                                and completion.phase
                                in {
                                    operation_lifetime.DrainPhase.POPULATED,
                                    operation_lifetime.DrainPhase.ACTIVE,
                                }
                                and observation.get("cancellation_deadline") is not None
                            ):
                                # A first signal arrived during observation. Read
                                # its original clock before the next join step.
                                continue
                            if completion.phase not in {
                                operation_lifetime.DrainPhase.POPULATED,
                                operation_lifetime.DrainPhase.ACTIVE,
                            }:
                                raise BoundaryError(
                                    completion.diagnostic()
                                    + "; native placement remains charged"
                                )
                            time.sleep(
                                min(
                                    0.02,
                                    operation_lifetime.admission_remaining(deadline),
                                )
                            )
                    finally:
                        if previous_mask is not None:
                            signal.pthread_sigmask(signal.SIG_SETMASK, previous_mask)

                try:
                    child_status = operation.run(
                        list(command),
                        child_env,
                        observation=observation,
                        foreground=handoff,
                        on_reaped=settle_scope,
                        cleanup_clock=lambda: normal_deadline,
                    )
                    return child_status
                finally:
                    observed_status = observation.get("child_status")
                    if observed_status is None:
                        raw_status = None
                    elif isinstance(observed_status, int) and not isinstance(
                        observed_status, bool
                    ):
                        raw_status = observed_status
                    else:
                        raise BoundaryError("Native child status is not an integer")
                    # run restores its handlers after on_reaped returns. Preserve
                    # a first signal received in that final handoff for release.
                    cancellation_deadline = selected_cancellation_clock(
                        operation._record(handoff)  # noqa: SLF001 -- final exact foreground cancellation handoff
                    )
        raw_status = subprocess.call(command, env=env)
        child_status = 128 - raw_status if raw_status < 0 else raw_status
        return child_status
    finally:
        outcome = f"child_exit={child_status!r}; raw_child_status={raw_status!r}"
        identity = env.get(host.MARKER, "<acquired allocation identity unavailable>")
        released = False
        try:
            released = allocation.release(
                **(
                    {"deadline": cancellation_deadline}
                    if cancellation_deadline is not None
                    else {}
                )
            )
        except Exception as error:
            raise BoundaryError(
                f"own allocation release failed: {identity}; {outcome}; {error}"
            ) from error
        if not released:
            raise BoundaryError(
                f"own allocation retained until actual drain: {identity}; {outcome}"
            )


def native(
    root: Path,
    requested: list[str],
    command: list[str],
    env: dict[str, str],
    *,
    scope: bool,
) -> int:
    """Native capabilities are admitted only under an operation owner."""
    if operation.owner_record() is not None:
        # Nested: the outer operation keeps ownership of setup and cancellation.
        return execute(command, operation.environment(requested, env))
    prefix = placement(env, native=True) if scope else []
    if prefix:
        return owned_scope(
            [
                *prefix,
                setup_python(root, env),
                str(root / "scripts/pse_env.py"),
                "--no-scope",
                "--native=" + ",".join(requested),
                "--",
                *command,
            ],
            env,
        )
    from scripts import native_cache as cache  # noqa: PLC0415 -- owner cycle

    os.environ.clear()
    os.environ.update(env)
    with operation.Operation(cache.cache_root(env)):
        if os.environ.get("PSE_NATIVE_HANDOFF"):
            operation.bind_handoff(Path(os.environ["PSE_NATIVE_HANDOFF"]))
        return operation.run(
            command, operation.environment(requested, dict(os.environ))
        )


SelectionSource = Literal[
    "caller", "local", "default", "compiler selection", "native selection"
]


class ChoiceDescription(TypedDict):
    value: str | None
    source: SelectionSource
    local_unresolved: bool


class ProviderDescription(TypedDict):
    inputs: dict[str, ChoiceDescription]
    reuse: str
    effect: str


class PlacementDescription(TypedDict):
    slice: str | None
    resource_class: str
    memory_request: str | None
    admission: str


class SelectorDescription(TypedDict):
    values: list[str | None]
    source: Literal["explicit argv", "tool default or configuration; not resolved"]


class BuildIntentDescription(TypedDict):
    package: SelectorDescription
    profile: SelectorDescription
    features: SelectorDescription
    target: SelectorDescription
    manifest: SelectorDescription
    binary: SelectorDescription
    role: str
    interpretation: str


class OperationDescription(TypedDict):
    version: Literal[1]
    kind: Literal["operation-selection"]
    selection_status: str
    checkout: str
    interpreter: str
    python_package: str
    path: list[str]
    tools: dict[str, str | None]
    compiler: dict[str, ChoiceDescription]
    target_directory: str
    native_capabilities: list[str] | None
    providers: dict[str, ProviderDescription]
    native_runtime_choices: dict[str, ChoiceDescription]
    placement: PlacementDescription
    command: list[str]
    build_context: BuildIntentDescription
    local_configuration: Literal["unresolved: not executed", "supplied", "absent"]
    refusals: list[str]


def build_intent(command: Sequence[str]) -> BuildIntentDescription:
    """Expose explicit selectors, without pretending to execute a tool's parser."""

    def selected(flags: tuple[str, ...]) -> list[str | None]:
        values = []
        for index, argument in enumerate(command):
            name, separator, value = argument.partition("=")
            if name in flags:
                values.append(
                    value
                    if separator
                    else command[index + 1]
                    if index + 1 < len(command)
                    else None
                )
        return values

    selectors: dict[str, SelectorDescription] = {}
    for label, flags in (
        ("package", ("--package", "-p")),
        ("profile", ("--profile",)),
        ("features", ("--features", "-F")),
        ("target", ("--target",)),
        ("manifest", ("--manifest-path", "--cdylib-manifest", "-m")),
        ("binary", ("--bin",)),
    ):
        values = selected(flags)
        selectors[label] = {
            "values": values,
            "source": "explicit argv"
            if values
            else "tool default or configuration; not resolved",
        }
    role = (
        "python"
        if command and Path(command[0]).name == "maturin"
        else {
            "pse-py": "python",
            "pse-runtime": "runtime",
            "xtask": "worker if pse-worker is selected",
        }.get(next(iter(selected(("--package", "-p"))), "") or "", "not selected")
    )
    return {
        "package": selectors["package"],
        "profile": selectors["profile"],
        "features": selectors["features"],
        "target": selectors["target"],
        "manifest": selectors["manifest"],
        "binary": selectors["binary"],
        "role": role,
        "interpretation": "intent only; actual installer child evidence is separate",
    }


def operation_description(
    root: Path,
    caller: Mapping[str, str],
    requested: list[str] | None,
    *,
    local: Mapping[str, str] | None = None,
    command: Sequence[str] = (),
    docs: bool = False,
    resource_class: str | None = None,
    recipe_default_class: str | None = None,
) -> OperationDescription:
    """Describe selection only: never evaluate local shell code or admit a workload.

    An execution owner can supply its already evaluated local mapping. Without it,
    defaults are provisional wherever .envrc.local could supply an absent value.
    The real execution path and this view both use compose's compiler selection.
    """
    local_pending = local is None and (root / LOCAL).is_file()
    selected_local = {} if local is None else local
    env = compose(root, caller, selected_local)
    if docs:
        env = documentation_environment(root, env)

    def choice(name: str, default: str | None = None) -> ChoiceDescription:
        value = env.get(name, default)
        source: SelectionSource = (
            "caller"
            if name in caller
            else "local"
            if name in selected_local
            else "default"
        )
        if name in CONFIGURED and {**selected_local, **caller}.get(name) != value:
            source = "compiler selection"
        if name == "CARGO_BUILD_JOBS" and env.get(CARGO_DEFAULT_JOBS) == value:
            source = "default"
        return {
            "value": value,
            "source": source,
            "local_unresolved": local_pending and name not in caller,
        }

    names = (
        "CARGO_BUILD_JOBS",
        "NEXTEST_TEST_THREADS",
        "PSE_BUILD_MODE",
        "RUSTC_WRAPPER",
        "RUSTC_WORKSPACE_WRAPPER",
        "PSE_NATIVE_COMPILER_CACHE",
        "PSE_SCCACHE_BINARY",
        "SCCACHE_DIR",
        "PSE_LLVM_PREFIX",
        "CLANG_PATH",
        "LLVM_CONFIG_PATH",
        "LIBCLANG_PATH",
        "BINDGEN_EXTRA_CLANG_ARGS",
        "PYO3_CONFIG_FILE",
        "PYO3_PYTHON",
        "PYO3_BUILD_EXTENSION_MODULE",
        "PYO3_ENVIRONMENT_SIGNATURE",
        "PYTHON_SYS_EXECUTABLE",
        "CARGO_BUILD_TARGET",
    )
    tools = {
        name: shutil.which(name, path=env.get("PATH"))
        for name in ("python", "cargo", "rustc", "maturin", "clang", "patchelf")
    }
    providers: dict[str, ProviderDescription] = {}
    for capability in requested or []:
        providers[capability] = {
            "inputs": {
                name: choice(name) for name in operation.CAPABILITY_PATHS[capability]
            },
            "reuse": "unknown until actual preparation/admission",
            "effect": "validate explicit provider or select/build a managed generation",
        }
    selected_class = selected_resource_class(
        command,
        env,
        native=requested is not None,
        explicit=resource_class,
        default=recipe_default_class,
    )
    runtime = (
        operation.runtime_selection(requested, env, root=root)
        if requested is not None
        else env
    )
    return {
        "version": 1,
        "kind": "operation-selection",
        "selection_status": "provisional: local configuration is unevaluated"
        if local_pending
        else "selected intent; execution/admission unobserved",
        "checkout": str(root),
        "interpreter": str(venv(root, env) / "bin/python"),
        "python_package": str(root / "python/pse"),
        "path": env.get("PATH", "").split(os.pathsep),
        "tools": tools,
        "compiler": {name: choice(name) for name in names},
        "target_directory": env.get("CARGO_TARGET_DIR", str(root / "target")),
        "native_capabilities": requested,
        "providers": providers,
        "native_runtime_choices": {
            name: {
                "value": runtime.get(name),
                "source": "native selection"
                if runtime.get(name) != env.get(name)
                else choice(name)["source"],
                "local_unresolved": local_pending and name not in caller,
            }
            for name in (*operation.OVERRIDABLE, *operation.AUTHORITATIVE)
        }
        if requested is not None
        else {},
        "placement": {
            "slice": "psebuild.slice" if selected_class == "build" else slice_name(env),
            "resource_class": selected_class,
            "memory_request": env.get("PSE_MEMORY_MAX"),
            "admission": "not observed or acquired",
        },
        "command": list(command[:2])
        if len(command) > 1
        and command[1]
        in {
            "build",
            "rustc",
            "run",
            "check",
            "test",
            "develop",
            "producer-identity",
            "producer-deployment",
        }
        else list(command[:1]),
        "build_context": build_intent(command),
        "local_configuration": "unresolved: not executed"
        if local_pending
        else "supplied"
        if local is not None
        else "absent",
        "refusals": refusals({**selected_local, **caller}, env),
    }


def explain(description: Mapping[str, object]) -> str:
    return (
        "\n".join(
            f"{name:22} {json.dumps(value, ensure_ascii=False)}"
            for name, value in description.items()
        )
        + "\n"
    )


def required_allocation(
    env: Mapping[str, str], *, handoff: bool = False
) -> host.Allocation:
    allocation = host.inherit(env, handoff=handoff)
    if allocation is None:
        raise BoundaryError("Missing registered allocation handoff")
    return allocation


def require_enclosing_owner(env: Mapping[str, str], *, no_scope: bool) -> None:
    if no_scope and host.inherit(env) is None:
        raise BoundaryError("--no-scope requires an actual admitted enclosing owner")


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="pse-env", description=__doc__)
    parser.add_argument("--print", action="store_true", dest="print_exports")
    parser.add_argument("--explain", action="store_true")
    parser.add_argument(
        "--json", action="store_true", help="structured selection with --explain"
    )
    parser.add_argument(
        "--docs",
        action="store_true",
        help="select the provisioned docs-only environment (never syncs)",
    )
    parser.add_argument("--native", nargs="?", const="__default__", default=None)
    parser.add_argument("--no-scope", action="store_true")
    parser.add_argument(
        "--resource-class",
        choices=RESOURCE_CLASSES,
    )
    parser.add_argument(
        "--recipe-default-class",
        choices=RESOURCE_CLASSES,
        help="launch class when neither an explicit option nor caller/local value selects one",
    )
    parser.add_argument("--bind-allocation", help=argparse.SUPPRESS)
    parser.add_argument(
        "--store",
        action="store_true",
        help="require a serving canonical server at $PSE_SURREAL_STATE first",
    )
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args(argv)
    if args.docs and args.native is not None:
        parser.error("--docs and --native select different environments")
    if args.json and not args.explain:
        parser.error("--json requires --explain")
    if args.print_exports and args.explain:
        parser.error("--print and --explain select different operations")
    caller = dict(os.environ)
    requested: list[str] | None = None
    if args.native is not None:
        value = (
            caller.get("PSE_NATIVE_CAPABILITIES", "solver,klu,isolation,uno,petsc")
            if args.native == "__default__"
            else args.native
        )
        requested = [item for item in value.split(",") if item]
        if any(item not in operation.CAPABILITIES for item in requested):
            parser.error(f"unknown native capability in {value!r}")
    command = args.command[1:] if args.command[:1] == ["--"] else args.command
    try:
        if args.print_exports:
            sys.stdout.write(render(ROOT, caller, docs=args.docs))
            if args.docs:
                print(
                    "pse-env: explicit --docs selects .venv-docs; ordinary product selector is overridden",
                    file=sys.stderr,
                )
            for note in refusals(caller, compose(ROOT, caller), ANNOUNCED):
                print(note, file=sys.stderr)
            return 0
        if args.explain:
            description = operation_description(
                ROOT,
                caller,
                requested,
                command=command,
                docs=args.docs,
                resource_class=args.resource_class,
                recipe_default_class=args.recipe_default_class,
            )
            sys.stdout.write(
                json.dumps(description, indent=2) + "\n"
                if args.json
                else explain(description)
            )
            return 0
        if not command:
            parser.error("give a command after --, or use --print/--explain")
        env = compose(ROOT, caller)
        if args.docs:
            prior = env.get("UV_PROJECT_ENVIRONMENT")
            env = documentation_environment(ROOT, env)
            if prior != env["UV_PROJECT_ENVIRONMENT"]:
                print(
                    f"pse-env: refused UV_PROJECT_ENVIRONMENT={prior}: explicit --docs; using {env['UV_PROJECT_ENVIRONMENT']}",
                    file=sys.stderr,
                )
            provisioning = command in (
                [
                    "uv",
                    "sync",
                    "--locked",
                    "--only-group",
                    "docs",
                    "--no-install-project",
                ],
                [
                    "uv",
                    "sync",
                    "--project",
                    str(ROOT),
                    "--locked",
                    "--only-group",
                    "docs",
                    "--no-install-project",
                ],
            )
            if not provisioning and not (venv(ROOT, env) / "bin/python").is_file():
                raise BoundaryError(  # noqa: TRY301 -- boundary reports selection failure before admission or command effects
                    "documentation interpreter missing; run just bootstrap-docs"
                )
        if args.resource_class:
            env["PSE_RESOURCE_CLASS"] = args.resource_class
        elif host.MARKER not in env:
            env["PSE_RESOURCE_CLASS"] = selected_resource_class(
                command,
                env,
                native=requested is not None,
                default=args.recipe_default_class,
            )
        if args.bind_allocation:
            allocation = required_allocation(env, handoff=True)
            allocation.bind(args.bind_allocation)
            # User managers can enforce memory/CPU quota without a delegated
            # cpuset controller. Bind inherited scheduler affinity on the child.
            os.sched_setaffinity(0, host.cpu_set(allocation.profile.cores))
        for note in refusals(caller, env, ANNOUNCED):
            print(note, file=sys.stderr)
        if args.store:
            env["PSE_REQUIRE_STORE"] = "1"
        if env.get("PSE_REQUIRE_STORE"):
            require_store(env)
            if host.inherit(env) is not None:
                env.pop("PSE_REQUIRE_STORE", None)
        status = unexecutable(command, env)
        if status is not None:
            return status
        require_enclosing_owner(env, no_scope=args.no_scope)
        if host.inherit(env) is not None:
            placement(env, native=requested is not None)
        if requested is not None:
            return native(ROOT, requested, command, env, scope=not args.no_scope)
        prefix = [] if args.no_scope else placement(env, native=False)
        if not prefix:
            return execute(command, env)
        return owned_scope([*prefix, *command], env)
    except BoundaryError as error:
        print(f"pse-env: {error}", file=sys.stderr)
        return FAILURE
    except Exception as error:
        print(f"pse-env: {type(error).__name__}: {error}", file=sys.stderr)
        return FAILURE


if __name__ == "__main__":
    raise SystemExit(main())
