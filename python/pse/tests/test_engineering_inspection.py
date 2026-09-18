# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Cold Python admission and owned streams of actual source-produced native graphs."""

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
    CompiledMathExprNodesRow,
    CompiledMathIndexedEquationsRow,
    CompiledSymbolsRow,
)
from pse.contracts.enums import SymbolRole
from pse.tests.test_native_registry import PublicationRoot
from pse.tests.test_publication_streams import (
    _assert_exported_owner,
    _assert_released,
)

T = TypeVar("T")


class EngineeringMeasurement(msgspec.Struct, forbid_unknown_fields=True):
    label: str
    publication: PublicationRoot
    state_template: str
    expected_states: int
    planning_seconds: float
    execution_seconds: float
    rust_reopen_seconds: float
    memory_limit_bytes: int
    compile_peak_bytes: int
    reopen_peak_bytes: int
    process_peak_rss_bytes: int | None
    threads: int
    partitions: int
    batch_size: int
    relation_rows: dict[str, int]


def _rows(snapshot: pse.Publication, name: str, cls: type[T]) -> list[T]:
    with (
        snapshot.table("artifact", *name.split(".", 1)) as stream,
        pa.RecordBatchReader.from_stream(stream) as reader,
    ):
        rows: list[dict[str, object]] = [
            row for batch in reader for row in batch.to_pylist()
        ]
    return structure_rows(rows, cls)


def _check_equations(snapshot: pse.Publication, label: str) -> None:
    equations = _rows(
        snapshot, "compiled.math_indexed_equations", CompiledMathIndexedEquationsRow
    )
    nodes = _rows(snapshot, "compiled.math_expr_nodes", CompiledMathExprNodesRow)
    by_id = {node.node_id: node for node in nodes}
    assert len(by_id) == len(nodes) > 0
    assert all(child in by_id for node in nodes for child in node.children)
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


def _check_symbols(
    snapshot: pse.Publication, measurement: EngineeringMeasurement
) -> None:
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


def _check_streams(snapshot: pse.Publication, expected: dict[str, int]) -> None:
    tables = snapshot.tables()
    assert {f"{schema}.{table}" for _, schema, table in tables} == set(expected)
    for catalog, schema, table in tables:
        with (
            snapshot.table(catalog, schema, table) as stream,
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
            assert count == expected[f"{schema}.{table}"]


def _retained_node(snapshot: pse.Publication) -> pa.Array:
    with (
        snapshot.table("artifact", "compiled", "math_expr_nodes") as stream,
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
def test_source_native_cold_python_inspection_and_final_export_owner(
    tmp_path: Path,
) -> None:
    configured = os.environ.get("PSE_ENGINEERING_PUBLICATION")
    if configured is None:
        pytest.fail(
            "Run just engineering-inspection <new-output-directory>.", pytrace=False
        )
    path = Path(configured)
    measurement = decode_json(
        (path / "engineering.json").read_bytes(), EngineeringMeasurement
    )
    before = _store_files(path)
    spill = tmp_path / "spill"
    spill.mkdir()
    settings = pse.EngineSettings(
        memory_limit_bytes=measurement.memory_limit_bytes,
        threads=measurement.threads,
        spill_dir=str(spill),
        max_spill_bytes=1 << 30,
        batch_size=7,
    )
    started = time.perf_counter()
    snapshot = pse.open(
        measurement.publication.location,
        version=measurement.publication.version,
        settings=settings,
    )
    reopen_seconds = time.perf_counter() - started
    assert snapshot.version == measurement.publication.version
    assert snapshot.location == measurement.publication.location
    started = time.perf_counter()
    _check_streams(snapshot, measurement.relation_rows)
    _check_equations(snapshot, measurement.label)
    _check_symbols(snapshot, measurement)
    stream_seconds = time.perf_counter() - started
    retained = _retained_node(snapshot)
    value = retained.to_pylist()
    snapshot.close()
    gc.collect()
    assert retained.to_pylist() == value
    _assert_exported_owner(snapshot)
    del retained
    gc.collect()
    usage = snapshot.resource_usage()
    _assert_released(snapshot)
    cached = sum(
        row.capacity_bytes + (row.live_bytes or 0) for row in snapshot.cache_usage()
    )
    assert _store_files(path) == before, (
        "cold admission and inspection must not publish or repair Delta tables"
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
                "final_cache_owned_bytes": cached,
            }
        )
    )
