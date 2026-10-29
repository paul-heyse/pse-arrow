# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Targeted generic source, fixture and owned-result boundary checks."""

import subprocess
import sys
from pathlib import Path
from typing import cast

import msgspec
import pyarrow as pa
import pyarrow.parquet as pq
import pytest

import pse
from pse import codec
from pse.conformance import RunSummary, load_manifest
from pse.conformance import main as conformance_main
from pse.contracts import documents
from pse.contracts.enums import (
    DynamicsMethod,
    NativeSolveIntent,
)
from pse.contracts.identities import DeclarationId
from pse.contracts.values import SemanticId
from pse.tests.canonical_fixture import CanonicalFixture
from pse.tests.physical_input_fixture import (
    coordinate_batch,
    inspection_request,
    member,
)


def identity(n: int) -> SemanticId:
    return SemanticId(bytes([n]) * 16)


def declaration(n: int) -> DeclarationId:
    """The identity an authored `@id` gives a case or fixture declaration."""
    return DeclarationId(identity(n))


def preparation_policy(class_proof_work: int) -> documents.PreparationSettings:
    """Deliberate finite allowances for the small boundary test models."""
    return documents.PreparationSettings(
        compiler=documents.StudyCompilerProfile(
            class_proof_work=class_proof_work,
            assembly=documents.AssemblyLimits(
                contributions=1_000_000,
                native_index=2_147_483_647,
                worker_bytes=1 << 30,
            ),
            optimization=documents.Optimization(
                cores=1,
                horner_iterations=2,
                cpe_iterations=1,
                evaluator=documents.EvaluatorInterpreted(),
            ),
            evaluation=documents.EvaluationLimits(
                derivative_components=4096,
                operations=1_000_000,
                scratch_bytes=8 << 20,
                provider_calls=4096,
            ),
        ),
        limits=documents.Limits(depth=64, items=100_000, members=100_000),
    )


