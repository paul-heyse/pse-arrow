# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Cold Python admission and owned streams of actual source-produced P10 graphs."""

import gc
import hashlib
import os
import time
from pathlib import Path
from typing import TypeVar

import msgspec
import pyarrow as pa
import pytest

import pse
from pse.codec import decode_json, structure_rows
from pse.contracts.compiled import (
    CompiledMathExprArgsRow,
    CompiledMathExprNodesRow,
    CompiledMathIndexedEquationsRow,
    CompiledSymbolsRow,
)
from pse.contracts.enums import SymbolRole
from pse.tests.test_native_registry import SnapshotReference

T = TypeVar("T")


class EngineeringMeasurement(msgspec.Struct, forbid_unknown_fields=True):
    label: str
    p3: SnapshotReference
    p10: SnapshotReference
    state_template: str
    expected_states: int
    commit_seconds: float
    compile_seconds: float
    rust_reopen_seconds: float
    memory_limit_bytes: int
    compile_peak_bytes: int
    reopen_peak_bytes: int
    process_peak_rss_bytes: int | None
    threads: int
    partitions: int
    batch_size: int
    stage_rows: dict[str, dict[str, int]]


def _rows(snapshot: pse.Snapshot, name: str, cls: type[T]) -> list[T]:
    with (
        snapshot.table(name) as stream,
        pa.RecordBatchReader.from_stream(stream) as reader,
    ):
        rows: list[dict[str, object]] = [
            row for batch in reader for row in batch.to_pylist()
        ]
    return structure_rows(rows, cls)


def _check_equations(snapshot: pse.Snapshot, label: str) -> None:
    equations = _rows(
        snapshot, "compiled.math_indexed_equations", CompiledMathIndexedEquationsRow
    )
    nodes = _rows(snapshot, "compiled.math_expr_nodes", CompiledMathExprNodesRow)
    args = _rows(snapshot, "compiled.math_expr_args", CompiledMathExprArgsRow)
    by_id = {node.node_id: node for node in nodes}
    assert len(by_id) == len(nodes) > 0
    assert all(
        arg.parent_node_id in by_id and arg.child_node_id in by_id for arg in args
    )
    assert all(
        by_id[row.body_node_id].quantity_type_id is not None for row in equations
    )
    assert all(row.residual_quantity_type_id is not None for row in equations)
    assert sum(row.law_instance_id is not None for row in equations) == 2
    declarations = [
        row.equation_decl_id.hex() for row in equations if row.equation_decl_id
    ]
    closure = (
        "c2c19be5d5204d80ab99802de54fa4cc"
        if label.endswith("fctp")
        else "809bcfe6144e4283a80b4affe686fa10"
    )
    assert declarations.count(closure) == 1
    pressure = (
        "7075234cdb2c4a86868e9e93b4974fc3"
        if label.startswith("heater")
        else "805b27eb8d654b70bc5e5f7bd8f1f9c3"
    )
    assert declarations.count(pressure) == 1
    assert "85eedc866a5a406183e8c5d5c9e2445f" not in declarations
    assert "d21198887b734bdaa6d975d784eb6001" not in declarations


def _check_symbols(snapshot: pse.Snapshot, measurement: EngineeringMeasurement) -> None:
    symbols = _rows(snapshot, "compiled.symbols", CompiledSymbolsRow)
    temperature = (
        "eb889bdb424c416d9e889da639a36879"
        if measurement.label.endswith("fctp")
        else "75d55d1c66da4e97bd17de4480d959d5"
    )
    states = [
        symbol for symbol in symbols if symbol.symbol_decl_id.hex() == temperature
    ]
    assert len(states) == measurement.expected_states
    assert (
        len({symbol.owner_instance_id for symbol in states})
        == measurement.expected_states
    )
    assert all(
        symbol.role == SymbolRole.VARIABLE and symbol.index == () for symbol in states
    )
    declarations = [symbol.symbol_decl_id.hex() for symbol in symbols]
    assert declarations.count("e0d5feb4f68748d6850f57f74b4b2b7f") == int(
        measurement.label.startswith("heater")
    )
    for disabled in (
        "cb57f6f755e04164ad6a9f5a7200f596",
        "ca583148b35e410ba0a7c6172c9ac328",
        "0b77b73c169547eda8ebcb2ca28b840f",
    ):
        assert disabled not in declarations


