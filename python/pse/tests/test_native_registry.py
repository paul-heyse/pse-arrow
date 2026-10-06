# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Rust registry reflection passes the generated Python contracts."""

import attrs
import cattrs
import pyarrow as pa
import pytest

import pse
from pse.codec import structure_rows
from pse.contracts.enums import Namespace, SnapshotClass
from pse.contracts.reference import (
    ReferenceSchemaColumnsRow,
    ReferenceSchemaRelationsRow,
)


def registry_rows(
    name: str, settings: pse.EngineSettings
) -> list[dict[str, object]]:
    with (
        pse.registry_table(f"reference.{name}", settings=settings) as stream,
        pa.RecordBatchReader.from_stream(stream) as reader,
    ):
        rows: list[dict[str, object]] = [
            row for batch in reader for row in batch.to_pylist()
        ]
    assert rows
    return rows


@pytest.mark.component
def test_stored_registry_relations_and_columns_decode_through_generated_contracts(
    inspection_settings: pse.EngineSettings,
) -> None:
    relations = structure_rows(
        registry_rows(
            "schema_relations", inspection_settings
        ),
        ReferenceSchemaRelationsRow,
    )
    columns = structure_rows(
        registry_rows(
            "schema_columns", inspection_settings
        ),
        ReferenceSchemaColumnsRow,
    )
    identities = {row.relation_id for row in relations}
    assert len(identities) == len(relations)
    assert all(row.relation_id in identities for row in columns)
    package = next(
        row
        for row in relations
        if row.namespace == Namespace.AUTHORED and row.name == "packages"
    )
    assert package.primary_key == ("package_id",)
    assert package.snapshot_class == SnapshotClass.MODEL
    assert any(
        row.relation_id == package.relation_id and row.name == "dependencies"
        for row in columns
    )
    assert all(attrs.has(type(row)) for row in relations)


@pytest.mark.component
def test_stored_registry_values_are_checked_even_when_identity_is_unchanged(
    inspection_settings: pse.EngineSettings,
) -> None:
    original = registry_rows(
        "schema_relations", inspection_settings
    )[0]
    changed = dict(original)
    changed["primary_key"] = [17]
    assert changed["relation_id"] == original["relation_id"]
    with pytest.raises(cattrs.BaseValidationError):
        structure_rows([changed], ReferenceSchemaRelationsRow)
