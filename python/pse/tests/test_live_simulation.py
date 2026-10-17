# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Public live simulation uses native samples and independent cursor ownership."""

import asyncio
import math
from pathlib import Path

import pyarrow as pa
import pytest

import pse
from pse.tests.canonical_fixture import CanonicalFixture

SAMPLES = [0.0, 0.25, 0.5, 0.75, 1.0]
SOURCE = """package live {
 def D {
  domain t:Time from 0{s} to 1{s};
  discretize grid on t using integrated(elements=1,order=1);
  param p:Scalar=0.5;
  var x[i in t]:Time;
  eq rate[i in t]:d(x[i])/di == -p*x[i]/1{s};
  eq initial:x[0{s}]==1{s};
  annotation report x("decay");
  annotation check x(x[i]>0{s});
 }
 case run fixture {
  dof 0; route integrated; procedure integrate;
  integrate samples(0{s},0.25{s},0.5{s},0.75{s},1{s}) relative(1e-8)
   normalized_absolute(1e-8) step(1e-4{s});
 } { child root:D=D(); }
}"""


def prepare(
    runtime: pse.Runtime, *, sensitivity: str = "none"
) -> pse.PreparedOperation:
    root = Path(__file__).resolve().parents[3]
    primitives = root / "tests/fixtures/packages/physical-primitives"
    physical = runtime.physical_from_documents(
        {
            str(path.relative_to(primitives)): path.read_text()
            for path in primitives.rglob("*")
            if path.is_file()
        }
    )
    manifest = (
        (root / "tests/fixtures/packages/minimal_explicit/package.toml")
        .read_text()
        .replace('id_policy = "explicit"', 'id_policy = "named"')
        .replace(
            "dependencies = []",
            'dependencies = [{ package_id = "5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a", '
            'version_req = { operator = "exact", major = 1, minor = 0, patch = 0 } }]',
        )
    )
    package = runtime.modeling_from_documents(
        [{"package.toml": manifest, "models/live.pse": SOURCE}], physical
    )
    case = next(
        row.declaration_id for row in package.declarations() if row.name == "run"
    )
    return package.prepare_simulation(
        case,
        pse.SimulationSettings(
            method="diffsol",
            start=0,
            end=1,
            samples=SAMPLES,
            atol=[1e-8],
            rtol=1e-8,
            parameter_scales=[1],
            sensitivity=sensitivity,
        ),
    )


@pytest.mark.integration
@pytest.mark.parametrize("compact", [False, True])
def test_live_samples_keep_attempt_identity_and_joined_value_retention(
    inspection_settings: pse.EngineSettings,
    canonical_substrate: CanonicalFixture,
    compact: bool,
) -> None:
    runtime = canonical_substrate.runtime(inspection_settings)
    prepared = prepare(runtime)
    handle, cursor = prepared.start_live(capacity=1, compact=compact)
    tables: list[pa.Table] = []
    kinds: list[str] = []
    acknowledged: list[int] = []

    async def consume() -> pse.RunResult:
        joined = None
        async for event in cursor:
            kinds.append(event.kind)
            if event.kind == "sample":
                assert event.accuracy_attempt == 0
                assert event.sample_ordinal == len(tables)
                assert event.canonical_attempt_key == handle.canonical_attempt_key
                assert event.canonical_attempt_key is not None
                assert event.result() is None
                stream = event.table()
                assert stream is not None
                table = pa.table(stream)
                assert set(table.column("accuracy_attempt").to_pylist()) == {0}
                assert set(table.column("sample").to_pylist()) == {event.sample_ordinal}
                assert set(table.column("time").to_pylist()) == {SAMPLES[len(tables)]}
                assert table.column("mode_identity").null_count == 0
                tables.append(table)
            elif event.kind == "attempt_finished":
                assert len(tables) == len(SAMPLES)
                assert event.accuracy_attempt == 0
                assert event.produced_samples == len(SAMPLES)
                assert event.completed_time == 1
                assert event.termination == "completed"
                assert event.checks_complete
                assert event.checks_satisfied
                assert event.result() is None
            elif event.kind == "selected":
                assert event.accuracy_attempt == 0
            elif event.kind == "recorded":
                assert event.recorded_rows is not None
                acknowledged.append(event.recorded_rows)
            elif event.kind == "joined":
                joined = event.result()
                assert joined is not None
            else:
                assert event.kind == "recording_closed", event.diagnostic()
        assert joined is not None
        assert (await handle.wait_async()).run_id == joined.run_id
        return joined

    async def bounded_consume() -> pse.RunResult:
        return await asyncio.wait_for(consume(), timeout=30)

    try:
        result = asyncio.run(bounded_consume())
    finally:
        cursor.close()
    assert result.usable
    assert result.run_id == handle.wait().run_id
    assert result.canonical_attempt_key == handle.canonical_attempt_key
    assert kinds.count("attempt_finished") == kinds.count("selected") == 1
    assert kinds.count("recording_closed") == kinds.count("joined") == 1
    assert (
        kinds.index("attempt_finished")
        < kinds.index("selected")
        < kinds.index("joined")
    )
    live = pa.concat_tables(tables)
    assert live.num_rows == len(SAMPLES)
    assert acknowledged == sorted(acknowledged)
    assert acknowledged[-1] == live.num_rows
    for row in live.to_pylist():
        assert row["value"] == pytest.approx(math.exp(-0.5 * row["time"]), abs=1e-6)
    terminal = pa.table(result.table("runtime.simulation_samples"))
    if compact:
        assert terminal.num_rows == 0
    else:
        columns = ["sample", "time", "symbol_id", "quantity_id", "unit_id", "value"]
        assert terminal.select(columns).to_pylist() == live.select(columns).to_pylist()
    computation = result.completion.computation
    assert computation is not None
    assert computation.completed_samples == len(SAMPLES)
    assert computation.completed_time == 1
    modes = pa.table(result.table("runtime.modeling_trajectory_modes"))
    assert modes.column("time").to_pylist() == SAMPLES
    checks = pa.table(result.table("runtime.modeling_checks")).to_pylist()
    assert checks
    assert all(row["satisfied"] for row in checks)


