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

Name = Annotated[str, Field(pattern=r"^[A-Za-z0-9_][A-Za-z0-9_.-]*$")]
Feature = Annotated[str, Field(pattern=r"^[A-Za-z0-9_][A-Za-z0-9_./-]*$")]
INVOCATION = TypeAdapter(test_invocation.RustInvocation)

if TYPE_CHECKING:
    from pathlib import Path


class RustTestSelection(Contract):
    model_config = ConfigDict(
        extra="forbid", strict=True, frozen=True, validate_default=True
    )

    package: Name
    kind: Literal["unit", "integration"] = "unit"
    target: Name | None = None
    expression: str | None = Field(default=None, min_length=1)
    features: list[Feature] = Field(default_factory=list)
    default_features: bool = True
    profile: Name | None = None
    cargo_profile: Name | None = None
    execution_effects: Literal["canonical", "native-local"] = "canonical"

    @model_validator(mode="after")
    def selected_target(self) -> RustTestSelection:
        if self.kind == "unit" and (self.expression is None or self.target is not None):
            raise ValueError(
                "unit selection requires an expression and has no named integration target"
            )
        if self.kind == "integration" and self.target is None:
            raise ValueError("integration selection requires one exact named target")
        return self

    def prepared(self) -> tuple[list[str], test_invocation.RustInvocation]:
        args = []
        if not self.default_features:
            args.append("--no-default-features")
        if self.profile is not None:
            args.extend(("--profile", self.profile))
        if self.cargo_profile is not None:
            args.extend(("--cargo-profile", self.cargo_profile))
        if self.kind == "unit":
            if self.expression is None:
                raise ValueError("unit selection requires an expression")
            command = select.unit(
                self.package, self.expression, args, features=self.features
            )
        else:
            if self.target is None:
                raise ValueError(
                    "integration selection requires one exact named target"
                )
            if self.expression is not None:
                args.extend(("-E", select.test_filter(self.expression)))
            command = select.integration(
                self.package, self.target, args, features=self.features
            )
        if command[2] != "run":
            raise ValueError(
                "scoped test execution requires the native run action; inherited PSE_NEXTEST_ACTION selects another action"
            )
        # Reject unsupported native options before output, build or preparation.
        # There is one parser: the same owner used by discovery and execution.
        invocation = test_invocation.rust(command[3:])
        if any(option.name == "--no-run" for option in invocation.options):
            raise ValueError("scoped test execution cannot inherit --no-run")
        return command, invocation


class RustTestRequest(RustTestSelection):
    output: str = Field(min_length=1)


def describe(request: RustTestSelection) -> dict[str, object]:
    command, invocation = request.prepared()
    return {
        "command": command,
        "normalized_invocation": INVOCATION.dump_python(invocation, mode="json"),
        "execution_effects": request.execution_effects,
        "catalogue": None,
        "compiled_or_enumerated": False,
        "package_scope": request.kind == "unit",
        "assurance": "Declared request only; actual force-validation closure and catalogue are prepared by the native runner during execution.",
    }


def execute(root: Path, request: RustTestRequest) -> dict[str, object]:
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
            {
                "invocation": str(path),
                "registry": str(test_resources.registry()),
                "ownership": "borrowed" if inherited is not None else "owned",
                "terminal_owner": "original outer owner"
                if inherited is not None
                else "native runner",
            },
        )

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
