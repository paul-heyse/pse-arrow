# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Fixed child-module declarations shared by dispatch and loaded-source fencing."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Literal


@dataclass(frozen=True)
class WorkerCall:
    """One binding's actual fixed child and lifetime, shared by dispatch and fencing."""

    worker: str
    operation: str
    lifetime: Literal["attached", "detached"] = "attached"

    @property
    def declaration(self) -> Worker:
        if self.worker not in WORKERS:
            raise ValueError("unsupported fixed worker operation")
        return WORKERS[self.worker]


@dataclass(frozen=True)
class Worker:
    module: str
    declared_operations: frozenset[str] | None
    environment: tuple[str, ...] = ()
    sources: tuple[str, ...] = ()
    launch: Literal["environment", "inherited"] = "environment"

    @property
    def operations(self) -> frozenset[str]:
        if self.declared_operations is not None:
            return self.declared_operations
        # Native bindings own their complete closed operation set. Resolve only
        # after module composition to avoid a second hand-maintained allowlist.
        from scripts.capabilities.native_contracts import BINDINGS
        return frozenset(binding.operation for binding in BINDINGS)


WORKERS = {
    "prep-import": Worker(
        "scripts.capabilities.preparation_worker",
        frozenset({"import-check", "currency-check"}),
        launch="inherited",
    ),
    "prep-dev": Worker(
        "scripts.capabilities.preparation_worker",
        frozenset({"stage"}),
        ("--recipe-default-class", "compile"),
    ),
    "prep-native": Worker(
        "scripts.capabilities.preparation_worker",
        frozenset({"stage"}),
        (
            "--recipe-default-class",
            "compile",
            "--native=solver,klu,isolation,uno,petsc",
        ),
    ),
    "host": Worker(
        "scripts.capabilities.host_worker",
        frozenset({"archive", "replay", "diagnostic"}),
    ),
    "qualification": Worker(
        "scripts.capabilities.qualification_worker",
        frozenset(
            {
                "assessment",
                "compilation",
                "contracts-check",
                "contracts-generate",
                "documentation",
            }
        ),
        sources=("justfile",),
    ),
    "rust-tests": Worker(
        "scripts.capabilities.qualification_worker",
        frozenset({"rust-test"}),
        (
            "--recipe-default-class",
            "exclusive",
            "--native=solver,klu,isolation,uno,petsc",
        ),
    ),
    "native": Worker(
        "scripts.capabilities.native_worker",
        None,
        ("--native=solver,klu,isolation,uno,petsc",),
    ),
}
