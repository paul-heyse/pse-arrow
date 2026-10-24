#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Qualify cargo-hack check features while retaining workspace feature unification.

cargo-hack 0.6.45 selects CARGO_HACK_CARGO_SRC before CARGO (upstream
src/main.rs, try_main). This executable adapts only bare check feature arguments;
cargo-hack still owns enumeration, ordering, and keep-going failure accounting.
"""

from __future__ import annotations

import os
import json
import hashlib
import re
import shlex
import shutil
import subprocess
import sys
import tomllib
from pathlib import Path

if not __package__:
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from scripts import build_environment
from scripts.arrow_validation import capture, compose


def qualify_features(arguments: list[str]) -> list[str]:
    """Preserve arguments except bare features of a selected check package."""
    if not arguments or arguments[0] != "check":
        return arguments.copy()
    manifest: str | None = None
    features: list[tuple[int, str, list[str]]] = []
    index = 1
    while index < len(arguments):
        argument = arguments[index]
        if argument == "--":
            break
        if argument in {"--manifest-path", "--features", "-F"}:
            if index + 1 == len(arguments):
                message = f"{argument} requires a value"
                raise ValueError(message)
            index += 1
            if argument == "--manifest-path":
                manifest = arguments[index]
            else:
                features.append(
                    (index, "", re.split(r"[\s,]+", arguments[index].strip()))
                )
        elif argument.startswith("--manifest-path="):
            manifest = argument.removeprefix("--manifest-path=")
        elif argument.startswith("--features="):
            features.append(
                (
                    index,
                    "--features=",
                    re.split(r"[\s,]+", argument.removeprefix("--features=").strip()),
                )
            )
        elif argument.startswith("-F") and len(argument) > 2:
            features.append((index, "-F", re.split(r"[\s,]+", argument[2:].strip())))
        index += 1

    if not any(item and "/" not in item for _, _, items in features for item in items):
        return arguments.copy()
    if not manifest:
        message = "bare check features require a selected --manifest-path"
        raise ValueError(message)
    with Path(manifest).open("rb") as source:
        document = tomllib.load(source)
    package = document.get("package", {})
    name = package.get("name") if isinstance(package, dict) else None
    if not isinstance(name, str) or not name:
        message = f"selected manifest lacks package.name: {manifest}"
        raise ValueError(message)
    qualified = arguments.copy()
    for position, prefix, items in features:
        if any(item and "/" not in item for item in items):
            qualified[position] = prefix + ",".join(
                item if "/" in item else f"{name}/{item}" for item in items if item
            )
    return qualified


def diagnostic_check(arguments: list[str]) -> tuple[list[str], dict[str, str] | None]:
    """Select the existing isolated taskdump route only for its explicit feature."""
    if not arguments or arguments[0] != "check":
        return arguments, None
    selected = False
    profile: str | None = None
    boundary = arguments.index("--") if "--" in arguments else len(arguments)
    index = 1
    while index < boundary:
        argument = arguments[index]
        if argument in {"--features", "-F", "--profile"}:
            if index + 1 == boundary:
                raise ValueError(f"{argument} requires a value")
            index += 1
            if argument == "--profile":
                profile = arguments[index]
            else:
                selected |= any(
                    feature.rsplit("/", 1)[-1] == "diagnostic-taskdump"
                    for feature in re.split(r"[\s,]+", arguments[index])
                )
        elif argument.startswith(("--features=", "-F")):
            value = (
                argument.split("=", 1)[1]
                if argument.startswith("--features=")
                else argument[2:]
            )
            selected |= any(
                feature.rsplit("/", 1)[-1] == "diagnostic-taskdump"
                for feature in re.split(r"[\s,]+", value)
            )
        elif argument.startswith("--profile="):
            profile = argument.removeprefix("--profile=")
        index += 1
    if not selected:
        return arguments, None
    if (
        profile is not None and profile != "diagnostic-taskdump"
    ) or "--release" in arguments[:boundary]:
        raise ValueError(
            "diagnostic-taskdump checks require the isolated diagnostic-taskdump profile"
        )
    command = arguments.copy()
    if profile is None:
        command[boundary:boundary] = ["--profile", "diagnostic-taskdump"]
    environment = build_environment.configure(
        Path(__file__).resolve().parents[1], dict(os.environ), taskdump=True
    )
    return command, environment


def cargo_executable() -> str:
    """Use Cargo's existing executable selection without a toolchain override."""
    cargo = os.environ.get("CARGO") or shutil.which("cargo")
    if not cargo:
        message = "Cargo executable unavailable: set CARGO or provide cargo on PATH"
        raise ValueError(message)
    if Path(cargo).resolve() == Path(__file__).resolve():
        message = "CARGO points to the feature adapter instead of Cargo"
        raise ValueError(message)
    return cargo


def main() -> int:
    """Forward to existing Cargo with inherited streams and its failure status."""
    try:
        literal = sys.argv[1:]
        # cargo-hack's printed commands are lossy display text. Its Cargo adapter
        # boundary supplies literal argv and is the sole enumeration authority.
        capture_path = os.environ.get("PSE_FEATURE_CAPTURE")
        if capture_path and literal[:1] == ["check"]:
            with Path(capture_path).open("a") as output:
                output.write(json.dumps(literal) + "\n")
                output.flush()
                os.fsync(output.fileno())
            return 0
        arguments, environment = diagnostic_check(qualify_features(literal))
        cargo = cargo_executable()
        command = ["cargo", *arguments]
        graphs = []

        def validation_capture(scope: list[str]) -> bytes:
            selected = [cargo, *scope[1:]]
            data = capture(selected, env=environment)
            graphs.append({"argv": selected, "sha256": hashlib.sha256(data).hexdigest()})
            return data
        if arguments[:1] == ["check"]:
            command = compose(
                command,
                run=validation_capture,
            )
            print(
                "cargo feature check: " + shlex.join(command),
                file=sys.stderr,
                flush=True,
            )
        command[0] = cargo
        if arguments[:1] == ["check"] and (attempt := os.environ.get("PSE_FEATURE_ATTEMPT")):
            path = Path(attempt)
            temporary = path.with_suffix(".pending")
            temporary.write_text(json.dumps({"schema": "cargo-feature-execution-v1", "literal_argv": literal, "qualified_argv": arguments, "actual_argv": command, "correctness": "arrow-unit-graph-force-validation-v1", "validation_graphs": graphs, "effective_environment": {key: hashlib.sha256(value.encode()).hexdigest() for key, value in sorted((os.environ if environment is None else environment).items())}}, sort_keys=True) + "\n")
            temporary.replace(path)
        if environment is None:
            code = subprocess.run(command, check=False).returncode
        else:
            code = subprocess.run(command, check=False, env=environment).returncode
        # Shell exit status for a child terminated by a signal.
        return code if code >= 0 else 128 - code
    except (OSError, ValueError, tomllib.TOMLDecodeError) as error:
        print(f"cargo feature check: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
