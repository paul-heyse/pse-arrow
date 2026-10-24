# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Characterize pinned Cargo builds in an isolated snapshot; never edit the worktree."""

from __future__ import annotations

import argparse
import atexit
import hashlib
import json
import math
import os
import resource
import shutil
import signal
import statistics
import subprocess
import sys
import tempfile
import threading
import time
import tomllib
from pathlib import Path
from typing import TYPE_CHECKING

from scripts import (
    arrow_validation,
    build_environment,
    host_admission,
    native_operation,
    pse_env,
    surreal_server,
    validation,
)

if TYPE_CHECKING:
    from collections.abc import Callable, Sequence


def snapshot(root: Path, output: Path) -> Path:
    """Copy exactly the source inventory, including dirty/untracked files and modes."""
    inventory = validation.sources(root)
    validation.write_json(output / "original-source.json", inventory)
    destination = output / "source"
    destination.mkdir()
    for name in inventory:
        source = root / name
        target = destination / name
        if not source.exists() and not source.is_symlink():
            continue
        target.parent.mkdir(parents=True, exist_ok=True)
        if source.is_symlink():
            target.symlink_to(source.readlink())
        else:
            shutil.copy2(source, target)
    subprocess.run(
        ["git", "init", "--quiet", "--initial-branch=measurement"],
        cwd=destination,
        check=True,
    )
    subprocess.run(["git", "add", "--force", "--all"], cwd=destination, check=True)
    subprocess.run(
        [
            "git",
            "-c",
            "user.name=Build measurement",
            "-c",
            "user.email=measurement@invalid",
            "-c",
            "core.hooksPath=/dev/null",
            "commit",
            "--quiet",
            "-m",
            "Exact working-source snapshot for build characterization",
        ],
        cwd=destination,
        check=True,
    )
    copied = validation.sources(destination)
    # Deleted tracked paths have no file to copy; all actual bytes and modes must agree.
    retained = {
        name: digest
        for name, digest in inventory.items()
        if (root / name).exists() or (root / name).is_symlink()
    }
    if copied != retained or validation.sources(root) != inventory:
        raise ValueError("source changed during isolated build capture")
    return destination


def artifacts(path: Path) -> dict:
    """Use Cargo's supported JSON artifact protocol, without parsing its HTML UI."""
    units = []
    finished = None
    for line in path.read_text().splitlines():
        message = json.loads(line)
        if message.get("reason") == "compiler-artifact":
            units.append(
                {
                    key: message[key]
                    for key in ("package_id", "target", "profile", "features", "fresh")
                }
            )
        elif message.get("reason") == "build-finished":
            if finished is not None:
                raise ValueError("Cargo reported multiple completion records")
            finished = message.get("success")
    if finished is not True or not units:
        raise ValueError("Cargo did not report a complete successful build")
    return {
        "artifacts": units,
        "rebuilt_artifacts": sum(not unit["fresh"] for unit in units),
        "fresh_artifacts": sum(unit["fresh"] for unit in units),
    }


NATIVE_PACKAGES = (
    "pse-backend-native",
    "pse-compiler",
    "pse-kernels",
    "pse-math",
    "pse-runtime",
    "pse-structural",
    "pse-tests-conformance",
)
OPERATION_OUTPUT = "PSE_BUILD_MEASUREMENT_OUTPUT"
STORE_SELECTION = "PSE_BUILD_MEASUREMENT_STORE_SELECTION"
STORE_LAUNCH = "PSE_BUILD_MEASUREMENT_STORE_LAUNCH"
STORE_PREPARATION_SECONDS = 40


def store_launch_handoff(
    output: Path, environment: dict[str, str], *, deadline: float | None = None
) -> dict:
    """Accept only the child of this live, newly started outer measurement."""
    if deadline is not None:
        surreal_server.remaining(deadline)
    launch = json.loads(environment.get(STORE_LAUNCH, "null"))
    if not isinstance(launch, dict):
        raise ValueError(  # noqa: TRY004 -- malformed handoff is a value refusal
            "store preparation requires a fresh outer launch handoff"
        )
    pid = launch.get("pid")
    started = launch.get("started_monotonic")
    if (
        type(pid) is not int
        or pid <= 1
        or pid == os.getpid()
        or not isinstance(started, (int, float))
        or not math.isfinite(started)
        or launch.get("output") != str(output.resolve())
        or native_operation.start_identity(pid) != launch.get("start")
    ):
        raise ValueError("store preparation outer launch identity differs")
    ancestor = os.getppid()
    while ancestor > 1 and ancestor != pid:
        if deadline is not None:
            surreal_server.remaining(deadline)
        status = Path(f"/proc/{ancestor}/status").read_text().splitlines()
        ancestor = int(
            next(line.split()[1] for line in status if line.startswith("PPid:"))
        )
    if ancestor != pid:
        raise ValueError("store preparation requires its live outer launch ancestor")
    timing = json.loads((output / "native-operation.json").read_text())
    setup = timing.get("setup_started_monotonic")
    if (
        not isinstance(setup, (int, float))
        or not math.isfinite(setup)
        or setup < started
        or setup > time.monotonic()
    ):
        raise ValueError("store preparation timing predates its fresh outer launch")
    if deadline is not None:
        surreal_server.remaining(deadline)
    return timing


def selected_store(
    environment: dict[str, str], *, quiesced: bool, deadline: float | None = None
) -> dict[str, object]:
    """Bind this explicit nonresident selection without granting startup authority."""
    state = Path(environment.get("PSE_SURREAL_STATE", ""))
    worker = Path(environment.get("PSE_WORKER_BINARY", ""))
    if not state.is_absolute() or not worker.is_absolute():
        raise ValueError("store preparation requires explicit state and worker paths")
    config = surreal_server.config_for(state, deadline=deadline)
    if surreal_server.service_directory(state, deadline=deadline) != state:
        raise ValueError("store preparation requires the selected service state")
    if config.get("resident") is not False or (
        quiesced and (config["admission"] != "quiesced" or config["accepting_writes"])
    ):
        raise ValueError("store preparation requires a quiesced nonresident profile")
    if config["resources"] == surreal_server.reference_resources():
        profile = "reference"
    elif config["resources"] == surreal_server.execution_resources("wide"):
        profile = "exclusive"
    else:
        raise ValueError("store preparation supports exact reference or wide profiles")
    memory = host_admission.select(profile).memory
    if (
        environment.get("PSE_RESOURCE_CLASS") != profile
        or host_admission.finite_bytes(environment.get("PSE_MEMORY_MAX", "")) != memory
    ):
        raise ValueError("store preparation requires its matching enclosing allocation")
    receiver = surreal_server.checked_primary(config)
    digest = surreal_server.file_digest(worker)
    if digest != receiver["worker_sha256"]:
        raise ValueError("selected store and worker bytes differ")
    if deadline is not None:
        surreal_server.remaining(deadline)
    return {
        "state": str(state),
        "worker": str(worker),
        "worker_sha256": digest,
        "receiver": receiver,
        "instance_id": config["instance_id"],
        "endpoint": config["endpoint"],
        "resources": config["resources"],
        "profile": profile,
        "memory_bytes": memory,
    }