@pytest.mark.unit
def test_conformance_runs_the_declared_reference_set(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    """A manifest's runs execute once each, with their reports and one verdict line."""
    root = Path(__file__).resolve().parents[3]
    # The declared reference set names existing package roots.
    reference = load_manifest(root / "packages/reference/conformance.toml")
    assert reference.runs
    # Its diagnostic thresholds resolve beside the manifest.
    assert reference.settings.diagnostics is not None
    assert Path(reference.settings.diagnostics).is_file()
    # A tiny synthetic set: one pure run whose fixture declares its own allowance.
    package = tmp_path / "pure"
    (package / "models").mkdir(parents=True)
    (package / "package.toml").write_text(
        (root / "tests/fixtures/packages/minimal_explicit/package.toml")
        .read_text()
        .replace('id_policy = "explicit"', 'id_policy = "named"')
        .replace("dependencies = []", PRIMITIVES)
    )
    source = """package pure {
      fn square(x:Scalar)->Scalar=x*x;
      test positive fixture {
        dof 0; route steady; procedure check; policy { limits items(1000); }
      } {
        expect square(2)==4 tolerance 1e-12 relative 1e-8;
      }
    }"""
    (package / "models/pure.pse").write_text(source)
    manifest = tmp_path / "conformance.toml"
    text = f"""
version = 1
[settings]
version = 1
memory_limit_bytes = {16 << 30}
threads = 16
math_jobs = 16
maximum_checks = 32

[[runs]]
version = 1
name = "tiny"
package = "pure"
physical = "{root / "tests/fixtures/packages/physical-primitives"}"
execution = "pure"
"""
    manifest.write_text(text)
    reports = tmp_path / "reports"
    arguments = ["--manifest", str(manifest), "--report-dir", str(reports)]
    assert conformance_main(arguments) == 0
    (line,) = capsys.readouterr().out.splitlines()
    assert line.startswith("tiny: passed=True complete=True coverage=package checks=")
    assert line.endswith("fixtures=1 (passed 1)")
    for suffix in (
        ".arrow",
        ".fixtures.arrow",
        ".findings.arrow",
        ".parity.arrow",
        ".routes.arrow",
        ".structure.arrow",
    ):
        assert (reports / f"tiny{suffix}").is_file()
    # Plan 23 H6: the parity report is written beside the checks; this fixture names no
    # oracle, so it is not parity evidence.
    parity = pa.ipc.open_file(str(reports / "tiny.parity.arrow")).read_all()
    assert parity.num_rows == 0
    assert "release_id" in parity.column_names
    # A failed fixture fails the command.
    (package / "models/pure.pse").write_text(source.replace("==4", "==5"))
    assert conformance_main(arguments) == 1
    assert "tiny: passed=False complete=True" in capsys.readouterr().out
    # Unknown settings, duplicate runs and missing roots are refused before any run.
    for invalid in (
        text.replace("maximum_checks", "maximum_check"),
        text + text[text.index("[[runs]]") :],
        text.replace('package = "pure"', 'package = "absent"'),
        text.replace(
            "maximum_checks = 32", 'maximum_checks = 32\ndiagnostics = "absent.json"'
        ),
    ):
        manifest.write_text(invalid)
        with pytest.raises((msgspec.ValidationError, pse.InspectionError)):
            conformance_main(arguments)
    with pytest.raises(SystemExit):
        conformance_main([*arguments, "--report", str(tmp_path / "one.arrow")])


@pytest.mark.unit
def test_conformance_runs_only_selected_fixtures(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    """Run only the selected test declarations.

    Both manifest and single-package forms refuse unknown identities.
    """
    root = Path(__file__).resolve().parents[3]
    physical_root = root / "tests/fixtures/packages/physical-primitives"
    package = tmp_path / "pure"
    (package / "models").mkdir(parents=True)
    (package / "package.toml").write_text(
        (root / "tests/fixtures/packages/minimal_explicit/package.toml")
        .read_text()
        .replace('id_policy = "explicit"', 'id_policy = "named"')
        .replace("dependencies = []", PRIMITIVES)
    )
    first, second, wrong = declaration(201), declaration(202), declaration(203)
    (package / "models/pure.pse").write_text(
        f"""package pure {{
      fn square(x:Scalar)->Scalar=x*x;
      @id("{first.to_hex()}") test first fixture {{
        dof 0; route steady; procedure check;
      }} {{
        expect square(2)==4 tolerance 1e-12;
      }}
      @id("{second.to_hex()}") test second fixture {{
        dof 0; route steady; procedure check;
      }} {{
        expect square(3)==9 tolerance 1e-12;
      }}
      @id("{wrong.to_hex()}") test wrong fixture {{
        dof 0; route steady; procedure check;
      }} {{
        expect square(1)==2 tolerance 1e-12;
      }}
    }}"""
    )
    manifest = tmp_path / "conformance.toml"
    manifest.write_text(
        f"""
version = 1
[settings]
version = 1
memory_limit_bytes = {16 << 30}
maximum_checks = 32

[[runs]]
version = 1
name = "tiny"
package = "pure"
physical = "{physical_root}"
execution = "pure"
"""
    )
    reports = tmp_path / "reports"
    run = ["--manifest", str(manifest), "--report-dir", str(reports)]
    # Control: the whole package runs its failing fixture.
    assert conformance_main(run) == 1
    assert "coverage=package" in capsys.readouterr().out
    selected = ["--fixture", first.to_hex(), "--fixture", second.to_hex()]
    assert conformance_main([*run, *selected]) == 0
    (line,) = capsys.readouterr().out.splitlines()
    assert line.startswith("tiny: passed=True complete=True coverage=selected checks=")
    assert line.endswith("fixtures=2 (passed 2)")
    with pa.ipc.open_file(reports / "tiny.fixtures.arrow") as reader:
        inventory = reader.read_all().column("fixture_id").to_pylist()
    assert sorted(inventory) == sorted([first, second])
    # The single-package form takes the same selection.
    single = [
        str(package),
        "--pure",
        "--physical",
        str(physical_root),
        "--memory-limit-bytes",
        str(16 << 30),
        "--maximum-checks",
        "32",
    ]
    assert conformance_main([*single, "--fixture", second.to_hex()]) == 0
    assert "coverage=selected checks=" in capsys.readouterr().out
    # An identity that names no authored test is refused before any fixture runs, in
    # both
    # forms; a malformed identity is refused by the command line.
    unknown = declaration(204)
    for form in (run, single):
        with pytest.raises(pse.InspectionError, match=unknown.to_hex()):
            conformance_main(
                [*form, "--fixture", first.to_hex(), "--fixture", unknown.to_hex()]
            )
    with pytest.raises(SystemExit):
        conformance_main([*single, "--fixture", "not-an-identity"])


@pytest.mark.unit
def test_conformance_selects_one_manifest_run(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    """Run selection precedes fixture selection; invalid selection executes nothing."""
    root = Path(__file__).resolve().parents[3]
    physical_root = root / "tests/fixtures/packages/physical-primitives"
    manifest = tmp_path / "conformance.toml"
    manifest.write_text(
        f"""version = 1
[settings]
version = 1
memory_limit_bytes = {8 << 30}
[[runs]]
version = 1
name = "first"
package = "{physical_root}"
physical = "{physical_root}"
execution = "pure"
[[runs]]
version = 1
name = "second"
package = "{physical_root}"
physical = "{physical_root}"
execution = "pure"
"""
    )
    called: list[str] = []

    def record(name: str, *args: object, **kwargs: object) -> RunSummary:
        called.append(name)
        coverage = "selected" if kwargs["fixtures"] is not None else "package"
        return RunSummary(name, True, True, coverage, 1, {"passed": 1})

    monkeypatch.setattr("pse.conformance.run_once", record)
    arguments = ["--manifest", str(manifest), "--run", "second"]
    assert conformance_main(arguments) == 0
    assert called == ["second"]
    assert capsys.readouterr().out.startswith(
        "second: passed=True complete=True coverage=package"
    )
    called.clear()
    assert conformance_main([*arguments, "--fixture", declaration(201).to_hex()]) == 0
    assert called == ["second"]
    assert "coverage=selected" in capsys.readouterr().out
    called.clear()
    with pytest.raises(ValueError, match="unknown run"):
        conformance_main(["--manifest", str(manifest), "--run", "absent"])
    with pytest.raises(ValueError, match="one run"):
        conformance_main(
            ["--manifest", str(manifest), "--fixture", declaration(201).to_hex()]
        )
    with pytest.raises(SystemExit):
        conformance_main(["--run", "second"])
    assert called == []


#: A manifest dependency on the physical primitives fixture. Its document names
#: `Scalar` and `Time` (ADR-0123 Outcome 6).
PRIMITIVES = (
    'dependencies = [{ package_id = "5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a", '
    'version_req = { operator = "exact", major = 1, minor = 0, patch = 0 } }]'
)


def on_primitives(manifest: str) -> str:
    """A manifest whose package depends on the physical primitives."""
    return manifest.replace("dependencies = []", PRIMITIVES)


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


@pytest.mark.component
def test_binary_documents_admit_keyed_rows_and_reject_integer_sequences(
    inspection_settings: pse.EngineSettings, canonical_substrate: CanonicalFixture
) -> None:
    root = Path(__file__).resolve().parents[3]
    physical_root = root / "tests/fixtures/packages/physical-primitives"
    physical_documents = {
        str(path.relative_to(physical_root)): path.read_bytes()
        for path in physical_root.rglob("*")
        if path.is_file()
    }
    manifest = on_primitives(
        (root / "tests/fixtures/packages/minimal_named/package.toml").read_text()
    )
    source = """package binary {
      identifier scheme cas;
      entity kind item {attribute cas:Id<cas> unique;}
      entity item a {cas=Id<cas>("71-43-2")}
      entity kind source provenance {} entity source origin {}
      enum role {published}
      entity kind form {key subject:item by cas; attribute value:Time storage {s};}
      dataset bank:form provenance(origin,role.published) from "data/bank.parquet";
      test converted fixture {dof 0; route steady; procedure check;} {
        expect form[a].value==1{s} tolerance 1e-12{s};
      }
    }"""
    sink = pa.BufferOutputStream()
    pq.write_table(pa.table({"subject": ["71-43-2"], "value": [1.0]}), sink)
    binary = sink.getvalue().to_pybytes()
    documents: dict[str, str | bytes] = {
        "package.toml": manifest,
        "models/bank.pse": source.encode(),
        "data/bank.parquet": binary,
    }
    result = pse.ModelingConformance.pure(
        [documents], physical_documents, inspection_settings
    )
    assert result.passed
    assert result.complete
    # A bytes object is preserved; iterable integer values are not a binary document.
    invalid = {**documents, "data/bank.parquet": cast("str | bytes", [1, 2, 3])}
    with pytest.raises(TypeError, match="document content must be str or bytes"):
        pse.ModelingConformance.pure([invalid], physical_documents, inspection_settings)
    runtime = canonical_substrate.runtime(inspection_settings)
    usage = runtime.resource_usage()
    assert usage.limit_bytes == inspection_settings.memory_limit_bytes
    assert usage.pool_peak_bytes >= usage.pool_reserved_now >= 0


@pytest.mark.integration
def test_modeling_expansion_limits_are_explicit_and_isolated(
    inspection_settings: pse.EngineSettings, canonical_substrate: CanonicalFixture
) -> None:
    runtime = canonical_substrate.runtime(inspection_settings)
    root = Path(__file__).resolve().parents[3]
    manifest = (
        (root / "tests/fixtures/packages/minimal_explicit/package.toml")
        .read_text()
        .replace('id_policy = "explicit"', 'id_policy = "named"')
    ).replace("dependencies = []", PRIMITIVES)
    settings = pse.SolveSettings(intent=NativeSolveIntent.ROOT)
    package = runtime.modeling_from_documents(
        [
            {
                "package.toml": manifest,
                "models/limits.pse": f"""package limits {{
          test bounded fixture {{dof 0; route steady; procedure solve;}} {{
            var x:Scalar; annotation start x(1); eq solution:x==2;
            expect x==2 tolerance 0 relative
                {settings.numerics.engineering_relative_fraction};
          }}
        }}""",
            }
        ],
        physical(runtime),
    )
    case = next(d.declaration_id for d in package.declarations() if d.name == "bounded")
    limits = pse.ModelingLimits(items=1)
    assert limits.items == 1
    assert limits.depth > 0
    assert limits.members > 0
    limited = package.with_limits(limits)
    with pytest.raises(pse.InspectionError, match="specialized item count"):
        limited.solve_case(case, settings)
    assert package.solve_case(case, settings).accepted
    assert package.with_limits(pse.ModelingLimits()).solve_case(case, settings).accepted
    assert limits.body_occurrences is None
    with pytest.raises(pse.InspectionError, match="occurrences"):
        package.with_limits(pse.ModelingLimits(body_occurrences=1)).solve_case(
            case, settings
        )
    assert (
        package.with_limits(pse.ModelingLimits(body_occurrences=65536))
        .solve_case(case, settings)
        .accepted
    )
    with pytest.raises(pse.InspectionError, match="positive"):
        pse.ModelingLimits(body_occurrences=0)
    assert limits.body_slots is None
    with pytest.raises(pse.InspectionError, match="slots"):
        package.with_limits(pse.ModelingLimits(body_slots=1)).solve_case(case, settings)
    with pytest.raises(pse.InspectionError, match="positive"):
        pse.ModelingLimits(body_slots=0)
    with pytest.raises(pse.InspectionError, match="positive"):
        pse.ModelingLimits(members=0)


@pytest.mark.integration
def test_complete_preparation_policy_controls_public_preparation_and_execution(
    inspection_settings: pse.EngineSettings, canonical_substrate: CanonicalFixture
) -> None:
    runtime = canonical_substrate.runtime(inspection_settings)
    root = Path(__file__).resolve().parents[3]
    manifest = on_primitives(
        (root / "tests/fixtures/packages/minimal_explicit/package.toml")
        .read_text()
        .replace('id_policy = "explicit"', 'id_policy = "named"')
    )
    settings = pse.SolveSettings(intent=NativeSolveIntent.ROOT)
    package = runtime.modeling_from_documents(
        [
            {
                "package.toml": manifest,
                "models/preparation.pse": f"""package p {{
          test bounded fixture {{dof 0; route steady; procedure solve;}} {{
            var x:Scalar; annotation start x(1); eq solution:x==2;
            expect x==2 tolerance 0 relative
                {settings.numerics.engineering_relative_fraction};
          }}
        }}""",
            }
        ],
        physical(runtime),
    )
    case = next(
        row.declaration_id for row in package.declarations() if row.name == "bounded"
    )
    assert package.solve_case(case, settings).accepted
    zero = preparation_policy(0)
    for operation in (package.prepare_solve, package.solve_case):
        with pytest.raises(pse.InspectionError, match="presolve tape extent"):
            operation(case, settings, preparation=zero)
    larger = preparation_policy(10_000_000)
    limited = package.with_limits(pse.ModelingLimits(items=1))
    with pytest.raises(pse.InspectionError, match="specialized item count"):
        limited.prepare_solve(case, settings)
    prepared = limited.prepare_solve(case, settings, preparation=larger)
    assert (
        prepared.identity
        == package.prepare_solve(case, settings, preparation=larger).identity
    )
    assert limited.solve_case(case, settings, preparation=larger).accepted
    incomplete = msgspec.convert(msgspec.to_builtins(larger), type=dict[str, object])
    compiler = msgspec.convert(incomplete["compiler"], type=dict[str, object])
    del compiler["class_proof_work"]
    incomplete["compiler"] = compiler
    with pytest.raises(msgspec.ValidationError, match="class_proof_work"):
        codec.decode_json(codec.encode_json(incomplete), documents.PreparationSettings)
    with pytest.raises(pse.InspectionError, match="class_proof_work"):
        package.prepare_solve(
            case,
            settings,
            preparation=cast("documents.PreparationSettings", incomplete),
        )


@pytest.mark.integration
def test_modeling_simulation_events_checks_and_terminal_reports(
    inspection_settings: pse.EngineSettings, canonical_substrate: CanonicalFixture
) -> None:
    runtime = canonical_substrate.runtime(inspection_settings)
    root = Path(__file__).resolve().parents[3]
    # This controlled event/reset journey deliberately selects high fidelity to
    # distinguish localization, reset and terminal quadrature conditions.
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
 case run fixture {
  dof 0;
  route integrated; procedure integrate;
  integrate samples(0{s}, 0.25{s}, 0.75{s}, 2{s}) relative(1e-8)
   normalized_absolute(1e-8) step(1e-4{s}) quadrature_relative(1e-8)
   quadrature_absolute(root.area=1e-9{s});
  mode rise;
  event root.hit[0{s}] direction(either) tolerance(1e-8{s})
   reset(root.x[0{s}] = root.jump[0{s}]) next(coast);
  mode coast facts(stage.coast = true);
 } { child root:D=D(); }
}"""
    manifest = (
        (root / "tests/fixtures/packages/minimal_explicit/package.toml")
        .read_text()
        .replace('id_policy = "explicit"', 'id_policy = "named"')
    )
    manifest = on_primitives(manifest)
    package = runtime.modeling_from_documents(
        [{"package.toml": manifest, "models/dynamic.pse": source}],
        physical(runtime),
    )
    # The case's fixture declares the modes and the event (ADR-0119).
    case = next(d.declaration_id for d in package.declarations() if d.name == "run")
    trajectory = package.simulate(
        case,
        pse.SimulationSettings(
            runtime=runtime,
            start=0,
            end=2,
            samples=[0, 0.25, 0.75, 2],
            atol=coordinate_batch(
                runtime, [(member(package, case, "x"), 1e-8, "Time")]
            ),
            parameter_scales=coordinate_batch(
                runtime, [(member(package, case, "p"), 1.0, "Scalar")]
            ),
            out_rtol=1e-8,
            out_atol=coordinate_batch(
                runtime, [(member(package, case, "area"), 1e-9, "Time")]
            ),
            method=DynamicsMethod.DIFFSOL,
        ),
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


@pytest.mark.integration
def test_modeling_authored_fixture_conformance_cli_and_adapter_contracts(
    inspection_settings: pse.EngineSettings,
    tmp_path: Path,
    canonical_substrate: CanonicalFixture,
) -> None:
    runtime = canonical_substrate.runtime(inspection_settings)
    source = f'''@id("{identity(201).to_hex()}") package p {{
 @id("{identity(202).to_hex()}") def D {{
  @id("{identity(203).to_hex()}") var x:Scalar;
  @id("{identity(204).to_hex()}") eq e:x==2;
  @id("{identity(205).to_hex()}") annotation report x("value");
  @id("{identity(206).to_hex()}") annotation valid x(0,3);
 }}
 @id("{identity(207).to_hex()}") test sample
  fixture {{ dof -1; route simultaneous; procedure initialize; policy {{ presolve off;
   derivatives step(1e-7) tolerance(1e-4) cells(100); }} fix root.x = 2; }} {{
  @id("{identity(208).to_hex()}") child root:D=D();
  @id("{identity(209).to_hex()}") expect root.x==2 tolerance 1e-6;
 }}
}}'''
    root = Path(__file__).resolve().parents[3]
    manifest = (
        root / "tests/fixtures/packages/minimal_explicit/package.toml"
    ).read_text()
    manifest = on_primitives(manifest)
    package = runtime.modeling_from_documents(
        [{"package.toml": manifest, "models/example.pse": source}],
        physical(runtime),
    )
    # Native fixed-fixture science and owned tables are covered by
    # native_diagnostics. This retained adapter test compares CLI projection and
    # keeps authored-dispatch/admission refusals that have not migrated there.
    settings = pse.SolveSettings(intent=NativeSolveIntent.ROOT)
    conformance = package.conform(
        settings, controls=pse.ConformanceControls(maximum_checks=64)
    )
    checks = pa.table(conformance.table())
    assert conformance.passed, checks.to_pylist()
    assert conformance.complete, checks.to_pylist()
    routes = pa.table(conformance.admission("runtime.route_decisions"))
    structure = pa.table(conformance.admission("runtime.structural_assessments"))
    assert routes.num_rows == 1
    assert structure.num_rows == 1
    with pytest.raises(pse.InspectionError, match="admission table absent"):
        conformance.admission("runtime.modeling_checks")
    inspection = package.inspect(inspection_request(declaration(207), settings))
    assert inspection.execution.procedure == "initialize"
    assert inspection.execution.route == "simultaneous"
    with pytest.raises(pse.InspectionError, match="authored solve procedure"):
        package.prepare_solve(declaration(207), settings)
    with pytest.raises(pse.InspectionError, match="authored integration procedure"):
        package.simulate(
            declaration(207),
            pse.SimulationSettings(
                runtime=runtime,
                start=0.0,
                end=1.0,
                samples=[0.0, 1.0],
                atol=coordinate_batch(runtime),
                parameter_scales=coordinate_batch(runtime),
            ),
        )
    del conformance, package, runtime

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
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    assert command.returncode == 0, command.stdout + command.stderr
    assert "passed=True complete=True" in command.stdout
    with pa.ipc.open_file(report_path) as reader:
        assert reader.read_all().num_rows == checks.num_rows


@pytest.mark.component
def test_pure_conformance_cli_exports_reports(
    tmp_path: Path,
) -> None:
    root = Path(__file__).resolve().parents[3]
    physical_root = root / "tests/fixtures/packages/physical-primitives"
    manifest = (
        (root / "tests/fixtures/packages/minimal_explicit/package.toml")
        .read_text()
        .replace('id_policy = "explicit"', 'id_policy = "named"')
        .replace("dependencies = []", PRIMITIVES)
    )
    source = """package pure {
      fn square(x:Scalar)->Scalar=x*x;
      fn root(x:Scalar)->Scalar valid(x >= 0) = x;
      test positive fixture {dof 0; route steady; procedure check;} {
        expect square(2)==4 tolerance 1e-12 relative 1e-8;
      }
      test negative fixture {
        dof 0; route steady; procedure check;
        failure trial_rejected validity(form) form(root) variable(x);
      } {expect root(-1)==1 tolerance 0;}
    }"""
    documents = {"package.toml": manifest, "models/pure.pse": source}
    # Native conformance owns the scientific, validity and preparation assertions
    # in native_conformance.rs; this retained developer adapter checks CLI export.
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
            str(16 << 30),
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
