# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Validate one record, key by key, against a struct.

`msgspec.convert` stops at the first error of a record. Validating each key separately
reports every deviation of the record; the record is then built from the validated data.
"""

from __future__ import annotations

import types
import typing
from collections.abc import Callable, Mapping
from dataclasses import dataclass, field

import msgspec

from thermo_knowledge.survey_index.diagnostics import Code, SurveyDiagnostic

KeyCheck = Callable[[object], str | None]
"""A vocabulary or format check of one key's value: a message, or `None` when it is accepted."""


@dataclass
class Context:
    """Where a record is, and where its diagnostics go."""

    file: str
    table: str
    record: str
    diagnostics: list[SurveyDiagnostic] = field(default_factory=list)

    def report(self, code: Code, key: str, message: str) -> None:
        self.diagnostics.append(
            SurveyDiagnostic(
                file=self.file,
                table=self.table,
                record=self.record,
                key=key,
                code=code,
                message=message,
            )
        )


def _describe(value: object) -> str:
    return type(value).__name__


def _is_struct_type(candidate: object) -> bool:
    return isinstance(candidate, type) and issubclass(candidate, msgspec.Struct)


def _unwrap_optional(annotation: object) -> object:
    """`X | None` -> `X`; any other annotation unchanged."""
    if typing.get_origin(annotation) in (typing.Union, types.UnionType):
        members = [arg for arg in typing.get_args(annotation) if arg is not type(None)]
        if len(members) == 1:
            return members[0]
    return annotation


def validate_struct(
    struct_type: type[msgspec.Struct],
    data: object,
    ctx: Context,
    *,
    path: str = "",
    checks: Mapping[str, KeyCheck] | None = None,
) -> bool:
    """Report every deviation of `data` from `struct_type` at its keys; whether there was none.

    `checks` maps a dotted key path (`fields.role`, without list positions) to a value check run
    after the type check. Nested structs (`list[Struct]`) are validated recursively.
    """
    where = path or "(record)"
    if not isinstance(data, dict):
        ctx.report(Code.NOT_A_TABLE, path, f"{where} is {_describe(data)}, not a table")
        return False
    ok = True
    known = {info.encode_name: info for info in msgspec.structs.fields(struct_type)}
    prefix = f"{path}." if path else ""
    for key in data:
        if key not in known:
            ctx.report(
                Code.UNKNOWN_KEY, f"{prefix}{key}", f"key {key!r} is not in the specification"
            )
            ok = False
    for name, info in known.items():
        key_path = f"{prefix}{name}"
        if name not in data:
            if info.required:
                ctx.report(Code.MISSING_KEY, key_path, f"required key {name!r} is missing")
                ok = False
            continue
        if not _validate_value(info.type, data[name], ctx, key_path, checks or {}):
            ok = False
    return ok


def _check_key(path: str) -> str:
    """`fields[2].role` -> `fields.role`."""
    out: list[str] = []
    skip = False
    for char in path:
        if char == "[":
            skip = True
        elif char == "]":
            skip = False
        elif not skip:
            out.append(char)
    return "".join(out)


def _validate_value(
    annotation: object,
    value: object,
    ctx: Context,
    path: str,
    checks: Mapping[str, KeyCheck],
) -> bool:
    inner = _unwrap_optional(annotation)
    origin = typing.get_origin(inner)
    if origin is list:
        (item_type,) = typing.get_args(inner)
        if not isinstance(value, list):
            ctx.report(Code.WRONG_TYPE, path, f"expected an array, got {_describe(value)}")
            return False
        ok = True
        if _is_struct_type(item_type):
            for index, item in enumerate(value):
                item_path = f"{path}[{index}]"
                if not validate_struct(item_type, item, ctx, path=item_path, checks=checks):
                    ok = False
            return ok
        for index, item in enumerate(value):
            if not _validate_value(item_type, item, ctx, f"{path}[{index}]", checks):
                ok = False
        return ok
    try:
        msgspec.convert(value, inner, strict=True)
    except msgspec.ValidationError as error:
        ctx.report(Code.WRONG_TYPE, path, _type_message(error, value))
        return False
    check = checks.get(_check_key(path))
    if check is not None:
        message = check(value)
        if message is not None:
            ctx.report(Code.BAD_VALUE, path, message)
            return False
    return True


def _type_message(error: msgspec.ValidationError, value: object) -> str:
    text = str(error)
    # msgspec phrases it as "Expected `str`, got `bool`"; keep that, it names the deviation.
    return text if text else f"unexpected {_describe(value)}"
