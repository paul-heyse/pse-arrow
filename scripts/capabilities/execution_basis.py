# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Private admitted-definition handoff; original workers retain execution authority."""

from __future__ import annotations

import hashlib
import json
import os
from contextlib import contextmanager
from contextvars import ContextVar
from pathlib import Path
from typing import TYPE_CHECKING

from scripts.capabilities.model import Contract
from scripts.capabilities.source_basis import Sources
from scripts.capabilities.workers import WORKERS

if TYPE_CHECKING:
    from collections.abc import Generator

    from fastmcp.tools import Tool
    from pydantic import JsonValue

ENVIRONMENT = "PSE_CAPABILITY_ADMISSION"
MAXIMUM_BYTES = 256 << 10


class AdmissionRefusedError(ValueError):
    """Controlled private-handoff refusal before owner effects."""

    def detail(self) -> dict[str, JsonValue]:
        return {
            "code": "pse.capabilities.stale_operation",
            "stage": "admission",
            "message": "The selected operation no longer matches its admitted definition.",
            "remedy": "Reattach to the selected checkout and inspect the original owner before choosing further action. No effects or retry are inferred.",
        }


def digest(value: object) -> str:
    return hashlib.sha256(
        json.dumps(
            value, sort_keys=True, allow_nan=False, separators=(",", ":")
        ).encode()
    ).hexdigest()


def wire(tool: Tool) -> dict[str, JsonValue]:
    return tool.to_mcp_tool().model_dump(mode="json", by_alias=True)


class Definition(Contract):
    checkout: str
    operation: str
    wire_revision: str
    sources: dict[str, str | None]


class Handoff(Contract):
    definition: Definition
    worker: str
    operation: str
    route_revision: str
    request_sha256: str


_CURRENT: ContextVar[Definition | None] = ContextVar(
    "pse_admitted_definition", default=None
)


@contextmanager
def selected(definition: Definition) -> Generator[None]:
    token = _CURRENT.set(definition)
    try:
        yield
    finally:
        _CURRENT.reset(token)


def route_revision(worker: str) -> str:
    declaration = WORKERS[worker]
    return digest(
        {
            "module": declaration.module,
            "operations": sorted(declaration.operations),
            "environment": declaration.environment,
            "sources": declaration.sources,
            "launch": declaration.launch,
            "environment_selection": declaration.environment_selection,
        }
    )


def handoff(root: Path, worker: str, operation: str, request: bytes) -> Handoff:
    definition = _CURRENT.get()
    if definition is None:
        # Original non-MCP callers still use the same fixed worker and source basis.
        declaration = WORKERS[worker]
        captured = Sources(
            root, (declaration.module.replace(".", "/") + ".py", *declaration.sources)
        )
        definition = Definition(
            checkout=str(root.resolve()),
            operation=operation,
            wire_revision=digest({}),
            sources={path: source.digest for path, source in captured.sources.items()},
        )
    if Path(definition.checkout) != root.resolve():
        raise AdmissionRefusedError("Admitted operation belongs to another checkout")
    return Handoff(
        definition=definition,
        worker=worker,
        operation=operation,
        route_revision=route_revision(worker),
        request_sha256=hashlib.sha256(request).hexdigest(),
    )


def publish(
    directory: Path, root: Path, worker: str, operation: str, request: bytes
) -> dict[str, str]:
    document = handoff(root, worker, operation, request).model_dump_json().encode()
    if len(document) > MAXIMUM_BYTES:
        raise AdmissionRefusedError("Admitted definition exceeds private handoff bound")
    path = directory / "admission.json"
    with path.open("xb") as output:
        output.write(document)
    path.chmod(0o600)
    return {ENVIRONMENT: str(path)}


def verify(root: Path, worker: str, operation: str, request: bytes) -> None:
    supplied = os.environ.get(ENVIRONMENT)
    if supplied is None:
        # Direct owner CLI entry is distinct from an admitted MCP invocation.
        return
    path = Path(supplied)
    if path.is_symlink() or not path.is_file() or path.stat().st_size > MAXIMUM_BYTES:
        raise AdmissionRefusedError("Invalid admitted definition handoff")
    received = Handoff.model_validate_json(path.read_bytes())
    verify_handoff(root, worker, operation, request, received)


def verify_handoff(
    root: Path,
    worker: str,
    operation: str,
    request: bytes,
    received: Handoff,
) -> None:
    """Bind an independent request to its admitted checkout and fixed route.

    Source provenance was captured by the construction owner. Serving-code edits
    require explicit drain/reload; an internal handoff does not rescan that code.
    """
    expected = received.definition
    if operation not in WORKERS[worker].operations:
        raise AdmissionRefusedError(
            "Admitted operation is outside its fixed worker route"
        )
    if (
        Path(expected.checkout) != root.resolve()
        or received.worker != worker
        or received.operation != operation
        or received.route_revision != route_revision(worker)
        or received.request_sha256 != hashlib.sha256(request).hexdigest()
    ):
        raise AdmissionRefusedError(
            "Admitted operation contract changed; reattach before further effects"
        )
