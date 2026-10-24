# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Typed selections compose the assessment, recipe and artifact owners."""

from __future__ import annotations

from typing import TYPE_CHECKING, Annotated, Literal

from fastmcp.tools import Tool
from mcp.types import ToolAnnotations
from pydantic import ConfigDict, Field, TypeAdapter, model_validator

from scripts.capabilities import process
from scripts.capabilities.host_actions import HostResult, project
from scripts.capabilities.model import Capability, Contract
from scripts.capabilities.rust_tests import (  # noqa: TC001 -- FastMCP resolves request annotations.
    RustTestRequest,
    RustTestSelection,
)
from scripts.validation_scope import (
    EXCLUSIONS,
    FUNCTIONAL_SCOPES,
    GROUPS,
    Gate,
    comprehensive,
    expand,
    functional_gates,
)

if TYPE_CHECKING:
    from pathlib import Path

Group = Annotated[str, Field(json_schema_extra={"enum": sorted(GROUPS)})]
FunctionalScope = Annotated[
    str, Field(json_schema_extra={"enum": sorted(FUNCTIONAL_SCOPES)})
]
Package = Annotated[str, Field(pattern=r"^[A-Za-z0-9_][A-Za-z0-9_.-]*$")]
Feature = Annotated[str, Field(pattern=r"^[A-Za-z0-9_][A-Za-z0-9_./-]*$")]
GATES = TypeAdapter(list[Gate])


class AssessmentSelection(Contract):
    model_config = ConfigDict(
        extra="forbid", strict=True, frozen=True, validate_default=True
    )

    group: Group | None = None
    functional_scopes: list[FunctionalScope] | None = Field(default=None, min_length=1)
    python_profile: Literal["dev", "producer"] = "dev"

    @model_validator(mode="after")
    def selected_scope(self) -> AssessmentSelection:
        if self.group is not None and self.group not in GROUPS:
            raise ValueError("select a declared assessment group")
        if self.group is not None and self.functional_scopes is not None:
            raise ValueError("select either a group or functional scopes")
        if self.functional_scopes is not None:
            if set(self.functional_scopes) - FUNCTIONAL_SCOPES.keys():
                raise ValueError("select declared functional scopes")
            if "native" in self.functional_scopes and set(self.functional_scopes) - {
                "native",
                "managed-primary",
            }:
                raise ValueError(
                    "native covers ordinary scopes; combine it only with managed-primary"
                )
        return self

    def gates(self) -> list[Gate]:
        if self.functional_scopes is not None:
            return functional_gates(self.functional_scopes)
        if self.group is not None:
            return expand((self.group,))
        return comprehensive(self.python_profile)


class AssessmentRequest(AssessmentSelection):
    output: str = Field(min_length=1)
    reuse_from: str | None = Field(default=None, min_length=1)
    reuse: list[str] = Field(default_factory=list)
    transfer: list[str] = Field(default_factory=list)
    change_reason: str | None = None

    @model_validator(mode="after")
    def retained_origin(self) -> AssessmentRequest:
        if (self.reuse or self.transfer) and self.reuse_from is None:
            raise ValueError("reuse and transfer require an origin report")
        names = {gate.name for gate in self.gates()}
        if set(self.reuse + self.transfer) - names:
            raise ValueError("reuse/transfer must select gates in this assessment")
        if set(self.reuse) & set(self.transfer):
            raise ValueError("a gate cannot be both reused and transferred")
        return self


class CompilationRequest(Contract):
    model_config = ConfigDict(
        extra="forbid", strict=True, frozen=True, validate_default=True
    )

    output: str = Field(min_length=1)
    package: Package | None = None
    target: Literal["all", "library", "test"] = "all"
    test: Package | None = None
    features: list[Feature] = Field(default_factory=list)
    default_features: bool = True

    @model_validator(mode="after")
    def selected_target(self) -> CompilationRequest:
        if self.target != "all" and self.package is None:
            raise ValueError("library/test selection requires an exact package")
        if (self.test is not None) != (self.target == "test"):
            raise ValueError("test selection requires exactly one named test target")
        return self

    def gates(self) -> list[Gate]:
        recipe = "check"
        args = []
        if self.package is not None:
            recipe = {
                "all": "check-package",
                "library": "check-library",
                "test": "check-test",
            }[self.target]
            args.append(self.package)
            if self.test is not None:
                args.append(self.test)
        if self.features:
            args.extend(("--features", ",".join(self.features)))
        if not self.default_features:
            args.append("--no-default-features")
        return [
            Gate(
                "compilation-check",
                args=tuple(args),
                recipe=recipe,
                input_scope="rust-product",
                mode="scope-aware-force-validation",
            )
        ]


class OutputRequest(Contract):
    model_config = ConfigDict(
        extra="forbid", strict=True, frozen=True, validate_default=True
    )

    output: str = Field(min_length=1)


