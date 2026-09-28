# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Targeted generic source, fixture and owned-result boundary checks."""

from pathlib import Path
import subprocess
import sys

import msgspec
import pyarrow as pa
import pytest

import pse
from pse import codec
from pse.contracts.enums import NativeBackend, NativeSolveIntent
from pse.contracts.values import SemanticId


def identity(n: int) -> SemanticId:
    return SemanticId(bytes([n]) * 16)


@pytest.mark.unit
@pytest.mark.parametrize("modes", [("invalid",), ("auto", "off")])
def test_conformance_cli_refuses_invalid_or_duplicate_fixture_presolve(
    modes: tuple[str, ...],
) -> None:
    from pse.conformance import main

    arguments = ["unused", "--physical", "unused", "--memory-limit-bytes", "1"]
    for mode in modes:
        arguments.extend(["--fixture-presolve", identity(207).to_hex(), mode])
    with pytest.raises(SystemExit) as refused:
        main(arguments)
    assert refused.value.code == 2


def quantity_aliases() -> str:
    return (
        f'\n[[quantity_aliases]]\nname = "Scalar"\nquantity_type_id = "{identity(31).to_hex()}"\n'
        f'\n[[quantity_aliases]]\nname = "Time"\nquantity_type_id = "{identity(222).to_hex()}"\n'
    )


def physical(runtime: pse.Runtime) -> pse.PhysicalContext:
    root = (
        Path(__file__).resolve().parents[3]
        / "tests/fixtures/packages/physical-primitives"
    )
    return runtime.physical_from_documents(
        {
            str(path.relative_to(root)): path.read_text()
            for path in root.rglob("*")
            if path.is_file()
        }
    )


@pytest.mark.unit
def test_modeling_expansion_limits_are_explicit_and_isolated(
    inspection_settings: pse.EngineSettings,
) -> None:
    runtime = pse.Runtime(inspection_settings)
    root = Path(__file__).resolve().parents[3]
    manifest = (
        (root / "tests/fixtures/packages/minimal_explicit/package.toml")
        .read_text()
        .replace('id_policy = "explicit"', 'id_policy = "named"')
    ) + quantity_aliases()
    package = runtime.modeling_from_documents(
        [
            {
                "package.toml": manifest,
                "models/limits.pse": """package limits {
          test bounded fixture {dof 0; run steady;} {
            var x:Scalar; annotation start x(1); eq solution:x==2;
            expect x==2 tolerance 1e-8;
          }
        }""",
            }
        ],
        physical(runtime),
    )
    case = next(d.declaration_id for d in package.declarations() if d.name == "bounded")
    limits = pse.ModelingLimits(items=1)
    assert limits.items == 1 and limits.depth > 0 and limits.members > 0
    limited = package.with_limits(limits)
    with pytest.raises(pse.InspectionError, match="specialized item count"):
        limited.solve_case(case, pse.SolveSettings(intent=NativeSolveIntent.ROOT))
    assert package.solve_case(
        case, pse.SolveSettings(intent=NativeSolveIntent.ROOT)
    ).accepted
    assert (
        package.with_limits(pse.ModelingLimits())
        .solve_case(case, pse.SolveSettings(intent=NativeSolveIntent.ROOT))
        .accepted
    )
    assert limits.body_occurrences is None
    with pytest.raises(pse.InspectionError, match="occurrences"):
        package.with_limits(pse.ModelingLimits(body_occurrences=1)).solve_case(
            case, pse.SolveSettings(intent=NativeSolveIntent.ROOT)
        )
    assert (
        package.with_limits(pse.ModelingLimits(body_occurrences=65536))
        .solve_case(case, pse.SolveSettings(intent=NativeSolveIntent.ROOT))
        .accepted
    )
    with pytest.raises(pse.InspectionError, match="positive"):
        pse.ModelingLimits(body_occurrences=0)
    with pytest.raises(pse.InspectionError, match="positive"):
        pse.ModelingLimits(members=0)


