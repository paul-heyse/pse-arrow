# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Exact Delta publications export bounded, independently owned Arrow data."""

import gc
import shutil
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from threading import Barrier, Event
from typing import get_type_hints

import attrs
import pyarrow as pa
import pytest

import pse
from pse.tests.test_native_registry import PublicationIndex, publication_index


@pytest.fixture
def inspection_publication(native_inspection_publication: Path) -> PublicationIndex:
    return publication_index(native_inspection_publication)


def _open(index: PublicationIndex, settings: pse.EngineSettings) -> pse.Publication:
    return pse.open(index.root.location, version=index.root.version, settings=settings)


def _assert_released(publication: pse.Publication) -> None:
    gc.collect()
    caches = publication.cache_usage()
    assert all(row.pinned_bytes in (None, 0) for row in caches)
    assert all(row.active_loads in (None, 0) for row in caches)
    assert all(row.inflight_bytes in (None, 0) for row in caches)
    cached = sum(row.capacity_bytes + (row.live_bytes or 0) for row in caches)
    assert publication.resource_usage().pool_reserved_now == cached


def _assert_exported_owner(publication: pse.Publication) -> None:
    caches = publication.cache_usage()
    cached = sum(row.capacity_bytes + (row.live_bytes or 0) for row in caches)
    assert publication.resource_usage().pool_reserved_now > cached or any(
        row.pinned_bytes is not None and row.pinned_bytes > 0 for row in caches
    )


@pytest.mark.component
def test_publication_streams_values_schema_empty_and_final_array_lease(
    inspection_publication: PublicationIndex, inspection_settings: pse.EngineSettings
) -> None:
    index = inspection_publication
    publication = _open(index, inspection_settings)
    assert publication.version == index.root.version
    assert publication.location == index.root.location
    assert {
        (name.catalog, name.schema, name.table) for name in publication.tables()
    } == set(index.tables)
    with (
        publication.table("artifact", "authored", "packages") as empty,
        pa.RecordBatchReader.from_stream(empty) as reader,
    ):
        assert "package_id" in reader.schema.names
        assert list(reader) == []
    stream = publication.table("artifact", "reference", "schema_relations")
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
    values = [row for batch in batches for row in batch.to_pylist()]
    assert len({row["relation_id"] for row in values}) == len(values)
    package = next(
        row
        for row in values
        if row["namespace"] == "authored" and row["name"] == "packages"
    )
    assert package["primary_key"] == ["package_id"]
    retained = batches[0].column("name").slice(0, 1)
    expected_name = retained.to_pylist()
    batches.clear()
    reader.close()
    stream.close()
    publication.close()
    gc.collect()
    assert retained.to_pylist() == expected_name
    _assert_exported_owner(publication)
    del retained
    _assert_released(publication)


@pytest.mark.component
@pytest.mark.parametrize("cancel", [False, True])
def test_partial_reader_release_and_cancellation_preserve_only_exported_owners(
    inspection_publication: PublicationIndex,
    inspection_settings: pse.EngineSettings,
    *,
    cancel: bool,
) -> None:
    publication = _open(inspection_publication, inspection_settings)
    stream = publication.table("artifact", "reference", "schema_columns")
    reader = pa.RecordBatchReader.from_stream(stream)
    batch = reader.read_next_batch()
    publication.close()
    if cancel:
        stream.cancel()
        with pytest.raises(pa.ArrowException, match="runtime::cancelled"):
            reader.read_next_batch()
    reader.close()
    _assert_exported_owner(publication)
    del batch
    _assert_released(publication)
    assert get_type_hints(type(stream))


@pytest.mark.component
def test_exact_selection_and_stream_ownership_survive_handle_closure(
    inspection_publication: PublicationIndex, inspection_settings: pse.EngineSettings
) -> None:
    publication = _open(inspection_publication, inspection_settings)
    stream = publication.table("artifact", "reference", "schema_relations")
    original_version = publication.version
    independently_opened = _open(inspection_publication, inspection_settings)
    assert independently_opened.version == original_version
    publication.close()
    independently_opened.close()
    with pa.RecordBatchReader.from_stream(stream) as reader:
        assert sum(batch.num_rows for batch in reader) > 0
    _assert_released(publication)


@pytest.mark.component
def test_malformed_control_log_fails_before_members_are_exposed(
    native_inspection_publication: Path,
    inspection_publication: PublicationIndex,
    inspection_settings: pse.EngineSettings,
    tmp_path: Path,
) -> None:
    # Only the control table is copied. Member locations remain exact original URIs.
    path = tmp_path / "control"
    shutil.copytree(native_inspection_publication / "control", path)
    selected = path / "_delta_log" / f"{inspection_publication.root.version:020}.json"
    selected.write_bytes(b"invalid delta action\n")
    with pytest.raises(pse.InspectionError) as failure:
        pse.open(
            path,
            version=inspection_publication.root.version,
            settings=inspection_settings,
        )
    assert failure.value.report.code is not None
    assert failure.value.args


