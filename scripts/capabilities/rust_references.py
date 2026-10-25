# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Read an original scoped-test reference, including while its action is attached."""

from __future__ import annotations

import hashlib
import os
import stat
from contextlib import ExitStack
from pathlib import Path
from typing import Literal

from fastmcp.tools import Tool
from mcp.types import ToolAnnotations
from pydantic import Field

from scripts.capabilities.diagnostics import refusal
from scripts.capabilities.model import Capability, Contract
from scripts.capabilities.rust_test_contracts import InvocationReference


class ReferenceRead(Contract):
    outcome: Literal["issued", "not_observed"]
    reference: InvocationReference | None = None
    source_sha256: str | None = None
    reader: Literal["product_run_read"] | None = None
    settlement: str = "Inspect the original native owner; absence, reference publication and reader success do not establish settlement or retry eligibility."


def read(root: Path, output: str) -> ReferenceRead:
    from scripts import (  # noqa: PLC0415 -- selected original registry reader.
        run_inspection,
    )

    root = root.absolute()
    path = Path(output)
    if not path.is_absolute():
        path = root / path
    if ".." in path.parts or not path.is_relative_to(root):
        raise ValueError("select an owned checkout output without symlinks")
    with ExitStack() as stack:
        try:
            # Hold every ancestor instead of checking a pathname that can be
            # swapped before a later open. Include the checkout's ancestors so
            # neither the root nor an output parent can redirect observation.
            directory = os.open(
                root.anchor, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW
            )
            stack.callback(os.close, directory)
            for part in (*root.parts[1:], *path.relative_to(root).parts):
                directory = os.open(
                    part,
                    os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW,
                    dir_fd=directory,
                )
                stack.callback(os.close, directory)
            descriptor = os.open(
                "invocation-reference.json",
                os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK,
                dir_fd=directory,
            )
            stack.callback(os.close, descriptor)
        except FileNotFoundError:
            return ReferenceRead(outcome="not_observed")
        with os.fdopen(os.dup(descriptor), "rb") as stream:
            before = os.fstat(descriptor)
            if not stat.S_ISREG(before.st_mode) or before.st_size > 16384:
                raise ValueError("reference must be a bounded ordinary owner record")
            raw = stream.read(16385)
            after = os.fstat(descriptor)
            current = os.stat(
                "invocation-reference.json", dir_fd=directory, follow_symlinks=False
            )

    def identity(item: os.stat_result) -> tuple[int, int, int, int, int]:
        return (
            item.st_dev,
            item.st_ino,
            item.st_size,
            item.st_mtime_ns,
            item.st_ctime_ns,
        )

    if (
        identity(before) != identity(after)
        or identity(before) != identity(current)
        or len(raw) > 16384
    ):
        raise ValueError("owner reference changed during observation")
    reference = InvocationReference.model_validate_json(raw)
    run_inspection.resolve(reference.invocation, Path(reference.registry))
    return ReferenceRead(
        outcome="issued",
        reference=reference,
        source_sha256=hashlib.sha256(raw).hexdigest(),
        reader="product_run_read",
    )


def bindings(root: Path) -> tuple[tuple[Capability, ...], dict[str, Tool]]:
    def rust_test_reference_read(
        output: str = Field(min_length=1, max_length=4096),
    ) -> ReferenceRead:
        """Read an actual scoped Rust action's invocation-reference.json before final return. The requested output alone is unconfirmed; an issued reference is checked against its original registry. Continue with product_run_read/workload_watch; no build, launch, replay or native settlement follows from reading."""
        try:
            return read(root, output)
        except (OSError, ValueError) as error:
            raise refusal(
                "pse.rust_test.reference_unavailable",
                operation="rust_test_reference_read",
            ) from error

    tool = Tool.from_function(
        rust_test_reference_read,
        annotations=ToolAnnotations(
            read_only_hint=True,
            destructive_hint=False,
            idempotent_hint=True,
            open_world_hint=False,
        ),
    )
    spec = Capability(
        id=tool.name,
        title="Read original scoped test reference",
        description=tool.description or "",
        domain="execution",
        family="workload-observation",
        guides=("root", "qualification"),
        owner="scripts/capabilities/rust_references.py",
        source="scripts/capabilities/rust_references.py",
        exposure="tool",
        invocation_lifetime="single",
        effects="Read a bounded original reference and its registry identity; no workload or preparation.",
    )
    tool.tags.update(spec.tags)
    return (spec,), {spec.id: tool}