@pytest.mark.unit
def test_modeling_simulation_events_checks_and_terminal_reports(
    inspection_settings: pse.EngineSettings,
) -> None:
    runtime = pse.Runtime(inspection_settings)
    root = Path(__file__).resolve().parents[3]
    # Named identities are source-owned; the case selects the same dynamic definition.
    source = """package dynamic {
 def D {
  domain t: Time from 0{s} to 2{s};
  discretize grid on t using integrated(elements=1,order=1);
  param p:Scalar = 2;
  var x[i in t]:Time;
  eq rate[i in t]:d(x[i])/di == p;
  eq initial:x[0{s}] == 1{s};
  let hit[i in t]:Time=x[i]-2{s};
  let jump[i in t]:Time=2*x[i];
  let area:Time=integral(i in t | p);
  let score:Scalar=log(area/1{s});
  stage coast { override eq rate[i in t]:d(x[i])/di == 0; }
  annotation report score("log-integral");
  annotation check area(area>3{s});
  annotation check x(x[i]<=4.01{s});
 }
 case run { child root:D=D(); }
}"""
    manifest = (
        (root / "tests/fixtures/packages/minimal_explicit/package.toml")
        .read_text()
        .replace('id_policy = "explicit"', 'id_policy = "named"')
    )
    manifest += quantity_aliases()
    package = runtime.modeling_from_documents(
        [{"package.toml": manifest, "models/dynamic.pse": source}],
        physical(runtime),
    )
    case = next(d.declaration_id for d in package.declarations() if d.name == "run")
    modes = (
        pse.ModelingModeSettings(
            name="rise",
            events=[
                pse.ModelingEventSettings(
                    guard="root.hit[0{s}]",
                    tolerance=1e-8,
                    next_mode="coast",
                    reset={"root.x[0{s}]": "root.jump[0{s}]"},
                )
            ],
        ),
        pse.ModelingModeSettings(name="coast", facts={"stage.coast": True}),
    )
    trajectory = package.simulate(
        case,
        pse.SimulationSettings(
            start=0,
            end=2,
            samples=[0, 0.25, 0.75, 2],
            atol=[1e-8],
            parameter_scales=[1],
            out_rtol=1e-8,
            out_atol=[1e-9],
            method="diffsol",
        ),
        modes=modes,
    )
    assert trajectory.accepted, trajectory.validation_error
    assert trajectory.checks_complete
    events = trajectory.table("runtime.simulation_events")
    samples = trajectory.table()
    checks = trajectory.table("runtime.modeling_checks")
    reports = trajectory.table("runtime.modeling_reports")
    mode_table = trajectory.table("runtime.modeling_trajectory_modes")
    del trajectory, package, runtime
    assert pa.table(mode_table).column("mode").to_pylist() == [
        "rise",
        "rise",
        "coast",
        "coast",
    ]
    event_rows = pa.table(events).to_pylist()
    assert len(event_rows) == 1
    assert event_rows[0]["time"] == pytest.approx(0.5, abs=1e-5)
    assert event_rows[0]["before"] == pytest.approx(2, abs=1e-5)
    assert event_rows[0]["after"] == pytest.approx(4, abs=1e-5)
    assert pa.table(samples).to_pylist()[-1]["value"] == pytest.approx(4, abs=1e-5)
    check_rows = pa.table(checks).to_pylist()
    assert len(check_rows) == 5
    assert all(row["satisfied"] for row in check_rows)
    report_rows = pa.table(reports).to_pylist()
    assert len(report_rows) == 1
    assert report_rows[0]["value"] == pytest.approx(1.3862943611198906, abs=1e-6)


