# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""One descriptive observation per fresh process; no timing acceptance threshold."""

import contextlib
import os
import resource
import subprocess
import sys
import time
from pathlib import Path

import msgspec
import pyarrow as pa
import pytest

import pse
from pse import codec, diagnostics
from pse.contracts import documents as d
from pse.contracts.enums import (
    DynamicSensitivity,
    DynamicsMethod,
    NativeBackend,
    NativeSolveIntent,
    PresolvePolicyKind,
)
from pse.tests.physical_input_fixture import coordinate_batch, member
from pse.tests.test_diagnostics import diagnostic_native_solve
from pse.tests.test_local_runtime import physical_documents
from pse.tests.test_native_workflow import package_documents


class ObserverFitMeasurement(msgspec.Struct, frozen=True, kw_only=True):
    case: str
    mode: str
    usable: bool
    values: tuple[float, ...]
    work: tuple[d.WorkObservationOutput | None, ...]
    kinsol_counters: tuple[d.NativeCountersOutputKinsol | None, ...]
    idas_hub_counters: tuple[tuple[str, int], ...]
    idas_segment_counters: tuple[tuple[tuple[str, int], ...], ...]
    curtailed_operations: int
    refused_item_updates: int
    dropped_detail_updates: int
    terminal: str | None
    diagnostics_enabled: bool
    operation_count: int
    timings_ns: dict[str, int]
    serialized_bytes: dict[str, int]
    process_peak_rss_bytes: int
    rss_scope: str
    detail: d.DetailDocumentOutput | None
    detail_pages: int
    detail_paging_truncated: bool
    limits: tuple[str, ...]


_DYNAMICS = """package observer {
 def D {
  domain t:Time from 0{s} to 1{s};
  discretize grid on t using integrated(elements=1,order=1);
  param p:Scalar=2;
  var x[i in t]:Time;
  eq rate[i in t]:d(x[i])/di == p;
  eq initial:x[0{s}]==1{s};
  annotation report x("constant-rate");
  annotation check x(x[i]>=1{s});
 }
 case run fixture {
  dof 0; route integrated; procedure integrate;
  integrate samples(0{s},0.5{s},1{s}) relative(0.001)
   normalized_absolute(0.001) step(0.001{s});
 } { child root:D=D(); }
}"""


def _start(runtime: pse.Runtime, case: str) -> pse.RunHandle:
    if case == "kinsol":
        return diagnostic_native_solve(runtime)
    source = (
        _DYNAMICS
        if case == "idas"
        else """package observer { def Nlp {
          var x:Scalar; let f:Scalar=(x-1)*(x-1);
          annotation bounds x(0,3); annotation start x(0.5);
          annotation objective f(minimize);
        } }"""
    )
    package = runtime.modeling_from_documents(
        [package_documents(source)],
        runtime.physical_from_documents(physical_documents()),
    )
    name = "run" if case == "idas" else "Nlp"
    selected = next(
        row.declaration_id for row in package.declarations() if row.name == name
    )
    if case == "idas":
        return package.prepare_simulation(
            selected,
            pse.SimulationSettings(
                runtime=runtime,
                method=DynamicsMethod.IDAS,
                start=0,
                end=1,
                samples=[0, 0.5, 1],
                rtol=0.001,
                atol=coordinate_batch(
                    runtime, [(member(package, selected, "x"), 0.001, "Time")]
                ),
                parameter_scales=coordinate_batch(
                    runtime, [(member(package, selected, "p"), 1.0, "Scalar")]
                ),
                sensitivity=DynamicSensitivity.NONE,
            ),
        ).start()
    return package.prepare_solve(
        selected,
        pse.SolveSettings(
            backend=NativeBackend.POUNCE,
            intent=NativeSolveIntent.OPTIMIZE,
            presolve=PresolvePolicyKind.OFF,
        ),
    ).start()