@pytest.mark.component
def test_open_is_existing_only_explicit_and_immutable(
    tmp_path: Path,
    inspection_publication: PublicationIndex,
    inspection_settings: pse.EngineSettings,
) -> None:
    missing = tmp_path / "absent"
    with pytest.raises(pse.InspectionError):
        pse.open(missing, version=0, settings=inspection_settings)
    assert not missing.exists()
    with pytest.raises(pse.InspectionError):
        pse.open(
            inspection_publication.root.location,
            version=-1,
            settings=inspection_settings,
        )
    publication = _open(inspection_publication, inspection_settings)
    for name in ("compile", "solve", "commit", "publish", "write", "query"):
        assert not hasattr(publication, name)
    name = "unexpected"
    with pytest.raises(attrs.exceptions.FrozenInstanceError):
        setattr(publication, name, 1)
    publication.close()
    with pytest.raises(pse.InspectionError, match="closed"):
        publication.tables()
    _assert_released(publication)


@pytest.mark.component
def test_tiny_explicit_budget_refuses_actual_admission_in_a_fresh_process(
    inspection_publication: PublicationIndex, tmp_path: Path
) -> None:
    program = """
import sys
import time
import pse
settings = pse.EngineSettings(memory_limit_bytes=1, threads=1, spill_dir=sys.argv[3],
    max_spill_bytes=1048576, batch_size=7)
try:
    pse.open(sys.argv[1], version=int(sys.argv[2]), settings=settings)
except pse.InspectionError as error:
    assert 'resource_limit' in str(error), str(error)
else:
    raise AssertionError('an admitted publication escaped a one-byte limit')
"""
    completed = subprocess.run(
        [
            sys.executable,
            "-c",
            program,
            inspection_publication.root.location,
            str(inspection_publication.root.version),
            str(tmp_path),
        ],
        capture_output=True,
        text=True,
        check=False,
        timeout=120,
    )
    assert completed.returncode == 0, completed.stderr


@pytest.mark.unit
def test_inspection_annotations_evaluate_on_the_running_interpreter() -> None:
    for target in (
        pse.FieldTransfer,
        pse.TableStream,
        pse.Publication,
        pse.open,
        pse.Publication.table,
        pse.TableStream.extension_report,
    ):
        assert get_type_hints(target), target
    # Native properties are typed by the stub emitted from the compiled API.
    # They deliberately have no duplicate runtime Python declaration.
    report_stub = (
        Path(pse.__file__).with_name("_native.pyi").read_text(encoding="utf-8")
    )
    assert "def pool_reserved_now(self, /) -> int:" in report_stub
    assert hasattr(pse.ResourceReport, "pool_reserved_now")


@pytest.mark.component
def test_close_before_export_and_exception_unwind_release_unread_sources(
    inspection_publication: PublicationIndex, inspection_settings: pse.EngineSettings
) -> None:
    publication = _open(inspection_publication, inspection_settings)
    closed = publication.table("artifact", "reference", "schema_columns")
    closed.close()
    with pytest.raises(pse.InspectionError, match="closed"):
        closed.__arrow_c_stream__()
    cancelled = publication.table("artifact", "reference", "schema_columns")
    cancelled.cancel()
    with pytest.raises(pse.InspectionError, match="runtime::cancelled"):
        cancelled.__arrow_c_stream__()
    stream = publication.table("artifact", "reference", "schema_columns")
    reader = pa.RecordBatchReader.from_stream(stream)
    retained = reader.read_next_batch()
    publication.close()
    message = "consumer stopped"
    with pytest.raises(ValueError, match="consumer stopped"), stream:
        raise ValueError(message)
    with pytest.raises(StopIteration):
        reader.read_next_batch()
    reader.close()
    assert retained.num_rows > 0
    _assert_exported_owner(publication)
    del retained
    _assert_released(publication)


