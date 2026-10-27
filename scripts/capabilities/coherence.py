# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Loaded serving contracts and optional source-provenance diagnostics.

Construction captures the interface once. Serving-code edits require explicit
drain/reload, including lazy imports; dispatch does not infer that lifecycle from
filesystem scans.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import TYPE_CHECKING

from fastmcp.exceptions import ToolError
from fastmcp.server.middleware import Middleware

from scripts.capabilities import execution_basis
from scripts.capabilities.source_basis import Sources
from scripts.capabilities.workers import WORKERS

if TYPE_CHECKING:
    from fastmcp import FastMCP
    from fastmcp.server.middleware import CallNext, MiddlewareContext
    from fastmcp.tools import ToolResult
    from mcp.types import CallToolRequestParams
    from pydantic import JsonValue

    from scripts.capabilities.catalogue import Catalogue

METRIC_DECLARATIONS = "scripts/generated/native_metric_declarations.json"
COMMON = (
    "scripts/capabilities/__init__.py",
    "scripts/capabilities/__main__.py",
    "scripts/capabilities/server.py",
    "scripts/capabilities/catalogue.py",
    "scripts/capabilities/model.py",
    "scripts/capabilities/inventory.py",
    "scripts/capabilities/coherence.py",
    "scripts/capabilities/guidance.py",
    "scripts/capabilities/presentation.py",
    "scripts/capabilities/observation.py",
)


class Interface:
    def __init__(self, sources: Sources, catalogue: Catalogue) -> None:
        self.sources = sources
        self.revision = catalogue.revision
        self.common = frozenset(path for path in COMMON if path in sources.sources)
        self.tools = catalogue.tools
        self.wires = {
            name: execution_basis.wire(tool) for name, tool in catalogue.tools.items()
        }
        self.operations = {}
        for name, tool in catalogue.tools.items():
            spec = catalogue.capabilities[name]
            function = getattr(tool, "fn", None)
            module = getattr(function, "__module__", "")
            paths = [spec.owner, spec.source]
            if spec.family == "telemetry-observation":
                paths.append(METRIC_DECLARATIONS)
            if module.startswith("scripts."):
                paths.append(module.replace(".", "/") + ".py")
            # Fixed workers are executed by name in fresh children, not imported
            # into the serving process. Their modules and local imports still
            # contribute provenance to the loaded contract.
            for call in spec.worker_calls:
                paths.append(call.declaration.module.replace(".", "/") + ".py")
                paths.extend(call.declaration.sources)
            # Custom Tool implementations have an owner class rather than fn.
            paths.append(type(tool).__module__.replace(".", "/") + ".py")
            self.operations[name] = sources.closure(paths)

    def status(self, capability: str | None = None) -> dict[str, JsonValue]:
        """Project the loaded contract for automatic consumers, without source I/O."""
        changed = []
        names = (
            (capability,)
            if capability in self.tools
            else self.tools
            if capability is None
            else ()
        )
        for name in names:
            if execution_basis.wire(self.tools[name]) != self.wires[name]:
                changed.append("exposed-tool-contract:" + name)
        return {
            "revision": self.revision,
            "checkout": str(self.sources.root),
            "interpreter": str(self.sources.root / ".venv/bin/python"),
            "operation_wire_revision": execution_basis.digest(self.wires[capability])
            if capability in self.wires
            else None,
            "restart_required": bool(changed),
            "changed_serving_sources": list(changed),
            "scope": "loaded operation wire contract; native/dependency readiness remains operation-owned",
        }

    def require(self, capability: str | None = None) -> None:
        # Live in-memory tool metadata can change independently of the captured
        # wire contract. This check concerns that contract, not source currency.
        if (
            capability in self.tools
            and execution_basis.wire(self.tools[capability]) != self.wires[capability]
        ):
            raise ToolError(
                json.dumps(
                    {
                        "code": "pse.capabilities.restart_required",
                        "revision": self.revision,
                        "changed_contract": capability,
                    }
                )
            )


class RestartMiddleware(Middleware):
    def __init__(self, interface: Interface) -> None:
        self.interface = interface

    async def on_call_tool(
        self,
        context: MiddlewareContext[CallToolRequestParams],
        call_next: CallNext[CallToolRequestParams, ToolResult],
    ) -> ToolResult:
        name = context.message.name
        # The invokers call canonical dispatch again under the loaded contract.
        if name in self.interface.operations:
            self.interface.require(name)
            paths = self.interface.common | self.interface.operations[name]
            definition = execution_basis.Definition(
                checkout=str(self.interface.sources.root),
                operation=name,
                wire_revision=execution_basis.digest(self.interface.wires[name]),
                sources={
                    path: self.interface.sources.sources[path].digest for path in paths
                },
            )
            with execution_basis.selected(definition):
                return await call_next(context)
        return await call_next(context)


def install(server: FastMCP, sources: Sources, catalogue: Catalogue) -> Interface:
    interface = Interface(sources, catalogue)
    server.add_middleware(RestartMiddleware(interface))
    return interface


# The server imports this module before its operation modules. Capture source
# bytes once for provenance; later build() calls cannot relabel old imports
# with newly edited source. Controlled editing owns explicit drain/reload.
LOADED = Sources(
    Path(__file__).resolve().parents[2],
    (
        *COMMON,
        METRIC_DECLARATIONS,
        *(worker.module.replace(".", "/") + ".py" for worker in WORKERS.values()),
        *(path for worker in WORKERS.values() for path in worker.sources),
    ),
)
