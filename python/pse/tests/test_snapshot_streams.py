# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Actual admitted local snapshots export bounded, independently owned Arrow data."""

import gc
import shutil
import subprocess
import sys
from pathlib import Path
from typing import get_type_hints

import attrs
import msgspec
import pyarrow as pa
import pytest

import pse
from pse._transfer import compare_schemas
from pse.codec import decode_json
from pse.contracts.extension_types import PseSemanticId
from pse.contracts.manifest import Manifest
from pse.tests.test_native_registry import StoreIndex


@pytest.fixture(scope="session")
def inspection_settings(tmp_path_factory: pytest.TempPathFactory) -> pse.EngineSettings:
    # Full-registry fixture ceiling: tests/support/workflow_budget.rs.
    return pse.EngineSettings(
        memory_limit_bytes=32 << 30,
        threads=1,
        spill_dir=str(tmp_path_factory.mktemp("inspection-spill")),
        max_spill_bytes=1 << 30,
        batch_size=7,
        max_object_bytes=1 << 30,
        max_control_bytes=64 << 20,
    )


@pytest.fixture
def inspection_store(
    tmp_path: Path, native_inspection_store: Path
) -> tuple[Path, StoreIndex]:
    source = native_inspection_store
    index = decode_json((source / "store-index.json").read_bytes(), StoreIndex)
    path = tmp_path / "store"
    shutil.copytree(source, path)
    _point_ref(path, index, "model")
    return path, index


def _point_ref(path: Path, index: StoreIndex, label: str) -> None:
    target = next(item for item in index.snapshots if item.label == label)
    refs = path / "refs"
    refs.mkdir(exist_ok=True)
    # This test changes the external ref wire, never a live immutable handle.
    (refs / "main.json").write_bytes(msgspec.json.encode(target.reference))


def _manifest(path: Path, index: StoreIndex) -> Manifest:
    target = next(item for item in index.snapshots if item.label == "model")
    checksum = target.reference.manifest_checksum.removeprefix("blake3:")
    return decode_json((path / "manifests" / f"{checksum}.json").read_bytes(), Manifest)


def _assert_released(store: pse.Store) -> None:
    gc.collect()
    assert store.resource_usage().reserved_bytes == 0


@pytest.mark.component
def test_python_snapshot_streams_values_schema_empty_and_final_array_lease(
    inspection_store: tuple[Path, StoreIndex], inspection_settings: pse.EngineSettings
) -> None:
    path, index = inspection_store
    store = pse.open(path, settings=inspection_settings)
    _assert_released(store)
    snapshot = store.head()
    manifest = _manifest(path, index)
    expected = next(row for row in manifest.relations if row.name == "schema_relations")
    assert snapshot.snapshot_id == manifest.snapshot_id
    assert ("reference.schema_relations", expected.port) in snapshot.tables()
    with (
        snapshot.table("authored.packages") as empty,
        pa.RecordBatchReader.from_stream(empty) as reader,
    ):
        assert "package_id" in reader.schema.names
        assert list(reader) == []
    stream = snapshot.table("reference", "schema_relations")
    with pytest.raises(pse.InspectionError, match="casts are unsupported"):
        stream.__arrow_c_stream__(pa.schema(stream).__arrow_c_schema__())
    reader = pa.RecordBatchReader.from_stream(stream)
    with pytest.raises(pse.InspectionError):
        pa.RecordBatchReader.from_stream(stream)
    assert all(
        item.state == "retained" for item in stream.extension_report(reader.schema)
    )
    batches = list(reader)
    assert len(batches) > 1
    assert all(0 < batch.num_rows <= 7 for batch in batches)
    assert sum(batch.num_rows for batch in batches) == expected.rows
    values = [row for batch in batches for row in batch.to_pylist()]
    package = next(
        row
        for row in values
        if row["namespace"] == "authored" and row["name"] == "packages"
    )
    assert package["namespace"] == "authored"
    assert package["primary_key"] == ["package_id"]
    # Retain a sliced primitive buffer after every source and reader is closed.
    retained = batches[0].column("name").slice(0, 1)
    expected_name = retained.to_pylist()
    batches.clear()
    reader.close()
    stream.close()
    snapshot.close()
    store.close()
    gc.collect()
    assert retained.to_pylist() == expected_name
    assert store.resource_usage().reserved_bytes > 0
    del retained
    _assert_released(store)


