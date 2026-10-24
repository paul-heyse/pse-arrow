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
    operations: frozenset[str]
    environment: tuple[str, ...] = ()


WORKERS = {
    "prep-dev": Worker(
        "scripts.capabilities.preparation_worker",
        frozenset({"stage", "import-check"}),
        ("--recipe-default-class", "compile"),
    ),
    "prep-native": Worker(
        "scripts.capabilities.preparation_worker",
        frozenset({"stage", "import-check"}),
        ("--recipe-default-class", "compile", "--native=solver,klu,isolation,uno,petsc"),
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
        frozenset({"source_admit", "preparation_discover"}),
        ("--native=solver,klu,isolation,uno,petsc",),
    ),
}
