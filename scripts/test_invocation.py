# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Pure interpretation shared by test discovery, preparation and execution.

This is the native runner boundary, not an agent command catalogue. Performing
owners consume these values; exposing an operation does not grant its effects.
"""

from __future__ import annotations

import os
from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING

import tomlkit

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence


CARGO_VALUE = frozenset({
    "--package", "--exclude", "--features", "--target", "--target-dir",
    "--manifest-path", "--config", "--bin", "--test", "--bench", "--example",
    "--build-jobs", "--cargo-profile", "--cargo-message-format", "--lockfile-path", "-Z",
})
CARGO_FLAG = frozenset({
    "--workspace", "--all", "--lib", "--bins", "--tests", "--benches", "--examples",
    "--all-targets", "--no-default-features", "--all-features", "--release", "--locked",
    "--offline", "--frozen", "--keep-going", "--cargo-quiet", "--cargo-verbose",
    "--ignore-rust-version", "--future-incompat-report", "--unit-graph", "--timings",
})
RUN_VALUE = frozenset({
    "--status-level", "--final-status-level", "--failure-output", "--success-output",
    "--test-threads", "--retries", "--max-fail", "--no-tests", "--flaky-result",
    "--stress-count", "--stress-duration", "--debugger", "--tracer",
    "--max-progress-running", "--message-format", "--message-format-version",
})
RUN_FLAG = frozenset({
    "--no-fail-fast", "--fail-fast", "--no-capture", "--no-output-indent",
    "--hide-progress-bar", "--no-input-handler", "--no-run",
})
SELECT_VALUE = frozenset({
    "--filter-expr", "--profile", "--config-file", "--tool-config-file", "--list-type",
    "--partition", "--archive-file", "--binaries-metadata", "--cargo-metadata",
    "--workspace-remap", "--target-dir-remap", "--run-ignored", "--skip", "--color",
})
SELECT_FLAG = frozenset({
    "--ignored", "--include-ignored", "--ignore-default-filter", "--no-deps", "--exact",
    "--list", "--verbose", "--quiet",
})
ALIASES = {
    "-p": "--package", "-F": "--features", "-P": "--profile", "-E": "--filter-expr",
    "--filterset": "--filter-expr", "-r": "--release", "-j": "--test-threads",
    "--jobs": "--test-threads", "--ff": "--fail-fast", "--nff": "--no-fail-fast",
    "--nocapture": "--no-capture", "-v": "--verbose", "-q": "--quiet",
}


@dataclass(frozen=True)
class Argument:
    name: str
    value: str | None = None
    original: tuple[str, ...] = ()

    def argv(self) -> list[str]:
        return list(self.original) if self.original else [self.name] if self.value is None else [self.name, self.value]


@dataclass(frozen=True)
class RustInvocation:
    options: tuple[Argument, ...]
    opaque: tuple[str, ...]
    nextest_profile: str

    def arguments(self, *, omit: frozenset[str] = frozenset()) -> list[str]:
        return [part for item in self.options if item.name not in omit for part in item.argv()] + list(self.opaque)

    def values(self, name: str) -> tuple[str, ...]:
        return tuple(item.value for item in self.options if item.name == name and item.value is not None)

    @property
    def cargo_profile(self) -> str:
        profiles = self.values("--cargo-profile")
        return profiles[-1] if profiles else "release" if any(item.name == "--release" for item in self.options) else "dev"

    def listing(self) -> list[str]:
        return self.arguments(omit=RUN_VALUE | RUN_FLAG)

    def preparation(self, features: str = "") -> tuple[list[str], list[str]]:
        execution = self.arguments(omit=CARGO_VALUE | CARGO_FLAG)
        selected_features = ([features] if features else []) + list(self.values("--features"))
        metadata = ["--features", ",".join(selected_features)] if selected_features else []
        for item in self.options:
            if item.name in {"--manifest-path", "--config", "-Z", "--lockfile-path"}:
                metadata.extend(item.argv())
            elif item.name == "--target":
                metadata.extend(["--filter-platform", str(item.value)])
            elif item.name in {"--all-features", "--no-default-features", "--frozen", "--offline"}:
                metadata.append(item.name)
        return execution, metadata

    def graph(self) -> list[str]:
        result = ["cargo", "test", "--unit-graph", "-Z", "unstable-options"]
        aliases = {"--cargo-profile": "--profile", "--build-jobs": "--jobs", "--cargo-message-format": "--message-format", "--cargo-quiet": "--quiet", "--cargo-verbose": "--verbose"}
        for item in self.options:
            if item.name in CARGO_VALUE | CARGO_FLAG:
                result.append(aliases.get(item.name, item.name))
                if item.value is not None:
                    result.append(item.value)
        return result


def rust(arguments: Sequence[str], environment: Mapping[str, str] | None = None) -> RustInvocation:
    """Interpret supported nextest options before any performing operation."""
    environment = os.environ if environment is None else environment
    options: list[Argument] = []
    opaque: tuple[str, ...] = ()
    index = 0
    while index < len(arguments):
        start = index
        argument = arguments[index]
        if argument == "--":
            opaque = tuple(arguments[index:])
            break
        name, separator, value = argument.partition("=")
        if not separator and len(argument) > 2 and argument[:2] in {"-p", "-F", "-P", "-E", "-j", "-Z"}:
            name, separator, value = argument[:2], "=", argument[2:]
        name = ALIASES.get(name, name)
        if name in CARGO_VALUE | RUN_VALUE | SELECT_VALUE:
            if not separator:
                index += 1
                if index == len(arguments) or arguments[index] == "--":
                    raise ValueError(f"{name} requires a value")
                value = arguments[index]
            if not value:
                raise ValueError(f"{name} requires a nonempty value")
            options.append(Argument(name, value, tuple(arguments[start:index + 1])))
        elif name in CARGO_FLAG | RUN_FLAG | SELECT_FLAG:
            if separator:
                raise ValueError(f"{name} does not accept a value")
            options.append(Argument(name, original=(argument,)))
        elif argument.startswith("-"):
            raise ValueError(f"unsupported scope option {argument}; cannot infer selected work")
        else:
            options.append(Argument(argument))
        index += 1
    request = RustInvocation(tuple(options), opaque, "default")
    profiles = request.values("--profile")
    if len(profiles) > 1 or len(request.values("--config-file")) > 1:
        raise ValueError("profile and repository configuration must each have one effective selection")
    if request.values("--cargo-profile") and any(item.name == "--release" for item in options):
        raise ValueError("--release conflicts with --cargo-profile")
    profile = profiles[0] if profiles else environment.get("NEXTEST_PROFILE", "default")
    if not profile:
        raise ValueError("NEXTEST_PROFILE must be nonempty")
    return RustInvocation(tuple(options), opaque, profile)


def report_configuration(configuration: Path, destination: Path, profile: str, report: Path) -> Path:
    """Copy repository configuration, editing only the selected report path.

    Tool configurations stay separate lower-priority inputs to nextest. Their
    profiles/defaults are neither flattened nor promoted into repository policy.
    """
    document = tomlkit.parse(configuration.read_text() if configuration.exists() else "")
    profiles = document.setdefault("profile", tomlkit.table())
    selected = profiles.setdefault(profile, tomlkit.table())
    junit = selected.setdefault("junit", tomlkit.table())
    junit["path"] = str(report.absolute())
    destination.write_text(tomlkit.dumps(document))
    return destination


PYTHON_VALUE = frozenset({
    "-m", "--markexpr", "-k", "-n", "--numprocesses", "--dist", "-o", "--override-ini",
    "-c", "--config-file", "--rootdir", "--basetemp", "--junitxml", "--junit-xml",
    "--maxfail", "--tb", "--capture", "--import-mode", "--log-level", "--log-cli-level",
    "--color", "--durations", "--durations-min", "--deselect", "--ignore", "--ignore-glob",
    "--confcutdir", "-p", "--cov", "--cov-report", "--cov-fail-under",
    "--terminal-owner", "--max-worker-restart", "--junit-prefix", "-r",
})
PYTHON_FLAG = frozenset({
    "-q", "-qq", "-v", "-vv", "-s", "-x", "--exitfirst", "--collect-only", "--collectonly",
    "--continue-on-collection-errors", "--strict-markers", "--strict-config", "--disable-warnings",
    "--no-header", "--no-summary", "--lf", "--last-failed", "--ff", "--failed-first",
    "--performance", "--parity", "--producer-deployment", "--unit-only", "--no-cov",
    "--managed-primary-route", "--managed-primary-child", "--functional-observer-child", "--pyargs",
})


@dataclass(frozen=True)
class PythonInvocation:
    marker: str
    forwarded: tuple[str, ...]
    opaque: tuple[str, ...]
    targets: tuple[str, ...]
    options: tuple[Argument, ...]

    def values(self, name: str) -> tuple[str, ...]:
        return tuple(item.value for item in self.options if item.name == name and item.value is not None)

    def arguments(self, *, omit: frozenset[str] = frozenset()) -> list[str]:
        return [part for item in self.options if item.name not in omit for part in item.argv()] + list(self.opaque)


def append_options(arguments: Sequence[str], options: Sequence[str]) -> list[str]:
    position = arguments.index("--") if "--" in arguments else len(arguments)
    return [*arguments[:position], *options, *arguments[position:]]


def python(arguments: Sequence[str]) -> PythonInvocation:
    """Validate the supported pytest adapter before preparation, without collecting."""
    marker = "unit or component or integration"
    forwarded: list[str] = []
    targets: list[str] = []
    opaque: tuple[str, ...] = ()
    options: list[Argument] = []
    index = 0
    while index < len(arguments):
        argument = arguments[index]
        if argument == "--":
            opaque = tuple(arguments[index:])
            targets.extend(arguments[index + 1:])
            break
        start = index
        name, separator, value = argument.partition("=")
        if not separator and len(argument) > 2 and argument[:2] in {"-m", "-k", "-n", "-o", "-c", "-p", "-r"}:
            name, separator, value = argument[:2], "=", argument[2:]
        if name in PYTHON_VALUE:
            if not separator:
                index += 1
                if index == len(arguments) or arguments[index] == "--":
                    raise ValueError(f"pytest {name} requires a value")
                value = arguments[index]
            if name in {"-m", "--markexpr"}:
                marker = value
            else:
                forwarded.extend(arguments[start:index + 1])
            options.append(Argument("--junitxml" if name == "--junit-xml" else name, value, tuple(arguments[start:index + 1])))
        elif name in PYTHON_FLAG:
            if separator:
                raise ValueError(f"pytest {name} does not accept a value")
            forwarded.append(argument)
            options.append(Argument(name, original=(argument,)))
        elif argument.startswith("-"):
            raise ValueError(f"unsupported pytest scope option {argument}")
        else:
            forwarded.append(argument)
            targets.append(argument)
            options.append(Argument(argument))
        index += 1
    return PythonInvocation(marker, tuple(forwarded), opaque, tuple(targets), tuple(options))