@pytest.mark.component
@pytest.mark.parametrize("cancel", [False, True])
def test_partial_reader_release_and_cancellation_preserve_only_exported_owners(
    inspection_store: tuple[Path, StoreIndex],
    inspection_settings: pse.EngineSettings,
    *,
    cancel: bool,
) -> None:
    path, _ = inspection_store
    store = pse.open(path, settings=inspection_settings)
    snapshot = store.head()
    stream = snapshot.table("reference.schema_columns")
    reader = pa.RecordBatchReader.from_stream(stream)
    batch = reader.read_next_batch()
    snapshot.close()
    store.close()
    if cancel:
        stream.cancel()
        with pytest.raises(pa.ArrowException, match="runtime::cancelled"):
            reader.read_next_batch()
    reader.close()
    # Keeping the Python stream object itself must not pin unread source batches.
    assert store.resource_usage().reserved_bytes > 0
    del batch
    _assert_released(store)
    assert get_type_hints(type(stream))


@pytest.mark.component
def test_ref_movement_does_not_change_pinned_handles_or_exported_streams(
    inspection_store: tuple[Path, StoreIndex], inspection_settings: pse.EngineSettings
) -> None:
    path, index = inspection_store
    store = pse.open(path, settings=inspection_settings)
    original = store.head()
    identity = original.snapshot_id
    stream = original.table("reference.schema_relations")
    _point_ref(path, index, "case")
    current = store.head()
    assert current.snapshot_id != identity
    assert original.snapshot_id == identity
    assert "authored.cases" in {name for name, _ in current.tables()}
    original.close()
    current.close()
    store.close()
    with pa.RecordBatchReader.from_stream(stream) as reader:
        count = sum(batch.num_rows for batch in reader)
    assert count > 0
    _assert_released(store)


@pytest.mark.component
def test_malformed_store_fails_admission_before_any_table_is_exposed(
    inspection_store: tuple[Path, StoreIndex], inspection_settings: pse.EngineSettings
) -> None:
    path, index = inspection_store
    manifest = _manifest(path, index)
    member = next(row for row in manifest.relations if row.name == "schema_relations")
    encoding = next(
        item for item in member.encodings if item.format == "arrow_ipc_file"
    )
    artifact = path / encoding.path
    data = bytearray(artifact.read_bytes())
    data[0] ^= 1
    artifact.write_bytes(data)
    store = pse.open(path, settings=inspection_settings)
    with pytest.raises(pse.InspectionError) as failure:
        store.head()
    assert failure.value.code is not None
    assert failure.value.args
    store.close()
    _assert_released(store)


@pytest.mark.component
def test_open_is_existing_only_explicit_and_has_no_mutation_surface(
    tmp_path: Path, inspection_settings: pse.EngineSettings
) -> None:
    missing = tmp_path / "absent"
    with pytest.raises(pse.InspectionError):
        pse.open(missing, settings=inspection_settings)
    assert not missing.exists()
    store = pse.open(tmp_path, settings=inspection_settings)
    for name in ("compile", "solve", "commit", "publish", "write", "query"):
        assert not hasattr(store, name)
    name = "unexpected"
    with pytest.raises(attrs.exceptions.FrozenInstanceError):
        setattr(store, name, 1)
    store.cancel()
    with pytest.raises(pse.InspectionError, match="runtime::cancelled"):
        store.head()
    store.close()
    _assert_released(store)


@pytest.mark.component
def test_tiny_explicit_budget_refuses_actual_admission_in_a_fresh_process(
    inspection_store: tuple[Path, StoreIndex], tmp_path: Path
) -> None:
    path, _ = inspection_store
    program = """
import sys
import pse
settings = pse.EngineSettings(memory_limit_bytes=1, threads=1, spill_dir=sys.argv[2],
    max_spill_bytes=1048576, batch_size=7, max_object_bytes=1073741824,
    max_control_bytes=67108864)
store = pse.open(sys.argv[1], settings=settings)
try:
    store.head()
except pse.InspectionError as error:
    assert 'resource_limit' in str(error), str(error)
else:
    raise AssertionError('an admitted snapshot escaped a one-byte limit')
assert store.resource_usage().reserved_bytes == 0
"""
    completed = subprocess.run(
        [sys.executable, "-c", program, str(path), str(tmp_path)],
        capture_output=True,
        text=True,
        check=False,
        timeout=120,
    )
    assert completed.returncode == 0, completed.stderr


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


@pytest.mark.unit
def test_inspection_annotations_evaluate_on_the_running_interpreter() -> None:
    for target in (
        pse.FieldTransfer,
        pse.ResourceUsage,
        pse.TableStream,
        pse.Snapshot,
        pse.Store,
        pse.open,
        pse.Store.head,
        pse.Store.snapshot,
        pse.Snapshot.table,
        pse.TableStream.extension_report,
    ):
        assert get_type_hints(target), target


