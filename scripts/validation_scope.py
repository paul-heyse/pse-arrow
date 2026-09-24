# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Declared local validation scope; commands remain owned by the justfile."""

from dataclasses import dataclass


@dataclass(frozen=True)
class Gate:
    name: str
    args: tuple[str, ...] = ()
    report: str | None = None
    dependencies: tuple[str, ...] = ()
    role: str = "required"
    phase: str = "functional"
    enumerate_native: bool = False
    recipe: str | None = None
    mode: str = ""
    profile: str = ""


# Explicit classifications cover direct recipes as well as aggregate expansion.
PERFORMANCE_GATES = frozenset(
    {
        "bench-smoke",
        "bench-cache",
        "bench-production",
        "bench-consolidation",
        "bench-consolidation-native",
        "bench-recovery",
        "plan14-measure",
        "plan14-reviews",
    }
)


def declared(name: str) -> Gate:
    """Tool findings and unsupported capabilities retain their declared authority."""
    if name in {
        "audit-dependencies",
        "audit-advisories",
        "audit-shear",
        "audit-machete",
    }:
        return Gate(name, role="advisory")
    if name == "doc-lint":
        return Gate(name, role="deferred")
    if name in PERFORMANCE_GATES:
        return Gate(name, phase="performance")
    return Gate(name)


# Aggregates expand recursively, so both ordinary recipes and the full campaign
# have the same continuation behavior. Mutating setup remains dependency ordered.
GROUPS = {
    "fmt-check": ("fmt-rust-check", "lint-toml"),
    "clippy": ("clippy-default", "clippy-no-default"),
    "lint-repo": (
        "lint-toml",
        "lint-typos",
        "lint-license",
        "lint-actions",
        "lint-zizmor",
        "lint-shell",
        "lint-ast",
    ),
    "quality": (
        "python-contracts-check",
        "fmt-py-check",
        "lint-py",
        "typecheck",
        "lint-imports",
        "lint-repo",
        "lint-agents",
        "setup-test",
        "solver-pin-check",
    ),
    "adr-lint": ("adr-frontmatter-check", "adr-index-check", "register-lint"),
    "codegen-check": (
        "codegen-relations-check",
        "codegen-python-check",
        "codegen-docs-check",
        "codegen-bindgen-check",
    ),
    "codegen-contracts-check": (
        "codegen-rust-contracts-check",
        "codegen-python-check",
        "codegen-docs-check",
    ),
    "features-powerset": ("features-combinations", "features-no-default"),
    "governance": ("governance-tests", "codegen-check", "family-check"),
    "ci-fast": ("fmt-check", "check", "clippy", "test", "doctest"),
    "ci-pr": (
        "ci-fast",
        "governance",
        "docs-rust",
        "bench-smoke",
        "quality",
        "adr-lint",
        "docs",
        "py-test",
    ),
    "policy": ("audit-dependencies", "audit-advisories"),
    "deps-report": (
        "audit-dependencies",
        "audit-advisories",
        "audit-shear",
        "audit-machete",
    ),
}


def expand(names: tuple[str, ...]) -> list[Gate]:
    """Expand independent recipe groups, running each leaf once per invocation."""
    output: dict[str, Gate] = {}
    for name in names:
        for gate in expand(GROUPS[name]) if name in GROUPS else [declared(name)]:
            output.setdefault(gate.name, gate)
    return list(output.values())