@pytest.mark.component
def test_compact_explicit_sensitivities_refuse_before_a_live_run_is_returned(
    inspection_settings: pse.EngineSettings,
) -> None:
    runtime = pse.Runtime(inspection_settings, ephemeral=True)
    try:
        prepared = prepare(runtime, sensitivity="forward")
        before = runtime.resource_usage()
        with pytest.raises(pse.InspectionError) as refused:
            prepared.start_live(capacity=1, compact=True)
        assert refused.value.report.boundary_class == "unsupported"
        after = runtime.resource_usage()
        assert after.pool_peak_bytes == before.pool_peak_bytes
        assert after.pool_reserved_now == before.pool_reserved_now
        # The unchanged request still delivers its explicit derivative product in Full.
        handle, cursor = prepared.start_live(capacity=1)
        try:
            events = list(cursor)
        finally:
            cursor.close()
        assert any(event.kind == "joined" for event in events)
        result = handle.wait()
        assert result.usable
        derivatives = pa.table(result.table("runtime.response_sensitivities"))
        assert derivatives.num_rows == len(SAMPLES)
        for row in derivatives.to_pylist():
            assert row["parameter_column"] == 0
            assert row["value"] == pytest.approx(
                -row["time"] * math.exp(-0.5 * row["time"]), abs=1e-6
            )
    finally:
        runtime.close()


@pytest.mark.component
def test_closing_live_cursor_detaches_without_cancelling_and_preserves_escaped_table(
    inspection_settings: pse.EngineSettings,
) -> None:
    runtime = pse.Runtime(inspection_settings, ephemeral=True)
    prepared = prepare(runtime)
    handle, cursor = prepared.start_live(capacity=1)

    async def detach() -> tuple[pse.RunResult, pse.TableStream]:
        first = await anext(cursor)
        assert first.kind == "sample"
        escaped = first.table()
        assert escaped is not None
        with cursor:
            pass
        cursor.close()
        with pytest.raises(StopAsyncIteration):
            await anext(cursor)
        result = await asyncio.wait_for(handle.wait_async(), timeout=30)
        return result, escaped

    async def bounded_detach() -> tuple[pse.RunResult, pse.TableStream]:
        return await asyncio.wait_for(detach(), timeout=30)

    try:
        result, escaped = asyncio.run(bounded_detach())
        assert result.usable
        computation = result.completion.computation
        assert computation is not None
        assert computation.completed_samples == len(SAMPLES)
        assert pa.table(result.table("runtime.simulation_samples")).num_rows == len(
            SAMPLES
        )
    finally:
        cursor.close()
        runtime.close()
    # The stream was never read while its producer/runtime aliases remained open.
    rows = pa.table(escaped).to_pylist()
    assert len(rows) == 1
    assert rows[0]["sample"] == 0
    assert rows[0]["value"] == pytest.approx(1)
