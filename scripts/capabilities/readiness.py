# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Selected readiness observations, distinct from declared requirements and preparation."""

from __future__ import annotations

import os
import subprocess
from typing import TYPE_CHECKING, Literal

from fastmcp.exceptions import ToolError
from fastmcp.tools import Tool
from mcp.types import ToolAnnotations
from pydantic import Field, ValidationError, model_validator

from scripts import python_extension
from scripts.capabilities import preparation
from scripts.capabilities.diagnostics import refusal
from scripts.capabilities.model import Capability, Contract, Identifier, Prerequisite

if TYPE_CHECKING:
    from collections.abc import Callable, Mapping
    from pathlib import Path

    from scripts.capabilities.specialists import Specialists


class ReadinessFact(Contract):
    value: bool | None = None
    observation: Literal["unobserved", "passive", "live"] = "unobserved"
    evidence: str


class ReceiptObservation(Contract):
    stage: Literal["extension", "stubs"]
    path: str
    recorded: bool
    mode: python_extension.Mode | None = None
    profile: str | None = None
    validated: bool | None = None
    artifact: python_extension.ExtensionArtifact | None = None
    currency: Literal["unknown"] = "unknown"


class CapabilityReadiness(Contract):
    capability: str
    requirements: tuple[Prerequisite, ...]
    declared_requirements: ReadinessFact
    artifact_prepared: ReadinessFact
    import_compatible: ReadinessFact
    selected_service_available: ReadinessFact
    language_provider_available: ReadinessFact
    operation_exercised: ReadinessFact
    receipts: list[ReceiptObservation] = Field(default_factory=list)
    imported_artifact: python_extension.ExtensionArtifact | None = None


class ReadinessCheckRequest(Contract):
    capability: Identifier
    checks: list[Literal["import-compatible", "selected-service", "language-provider"]] = Field(min_length=1, max_length=3)
    language: Literal["python", "rust"] | None = None
    file_path: str | None = Field(default=None, min_length=1)
    mode: python_extension.Mode | None = None
    validated: bool | None = None

    @model_validator(mode="after")
    def distinct_checks(self) -> ReadinessCheckRequest:
        if len(self.checks) != len(set(self.checks)):
            raise ValueError("select each readiness check once")
        return self


def receipt_observation(root: Path, stage: Literal["extension", "stubs"]) -> ReceiptObservation:
    path = python_extension.stage_receipt(stage, os.environ, root=root)
    document = python_extension.read_receipt(path)
    artifact = None
    inputs = document.get("inputs")
    if isinstance(inputs, dict):
        try:
            artifact = python_extension.ExtensionArtifact.model_validate(document.get("artifact"))
        except ValidationError:
            pass
    if artifact is None or not isinstance(inputs, dict):
        return ReceiptObservation(stage=stage, path=str(path), recorded=False)
    mode, profile, validated = inputs.get("mode"), inputs.get("profile"), inputs.get("validated")
    if not isinstance(mode, str) or mode not in {"dev", "native"} or not isinstance(profile, str) or not isinstance(validated, bool):
        return ReceiptObservation(stage=stage, path=str(path), recorded=False)
    return ReceiptObservation(
        stage=stage, path=str(path), recorded=True, artifact=artifact,
        mode=mode if isinstance(mode, str) and mode in {"dev", "native"} else None,
        profile=profile if isinstance(profile, str) else None,
        validated=validated if isinstance(validated, bool) else None,
    )


def passive(root: Path, capability: Capability) -> CapabilityReadiness:
    requirements = capability.requirements
    extension = [item for item in requirements if item.kind == "python-extension"]
    receipts = [receipt_observation(root, "extension")] if extension else []
    recorded = any(
        receipt.recorded
        and receipt.stage == "extension"
        and all(
            (item.mode is None or receipt.mode == item.mode)
            and (item.validated is not True or receipt.validated is True)
            for item in extension
        )
        for receipt in receipts
    )
    return CapabilityReadiness(
        capability=capability.id, requirements=requirements,
        declared_requirements=ReadinessFact(
            value=True if requirements else None, observation="passive",
            evidence="Owner-declared prerequisites; no prerequisites executed." if requirements else "No structural requirements are declared for this binding; readiness remains unknown.",
        ),
        artifact_prepared=ReadinessFact(
            value=True if recorded else None, observation="passive" if extension else "unobserved",
            evidence="Existing owner receipt records a matching completed preparation; current input currency and import compatibility remain unknown." if recorded else "No matching completed preparation receipt observed; absence does not establish whether an artifact exists.",
        ),
        import_compatible=ReadinessFact(evidence="No selected live import capture performed."),
        selected_service_available=ReadinessFact(evidence="No selected live service check performed."),
        language_provider_available=ReadinessFact(evidence="No selected language/file support check performed."),
        operation_exercised=ReadinessFact(evidence="Readiness inspection does not exercise or qualify the selected operation."),
        receipts=receipts,
    )


