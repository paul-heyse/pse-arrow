# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Closed, versioned context for the custom server installer, not a Cargo interpreter.

Cargo still merges and interprets configuration. We retain every parsed declaration,
its ordering and its definition root; only syntax equivalence is admitted. The child
receives a controlled environment. Unsupported external-input mechanisms fail closed.
"""

from __future__ import annotations

import datetime
import hashlib
import json
import os
import platform
import re
import shlex
import shutil
import subprocess
import tomllib
from pathlib import Path

SCHEMA = "server-build-context-v1"
# These are build inputs or compiler-cache controls. All other caller variables are
# removed from the compiler child, rather than guessed irrelevant for build scripts.
ENVIRONMENT = frozenset("""
HOME PATH USER LOGNAME TMPDIR LANG LC_ALL TZ CARGO_HOME RUSTUP_HOME
CARGO_BUILD_JOBS CARGO_BUILD_TARGET CARGO_INCREMENTAL CARGO_TERM_COLOR
RUSTC RUSTDOC RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER RUSTFLAGS RUSTDOCFLAGS
CARGO_ENCODED_RUSTFLAGS CARGO_ENCODED_RUSTDOCFLAGS
CC CXX FC AR RANLIB CFLAGS CXXFLAGS FFLAGS CPPFLAGS LDFLAGS
CMAKE_PREFIX_PATH CMAKE_TOOLCHAIN_FILE CMAKE_GENERATOR SDKROOT
MACOSX_DEPLOYMENT_TARGET LIBCLANG_PATH CLANG_PATH LLVM_CONFIG_PATH LLVM_DIR
LIBRARY_PATH LD_LIBRARY_PATH PKG_CONFIG_PATH PKG_CONFIG_LIBDIR PKG_CONFIG_SYSROOT_DIR
BINDGEN_EXTRA_CLANG_ARGS PSE_LLVM_PREFIX PSE_SCCACHE_BINARY
SCCACHE_DIR SCCACHE_CACHE_SIZE SCCACHE_CONF SCCACHE_SERVER_UDS
SCCACHE_CLIENT_SIDE SCCACHE_DIRECT SCCACHE_NO_DAEMON
PSE_NATIVE_COMPILER_CACHE SOURCE_DATE_EPOCH
""".split())
# Operational handles are consumed by the existing cache/admission owners. They
# are never identity inputs and are not selectable compiler/native controls.
OPERATIONAL = frozenset("""
PSE_HOST_ALLOCATION PSE_ADMISSION_DEADLINE PSE_RESOURCE_CLASS PSE_BUILD_MODE
PSE_NATIVE_OPERATION PSE_NATIVE_HANDOFF PSE_CARGO_DEFAULT_JOBS
XDG_RUNTIME_DIR DBUS_SESSION_BUS_ADDRESS
""".split())
NATIVE_PATHS = frozenset("""
CMAKE_PREFIX_PATH CMAKE_TOOLCHAIN_FILE SDKROOT LIBCLANG_PATH LLVM_DIR
LIBRARY_PATH LD_LIBRARY_PATH PKG_CONFIG_PATH PKG_CONFIG_LIBDIR PKG_CONFIG_SYSROOT_DIR
PSE_LLVM_PREFIX SCCACHE_CONF
""".split())
CONFIG_KEYS = {
    "build": {"jobs", "rustc", "rustdoc", "rustc-wrapper", "rustc-workspace-wrapper", "rustflags", "rustdocflags", "incremental"},
    "unstable": {"feature-unification", "fine-grain-locking"},
    "resolver": {"feature-unification"},
    "future-incompat-report": {"frequency"},
}
_CACHE: dict[tuple, str] = {}


class ContextError(ValueError):
    """An external build input cannot be closed by this installer's contract."""


def sha(path: Path) -> str:
    stat = path.stat()
    key = (str(path.resolve()), stat.st_dev, stat.st_ino, stat.st_size, stat.st_mtime_ns, stat.st_ctime_ns)
    if key not in _CACHE:
        with path.open("rb") as stream:
            _CACHE[key] = hashlib.file_digest(stream, "sha256").hexdigest()
    return _CACHE[key]