def observer_fit_probe(
    case: str,
    mode: str,
    directory: Path,
    memory_bytes: int,
) -> ObserverFitMeasurement:
    """Importable child entry point; environment selects diagnostics before import."""
    started = time.perf_counter_ns()
    process_cpu = time.process_time_ns()
    spill = directory / "spill"
    spill.mkdir()
    runtime = pse.Runtime(
        pse.EngineSettings(
            memory_limit_bytes=memory_bytes,
            threads=1,
            target_partitions=1,
            spill_dir=str(spill),
            max_spill_bytes=1 << 30,
            batch_size=1024,
            cache=pse.CacheSettings(
                working_bytes=16 << 30,
                concurrent_loads=1,
                inflight_bytes=64 << 20,
                inspection_bytes=4 << 20,
            ),
        ),
        ephemeral=True,
    )
    timings: dict[str, int] = {}
    sizes: dict[str, int] = {}
    limits = [
        (
            "One observation per case/mode; descriptive times, "
            "no statistical or timing gate."
        ),
        (
            "Process peak RSS includes imports/runtime/science/copy/export; "
            "not observer private peak."
        ),
        "Analysis reservation and native fill are not exposed by these Python calls.",
        (
            "Cold process and fresh spill/diagnostic directories; "
            "shared compiler/OS caches unchanged."
        ),
    ]
    try:
        observations = runtime.observations()
        families = ("root_evaluation.v1", "native_owner_return.v1")
        if case == "pounce":
            families += (
                "native_factor.v2",
                "native_refinement.v2",
                "native_linear_call.v1",
            )
        if case == "idas":
            families += ("dynamic_local_error.v1",)
        collecting = (
            observations.collect(
                d.DetailRequest(
                    families=families,
                    requirement="complete",
                    sample_every=1,
                    boundary_thread_cpu=True,
                    duration=d.ClosedDuration(secs=30, nanos=0),
                    output=str(directory / "detail"),
                    budget=d.CaptureBudget(
                        memory_bytes=16 << 20, output_bytes=16 << 20
                    ),
                )
            )
            if mode == "selected_detail"
            else contextlib.nullcontext(None)
        )
        science_wall = time.perf_counter_ns()
        science_cpu = time.process_time_ns()
        with collecting as selected:
            handle = _start(runtime, case)
            if selected is not None:
                selected.leave()
            result = handle.wait()
        timings["prepare_start_join_wall"] = time.perf_counter_ns() - science_wall
        timings["prepare_start_join_process_cpu"] = time.process_time_ns() - science_cpu
        assert result.usable
        summary = result.attempt_summary
        before = codec.encode_json(summary)
        works = tuple(step.work for step in summary.steps)
        counters = tuple(
            step.native_observations.counters
            if step.native_observations is not None
            and isinstance(
                step.native_observations.counters, d.NativeCountersOutputKinsol
            )
            else None
            for step in summary.steps
        )
        relation = (
            "runtime.simulation_samples"
            if case == "idas"
            else "runtime.solve_variables"
        )
        table = pa.table(result.table(relation))
        values = tuple(float(value) for value in table.column("value").to_pylist())
        if case == "idas":
            times = tuple(float(value) for value in table.column("time").to_pylist())
            assert len(times) == 3
            assert values == pytest.approx(
                [1 + 2 * value for value in times], rel=0.001, abs=0.001
            )
            computation = result.completion.computation
            assert computation is not None
            assert computation.backend == "idas"
            terminal = computation.native_status
            limits.append(
                "IDAS segment getter populations retain their original "
                "per-segment resets; they are not summed across reinitializations."
            )
        else:
            assert values == pytest.approx(
                [3.0 if case == "kinsol" else 1.0], rel=0.001, abs=0.001
            )
            terminal = str(summary.steps[0].observation)
            work = works[0]
            assert work is not None
            assert work.attempts > 0
            assert summary.steps[0].backend == (
                NativeBackend.KINSOL if case == "kinsol" else NativeBackend.POUNCE
            )
            if case == "kinsol":
                native_counters = counters[0]
                assert native_counters is not None
                assert native_counters.function_evaluations is not None
            else:
                limits.append(
                    "POUNCE uses retained executing-owner work; "
                    "no POUNCE-specific public NativeCounters variant exists."
                )
        page = observations.list(limit=1)
        readers: list[tuple[pse.OperationObservations, d.OperationSnapshotOutput]] = []
        original_reader = handle.observations() if page.coverage.enabled else None
        pending = [] if original_reader is None else [original_reader.id]
        seen: set[d.OperationId] = set()
        while pending and len(readers) < 64:
            operation = pending.pop()
            if operation in seen:
                continue
            seen.add(operation)
            reader = observations.reader(operation)
            if reader is None:
                limits.append(
                    "A declared related operation is unavailable from the retained hub."
                )
                continue
            snapshot = reader.snapshot()
            readers.append((reader, snapshot))
            pending.extend(codec.convert_document(
                codec.document_builtins(child.id),
                d.OperationId,
            ) for child in snapshot.children)
        if pending:
            limits.append(
                "Selected related-operation traversal stopped at its 64-member bound."
            )
        copied_at = time.perf_counter_ns()
        for _ in range(4):
            for reader, snapshot in readers:
                copied = reader.inspect().operation
                assert copied is not None
                assert copied.metrics == snapshot.metrics
                assert copied.observed_at == snapshot.observed_at
        timings["four_passive_inspection_rounds"] = time.perf_counter_ns() - copied_at
        snapshots = tuple(snapshot for _, snapshot in readers)
        idas_counts = tuple(
            sorted(
                (metric.name, metric.cumulative.value)
                for snapshot in snapshots
                for metric in snapshot.metrics
                if metric.name.startswith("idas.work.")
                and isinstance(metric.cumulative, d.ValueCount)
            )
        )
        sizes["selected_operation_snapshots"] = len(codec.encode_json(snapshots))
        copied_at = time.perf_counter_ns()
        context = result.review_context()
        sizes["context"] = len(codec.encode_json(context))
        timings["context_and_encoding"] = time.perf_counter_ns() - copied_at
        segment_counts: tuple[tuple[tuple[str, int], ...], ...] = ()
        if case == "idas":
            fact = next(
                fact
                for fact in context.facts
                if fact.codec == "dynamic-native-statistics.v1"
            )
            fields = codec.decode_json(bytes(fact.payload), dict[str, msgspec.Raw])
            assert (
                codec.decode_json(bytes(fields["run_id"]), str)
                == result.run_id.to_hex()
            )
            segments = codec.decode_json(
                bytes(fields["segments"]), tuple[dict[str, msgspec.Raw], ...]
            )
            count_names = (
                "steps",
                "residual_evaluations",
                "nonlinear_iterations",
                "nonlinear_convergence_failures",
                "error_test_failures",
                "initialization_backtracks",
                "root_evaluations",
                "linear_iterations",
                "preconditioner_evaluations",
                "jacobian_products",
                "linear_setups",
                "jacobian_evaluations",
                "preconditioner_solves",
                "linear_convergence_failures",
                "linear_residual_evaluations",
            )
            segment_counts = tuple(
                tuple(
                    (name, codec.decode_json(bytes(segment[name]), int))
                    for name in count_names
                    if name in segment
                )
                for segment in segments
            )
            assert segment_counts
            assert all(segment_counts)
            if not idas_counts:
                assert (
                    any(snapshot.coverage.curtailed for snapshot in snapshots)
                    or not page.coverage.enabled
                )
                limits.append(
                    "Compact hub native getters are unavailable after its metric cap; "
                    "retained original segment statistics supply the independent "
                    "counter comparison."
                )
        detail = None if selected is None else selected.snapshot()
        detail_pages: list[d.DetailRecordPageOutput] = []
        truncated = False
        if detail is not None:
            offset = 0
            for _ in range(32):
                detail_page = diagnostics.detail_records(
                    detail.records, offset=offset, limit=128
                )
                detail_pages.append(detail_page)
                if detail_page.next_offset is None:
                    break
                offset = detail_page.next_offset
            else:
                truncated = True
            sizes["detail_acknowledged_raw"] = detail.readable_prefix_bytes
            sizes["detail_pages_json"] = len(codec.encode_json(tuple(detail_pages)))
        invocation = result.invocation_evidence
        if invocation.operation is None:
            limits.append(
                "No selected operation identity with diagnostics off; "
                "analysis/export not requested, absent metrics are not zero work."
            )
        else:
            evidence = d.FrozenEvidenceSetOutput(
                checkpoints=(),
                operations=tuple(codec.convert_document(
                    codec.document_builtins(snapshot),
                    d.OperationSnapshotBodyOutput,
                ) for snapshot in snapshots),
                contexts=(context,),
                invocations=(invocation,),
                detail_pages=tuple(detail_pages),
                details=() if detail is None else (detail,),
            )
            sizes["frozen_evidence_json"] = len(codec.encode_json(evidence))
            copied_at = time.perf_counter_ns()
            analysis = diagnostics.analyze_evidence(
                evidence, d.AnalysisRequest(operation=invocation.operation)
            )
            sizes["analysis_json"] = len(codec.encode_json(analysis))
            timings["analysis_and_encoding"] = time.perf_counter_ns() - copied_at
            copied_at = time.perf_counter_ns()
            index = diagnostics.export_review_pack(
                directory / "pack",
                evidence,
                analysis,
                d.PackExportRequest(
                    selection=d.PackSelection(
                        members=(),
                        maximum_output_bytes=256 << 20,
                        maximum_document_bytes=64 << 20,
                    )
                ),
            )
            timings["export"] = time.perf_counter_ns() - copied_at
            sizes["pack_payload"] = sum(
                member.readable_bytes for member in index.members
            )
        assert codec.encode_json(result.attempt_summary) == before
        assert result.usable
        timings["child_before_runtime_close_wall"] = time.perf_counter_ns() - started
        timings["child_before_runtime_close_process_cpu"] = (
            time.process_time_ns() - process_cpu
        )
        return ObserverFitMeasurement(
            case=case,
            mode=mode,
            usable=result.usable,
            values=values,
            work=works,
            kinsol_counters=counters,
            idas_hub_counters=idas_counts,
            idas_segment_counters=segment_counts,
            curtailed_operations=sum(
                snapshot.coverage.curtailed for snapshot in snapshots
            ),
            refused_item_updates=sum(
                snapshot.coverage.refused_items for snapshot in snapshots
            ),
            dropped_detail_updates=sum(
                snapshot.coverage.dropped_detail for snapshot in snapshots
            ),
            terminal=terminal,
            diagnostics_enabled=page.coverage.enabled,
            operation_count=len(snapshots),
            timings_ns=timings,
            serialized_bytes=sizes,
            process_peak_rss_bytes=(
                resource.getrusage(resource.RUSAGE_SELF).ru_maxrss * 1024
            ),
            rss_scope="Linux ru_maxrss: lifetime peak RSS of this fresh child process",
            detail=detail,
            detail_pages=len(detail_pages),
            detail_paging_truncated=truncated,
            limits=tuple(limits),
        )
    finally:
        runtime.close()


