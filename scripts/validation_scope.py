# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Declared local validation scope; commands remain owned by the justfile."""

from dataclasses import dataclass, replace


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
        "case-measure",
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
        for declared_gate in (
            expand(GROUPS[name]) if name in GROUPS else [declared(name)]
        ):
            gate = declared_gate
            if name == "policy":
                gate = replace(gate, role="required")
            existing = output.get(gate.name)
            if existing is None or gate.role == "required":
                output[gate.name] = gate
    return list(output.values())


def comprehensive() -> list[Gate]:
    """Local qualification: default Rust, linked native Rust, and Python once."""
    static = expand(
        (
            "fmt-check",
            "clippy",
            "quality",
            "governance",
            "adr-lint",
            "check",
            "docs-rust",
            "docs",
            "conformance-fixtures-check",
        )
    )
    static = [
        replace(
            gate,
            args=("{output}",),
            report="{output}/setup-test.xml",
            recipe="setup-test-report",
            mode="python-setup-unit",
        )
        if gate.name == "setup-test"
        else gate
        for gate in static
    ]
    return [
        Gate("py-sync-native"),
        *static,
        Gate("python-stubs", ("--check",), dependencies=("py-sync-native",)),
        Gate(
            "test",
            ("--profile", "ci"),
            "{target}/nextest/ci/junit.xml",
            enumerate_native=True,
            mode="force-validate",
            profile="ci",
        ),
        Gate("doctest"),
        Gate(
            "native-test",
            ("--profile", "ci"),
            "{target}/nextest/ci/junit.xml",
            enumerate_native=True,
            mode="native-force-validate",
            profile="ci",
        ),
        Gate("inspection-fixture", ("{output}/inspection",)),
        Gate(
            "native-python",
            ("{output}",),
            "{output}/native-python.xml",
            dependencies=("py-sync-native", "inspection-fixture"),
            mode="python-native",
        ),
    ]


EXCLUSIONS = {
    "other platforms and distribution": "This command qualifies the local pinned environment; remote CI, wheels and other hosts have separate commands.",
    "release, coverage, feature powerset, alternate toolchains": "Available separately; not part of the default and native local profiles.",
    "parity": "Run parity-container separately when IDAES parity is in scope.",
    "performance": "Run case-measure after functional qualification, with an explicit functional report.",
    "reviews": "Architecture and scientific review are judgments recorded in the owning plan, not command-exit evidence.",
    "scheduled register checks": "register-check is time-dependent; deterministic adr-lint uses register-lint.",
    "dependency inventory": "deps-report is advisory; policy is separately available and strict.",
}
