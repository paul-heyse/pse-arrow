# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit Python preparation bindings; the native stage owner retains all decisions."""

from __future__ import annotations

from typing import TYPE_CHECKING

from fastmcp.tools import Tool
from mcp.types import ToolAnnotations
from pydantic import Field, model_validator

from scripts import python_extension
from scripts.capabilities import process
from scripts.capabilities.model import Capability, Contract, Prerequisite
from scripts.capabilities.workers import WorkerCall

if TYPE_CHECKING:
    from pathlib import Path


class StageRequest(Contract):
    stage: python_extension.Stage
    profile: str = Field(default="dev", pattern=r"^[A-Za-z0-9][A-Za-z0-9_-]{0,63}$")
    validated: bool = True


class StageInvocation(StageRequest):
    mode: python_extension.Mode


class ImportCheckRequest(Contract):
    mode: python_extension.Mode
    validated: bool = True


class ImportCheckResult(Contract):
    mode: python_extension.Mode
    validated: bool
    compatible: bool
    artifact: python_extension.ExtensionArtifact | None = None
    cause: str | None = None

    @model_validator(mode="after")
    def observed_identity(self) -> ImportCheckResult:
        if self.compatible and self.artifact is None:
            raise ValueError("compatible import requires its observed artifact identity")
        return self


STAGE_CALLS = {
    "dev": WorkerCall("prep-dev", "stage"),
    "native": WorkerCall("prep-native", "stage"),
}
IMPORT_CALLS = {
    "dev": WorkerCall("prep-dev", "import-check"),
    "native": WorkerCall("prep-native", "import-check"),
}


async def import_check(
    root: Path, mode: python_extension.Mode, *, validated: bool
) -> ImportCheckResult:
    result = await process.run_worker(
        root, IMPORT_CALLS[mode], ImportCheckRequest(mode=mode, validated=validated)
    )
    observed = ImportCheckResult.model_validate(result)
    if observed.mode != mode or observed.validated != validated:
        raise ValueError("import check result differs from the selected requirement")
    return observed


def bindings(root: Path) -> tuple[tuple[Capability, ...], dict[str, Tool]]:
    async def python_extension_dev_stage(request: StageRequest) -> python_extension.PreparationResult:
        """Explicitly run one development extension owner stage in one admitted environment. environment ensures locked dependencies; prepare may build/install; stubs publishes generated outputs; stubs-check compares without publication; admit imports and fences actual bytes. Select profile and correctness/interactive validation explicitly. No automatic sync composition."""
        result = await process.run_worker(
            root, STAGE_CALLS["dev"], StageInvocation(**request.model_dump(), mode="dev")
        )
        return python_extension.PreparationResult.model_validate(result)

    async def python_extension_native_stage(request: StageRequest) -> python_extension.PreparationResult:
        """Explicitly run one linked native extension owner stage after its admitted provider environment. environment ensures locked dependencies; prepare may build/install; stubs publishes; stubs-check compares without publication; admit imports and fences actual bytes. Select profile and correctness/interactive validation explicitly. No automatic sync composition."""
        result = await process.run_worker(
            root, STAGE_CALLS["native"], StageInvocation(**request.model_dump(), mode="native")
        )
        return python_extension.PreparationResult.model_validate(result)

    specs = []
    tools = {}
    for function, mode in ((python_extension_dev_stage, "dev"), (python_extension_native_stage, "native")):
        tool = Tool.from_function(
            function,
            annotations=ToolAnnotations(read_only_hint=False, destructive_hint=True, idempotent_hint=False, open_world_hint=False),
        )
        spec = Capability(
            id=tool.name, title=tool.name.replace("_", " ").title(),
            description=tool.description or tool.name,
            domain="environment", family="python-extension", guides=("python", "environment"),
            owner="scripts/python_extension.py", source="scripts/capabilities/preparation.py",
            exposure="tool", effects="Explicit selected owner stage; dependency installation, extension replacement and stub publication occur only in their chosen stages. Admission observes actual import identity; checks do not publish.",
            worker_calls=(STAGE_CALLS[mode],),
            requirements=(Prerequisite(kind="checkout-environment", owner="scripts/pse_env.py", mode=mode),),
            prerequisites=("Selected stage owns its actual environment, compiler, extension and generator prerequisites; describing this binding prepares nothing.",),
        )
        tool.tags.update(spec.tags)
        specs.append(spec)
        tools[spec.id] = tool
    return tuple(specs), tools
