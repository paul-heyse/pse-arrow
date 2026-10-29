# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Legacy graph helpers for unmigrated frontend builders, pending their Plan58 cutover.

The public command entry point has retired; native correctness owns generic commands.

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
    from collections.abc import Callable, Mapping, Sequence

if not __package__:
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from scripts import host_admission


def rust_test_environment(environment: Mapping[str, str]) -> dict[str, str]:
    """Supply a finite libtest thread stack without changing native job policy."""
    selected = dict(environment)
    if "RUST_MIN_STACK" not in selected:
        policy = host_admission.policy()["test-runner"]
        if not isinstance(policy, dict):
            raise ValueError("Missing Rust test thread stack policy")
        mib = policy["rust_thread_stack_mib"]
        if type(mib) is not int or mib <= 0:
            raise ValueError(
                "Rust test thread stack policy must be a positive integer MiB"
            )
        selected["RUST_MIN_STACK"] = str(mib * (1 << 20))
    value = selected["RUST_MIN_STACK"]
    if (
        not value
        or any(character not in "0123456789" for character in value)
        or not 0 < int(value) <= sys.maxsize * 2 + 1
    ):
        raise ValueError("RUST_MIN_STACK must be a positive finite byte count")
    return selected


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
        from scripts import test_invocation  # noqa: PLC0415 -- selected scope owner

        # Reporting flags belong to cargo-llvm-cov, not the selected Nextest
        # closure. Interpret a copy; compose retains the original execution argv.
        selected: list[str] = []
        arguments = command[3:]
        index = 0
        while index < len(arguments):
            argument = arguments[index]
            if argument == "--":
                selected.extend(arguments[index:])
                break
            name, separator, value = argument.partition("=")
            if name in {"--output-path", "--ignore-filename-regex"}:
                if separator:
                    if not value:
                        raise ValidationError(f"missing value for {name}")
                else:
                    index += 1
                    if index == len(arguments) or arguments[index] == "--":
                        raise ValidationError(f"missing value for {name}")
            elif argument not in {"--json", "--branch"}:
                selected.append(argument)
                scoped_name = test_invocation.ALIASES.get(name, name)
                if not separator and scoped_name in (
                    test_invocation.CARGO_VALUE
                    | test_invocation.RUN_VALUE
                    | test_invocation.SELECT_VALUE
                ):
                    index += 1
                    if index == len(arguments):
                        raise ValidationError(f"missing value for {name}")
                    selected.append(arguments[index])
            index += 1
        command = ["cargo", "nextest", "run", *selected]
    nextest = command[1:2] == ["nextest"]
    if nextest:
        if command[2:3] not in (["run"], ["list"]):
            raise ValidationError("only nextest run/list scopes are supported")
        # selected Arrow-closure owner; do not prepare native tests on import
        from scripts import test_invocation  # noqa: PLC0415

        try:
            return test_invocation.rust(command[3:]).graph()
        except ValueError as error:
            raise ValidationError(str(error)) from error
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


def capture(command: Sequence[str], *, env: Mapping[str, str] | None = None) -> bytes:
    response = subprocess.run(command, capture_output=True, check=False, env=env)
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
    graph: Graph
    metadata: bytes
    unit_graph: bytes


def resolve(
    command: Sequence[str],
    *,
    run: Callable[[Sequence[str]], bytes] = capture,
) -> Resolution:
    graph_args = graph_command(command)
    metadata_bytes = run(
        [
            "cargo",
            "metadata",
            "--format-version",
            "1",
            "--locked",
            "--all-features",
            *manifest_options(graph_args),
        ]
    )
    metadata = msgspec.json.decode(metadata_bytes, type=Metadata)
    packages = {package.id: package for package in metadata.packages}
    original_bytes = run(graph_args)
    original = msgspec.json.decode(original_bytes, type=Graph)
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
        if opt_in not in original.units[root].features:
            features.add(f"{package.name}/{opt_in}")
    result = with_features(command, sorted(features))
    validated = original
    validated_bytes = original_bytes
    if features:
        validated_bytes = run(graph_command(result))
        validated = msgspec.json.decode(validated_bytes, type=Graph)
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
    return Resolution(result, names, validated, metadata_bytes, validated_bytes)


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


if __name__ == "__main__":
    print(
        "arrow-validation: the Python command entry point is retired; use "
        "scripts/cargo-bin/cargo-pse-correctness. Unmigrated frontend builders remain "
        "outside the accepted Plan58 native scope.",
        file=sys.stderr,
    )
    raise SystemExit(125)
