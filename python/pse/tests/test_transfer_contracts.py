# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Pure native-schema transfer observations; no publication/compiler fixtures."""

import pyarrow as pa
import pytest

import pse._transfer as transfer
from pse._transfer import compare_schemas
from pse.contracts.extension_types import PseSemanticId


@pytest.mark.unit
def test_consumer_schema_reports_nested_semantics_and_degradation() -> None:
    extension = PseSemanticId()
    semantic = pa.field(
        "sid", extension, nullable=False, metadata={b"owner": b"actual"}
    )
    source = pa.schema([pa.field("outer", pa.struct([semantic]))])
    assert all(item.state == "retained" for item in compare_schemas(source, source))
    raw = pa.field(
        "sid",
        extension.storage_type,
        nullable=False,
        metadata={
            b"owner": b"actual",
            b"ARROW:extension:name": extension.extension_name.encode(),
            b"ARROW:extension:metadata": extension.__arrow_ext_serialize__(),
        },
    )
    storage = pa.schema([pa.field("outer", pa.struct([raw]))])
    assert compare_schemas(source, storage)[1].state == "storage_only"
    lost = pa.schema(
        [pa.field("outer", pa.struct([raw.with_metadata({b"owner": b"actual"})]))]
    )
    assert compare_schemas(source, lost)[1].state == "metadata_lost"
    assert raw.metadata is not None
    changed = raw.with_metadata({**raw.metadata, b"owner": b"different"})
    mismatch = pa.schema([pa.field("outer", pa.struct([changed]))])
    assert compare_schemas(source, mismatch)[1].state == "mismatch"
    version = raw.with_metadata(
        {**raw.metadata, b"ARROW:extension:metadata": b'{"v":99}'}
    )
    assert (
        compare_schemas(source, pa.schema([pa.field("outer", pa.struct([version]))]))[
            1
        ].state
        == "mismatch"
    )
    wrong = pa.schema(
        [pa.field("outer", pa.struct([pa.field("sid", pa.binary(32), nullable=False)]))]
    )
    assert compare_schemas(source, wrong)[1].state == "mismatch"


@pytest.mark.unit
def test_consumer_schema_report_checks_order_metadata_and_unknown_fields() -> None:
    left, right = pa.field("left", pa.int64()), pa.field("right", pa.int64())
    source = pa.schema([left, right], metadata={b"contract": b"complete"})
    reordered = pa.schema([right, left], metadata=source.metadata)
    assert compare_schemas(source, reordered)[0].state == "mismatch"
    assert compare_schemas(source, source.remove_metadata())[0].state == "mismatch"
    unknown = pa.schema(
        [left, right, pa.field("unknown", pa.int64())], metadata=source.metadata
    )
    assert compare_schemas(source, unknown)[-1].state == "mismatch"


def _nested(kind: str, field: pa.Field) -> pa.DataType:
    match kind:
        case "struct":
            return pa.struct([field])
        case "map-item":
            return pa.map_(pa.string(), field)
        case "map-key":
            return pa.map_(field.with_name("key").with_nullable(False), pa.int64())
        case "union":
            return pa.union([field], mode="dense", type_codes=[7])
        case "list-view":
            return pa.list_view(field)
        case "large-list-view":
            return pa.large_list_view(field)
        case "fixed-list":
            return pa.list_(field, 2)
        case "dictionary":
            return pa.dictionary(pa.int8(), pa.struct([field]))
        case "run-end":
            return pa.run_end_encoded(pa.int16(), pa.struct([field]))
        case _:
            raise AssertionError(kind)


@pytest.mark.unit
@pytest.mark.parametrize(
    "kind",
    [
        "struct",
        "map-item",
        "map-key",
        "union",
        "list-view",
        "large-list-view",
        "fixed-list",
        "dictionary",
        "run-end",
    ],
)
def test_every_native_child_is_observed(kind: str) -> None:
    field = pa.field("value", pa.int64(), metadata={b"meaning": b"first"})
    changed = field.with_metadata({b"meaning": b"second"})
    source = pa.schema([pa.field("root", _nested(kind, field))])
    actual = pa.schema([pa.field("root", _nested(kind, changed))])
    result = compare_schemas(source, actual)
    assert result[0].state == "mismatch"
    child_name = "key" if kind == "map-key" else "value"
    assert any(
        item.path[-1] == child_name and item.state == "mismatch" for item in result
    )
    assert all(item.state == "retained" for item in compare_schemas(source, source))


@pytest.mark.unit
@pytest.mark.parametrize(
    "kind", ["struct", "map-item", "union", "list-view", "dictionary", "run-end"]
)
def test_nested_extensions_retain_all_four_observation_states(kind: str) -> None:
    extension = PseSemanticId()
    field = pa.field("value", extension, nullable=False)
    source = pa.schema([pa.field("root", _nested(kind, field))])
    raw = pa.field(
        "value",
        extension.storage_type,
        nullable=False,
        metadata={
            b"ARROW:extension:name": extension.extension_name.encode(),
            b"ARROW:extension:metadata": extension.__arrow_ext_serialize__(),
        },
    )
    variants = [
        (field, "retained"),
        (raw, "storage_only"),
        (raw.remove_metadata(), "metadata_lost"),
        (raw.with_nullable(True), "mismatch"),
    ]
    for variant, state in variants:
        actual = pa.schema([pa.field("root", _nested(kind, variant))])
        result = compare_schemas(source, actual)
        assert result[0].state == state
        assert result[-1].state == state


