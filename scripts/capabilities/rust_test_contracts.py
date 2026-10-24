# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Scoped test selection and normalization, without test execution owners."""
from __future__ import annotations
from typing import Annotated, Literal
from pydantic import ConfigDict, Field, TypeAdapter, model_validator
from scripts import select, test_invocation
from scripts.capabilities.model import Contract

Name = Annotated[str, Field(pattern=r"^[A-Za-z0-9_][A-Za-z0-9_.-]*$")]
Feature = Annotated[str, Field(pattern=r"^[A-Za-z0-9_][A-Za-z0-9_./-]*$")]
INVOCATION = TypeAdapter(test_invocation.RustInvocation)


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


class InvocationReference(Contract):
    invocation: str = Field(min_length=1, max_length=4096)
    registry: str = Field(min_length=1, max_length=4096)
    ownership: Literal["owned", "borrowed"]
    terminal_owner: Literal["native runner", "original outer owner"]


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


