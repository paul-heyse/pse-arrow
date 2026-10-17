# SPDX-License-Identifier: MIT OR Apache-2.0
"""Small production-API probes for the pipeline performance design review.

Run through scripts/pse-env --native=solver,klu. Each process owns one runtime;
the timer requests cancellation of actual handles and never kills native work.
"""

# This executable evidence fixture is not a package. Its console progress,
# conditional case imports and scientific assertions are deliberate probe controls.
# ruff: noqa: INP001, T201, PLC0415, S101

from __future__ import annotations

import argparse
import ast
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
from pse.contracts.documents import CaptureBudget, ClosedDuration, DetailRequest
from pse.contracts.enums import NativeBackend, NativeSolveIntent, PresolvePolicyKind
from pse.tests.test_local_runtime import physical_documents
from pse.tests.test_native_workflow import package_documents

if TYPE_CHECKING:
    from collections.abc import Iterator


def main() -> None:
    """Run one selected case and persist its observations in a fresh directory."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("case")
    parser.add_argument("output", type=Path)
    parser.add_argument("--detail", action="store_true")
    parser.add_argument("--backend", default="kinsol")
    parser.add_argument("--size", type=int, default=2)
    parser.add_argument("--parameters", type=int, default=2)
    parser.add_argument("--setup-interval", type=int)
    parser.add_argument("--rank-cells", type=int)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    (args.output / "spill").mkdir()
    started = time.monotonic()
    settings = pse.EngineSettings(
        memory_limit_bytes=16 << 30,
        threads=2,
        target_partitions=1,
        spill_dir=str(args.output / "spill"),
        max_spill_bytes=1 << 30,
        batch_size=1024,
        cache=pse.CacheSettings(working_bytes=4 << 30),
    )
    runtime = pse.Runtime(settings, ephemeral=True)
    observations = runtime.observations()
    handles: list[pse.RunHandle] = []
    phases: list[dict[str, object]] = []
    results: list[dict[str, object]] = []
    finished = threading.Event()
    expired = threading.Event()

    def write(name: str, value: object) -> None:
        (args.output / name).write_bytes(codec.encode_json(value))

    def snapshots(name: str) -> None:
        offset = 0
        records = []
        while offset < 512:
            page = observations.list(offset=offset, limit=32)
            for operation in page.operations:
                reader = observations.reader(operation.id)
                if reader is not None:
                    records.append(reader.inspect(window=120))
            if len(page.operations) < 32:
                break
            offset += len(page.operations)
        write(name, records)

    def watch() -> None:
        sequence = 0
        while not finished.wait(5):
            try:
                snapshots(f"live-{sequence:02d}.json")
                write(
                    f"progress-{sequence:02d}.json",
                    [h.current_progress() for h in handles],
                )
            except Exception as error:
                (args.output / "observer-error.txt").write_text(str(error))
            sequence += 1
            if time.monotonic() - started >= 60:
                expired.set()
                for handle in tuple(handles):
                    handle.cancel()

    watcher = threading.Thread(target=watch, daemon=True)
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

    def run(
        prepared: pse.PreparedOperation, name: str, *, cancel: bool = False
    ) -> pse.RunResult:
        with phase(name):
            handle = prepared.start()
            handles.append(handle)
            if cancel or expired.is_set():
                handle.cancel()
            result = handle.wait()
            values = (
                []
                if args.case in {"ode", "dae", "events", "fit"}
                else pa.table(result.table("runtime.solve_variables")).to_pylist()
            )
            results.append(
                {
                    "name": name,
                    "usable": result.usable,
                    "values": values,
                    "invocation": result.invocation_evidence,
                    "summary": result.attempt_summary,
                    "progress": handle.current_progress(),
                }
            )
            write("results.json", results)
            if args.case in {"ode", "dae", "events", "fit"}:
                tables = (
                    ("runtime.fit_parameters", "runtime.response_sensitivities")
                    if args.case == "fit"
                    else ("runtime.simulation_samples",)
                )
                for table in tables:
                    rows = pa.table(result.table(table)).to_pylist()
                    write(table + ".json", rows)
                    if table == "runtime.fit_parameters":
                        assert len(rows) == 1
                        assert abs(rows[0]["value"] - 3.) <= 0.001
                    if table == "runtime.response_sensitivities" and rows:
                        assert abs(rows[0]["value"] - 1.) <= 0.001
                    if table == "runtime.simulation_samples" and args.case == "ode":
                        end = [row["value"] for row in rows if row["time"] == 1.]
                        assert end
                        assert all(
                            abs(value - math.exp(-0.5)) <= 0.002 for value in end
                        )
            if not cancel and not expired.is_set():
                assert result.usable, result.completion
            return result

    status = "failed"
    collection = None
    try:
        def source_documents(path: Path) -> dict[str, bytes]:
            return {
                str(p.relative_to(path)): p.read_bytes()
                for p in path.rglob("*")
                if p.is_file()
                and p.suffix in {".toml", ".yaml", ".yml", ".pse", ".parquet"}
            }

        if args.detail:
            family = (
                "dynamic_native_return.v1"
                if args.case in {"ode", "dae", "events"}
                else "root_evaluation.v1"
                if args.backend == "kinsol"
                else "nonlinear_iteration.v1"
            )
            collection = observations.collect(
                DetailRequest(
                    families=(
                        (family, "native_owner_return.v1", "workflow.integration.v1")
                        if args.case == "fit"
                        else (family, "native_owner_return.v1")
                    ),
                    requirement="complete",
                    sample_every=1,
                    boundary_thread_cpu=True,
                    duration=ClosedDuration(secs=60, nanos=0),
                    output=str(args.output / "detail"),
                    budget=CaptureBudget(memory_bytes=16 << 20, output_bytes=16 << 20),
                )
            )
            collection.__enter__()
        with phase("physical_admission"):
            physical = runtime.physical_from_documents(
                source_documents(Path.cwd() / "packages/reference/physical")
                if args.case == "reference"
                else physical_documents()
            )
        if args.case == "reference":
            with phase("reference_fixture_read"):
                names = (
                    "data/oracles/teqp-0.23.1", "data/oracles/feos-0.10.1",
                    "data/gross-sadowski-2001", "data/references", "data/nist",
                    "data/perry7", "data/poling2000", "data/oracles/idaes-2.13",
                    "data/species", "data/ciaaw", "seed-data", "process",
                    "thermodynamics", "methods", "domain", "physical",
                )
                documents = [
                    source_documents(Path.cwd() / "packages/reference" / name)
                    for name in names
                ]
                write(
                    "fixture-size.json",
                    {
                        "packages": len(documents),
                        "files": sum(map(len, documents)),
                        "bytes": sum(
                            len(value) for d in documents for value in d.values()
                        ),
                    },
                )
            with phase("source_admission"):
                package = runtime.modeling_from_documents(documents, physical)
            write("declarations.json", {"count": len(package.declarations())})
            status = "completed"
            return
        n = args.size
        source = (
            "package short {def Root {param a:Scalar=4; var x:Scalar; "
            "eq e:x*x==a; annotation start x(1); annotation check x(x>1);}}"
        )
        optimize = args.case in {"qp", "nlp", "discrete"}
        if optimize:
            role = "discrete" if args.case == "discrete" else ""
            source = (
                "package short {def Root {var x:Scalar; var y:Scalar; "
                "annotation start x(0); annotation start y(0); "
                "annotation bounds x(0,10); annotation bounds y(0,10); "
                + ("annotation discrete x; " if role else "")
                + "let cost:Scalar=(x-2)*(x-2)+(y-3)*(y-3); "
                "annotation objective cost(minimize); "
                + ("eq curve:x*x+y>=4; " if args.case == "nlp" else "")
                + "annotation check x(x>1);}}"
            )
        elif args.case in {"sparse", "dense"}:
            declarations = "".join(
                f"var x{i}:Scalar; annotation start x{i}(1);" for i in range(n)
            )
            equations = "".join(
                f"eq e{i}:x{i}*x{i}"
                + (
                    "+" + "+".join(f"0.01*x{j}" for j in range(n) if j != i)
                    if args.case == "dense" and n > 1
                    else ""
                )
                + f"=={4 + (0.02*(n-1) if args.case == 'dense' else 0)};"
                for i in range(n)
            )
            source = f"package short {{def Root {{{declarations}{equations}}}}}"
        elif args.case == "mixed":
            source = (
                "package short {def Root {var x:Scalar; var y:Scalar; "
                "eq e:x==1e-6; eq f:y==1e6; annotation nominal x(1e-6); "
                "annotation nominal y(1e6); annotation start x(0); "
                "annotation start y(0);}}"
            )
        elif args.case == "guard":
            source = (
                "package short {def Root {var x:Scalar; eq e:log(x)==0; "
                "annotation start x(0.1);}}"
            )
        elif args.case == "implicit":
            variables = "".join(
                f"var y{i}:Scalar; eq e{i}:y{i}*y{i}==x; "
                f"annotation start y{i}(1); annotation bounds y{i}(0.5,3);"
                for i in range(n)
            )
            total = "+".join(f"root.y{i}" for i in range(n))
            source = (
                "package short {def Root {var x:Scalar; "
                f"implicit root select branch(y0>=0) {{{variables}}} "
                f"realize policy on root using nested; eq pin:({total})=={2*n}; "
                "annotation start x(2);}}"
            )
        elif args.case in {"ode", "dae", "events", "fit"}:
            extras = "".join(f"param q{i}:Scalar=0;" for i in range(args.parameters))
            extra_rate = "".join(f"+q{i}" for i in range(args.parameters))
            algebraic = (
                "var y[i in t]:Scalar; eq closure[i in t]:y[i]==2*x[i]/1{s}; "
                "annotation start y(2);"
                if args.case == "dae"
                else ""
            )
            rate = (
                "-p*y[i]"
                if args.case == "dae"
                else "-p*x[i]/1{s}"
                if args.case != "fit"
                else "p"
            )
            source = (
                "package short {def D {domain t:Time from 0{s} to 1{s}; "
                "discretize grid on t using integrated(elements=1,order=1);"
                f"param p:Scalar=0.5; {extras} var x[i in t]:Time; {algebraic}"
                f"eq rate[i in t]:d(x[i])/di=={rate}{extra_rate}; "
                "eq initial:x[0{s}]==1{s};"
                "let observed[i in t]:Time=x[i]; "
                "annotation report observed(\"measurement\"); "
                "annotation check x(x[i]>0{s});} "
                "case run fixture {dof 0;route integrated;procedure integrate; "
                "integrate samples(0{s},0.5{s},1{s}) relative(global) "
                "normalized_absolute(global) step(1e-4{s});}{child root:D=D();} "
                "entity kind origin provenance {attribute title:Text;} "
                "entity origin experiment {title=\"analytic linear accumulation\"} "
                "enum role {measured facets(measured)} entity kind reading "
                "{attribute value:Time?;attribute sigma:Time?;} "
                "@id(\"68686868686868686868686868686868\") "
                "entity reading measured provenance(experiment,role.measured) "
                "{value=4{s},sigma=1{s}} }"
            )
            if args.case == "events":
                tree = ast.parse(
                    (
                        Path.cwd() / "python/pse/tests/test_modeling_kernel.py"
                    ).read_text()
                )
                function = next(
                    f for f in tree.body
                    if isinstance(f, ast.FunctionDef)
                    and f.name == (
                        "test_modeling_simulation_events_checks_and_terminal_reports"
                    )
                )
                assignment = next(
                    f for f in ast.walk(function)
                    if isinstance(f, ast.Assign)
                    and any(
                        isinstance(t, ast.Name) and t.id == "source" for t in f.targets
                    )
                )
                source = (
                    ast.literal_eval(assignment.value)
                    .replace("relative(1e-8)", "relative(global)")
                    .replace("normalized_absolute(1e-8)", "normalized_absolute(global)")
                    .replace("quadrature_relative(1e-8)", "quadrature_relative(global)")
                    .replace(
                        "quadrature_absolute(root.area=1e-9{s})",
                        "quadrature_absolute(root.area=0.001{s})",
                    )
                    .replace("tolerance(1e-8{s})", "tolerance(0.001{s})")
                )
        with phase("source_admission"):
            (args.output / "source.pse").write_text(source)
            package = runtime.modeling_from_documents(
                [package_documents(source)], physical
            )
            case = next(
                d.declaration_id for d in package.declarations()
                if d.name == (
                    "run" if args.case in {"ode", "dae", "events", "fit"} else "Root"
                )
            )
        solve = pse.SolveSettings(
            backend=NativeBackend(
                args.backend if args.backend not in {"diffsol", "idas"} else "ipopt"
            ),
            intent=NativeSolveIntent.OPTIMIZE if optimize else NativeSolveIntent.ROOT,
            presolve=PresolvePolicyKind.OFF,
        )
        if args.setup_interval is not None:
            import msgspec

            from pse.contracts.documents import KinsolSettings

            solve = msgspec.structs.replace(
                solve, settings=KinsolSettings(setup_interval=args.setup_interval)
            )
        with phase("prepare_cold"):
            if args.case in {"ode", "dae", "events", "fit"}:
                from pse.contracts.documents import NumericalPolicy, Profile

                tolerance = NumericalPolicy().engineering_relative_fraction
                samples = (
                    [i / (n - 1) for i in range(n)]
                    if args.case != "fit" else [0., 0.5, 1.]
                )
                simulation = pse.SimulationSettings(
                    method=(
                        args.backend
                        if args.backend in {"diffsol", "idas"} else "diffsol"
                    ),
                    start=0,
                    end=2 if args.case == "events" else 1,
                    samples=[0., 0.25, 0.75, 2.] if args.case == "events" else samples,
                    atol=[tolerance] * (2 if args.case == "dae" else 1),
                    rtol=tolerance,
                    parameter_scales=(
                        [1.] * (1 if args.case == "events" else args.parameters + 1)
                    ),
                    sensitivity="forward" if args.case == "fit" else "none",
                    out_rtol=tolerance if args.case == "events" else None,
                    out_atol=[tolerance] if args.case == "events" else None,
                )
                if args.case == "fit":
                    from pse.contracts import authored
                    from pse.contracts.values import SemanticId

                    def identity(i: int) -> SemanticId:
                        return SemanticId(bytes([i]) * 16)

                    fit = codec.converter().structure(
                        {
                            "fit_id": identity(101),
                            "parameters": [{
                                "symbol_id": identity(102), "fixed": False,
                                "value": 1., "lower": 0., "upper": 10., "scale": 1.,
                            }],
                            "experiments": [{
                                "experiment_id": identity(103), "case_id": case,
                                "route": "integrated",
                                "bindings": [{
                                    "parameter_id": identity(102), "path": "root.p",
                                }],
                            }],
                            "observations": [{
                                "observation_id": identity(104),
                                "value_attribute": "value",
                                "standard_deviation_attribute": "sigma",
                                "experiment_id": identity(103),
                                "output_path": "root.observed[0{s}]", "time": 1.,
                                "time_basis": None, "time_unit_id": None,
                                "included": True, "importance": 1.,
                            }],
                        },
                        authored.AuthoredFitCasesRow,
                    )
                    package = package.with_fit_declarations((fit,))
                    from pse.contracts.enums import HessianMode

                    solver = pse.SolveSettings(
                        backend=NativeBackend.IPOPT,
                        intent=NativeSolveIntent.OPTIMIZE,
                        presolve=PresolvePolicyKind.OFF,
                        controls=pse.SolveControls(hessian=HessianMode.LIMITED_MEMORY),
                    )
                    kwargs = (
                        {} if args.rank_cells is None
                        else {"max_cells": args.rank_cells}
                    )
                    prepared = package.prepare_fit(
                        fit.fit_id,
                        pse.FitPreparationDocument(
                            solver=solver,
                            simulations={
                                fit.experiments[0].experiment_id.to_hex():
                                    codec.decode_json(simulation.to_json(), Profile),
                            },
                            **kwargs,
                        ),
                    )
                else:
                    prepared = package.prepare_simulation(case, simulation)
            else:
                prepared = package.prepare_solve(case, solve)
        if args.case == "cancel":
            run(prepared, "cancelled", cancel=True)
        else:
            first = run(prepared, "cold_solve")
            if args.case in {"root", "edits"}:
                run(prepared, "same_prepared_repeat")
                start = first.available_start()
                if start is not None:
                    run(prepared.with_start(start), "accepted_primal_start")
                with phase("prepare_equal"):
                    equal = package.prepare_solve(case, solve)
                run(equal, "equal_preparation_solve")
            if args.case == "edits":
                from pse.contracts.documents import (
                    BindingAssignment,
                    BindingQuantity,
                    BindingTargetPath,
                    PointOverlay,
                )

                with phase("value_overlay_solve"):
                    bound = package.solve_bound(
                        case,
                        PointOverlay(assignments=(
                            BindingAssignment(
                                target=BindingTargetPath(value="a"),
                                value=BindingQuantity(
                                    magnitude=9., quantity="1f" * 16, unit="0a" * 16
                                ),
                            ),
                        )),
                        solve,
                    )
                    attempt = bound.attempt()
                    assert attempt is not None
                    primal = attempt.primal()
                    write(
                        "overlay.json", {"accepted": bound.accepted, "primal": primal}
                    )
                    assert bound.accepted
                    assert len(primal) == 1
                    assert abs(primal[0][1] - 3.) <= 0.003
                for name, text in (
                    ("value", source.replace("a:Scalar=4", "a:Scalar=9")),
                    ("structure", source.replace("x*x==a", "x*x+x==a")),
                ):
                    with phase(name + "_reload"):
                        changed = package.reload_from_parent([package_documents(text)])
                    with phase(name + "_prepare"):
                        bound = changed.prepare_solve(case, solve)
                    run(bound, name + "_solve")
        status = "cancelled" if expired.is_set() else "completed"
    except Exception:
        (args.output / "error.txt").write_text(traceback.format_exc())
        raise
    finally:
        if collection is not None:
            write("detail.json", collection.finish())
        snapshots("operations.json")
        finished.set()
        watcher.join()
        with phase("runtime_release"):
            runtime.close()
        write(
            "outcome.json",
            {
                "case": args.case,
                "backend": args.backend,
                "size": args.size,
                "parameters": args.parameters,
                "status": status,
                "elapsed_seconds": time.monotonic() - started,
                "cancel_requested": expired.is_set(),
                "phases": phases,
            },
        )


if __name__ == "__main__":
    main()