def bindings(root: Path) -> tuple[tuple[Capability, ...], dict[str, Tool]]:
    def assessment_select(request: AssessmentSelection) -> HostResult:
        """Describe the actual declared assessment expansion and prerequisites without building, running, allocating output or probing services. Explicit empty functional selections are refused; omitted/null selection chooses the comprehensive local campaign."""
        from scripts import validation  # noqa: PLC0415

        gates = request.gates()
        return project(
            {
                "checks": GATES.dump_python(gates, mode="json"),
                "exclusions": EXCLUSIONS,
                "prerequisites": validation.preflight_kinds(gates),
                "executed": False,
            },
            "assessment_select",
        )

    async def assessment_run(request: AssessmentRequest) -> HostResult:
        """Execute the declared assessment in an explicit fresh native report directory. Preserves source/input coverage, check outcomes and original receipt handles. Native prerequisites and cancellation remain owned by the assessment; use assessment_read for detail. A request cancellation never establishes rollback or qualification."""
        return project(
            await process.run_worker(root, "qualification", "assessment", request),
            "assessment_run",
        )

    async def compilation_check(request: CompilationRequest) -> HostResult:
        """Compile the selected workspace/package/library/test closure through the existing scope-aware force-validation recipes. Does not execute product tests. Compiler allocation and environment come from the native owner; no arbitrary Cargo flags or job override."""
        return project(
            await process.run_worker(root, "qualification", "compilation", request),
            "compilation_check",
        )

    async def contracts_check(request: OutputRequest) -> HostResult:
        """Compare generated Rust/Python/schema contracts and reference docs through the declared codegen-contracts-check assessment. Preserves actual drift/failure and report handles; comparison does not refresh generated files."""
        return project(
            await process.run_worker(root, "qualification", "contracts-check", request),
            "contracts_check",
        )

    async def contracts_generate(request: OutputRequest) -> HostResult:
        """Publish generated Rust/Python contracts, canonical schema and reference docs through the existing codegen-contracts recipe. This changes generated outputs; no physical fixture generation, source edits, commit or product qualification is implied. Explicit fresh assessment output retains execution evidence."""
        return project(
            await process.run_worker(
                root, "qualification", "contracts-generate", request
            ),
            "contracts_generate",
        )

    async def documentation_build(request: OutputRequest) -> HostResult:
        """Build the documentation site's HTML and scoped search through the native docs recipe and docs environment. Replaces the local published site under docs/book; retains a fresh assessment report. Does not publish remotely or qualify scientific behavior."""
        return project(
            await process.run_worker(root, "qualification", "documentation", request),
            "documentation_build",
        )

    def rust_test_describe(request: RustTestSelection) -> HostResult:
        """Describe a selected package's unit or named integration test request through the native selector and normalizer. No build or enumeration occurs; the real catalogue and force-validation closure are obtained only when the scoped run prepares them."""
        from scripts.capabilities import rust_tests  # noqa: PLC0415

        return project(rust_tests.describe(request), "rust_test_describe")

    async def rust_test_run(request: RustTestRequest) -> HostResult:
        """Run one selected package's unit or named integration tests through the native selector, normalized invocation, force-validation preparation and terminal owner. Returns the authentic invocation, actual catalogue and separate native terminal/drain evidence; borrowed invocations retain their outer reconciliation owner. Native features use the existing prepared caller environment. Protocol success never means the tests passed."""
        return project(
            await process.run_worker(root, "rust-tests", "rust-test", request),
            "rust_test_run",
        )

    rows = (
        (
            assessment_select,
            "validation",
            "qualification-assessment",
            "scripts/validation_scope.py",
            True,
        ),
        (
            assessment_run,
            "validation",
            "qualification-assessment",
            "scripts/validation.py",
            False,
        ),
        (compilation_check, "environment", "build-cache", "justfile", False),
        (contracts_check, "generation", "generated-contracts", "justfile", False),
        (contracts_generate, "generation", "generated-contracts", "justfile", False),
        (
            documentation_build,
            "governance",
            "documentation-lifecycle",
            "scripts/docs.py",
            False,
        ),
        (
            rust_test_describe,
            "validation",
            "validation-selection",
            "scripts/select.py",
            True,
        ),
        (
            rust_test_run,
            "validation",
            "validation-selection",
            "scripts/test_run.py",
            False,
        ),
    )
    specs = []
    tools: dict[str, Tool] = {}
    for function, domain, family, source, readonly in rows:
        tool = Tool.from_function(
            function,
            annotations=ToolAnnotations(
                read_only_hint=readonly,
                destructive_hint=not readonly,
                idempotent_hint=readonly,
                open_world_hint=not readonly,
            ),
        )
        spec = Capability(
            id=tool.name,
            title=tool.name.replace("_", " "),
            description=tool.description or "",
            domain=domain,
            family=family,
            workflows=("qualify-change",),
            guides=("root", "qualification"),
            owner=source,
            source=source,
            exposure="tool",
            effects="Read declared selection only; no workload or prerequisites executed."
            if readonly
            else "Run the selected existing recipe/assessment with native admission and explicit retained output. Generation and documentation actions change their native output owners. No rollback, commit, push or scientific qualification follows from protocol success.",
        )
        tool.tags.update(spec.tags)
        specs.append(spec)
        tools[spec.id] = tool
    return tuple(specs), tools
