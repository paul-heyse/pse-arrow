# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Scoped Rust test requests consume selection, normalization and execution owners."""

from __future__ import annotations

import json
import os
from typing import TYPE_CHECKING, Annotated, Literal

from pydantic import ConfigDict, Field, TypeAdapter, model_validator

from scripts import (
    arrow_validation,
    select,
    test_invocation,
    test_resources,
    test_run,
    validation,
)
from scripts.capabilities.model import Contract
from scripts.capabilities.worker_progress import ProgressReporter, OwnerReference

from scripts.capabilities.rust_test_contracts import RustTestRequest, INVOCATION, InvocationReference

def execute(root: Path, request: RustTestRequest, *, progress: ProgressReporter | None = None) -> dict[str, object]:
    command, normalized = request.prepared()
    environment = arrow_validation.rust_test_environment(os.environ)
    inherited = environment.get(test_resources.MARKER) or None
    output = validation.fresh_output(root, root / request.output, "scoped-rust-test")
    environment["PSE_NATIVE_PROVENANCE"] = str(output / "native.json")
    environment["PSE_NATIVE_SELECTION"] = str(output / "native-selected.json")
    invocation: Path | None = None

    def selected(path: Path) -> None:
        nonlocal invocation
        if invocation is not None and invocation != path:
            raise ValueError(
                "native scoped runner reported conflicting invocation identities"
            )
        invocation = path
        validation.write_json(
            output / "invocation-reference.json",
            InvocationReference(
                invocation=str(path), registry=str(test_resources.registry()),
                ownership="borrowed" if inherited is not None else "owned",
                terminal_owner="original outer owner" if inherited is not None else "native runner",
            ).model_dump(mode="json"),
        )
        if progress is not None:
            progress.publish("execution", "started", reference=OwnerReference(
                kind="invocation", locator=str(path), registry=str(test_resources.registry()),
                ownership="borrowed" if inherited is not None else "owned",
            ))

    code = test_run.run_rust(
        command,
        env=environment,
        effects=request.execution_effects,
        package_scope=request.kind == "unit",
        on_invocation=selected,
    )
    declaration = (
        test_resources.read_invocation(invocation) if invocation is not None else None
    )
    # Borrowed invocations are reconciled by their original outer owner. A prior
    # terminal file there must never be promoted to this call's new terminal.
    terminal_path = (
        invocation.with_suffix(".terminal.json")
        if invocation is not None and inherited is None
        else None
    )
    terminal = (
        json.loads(terminal_path.read_text())
        if terminal_path is not None and terminal_path.is_file()
        else None
    )
    provenance = output / "native.json"
    prepared = json.loads(provenance.read_text()) if provenance.is_file() else None
    catalogue = declaration["selected"] if declaration is not None else []
    return {
        "exit_code": code,
        "output": str(output),
        "invocation": str(invocation) if invocation is not None else None,
        "invocation_reference": str(output / "invocation-reference.json")
        if invocation is not None
        else None,
        "invocation_ownership": "borrowed"
        if inherited is not None
        else "owned"
        if invocation is not None
        else "not allocated",
        "terminal": str(terminal_path)
        if terminal_path is not None and terminal_path.is_file()
        else None,
        "terminal_observed": terminal is not None,
        "native_terminal": terminal,
        "selected": catalogue[:100],
        "selected_count": len(catalogue),
        "catalogue_complete": len(catalogue) <= 100,
        "catalogue_reader": {
            "capability": "product_run_read",
            "invocation": str(invocation),
            "registry": str(test_resources.registry()),
            "offset": 100 if len(catalogue) > 100 else 0,
        }
        if invocation is not None
        else None,
        "omissions": [
            "Selected catalogue shows first 100 tests; continue with the authentic invocation reader."
        ]
        if len(catalogue) > 100
        else [],
        "native_provenance": str(provenance) if prepared is not None else None,
        "prepared_invocation": prepared.get("rust_invocation")
        if prepared is not None
        else None,
        "requested_invocation": INVOCATION.dump_python(normalized, mode="json"),
        "execution_effects": request.execution_effects,
    }
