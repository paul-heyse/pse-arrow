# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Published registry Delta values pass the generated Python contracts."""

from pathlib import Path

import attrs
import cattrs
import msgspec
import pyarrow as pa
import pytest

import pse
from pse.codec import decode_json, structure_rows
from pse.contracts.enums import Namespace, SnapshotClass
from pse.contracts.reference import (
    ReferenceSchemaColumnsRow,
    ReferenceSchemaRelationsRow,
)


class PublicationRoot(msgspec.Struct, forbid_unknown_fields=True):
    location: str
    version: int


class PublicationIndex(msgspec.Struct, forbid_unknown_fields=True):
    root: PublicationRoot
    tables: list[tuple[str, str, str]]


def publication_index(path: Path) -> PublicationIndex:
    return decode_json((path / "publication-index.json").read_bytes(), PublicationIndex)


def stored_rows(
    path: Path, name: str, settings: pse.EngineSettings
) -> list[dict[str, object]]:
    root = publication_index(path).root
    with (
        pse.open(root.location, version=root.version, settings=settings) as publication,
        publication.table("artifact", "reference", name) as stream,
        pa.RecordBatchReader.from_stream(stream) as reader,
    ):
        rows: list[dict[str, object]] = [
            row for batch in reader for row in batch.to_pylist()
        ]
    assert rows
    return rows


@pytest.mark.component
def test_stored_registry_relations_and_columns_decode_through_generated_contracts(
    native_inspection_publication: Path,
    inspection_settings: pse.EngineSettings,
) -> None:
    relations = structure_rows(
        stored_rows(
            native_inspection_publication, "schema_relations", inspection_settings
        ),
        ReferenceSchemaRelationsRow,
    )
    columns = structure_rows(
        stored_rows(
            native_inspection_publication, "schema_columns", inspection_settings
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
    native_inspection_publication: Path,
    inspection_settings: pse.EngineSettings,
) -> None:
    original = stored_rows(
        native_inspection_publication, "schema_relations", inspection_settings
    )[0]
    changed = dict(original)
    changed["primary_key"] = [17]
    assert changed["relation_id"] == original["relation_id"]
    with pytest.raises(cattrs.BaseValidationError):
        structure_rows([changed], ReferenceSchemaRelationsRow)