def _check_streams(snapshot: pse.Snapshot, expected: dict[str, int]) -> None:
    tables = snapshot.tables()
    assert {port for _, port in tables} == set(expected)
    for name, port in tables:
        with (
            snapshot.table(name, port=port) as stream,
            pa.RecordBatchReader.from_stream(stream) as reader,
        ):
            assert all(
                item.state == "retained"
                for item in stream.extension_report(reader.schema)
            )
            count = 0
            for batch in reader:
                assert 0 < batch.num_rows <= 7
                count += batch.num_rows
            assert count == expected[port]


def _retained_node(snapshot: pse.Snapshot) -> pa.Array:
    with (
        snapshot.table("compiled.math_expr_nodes") as stream,
        pa.RecordBatchReader.from_stream(stream) as reader,
    ):
        return reader.read_next_batch().column("node_id").slice(0, 1)


def _store_files(path: Path) -> dict[str, bytes]:
    result = {}
    for file in path.rglob("*"):
        if file.is_file():
            with file.open("rb") as source:
                result[str(file.relative_to(path))] = hashlib.file_digest(
                    source, "sha256"
                ).digest()
    return result


@pytest.mark.integration
def test_source_p10_cold_python_inspection_and_final_export_owner(
    tmp_path: Path,
) -> None:
    configured = os.environ.get("PSE_ENGINEERING_STORE")
    if configured is None:
        pytest.fail(
            "Run just engineering-inspection <new-output-directory>.", pytrace=False
        )
    path = Path(configured)
    measurement = decode_json(
        (path / "engineering.json").read_bytes(), EngineeringMeasurement
    )
    before = _store_files(path)
    settings = pse.EngineSettings(
        memory_limit_bytes=measurement.memory_limit_bytes,
        threads=measurement.threads,
        spill_dir=str(tmp_path / "spill"),
        max_spill_bytes=1 << 30,
        batch_size=7,
        max_object_bytes=1 << 30,
        max_control_bytes=64 << 20,
    )
    store = pse.open(path, settings=settings)
    started = time.perf_counter()
    snapshot = store.snapshot(
        measurement.p10.snapshot_id, measurement.p10.manifest_checksum
    )
    reopen_seconds = time.perf_counter() - started
    assert snapshot.snapshot_id == measurement.p10.snapshot_id
    started = time.perf_counter()
    _check_streams(snapshot, measurement.stage_rows["P10"])
    _check_equations(snapshot, measurement.label)
    _check_symbols(snapshot, measurement)
    stream_seconds = time.perf_counter() - started
    retained = _retained_node(snapshot)
    value = retained.to_pylist()
    snapshot.close()
    store.close()
    gc.collect()
    assert retained.to_pylist() == value
    assert store.resource_usage().reserved_bytes > 0
    del retained
    gc.collect()
    usage = store.resource_usage()
    assert usage.reserved_bytes == 0
    assert _store_files(path) == before, (
        "cold admission and inspection must not publish or repair store objects"
    )
    (path / "engineering-python.json").write_bytes(
        msgspec.json.encode(
            {
                "label": measurement.label,
                "reopen_seconds": reopen_seconds,
                "stream_seconds": stream_seconds,
                "peak_bytes": usage.peak_bytes,
                "process_peak_rss_bytes": usage.process_peak_rss_bytes,
                "final_reserved_bytes": usage.reserved_bytes,
            }
        )
    )
