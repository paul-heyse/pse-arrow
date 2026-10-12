# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Generated edit transport agrees with the strict JSON boundary for every arm."""

from concurrent.futures import ThreadPoolExecutor
from threading import Barrier

import attrs
import msgspec
import msgspec.inspect as inspect
import pytest

from pse import codec
from pse.codec import _generated
from pse.contracts import authored, documents, enums
from pse.contracts.values import ContentHash, SemanticId


def _sample(info: inspect.Type) -> object:
    if isinstance(info, inspect.StructType):
        return info.cls(
            **{
                field.name: _sample(field.type)
                for field in info.fields
                if field.required
            }
        )
    if isinstance(info, inspect.UnionType):
        return _sample(info.types[0])
    if isinstance(info, inspect.EnumType):
        return next(iter(info.cls))
    if isinstance(info, inspect.LiteralType):
        return info.values[0]
    if isinstance(info, inspect.CollectionType):
        return ()
    if isinstance(info, inspect.TupleType):
        return tuple(_sample(item) for item in info.item_types)
    if isinstance(info, inspect.StrType):
        return "1" * 32
    if isinstance(info, inspect.IntType):
        return 1
    if isinstance(info, inspect.FloatType):
        return -0.0
    if isinstance(info, inspect.BoolType):
        return False
    if isinstance(info, inspect.NoneType):
        return None
    raise AssertionError(f"Unhandled generated shape: {info}")


def _row(
    arm: str | None = None, info: inspect.Type | None = None
) -> authored.AuthoredModelingDeclarationsRow:
    values: dict[str, object] = {"kind": enums.ModelingDeclarationKind.PACKAGE}
    if arm is not None and info is not None:
        values[arm] = _sample(info)
    value = documents.AuthoredModelingDeclarationsFieldValue(**values)
    document = documents.AuthoredModelingDeclarationsRow(
        declaration_id="1" * 32,
        document_id="2" * 32,
        is_override=False,
        name="p",
        ordinal=0,
        source_start=0,
        source_end=1,
        value=value,
    )
    return codec.document_rows([document], authored.AuthoredModelingDeclarationsRow)[0]


def _old(rows: tuple[object, ...], field: str, document_type: type) -> bytes:
    converter = codec.converter()
    converter.register_unstructure_hook(SemanticId, SemanticId.to_hex)
    converter.register_unstructure_hook(ContentHash, ContentHash.to_prefixed)
    plain = {field: tuple(converter.unstructure(row) for row in rows)}
    return codec.encode_json(codec.decode_json(codec.encode_json(plain), document_type))


@pytest.mark.unit
def test_every_declaration_arm_retains_strict_transport_bytes() -> None:
    info = inspect.type_info(documents.AuthoredModelingDeclarationsFieldValue)
    assert isinstance(info, inspect.StructType)
    rows = []
    for field in info.fields:
        if field.name == "kind":
            continue
        assert isinstance(field.type, inspect.UnionType)
        arm = next(
            child
            for child in field.type.types
            if not isinstance(child, inspect.NoneType)
        )
        rows.append(_row(field.name, arm))
    assert len(rows) > 40
    for values in ((), tuple(rows), tuple(rows * 3)):
        assert _generated.encode_declarations(values) == _old(
            values, "declarations", documents.DeclarationEdit
        )


def _fit() -> authored.AuthoredFitCasesRow:
    fit = documents.AuthoredFitCasesRow(
        fit_id="3" * 32,
        parameters=(
            documents.AuthoredFitCasesFieldParametersItem(
                symbol_id="4" * 32,
                fixed=False,
                value=-0.0,
                scale=2.0,
                lower=-10.0,
                upper=10.0,
            ),
        ),
        experiments=(
            documents.AuthoredFitCasesFieldExperimentsItem(
                experiment_id="5" * 32,
                case_id="6" * 32,
                route=enums.ModelingAnalysisRoute.STEADY,
                bindings=(),
            ),
        ),
        observations=(
            documents.AuthoredFitCasesFieldObservationsItem(
                observation_id="7" * 32,
                experiment_id="5" * 32,
                output_path="unit.x",
                value_attribute="value",
                importance=1.0,
                included=True,
            ),
        ),
    )
    return codec.document_rows([fit], authored.AuthoredFitCasesRow)[0]


@pytest.mark.unit
def test_nested_fits_preserve_identities_enums_and_signed_zero() -> None:
    for rows in ((), (_fit(),), (_fit(),) * 128):
        encoded = _generated.encode_fits(rows)
        assert encoded == _old(rows, "fits", documents.FitDeclarations)
        if rows:
            assert b'"value":-0.0' in encoded


@pytest.mark.unit
def test_public_hooks_cannot_change_private_edits_across_threads() -> None:
    row = _row()
    expected = _old((row,), "declarations", documents.DeclarationEdit)
    barrier = Barrier(2)

    def run() -> bytes:
        caller = codec.converter()
        caller.register_unstructure_hook(SemanticId, lambda _: "corrupted")
        barrier.wait(timeout=10)
        return _generated.encode_declarations((row,))

    with ThreadPoolExecutor(max_workers=2) as executor:
        first, second = executor.submit(run), executor.submit(run)
        assert first.result() == second.result() == expected
    assert codec.converter() is not codec.converter()


@pytest.mark.unit
@pytest.mark.parametrize("value", [float("nan"), float("inf"), float("-inf")])
def test_private_edit_refuses_nonfinite_even_if_attrs_validation_was_bypassed(
    value: float,
) -> None:
    fit = _fit()
    # Simulate an invalid foreign producer, rather than normal attrs construction.
    object.__setattr__(fit.parameters[0], "value", value)
    with pytest.raises(msgspec.ValidationError, match="finite number"):
        _generated.encode_fits((fit,))


@pytest.mark.unit
def test_private_edit_checks_nested_types_and_accepts_only_generated_rows() -> None:
    with pytest.raises(TypeError, match="generated"):
        _generated.encode_declarations((object(),))  # pyrefly: ignore[bad-argument-type]
    invalid = _row()
    object.__setattr__(invalid, "ordinal", "wrong")
    with pytest.raises(msgspec.ValidationError):
        _generated.encode_declarations((invalid,))
    assert attrs.has(authored.AuthoredModelingDeclarationsRow)
