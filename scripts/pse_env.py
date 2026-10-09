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
import functools
import json
import os
import re
import shlex
import shutil
import subprocess
import sys
import uuid
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence

ROOT = Path(__file__).resolve().parents[1]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))
from scripts import (  # noqa: E402 -- direct-script path routing
    build_environment,
    test_resources,
)
from scripts import host_admission as host  # noqa: E402 -- placement owner
from scripts import native_operation as operation  # noqa: E402 -- same routing

FAILURE = 125
LOCAL = ".envrc.local"
DEFAULT_SLICE = "pse.slice"
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


class BoundaryError(Exception):
    """This boundary, not the command, failed."""


def local_keys(root: Path) -> dict[str, str]:
    """Exported values of ``.envrc.local``, read in a clean shell and never printed."""
    path = root / LOCAL
    if not path.is_file():
        return {}
    clean = {"HOME": os.environ.get("HOME", "/"), "PATH": "/usr/bin:/bin"}
    script = '. "$1" >/dev/null 2>&1; env -0'
    result = subprocess.run(
        ["bash", "-c", script, "pse-env", str(path)],
        check=False,
        capture_output=True,
        env=clean,
        timeout=10,
    )
    if result.returncode:
        raise BoundaryError(f"{LOCAL} failed to load")
    after = dict(
        item.split("=", 1) for item in result.stdout.decode().split("\0") if "=" in item
    )
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


def compose(
    root: Path, caller: Mapping[str, str], local: Mapping[str, str] | None = None
) -> dict[str, str]:
    """The ordinary environment: caller, then local file, then repository defaults."""
    env = dict(caller)
    for name, value in (local_keys(root) if local is None else local).items():
        env.setdefault(name, value)
    if "PSE_PYTHON" not in env:
        version = root / ".python-version"
        env["PSE_PYTHON"] = (
            version.read_text().strip() if version.is_file() else "3.14.7"
        )
    env.setdefault("UV_PROJECT_ENVIRONMENT", ".venv")
    # The supervisor's own default state; recipes that need the server check it (--store).
    state_home = env.get("XDG_STATE_HOME") or str(
        Path(env.get("HOME", str(Path.home()))) / ".local/state"
    )
    env.setdefault(
        "PSE_SURREAL_STATE", str(Path(state_home) / "pse-arrow/surreal-functional-v2")
    )
    env["PATH"] = prepend(str(venv(root, env) / "bin"), env.get("PATH"))
    if (SOLVER_STACK / "bin").is_dir():
        env["PATH"] = prepend(str(SOLVER_STACK / "bin"), env["PATH"])
        env["PKG_CONFIG_PATH"] = prepend(
            str(SOLVER_STACK / "lib/pkgconfig"), env.get("PKG_CONFIG_PATH")
        )
    try:
        return build_environment.configure(root, env)
    except ValueError as error:
        raise BoundaryError(str(error)) from error


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


def render(root: Path, caller: Mapping[str, str], *, complete: bool = False) -> str:
    """Shell text for ``eval``: ordinary exports only, local values deferred.

    PATH is rendered as idempotent prepends, because the shell that evaluates the text
    (direnv, or a Claude Bash command after its shell snapshot) may not share the
    caller's PATH. ``complete`` also exports values equal to the caller's, for a consumer
    whose environment is not the caller's (a SessionStart hook's env file).
    """
    local = local_keys(root)
    env = compose(root, caller, local)
    touched = set(
        compose(root, {"HOME": caller.get("HOME", "/"), "PATH": ""}, local)
    ) - {"HOME"}
    lines = [
        f'case ":$PATH:" in *:{shlex.quote(entry)}:*) ;; *) PATH={shlex.quote(entry)}"${{PATH:+:$PATH}}" ;; esac'
        for entry in reversed(required_path(root, caller, local))
    ]
    lines.append("export PATH")
    for name in sorted(env):
        if name in local or name == "PATH" or name.startswith("PSE_NATIVE_"):
            continue
        if caller.get(name) != env[name] or (complete and name in touched):
            lines.append(f"export {name}={shlex.quote(env[name])}")
    lines.extend(
        f"unset {name}" for name in sorted(caller.keys() - env.keys()) if name != "PATH"
    )
    lines.extend(deferred(root, name) for name in sorted(local) if name not in caller)
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


def slice_group(name: str) -> Path:
    """Systemd nests dashed slice names: pse-x.slice lives inside pse.slice."""
    stem = name.removesuffix(".slice").split("-")
    parts = ["-".join(stem[: index + 1]) + ".slice" for index in range(len(stem))]
    service = f"user.slice/user-{os.getuid()}.slice/user@{os.getuid()}.service"
    return Path("/sys/fs/cgroup", service, *parts)


def limits(name: str | None) -> list[tuple[str, str]]:
    """Effective memory limits from the selected slice up to the user manager."""
    if name is None:
        return []
    group = slice_group(name)
    found = []
    while group != Path("/sys/fs/cgroup") and group.name:
        limit = group / "memory.max"
        if limit.is_file():
            value = limit.read_text().strip()
            if value != "max":
                found.append((group.name, value))
        group = group.parent
    return found