def canonical(value: object) -> object:
    """Typed TOML tree: maps sort, array order and all scalar types survive."""
    if isinstance(value, dict):
        return ["table", [[key, canonical(value[key])] for key in sorted(value)]]
    if isinstance(value, list):
        return ["array", [canonical(item) for item in value]]
    if isinstance(value, (datetime.datetime, datetime.date, datetime.time)):
        return [type(value).__name__, value.isoformat()]
    if isinstance(value, float):
        return ["float", value.hex()]
    if type(value) in {bool, int, str}:
        return [type(value).__name__, value]
    raise ContextError(f"Unsupported TOML value type: {type(value).__name__}")


def identity(value: object) -> str:
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def controlled_environment(root: Path, caller: dict[str, str] | None = None) -> dict[str, str]:
    from scripts import pse_env  # noqa: PLC0415 -- environment remains owned by pse_env

    composed = pse_env.compose(root, dict(os.environ) if caller is None else caller)
    if composed.get("CARGO_TARGET_DIR") or composed.get("PSE_CARGO_TARGET_DIR"):
        raise ContextError("Server installer requires its own default source/target directory")
    # Rustup resolves rust-toolchain.toml in the fixed invocation cwd. A caller's
    # explicit toolchain selection is not a second supported authority.
    if composed.get("RUSTUP_TOOLCHAIN"):
        active = subprocess.check_output(["rustup", "show", "active-toolchain"], cwd=root, env=composed, text=True).split()[0]
        declared = tomllib.loads((root / "rust-toolchain.toml").read_text())["toolchain"]["channel"]
        if not active.startswith(declared + "-"):
            raise ContextError("Server installer refuses an inherited different toolchain")
    unknown = [key for key in composed if key.startswith(("CARGO_", "RUSTC_")) and key not in ENVIRONMENT and not key.startswith(("CARGO_PKG_", "CARGO_MANIFEST_"))]
    if unknown:
        raise ContextError("Unsupported Cargo environment controls: " + ", ".join(sorted(unknown)))
    return {key: value for key, value in composed.items() if key in ENVIRONMENT or key in OPERATIONAL}


def configuration(root: Path, env: dict[str, str]) -> tuple[list[dict], list[dict], list[tuple[str, Path]]]:
    """Retain Cargo's ordered discovery, including legacy-config precedence."""
    home = Path(env.get("CARGO_HOME", str(Path(env["HOME"]) / ".cargo"))).resolve()
    directories = [home, *(parent / ".cargo" for parent in reversed((root, *root.parents)))]
    records, raw, executables = [], [], []
    seen = set()
    for directory in directories:
        directory = directory.resolve()
        if directory in seen:
            continue
        seen.add(directory)
        selected = directory / "config"
        if not selected.is_file():
            selected = directory / "config.toml"
        if not selected.is_file():
            # Absence is retained: adding a parent config changes the key.
            records.append({"directory": str(directory), "values": None})
            continue
        document = tomllib.loads(selected.read_text())
        for table, values in document.items():
            if table in {"alias"}:
                continue  # built-in `build` cannot be replaced by a Cargo alias
            if table == "target":
                for name, settings in values.items():
                    if set(settings) - {"linker", "rustflags", "rustdocflags"}:
                        raise ContextError(f"Unsupported Cargo target configuration in {selected}")
                    if "linker" in settings:
                        executables.append((settings["linker"], directory.parent))
            elif table not in CONFIG_KEYS or not isinstance(values, dict) or set(values) - CONFIG_KEYS[table]:
                raise ContextError(f"Unsupported Cargo configuration {table} in {selected}")
        build = document.get("build", {})
        for name in ("rustc", "rustdoc", "rustc-wrapper", "rustc-workspace-wrapper"):
            if build.get(name):
                executables.append((build[name], directory.parent))
        records.append({"directory": str(directory), "path": str(selected), "definition_root": str(directory.parent), "values": canonical(document)})
        raw.append({"path": str(selected), "sha256": sha(selected)})
    return records, raw, executables