@pytest.mark.unit
def test_modeling_diagnostics_inspect_singular_case_without_solver_admission(
    inspection_settings: pse.EngineSettings,
) -> None:
    runtime = pse.Runtime(inspection_settings)
    root = Path(__file__).resolve().parents[3]
    source = f'''@id("{identity(221).to_hex()}") package diagnostic {{
 @id("{identity(222).to_hex()}") def D {{
  @id("{identity(223).to_hex()}") var x:Scalar;
  @id("{identity(224).to_hex()}") var y:Scalar;
  @id("{identity(235).to_hex()}") annotation nominal x(2);
  @id("{identity(236).to_hex()}") annotation nominal y(4);
  @id("{identity(225).to_hex()}") eq a:x+y==0;
  @id("{identity(226).to_hex()}") eq b:2*x+2*y==0;
 }}
 @id("{identity(220).to_hex()}") test sample fixture {{ dof 0; value root.x=1; value root.y=-1; }} {{
  @id("{identity(227).to_hex()}") child root:D=D();
  @id("{identity(228).to_hex()}") expect root.x==1 tolerance 1e-6;
 }}
 @id("{identity(229).to_hex()}") case unstarted {{ @id("{identity(232).to_hex()}") child root:D=D(); }}
}}'''
    package = runtime.modeling_from_documents(
        [
            {
                "package.toml": (
                    root / "tests/fixtures/packages/minimal_explicit/package.toml"
                ).read_text()
                + quantity_aliases(),
                "models/diagnostic.pse": source,
            }
        ],
        physical(runtime),
    )
    policy = {
        "profile": "synthetic",
        "maximum_findings": 50,
        "near_bound_absolute": 1e-4,
        "near_bound_relative": 1e-4,
        "bound_violation": 0.0,
        "residual": 1e-5,
        "variable_small": 1e-4,
        "variable_large": 1e4,
        "variable_zero": 1e-8,
        "jacobian_small": 1e-4,
        "jacobian_large": 1e4,
        "matrix": {
            "dense_entries": 100,
            "findings": 50,
            "parallel_tolerance": 1e-8,
            "rank_absolute": 1e-12,
            "rank_relative": 1e-8,
        },
        "terms": {
            "zero": 1e-10,
            "mismatch": 1e6,
            "cancellation": 1e-4,
            "maximum_terms": 5,
            "combinations": 100,
            "findings": 50,
        },
    }
    settings = pse.ModelingDiagnosticSettings.from_json(
        msgspec.json.encode(policy).decode()
    )
    assert msgspec.json.decode(settings.to_json()) == policy
    report = package.diagnose(
        identity(220), pse.SolveSettings(intent=NativeSolveIntent.ROOT), settings
    )
    missing = package.diagnose(
        identity(229), pse.SolveSettings(intent=NativeSolveIntent.ROOT), settings
    )
    assert missing.statistics()["missing_values"] == 2 and not missing.complete
    assert pa.table(missing.table()).to_pylist()[0]["point"] == []
    _, missing_columns = missing.coordinates()
    supplied = package.diagnose_samples(
        identity(229),
        pse.SolveSettings(intent=NativeSolveIntent.ROOT),
        settings,
        ((identity(233), {missing_columns[0]: 1.0, missing_columns[1]: -1.0}),),
    )
    supplied_report = supplied.result(0)
    assert supplied_report is not None and supplied_report.complete
    with pytest.raises(pse.InspectionError, match="missing case value"):
        package.solve_case(
            identity(229), pse.SolveSettings(intent=NativeSolveIntent.ROOT)
        )
    rows, columns = report.coordinates()
    samples = package.diagnose_samples(
        identity(220),
        pse.SolveSettings(intent=NativeSolveIntent.ROOT),
        settings,
        (
            (identity(230), {columns[0]: 2.0}),
            (identity(231), {identity(249): 1.0}),
            (identity(232), {}),
        ),
    )
    assert samples.stop == "completed" and samples.unattempted == 0
    assert samples.ids() == (identity(230), identity(231), identity(232))
    assert samples.result(0) is not None and samples.failure(1) is not None
    sample_outcomes = pa.table(samples.table()).to_pylist()[0]["outcomes"]
    sample_findings = samples.findings()
    failure_rows = pa.table(sample_findings).to_pylist()
    assert sample_outcomes[1]["failure_ordinal"] == 1
    assert failure_rows[0]["ordinal"] == 1
    assert failure_rows[0]["rule"]
    sampled = samples.result(2)
    assert sampled is not None and sampled.complete
    reference = pse.ModelingDiagnosticSettings.from_json(
        (root / "packages/reference/diagnostics/idaes-2.13.json").read_text()
    )
    assert msgspec.json.decode(reference.to_json())["profile"] == "idaes-2.13"
    capped = package.diagnose_samples(
        identity(220),
        pse.SolveSettings(intent=NativeSolveIntent.ROOT),
        reference,
        ((identity(230), {}), (identity(231), {})),
        maximum_samples=1,
    )
    assert capped.stop == "sample_limit" and capped.unattempted == 1
    native = package.diagnose_jacobian(
        identity(220),
        pse.SolveSettings(intent=NativeSolveIntent.ROOT),
        maximum_attempts=4,
    )
    assert len(native.attempts()) == 4
    native_table = native.table()
    limited = package.diagnose_jacobian(
        identity(220),
        pse.SolveSettings(intent=NativeSolveIntent.ROOT),
        maximum_attempts=1,
    )
    limited_row = pa.table(limited.table()).to_pylist()[0]
    assert not limited_row["complete"] and len(limited_row["attempts"]) == 1
    del native, limited
    sample_table = samples.table()
    capped_table = capped.table()
    diagnostic_table = report.table()
    finding_table = report.table("runtime.modeling_findings")
    nonfinite = package.diagnose_samples(
        identity(220),
        pse.SolveSettings(intent=NativeSolveIntent.ROOT),
        settings,
        ((identity(234), {columns[0]: float("nan")}),),
    )
    nonfinite_report = nonfinite.result(0)
    assert nonfinite_report is not None and not nonfinite_report.complete
    nonfinite_point = nonfinite_report.table()
    nonfinite_findings = nonfinite_report.table("runtime.modeling_findings")
    del nonfinite, nonfinite_report
    del samples, capped
    del package, runtime
    assert sampled.rank == 1
    assert report.complete and report.profile == "synthetic" and report.rank == 1
    assert report.cutoff is not None and report.cutoff > 0
    assert report.statistics()["variables"] == 2
    rows, columns = report.coordinates()
    assert len(rows) == len(columns) == 2
    modes = report.singular_modes()
    assert len(modes) == 2 and len(modes[0][1]) == len(modes[0][2]) == 2
    assert report.findings()
    del report
    durable = pa.table(diagnostic_table).to_pylist()[0]
    assert durable["matrix"]["rank"] == 1
    assert [v["source_id"] for v in durable["row_nominals"]] == durable["rows"]
    assert [v["source_id"] for v in durable["variable_nominals"]] == durable["columns"]
    assert sorted(v["value"] for v in durable["variable_nominals"]) == [2.0, 4.0]
    assert all(
        v["value"] > 0 for v in durable["row_nominals"] + durable["variable_nominals"]
    )
    assert all(
        v["source_id"] in durable["rows"] for v in durable["matrix"]["modes"][0]["left"]
    )
    assert pa.table(finding_table).num_rows > 0
    assert (
        pa.table(sample_table).to_pylist()[0]["outcomes"][1]["error_class"]
        == "invalid_model"
    )
    assert pa.table(capped_table).to_pylist()[0]["unattempted"] == 1
    invalid_point = pa.table(nonfinite_point).to_pylist()[0]
    assert any(
        v["kind"] == "indeterminate" and v["value"] is None
        for v in invalid_point["point"]
    )
    invalid_finding = next(
        row
        for row in pa.table(nonfinite_findings).to_pylist()
        if row["rule"] == "variable.nonfinite"
    )
    assert any(
        v["real_kind"] == "indeterminate" and v["real"] is None
        for v in invalid_finding["observations"]
    )
    native_row = pa.table(native_table).to_pylist()[0]
    assert native_row["complete"] and len(native_row["degenerate"]) == 1
    assert native_row["degenerate"][0]["irreducible_at_tolerance"]
    assert {v["source_id"] for v in native_row["row_nominals"]} == set(durable["rows"])
    assert native_row["row_nominals"] == durable["row_nominals"]
    assert native_row["variable_nominals"] == durable["variable_nominals"]
    assert {v["source_id"] for v in native_row["conditioning"][0]["weights"]} == set(
        durable["rows"]
    )
    with pytest.raises(pse.InspectionError):
        pse.ModelingDiagnosticSettings.from_json(
            settings.to_json().replace('"maximum_terms":5', '"maximum_terms":1')
        )