def current_memory_ceiling() -> int | None:
    """Smallest actual cgroup ancestor, including a narrower observer role."""
    group = operation.process_group(os.getpid())
    if not group:
        raise BoundaryError("Cannot observe actual enclosing memory placement")
    root = Path("/sys/fs/cgroup")
    current = root / group.lstrip("/")
    ceilings = []
    while current.is_relative_to(root):
        path = current / "memory.max"
        if path.is_file():
            value = path.read_text().strip()
            if value != "max":
                ceilings.append(int(value))
        if current == root:
            break
        current = current.parent
    return min(ceilings) if ceilings else None


def require_store(env: Mapping[str, str]) -> None:
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

    config = json.loads(config_path.read_text())
    owner = host.inherit(env)
    if owner is None:
        if config.get("admission") not in {"open", "quiesced"} or (
            config.get("admission") == "quiesced" and not config.get("parked")
        ):
            raise BoundaryError("Selected canonical service is not admitted")
        return  # Demand readiness inside the registered actual allocation.
    if config.get("parked") and owner.profile.exclusive:
        surreal_server.unpark_service(state)
        config = surreal_server.config_for(state)
    if config.get("admission") == "open" and not surreal_server.ready(state, config):
        surreal_server.start(state, config)
    if config.get("admission") != "open" or not surreal_server.ready(state, config):
        raise BoundaryError(
            f"canonical server for {state} is not ready; run: just surreal start --state {state}"
        )


SLICE_VALUE = re.compile(r"[A-Za-z0-9_:.-]+\.slice")


def manager_environment() -> dict[str, str]:
    runtime = Path(f"/run/user/{os.getuid()}")
    return {
        "XDG_RUNTIME_DIR": os.environ.get("XDG_RUNTIME_DIR", str(runtime)),
        "DBUS_SESSION_BUS_ADDRESS": os.environ.get(
            "DBUS_SESSION_BUS_ADDRESS", f"unix:path={runtime}/bus"
        ),
    }


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


