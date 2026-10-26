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
    binding_family: str | None = None
    read_eligibility: str | None = None
    prepare_import: bool | None = None

    @property
    def operations(self) -> frozenset[str]:
        if self.declared_operations is not None:
            return self.declared_operations
        # Native bindings own their complete closed operation set. Resolve only
        # after module composition to avoid a second hand-maintained allowlist.
        from scripts.capabilities.native_contracts import (  # noqa: PLC0415 -- selected owner loads only at this boundary
            worker_bindings,
        )

        return frozenset(
            binding.operation
            for binding in worker_bindings()
            if binding.worker.worker == self.binding_family
            and (self.read_eligibility is None or binding.read_eligibility == self.read_eligibility)
            and (self.prepare_import is None or binding.prepare_import is self.prepare_import)
        )


_NATIVE_ENVIRONMENT = ("--native=solver,klu,isolation,uno,petsc",)


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
    "evidence": Worker(
        "scripts.capabilities.evidence_worker",
        None,
        sources=("xtask/src/bin/pse-evidence.rs",),
        binding_family="evidence",
    ),
    "native": Worker(
        "scripts.capabilities.native_worker",
        None,
        _NATIVE_ENVIRONMENT,
        binding_family="native",
    ),
    "native-read": Worker(
        "scripts.capabilities.native_read_worker",
        None,
        binding_family="native",
        read_eligibility="exact_context",
        prepare_import=False,
    ),
    "native-read-prepared": Worker(
        "scripts.capabilities.native_read_worker",
        None,
        _NATIVE_ENVIRONMENT,
        binding_family="native",
        read_eligibility="exact_context",
        prepare_import=True,
    ),
}