def tool(command: str, root: Path, env: dict[str, str]) -> dict:
    path = Path(command)
    if "/" not in command:
        found = shutil.which(command, path=env.get("PATH"))
        if not found:
            raise ContextError(f"Build tool unavailable: {command}")
        path = Path(found)
    elif not path.is_absolute():
        path = root / path
    resolved = path.resolve(strict=True)
    return {"command": command, "resolved": str(resolved), "sha256": sha(resolved)}


def tree(path: Path) -> dict:
    """Hash selected native bytes as well as path/symlink and absence premises."""
    if not path.exists():
        return {"path": str(path), "absent": True}
    resolved = path.resolve(strict=True)
    if resolved.is_file():
        return {"path": str(path), "resolved": str(resolved), "sha256": sha(resolved)}
    entries = []
    visited = set()

    def visit(directory: Path, relative: str) -> None:
        target = directory.resolve(strict=True)
        if target in visited:
            return
        visited.add(target)
        for candidate in sorted(target.iterdir()):
            name = relative + candidate.name
            if candidate.is_file():
                entries.append([name, str(candidate.resolve()), sha(candidate)])
            elif candidate.is_dir():
                entries.append([name, str(candidate.resolve()), "directory"])
                visit(candidate, name + "/")
            elif candidate.is_symlink():
                entries.append([name, os.readlink(candidate), "missing-link-target"])
    visit(resolved, "")
    return {"path": str(path), "resolved": str(resolved), "files": len(entries), "sha256": identity(entries)}