def prepare_selected_store(output: Path, environment: dict[str, str]) -> dict:
    """Prepare only the selected store inside this measured complete-operation owner."""
    started = time.monotonic()
    deadline = started + STORE_PREPARATION_SECONDS
    expected = json.loads(environment[STORE_SELECTION])
    owner = native_operation.owner_record(deadline=deadline)
    allocation = host_admission.inherit(environment, deadline=deadline)
    if owner is None or allocation is None:
        raise ValueError("store preparation requires its authentic measured owner")
    timing = store_launch_handoff(output, environment, deadline=deadline)
    if (
        timing.get("owner_record_path") != str(owner)
        or timing.get("setup_status") != "passed"
        or timing.get("host_allocation", {}).get("path")
        != str(allocation.directory / allocation.nonce)
    ):
        raise ValueError("store preparation handoff differs from the measured owner")
    selected = selected_store(environment, quiesced=True, deadline=deadline)
    if (
        selected != expected
        or allocation.profile.name != selected["profile"]
        or allocation.profile.memory != selected["memory_bytes"]
    ):
        raise ValueError("store preparation selection changed before admission")
    state = Path(environment["PSE_SURREAL_STATE"])
    with (
        surreal_server.lifecycle_deadline(deadline),
        surreal_server.lifecycle_reservation(
            state, purpose="startup", deadline=deadline
        ),
    ):
        surreal_server.readmit_supervisor(state, deadline=deadline)
        if selected_store(environment, quiesced=True, deadline=deadline) != selected:
            raise ValueError("store preparation selection changed during readmission")
        surreal_server.start(
            state,
            surreal_server.config_for(state, deadline=deadline),
            deadline=deadline,
        )
        pse_env.require_store(environment, deadline=deadline)
        if selected_store(environment, quiesced=False, deadline=deadline) != selected:
            raise ValueError("store preparation selection changed during startup")
        surreal_server.remaining(deadline)
    finished = time.monotonic()
    return {
        "wall_seconds": finished - started,
        "started_monotonic": started,
        "finished_monotonic": finished,
        "status": "passed",
        "deadline": deadline,
        "selected": selected,
        "scope": "authenticated selected nonresident readmission, startup and normal store readiness",
    }


def snapshot_environment(env: dict[str, str]) -> dict[str, str]:
    """Use the snapshot's Cargo default, preserving composed compiler flags."""
    result = env.copy()
    result.pop("CARGO_TARGET_DIR", None)
    result.pop("PSE_CARGO_TARGET_DIR", None)
    return result


def workflow_environment(env: dict[str, str], source: Path) -> dict[str, str]:
    """Keep nested checkout composition and Python installation in the snapshot."""
    result = {
        **snapshot_environment(env),
        "UV_PROJECT_ENVIRONMENT": str(source / ".venv"),
    }
    result.pop("VIRTUAL_ENV", None)
    return result


def directory_identity(path: Path) -> dict[str, int]:
    if path.is_symlink() or not path.is_dir():
        raise ValueError(f"measurement directory is missing or symlinked: {path}")
    status = path.stat()
    return {"device": status.st_dev, "inode": status.st_ino}


def snapshot_target(source: Path, output: Path) -> Path:
    """Select only the default target of this freshly captured source directory."""
    if source != output / "source" or source.resolve() != source:
        raise ValueError("measurement source must be the selected snapshot directory")
    directory_identity(source)
    return source / "target"


def validate_target_path(source: Path, target: Path, env: dict[str, str]) -> None:
    if (
        source.resolve() != source
        or target != source / "target"
        or target.resolve() != target
        or target.is_symlink()
        or any(key in env for key in ("CARGO_TARGET_DIR", "PSE_CARGO_TARGET_DIR"))
    ):
        raise ValueError("measurement requires the snapshot's default Cargo target")
    directory_identity(source)
    if target.exists():
        directory_identity(target)


def validate_target(
    source: Path, target: Path, cargo: list[str], env: dict[str, str]
) -> None:
    """Refuse escaped Cargo selection before compilation or artifact deletion."""
    validate_target_path(source, target, env)
    metadata = json.loads(
        subprocess.check_output(
            [
                *cargo,
                "metadata",
                "--no-deps",
                "--format-version=1",
                "--locked",
                "--offline",
            ],
            cwd=source,
            env=env,
            text=True,
        )
    )
    if Path(metadata["target_directory"]) != target:
        raise ValueError("effective Cargo target escapes the selected snapshot target")
    if metadata.get("build_directory") != str(target):
        raise ValueError(
            "effective Cargo build directory differs from the selected snapshot target"
        )


def recover_target(
    source: Path,
    target: Path,
    cargo: list[str],
    env: dict[str, str],
    *,
    source_identity: dict[str, int],
    target_identity: dict[str, int],
) -> None:
    """Remove only the same snapshot and target observed during its last build."""
    if (
        directory_identity(source) != source_identity
        or directory_identity(target) != target_identity
    ):
        raise ValueError(
            "measurement snapshot or target identity changed before recovery"
        )
    validate_target(source, target, cargo, env)
    if (
        directory_identity(source) != source_identity
        or directory_identity(target) != target_identity
    ):
        raise ValueError(
            "measurement snapshot or target identity changed during recovery"
        )
    shutil.rmtree(target)


def selected_packages(
    packages: list[str] | None, *, native: bool, workload: str
) -> tuple[str, ...]:
    if native:
        if packages or workload != "compiler":
            raise ValueError(
                "--native selects its named native workload; omit --package/--workload"
            )
        return NATIVE_PACKAGES
    if packages:
        if workload != "compiler":
            raise ValueError("select --package or a named --workload")
        return tuple(dict.fromkeys(packages))
    if workload == "compiler-relations":
        return ("pse-compiler", "pse-relations")
    return ("pse-compiler",)