@pytest.mark.unit
def test_modeling_native_linear_diagnostics_preserve_scope_and_source_coordinates(
    inspection_settings: pse.EngineSettings,
) -> None:
    runtime = pse.Runtime(inspection_settings)
    root = Path(__file__).resolve().parents[3]
    source = """package linear {
 def Bad {var x:Scalar; eq lo:x>=2; eq hi:x<=1;}
 def Good {var x:Scalar; eq lo:x>=0; eq hi:x<=2;}
 def Curved {var x:Scalar; eq e:x*x==1;}
 case bad {child root:Bad=Bad();}
 case good {child root:Good=Good();}
 case curved {child root:Curved=Curved();}
}"""
    manifest = (
        (root / "tests/fixtures/packages/minimal_explicit/package.toml")
        .read_text()
        .replace('id_policy = "explicit"', 'id_policy = "named"')
    )
    manifest += quantity_aliases()
    package = runtime.modeling_from_documents(
        [{"package.toml": manifest, "models/linear.pse": source}],
        physical(runtime),
    )
    cases = {
        d.name: d.declaration_id
        for d in package.declarations()
        if d.name in {"bad", "good", "curved"}
    }
    settings = pse.SolveSettings(
        backend=NativeBackend.HIGHS, controls=pse.SolveControls(time_limit=30)
    )
    bad = package.diagnose_linear(
        cases["bad"], settings, iis=True, rays=True, relaxation=(-1, -1, 1)
    )
    assert bad.relation == "runtime.modeling_linear_diagnostics"
    assert bad.attempts()[0].termination == "infeasible"
    row = pa.table(bad.table()).to_pylist()[0]
    bad_table = bad.table()
    assert not row["iis"]["relaxation_only"]
    assert {r["source_id"] for r in row["iis"]["rows"]} == set(row["rows"])
    assert row["relaxation"]["operation_status"] == 0
    assert row["relaxation"]["penalty"] == pytest.approx(1.0)
    assert row["relaxation"]["restored_status"]["category"] == "inconclusive"
    assert row["relaxation"]["primal"][0]["source_id"] == row["columns"][0]
    assert row["attempt"]["termination"]["category"] == "infeasible"
    with pytest.raises(pse.InspectionError, match="every source coordinate"):
        package.diagnose_linear(
            cases["bad"], settings, relaxation=(-1, -1, 1), row_penalties={}
        )
    with pytest.raises(pse.InspectionError, match="affine coefficient"):
        package.diagnose_linear(cases["curved"], settings, iis=True)
    ranged = package.diagnose_linear(cases["good"], settings, ranging=True)
    ranges = ranged.table()
    del bad, ranged, package, runtime
    ranged_row = pa.table(ranges).to_pylist()[0]
    assert ranged_row["ranging"], ranged_row["unavailable"]
    coordinates = set(ranged_row["columns"] + ranged_row["rows"])
    for record in ranged_row["ranging"]:
        assert record["source_id"] in coordinates
        for name in ("value", "objective"):
            value = record[name]
            assert (value["kind"] == "finite") == (value["value"] is not None)
        for side in ("entering", "leaving"):
            endpoint = record[side]
            assert (endpoint["source_id"] in coordinates) != (
                endpoint["sentinel"] is not None
            )
    assert any(
        r["value"]["kind"] in {"positive_infinity", "negative_infinity"}
        for r in ranged_row["ranging"]
    )
    assert pa.table(bad_table).num_rows == 1