@pytest.mark.unit
@pytest.mark.parametrize(
    "types",
    [
        (
            pa.map_(pa.string(), pa.int64(), keys_sorted=False),
            pa.map_(pa.string(), pa.int64(), keys_sorted=True),
        ),
        (
            pa.union([pa.field("x", pa.int64())], mode="dense", type_codes=[1]),
            pa.union([pa.field("x", pa.int64())], mode="dense", type_codes=[2]),
        ),
        (
            pa.union([pa.field("x", pa.int64())], mode="dense"),
            pa.union([pa.field("x", pa.int64())], mode="sparse"),
        ),
        (pa.list_(pa.int64(), 2), pa.list_(pa.int64(), 3)),
        (pa.dictionary(pa.int8(), pa.string()), pa.dictionary(pa.int16(), pa.string())),
        (
            pa.dictionary(pa.int8(), pa.string(), ordered=False),
            pa.dictionary(pa.int8(), pa.string(), ordered=True),
        ),
    ],
)
def test_container_parameters_are_semantic(
    types: tuple[pa.DataType, pa.DataType],
) -> None:
    source, actual = (pa.schema([pa.field("root", ty)]) for ty in types)
    assert compare_schemas(source, actual)[0].state == "mismatch"


@pytest.mark.unit
def test_native_canonical_extensions_are_not_treated_as_plain_storage() -> None:
    source = pa.schema([pa.field("id", pa.uuid())])
    assert compare_schemas(source, source)[0].state == "retained"
    lost = pa.schema([pa.field("id", pa.binary(16))])
    assert compare_schemas(source, lost)[0].state == "metadata_lost"
    opaque = pa.schema(
        [
            pa.field(
                "id",
                pa.binary(16),
                metadata={
                    b"ARROW:extension:name": b"arrow.uuid",
                    b"ARROW:extension:metadata": b"unverified",
                },
            )
        ]
    )
    assert compare_schemas(source, opaque)[0].state == "mismatch"


@pytest.mark.unit
def test_dictionary_value_paths_and_duplicate_fields() -> None:
    source = pa.schema([pa.field("root", pa.dictionary(pa.int8(), pa.json_()))])
    actual = pa.schema([pa.field("root", pa.dictionary(pa.int8(), pa.string()))])
    result = compare_schemas(source, actual)
    assert result[-1].path == ("root", "<dictionary-values>")
    assert result[-1].state == "metadata_lost"
    duplicate = pa.schema(
        [
            pa.field(
                "root",
                pa.struct([pa.field("same", pa.int64()), pa.field("same", pa.int64())]),
            )
        ]
    )
    with pytest.raises(ValueError, match="duplicate nested field path"):
        compare_schemas(duplicate, duplicate)


@pytest.mark.unit
@pytest.mark.parametrize(
    "actual",
    [
        pa.field("renamed", pa.int64()),
        pa.field("value", pa.int32()),
        pa.field("value", pa.int64(), nullable=False),
    ],
)
def test_native_name_type_and_nullability_are_preserved(actual: pa.Field) -> None:
    source = pa.schema([pa.field("root", pa.struct([pa.field("value", pa.int64())]))])
    consumer = pa.schema([pa.field("root", pa.struct([actual]))])
    assert compare_schemas(source, consumer)[0].state == "mismatch"


@pytest.mark.unit
@pytest.mark.parametrize("level", ["field", "schema"])
def test_duplicate_metadata_keys_use_native_equality(level: str) -> None:
    original = pa.KeyValueMetadata([(b"key", b"same"), (b"key", b"original")])
    changed = pa.KeyValueMetadata([(b"key", b"same"), (b"key", b"changed")])
    field = pa.field("value", pa.int64())
    if level == "field":
        source = pa.schema([field.with_metadata(original)])
        consumer = pa.schema([field.with_metadata(changed)])
        assert source.field(0).metadata == consumer.field(0).metadata
    else:
        source = pa.schema([field], metadata=original)
        consumer = pa.schema([field], metadata=changed)
        assert source.metadata == consumer.metadata
    assert all(item.state == "retained" for item in compare_schemas(source, source))
    result = compare_schemas(source, consumer)
    assert result[0].state == "mismatch"
    assert result[0].path == (() if level == "schema" else ("value",))


@pytest.mark.unit
def test_duplicate_schema_metadata_does_not_mask_extension_degradation() -> None:
    metadata = pa.KeyValueMetadata([(b"key", b"first"), (b"key", b"second")])
    source = pa.schema([pa.field("value", pa.uuid())], metadata=metadata)
    consumer = pa.schema([pa.field("value", pa.binary(16))], metadata=metadata)
    result = compare_schemas(source, consumer)
    assert len(result) == 1
    assert result[0].path == ("value",)
    assert result[0].state == "metadata_lost"


@pytest.mark.unit
def test_equal_schema_uses_native_equality_without_rechecking_fields(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    schema = pa.schema([pa.field("outer", pa.struct([pa.field("value", pa.int64())]))])
    expected = transfer.compare_schemas(schema, schema)

    def unexpected(_left: pa.Field, _right: pa.Field) -> transfer.TransferState:
        message = "equal schemas must not recursively compare each field"
        raise AssertionError(message)

    monkeypatch.setattr(transfer, "_state", unexpected)
    assert transfer.compare_schemas(schema, pa.schema(list(schema))) == expected
