# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""A bounded serving-source cut, distinct from current project/source facts.

Capture before importing operation bindings, verify after composition, and check
only the common interpretation plus the selected operation's local dependency
closure before dispatch. This is a restart fence, not a Python hot reloader or
proof about arbitrary third-party/runtime dependencies.
"""

from __future__ import annotations

import ast
import hashlib
import json
import stat
from dataclasses import dataclass
from pathlib import Path
from threading import RLock
from typing import TYPE_CHECKING

from fastmcp.exceptions import ToolError
from fastmcp.server.middleware import Middleware

from scripts.capabilities.source_modules import module_candidates
from scripts.capabilities.workers import WORKERS

if TYPE_CHECKING:
    from collections.abc import Iterable

    from fastmcp import FastMCP
    from fastmcp.server.middleware import CallNext, MiddlewareContext
    from fastmcp.tools import ToolResult
    from mcp.types import CallToolRequestParams
    from pydantic import JsonValue

    from scripts.capabilities.catalogue import Catalogue

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


@dataclass(frozen=True)
class Source:
    digest: str | None
    dependencies: tuple[str, ...]


def local_imports(raw: bytes) -> tuple[str, ...]:
    """Follow declared local imports, including lazy imports inside operations."""
    paths = set()
    for node in ast.walk(ast.parse(raw)):
        names: list[str] = []
        if isinstance(node, ast.Import):
            names = [alias.name for alias in node.names]
        elif isinstance(node, ast.ImportFrom) and node.module:
            names = [node.module, *(node.module + "." + a.name for a in node.names)]
        for name in names:
            if name.startswith("scripts."):
                paths.update(module_candidates(name))
    return tuple(sorted(paths))


class Sources:
    def __init__(self, root: Path, paths: Iterable[str]) -> None:
        self.root = root.resolve(strict=True)
        self.sources: dict[str, Source] = {}
        self.changed: set[str] = set()
        self._lock = RLock()
        pending = list(paths)
        while pending:
            relative = pending.pop()
            if relative in self.sources:
                continue
            path = self.root / relative
            raw = self._read_optional(relative)
            dependencies = (
                local_imports(raw) if raw is not None and path.suffix == ".py" else ()
            )
            self.sources[relative] = Source(
                hashlib.sha256(raw).hexdigest() if raw is not None else None,
                dependencies,
            )
            pending.extend(dependencies)
        self.revision = hashlib.sha256(
            json.dumps(
                {path: source.digest for path, source in sorted(self.sources.items())},
                sort_keys=True,
            ).encode()
        ).hexdigest()

    def _read_optional(self, relative: str) -> bytes | None:
        try:
            return self._read(relative)
        except FileNotFoundError:
            return None

    def _read(self, relative: str) -> bytes:
        path = self.root / relative
        if path.is_symlink() or not path.resolve(strict=True).is_relative_to(self.root):
            raise ValueError("Serving source must remain within its selected checkout")
        before = path.stat()
        if not stat.S_ISREG(before.st_mode) or before.st_size > 4 << 20:
            raise ValueError("Serving source must be a bounded ordinary file")
        raw = path.read_bytes()
        after = path.stat()
        if (
            before.st_dev,
            before.st_ino,
            before.st_size,
            before.st_mtime_ns,
            before.st_ctime_ns,
            before.st_mode,
        ) != (
            after.st_dev,
            after.st_ino,
            after.st_size,
            after.st_mtime_ns,
            after.st_ctime_ns,
            after.st_mode,
        ):
            raise ValueError("Serving source changed during interface capture")
        return raw

    def closure(self, paths: Iterable[str]) -> frozenset[str]:
        selected: set[str] = set()
        pending = list(paths)
        while pending:
            path = pending.pop()
            if path in selected or path not in self.sources:
                continue
            selected.add(path)
            pending.extend(self.sources[path].dependencies)
        return frozenset(selected)

    def check(self, paths: Iterable[str]) -> tuple[str, ...]:
        with self._lock:
            for path in paths:
                if path not in self.sources or path in self.changed:
                    continue
                try:
                    raw = self._read_optional(path)
                    digest = (
                        hashlib.sha256(raw).hexdigest() if raw is not None else None
                    )
                except (OSError, ValueError):
                    self.changed.add(path)
                else:
                    if digest != self.sources[path].digest:
                        self.changed.add(path)
            return tuple(sorted(self.changed.intersection(paths)))

    def verify_composition(self) -> None:
        if self.check(self.sources):
            raise ValueError(
                "Serving source changed while loading; restart the endpoint"
            )


class Interface:
    def __init__(self, sources: Sources, catalogue: Catalogue) -> None:
        self.sources = sources
        self.revision = catalogue.revision
        self.common = frozenset(path for path in COMMON if path in sources.sources)
        self.operations = {}
        for name, tool in catalogue.tools.items():
            spec = catalogue.capabilities[name]
            function = getattr(tool, "fn", None)
            module = getattr(function, "__module__", "")
            paths = [spec.owner, spec.source]
            if module.startswith("scripts."):
                paths.append(module.replace(".", "/") + ".py")
            # Fixed workers are executed by name in fresh children, not imported
            # into the serving process. Their modules and local imports still
            # interpret the loaded contract and must share its restart fence.
            for call in spec.worker_calls:
                paths.append(call.declaration.module.replace(".", "/") + ".py")
                paths.extend(call.declaration.sources)
            # Custom Tool implementations have an owner class rather than fn.
            paths.append(type(tool).__module__.replace(".", "/") + ".py")
            self.operations[name] = sources.closure(paths)

    def status(self, capability: str | None = None) -> dict[str, JsonValue]:
        paths = self.common | self.operations.get(capability, frozenset())
        changed = self.sources.check(paths)
        return {
            "revision": self.revision,
            "restart_required": bool(changed),
            "changed_serving_sources": list(changed),
            "scope": "known local serving sources; native/dependency readiness remains operation-owned",
        }

    def require(self, capability: str | None = None) -> None:
        status = self.status(capability)
        if status["restart_required"]:
            raise ToolError(
                json.dumps({"code": "pse.capabilities.restart_required", **status})
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
        # Bootstrap still describes the loaded snapshot and the restart cause.
        # The invokers call canonical dispatch again, where the selected owner is fenced.
        if name in self.interface.operations:
            self.interface.require(name)
        return await call_next(context)


def install(server: FastMCP, sources: Sources, catalogue: Catalogue) -> Interface:
    sources.verify_composition()
    interface = Interface(sources, catalogue)
    server.add_middleware(RestartMiddleware(interface))
    return interface


# The server imports this module before its operation modules. Capture source
# bytes once for this process; later build() calls cannot relabel old imports
# with newly edited source. The launch owner performs a second composition check.
LOADED = Sources(
    Path(__file__).resolve().parents[2],
    (
        *COMMON,
        *(worker.module.replace(".", "/") + ".py" for worker in WORKERS.values()),
        *(path for worker in WORKERS.values() for path in worker.sources),
    ),
)