def correctness_command(
    source: Path, cargo: list[str], args: list[str], env: dict[str, str]
) -> list[str]:
    """Resolve this snapshot's exact roots and actual Arrow validation closure."""

    def capture(command: Sequence[str]) -> bytes:
        return subprocess.check_output([*cargo, *command[1:]], cwd=source, env=env)

    resolved = arrow_validation.resolve(["cargo", *args], run=capture)
    return [*cargo, *resolved.command[1:]]


def measure(
    source: Path,
    target: Path,
    output: Path,
    name: str,
    cargo: list[str],
    env: dict[str, str],
    *,
    workspace: bool = False,
    packages: tuple[str, ...] = ("pse-compiler",),
    native: bool = False,
    owned_cache: bool = False,
    validate_selection: bool = True,
) -> dict:
    destination = output / name
    destination.mkdir()
    env = env.copy()
    server = start_cache_server(env, destination) if owned_cache else None
    try:
        return _measure_build(
            source,
            target,
            output,
            name,
            cargo,
            env,
            workspace=workspace,
            packages=packages,
            native=native,
            server=server,
            validate_selection=validate_selection,
        )
    finally:
        if server is not None:
            stop_cache_server(server, env)


def _measure_build(
    source: Path,
    target: Path,
    output: Path,
    name: str,
    cargo: list[str],
    env: dict[str, str],
    *,
    workspace: bool,
    packages: tuple[str, ...],
    native: bool,
    server: subprocess.Popen | None,
    validate_selection: bool,
) -> dict:
    """Resolve and run Cargo only after the selected foreground cache is ready."""
    preflight_started = time.monotonic()
    if validate_selection:
        validate_target(source, target, cargo, env)
    else:
        validate_target_path(source, target, env)
    preflight = {
        "cargo_selection_queried": validate_selection,
        "wall_seconds": time.monotonic() - preflight_started,
        "included_in_build_wall_seconds": False,
    }
    destination = output / name
    selection = (
        ["--workspace"]
        if workspace
        else [arg for package in packages for arg in ("-p", package)]
    )
    features = []
    if native:
        features = [
            "--features",
            "pse-runtime/native-solvers,pse-tests-conformance/native-acceptance",
        ]
    command = [
        *cargo,
        "build",
        "--tests",
        *selection,
        "--locked",
        *features,
    ]
    command = correctness_command(source, cargo, command[len(cargo) :], env)
    # Reporting controls do not change the selected validation closure; the resolver
    # deliberately accepts only scope/feature options for its unit-graph query.
    command.extend(["--timings", "--message-format=json-render-diagnostics"])
    # Use exactly the features resolved for the measured build in its feature report.
    # Correctness composition appends its feature clause after all build options;
    # preserve any original native feature clause separately as well.
    features = [
        arg
        for index, arg in enumerate(command)
        if arg == "--features" or (index and command[index - 1] == "--features")
    ]
    validation.write_json(destination / "source-files.json", validation.sources(source))
    validation.write_json(
        destination / "command.json",
        {
            "command": command,
            "source_directory": str(source),
            "target_directory": str(target),
            "environment": {
                key: env.get(key)
                for key in (
                    "CARGO_TARGET_DIR",
                    "CARGO_BUILD_JOBS",
                    "CARGO_INCREMENTAL",
                    "RUSTC_WRAPPER",
                    "RUSTC_WORKSPACE_WRAPPER",
                    "RUSTFLAGS",
                    "CARGO_ENCODED_RUSTFLAGS",
                    "RUSTUP_TOOLCHAIN",
                    "SCCACHE_DIR",
                    "SCCACHE_CACHE_SIZE",
                    "SCCACHE_MAX_FRAME_LENGTH",
                    "SCCACHE_SERVER_UDS",
                    "IPOPT_DIR",
                    "SUITESPARSE_LIBRARY_DIR",
                    "CC",
                    "CXX",
                    "LIBCLANG_PATH",
                    "BINDGEN_EXTRA_CLANG_ARGS",
                    "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER",
                    "CMAKE_TOOLCHAIN_FILE",
                    "PSE_NATIVE_COMPILER_CACHE",
                    "PSE_LLVM_PREFIX",
                    "CLANG_PATH",
                    "LLVM_CONFIG_PATH",
                )
            },
            "repetitions": 1,
            "dispersion": "unmeasured",
            "correctness_execution": False,
        },
    )
    cache_before = cache_stats(env)
    free_before = shutil.disk_usage(output).free
    before = resource.getrusage(resource.RUSAGE_CHILDREN)
    started = time.monotonic()
    with (
        (destination / "cargo.jsonl").open("w") as stdout,
        (destination / "cargo.stderr.log").open("w") as stderr,
    ):
        result = subprocess.Popen(
            command,
            cwd=source,
            env=env,
            stdout=stdout,
            stderr=stderr,
            start_new_session=True,
        )
        peak = 0
        sampling_done = threading.Event()

        def sample_memory() -> None:
            nonlocal peak
            while not sampling_done.is_set():
                memory = process_tree_rss(result.pid)
                if server is not None:
                    memory += process_tree_rss(server.pid)
                peak = max(peak, memory)
                sampling_done.wait(0.1)

        sampler = threading.Thread(target=sample_memory, daemon=True)
        sampler.start()
        try:
            result.wait()
            elapsed = time.monotonic() - started
        except BaseException:
            os.killpg(result.pid, signal.SIGTERM)
            try:
                result.wait(timeout=10)
            except subprocess.TimeoutExpired:
                os.killpg(result.pid, signal.SIGKILL)
                result.wait()
            raise
        finally:
            sampling_done.set()
            sampler.join()
            cache_after = cache_stats(env)
            if server is not None:
                stop_cache_server(server, env)
    after = resource.getrusage(resource.RUSAGE_CHILDREN)
    validation.write_json(destination / "cache-before.json", cache_before)
    validation.write_json(destination / "cache-after.json", cache_after)
    receipt = {
        "source_directory": str(source),
        "target_directory": str(target),
        "target_selection_preflight": preflight,
        "exit_code": result.returncode,
        "wall_seconds": elapsed,
        "user_seconds": after.ru_utime - before.ru_utime,
        "system_seconds": after.ru_stime - before.ru_stime,
        "sampled_process_tree_peak_rss_bytes": peak,
        "peak_rss_scope": "Linux /proc descendants sampled every 100ms; shared pages may be counted more than once",
        "resource_scope": (
            "build and owned foreground cache server; CPU includes the complete server "
            "lifetime (startup, pre-timing metadata service and shutdown); wall/RSS "
            "cover the build interval"
            if server is not None
            else "build descendants only; excludes any independent cache server"
        ),
        "cache_delta": counter_delta(cache_before, cache_after),
        "disk_free_before": free_before,
        "disk_free_after": shutil.disk_usage(output).free,
    }
    validation.write_json(destination / "result.json", receipt)
    if result.returncode:
        raise RuntimeError(f"{name} build failed; retained complete logs")
    receipt["target_identity"] = directory_identity(target)
    compiled = artifacts(destination / "cargo.jsonl")
    receipt.update(compiled)
    # Stable Cargo documents HTML as a human report. Keep the complete library-owned
    # graph for critical-path/largest-unit assessment instead of treating its private JS as an API.
    shutil.copy2(
        target / "cargo-timings/cargo-timing.html",
        destination / "cargo-timing.html",
    )
    with (destination / "features.txt").open("w") as stdout:
        subprocess.run(
            [
                *cargo,
                "tree",
                "--locked",
                *selection,
                *features,
                "-e",
                "features",
                "--format",
                "{p} features=[{f}]",
            ],
            cwd=source,
            env=env,
            stdout=stdout,
            check=True,
        )
    validation.write_json(destination / "result.json", receipt)
    return receipt