def capture(root: Path, env: dict[str, str]) -> tuple[dict, dict]:
    if platform.system() != "Linux":
        raise ContextError("Safe installation equivalence is qualified only for Linux")
    configs, raw, configured_tools = configuration(root, env)
    tools = {}
    toolchain = {}
    for name in ("cargo", "rustc", "rustdoc"):
        executable = subprocess.check_output(["rustup", "which", name], cwd=root, env=env, text=True).strip()
        tools[name] = tool(executable, root, env)
        toolchain[name] = subprocess.check_output([executable, "--version", "--verbose"], cwd=root, env=env, text=True).strip()
    sysroot = Path(subprocess.check_output([tools["rustc"]["resolved"], "--print", "sysroot"], cwd=root, env=env, text=True).strip())
    # Unknown Cargo environment controls never reach the child. Known native
    # override files/roots and system headers/libs are bound by actual bytes.
    for key, default in (("CC", "cc"), ("CXX", "c++"), ("AR", "ar"), ("RANLIB", "ranlib"), ("CLANG_PATH", "clang"), ("LLVM_CONFIG_PATH", "llvm-config")):
        tools[key] = tool(env.get(key, default), root, env)
    for index, (command, base) in enumerate(configured_tools):
        tools[f"config-{index}"] = tool(command, base, env)
    for key in ("RUSTC", "RUSTDOC", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "PSE_SCCACHE_BINARY", "PSE_NATIVE_COMPILER_CACHE", "FC"):
        if env.get(key):
            tools[key] = tool(env[key], root, env)
    if env.get("CMAKE_TOOLCHAIN_FILE"):
        raise ContextError("External CMake toolchain programs require explicit installer support")
    for key in ("CFLAGS", "CXXFLAGS", "FFLAGS", "CPPFLAGS", "LDFLAGS", "BINDGEN_EXTRA_CLANG_ARGS"):
        if env.get(key):
            raise ContextError(f"External native flags {key} require explicit installer support")
    # The only qualified rustflag external mechanism is the selected linker
    # backend. Other flags that can open files/directories must not alias.
    flag_groups = []
    for entry in configs:
        if entry.get("path"):
            document = tomllib.loads(Path(entry["path"]).read_text())
            flag_groups.extend(settings[key] for settings in [document.get("build", {}), *document.get("target", {}).values()] for key in ("rustflags", "rustdocflags") if key in settings)
    flag_groups.extend(env[key].split("\x1f") if key.startswith("CARGO_ENCODED_") else shlex.split(env[key]) for key in ("RUSTFLAGS", "RUSTDOCFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_ENCODED_RUSTDOCFLAGS") if env.get(key))
    for group in flag_groups:
        arguments = shlex.split(group) if isinstance(group, str) else group
        index = 0
        while index < len(arguments):
            argument = arguments[index]
            if argument in {"-C", "--cfg", "--check-cfg", "-Z"}:
                index += 1
                if index >= len(arguments):
                    raise ContextError("Incomplete compiler flag")
                value = arguments[index]
                if argument == "-Z" and not value.startswith("threads="):
                    raise ContextError("Unsupported unstable compiler flag input")
                if argument == "-C" and value.startswith("link-arg=-fuse-ld=") and value.rsplit("=", 1)[-1] not in {"mold", "bfd", "lld"}:
                    raise ContextError("Unsupported linker backend")
                if argument == "-C" and not value.startswith(("link-arg=-fuse-ld=", "opt-level=", "debuginfo=", "codegen-units=", "panic=", "lto=", "target-feature=")):
                    raise ContextError("Unsupported compiler codegen input")
            elif not argument.startswith(("-Copt-level=", "-Cdebuginfo=", "--cfg=", "--check-cfg=", "-Zthreads=")):
                raise ContextError("Unsupported compiler flag input")
            index += 1
    # Linker selection by -fuse-ld uses PATH, so capture both the driver and
    # supported backend independently. Arbitrary response/plugin files refuse.
    flags = " ".join(str(value) for value in [configs, *[env.get(key, "") for key in ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CFLAGS", "CXXFLAGS", "LDFLAGS", "BINDGEN_EXTRA_CLANG_ARGS")]])
    if "@" in flags or "linker-plugin-lto" in flags or "--sysroot" in flags or "-fplugin" in flags:
        raise ContextError("External response/plugin/sysroot flags are unsupported")
    for backend in ("mold", "ld", "ld.lld", "pkg-config", "cmake", "make", "ninja", "git", "sh", "bash", "env", "perl", "python3", "clang++"):
        if shutil.which(backend, path=env.get("PATH")):
            tools[backend] = tool(backend, root, env)
    # Bind actual dynamic-loader resolution as well as executable bytes. ASLR
    # addresses are excluded; unresolved libraries fail the supported context.
    for selected in tools.values():
        linked = subprocess.run(["ldd", selected["resolved"]], cwd=root, env=env, text=True, capture_output=True, check=False)
        if "not found" in linked.stdout:
            raise ContextError("Unresolved build-tool shared library")
        if linked.returncode not in {0, 1}:
            raise ContextError("Cannot inspect build-tool dynamic dependencies")
        selected["linked_files"] = {str(path.resolve()): sha(path.resolve()) for name in re.findall(r"(?<!\S)(/[^\s]+)", linked.stdout) if (path := Path(name)).is_file()}
    roots = {sysroot, Path("/etc/ld.so.cache"), Path("/etc/ld.so.conf"), Path("/etc/ld.so.conf.d"), Path("/usr/include"), Path("/usr/local/include"), Path("/usr/local/lib"), Path("/usr/lib") / {"x86_64": "x86_64-linux-gnu", "aarch64": "aarch64-linux-gnu"}[platform.machine()], Path("/usr/lib/gcc"), Path("/usr/lib64")}
    for key in NATIVE_PATHS:
        for value in env.get(key, "").split(os.pathsep):
            if value:
                path = Path(value)
                roots.add(path if path.is_absolute() else root / path)
    # A managed cache wrapper executes these owners; changed behavior invalidates
    # its build identity even though the wrapper path is unchanged.
    owners = {name: sha(Path(__file__).resolve().parent / name) for name in ("compiler_cache.py", "build_environment.py", "pse_env.py")}
    context = {"schema": SCHEMA, "cwd": str(root.resolve()), "configuration": configs, "toolchain_declaration": canonical(tomllib.loads((root / "rust-toolchain.toml").read_text())), "tools": tools, "toolchain": toolchain, "native": [tree(path) for path in sorted(roots)], "environment": {key: hashlib.sha256(value.encode()).hexdigest() for key, value in sorted(env.items()) if key in ENVIRONMENT}, "owners": owners}
    return context, {"cargo_configuration": raw, "toolchain_sha256": sha(root / "rust-toolchain.toml")}