@pytest.mark.component
def test_exact_open_and_streams_never_write_or_repair_delta_tables(
    native_inspection_publication: Path,
    inspection_publication: PublicationIndex,
    inspection_settings: pse.EngineSettings,
) -> None:
    path = native_inspection_publication
    before = {
        file.relative_to(path): file.read_bytes()
        for file in path.rglob("*")
        if file.is_file()
    }
    publication = _open(inspection_publication, inspection_settings)
    with pytest.raises(pse.InspectionError):
        publication.table("artifact", "reference", "absent")
    with pytest.raises(pse.InspectionError):
        publication.table("wrong_catalog", "reference", "schema_columns")
    with (
        publication.table("artifact", "reference", "schema_columns") as stream,
        pa.RecordBatchReader.from_stream(stream) as reader,
    ):
        assert sum(batch.num_rows for batch in reader) > 0
    publication.close()
    after = {
        file.relative_to(path): file.read_bytes()
        for file in path.rglob("*")
        if file.is_file()
    }
    assert after == before
    _assert_released(publication)


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
        hashing_may_use_pool=True,
        concurrent_queries=1,
        concurrent_outputs=3,
        model_result_bytes=1024,
    )
    assert settings.threads == 2
    assert settings.target_partitions == 8
    assert settings.spill_compression == "zstd"
    assert settings.time_zone == "Europe/Brussels"
    assert settings.top_consumers == 5
    assert settings.hashing_may_use_pool
    assert settings.concurrent_queries == 1
    assert settings.concurrent_outputs == 3
    assert settings.model_result_bytes == 1024
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
        "memory_limit_bytes",
    ],
)
def test_engine_settings_reject_invalid_explicit_limits(
    tmp_path: Path, field: str
) -> None:
    with pytest.raises(pse.InspectionError):
        pse.EngineSettings(
            memory_limit_bytes=0 if field == "memory_limit_bytes" else 32 << 30,
            threads=0 if field == "threads" else 1,
            target_partitions=0 if field == "target_partitions" else 1,
            spill_dir=str(tmp_path),
            max_spill_bytes=1 << 30,
            batch_size=0 if field == "batch_size" else 7,
        )


@pytest.mark.unit
def test_engine_settings_preserve_zero_native_spill_capacity(tmp_path: Path) -> None:
    settings = pse.EngineSettings(
        memory_limit_bytes=32 << 30,
        threads=1,
        spill_dir=str(tmp_path),
        max_spill_bytes=0,
        batch_size=7,
    )
    assert settings.max_spill_bytes == 0


def _two_reader_progress(publication: pse.Publication) -> dict[str, int | float]:
    """Measure whole reader tasks and progress; never infer a particular GIL span."""
    start = Barrier(3)
    stop = Event()
    finished = [Event(), Event()]
    iterations = 16

    def read(index: int) -> tuple[pa.Table, float]:
        start.wait(timeout=30)
        started = time.perf_counter()
        reference: pa.Table | None = None
        try:
            for _ in range(iterations):
                with (
                    publication.table(
                        "artifact", "reference", "schema_relations"
                    ) as stream,
                    pa.RecordBatchReader.from_stream(stream) as reader,
                ):
                    actual = reader.read_all()
                if reference is None:
                    reference = actual
                else:
                    assert actual.equals(reference, check_metadata=True)
            assert reference is not None
            return reference, time.perf_counter() - started
        finally:
            finished[index].set()

    def python_work() -> tuple[int, int]:
        start.wait(timeout=30)
        cycles = 0
        overlapping_cycles = 0
        while not stop.wait(0.0001):
            # Count completed independent work, not a nonzero initial value.
            assert sum(range(1000)) == 499500
            cycles += 1
            if not finished[0].is_set() and not finished[1].is_set():
                overlapping_cycles += 1
        return cycles, overlapping_cycles

    started = time.perf_counter()
    with ThreadPoolExecutor(max_workers=3) as workers:
        first = workers.submit(read, 0)
        second = workers.submit(read, 1)
        progress = workers.submit(python_work)
        try:
            left, left_seconds = first.result(timeout=60)
            right, right_seconds = second.result(timeout=60)
        finally:
            stop.set()
        cycles, overlapping_cycles = progress.result(timeout=30)
    assert cycles > 0
    assert overlapping_cycles > 0
    assert left.equals(right, check_metadata=True)
    return {
        "reader_tasks": 2,
        "streams_per_reader": iterations,
        "rows_per_stream": left.num_rows,
        "left_task_seconds": left_seconds,
        "right_task_seconds": right_seconds,
        "total_seconds": time.perf_counter() - started,
        "python_work_cycles": cycles,
        "cycles_while_both_reader_tasks_active": overlapping_cycles,
        "python_wait_seconds": 0.0001,
    }


@pytest.mark.integration
def test_two_arrow_consumers_and_python_worker_make_independent_progress(
    inspection_publication: PublicationIndex, inspection_settings: pse.EngineSettings
) -> None:
    """I18 actual PyArrow C-stream exercise; never an implementation unit gate."""
    publication = _open(inspection_publication, inspection_settings)
    try:
        _two_reader_progress(publication)
    finally:
        publication.close()
