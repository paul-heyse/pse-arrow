# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Characterize pinned Cargo builds in an isolated snapshot; never edit the worktree."""

from __future__ import annotations

import argparse
import json
import resource
import shutil
import subprocess
import time
import tomllib
from pathlib import Path

from scripts import implementation_phase, validation


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
    validate: bool = True,
) -> dict:
    destination = output / name
    destination.mkdir()
    selection = ["--workspace"] if workspace else ["-p", "pse-tests-engine"]
    features = ["--features", "pse-relations/force-validate"] if validate else []
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
                )
            },
            "repetitions": 1,
            "dispersion": "unmeasured",
            "correctness_execution": False,
        },
    )
    before = resource.getrusage(resource.RUSAGE_CHILDREN)
    started = time.monotonic()
    with (
        (destination / "cargo.jsonl").open("w") as stdout,
        (destination / "cargo.stderr.log").open("w") as stderr,
    ):
        result = subprocess.run(
            command, cwd=source, env=env, stdout=stdout, stderr=stderr, check=False
        )
    elapsed = time.monotonic() - started
    after = resource.getrusage(resource.RUSAGE_CHILDREN)
    receipt = {
        "exit_code": result.returncode,
        "wall_seconds": elapsed,
        "user_seconds": after.ru_utime - before.ru_utime,
        "system_seconds": after.ru_stime - before.ru_stime,
        "campaign_child_peak_rss_bytes": after.ru_maxrss * 1024,
        "peak_rss_scope": "maximum of all waited-for children so far; not per-build or aggregate concurrent RSS",
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


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    implementation_phase.guard(root, ["bench-builds"])
    output = validation.fresh_output(root, args.output)
    source = snapshot(root, output)
    channel = tomllib.loads((root / "rust-toolchain.toml").read_text())["toolchain"][
        "channel"
    ]
    cargo = ["rustup", "run", channel, "cargo"]
    env = validation.command_env()
    env.update(
        CARGO_TARGET_DIR=str(output / "target"),
        RUSTC_WRAPPER="",
        RUSTC_WORKSPACE_WRAPPER="",
    )
    receipts = {}
    receipts["cold-engine-tests"] = measure(
        source, output, "cold-engine-tests", cargo, env
    )
    private = source / "crates/pse-compiler/src/topology.rs"
    original = private.read_text()
    marker = ".saturating_add(1)"
    if original.count(marker) != 1:
        raise ValueError("private build probe no longer has one exact body-edit anchor")
    private.write_text(
        original.replace(marker, ".saturating_add(std::hint::black_box(1))")
    )
    receipts["private-body-edit"] = measure(
        source, output, "private-body-edit", cargo, env
    )
    private.write_text(original)
    measure(source, output, "restore-private-baseline", cargo, env)
    public = source / "crates/pse-compiler/src/lib.rs"
    original = public.read_text()
    public.write_text(
        original
        + "\n/// Isolated build characterization marker.\npub const BUILD_CHARACTERIZATION: usize = 1;\n"
    )
    receipts["public-api-edit"] = measure(source, output, "public-api-edit", cargo, env)
    public.write_text(original)
    measure(source, output, "restore-public-baseline", cargo, env)
    receipts["validation-feature-flip"] = measure(
        source, output, "validation-feature-flip", cargo, env, validate=False
    )
    receipts["workspace-tests"] = measure(
        source, output, "workspace-tests", cargo, env, workspace=True
    )
    if validation.sources(root) != json.loads(
        (output / "original-source.json").read_text()
    ):
        raise ValueError("original source changed during build measurement")
    validation.write_json(
        output / "summary.json",
        {
            "builds": receipts,
            "cold_repetitions": 1,
            "cold_dispersion": "unmeasured",
            "cache_state": "new target directory; shared dependency downloads and host filesystem cache; compiler wrappers disabled",
            "critical_path": "assess each retained Cargo HTML report; summed CPU is not the critical path",
        },
    )


if __name__ == "__main__":
    main()