def cache_stats(env: dict[str, str]) -> dict:
    wrapper = env.get("RUSTC_WRAPPER", "")
    if Path(wrapper).name != "sccache":
        return {}
    result = subprocess.run(
        [wrapper, "--show-stats", "--stats-format=json"],
        env=env,
        capture_output=True,
        text=True,
        check=True,
    )
    return json.loads(result.stdout)


def stop_cache_server(server: subprocess.Popen, env: dict[str, str]) -> None:
    if server.poll() is None:
        subprocess.run(
            [env["RUSTC_WRAPPER"], "--stop-server"],
            env=env,
            check=False,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            timeout=10,
        )
        try:
            server.wait(timeout=10)
        except subprocess.TimeoutExpired:
            server.terminate()
            server.wait(timeout=10)
        Path(env["SCCACHE_SERVER_UDS"]).unlink(missing_ok=True)


def start_cache_server(env: dict[str, str], output: Path) -> subprocess.Popen:
    """Own and reap a foreground server so its compiler CPU and RSS are accounted."""
    endpoint = Path(env["SCCACHE_SERVER_UDS"])
    if endpoint.exists():
        raise ValueError("measurement endpoint is already occupied")
    # Keep compilation and observation on this explicitly owned endpoint, including
    # nested pse-env/just composition in workflow measurements.
    binary = env.get("PSE_SCCACHE_BINARY", env["RUSTC_WRAPPER"])
    env.update(
        RUSTC_WRAPPER=binary,
        SCCACHE_NO_DAEMON="1",
        **{build_environment.FOREGROUND_CACHE: str(endpoint)},
    )
    if env.get("PSE_NATIVE_COMPILER_CACHE"):
        env["PSE_NATIVE_COMPILER_CACHE"] = binary
    with (output / "cache-server.log").open("w") as log:
        server = subprocess.Popen(
            [env["RUSTC_WRAPPER"]],
            env={**env, "SCCACHE_START_SERVER": "1", "SCCACHE_NO_DAEMON": "1"},
            stdout=log,
            stderr=subprocess.STDOUT,
        )
    atexit.register(stop_cache_server, server, env.copy())
    deadline = time.monotonic() + 10
    while not endpoint.exists():
        if server.poll() is not None or time.monotonic() > deadline:
            stop_cache_server(server, env)
            raise RuntimeError("foreground compiler-cache server failed to start")
        time.sleep(0.01)
    return server


def counter_delta(before: dict, after: dict) -> dict:
    result: dict[str, object] = {}
    for key, value in after.items():
        old = before.get(key, {} if isinstance(value, dict) else 0)
        if isinstance(value, dict) and isinstance(old, dict):
            result[key] = counter_delta(old, value)
        elif type(value) is int and type(old) is int:
            result[key] = value - old
    return result


def process_tree_rss(pid: int) -> int:
    """Sample only this build's live descendants; never reuse a campaign high-water mark."""
    total = 0
    pending = [pid]
    seen = set()
    while pending:
        child = pending.pop()
        if child in seen:
            continue
        seen.add(child)
        try:
            status = Path(f"/proc/{child}/status").read_text()
            total += next(
                (
                    int(line.split()[1]) * 1024
                    for line in status.splitlines()
                    if line.startswith("VmRSS:")
                ),
                0,
            )
            for task in Path(f"/proc/{child}/task").iterdir():
                pending.extend(map(int, (task / "children").read_text().split()))
        except (FileNotFoundError, ProcessLookupError):
            continue
    return total


def private_edit(source: Path) -> tuple[Path, str, str]:
    """Validate a real private-body probe before starting expensive samples."""
    private = source / "crates/pse-compiler/src/workspace.rs"
    original = private.read_text()
    marker = "fn name_key_heap((text,): &(String,)) -> usize {\n    text.capacity()\n}"
    if original.count(marker) != 1:
        raise ValueError("private build probe needs one exact body-edit anchor")
    return (
        private,
        original,
        original.replace(
            marker,
            marker.replace("text.capacity()", "std::hint::black_box(text).capacity()"),
        ),
    )


def edit_body(source: Path) -> tuple[Path, str]:
    private, original, replacement = private_edit(source)
    private.write_text(replacement)
    return private, original


def build_phases(
    source: Path,
    repetitions: int,
    sample: Callable[[str], None],
    *,
    screen: bool,
) -> None:
    """Observe each isolated edit and restoration using the same selected build."""
    if not screen:
        private_edit(source)
        (source / "crates/pse-compiler/src/lib.rs").read_text()
        (source / "docs/dev/documentation.md").read_bytes()
    sample("cold")
    for repetition in range(repetitions):
        sample(f"warm-{repetition}")
        if screen:
            continue
        private, _, _ = private_edit(source)
        os.utime(private, None)
        sample(f"identical-touch-{repetition}")
        private, original = edit_body(source)
        try:
            sample(f"private-edit-{repetition}")
        finally:
            private.write_text(original)
        sample(f"restore-private-{repetition}")
        public = source / "crates/pse-compiler/src/lib.rs"
        original = public.read_text()
        try:
            public.write_text(
                original
                + "\n/// Build measurement marker.\npub const BUILD_MEASUREMENT: usize = 1;\n"
            )
            sample(f"public-edit-{repetition}")
        finally:
            public.write_text(original)
        sample(f"restore-public-{repetition}")
        unrelated = source / "docs/dev/documentation.md"
        original_bytes = unrelated.read_bytes()
        try:
            unrelated.write_bytes(
                original_bytes + b"\n<!-- Build measurement unrelated edit. -->\n"
            )
            sample(f"unrelated-edit-{repetition}")
        finally:
            unrelated.write_bytes(original_bytes)
        sample(f"restore-unrelated-{repetition}")


