# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Fixed child-module declarations shared by dispatch and loaded-source fencing."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING, Literal

if TYPE_CHECKING:
    from collections.abc import Mapping


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
    environment_selection: Literal["fixed", "installed-extension"] = "fixed"

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
            and (
                self.read_eligibility is None
                or binding.read_eligibility == self.read_eligibility
            )
            and (
                self.prepare_import is None
                or binding.prepare_import is self.prepare_import
            )
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
        environment_selection="installed-extension",
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
        environment_selection="installed-extension",
    ),
}


def selected_worker_environment(
    worker: str, operation: str, environment: Mapping[str, str]
) -> tuple[str, ...]:
    """Select only the environment policy declared for this fixed worker route."""
    declaration = WORKERS[worker]
    if declaration.environment_selection == "fixed":
        return declaration.environment
    from scripts import python_extension  # noqa: PLC0415 -- selected native route only
    from scripts.capabilities.native_contracts import (  # noqa: PLC0415
        binding_for,
    )
    from scripts.capabilities.native_extension import (  # noqa: PLC0415 -- receipt read only
        ExtensionSelectionError,
        selected_extension,
    )

    requirement = binding_for(operation).extension_requirement
    if requirement is None or requirement.validated is not True:
        raise ExtensionSelectionError("missing_owned_correctness_requirement")
    # Environment preparation is permitted only by the actual installed image,
    # even if a future operation requires a stronger correctness mode.
    selection = selected_extension(
        python_extension.bind_interpreter(environment),
        "installed_correctness",
        validated=requirement.validated,
    )
    if requirement.mode not in {"installed_correctness", "native_correctness"}:
        raise ExtensionSelectionError("unknown_owned_mode_requirement")
    if requirement.mode == "native_correctness" and selection.mode != "native":
        raise ExtensionSelectionError("installed_mode_conflicts_with_owner")
    return declaration.environment if selection.mode == "native" else ()