def comprehensive(phase: str = "functional") -> list[Gate]:
    """Surviving general gates; target journeys and measurements are owned by M20."""
    if phase == "performance":
        return [
            Gate(
                "plan14-measure",
                ("{output}",),
                phase="performance",
                mode="native-force-validate",
                profile="performance-native",
            ),
            Gate(
                "plan14-reviews",
                ("{output}",),
                phase="performance",
                mode="native-force-validate",
                profile="performance-native",
            ),
        ]
    if phase != "functional":
        raise ValueError("unknown qualification phase")
    static = expand(
        (
            "fmt-check",
            "quality",
            "family-check",
            "codegen-check",
            "adr-lint",
            "register-check",
            "check",
            "clippy",
            "docs-rust",
            "docs",
            "deps-report",
            "unsafe-surface",
            "conformance-fixtures-check",
        )
    )
    static = [
        Gate(
            "setup-test",
            ("{output}",),
            "{output}/setup-test.xml",
            recipe="setup-test-report",
            mode="python-setup-unit",
        )
        if gate.name == "setup-test"
        else gate
        for gate in static
    ]
    gates = [
        Gate("py-sync-native"),
        *static,
        Gate("python-stubs", ("--check",), dependencies=("py-sync-native",)),
    ]
    gates.extend(
        Gate(
            name,
            ("--profile", "ci", "--success-output", "final"),
            "{target}/nextest/ci/junit.xml",
            enumerate_native=True,
            mode="pse-relations/force-validate",
            profile="rust-boundary",
        )
        for name in ("test", "governance-tests", "test-release")
    )
    gates.extend(
        (
            Gate("doctest"),
            Gate("doctest-release"),
            Gate(
                "inspection-fixture",
                ("{output}/inspection",),
                dependencies=("py-sync-native",),
            ),
        )
    )
    gates.extend(
        Gate(
            "assessment-python-" + kind,
            ("{output}",),
            "{output}/python-" + kind + ".xml",
            ("py-sync-native",)
            if kind == "unit"
            else ("py-sync-native", "inspection-fixture"),
            mode="python-" + kind,
            profile="python-" + kind,
        )
        for kind in ("unit", "component", "integration")
    )
    gates.extend(expand(("features-powerset",)))
    gates.append(
        Gate(
            "coverage", ("{output}/coverage",), "{output}/coverage/nextest/ci/junit.xml"
        )
    )
    gates.append(
        Gate(
            "plan14-tools",
            ("{output}",),
            "{output}/plan14-tools.xml",
            mode="python-tools",
            profile="python-tools",
        )
    )
    gates.append(
        Gate(
            "plan14-native",
            ("--profile", "ci", "--success-output", "final"),
            "{target}/nextest/ci/junit.xml",
            enumerate_native=True,
            mode="native-force-validate",
            profile="rust-native",
        )
    )
    gates.append(
        Gate(
            "plan14-python",
            ("{output}",),
            "{output}/plan14-python.xml",
            dependencies=("py-sync-native",),
            mode="python-native",
            profile="python-native",
        )
    )
    return gates


EXCLUSIONS = {
    "parity / parity-container": "Requires another Python environment and IDAES; excluded from current-environment assessment.",
    "solver-rebuild-check": "Qualifies a rebuilt container environment; solver execution routes are assigned to Plan 14 M11-M17/M20.",
    "udeps / MSRV / floors-latest": "Alternate toolchains and hypothetical dependency updates are outside current pinned-environment execution.",
    "gh-setup-check": "Remote CI/repository configuration is outside current-environment execution.",
    "wheels-check": "Dispatches the remote multi-platform release qualification workflow; not a local check.",
    "mutants-file": "Needs an explicit mutation target and campaign budget; no finite repository-wide target is declared by this recipe.",
    "platform matrix": "This campaign executes this host and its pinned Linux solver container; Windows/macOS and other Python versions require their own hosts.",
    "parameterized unit/check recipes": "Their complete test/compile scopes run through workspace gates; arbitrary filters are not separate test obligations.",
    "mutating and serving recipes": "Release, publish, repair, regeneration, upgrades and long-running servers are not validation checks.",
    "historical capability probes": "Evidence regeneration changes generated artifacts and external captures; current family/source checks are included.",
}


def development() -> list[Gate]:
    """Unit-only selections retain the same profiles and existing report collectors."""
    return [
        Gate(
            "plan14-native",
            ("rust-native", "{output}", "--profile", "ci", "--success-output", "final"),
            "{target}/nextest/ci/junit.xml",
            enumerate_native=True,
            recipe="plan14-development-run",
            phase="development",
            mode="native-force-validate",
            profile="rust-native",
        ),
        Gate(
            "plan14-python",
            ("python-native", "{output}", "--junitxml={output}/plan14-python.xml"),
            "{output}/plan14-python.xml",
            recipe="plan14-development-run",
            phase="development",
            mode="python-native",
            profile="python-native",
        ),
        Gate(
            "plan14-tools",
            ("python-tools", "{output}"),
            "{output}/plan14-tools.xml",
            recipe="plan14-development-run",
            phase="development",
            mode="python-tools",
            profile="python-tools",
        ),
    ]