@pytest.mark.component
def test_close_before_export_and_exception_unwind_release_unread_sources(
    inspection_store: tuple[Path, StoreIndex], inspection_settings: pse.EngineSettings
) -> None:
    path, _ = inspection_store
    store = pse.open(path, settings=inspection_settings)
    snapshot = store.head()
    closed = snapshot.table("reference.schema_columns")
    closed.close()
    with pytest.raises(pse.InspectionError, match="closed"):
        closed.__arrow_c_stream__()
    cancelled = snapshot.table("reference.schema_columns")
    cancelled.cancel()
    with pytest.raises(pse.InspectionError, match="runtime::cancelled"):
        cancelled.__arrow_c_stream__()
    stream = snapshot.table("reference.schema_columns")
    reader = pa.RecordBatchReader.from_stream(stream)
    retained = reader.read_next_batch()
    snapshot.close()
    store.close()
    message = "consumer stopped"
    with pytest.raises(ValueError, match="consumer stopped"), stream:
        raise ValueError(message)
    # Ordinary close is EOF; explicit cancellation above is a typed failure.
    with pytest.raises(StopIteration):
        reader.read_next_batch()
    reader.close()
    assert retained.num_rows > 0
    assert store.resource_usage().reserved_bytes > 0
    del retained
    _assert_released(store)


@pytest.mark.component
def test_exact_manifest_open_and_streams_never_write_or_repair_the_store(
    inspection_store: tuple[Path, StoreIndex], inspection_settings: pse.EngineSettings
) -> None:
    path, index = inspection_store
    target = next(item for item in index.snapshots if item.label == "model")
    before = {
        file.relative_to(path): file.read_bytes()
        for file in path.rglob("*")
        if file.is_file()
    }
    store = pse.open(path, settings=inspection_settings)
    snapshot = store.snapshot(
        target.reference.snapshot_id, target.reference.manifest_checksum
    )
    with pytest.raises(pse.InspectionError):
        snapshot.table("reference.schema_columns", port="absent")
    with pytest.raises(pse.InspectionError):
        snapshot.table("absent.relation")
    with (
        snapshot.table("reference.schema_columns") as stream,
        pa.RecordBatchReader.from_stream(stream) as reader,
    ):
        assert sum(batch.num_rows for batch in reader) > 0
    snapshot.close()
    store.close()
    after = {
        file.relative_to(path): file.read_bytes()
        for file in path.rglob("*")
        if file.is_file()
    }
    assert after == before
    _assert_released(store)


@pytest.mark.unit
def test_engine_settings_preserve_independent_worker_and_partition_policies(
    tmp_path: Path,
) -> None:
    settings = pse.EngineSettings(
        memory_limit_bytes=32 << 30,
        threads=2,
        target_partitions=8,
        spill_dir=str(tmp_path),
        max_spill_bytes=16 << 30,
        top_consumers=5,
        batch_size=1024,
        spill_compression="zstd",
        max_spill_file_size_bytes=2 << 30,
        sort_spill_reservation_bytes=16 << 20,
        time_zone="Europe/Brussels",
        max_object_bytes=32 << 30,
        max_control_bytes=64 << 20,
        max_rows=200_000_000,
        max_normalized_bytes=32 << 30,
        hashing_may_use_pool=True,
    )
    assert settings.threads == 2
    assert settings.target_partitions == 8
    assert settings.spill_compression == "zstd"
    assert settings.time_zone == "Europe/Brussels"
    assert settings.top_consumers == 5
    assert settings.max_rows == 200_000_000
    assert settings.max_normalized_bytes == 32 << 30
    assert settings.hashing_may_use_pool
    name = "threads"
    with pytest.raises(AttributeError):
        setattr(settings, name, 7)


@pytest.mark.unit
@pytest.mark.parametrize(
    "field",
    [
        "threads",
        "target_partitions",
        "batch_size",
        "max_object_bytes",
        "max_control_bytes",
    ],
)
def test_engine_settings_reject_invalid_explicit_limits(
    tmp_path: Path, field: str
) -> None:
    with pytest.raises(pse.InspectionError):
        pse.EngineSettings(
            memory_limit_bytes=32 << 30,
            threads=0 if field == "threads" else 1,
            target_partitions=0 if field == "target_partitions" else 1,
            spill_dir=str(tmp_path),
            max_spill_bytes=1 << 30,
            batch_size=0 if field == "batch_size" else 7,
            max_object_bytes=0 if field == "max_object_bytes" else 1 << 30,
            max_control_bytes=0 if field == "max_control_bytes" else 64 << 20,
        )
