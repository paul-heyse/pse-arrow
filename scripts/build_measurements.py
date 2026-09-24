# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Characterize pinned Cargo builds in an isolated snapshot; never edit the worktree."""

from __future__ import annotations

import argparse
import atexit
import hashlib
import json
import os
import resource
import shutil
import signal
import statistics
import subprocess
import tempfile
import threading
import time
import tomllib
from pathlib import Path

from scripts import build_environment, implementation_phase, validation


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


def require_validation_mode(units: list[dict], expected: bool) -> None:
    """A requested feature switch is not evidence that unification changed it."""
    actual = {
        "force_validate" in unit["features"]
        for unit in units
        if unit["target"]["name"] == "arrow_data"
    }
    if actual != {expected}:
        raise ValueError(
            f"Arrow validation feature differs from requested build mode: {actual}"
        )


def measure(
    source: Path,
    output: Path,
    name: str,
    cargo: list[str],
    env: dict[str, str],
    *,
    workspace: bool = False,
    packages: tuple[str, ...] = ("pse-compiler", "pse-relations"),
    native: bool = False,
    validate: bool = True,
    owned_cache: bool = False,
) -> dict:
    destination = output / name
    destination.mkdir()
    selection = (
        ["--workspace"]
        if workspace
        else [arg for package in packages for arg in ("-p", package)]
    )
    features = ["--features", "pse-relations/force-validate"] if validate else []
    if native:
        features = [
            "--features",
            "pse-relations/force-validate,pse-runtime/native-solvers,pse-tests-conformance/native-acceptance",
        ]
    command = [
        *cargo,
        "build",
        "--tests",
        *selection,
        "--locked",
        *features,
        "--timings",
        "--message-format=json-render-diagnostics",
    ]
    validation.write_json(destination / "source-files.json", validation.sources(source))
    validation.write_json(
        destination / "command.json",
        {
            "command": command,
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
    server = start_cache_server(env, destination) if owned_cache else None
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
        "exit_code": result.returncode,
        "wall_seconds": elapsed,
        "user_seconds": after.ru_utime - before.ru_utime,
        "system_seconds": after.ru_stime - before.ru_stime,
        "sampled_process_tree_peak_rss_bytes": peak,
        "peak_rss_scope": "Linux /proc descendants sampled every 100ms; shared pages may be counted more than once",
        "resource_scope": (
            "build and owned foreground cache server, including server startup/shutdown CPU"
            if owned_cache
            else "build descendants only; excludes any independent cache server"
        ),
        "cache_delta": counter_delta(cache_before, cache_after),
        "disk_free_before": free_before,
        "disk_free_after": shutil.disk_usage(output).free,
    }
    validation.write_json(destination / "result.json", receipt)
    if result.returncode:
        raise RuntimeError(f"{name} build failed; retained complete logs")
    compiled = artifacts(destination / "cargo.jsonl")
    require_validation_mode(compiled["artifacts"], validate)
    receipt.update(compiled)
    # Stable Cargo documents HTML as a human report. Keep the complete library-owned
    # graph for critical-path/largest-unit assessment instead of treating its private JS as an API.
    shutil.copy2(
        Path(env["CARGO_TARGET_DIR"]) / "cargo-timings/cargo-timing.html",
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


def edit_body(source: Path) -> tuple[Path, str]:
    private = source / "crates/pse-compiler/src/physical_identity.rs"
    original = private.read_text()
    marker = "h.u64(v.to_bits());"
    if original.count(marker) != 1:
        raise ValueError("private build probe needs one exact body-edit anchor")
    private.write_text(
        original.replace(marker, "h.u64(std::hint::black_box(v).to_bits());")
    )
    return private, original


def profile_override(source: Path, level: int, delta_cache: bool) -> None:
    """Apply candidate policy only in the snapshot, including nested just/maturin."""
    config = source / ".cargo/config.toml"
    original = config.read_text()
    if "profile" in tomllib.loads(original):
        raise ValueError("measurement expects profile policy in the workspace manifest")
    overrides = f'\n[profile.dev.package."*"]\nopt-level = {level}\n'
    if delta_cache:
        overrides += "\n[profile.dev.package.deltalake-core]\nincremental = false\n"
    config.write_text(original + overrides)


def execute_workload(
    command: list[str],
    source: Path,
    env: dict[str, str],
    log: Path,
    *,
    owned_cache: bool,
) -> dict:
    server = start_cache_server(env, log.parent) if owned_cache else None
    try:
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
            "wall_seconds": time.monotonic() - started,
            "exit_code": result.returncode,
        }
    finally:
        if server is not None:
            stop_cache_server(server, env)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    parser.add_argument("--mode", choices=("stable", "nightly"), default="stable")
    parser.add_argument("--cache", choices=("auto", "on", "off"), default="auto")
    parser.add_argument("--frontend", type=int, choices=(1, 2, 4, 8))
    parser.add_argument("--jobs", type=int, default=16)
    parser.add_argument("--repetitions", type=int, default=3)
    parser.add_argument("--native", action="store_true")
    parser.add_argument(
        "--execute",
        action="store_true",
        help="repeat compiler/relations library tests after build samples",
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
        "--cold-cache",
        action="store_true",
        help="use an empty task-owned compiler cache",
    )
    parser.add_argument("--dependency-opt", type=int, choices=(1, 2), default=2)
    parser.add_argument("--delta-cache", action="store_true")
    parser.add_argument("--recovery", action="store_true")
    parser.add_argument("--second-worktree", action="store_true")
    args = parser.parse_args()
    if args.repetitions < 3:
        parser.error("warm/edit workloads require at least three repetitions")
    if args.execute and args.native:
        parser.error("use --workflow for the native unit selection")
    root = Path(__file__).resolve().parents[1]
    implementation_phase.guard(root, ["bench-builds"])
    settings = tomllib.loads((root / ".config/build.toml").read_text())
    if shutil.disk_usage(root).free < settings["free_space_gib"] * 1024**3:
        raise ValueError(
            "insufficient free space; inventory and reclaim inactive outputs first"
        )
    output = validation.fresh_output(root, args.output)
    source = snapshot(root, output)
    profile_override(source, args.dependency_opt, args.delta_cache)
    env = build_environment.configure(
        root,
        validation.command_env(),
        mode=args.mode,
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
    env["CARGO_TARGET_DIR"] = str(output / "target")
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
    cargo = ["rustup", "run", env["RUSTUP_TOOLCHAIN"], "cargo"]
    packages = (
        (
            "pse-backend-native",
            "pse-compiler",
            "pse-kernels",
            "pse-math",
            "pse-runtime",
            "pse-structural",
            "pse-tests-conformance",
        )
        if args.native
        else ("pse-compiler", "pse-relations")
    )
    validation.write_json(
        output / "host.json",
        {
            "rustc": subprocess.check_output(
                ["rustup", "run", env["RUSTUP_TOOLCHAIN"], "rustc", "-Vv"], text=True
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

    def sample(name: str) -> None:
        receipts[name] = measure(
            source,
            output,
            name,
            cargo,
            env,
            packages=packages,
            native=args.native,
            owned_cache=args.cold_cache,
        )

    sample("cold")
    for repetition in range(args.repetitions):
        sample(f"warm-{repetition}")
        if args.screen:
            continue
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
    if args.execute:
        execution = []
        command = [
            *cargo,
            "test",
            "-p",
            "pse-compiler",
            "-p",
            "pse-relations",
            "--lib",
            "--locked",
            "--features",
            "pse-relations/force-validate",
        ]
        for repetition in range(args.repetitions):
            result = execute_workload(
                command,
                source,
                env,
                output / f"unit-execution-{repetition}.log",
                owned_cache=args.cold_cache,
            )
            execution.append({**result, "repetition": repetition})
            validation.write_json(output / "execution.json", execution)
            if result["exit_code"]:
                raise RuntimeError("compiler/relations unit execution failed")
    if args.workflow:
        workflow = []
        workflow_env = {**env, "UV_PROJECT_ENVIRONMENT": str(source / ".venv")}
        workflow_env.pop("VIRTUAL_ENV", None)
        for repetition in range(args.repetitions):
            for recipe in ("py-sync-native", "python-stubs", "unit-native-contracts"):
                name = f"workflow-{repetition}-{recipe}"
                process = execute_workload(
                    ["just", recipe],
                    source,
                    workflow_env,
                    output / f"{name}.log",
                    owned_cache=args.cold_cache,
                )
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
        target = Path(env["CARGO_TARGET_DIR"])
        if target != output / "target":
            raise ValueError("recovery may remove only the campaign-owned target")
        shutil.rmtree(target)
        sample("artifact-recovery")
    if args.second_worktree:
        second = output / "second"
        second.mkdir()
        source = snapshot(root, second)
        profile_override(source, args.dependency_opt, args.delta_cache)
        env["CARGO_TARGET_DIR"] = str(output / "second-target")
        sample("second-worktree")
    source_unchanged = validation.sources(root) == json.loads(
        (output / "original-source.json").read_text()
    )
    statistics_by_kind = {}
    for kind in ("warm", "private-edit", "public-edit"):
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
            "builds": receipts,
            "statistics": statistics_by_kind,
            "original_source_unchanged": source_unchanged,
            "cold_repetitions": 1,
            "cold_dispersion": "unmeasured",
            "cache_state": "new target; shared downloaded sources, native prerequisites and selected compiler cache",
            "critical_path": "retained Cargo HTML; summed CPU is not the critical path",
            "correctness_execution": args.execute or args.workflow,
        },
    )

    if not source_unchanged:
        raise ValueError(
            "original source changed during build measurement; diagnostic timings retained"
        )


if __name__ == "__main__":
    main()
