# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Historical report references and passive telemetry records.

References retain their original reader vocabulary without launching that reader
or asserting current execution, readiness or scientific acceptance.
"""

from __future__ import annotations

from typing import Annotated, Literal

from pydantic import ConfigDict, Field, JsonValue, TypeAdapter, model_validator

from scripts.host_contracts import Contract as BaseContract


class Contract(BaseContract):
    model_config = ConfigDict(
        extra="forbid",
        strict=True,
        frozen=True,
        validate_default=True,
        serialize_by_alias=True,
    )


ReferenceText = Annotated[str, Field(min_length=1, max_length=4096)]


class InvocationReference(Contract):
    kind: Literal["invocation"]
    invocation: ReferenceText
    registry: ReferenceText | None = None


class AssessmentReference(Contract):
    kind: Literal["assessment"]
    directory: ReferenceText
    gate: Annotated[str, Field(min_length=1, max_length=160)] | None = None


class NativeReaderReference(Contract):
    kind: Literal["native"]
    reader: Literal[
        "native_result_inspect",
        "native_diagnostic_inspect",
        "native_diagnostic_checkpoint",
    ]
    arguments: Annotated[
        dict[str, JsonValue],
        Field(
            max_length=32,
            description="Original reader envelope; copying it neither validates native eligibility nor executes the reader.",
        ),
    ]
    source_basis: Annotated[dict[str, JsonValue], Field(max_length=32)] = Field(
        default_factory=dict
    )


OwnerReference = Annotated[
    InvocationReference | AssessmentReference | NativeReaderReference,
    Field(discriminator="kind"),
]


class ReaderSuggestion(Contract):
    reference: OwnerReference
    reason: Annotated[str, Field(min_length=1, max_length=2048)]
    disposition: Literal["unexecuted"] = "unexecuted"


JSON_RECORD = TypeAdapter(dict[str, JsonValue])


class OwnerRecord(Contract):
    """Original owner vocabulary; extensions must remain real JSON values."""

    model_config = ConfigDict(
        extra="allow", strict=True, frozen=True, serialize_by_alias=True
    )

    @model_validator(mode="before")
    @classmethod
    def json_record(cls, value: object) -> object:
        # BaseModel owns its mutable private storage annotation. Validate the
        # original wire record rather than illegally narrow that inherited slot.
        return JSON_RECORD.validate_python(value)