@pytest.mark.performance
@pytest.mark.parametrize("case", ["kinsol", "pounce", "idas"])
def test_plan47f_fresh_process_observer_fit(
    case: str,
    inspection_settings: pse.EngineSettings,
    tmp_path: Path,
) -> None:
    program = (
        "import sys\nfrom pathlib import Path\nfrom pse import codec\n"
        "from pse.tests.test_review_observer_fit import observer_fit_probe\n"
        "print(codec.encode_json(observer_fit_probe(sys.argv[1], sys.argv[2], "
        "Path(sys.argv[3]), int(sys.argv[4]))).decode())\n"
    )
    measurements: list[ObserverFitMeasurement] = []
    for mode in ("off", "ordinary", "selected_detail"):
        directory = tmp_path / mode
        directory.mkdir()
        environment = dict(
            os.environ,
            PSE_DIAGNOSTICS="off" if mode == "off" else "on",
            PSE_DIAGNOSTICS_DIR=str(directory / "diagnostics"),
        )
        process = subprocess.run(
            [
                sys.executable,
                "-c",
                program,
                case,
                mode,
                str(directory),
                str(inspection_settings.memory_limit_bytes),
            ],
            env=environment,
            capture_output=True,
            text=True,
            check=False,
            timeout=120,
        )
        assert process.returncode == 0, process.stderr
        measurement = codec.decode_json(
            process.stdout.splitlines()[-1], ObserverFitMeasurement
        )
        measurements.append(measurement)
        print(  # noqa: T201 -- root captures descriptive qualification metrics
            "plan47f_measurement=" + codec.encode_json(measurement).decode()
        )
    baseline, ordinary, detail = measurements
    for measured in measurements:
        assert measured.usable
        assert measured.values == pytest.approx(baseline.values, rel=0.001, abs=0.001)
        assert measured.terminal == baseline.terminal
        assert measured.work == baseline.work
        assert measured.kinsol_counters == baseline.kinsol_counters
        assert measured.idas_segment_counters == baseline.idas_segment_counters
    assert not baseline.diagnostics_enabled
    assert baseline.operation_count == 0
    assert ordinary.diagnostics_enabled
    assert ordinary.operation_count > 0
    assert detail.diagnostics_enabled
    assert detail.operation_count > 0
    assert ordinary.idas_hub_counters == detail.idas_hub_counters
    if case == "idas":
        assert ordinary.idas_segment_counters
    assert detail.detail is not None
    assert not detail.detail_paging_truncated
    assert detail.detail.readable_prefix_bytes > 0
