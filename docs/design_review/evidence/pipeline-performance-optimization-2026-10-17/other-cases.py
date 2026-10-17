# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Compact production Python API probes; one fresh ephemeral runtime per process.

Run via scripts/pse-env --native=solver,klu -- python THIS_FILE CASE OUTPUT.
Study width is selected with --workers 1/2, retaining two engine threads.
Opaque synchronous strategy/study APIs cancel and join on Python interruption;
they expose no public handle for a timer to cancel. No hard deadline is claimed.
"""
# ruff: noqa: INP001 -- standalone evidence driver, not an importable package
# ruff: noqa: S101 -- preserve the original scientific fixture assertions
# ruff: noqa: T201 -- phase progress is the driver's console interface

from __future__ import annotations

import argparse
import contextlib
import math
import threading
import time
import traceback
from pathlib import Path
from typing import TYPE_CHECKING

import pyarrow as pa

import pse
from pse import codec
from pse.contracts import runtime as result_contracts
from pse.contracts.enums import (
    NativeBackend,
    NativeSolveIntent,
    PresolvePolicyKind,
    TearMethod,
)
from pse.tests.study_fixtures import assignment, physical_ids, point
from pse.tests.study_fixtures import request as study_request
from pse.tests.test_local_runtime import physical_documents
from pse.tests.test_native_workflow import identity, package_documents

if TYPE_CHECKING:
    from collections.abc import Iterator

    from pse.contracts.identities import DeclarationId


def main() -> None:
    """Run one selected compact case and retain its actual observations."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "case", choices=(
            "conic", "recycle", "discrete", "study-independent",
            "study-continuation", "result-reads",
        )
    )
    parser.add_argument("output", type=Path)
    parser.add_argument("--workers", type=int, choices=(1, 2), default=2)
    parser.add_argument("--memory-gib", type=int, choices=(16, 64), default=16)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    spill = args.output / "spill"
    spill.mkdir()
    settings = pse.EngineSettings(
        memory_limit_bytes=args.memory_gib << 30, threads=2, target_partitions=1,
        spill_dir=str(spill.resolve()), max_spill_bytes=1 << 30, batch_size=1024,
        math_jobs=args.workers, cache=pse.CacheSettings(working_bytes=4 << 30),
    )
    started = time.monotonic()
    runtime = pse.Runtime(settings, ephemeral=True)
    observations = runtime.observations()
    phases = []
    handles = []
    done = threading.Event()
    status = "failed"
    scientific_assertions = "not_completed"

    def write(name: str, value: object) -> None:
        encoded = codec.encode_json(codec.converter().unstructure(value))
        (args.output / name).write_bytes(encoded)

    def snapshot(name: str) -> None:
        records = []
        offset = 0
        while True:
            page = observations.list(offset=offset, limit=32)
            for operation in page.operations:
                reader = observations.reader(operation.id)
                records.append({
                    "operation": operation,
                    "inspection": (
                        None if reader is None else reader.inspect(window=120)
                    ),
                })
            if len(page.operations) < 32:
                break
            offset += len(page.operations)
        write(name, records)

    def watch() -> None:
        sequence = 0
        while not done.wait(5):
            try:
                snapshot(f"live-{sequence:02d}.json")
            except Exception:
                (args.output / "observer-error.txt").write_text(traceback.format_exc())
            sequence += 1

    watcher = threading.Thread(target=watch)
    watcher.start()

    @contextlib.contextmanager
    def phase(name: str) -> Iterator[None]:
        begin = time.monotonic()
        print(f"{args.case}: {name}", flush=True)
        try:
            yield
        finally:
            phases.append({"name": name, "seconds": time.monotonic() - begin})
            write("phases.json", phases)

    def package(source: str) -> tuple[pse.ModelingPackage, DeclarationId]:
        with phase("source_admission"):
            documents = package_documents(source)
            if args.case == "discrete":
                documents["package.toml"] = documents["package.toml"].replace(
                    "5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a",
                    "b27409be5572b8712e47db271fae28cd",
                )
            result = runtime.modeling_from_documents([documents], physical)
            case = next(
                row.declaration_id
                for row in result.declarations() if row.name == "Root"
            )
            return result, case

    def assess_root(table: pa.Table, expected: float) -> None:
        values = [row for row in table.to_pylist() if not row["parameter"]]
        assert len(values) == 1
        value, tolerance = values[0]["value"], values[0]["tolerance"]
        assert math.isfinite(value)
        assert math.isfinite(tolerance)
        assert tolerance > 0
        assert abs(value - expected) <= tolerance, (value, expected, tolerance)

    def record_run(result: pse.RunResult, name: str) -> None:
        write(name + ".json", {
            "usable": result.usable, "completion": result.completion,
            "invocation": result.invocation_evidence,
            "summary": result.attempt_summary,
        })

    def record_strategy(
        result: pse.StrategyResult, solve: pse.SolveSettings
    ) -> pse.NativeAttempt:
        assert not result.failures(), result.failures()
        events = codec.structure_rows(
            pa.table(result.strategy_events()).to_pylist(),
            result_contracts.RuntimeSolveStrategyEventsRow,
        )
        assert events
        assert result.run_id is not None
        assert {event.run_id for event in events} == {result.run_id}
        assert codec.structure_rows(
            pa.table(result.strategy_events()).to_pylist(),
            result_contracts.RuntimeSolveStrategyEventsRow,
        ) == events
        write("strategy-events.json", events)
        (row,) = result.attempts()
        assert row.failure is None
        assert row.report is not None
        attempt = row.report
        write("attempt.json", {"qualification": attempt.qualification,
            "termination": attempt.termination, "backend": attempt.backend,
            "primal": attempt.primal(), "objective": attempt.objective,
            "normalized_violation": attempt.normalized_violation,
            "metrics": attempt.metrics(), "options": attempt.options(),
            "provenance": attempt.provenance(), "numerics": solve.numerics,
            "invocation": None,
            "invocation_unavailable": (
                "PreparedStrategy returns StrategyResult rather than RunResult"
            ),
        })
        return attempt

    try:
        write("conditions.json", {"case": args.case, "threads": settings.threads,
            "math_jobs": settings.math_jobs,
            "memory_limit_bytes": settings.memory_limit_bytes,
            "cache_working_bytes": settings.cache.working_bytes, "ephemeral": True,
            "cancellation": (
                "RunHandle.cancel followed by wait; opaque synchronous "
                "strategy/study uses native Python-interrupt cancellation and "
                "join, no timer deadline"
            ),
        })
        with phase("physical_admission"):
            documents = physical_documents()
            if args.case == "discrete":
                root = (
                    Path(__file__).resolve().parents[4] / "packages/reference/physical"
                )
                documents = {
                    str(path.relative_to(root)): path.read_text()
                    for path in root.rglob("*")
                    if path.is_file()
                    and path.suffix in {".toml", ".yaml", ".yml", ".pse"}
                }
            physical = runtime.physical_from_documents(documents)
        if args.case == "conic":
            def port(n: int) -> dict[str, str]:
                return {
                    "symbol_id": identity(n).to_hex(),
                    "quantity_id": identity(31).to_hex(),
                    "unit_id": identity(10).to_hex(),
                }

            request = codec.decode_json(codec.encode_json({
                "variables": [port(201)], "rows": [port(202)],
                "objective_port": port(0),
                "quadratic": {
                    "rows": 1, "columns": 1, "column_starts": [0, 0],
                    "row_indices": [], "values": [],
                },
                "objective": [1.], "constraints": {
                    "rows": 1, "columns": 1, "column_starts": [0, 1],
                    "row_indices": [0], "values": [-1.],
                },
                "rhs": [-2.],
                "cones": [{"kind": "nonnegative", "dimension": 1}],
                "objective_constant": 3.,
            }), pse.ConicRequest)
            solve = pse.SolveSettings(backend=NativeBackend.CLARABEL)
            with phase("prepare_conic"):
                prepared = runtime.prepare_conic(request, physical, solve)
                assert [route.backend for route in prepared.routes] == ["clarabel"]
            with phase("conic_run"):
                result = prepared.run()
            attempt = record_strategy(result, solve)
            assert attempt.qualification == "optimal_within_tolerance"
            assert math.isclose(
                attempt.objective, 5., rel_tol=solve.numerics.gap_relative,
                abs_tol=solve.numerics.gap_absolute,
            )
            assert math.isclose(
                dict(attempt.primal())[identity(201).to_hex()], 2.,
                rel_tol=solve.numerics.engineering_relative_fraction,
            )
            assert attempt.normalized_violation is not None
            assert attempt.normalized_violation <= 1.
        elif args.case == "recycle":
            model, case = package(
                "package recycle {def Root {param a:Scalar=2; var x:Scalar; "
                "let output:Scalar=x/2+a; port inlet:Scalar=x; "
                "annotation connectivity inlet(1,0); port outlet:Scalar=output; "
                "annotation connectivity outlet(0,1); connect outlet -> inlet; "
                "annotation start x(1);}}"
            )
            solve = pse.SolveSettings(
                intent=NativeSolveIntent.ROOT, backend=NativeBackend.KINSOL,
                presolve=PresolvePolicyKind.OFF,
            )
            with phase("inspect_and_prepare_flow"):
                inspected = model.inspect(case, solve)
                inlet = next(
                    p for p in inspected.ports if p.lineage.path.endswith(".inlet")
                )
                outlet = next(
                    p for p in inspected.ports if p.lineage.path.endswith(".outlet")
                )
                selection = codec.decode_json(codec.encode_json({
                    "nodes": [case.to_hex()], "connections": [
                        {"connection": c.id, "group": c.id, "cost": 2.,
                         "policy": "mandatory"}
                        for c in inspected.connections
                    ],
                }), pse.FlowSelectionDocument)
                flow = model.prepare_flow(case, selection, solve)
            with phase("tear_selection"):
                selected = flow.select_tears(
                    TearMethod.UNWEIGHTED_HEURISTIC, solve
                ).tears()
                assert selected is not None
                assert selected.cost == 2.
            request = codec.decode_json(codec.encode_json({"tears": selected.decisions,
                "units": [{
                    "node": case.to_hex(), "realization": {"kind": "explicit_map"},
                    "inputs": [inlet.id], "outputs": [outlet.id],
                }],
                "anderson": 1, "damping": 1.}), pse.RecycleRequest)
            with phase("prepare_recycle"):
                prepared = model.prepare_recycle(case, selection, request, solve)
                assert [route.backend for route in prepared.routes] == ["kinsol"]
            with phase("recycle_run"):
                result = prepared.run()
            attempt = record_strategy(result, solve)
            assert attempt.qualification == "feasible"
            assert math.isclose(
                dict(attempt.primal())[inlet.id], 4.,
                rel_tol=solve.numerics.engineering_relative_fraction,
            )
        elif args.case.startswith("study-"):
            model, case = package(
                "package algebraic {def Root {param a:Scalar=4; var x:Scalar; "
                "eq square:x*x==a; annotation start x(1); "
                "annotation bounds x(0,10); annotation report x(\"root\"); "
                "annotation check x(x>1);}}"
            )
            solve = pse.SolveSettings(
                backend=NativeBackend.IPOPT, intent=NativeSolveIntent.FEASIBLE_POINT,
                presolve=PresolvePolicyKind.OFF,
            )
            root = Path(__file__).resolve().parents[4]
            quantity, unit = physical_ids(
                root / "tests/fixtures/packages/physical-primitives"
                / "materials/physical.yaml",
                "Scalar", "dimensionless",
            )
            continuation = args.case == "study-continuation"
            points = tuple(
                point(
                    case, solve, i,
                    assignments=(
                        assignment("a", float((i + 2) ** 2), quantity, unit),
                    ),
                    predecessor=i - 1 if continuation and i else None,
                    continuation=continuation and i > 0,
                )
                for i in range(4)
            )
            with phase("study_admission"):
                definition = model.admit_study(study_request(*points))
            write("resources-before-study.json", runtime.resource_usage())
            with phase("study_execute"):
                study = model.study(
                    definition, controls=pse.StudyRunControls(maximum_points=4)
                )
            write("resources-after-study.json", runtime.resource_usage())
            write("study.json", {
                "count": study.count, "unattempted": study.unattempted,
                "conclusion": study.conclusion, "preparations": study.preparations,
                "outcomes": [study.outcome(i) for i in range(study.count)],
                "history": pa.table(study.table()).to_pylist(),
            })
            assert study.count == 4
            assert study.unattempted == 0
            assert study.conclusion.availability == "complete"
            assert study.conclusion.lifecycle == "terminal"
            for i in range(4):
                outcome = study.outcome(i)
                result = study.result(i)
                assert isinstance(result, pse.RunResult), study.failure(i)
                record_run(result, f"point-{i}")
                assert outcome.key == i
                assert outcome.lifecycle == "completed"
                assert outcome.scientific.usable
                assert result.usable
                assert len(outcome.attempts) == 1
                assert outcome.diagnostic is None
                assert study.failure(i) is None
                with phase(f"point_{i}_result_projection"):
                    table = pa.table(result.table("runtime.solve_variables"))
                    write(f"point-{i}-variables.json", table.to_pylist())
                    assess_root(table, float(i + 2))
            scientific_assertions = "passed"
            counts = study.preparations
            reuse_expected = 1 <= counts.views <= args.workers
            write("experimental-expectations.json", {
                "views_within_workers": {
                    "satisfied": reuse_expected,
                    "expected_maximum": args.workers,
                    "observed_views": counts.views,
                    "standing": (
                        "borrowed reuse hypothesis, not an established product contract"
                    ),
                },
                "scientific_assertions": scientific_assertions,
            })
            if not reuse_expected:
                status = "experimental_expectation_violated"
            assert 1 <= counts.views <= args.workers, counts
            assert counts.views + counts.rebuilt + counts.shared == 4, counts
        else:
            discrete = args.case == "discrete"
            source = (
                "package discrete {def Root {var n:Count in integer; "
                "annotation bounds n(0{1},5{1}); annotation start n(0{1}); "
                "eq floor:n>=2{1}; let cost:Count=n; "
                "annotation objective cost(minimize);}}"
                if discrete else
                "package reads {def Root {param a:Scalar=4; var x:Scalar; "
                "eq square:x*x==a; annotation start x(1);}}"
            )
            model, case = package(source)
            solve = pse.SolveSettings(
                backend=NativeBackend.HIGHS if discrete else NativeBackend.KINSOL,
                intent=(
                    NativeSolveIntent.OPTIMIZE if discrete else NativeSolveIntent.ROOT
                ),
                presolve=PresolvePolicyKind.OFF,
            )
            with phase("prepare_solve"):
                prepared = model.prepare_solve(case, solve)
            with phase("solve"):
                handle = prepared.start()
                handles.append(handle)
                result = handle.wait()
            record_run(result, "result")
            assert result.usable, result.completion
            retained = None
            for i in range(3):
                with phase(f"result_projection_read_{i}"):
                    table = pa.table(result.table("runtime.solve_variables"))
                    write(f"variables-{i}.json", table.to_pylist())
                    assess_root(table, 2.)
                    if retained is not None:
                        assert table.equals(retained, check_metadata=True)
                    retained = table
            write(
                "solve-runs.json",
                pa.table(result.table("runtime.solve_runs")).to_pylist(),
            )
            write(
                "strategy-events.json",
                pa.table(result.table("runtime.solve_strategy_events")).to_pylist(),
            )
        scientific_assertions = "passed"
        status = "passed"
    except BaseException:
        (args.output / "error.txt").write_text(traceback.format_exc())
        raise
    finally:
        done.set()
        watcher.join()
        for handle in handles:
            if handle.result() is None:
                handle.cancel()
                handle.wait()
        try:
            snapshot("operations.json")
        finally:
            with phase("runtime_release"):
                runtime.close()
            write("outcome.json", {
                "case": args.case, "workers": args.workers,
                "memory_gib": args.memory_gib, "status": status,
                "scientific_assertions": scientific_assertions,
                "elapsed_seconds": time.monotonic() - started, "phases": phases,
            })


if __name__ == "__main__":
    main()
