# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Qualification selection contracts independent of worker transport and binding factories."""
from __future__ import annotations
from typing import Annotated, Literal
from pydantic import ConfigDict, Field, TypeAdapter, model_validator
from scripts.capabilities.model import Contract
from scripts.validation_scope import EXCLUSIONS, FUNCTIONAL_SCOPES, GROUPS, Gate, comprehensive, expand, functional_gates

Group = Annotated[str, Field(json_schema_extra={"enum": sorted(GROUPS)})]
FunctionalScope = Annotated[
    str, Field(json_schema_extra={"enum": sorted(FUNCTIONAL_SCOPES)})
]
Package = Annotated[str, Field(pattern=r"^[A-Za-z0-9_][A-Za-z0-9_.-]*$")]
Feature = Annotated[str, Field(pattern=r"^[A-Za-z0-9_][A-Za-z0-9_./-]*$")]
GATES = TypeAdapter(list[Gate])


class AssessmentSelection(Contract):
    model_config = ConfigDict(
        extra="forbid", strict=True, frozen=True, validate_default=True
    )

    group: Group | None = None
    functional_scopes: list[FunctionalScope] | None = Field(default=None, min_length=1)
    python_profile: Literal["dev", "producer"] = "dev"

    @model_validator(mode="after")
    def selected_scope(self) -> AssessmentSelection:
        if self.group is not None and self.group not in GROUPS:
            raise ValueError("select a declared assessment group")
        if self.group is not None and self.functional_scopes is not None:
            raise ValueError("select either a group or functional scopes")
        if self.functional_scopes is not None:
            if set(self.functional_scopes) - FUNCTIONAL_SCOPES.keys():
                raise ValueError("select declared functional scopes")
            if "native" in self.functional_scopes and set(self.functional_scopes) - {
                "native",
                "managed-primary",
            }:
                raise ValueError(
                    "native covers ordinary scopes; combine it only with managed-primary"
                )
        return self

    def gates(self) -> list[Gate]:
        if self.functional_scopes is not None:
            return functional_gates(self.functional_scopes)
        if self.group is not None:
            return expand((self.group,))
        return comprehensive(self.python_profile)


class AssessmentRequest(AssessmentSelection):
    output: str = Field(min_length=1)
    reuse_from: str | None = Field(default=None, min_length=1)
    reuse: list[str] = Field(default_factory=list)
    transfer: list[str] = Field(default_factory=list)
    change_reason: str | None = None

    @model_validator(mode="after")
    def retained_origin(self) -> AssessmentRequest:
        if (self.reuse or self.transfer) and self.reuse_from is None:
            raise ValueError("reuse and transfer require an origin report")
        names = {gate.name for gate in self.gates()}
        if set(self.reuse + self.transfer) - names:
            raise ValueError("reuse/transfer must select gates in this assessment")
        if set(self.reuse) & set(self.transfer):
            raise ValueError("a gate cannot be both reused and transferred")
        return self


class CompilationRequest(Contract):
    model_config = ConfigDict(
        extra="forbid", strict=True, frozen=True, validate_default=True
    )

    output: str = Field(min_length=1)
    package: Package | None = None
    target: Literal["all", "library", "test"] = "all"
    test: Package | None = None
    features: list[Feature] = Field(default_factory=list)
    default_features: bool = True

    @model_validator(mode="after")
    def selected_target(self) -> CompilationRequest:
        if self.target != "all" and self.package is None:
            raise ValueError("library/test selection requires an exact package")
        if (self.test is not None) != (self.target == "test"):
            raise ValueError("test selection requires exactly one named test target")
        return self

    def gates(self) -> list[Gate]:
        recipe = "check"
        args = []
        if self.package is not None:
            recipe = {
                "all": "check-package",
                "library": "check-library",
                "test": "check-test",
            }[self.target]
            args.append(self.package)
            if self.test is not None:
                args.append(self.test)
        if self.features:
            args.extend(("--features", ",".join(self.features)))
        if not self.default_features:
            args.append("--no-default-features")
        return [
            Gate(
                "compilation-check",
                args=tuple(args),
                recipe=recipe,
                input_scope="rust-product",
                mode="scope-aware-force-validation",
            )
        ]


class OutputRequest(Contract):
    model_config = ConfigDict(
        extra="forbid", strict=True, frozen=True, validate_default=True
    )

    output: str = Field(min_length=1)