def bindings(
    root: Path,
    *,
    capabilities: Callable[[], Mapping[str, Capability]],
    specialists: Specialists,
) -> tuple[tuple[Capability, ...], dict[str, Tool]]:
    def selected(identifier: str) -> Capability:
        result = capabilities().get(identifier)
        if result is None:
            raise refusal("pse.capabilities.unknown_capability")
        return result

    def capability_readiness(capability: Identifier) -> CapabilityReadiness:
        """Inspect one exact capability's declared requirements and existing preparation receipts without compiler probes, imports, service connections, builds or publication. Receipt presence records prior preparation; current currency, import compatibility, service availability and operation qualification remain unknown until separately observed."""
        return passive(root, selected(capability))

    async def capability_readiness_check(request: ReadinessCheckRequest) -> CapabilityReadiness:
        """Explicitly check only selected live prerequisites of one capability. Imports use an admitted fixed child in the inherited endpoint environment with actual byte/origin/features fences; selected toolkit service and language checks use their original specialist owner. Language checks need an explicit language and file_path. No build, installation, synchronization, native provider preparation or source publication. Request-specific stores remain unknown without their real request; a check never qualifies the selected operation."""
        capability = selected(request.capability)
        result = passive(root, capability)
        update: dict[str, object] = {}
        # Refuse inconsistent/missing selections before acquiring any live owner.
        if "selected-service" in request.checks and not any(item.kind == "selected-service" for item in capability.requirements):
            raise refusal("pse.readiness.service_selection_required")
        if "language-provider" in request.checks:
            language_requirements = [item for item in capability.requirements if item.kind == "language-provider"]
            if not language_requirements or request.language is None or request.file_path is None or any(item.provider is not None and item.provider != request.language for item in language_requirements):
                raise refusal("pse.readiness.language_selection_required")
        if "import-compatible" in request.checks:
            extensions = [item for item in capability.requirements if item.kind == "python-extension"]
            if len(extensions) != 1:
                raise refusal("pse.readiness.extension_selection_required")
            requirement = extensions[0]
            mode = requirement.mode or request.mode
            validated = request.validated if request.validated is not None else requirement.validated
            if mode is None or validated is None or (request.mode is not None and requirement.mode is not None and request.mode != requirement.mode) or (requirement.validated is True and request.validated is False):
                raise refusal("pse.readiness.extension_selection_required")
            try:
                imported = await preparation.import_check(root, mode, validated=validated)
            except (OSError, ValueError, RuntimeError, subprocess.SubprocessError, ToolError):
                update["import_compatible"] = ReadinessFact(observation="live", evidence="The selected import check did not return an owner observation; environment or transport refusal does not establish imported compatibility.")
            else:
                update["import_compatible"] = ReadinessFact(value=imported.compatible, observation="live", evidence="Actual selected-interpreter import origin, bytes and requested features checked by python_extension.capture." if imported.compatible else "The actual selected import capture refused compatibility; no preparation was attempted.")
                update["imported_artifact"] = imported.artifact
        if "selected-service" in request.checks:
            services = [item for item in capability.requirements if item.kind == "selected-service"]
            if not services:
                raise refusal("pse.readiness.service_selection_required")
            if all(item.provider == "surreal-selected" for item in services):
                try:
                    await specialists.select_database()
                except (OSError, ValueError, RuntimeError, subprocess.SubprocessError):
                    update["selected_service_available"] = ReadinessFact(value=False, observation="live", evidence="The existing selected toolkit service was unavailable to its original owner; no service startup was attempted.")
                else:
                    update["selected_service_available"] = ReadinessFact(value=True, observation="live", evidence="Original specialist owner checked its existing exact toolkit service generation; no request-specific native store is inferred.")
            else:
                update["selected_service_available"] = ReadinessFact(evidence="This prerequisite names a request-specific service; toolkit defaults cannot establish its availability without the actual request.")
        if "language-provider" in request.checks:
            languages = [item for item in capability.requirements if item.kind == "language-provider"]
            if not languages or request.language is None or request.file_path is None or any(item.provider is not None and item.provider != request.language for item in languages):
                raise refusal("pse.readiness.language_selection_required")
            try:
                support = await specialists.language(request.language, "get_tool_support", {"file_path": request.file_path})
                native = support.get("native")
                available = isinstance(native, dict) and native.get("isError") is False
            except (OSError, ValueError, RuntimeError, subprocess.SubprocessError):
                available = False
            update["language_provider_available"] = ReadinessFact(value=available, observation="live", evidence="Original specialist owner checked the explicitly selected language/file support; this does not exercise the requested language operation.")
        return result.model_copy(update=update)

    specs = []
    tools = {}
    for function, read_only in ((capability_readiness, True), (capability_readiness_check, False)):
        tool = Tool.from_function(function, annotations=ToolAnnotations(read_only_hint=read_only, destructive_hint=False, idempotent_hint=read_only, open_world_hint=False))
        spec = Capability(
            id=tool.name, title=tool.name.replace("_", " ").title(), description=tool.description or tool.name,
            domain="environment", guides=("environment", "python"), owner="scripts/capabilities/readiness.py", source="scripts/capabilities/readiness.py",
            exposure="tool", worker_calls=() if read_only else tuple(preparation.IMPORT_CALLS.values()),
            effects="Read declared requirements and existing owner receipt metadata only." if read_only else "Explicit selected live import or specialist prerequisite checks; no extension build, source synchronization or stub publication.",
        )
        tool.tags.update(spec.tags)
        specs.append(spec)
        tools[spec.id] = tool
    return tuple(specs), tools