def profile_override(source: Path, level: int | None) -> None:
    """Apply candidate policy only in the snapshot, including nested just/maturin."""
    if level is None:
        return
    config = source / ".cargo/config.toml"
    original = config.read_text()
    if "profile" in tomllib.loads(original):
        raise ValueError("measurement expects profile policy in the workspace manifest")
    overrides = (
        f'\n[profile.dev.package."*"]\nopt-level = {level}\n'
        if level is not None
        else ""
    )
    config.write_text(original + overrides)


def execute_workload(
    command: list[str],
    source: Path,
    env: dict[str, str],
    log: Path,
    *,
    target: Path,
    target_cargo: list[str],
    owned_cache: bool,
    correctness_cargo: list[str] | None = None,
    validate_selection: bool = True,
) -> dict:
    env = env.copy()
    server = start_cache_server(env, log.parent) if owned_cache else None
    try:
        preflight_started = time.monotonic()
        if validate_selection:
            validate_target(source, target, target_cargo, env)
        else:
            validate_target_path(source, target, env)
        preflight = {
            "cargo_selection_queried": validate_selection,
            "wall_seconds": time.monotonic() - preflight_started,
            "included_in_workload_wall_seconds": False,
        }
        if correctness_cargo is not None:
            command = correctness_command(
                source,
                correctness_cargo,
                command[len(correctness_cargo) :],
                env,
            )
        started = time.monotonic()
        with log.open("w") as stream:
            result = subprocess.run(
                command,
                cwd=source,
                env=env,
                stdout=stream,
                stderr=subprocess.STDOUT,
                check=False,
            )
        return {
            "command": command,
            "source_directory": str(source),
            "target_directory": str(target),
            "target_selection_preflight": preflight,
            "wall_seconds": time.monotonic() - started,
            "exit_code": result.returncode,
        }
    finally:
        if server is not None:
            stop_cache_server(server, env)


def capability_environment(
    original: dict[str, str], *, native: bool, workflow: bool
) -> dict[str, str]:
    """Prepare linked capabilities only when the selected workload consumes them."""
    if not native and not workflow:
        return original.copy()
    if native_operation.owner_record() is None:
        raise ValueError(
            "native build measurement requires its operation owner before snapshot admission"
        )
    return native_operation.environment(
        ["solver", "klu", "isolation", "uno", "petsc"], original
    )


def prepare_inputs(
    root: Path,
    output: Path,
    original: dict[str, str],
    *,
    native: bool,
    workflow: bool,
    conditions: dict[str, object],
    prepare_store: bool = False,
) -> tuple[Path, dict[str, str], dict[str, object]]:
    """Time the snapshot and admission actually executed after target entry.

    The operation record proves existing admission/generation state, not elapsed
    setup time or a cache hit. Initial launcher setup has already happened here.
    """
    owner = (
        native_operation.current() if prepare_store else native_operation.owner_record()
    )

    def observation() -> dict[str, object] | None:
        if owner is None:
            return None
        with native_operation.record_lock(owner):
            return json.loads(owner.read_text())

    operation: dict[str, object] = {
        "preexisting_at_measurement_target_entry": owner is not None,
        "record_path": str(owner) if owner is not None else None,
        # Store authentication owns its clock and the native-operation timing
        # receipt already binds the entry owner. Do not wait on a record lock
        # before that phase has established its deadline.
        "before": None if prepare_store else observation(),
        "evidence_scope": "authentic existing admissions and generation pins; no setup durations or cache-hit inference",
    }
    receipt: dict[str, object] = {
        "scope": "measurement target selected store preparation, snapshot and capability_environment calls",
        "excludes": [
            "native operation launch and capability setup before target entry",
            "Cargo samples and compiler-cache server startup/shutdown",
            "receipt serialization and final operation drain",
        ],
        "conditions": conditions,
        "requested_capabilities": ["solver", "klu", "isolation", "uno", "petsc"]
        if native or workflow
        else [],
        "phase_order": [
            *(["selected_store"] if prepare_store else []),
            "snapshot",
            "capability_environment",
        ],
        "phases": {},
        "native_operation": operation,
        "cache_scope": "native prerequisites follow the existing operation; cold-cache selects a downstream empty compiler cache only",
    }
    phases: dict[str, object] = {}
    receipt["phases"] = phases
    phase = "selected_store" if prepare_store else "snapshot"
    started = time.monotonic()
    try:
        if prepare_store:
            phases[phase] = prepare_selected_store(output, original)
            os.environ.pop(STORE_LAUNCH, None)
            original.pop(STORE_LAUNCH, None)
            os.environ["PSE_REQUIRE_STORE"] = "1"
            original = {**original, "PSE_REQUIRE_STORE": "1"}
            phase = "snapshot"
            started = time.monotonic()
        source = snapshot(root, output)
        phases[phase] = {"wall_seconds": time.monotonic() - started, "status": "passed"}
        phase = "capability_environment"
        started = time.monotonic()
        env = capability_environment(original, native=native, workflow=workflow)
        phases[phase] = {"wall_seconds": time.monotonic() - started, "status": "passed"}
        operation["after"] = observation()
        receipt["native_environment"] = {
            name: env.get(name)
            for name in (
                "PSE_NATIVE_CACHE",
                "PSE_NATIVE_COMPILER_CACHE",
                "RUSTC_WRAPPER",
                "SCCACHE_DIR",
                "SCCACHE_CACHE_SIZE",
                "SCCACHE_MAX_FRAME_LENGTH",
                "IPOPT_DIR",
                "SUITESPARSE_LIBRARY_DIR",
                "PSE_ROOT_ISOLATION_DIR",
                "UNO_DIR",
                "PETSC_DIR",
                "CLANG_PATH",
                "CMAKE_TOOLCHAIN_FILE",
                "OMP_NUM_THREADS",
                "OPENBLAS_NUM_THREADS",
                "MKL_NUM_THREADS",
            )
        }
        receipt["status"] = "passed"
    except BaseException as error:
        if phase not in phases:
            failed_phase: dict[str, object] = {
                "wall_seconds": time.monotonic() - started,
                "status": "failed",
                "error_type": type(error).__name__,
            }
            if phase == "selected_store":
                failed_phase.update(
                    started_monotonic=started,
                    finished_monotonic=time.monotonic(),
                )
            phases[phase] = failed_phase
        receipt["status"] = "failed"
        receipt["error_type"] = type(error).__name__
        raise
    else:
        return source, env, receipt
    finally:
        validation.write_json(output / "preparation.json", receipt)