@pytest.mark.unit
def test_modeling_authored_fixture_shared_checks_and_owned_tables(
    inspection_settings: pse.EngineSettings, tmp_path: Path
) -> None:
    runtime = pse.Runtime(inspection_settings)
    source = f'''@id("{identity(201).to_hex()}") package p {{
 @id("{identity(202).to_hex()}") def D {{
  @id("{identity(203).to_hex()}") var x:Scalar;
  @id("{identity(204).to_hex()}") eq e:x==2;
  @id("{identity(205).to_hex()}") annotation report x("value");
  @id("{identity(206).to_hex()}") annotation valid x(0,1,extrapolate);
 }}
 @id("{identity(207).to_hex()}") test sample source "analytic:constant" revision "v1" fixture {{ dof -1; fix root.x = 2; }} {{
  @id("{identity(208).to_hex()}") child root:D=D();
  @id("{identity(209).to_hex()}") expect root.x==2 tolerance 1e-6;
 }}
}}'''
    root = Path(__file__).resolve().parents[3]
    manifest = (
        root / "tests/fixtures/packages/minimal_explicit/package.toml"
    ).read_text()
    manifest += quantity_aliases()
    package = runtime.modeling_from_documents(
        [{"package.toml": manifest, "models/example.pse": source}],
        physical(runtime),
    )
    assert len(package.declarations()) == 9
    settings = pse.SolveSettings(intent=NativeSolveIntent.ROOT)
    result = package.solve_case(identity(207), settings)
    assert result.accepted and result.outcome_kind == "constant_evaluation"
    assert result.attempt() is None and result.failure() is None
    local_policy = pse.ModelingFixturePolicy(
        codec.encode_json(settings), derivative_step=1e-7, derivative_cells=100
    )
    with pytest.raises(pse.InspectionError, match="unknown fixture"):
        package.conform(settings, fixture_policies={identity(250): local_policy})
    with pytest.raises(pse.InspectionError):
        pse.ModelingFixturePolicy(derivative_step=0)
    conformance = package.conform(
        settings,
        maximum_checks=64,
        derivative_cells=100,
        fixture_policies={identity(207): local_policy},
    )
    checks = pa.table(conformance.table())
    assert conformance.passed and conformance.complete, checks.to_pylist()
    assert conformance.fixtures() == (identity(207),)
    extracted = conformance.result(identity(207))
    initialized = package.initialize(identity(207), settings, maximum_attempts=2)
    assert initialized.completed and initialized.failure is None
    history = initialized.attempts()
    assert len(history) == 1 and history[0].kind == "original" and history[0].accepted
    assert history[0].fraction is None and history[0].stage is None
    assert history[0].preparation_error is None and history[0].interruption is None
    assert initialized.committed_values() is not None
    final = history[0].result()
    assert final is not None and final.accepted
    study = package.study(
        (identity(207), identity(207)), settings, predecessors=(None, 0)
    )
    assert study.count == 2 and study.unattempted == 0
    assert study.failure(0) is None and study.failure(1) is None
    second = study.result(1)
    assert second is not None and second.accepted
    isolated = package.study(
        (identity(207), identity(250), identity(207), identity(207)),
        settings,
        predecessors=(None, None, 1, None),
    )
    assert isolated.count == 4 and isolated.unattempted == 0
    assert isolated.result(0) is not None and isolated.result(3) is not None
    assert isolated.result(1) is None and isolated.failure(1) is not None
    assert isolated.result(2) is None
    failed_predecessor = isolated.failure(2)
    assert failed_predecessor is not None
    assert failed_predecessor.rule == "modeling.study.predecessor"
    with pytest.raises(pse.InspectionError, match="integrated time axis"):
        package.simulate(
            identity(207),
            pse.SimulationSettings(
                start=0.0,
                end=1.0,
                samples=[0.0, 1.0],
                atol=[],
                parameter_scales=[],
            ),
        )
    with pytest.raises(pse.InspectionError, match="earlier point"):
        package.study((identity(207),), settings, predecessors=(0,))
    initialization_table = initialized.table()
    study_table = study.table()
    isolated_table = isolated.table()
    reports = pa.table(extracted.table("runtime.modeling_reports"))
    validation = pa.table(result.table("runtime.modeling_checks"))
    del extracted, conformance, result, package, runtime
    del initialized, history, study, final, second
    initial = pa.table(initialization_table).to_pylist()[0]
    assert initial["complete"] and initial["attempts"][0]["kind"] == "original"
    assert pa.table(study_table).to_pylist()[0]["points"][1]["predecessor"] == 0
    isolated_rows = pa.table(isolated_table).to_pylist()[0]["points"]
    assert (
        isolated_rows[1]["error"] is not None and isolated_rows[1]["result_id"] is None
    )
    assert isolated_rows[2]["predecessor"] == 1 and not isolated_rows[2]["accepted"]
    assert reports.column("value").to_pylist() == [2.0]
    assert any(
        row["within_validity"] is False and row["extrapolation_allowed"] is True
        for row in validation.to_pylist()
    )
    assert "coverage" in checks.column("kind").to_pylist()

    package_path = tmp_path / "model"
    package_path.mkdir()
    (package_path / "package.toml").write_text(manifest)
    (package_path / "models").mkdir()
    (package_path / "models/example.pse").write_text(source)
    report_path = tmp_path / "checks.arrow"
    command = subprocess.run(
        [
            sys.executable,
            "-m",
            "pse.conformance",
            str(package_path),
            "--physical",
            str(root / "tests/fixtures/packages/physical-primitives"),
            "--memory-limit-bytes",
            str(64 << 30),
            "--report",
            str(report_path),
            "--maximum-checks",
            "64",
            "--derivative-cells",
            "100",
            "--fixture-solver",
            identity(207).to_hex(),
            "root",
            "auto",
            "--fixture-presolve",
            identity(207).to_hex(),
            "off",
            "--fixture-derivatives",
            identity(207).to_hex(),
            "1e-7",
            "1e-4",
            "100",
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    assert command.returncode == 0, command.stdout + command.stderr
    assert "passed=True complete=True" in command.stdout
    with pa.ipc.open_file(report_path) as reader:
        assert reader.read_all().num_rows == checks.num_rows


@pytest.mark.unit
def test_modeling_nonlinear_explanation_retains_local_evidence(
    inspection_settings: pse.EngineSettings,
) -> None:
    runtime = pse.Runtime(inspection_settings)
    root = Path(__file__).resolve().parents[3]
    manifest = (
        (root / "tests/fixtures/packages/minimal_explicit/package.toml")
        .read_text()
        .replace('id_policy = "explicit"', 'id_policy = "named"')
    )
    source = """package p {
 def D { var x:Scalar; eq lo:x*x>=4; eq hi:x*x<=1; eq spare:x*x<=100; annotation start x(1.5); }
 case run { child root:D=D(); }
}"""
    manifest += quantity_aliases()
    package = runtime.modeling_from_documents(
        [{"package.toml": manifest, "models/explanation.pse": source}],
        physical(runtime),
    )
    case = next(d.declaration_id for d in package.declarations() if d.name == "run")
    settings = pse.SolveSettings(
        backend=NativeBackend.IPOPT, intent=NativeSolveIntent.OPTIMIZE
    )
    diagnostics = package.diagnose(
        case,
        settings,
        pse.ModelingDiagnosticSettings.from_json(
            (root / "packages/reference/diagnostics/idaes-2.13.json").read_text(),
        ),
    )
    rows, _ = diagnostics.coordinates()
    explanation = package.explain_nonlinear(
        case,
        settings,
        {row: 1.0 for row in rows},
        penalty_tolerance=1e-6,
        maximum_attempts=4,
        time_limit=30,
    )
    assert explanation.complete
    assert len(explanation.candidate_rows()) == 2
    assert explanation.background_variables() == ()
    attempts = explanation.attempts()
    explanation_table = explanation.table()
    findings_table = explanation.findings()
    limited = package.explain_nonlinear(
        case,
        settings,
        {row: 1.0 for row in rows},
        penalty_tolerance=1e-6,
        maximum_attempts=1,
        time_limit=30,
    )
    assert not limited.complete and limited.stop is not None
    limited_findings = pa.table(limited.findings()).to_pylist()
    assert limited_findings[0]["ordinal"] == 0
    assert limited_findings[0]["class"] == "resource_limit"
    assert limited_findings[0]["rule"] == "modeling.nonlinear.attempt_limit"
    del explanation, diagnostics, package, runtime
    durable = pa.table(explanation_table).to_pylist()[0]
    assert durable["complete"] and len(durable["candidate_rows"]) == 2
    assert durable["attempts"][0]["observation"] == "local_obstruction"
    findings = {row["ordinal"]: row for row in pa.table(findings_table).to_pylist()}
    assert durable["attempts"][0]["failure_ordinal"] == 1
    assert findings[1]["rule"] == "modeling.qualification.rejected"
    assert findings[1]["sources"] and findings[1]["observations"]
    # The ℓ1 route builds no elastic model (ADR-0109 item 2a): the least-infeasible point
    # names the original rows it leaves violated, all among the candidate rows (item 3).
    assert set(findings[1]["sources"]) <= set(durable["candidate_rows"])
    observations = {o["name"]: o["text"] for o in findings[1]["observations"]}
    assert observations["candidate_use"] == "diagnostic_only"
    assert len(attempts) == 4
    assert attempts[0].observation == "local_obstruction"
    assert attempts[0].failure() is not None
    assert attempts[0].penalty == pytest.approx(3.0, abs=1e-5)
    assert sum(a.observation == "feasible_witness" for a in attempts) == 2
    result = attempts[0].result()
    assert result is not None
    assert not result.accepted
    attempt = result.attempt()
    assert attempt is not None and attempt.backend == "pounce"


@pytest.mark.unit
def test_pure_conformance_has_no_process_runtime_and_retains_findings(
    tmp_path: Path,
) -> None:
    root = Path(__file__).resolve().parents[3]
    physical_root = root / "tests/fixtures/packages/physical-primitives"
    physical_documents = {
        str(path.relative_to(physical_root)): path.read_text()
        for path in physical_root.rglob("*")
        if path.is_file()
    }
    manifest = (
        root / "tests/fixtures/packages/minimal_explicit/package.toml"
    ).read_text().replace(
        'id_policy = "explicit"', 'id_policy = "named"'
    ) + quantity_aliases()
    source = """package pure {
      fn square(x:Scalar)->Scalar=x*x;
      test positive fixture {dof 0; run pure;} {expect square(2)==4 tolerance 1e-12 relative 1e-8;}
      test negative fixture {dof 0; run pure; failure invalid_model "compiler.missing";} {expect 1==1 tolerance 0;}
    }"""
    documents = {"package.toml": manifest, "models/pure.pse": source}
    # Distinct budgets must be accepted even if another test already owns the process Runtime.
    for threads in (1, 2):
        result = pse.ModelingConformance.pure(
            [documents],
            physical_documents,
            pse.EngineSettings(
                memory_limit_bytes=8 << 30,
                threads=threads,
                spill_dir=str(tmp_path),
                max_spill_bytes=1 << 30,
                batch_size=1024,
            ),
            maximum_checks=32,
        )
        assert result.passed and result.complete
        inventory = pa.table(result.fixture_statuses())
        assert inventory.num_rows == 2
        assert inventory.column("status").to_pylist() == ["passed", "passed"]
        findings = pa.table(result.findings())
        assert findings.column("rule").to_pylist() == ["compiler.missing"]
        assert result.failure(0).boundary_class == "invalid_model"
        table = result.table()
        del result
        assert pa.table(table).num_rows > 0
    limited = pse.ModelingConformance.pure(
        [documents],
        physical_documents,
        pse.EngineSettings(
            memory_limit_bytes=8 << 30,
            threads=1,
            spill_dir=str(tmp_path),
            max_spill_bytes=1 << 30,
            batch_size=1024,
        ),
        maximum_checks=1,
    )
    assert not limited.complete and not limited.passed
    assert len(limited.fixtures()) == 2
    statuses = pa.table(limited.fixture_statuses()).column("status").to_pylist()
    assert sorted(statuses) == ["inconclusive", "unattempted"]
    package_path = tmp_path / "pure"
    (package_path / "models").mkdir(parents=True)
    for name, content in documents.items():
        (package_path / name).write_text(content)
    command = subprocess.run(
        [
            sys.executable,
            "-m",
            "pse.conformance",
            str(package_path),
            "--pure",
            "--physical",
            str(physical_root),
            "--memory-limit-bytes",
            str(8 << 30),
            "--maximum-checks",
            "32",
            "--report",
            str(tmp_path / "checks.arrow"),
        ],
        capture_output=True,
        text=True,
        check=False,
    )
    assert command.returncode == 0, command.stdout + command.stderr
    assert (tmp_path / "checks.fixtures.arrow").is_file()
    assert (tmp_path / "checks.findings.arrow").is_file()
