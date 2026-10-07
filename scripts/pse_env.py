# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""One environment and placement boundary for recipes, bare commands and agent runtimes.

``scripts/pse-env [--native[=caps]] [--no-scope] -- <command>`` runs a command with this
checkout's environment: the caller's values, then ``.envrc.local`` for anything the caller
has not set, then repository defaults (the venv, the local solver stack), then the
compiler/cache configuration of ``build_environment.configure``. Native capabilities are
admitted only on request, under ``native_operation``'s operation owner. Unless asked not
to, the command runs in its own systemd scope inside ``$PSE_SLICE`` (default
``pse.slice``) with ``$PSE_MEMORY_MAX`` as its failure boundary.

``--print`` emits shell exports of the ordinary environment (never native setup, never a
local secret's value), for ``.envrc`` and a Claude ``SessionStart`` hook. ``--explain``
reports what a command would get and why, without preparing anything.

Exit status: the command's own; 125 when this boundary fails (with a ``pse-env:`` line);
126/127 when the command cannot be executed.
"""

from __future__ import annotations

import argparse
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
from scripts import build_environment  # noqa: E402 -- direct-script path routing
from scripts import native_operation as operation  # noqa: E402 -- same routing

FAILURE = 125
LOCAL = ".envrc.local"
DEFAULT_SLICE = "pse.slice"
DEFAULT_MEMORY_MAX = "120G"
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


class Failure(Exception):
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
        raise Failure(f"{LOCAL} failed to load")
    after = dict(
        item.split("=", 1) for item in result.stdout.decode().split("\0") if "=" in item
    )
    return {
        name: value
        for name, value in after.items()
        if clean.get(name) != value and name not in {"_", "SHLVL", "PWD", "OLDPWD"}
    }


def prepend(path: str, existing: str | None) -> str:
    parts = [part for part in (existing or "").split(os.pathsep) if part and part != path]
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
        env["PSE_PYTHON"] = version.read_text().strip() if version.is_file() else "3.14.7"
    env.setdefault("UV_PROJECT_ENVIRONMENT", ".venv")
    env["PATH"] = prepend(str(venv(root, env) / "bin"), env.get("PATH"))
    if (SOLVER_STACK / "bin").is_dir():
        env["PATH"] = prepend(str(SOLVER_STACK / "bin"), env["PATH"])
        env.setdefault("IPOPT_DIR", str(SOLVER_STACK))
        env["PKG_CONFIG_PATH"] = prepend(
            str(SOLVER_STACK / "lib/pkgconfig"), env.get("PKG_CONFIG_PATH")
        )
    try:
        return build_environment.configure(root, env)
    except ValueError as error:
        raise Failure(str(error)) from error


# Values an agent sets deliberately; the others usually come from a login profile, so
# their replacement is reported by --explain rather than on every command.
ANNOUNCED = ("CARGO_TARGET_DIR", "RUSTC_WRAPPER")


def refusals(
    caller: Mapping[str, str], env: Mapping[str, str], names: Sequence[str] = tuple(CONFIGURED)
) -> list[str]:
    notes = []
    for name in names:
        reason = CONFIGURED[name]
        prior = caller.get(name)
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


def required_path(root: Path, caller: Mapping[str, str]) -> list[str]:
    """PATH entries composition puts first, independent of the caller's own PATH."""
    bare = compose(root, {"HOME": caller.get("HOME", "/"), "PATH": ""}, {})
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
    touched = set(compose(root, {"HOME": caller.get("HOME", "/"), "PATH": ""}, {})) - {"HOME"}
    lines = [
        f'case ":$PATH:" in *:{shlex.quote(entry)}:*) ;; *) PATH={shlex.quote(entry)}"${{PATH:+:$PATH}}" ;; esac'
        for entry in reversed(required_path(root, caller))
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
    return None if name in {"", "none"} else name


def slice_group(name: str) -> Path:
    """systemd nests dashed slice names: pse-x.slice lives inside pse.slice."""
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


def manager_available() -> bool:
    bus = Path(f"/run/user/{os.getuid()}/bus")
    return bus.is_socket() and shutil.which("systemd-run") is not None


def placement(env: dict[str, str], *, native: bool) -> list[str]:
    """systemd-run arguments that give the command its own scope, or nothing."""
    if operation.scope_owner() is not None:
        # Never leave a scope that protects native generations or a managed worker.
        return []
    if not manager_available():
        return []
    runtime = Path(f"/run/user/{os.getuid()}")
    env.setdefault("XDG_RUNTIME_DIR", str(runtime))
    env.setdefault("DBUS_SESSION_BUS_ADDRESS", f"unix:path={runtime}/bus")
    kind = "native" if native else "cmd"
    command = [
        "systemd-run",
        "--user",
        "--scope",
        "--quiet",
        "--collect",
        f"--unit=pse-{kind}-{uuid.uuid4().hex}.scope",
    ]
    selected = slice_name(env)
    if selected is not None:
        command.append(f"--slice={selected}")
    cap = env.get("PSE_MEMORY_MAX", DEFAULT_MEMORY_MAX)
    if cap != "off":
        command += ["-p", f"MemoryMax={cap}", "-p", "MemorySwapMax=0"]
        requested = parse_bytes(cap)
        for group, value in limits(selected):
            if requested is not None and int(value) < requested:
                print(
                    f"pse-env: PSE_MEMORY_MAX={cap} is bounded by {group} MemoryMax={value}",
                    file=sys.stderr,
                )
                break
    return [*command, "--"]


def setup_python(root: Path, env: Mapping[str, str]) -> str:
    chosen = env.get("PSE_NATIVE_SETUP_PYTHON")
    if chosen:
        return chosen
    candidate = venv(root, env) / "bin/python"
    return str(candidate) if candidate.exists() else sys.executable


def execute(command: Sequence[str], env: Mapping[str, str]) -> int:
    try:
        os.execvpe(command[0], list(command), dict(env))
    except FileNotFoundError:
        print(f"pse-env: command not found: {command[0]}", file=sys.stderr)
        return 127
    except PermissionError:
        print(f"pse-env: command not executable: {command[0]}", file=sys.stderr)
        return 126


def native(
    root: Path, requested: list[str], command: list[str], env: dict[str, str], *, scope: bool
) -> int:
    """Native capabilities are admitted only under an operation owner."""
    if operation.owner_record() is not None:
        # Nested: the outer operation keeps ownership of setup and cancellation.
        return execute(command, operation.environment(requested, env))
    prefix = placement(env, native=True) if scope else []
    if prefix:
        return execute(
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
        return operation.run(command, operation.environment(requested, dict(os.environ)))


def describe(name: str, env: Mapping[str, str], caller: Mapping[str, str], local: set[str]) -> str:
    if name in local:
        return "<set: .envrc.local>" if name not in caller else "<set: caller, overrides .envrc.local>"
    value = env.get(name)
    if value is None:
        return "<unset>"
    return value if caller.get(name) != value else f"{value} (caller)"


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
    except Exception as error:  # noqa: BLE001 -- an observation, never a failure
        lines.append(f"extension         unobserved: {error}")
    if requested is None:
        lines.append("native            none requested (use --native[=caps])")
    else:
        lines.append(
            "native            "
            + (",".join(requested) or "operation without capabilities")
            + " (prepared lazily when a command runs)"
        )
    owner = operation.scope_owner()
    selected = slice_name(env)
    if owner is not None:
        lines.append(f"placement         inside {owner['unit']} (kept)")
    elif not manager_available():
        lines.append("placement         in place (no systemd user manager)")
    else:
        lines.append(
            f"placement         new scope in {selected or 'the caller slice'};"
            f" PSE_MEMORY_MAX={env.get('PSE_MEMORY_MAX', DEFAULT_MEMORY_MAX)}"
        )
        bounds = limits(selected)
        lines.append(
            "aggregate limits  "
            + (", ".join(f"{group} MemoryMax={value}" for group, value in bounds) or "none")
        )
    lines.append("local values      " + (", ".join(
        f"{name}={describe(name, env, caller, set(local))}" for name in sorted(local)
    ) or "none"))
    lines.extend(refusals(caller, env))
    return "\n".join(lines) + "\n"


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="pse-env", description=__doc__)
    parser.add_argument("--print", action="store_true", dest="print_exports")
    parser.add_argument("--explain", action="store_true")
    parser.add_argument("--native", nargs="?", const="__default__", default=None)
    parser.add_argument("--no-scope", action="store_true")
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
            return 0
        if args.explain:
            sys.stdout.write(explain(ROOT, caller, requested))
            return 0
        if not command:
            parser.error("give a command after --, or use --print/--explain")
        env = compose(ROOT, caller)
        for note in refusals(caller, env, ANNOUNCED):
            print(note, file=sys.stderr)
        if requested is not None:
            return native(ROOT, requested, command, env, scope=not args.no_scope)
        prefix = [] if args.no_scope else placement(env, native=False)
        return execute([*prefix, *command], env)
    except Failure as error:
        print(f"pse-env: {error}", file=sys.stderr)
        return FAILURE
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        print(f"pse-env: {error}", file=sys.stderr)
        return FAILURE


if __name__ == "__main__":
    raise SystemExit(main())