def complete_operation(output: Path, started: float, exit_code: int) -> int:
    """Settle the observed native owner after the existing launcher returns.

    A failed/unknown drain is a failed measurement, never a completed interval.
    Pins remain owned by native_operation regardless of this receipt's status.
    """
    receipt: dict[str, object] = {
        "status": "failed",
        "exit_code": exit_code,
        "setup_inclusive": False,
        "scope": "before pse-env launch through authenticated native scope drain observation",
        "excludes": [
            "argument parsing and fresh output directory creation in observing parent",
            "final complete-operation and summary receipt serialization",
        ],
        "phases": {},
    }
    phases: dict[str, object] = {}
    receipt["phases"] = phases
    store_prepared = True
    try:
        native = json.loads((output / "native-operation.json").read_text())
        receipt["native_operation"] = native
        setup_start = native["setup_started_monotonic"]
        setup_end = native["setup_finished_monotonic"]
        if not started <= setup_start <= setup_end:
            raise ValueError("native setup boundaries are out of order")  # noqa: TRY301 -- observer records malformed timing as a failed operation
        phases["launch_and_admission"] = {
            "wall_seconds": setup_start - started,
            "status": "passed",
            "scope": "pse-env launch, environment composition, host admission and native owner creation",
        }
        phases["native_setup"] = {
            "wall_seconds": setup_end - setup_start,
            "status": native["setup_status"],
            "scope": "initial selected native capability environment call; no cache-hit inference",
        }
        child_end = native.get("child_finished_monotonic")
        if child_end is not None:
            child_start = native["child_started_monotonic"]
            if not setup_end <= child_start <= child_end:
                raise ValueError("native child boundaries are out of order")  # noqa: TRY301 -- observer records malformed timing as a failed operation
            phases["measurement_child"] = {
                "wall_seconds": child_end - child_start,
                "status": "passed" if native["child_exit_code"] == 0 else "failed",
                "scope": "native supervisor child launch through wait return; includes target interpreter and serialization",
            }
            phases["setup_handoff"] = {
                "wall_seconds": child_start - setup_end,
                "status": "passed",
                "scope": "setup receipt serialization and native supervisor child preparation",
            }
            preparation_path = output / "preparation.json"
            if preparation_path.exists():
                preparation = json.loads(preparation_path.read_text())
                store = preparation.get("phases", {}).get("selected_store")
                if store is not None:
                    if not (
                        child_start
                        <= store["started_monotonic"]
                        <= store["finished_monotonic"]
                        <= child_end
                    ):
                        raise ValueError(  # noqa: TRY301 -- record malformed timing
                            "store preparation boundaries are out of order"
                        )
                    phases["selected_store_preparation"] = {
                        **store,
                        "included_in": "measurement_child",
                    }
                    store_prepared = store.get("status") == "passed"
                    if not store_prepared:
                        receipt["error"] = "selected store preparation did not complete"
        drain_start = child_end if child_end is not None else setup_end
        native_drain = native_operation.drained(native["owner"])
        native_drained = native_drain.complete
        allocation = Path(native["host_allocation"]["path"])
        # Read the current ledger, not a pre-return copy which could miss later
        # registered handoff scopes. A removed owner was retired by host admission.
        ledger = host_admission.readonly_snapshot(
            allocation.parent, deadline=time.monotonic() + 10
        )
        allocation_owner = ledger["owners"].get(allocation.name)
        allocation_drained = allocation_owner is None or host_admission.drained(
            allocation_owner
        )
        receipt["drain_observation"] = {
            "native_scope_drained": native_drained,
            "native_scope_observation": {
                "phase": native_drain.phase.value,
                "cause": native_drain.cause,
                "selected": native_drain.owner,
                "observed": native_drain.observed,
            },
            "host_allocation_drained": allocation_drained,
            "host_allocation_retired": allocation_owner is None,
            "retained_host_allocation": allocation_owner,
        }
        drained = native_drained and allocation_drained
        finished = time.monotonic()
        if drain_start > finished:
            raise ValueError("native drain boundaries are out of order")  # noqa: TRY301 -- observer records malformed timing as a failed operation
        drain_phase: dict[str, object] = {
            "observed_wall_seconds": finished - drain_start,
            "status": "passed" if drained else "failed",
            "scope": "after native child wait (or failed setup) through launcher return and native scope drain observation",
        }
        phases["final_drain"] = drain_phase
        receipt["observed_wall_seconds"] = finished - started
        if drained:
            receipt["setup_inclusive"] = True
            drain_phase["wall_seconds"] = finished - drain_start
            receipt["wall_seconds"] = finished - started
            if (
                exit_code == 0
                and native["setup_status"] == "passed"
                and child_end is not None
                and native["child_exit_code"] == 0
                and store_prepared
            ):
                receipt["status"] = "passed"
        else:
            receipt["error"] = (
                "native scope drain could not be verified; generation pins remain owned"
            )
    except (OSError, ValueError, KeyError, TypeError) as error:
        receipt["error"] = str(error)
        receipt["observed_wall_seconds"] = time.monotonic() - started
    validation.write_json(output / "complete-operation.json", receipt)
    summary = output / "summary.json"
    if summary.exists():
        report = json.loads(summary.read_text())
        report["complete_operation"] = receipt
        validation.write_json(summary, report)
    return exit_code or (0 if receipt["status"] == "passed" else 125)


