# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Compose correctness commands from Cargo's actual target/host unit graph.

No package root is added. Cargo owns optional/target/dev dependency selection and
feature resolution; manifests own opt-in propagation. The second graph establishes
actual validation-capable Arrow leaf features, rather than trusting a feature's name.
"""

from __future__ import annotations

import shlex
import subprocess
import sys
from pathlib import Path
from typing import TYPE_CHECKING

import msgspec

if TYPE_CHECKING:
    from collections.abc import Callable, Sequence

if not __package__:
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))


class ValidationError(ValueError):
    """The requested correctness scope cannot establish full Arrow validation."""


class Package(msgspec.Struct):
    id: str
    name: str
    features: dict[str, list[str]]


class Metadata(msgspec.Struct):
    packages: list[Package]


class Dependency(msgspec.Struct):
    index: int


class Target(msgspec.Struct):
    name: str
    kind: list[str]
    src_path: str


class Unit(msgspec.Struct):
    pkg_id: str
    target: Target
    features: list[str]
    dependencies: list[Dependency]
    platform: str | None
    mode: str


class Graph(msgspec.Struct):
    version: int
    units: list[Unit]
    roots: list[int]


# These options affect Cargo compilation or are Cargo execution diagnostics. Their
# values pass through unchanged; Cargo, not this composer, interprets their meaning.
VALUE = {
    "-p",
    "--package",
    "--exclude",
    "--features",
    "-F",
    "--target",
    "--target-dir",
    "--manifest-path",
    "--config",
    "--bin",
    "--test",
    "--bench",
    "--example",
    "-j",
    "--jobs",
    "--color",
    "--message-format",
    "--lockfile-path",
    "-Z",
}
FLAG = {
    "--workspace",
    "--all",
    "--lib",
    "--bins",
    "--tests",
    "--benches",
    "--examples",
    "--all-targets",
    "--no-default-features",
    "--all-features",
    "--release",
    "--locked",
    "--offline",
    "--frozen",
    "--keep-going",
    "--quiet",
    "-q",
    "--verbose",
    "-v",
    "-vv",
    "--future-incompat-report",
    "--doc",
}
NEXTEST_VALUE = {
    "-E",
    "--filter-expr",
    "--profile",
    "-P",
    "--config-file",
    "--tool-config-file",
    "--list-type",
    "--status-level",
    "--final-status-level",
    "--failure-output",
    "--success-output",
    "--test-threads",
    "--retries",
    "--partition",
    "--archive-file",
    "--binaries-metadata",
    "--cargo-metadata",
    "--workspace-remap",
    "--max-fail",
    "--run-ignored",
    "--no-tests",
}
NEXTEST_FLAG = {
    "--no-fail-fast",
    "--fail-fast",
    "--no-capture",
    "--nocapture",
    "--ignored",
    "--include-ignored",
    "--run-ignored",
    "--no-tests",
    "--ignore-default-filter",
    "--hide-progress-bar",
    "--no-run",
    "--no-deps",
    "--exact",
    "--list",
}


def graph_command(command: Sequence[str]) -> list[str]:
    if not command or command[0] != "cargo":
        raise ValidationError("correctness composition requires a cargo command")
    coverage = command[1:3] == ["llvm-cov", "nextest"]
    if coverage:
        command = ["cargo", "nextest", "run", *command[3:]]
    nextest = command[1:2] == ["nextest"]
    if nextest:
        if command[2:3] not in (["run"], ["list"]):
            raise ValidationError("only nextest run/list scopes are supported")
        mode, args = "test", command[3:]
    else:
        if command[1:2] == ["miri"] and command[2:3] == ["test"]:
            mode, args = "test", command[3:]
        elif command[1:2] in (
            ["check"],
            ["test"],
            ["build"],
            ["bench"],
            ["clippy"],
            ["run"],
        ):
            mode, args = command[1], command[2:]
        else:
            raise ValidationError("unsupported Cargo correctness command")
        if mode in {"clippy", "run"}:
            mode = "check" if mode == "clippy" else "build"
    # Pinned Cargo run --unit-graph panics after producing no executable. A build
    # graph is exact only when the run target is explicit; never guess default-run.
    if command[1:2] == ["run"] and not any(
        arg.split("=", 1)[0] in {"--bin", "--example"} for arg in args
    ):
        raise ValidationError(
            "run correctness scope requires an explicit --bin or --example target"
        )
    result = ["cargo", mode, "--unit-graph", "-Z", "unstable-options"]
    index = 0
    while index < len(args):
        arg = args[index]
        if arg == "--":
            break
        key = arg.split("=", 1)[0]
        if (
            key in VALUE
            or key in {"--cargo-profile", "--cargo-message-format"}
            or (key == "--profile" and not nextest)
        ):
            value = arg.split("=", 1)[1] if "=" in arg else None
            if value is None:
                index += 1
                if index == len(args):
                    raise ValidationError(f"missing value for {arg}")
                value = args[index]
            result.extend(
                [
                    {
                        "--cargo-profile": "--profile",
                        "--cargo-message-format": "--message-format",
                    }.get(key, key),
                    value,
                ]
            )
        elif arg in FLAG:
            result.append(arg)
        elif nextest and arg in {"--cargo-quiet", "--cargo-verbose"}:
            result.append("--quiet" if arg == "--cargo-quiet" else "--verbose")
        elif coverage and key in {"--output-path", "--ignore-filename-regex"}:
            if "=" not in arg:
                index += 1
                if index == len(args):
                    raise ValidationError(f"missing value for {arg}")
        elif coverage and arg in {
            "--lcov",
            "--html",
            "--text",
            "--json",
            "--summary-only",
            "--branch",
        }:
            pass
        elif nextest and key in NEXTEST_VALUE:
            if "=" not in arg:
                index += 1
                if index == len(args):
                    raise ValidationError(f"missing value for {arg}")
        elif arg in NEXTEST_FLAG:
            pass
        elif (
            arg.startswith(("-p", "-F", "-j"))
            and not arg.startswith("--")
            and len(arg) > 2
        ):
            result.append(arg)
        elif arg.startswith("-"):
            raise ValidationError(
                f"unsupported scope option {arg}; cannot infer an unvalidated closure"
            )
        # Positional test-name filters do not select a Cargo compilation target.
        index += 1
    return result


def capture(command: Sequence[str]) -> bytes:
    response = subprocess.run(command, capture_output=True, check=False)
    if response.returncode:
        raise ValidationError(
            f"{shlex.join(command)} failed:\n{response.stderr.decode()}"
        )
    return response.stdout


def reached(
    graph: Graph, packages: dict[str, Package], roots: Sequence[int]
) -> set[int]:
    pending, seen = list(roots), set()
    while pending:
        index = pending.pop()
        if index in seen:
            continue
        if index < 0 or index >= len(graph.units):
            raise ValidationError("invalid Cargo unit dependency index")
        unit = graph.units[index]
        if unit.pkg_id not in packages:
            raise ValidationError(f"unresolved Cargo unit package {unit.pkg_id}")
        if packages[unit.pkg_id].name == "pse-workspace-hack":
            continue
        seen.add(index)
        pending.extend(edge.index for edge in unit.dependencies)
    return seen


def arrow(package: Package) -> bool:
    return package.name == "arrow" or package.name.startswith("arrow-")


def with_features(command: Sequence[str], features: Sequence[str]) -> list[str]:
    result = list(command)
    if features:
        position = result.index("--") if "--" in result else len(result)
        result[position:position] = ["--features", ",".join(features)]
    return result


class Resolution(msgspec.Struct):
    command: list[str]
    packages: list[str]


def resolve(
    command: Sequence[str],
    *,
    run: Callable[[Sequence[str]], bytes] = capture,
) -> Resolution:
    graph_args = graph_command(command)
    metadata = msgspec.json.decode(
        run(
            [
                "cargo",
                "metadata",
                "--format-version",
                "1",
                "--locked",
                "--all-features",
                *manifest_options(graph_args),
            ]
        ),
        type=Metadata,
    )
    packages = {package.id: package for package in metadata.packages}
    original = msgspec.json.decode(run(graph_args), type=Graph)
    if original.version != 1:
        raise ValidationError("unsupported Cargo unit-graph version")
    features: set[str] = set()
    for root in original.roots:
        consumed = reached(original, packages, [root])
        if not any(arrow(packages[original.units[index].pkg_id]) for index in consumed):
            continue
        package = packages[original.units[root].pkg_id]
        opt_in = "force_validate" if arrow(package) else "force-validate"
        if opt_in not in package.features:
            raise ValidationError(
                f"{package.name}: actual Arrow closure has no explicit {opt_in} propagation"
            )
        features.add(f"{package.name}/{opt_in}")
    result = with_features(command, sorted(features))
    validated = original
    if features:
        validated = msgspec.json.decode(run(graph_command(result)), type=Graph)
    if validated.version != 1:
        raise ValidationError("unsupported Cargo unit-graph version")
    before = {root_identity(original.units[index]) for index in original.roots}
    after = {root_identity(validated.units[index]) for index in validated.roots}
    if before != after:
        raise ValidationError("validation changed selected package/target roots")
    for index in reached(validated, packages, validated.roots):
        unit = validated.units[index]
        package = packages[unit.pkg_id]
        if (
            arrow(package)
            and "force_validate" in package.features
            and "force_validate" not in unit.features
        ):
            raise ValidationError(
                f"{package.name} lacks force_validate in {unit.platform or 'host'} {unit.mode} unit; "
                "repair the selected owner's propagation"
            )
    names = sorted(
        {packages[original.units[index].pkg_id].name for index in original.roots}
    )
    return Resolution(result, names)


def compose(
    command: Sequence[str],
    *,
    run: Callable[[Sequence[str]], bytes] = capture,
) -> list[str]:
    return resolve(command, run=run).command


def root_identity(unit: Unit) -> tuple[str, str, tuple[str, ...], str, str | None, str]:
    return (
        unit.pkg_id,
        unit.target.name,
        tuple(unit.target.kind),
        unit.target.src_path,
        unit.platform,
        unit.mode,
    )


def manifest_options(args: Sequence[str]) -> list[str]:
    result = []
    for index, arg in enumerate(args):
        if arg in {"--manifest-path", "--config", "--lockfile-path"}:
            result.extend([arg, args[index + 1]])
        elif arg in {"--offline", "--frozen"}:
            result.append(arg)
    return result


def main(argv: Sequence[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    if args[:1] == ["--"]:
        args.pop(0)
    try:
        command = compose(["cargo", *args])
    except (ValidationError, msgspec.DecodeError) as error:
        print(f"arrow-validation: {error}", file=sys.stderr)
        return 2
    print("arrow-validation: " + shlex.join(command), file=sys.stderr, flush=True)
    return subprocess.call(command)


if __name__ == "__main__":
    raise SystemExit(main())
