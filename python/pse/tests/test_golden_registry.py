# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Published registry IPC values pass the generated Python contracts."""

from pathlib import Path

import attrs
import cattrs
import msgspec
import pyarrow as pa
import pyarrow.ipc as ipc
import pytest

from pse.codec import decode_json, structure_rows
from pse.contracts.enums import Namespace, SnapshotClass
from pse.contracts.manifest import Manifest
from pse.contracts.reference import (
    ReferenceSchemaColumnsRow,
    ReferenceSchemaRelationsRow,
)


class SnapshotReference(msgspec.Struct, forbid_unknown_fields=True):
    snapshot_id: str
    manifest_checksum: str


class StoredSnapshot(msgspec.Struct, forbid_unknown_fields=True):
    label: str
    reference: SnapshotReference
    parents: dict[str, str]
    stage_pass: str | None


class StoreIndex(msgspec.Struct, forbid_unknown_fields=True):
    snapshots: list[StoredSnapshot]
    sources: dict[str, str]


def registry_manifest() -> tuple[Path, Manifest]:
    store = Path(__file__).resolve().parents[3] / "tests/golden/registry"
    index = decode_json((store / "store-index.json").read_bytes(), StoreIndex)
    model = next(snapshot for snapshot in index.snapshots if snapshot.label == "model")
    checksum = model.reference.manifest_checksum.removeprefix("blake3:")
    assert len(checksum) == 64
    assert all(c in "0123456789abcdef" for c in checksum)
    manifest = decode_json(
        (store / "manifests" / f"{checksum}.json").read_bytes(), Manifest
    )
    assert manifest.snapshot_id == model.reference.snapshot_id
    return store, manifest


def stored_rows(name: str) -> list[dict[str, object]]:
    store, manifest = registry_manifest()
    member = next(
        row
        for row in manifest.relations
        if row.namespace == "reference" and row.name == name
    )
    encoding = next(
        value for value in member.encodings if value.format == "arrow_ipc_file"
    )
    path = (store / encoding.path).resolve()
    assert path.is_relative_to(store.resolve())
    assert path.stat().st_size == encoding.bytes
    with pa.memory_map(str(path), "r") as source:
        table = ipc.open_file(source).read_all()
        assert table.num_rows == member.rows > 0
        # Values are materialized while the map is alive; generated hooks validate them.
        rows: list[dict[str, object]] = table.to_pylist()
    return rows


@pytest.mark.component
def test_stored_registry_relations_and_columns_decode_through_generated_contracts() -> (
    None
):
    relations = structure_rows(
        stored_rows("schema_relations"), ReferenceSchemaRelationsRow
    )
    columns = structure_rows(stored_rows("schema_columns"), ReferenceSchemaColumnsRow)
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
def test_stored_registry_values_are_checked_even_when_identity_is_unchanged() -> None:
    original = stored_rows("schema_relations")[0]
    changed = dict(original)
    changed["primary_key"] = [17]
    assert changed["relation_id"] == original["relation_id"]
    with pytest.raises(cattrs.BaseValidationError):
        structure_rows([changed], ReferenceSchemaRelationsRow)