def ensure_capability_operation(
    root: Path,
    output: Path,
    *,
    native: bool,
    workflow: bool,
    prepare_store: bool = False,
) -> int | None:
    """Observe the real operation before setup, snapshot and child builds."""
    if os.environ.get(OPERATION_OUTPUT):
        if prepare_store:
            store_launch_handoff(output, dict(os.environ))
        return None
    existing = native_operation.owner_record()
    if prepare_store and (
        existing is not None or os.environ.get(host_admission.MARKER)
    ):
        raise ValueError("--prepare-store requires a new complete-operation owner")
    if existing is not None:
        return None
    selection = (
        selected_store(dict(os.environ), quiesced=True) if prepare_store else None
    )
    output = validation.fresh_output(root, output)
    environment = dict(os.environ)
    environment[OPERATION_OUTPUT] = str(output)
    if selection is not None:
        environment[STORE_SELECTION] = json.dumps(selection, sort_keys=True)
        # Deliberately defer this demand until the measured child owns placement.
        # An explicit empty caller value also prevents a local default restoring it.
        environment["PSE_REQUIRE_STORE"] = ""
    linked = native or workflow
    if linked:
        environment["PSE_NATIVE_CAPABILITIES"] = "solver,klu,isolation,uno,petsc"
        environment[native_operation.BUILD_TIMING] = str(
            output / "native-operation.json"
        )
    started = time.monotonic()
    if prepare_store:
        environment[STORE_LAUNCH] = json.dumps(
            {
                "pid": os.getpid(),
                "start": native_operation.start_identity(os.getpid()),
                "started_monotonic": started,
                "output": str(output),
            }
        )
    command = [
        str(root / "scripts/pse-env"),
        *(["--native"] if linked else []),
        "--",
        sys.executable,
        "-m",
        "scripts.build_measurements",
        *sys.argv[1:],
    ]
    try:
        status = native_operation.run(command, environment)
    except BaseException:
        if linked:
            complete_operation(output, started, 125)
        raise
    return complete_operation(output, started, status) if linked else status


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    parser.add_argument("--cache", choices=("auto", "on", "off"), default="auto")
    parser.add_argument("--frontend", type=int, choices=(1, 2, 4, 8))
    parser.add_argument(
        "--jobs", type=int, help="override the repository Cargo job budget"
    )
    parser.add_argument("--repetitions", type=int, default=3)
    parser.add_argument("--native", action="store_true")
    parser.add_argument(
        "--package",
        action="append",
        help="measure only these selected Cargo roots (repeatable)",
    )
    parser.add_argument(
        "--workload", choices=("compiler", "compiler-relations"), default="compiler"
    )
    parser.add_argument(
        "--execute",
        action="store_true",
        help="repeat selected workload library tests after build samples",
    )
    parser.add_argument(
        "--screen",
        action="store_true",
        help="cold and warm builds only; no edit/profile promotion claim",
    )
    parser.add_argument(
        "--workflow",
        action="store_true",
        help="also measure extension, compiled stubs and native units in the snapshot",
    )
    parser.add_argument(
        "--prepare-store",
        action="store_true",
        help="readmit/start the selected quiesced nonresident store inside a new native workflow owner",
    )
    parser.add_argument(
        "--cold-cache",
        action="store_true",
        help="use an empty task-owned compiler cache",
    )
    parser.add_argument(
        "--dependency-opt",
        type=int,
        choices=(1, 2, 3),
        help="override dependency optimization; default preserves the workspace manifest",
    )
    parser.add_argument("--recovery", action="store_true")
    parser.add_argument("--second-worktree", action="store_true")
    args = parser.parse_args()
    target_started = time.monotonic()
    if args.repetitions < 3:
        parser.error("warm/edit workloads require at least three repetitions")
    if args.execute and args.native:
        parser.error("use --workflow for the native unit selection")
    if args.prepare_store and not (args.native and args.workflow):
        parser.error("--prepare-store requires --native --workflow")
    try:
        packages = selected_packages(
            args.package, native=args.native, workload=args.workload
        )
    except ValueError as error:
        parser.error(str(error))
    root = Path(__file__).resolve().parents[1]
    settings = tomllib.loads((root / ".config/build.toml").read_text())
    if shutil.disk_usage(root).free < settings["free_space_gib"] * 1024**3:
        raise ValueError(
            "insufficient free space; inventory and reclaim inactive outputs first"
        )
    status = ensure_capability_operation(
        root,
        args.output,
        native=args.native,
        workflow=args.workflow,
        prepare_store=args.prepare_store,
    )
    if status is not None:
        raise SystemExit(status)
    if shutil.disk_usage(root).free < settings["free_space_gib"] * 1024**3:
        raise ValueError(
            "insufficient free space; inventory and reclaim inactive outputs first"
        )
    selected_output = os.environ.pop(OPERATION_OUTPUT, None)
    if selected_output is None:
        if args.prepare_store:
            raise ValueError("store preparation requires its measured output handoff")
        output = validation.fresh_output(root, args.output)
    else:
        output = Path(selected_output)
        if (
            output != args.output.resolve()
            or (
                not args.prepare_store
                and host_admission.inherit(dict(os.environ)) is None
            )
            or (
                not args.prepare_store
                and (args.native or args.workflow)
                and native_operation.current() is None
            )
        ):
            raise ValueError(
                "measurement output handoff requires its authentic workload owner"
            )
    source, capability_env, preparation = prepare_inputs(
        root,
        output,
        validation.command_env(),
        native=args.native,
        workflow=args.workflow,
        prepare_store=args.prepare_store,
        conditions={
            "native": args.native,
            "packages": packages,
            "workload": "native"
            if args.native
            else args.workload
            if not args.package
            else "selected-packages",
            "workflow": args.workflow,
            "prepare_store": args.prepare_store,
            "compiler_cache_selection": args.cache,
            "cold_compiler_cache": args.cold_cache,
            "frontend": args.frontend,
            "jobs": args.jobs,
            "dependency_opt": args.dependency_opt,
            "repetitions": args.repetitions,
            "screen": args.screen,
            "execute": args.execute,
            "recovery": args.recovery,
            "second_worktree": args.second_worktree,
        },
    )
    profile_override(source, args.dependency_opt)
    env = build_environment.configure(
        root,
        capability_env,
        cache=args.cache,
        frontend=args.frontend,
        jobs=args.jobs,
    )
    # A snapshot nested under the repository otherwise inherits both .cargo files,
    # concatenating target rustflags (including mold) twice. Encode the root's
    # effective flags once so the snapshot uses the same compiler invocation.
    env["CARGO_ENCODED_RUSTFLAGS"] = "\x1f".join(
        build_environment.effective_flags(root, env)
    )
    env = snapshot_environment(env)
    target = snapshot_target(source, output)
    source_identity = directory_identity(source)
    if args.cold_cache:
        if Path(env.get("RUSTC_WRAPPER", "")).name != "sccache":
            raise ValueError("--cold-cache requires the sccache wrapper")
        env["SCCACHE_DIR"] = str(output / "compiler-cache")
        # Keep the endpoint short even when the report destination is deeply nested.
        env["SCCACHE_SERVER_UDS"] = str(
            Path(tempfile.gettempdir())
            / f"pse-bench-{hashlib.sha256(str(output).encode()).hexdigest()[:16]}.sock"
        )
    # Original worktree source and its selected optimization policy stay untouched.
    # The snapshot's own toolchain file names the one pinned nightly.
    toolchain = tomllib.loads((source / "rust-toolchain.toml").read_text())[
        "toolchain"
    ]["channel"]
    cargo = ["rustup", "run", toolchain, "cargo"]
    validation.write_json(
        output / "host.json",
        {
            "rustc": subprocess.check_output(
                ["rustup", "run", toolchain, "rustc", "-Vv"], text=True
            ),
            "affinity": sorted(os.sched_getaffinity(0)),
            "cpu_quota": Path("/sys/fs/cgroup/cpu.max").read_text()
            if Path("/sys/fs/cgroup/cpu.max").exists()
            else None,
            "competing_processes": subprocess.check_output(
                ["ps", "-eo", "pid,comm"], text=True
            ),
            "linker": subprocess.check_output(["mold", "--version"], text=True),
        },
    )
    receipts = {}
    target_selections = [
        {"source_directory": str(source), "target_directory": str(target)}
    ]
    selection_validated = False
    target_identity = None

    def sample(name: str) -> None:
        nonlocal selection_validated, target_identity
        receipts[name] = measure(
            source,
            target,
            output,
            name,
            cargo,
            env,
            packages=packages,
            native=args.native,
            owned_cache=args.cold_cache,
            validate_selection=not selection_validated,
        )
        selection_validated = True
        target_identity = receipts[name].get("target_identity")

    build_phases(source, args.repetitions, sample, screen=args.screen)
    if args.execute:
        execution = []
        command = [
            *cargo,
            "test",
            *[arg for package in packages for arg in ("-p", package)],
            "--lib",
            "--locked",
            *(
                [
                    "--features",
                    "pse-runtime/native-solvers,pse-tests-conformance/native-acceptance",
                ]
                if args.native
                else []
            ),
        ]
        for repetition in range(args.repetitions):
            result = execute_workload(
                command,
                source,
                env,
                output / f"unit-execution-{repetition}.log",
                target=target,
                target_cargo=cargo,
                owned_cache=args.cold_cache,
                correctness_cargo=cargo,
                validate_selection=False,
            )
            execution.append({**result, "repetition": repetition})
            validation.write_json(output / "execution.json", execution)
            if result["exit_code"]:
                raise RuntimeError("selected workload unit execution failed")
    if args.workflow:
        workflow = []
        workflow_env = workflow_environment(env, source)
        workflow_validated = False
        for repetition in range(args.repetitions):
            for recipe in ("py-sync-native", "python-stubs", "unit-native-contracts"):
                name = f"workflow-{repetition}-{recipe}"
                process = execute_workload(
                    ["just", recipe],
                    source,
                    workflow_env,
                    output / f"{name}.log",
                    target=target,
                    target_cargo=cargo,
                    owned_cache=args.cold_cache,
                    validate_selection=not workflow_validated,
                )
                workflow_validated = True
                workflow.append(
                    {
                        "recipe": recipe,
                        "repetition": repetition,
                        **process,
                    }
                )
                validation.write_json(output / "workflow.json", workflow)
                if process["exit_code"]:
                    raise RuntimeError(f"workflow failed: {name}")
    if args.recovery:
        # Compiler cache keys include artifact paths. Lose only this campaign's
        # artifacts, then rebuild at the same path with its compiler cache intact.
        if target_identity is None:
            raise ValueError("recovery requires a successful build's target identity")
        recovery_env = env.copy()
        recovery_server = None
        if args.cold_cache:
            preflight_output = output / "recovery-preflight"
            preflight_output.mkdir()
            recovery_server = start_cache_server(recovery_env, preflight_output)
        try:
            recover_target(
                source,
                target,
                cargo,
                recovery_env,
                source_identity=source_identity,
                target_identity=target_identity,
            )
        finally:
            if recovery_server is not None:
                stop_cache_server(recovery_server, recovery_env)
        sample("artifact-recovery")
    if args.second_worktree:
        second = output / "second"
        second.mkdir()
        phases = preparation["phases"]
        phase_order = preparation["phase_order"]
        if not isinstance(phases, dict) or not isinstance(phase_order, list):
            raise ValueError("measurement preparation has invalid phase records")
        phase_order.append("second_snapshot")
        started = time.monotonic()
        try:
            source = snapshot(root, second)
            phases["second_snapshot"] = {
                "wall_seconds": time.monotonic() - started,
                "status": "passed",
            }
        except BaseException as error:
            phases["second_snapshot"] = {
                "wall_seconds": time.monotonic() - started,
                "status": "failed",
                "error_type": type(error).__name__,
            }
            preparation["status"] = "failed"
            raise
        finally:
            validation.write_json(output / "preparation.json", preparation)
        profile_override(source, args.dependency_opt)
        target = snapshot_target(source, second)
        source_identity = directory_identity(source)
        selection_validated = False
        target_selections.append(
            {"source_directory": str(source), "target_directory": str(target)}
        )
        sample("second-worktree")
    source_unchanged = validation.sources(root) == json.loads(
        (output / "original-source.json").read_text()
    )
    statistics_by_kind = {}
    for kind in (
        "warm",
        "identical-touch",
        "private-edit",
        "public-edit",
        "unrelated-edit",
    ):
        values = [
            receipt["wall_seconds"]
            for name, receipt in receipts.items()
            if name.startswith(kind + "-")
        ]
        if not values:
            continue
        statistics_by_kind[kind] = {
            "median": statistics.median(values),
            "min": min(values),
            "max": max(values),
            "stdev": statistics.stdev(values),
        }
    validation.write_json(
        output / "summary.json",
        {
            "snapshots": target_selections,
            "builds": receipts,
            "statistics": statistics_by_kind,
            "original_source_unchanged": source_unchanged,
            "cold_repetitions": 1,
            "cold_dispersion": "unmeasured",
            "cache_state": "new target; shared downloaded sources, native prerequisites and selected compiler cache",
            "critical_path": "retained Cargo HTML; summed CPU is not the critical path",
            "correctness_execution": args.execute or args.workflow,
            "preparation": preparation,
            "target_operation": {
                "wall_seconds": time.monotonic() - target_started,
                "scope": "actual measurement target after argument parsing through summary assembly",
                "excludes": [
                    "launcher and initial native capability setup before measurement target entry",
                    "final summary serialization and operation drain after target return",
                ],
                "setup_inclusive": False,
            },
        },
    )

    if not source_unchanged:
        raise ValueError(
            "original source changed during build measurement; diagnostic timings retained"
        )


if __name__ == "__main__":
    main()
