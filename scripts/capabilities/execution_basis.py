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
from scripts.producing_currency import Witness

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
    root: Path, worker: str, operation: str, request: bytes, received: Handoff,
    *, witness: Witness | None = None,
) -> None:
    """Verify one independent request, including inside an admitted foreground session.

    A session-owned witness rehashes only changed selected source files. One-shot
    callers keep their original source capture; neither route adopts a new definition.
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
    actual: dict[str, str | None]
    try:
        if witness is None:
            observed = Sources(root, expected.sources, follow_imports=False)
            actual = {name: observed.sources[name].digest for name in expected.sources}
        else:
            actual = {}
            selected_root = root.resolve(strict=True)
            for name in expected.sources:
                path = selected_root / name
                if path.is_symlink() or not path.resolve().is_relative_to(selected_root):
                    raise ValueError("admitted source is outside the selected checkout")
                if path.exists() and path.stat().st_size > 4 << 20:
                    raise ValueError("admitted source exceeds its original extent")
                value = witness.file(path)
                if value.get("link") is not None or not Path(str(value["resolved"])).is_relative_to(selected_root):
                    raise ValueError("admitted source changed its link selection")
                digest = value.get("sha256")
                if digest is not None and not isinstance(digest, str):
                    raise ValueError("admitted source witness is malformed")
                actual[name] = digest
    except (OSError, ValueError, SyntaxError):
        raise AdmissionRefusedError(
            "Admitted source changed or became unreadable; reattach before further effects"
        ) from None
    # Sources follows local imports to read safely. Shared serving modules may
    # import unrelated operations: only the admitted operation's selected basis
    # is authoritative here, exactly as in the parent's restart fence.
    if actual != expected.sources:
        raise AdmissionRefusedError(
            "Admitted source changed; reattach before further effects"
        )
