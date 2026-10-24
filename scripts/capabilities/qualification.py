# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Typed selections compose the assessment, recipe and artifact owners."""

from __future__ import annotations

from typing import TYPE_CHECKING

from fastmcp.tools import Tool
from mcp.types import ToolAnnotations

from scripts.capabilities import process
from scripts.capabilities import rust_test_contracts as rust_tests
from scripts.capabilities.model import Capability, Prerequisite
from scripts.capabilities.responses import HostResult, project
from scripts.capabilities.rust_test_contracts import (  # noqa: TC001 -- FastMCP resolves request annotations.
    RustTestRequest,
    RustTestSelection,
)
from scripts.validation_scope import (
    EXCLUSIONS,
)

if TYPE_CHECKING:
    from pathlib import Path

from scripts.capabilities.qualification_contracts import (
    GATES,
    AssessmentRequest,
    AssessmentSelection,
    CompilationRequest,
    OutputRequest,
)
from scripts.capabilities.workers import WorkerCall

QUALIFICATION_CALLS = {
    "assessment_run": WorkerCall("qualification", "assessment"),
    "compilation_check": WorkerCall("qualification", "compilation"),
    "contracts_check": WorkerCall("qualification", "contracts-check"),
    "contracts_generate": WorkerCall("qualification", "contracts-generate"),
    "documentation_build": WorkerCall("qualification", "documentation"),
    "rust_test_run": WorkerCall("rust-tests", "rust-test"),
}


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
            await process.run_worker(
                root, QUALIFICATION_CALLS["assessment_run"], request
            ),
            "assessment_run",
        )

    async def compilation_check(request: CompilationRequest) -> HostResult:
        """Compile the selected workspace/package/library/test closure through the existing scope-aware force-validation recipes. Does not execute product tests. Compiler allocation and environment come from the native owner; no arbitrary Cargo flags or job override."""
        return project(
            await process.run_worker(
                root, QUALIFICATION_CALLS["compilation_check"], request
            ),
            "compilation_check",
        )

    async def contracts_check(request: OutputRequest) -> HostResult:
        """Compare generated Rust/Python/schema contracts and reference docs through the declared codegen-contracts-check assessment. Preserves actual drift/failure and report handles; comparison does not refresh generated files."""
        return project(
            await process.run_worker(
                root, QUALIFICATION_CALLS["contracts_check"], request
            ),
            "contracts_check",
        )

    async def contracts_generate(request: OutputRequest) -> HostResult:
        """Publish generated Rust/Python contracts, canonical schema and reference docs through the existing codegen-contracts recipe. This changes generated outputs; no physical fixture generation, source edits, commit or product qualification is implied. Explicit fresh assessment output retains execution evidence."""
        return project(
            await process.run_worker(
                root, QUALIFICATION_CALLS["contracts_generate"], request
            ),
            "contracts_generate",
        )

    async def documentation_build(request: OutputRequest) -> HostResult:
        """Build the documentation site's HTML and scoped search through the native docs recipe and docs environment. Replaces the local published site under docs/book; retains a fresh assessment report. Does not publish remotely or qualify scientific behavior."""
        return project(
            await process.run_worker(
                root, QUALIFICATION_CALLS["documentation_build"], request
            ),
            "documentation_build",
        )

    def rust_test_describe(request: RustTestSelection) -> HostResult:
        """Describe a selected package's unit or named integration test request through the native selector and normalizer. No build or enumeration occurs; the real catalogue and force-validation closure are obtained only when the scoped run prepares them."""
        return project(rust_tests.describe(request), "rust_test_describe")

    async def rust_test_run(request: RustTestRequest) -> HostResult:
        """Run one selected package's unit or named integration tests through the native selector, normalized invocation, force-validation preparation and terminal owner. Returns the authentic invocation, actual catalogue and separate native terminal/drain evidence; borrowed invocations retain their outer reconciliation owner. Native features use the existing prepared caller environment. If notifications are unavailable, rust_test_reference_read(output) exposes the original issued handle while this action remains attached. Protocol success never means the tests passed."""
        return project(
            await process.run_worker(
                root, QUALIFICATION_CALLS["rust_test_run"], request
            ),
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
            guides=("root", "qualification"),
            owner=source,
            source=source,
            exposure="tool",
            worker_calls=(QUALIFICATION_CALLS[tool.name],)
            if tool.name in QUALIFICATION_CALLS
            else (),
            requirements=()
            if readonly
            else (
                Prerequisite(
                    kind="checkout-environment",
                    owner="scripts/pse_env.py",
                    preparation=("environment_explain",),
                ),
                Prerequisite(
                    kind="executable", owner="scripts/pse_env.py", provider="just"
                ),
            ),
            effects="Read declared selection only; no workload or prerequisites executed."
            if readonly
            else "Run the selected existing recipe/assessment with native admission and explicit retained output. Generation and documentation actions change their native output owners. No rollback, commit, push or scientific qualification follows from protocol success.",
        )
        tool.tags.update(spec.tags)
        specs.append(spec)
        tools[spec.id] = tool
    return tuple(specs), tools