def placement(env: dict[str, str], *, native: bool) -> list[str]:
    """systemd-run arguments that give the command its own scope, or nothing."""
    allocation = host.inherit(env)
    if allocation is not None:
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
    if operation.scope_owner() is not None:
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
    allocation = host.acquire(profile, directory=host.root_path(env))
    env.update(allocation.environment())
    try:
        host.enforce_parent(profile, env)
        host.enforce_allocation(allocation, env)
    except Exception:
        allocation.release()
        raise
    selected = host.allocation_slice(allocation)
    cap = str(profile.memory)
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
        f"CPUQuota={len(profile.cores) * 100}%",
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
        try:
            status = subprocess.call(
                [
                    *prefix,
                    setup_python(root, env),
                    str(root / "scripts/pse_env.py"),
                    "--no-scope",
                    "--native=" + ",".join(requested),
                    "--",
                    *command,
                ],
                env=env,
            )
            return 128 - status if status < 0 else status
        finally:
            allocation = host.inherit(env)
            if allocation is not None:
                allocation.release()
            for error in test_resources.reclaim_reports():
                print(
                    f"test-resources: report remains pinned after cleanup error: {error}",
                    file=sys.stderr,
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


def describe(
    name: str, env: Mapping[str, str], caller: Mapping[str, str], local: set[str]
) -> str:
    if name in local:
        return (
            "<set: .envrc.local>"
            if name not in caller
            else "<set: caller, overrides .envrc.local>"
        )
    value = env.get(name)
    if value is None:
        return "<unset>"
    return value if caller.get(name) != value else f"{value} (caller)"


def native_choices(
    root: Path, chosen: Mapping[str, str], requested: list[str]
) -> list[str]:
    """Thread budgets and solver runtime keys a native command would get, and refusals."""
    from scripts import native_cache as cache  # noqa: PLC0415 -- owner cycle

    runtime = (
        cache.runtime_env(root / "docker/solvers/Dockerfile")
        if {"solver", "uno", "petsc"} & set(requested)
        else {}
    )
    off = {name for name in chosen.get(operation.OFF_MARKER, "").split(",") if name}
    lines = []
    for name in operation.OVERRIDABLE:
        value = chosen.get(name)
        default = "1" if name in operation.THREAD_VARIABLES else runtime.get(name)
        if value == "off" or (value is None and name in off):
            lines.append(f"  {name:22} off (removed)")
        elif value is not None:
            lines.append(f"  {name:22} {value} (chosen; default {default or 'unset'})")
        elif default is not None:
            lines.append(f"  {name:22} {default} (default)")
    for name, reason in operation.AUTHORITATIVE.items():
        if name in runtime and chosen.get(name) not in (None, "", runtime[name]):
            lines.append(
                f"pse-env: refused {name}={chosen[name]}: {reason}; using {runtime[name]}"
            )
    return lines


def explain(root: Path, caller: Mapping[str, str], requested: list[str] | None) -> str:
    local = local_keys(root)
    env = compose(root, caller, local)
    python = venv(root, env) / "bin/python"
    lines = [
        f"checkout          {root}",
        f"interpreter       {python}{'' if python.exists() else '  (missing: just bootstrap-venv)'}",
        f"rust wrapper      {env.get('RUSTC_WRAPPER') or 'disabled'}",
        f"target directory  {env.get('CARGO_TARGET_DIR', str(root / 'target'))}",
        f"sccache           {env.get('SCCACHE_DIR', '<default>')}",
    ]
    try:
        from scripts import doctor  # noqa: PLC0415 -- optional observation

        check = doctor.check_extension()
        lines.append(f"extension         {check.detail} (observed, not admitted)")
    except Exception as error:
        lines.append(f"extension         unobserved: {error}")
    if requested is None:
        lines.append("native            none requested (use --native[=caps])")
    else:
        lines.append(
            "native            "
            + (",".join(requested) or "operation without capabilities")
            + " (prepared lazily when a command runs)"
        )
        lines.extend(native_choices(root, {**local, **caller}, requested))
    owner = operation.scope_owner()
    selected = slice_name(env)
    if owner is not None:
        lines.append(f"placement         inside {owner['unit']} (kept)")
    elif not manager_available():
        lines.append("placement         refused (systemd user manager required)")
    else:
        lines.append(
            f"placement         new scope in {selected or 'the caller slice'};"
            f" PSE_MEMORY_MAX={env.get('PSE_MEMORY_MAX', 'declared capacity profile')}"
        )
    try:
        profile = host.select(
            env.get("PSE_RESOURCE_CLASS", "functional"), env.get("PSE_MEMORY_MAX")
        )
        lines.append(
            f"capacity class    {profile.name}; {profile.memory // host.GIB} GiB; physical cores {','.join(map(str, profile.cores))}; aggregate {host.aggregate(profile) // host.GIB} GiB"
        )
        with host.allocation_metadata(host.root_path(env)) as ledger:
            owners = list(ledger["owners"].values())
        lines.append(
            "current owners    "
            + (
                ", ".join(
                    f"{item['class']} pid={item['pid']} units={len(item['units'])}"
                    for item in owners
                )
                or "none"
            )
        )
        lines.append(
            f"host pressure     MemAvailable={host.memory_info()['MemAvailable'] // host.GIB} GiB; startup guard={host.policy()['pressure_guard_gib']} GiB (ceilings are not reservations)"
        )
    except (ValueError, OSError) as error:
        lines.append(f"capacity          refused: {error}")
    bounds = limits(selected)
    lines.append(
        "aggregate limits  "
        + (", ".join(f"{group} MemoryMax={value}" for group, value in bounds) or "none")
    )
    lines.append(
        "local values      "
        + (
            ", ".join(
                f"{name}={describe(name, env, caller, set(local))}"
                for name in sorted(local)
            )
            or "none"
        )
    )
    lines.extend(refusals({**local, **caller}, env))
    return "\n".join(lines) + "\n"


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
    parser.add_argument("--native", nargs="?", const="__default__", default=None)
    parser.add_argument("--no-scope", action="store_true")
    parser.add_argument(
        "--resource-class",
        choices=(
            "light",
            "compile",
            "functional",
            "wide",
            "timing",
            "reference",
            "exclusive",
        ),
    )
    parser.add_argument("--bind-allocation", help=argparse.SUPPRESS)
    parser.add_argument(
        "--store",
        action="store_true",
        help="require a serving canonical server at $PSE_SURREAL_STATE first",
    )
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args(argv)
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
            sys.stdout.write(render(ROOT, caller))
            for note in refusals(caller, compose(ROOT, caller), ANNOUNCED):
                print(note, file=sys.stderr)
            return 0
        if args.explain:
            sys.stdout.write(explain(ROOT, caller, requested))
            return 0
        if not command:
            parser.error("give a command after --, or use --print/--explain")
        env = compose(ROOT, caller)
        if args.resource_class:
            env["PSE_RESOURCE_CLASS"] = args.resource_class
        elif host.MARKER not in env:
            env.setdefault(
                "PSE_RESOURCE_CLASS", host.classify(command, requested is not None)
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
        try:
            status = subprocess.call([*prefix, *command], env=env)
            return 128 - status if status < 0 else status
        finally:
            allocation = host.inherit(env)
            if allocation is not None and not allocation.release():
                print(
                    f"pse-env: allocation retained until actual drain: {env[host.MARKER]}",
                    file=sys.stderr,
                )
            for error in test_resources.reclaim_reports():
                print(
                    f"test-resources: report remains pinned after cleanup error: {error}",
                    file=sys.stderr,
                )
    except BoundaryError as error:
        print(f"pse-env: {error}", file=sys.stderr)
        return FAILURE
    except Exception as error:
        print(f"pse-env: {type(error).__name__}: {error}", file=sys.stderr)
        return FAILURE


if __name__ == "__main__":
    raise SystemExit(main())
