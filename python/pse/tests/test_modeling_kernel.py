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
    DiagnosticCode,
    DiagnosticRule,
    NativeBackend,
    NativeBoundaryClass,
    NativeSolveIntent,
)
from pse.contracts.identities import DeclarationId
from pse.contracts.values import SemanticId


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
                cores=1, horner_iterations=2, cpe_iterations=1
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
[settings]
memory_limit_bytes = {16 << 30}
threads = 16
math_jobs = 16
maximum_checks = 32

[[runs]]
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
        with pytest.raises((msgspec.ValidationError, ValueError)):
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
[settings]
memory_limit_bytes = {16 << 30}
maximum_checks = 32

[[runs]]
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
        f"""[settings]
memory_limit_bytes = {8 << 30}
[[runs]]
name = "first"
package = "{physical_root}"
physical = "{physical_root}"
execution = "pure"
[[runs]]
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
    inspection_settings: pse.EngineSettings, canonical_substrate: str
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
    runtime = pse.Runtime(inspection_settings, substrate=canonical_substrate)
    usage = runtime.resource_usage()
    assert usage.limit_bytes == inspection_settings.memory_limit_bytes
    assert usage.pool_peak_bytes >= usage.pool_reserved_now >= 0


@pytest.mark.integration
def test_modeling_expansion_limits_are_explicit_and_isolated(
    inspection_settings: pse.EngineSettings, canonical_substrate: str
) -> None:
    runtime = pse.Runtime(inspection_settings, substrate=canonical_substrate)
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
    inspection_settings: pse.EngineSettings, canonical_substrate: str
) -> None:
    runtime = pse.Runtime(inspection_settings, substrate=canonical_substrate)
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


@pytest.mark.component
def test_conformance_preparation_policy_reaches_native_and_pure_compilers(
    tmp_path: Path, inspection_settings: pse.EngineSettings, canonical_substrate: str
) -> None:
    root = Path(__file__).resolve().parents[3]
    physical_root = root / "tests/fixtures/packages/physical-primitives"
    manifest = on_primitives(
        (root / "tests/fixtures/packages/minimal_explicit/package.toml")
        .read_text()
        .replace('id_policy = "explicit"', 'id_policy = "named"')
    )
    source = {
        "package.toml": manifest,
        "models/pure.pse": """package p {
      fn square(x:Scalar)->Scalar=x*x;
      test positive fixture {dof 0; route steady; procedure check;} {
        expect square(2)==4 tolerance 1e-12;
      }
    }""",
    }
    physical_documents = {
        str(path.relative_to(physical_root)): path.read_text()
        for path in physical_root.rglob("*")
        if path.is_file()
    }
    runtime = pse.Runtime(inspection_settings, substrate=canonical_substrate)
    package = runtime.modeling_from_documents([source], physical(runtime))
    default = package.conform(pse.SolveSettings())
    assert default.passed
    assert default.complete
    policy = preparation_policy(1_000_000)
    positive = package.conform(pse.SolveSettings(), preparation=policy)
    assert positive.passed
    assert positive.complete
    # Even a constant result needs a numeric evaluator frame. One positive byte
    # cannot hold its scalar, unlike an unused coefficient-class proof allowance.
    insufficient = msgspec.structs.replace(
        policy,
        compiler=msgspec.structs.replace(
            policy.compiler,
            evaluation=msgspec.structs.replace(
                policy.compiler.evaluation, scratch_bytes=1
            ),
        ),
    )
    refused = package.conform(pse.SolveSettings(), preparation=insufficient)
    assert not refused.passed
    failure = refused.failure(0)
    assert failure is not None
    assert failure.envelope is not None
    assert failure.envelope.code == DiagnosticCode.RUNTIME_RESOURCE_LIMIT
    assert failure.envelope.rule == DiagnosticRule.MATH_LIMIT
    assert failure.envelope.class_ == NativeBoundaryClass.RESOURCE_LIMIT
    detail = failure.envelope.observations["detail"]
    assert isinstance(detail, documents.ObservationText)
    assert detail.value == "math limit exceeded: worker scratch bytes"
    engine = pse.EngineSettings(
        memory_limit_bytes=8 << 30,
        math_worker_bytes=8 << 20,
        math_workspace_bytes=8 << 20,
        threads=1,
        spill_dir=str(tmp_path),
        max_spill_bytes=1 << 30,
        batch_size=1024,
    )
    positive = pse.ModelingConformance.pure(
        [source], physical_documents, engine, preparation=policy
    )
    assert positive.passed
    assert positive.complete
    refused = pse.ModelingConformance.pure(
        [source], physical_documents, engine, preparation=insufficient
    )
    assert not refused.passed
    failure = refused.failure(0)
    assert failure is not None
    assert failure.envelope is not None
    assert failure.envelope.code == DiagnosticCode.RUNTIME_RESOURCE_LIMIT
    assert failure.envelope.rule == DiagnosticRule.MATH_LIMIT
    assert failure.envelope.class_ == NativeBoundaryClass.RESOURCE_LIMIT
    detail = failure.envelope.observations["detail"]
    assert isinstance(detail, documents.ObservationText)
    assert detail.value == "math limit exceeded: worker scratch bytes"


@pytest.mark.integration
def test_modeling_simulation_events_checks_and_terminal_reports(
    inspection_settings: pse.EngineSettings, canonical_substrate: str
) -> None:
    runtime = pse.Runtime(inspection_settings, substrate=canonical_substrate)
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
            start=0,
            end=2,
            samples=[0, 0.25, 0.75, 2],
            atol=[1e-8],
            parameter_scales=[1],
            out_rtol=1e-8,
            out_atol=[1e-9],
            method="diffsol",
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
def test_modeling_diagnostics_inspect_singular_case_without_solver_admission(
    inspection_settings: pse.EngineSettings, canonical_substrate: str
) -> None:
    runtime = pse.Runtime(inspection_settings, substrate=canonical_substrate)
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
 @id("{identity(220).to_hex()}") test sample fixture {{
  dof 0; value root.x=1; value root.y=-1;
 }} {{
  @id("{identity(227).to_hex()}") child root:D=D();
  @id("{identity(228).to_hex()}") expect root.x==1 tolerance 1e-6;
 }}
 @id("{identity(229).to_hex()}") case unstarted {{
  @id("{identity(232).to_hex()}") child root:D=D();
 }}
}}'''
    package = runtime.modeling_from_documents(
        [
            {
                "package.toml": on_primitives(
                    (
                        root / "tests/fixtures/packages/minimal_explicit/package.toml"
                    ).read_text()
                ),
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
            "singular_vector": 0.1,
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
        declaration(220), pse.SolveSettings(intent=NativeSolveIntent.ROOT), settings
    )
    missing = package.diagnose(
        declaration(229), pse.SolveSettings(intent=NativeSolveIntent.ROOT), settings
    )
    assert missing.statistics()["missing_values"] == 2
    assert not missing.complete
    assert pa.table(missing.table()).to_pylist()[0]["point"] == []
    _, missing_columns = missing.coordinates()
    supplied = package.diagnose_samples(
        declaration(229),
        pse.SolveSettings(intent=NativeSolveIntent.ROOT),
        settings,
        ((identity(233), {missing_columns[0]: 1.0, missing_columns[1]: -1.0}),),
    )
    supplied_report = supplied.result(0)
    assert supplied_report is not None
    assert supplied_report.complete
    with pytest.raises(pse.InspectionError, match="missing case value"):
        package.solve_case(
            declaration(229), pse.SolveSettings(intent=NativeSolveIntent.ROOT)
        )
    rows, columns = report.coordinates()
    samples = package.diagnose_samples(
        declaration(220),
        pse.SolveSettings(intent=NativeSolveIntent.ROOT),
        settings,
        (
            (identity(230), {columns[0]: 2.0}),
            (identity(231), {identity(249): 1.0}),
            (identity(232), {}),
        ),
    )
    assert samples.stop == "completed"
    assert samples.unattempted == 0
    assert samples.ids() == (identity(230), identity(231), identity(232))
    assert samples.result(0) is not None
    assert samples.failure(1) is not None
    sample_outcomes = pa.table(samples.table()).to_pylist()[0]["outcomes"]
    sample_findings = samples.findings()
    failure_rows = pa.table(sample_findings).to_pylist()
    assert sample_outcomes[1]["failure_ordinal"] == 1
    assert failure_rows[0]["ordinal"] == 1
    assert failure_rows[0]["rule"]
    sampled = samples.result(2)
    assert sampled is not None
    assert sampled.complete
    reference = pse.ModelingDiagnosticSettings.from_json(
        (root / "packages/reference/diagnostics/idaes-2.13.json").read_text()
    )
    assert msgspec.json.decode(reference.to_json())["profile"] == "idaes-2.13"
    capped = package.diagnose_samples(
        declaration(220),
        pse.SolveSettings(intent=NativeSolveIntent.ROOT),
        reference,
        ((identity(230), {}), (identity(231), {})),
        controls=pse.DiagnosticSamplesControls(maximum_samples=1),
    )
    assert capped.stop == "sample_limit"
    assert capped.unattempted == 1
    native = package.diagnose_jacobian(
        declaration(220),
        pse.SolveSettings(intent=NativeSolveIntent.ROOT),
        controls=pse.JacobianDiagnosticControls(maximum_attempts=4),
    )
    assert len(native.attempts()) == 4
    native_table = native.table()
    limited = package.diagnose_jacobian(
        declaration(220),
        pse.SolveSettings(intent=NativeSolveIntent.ROOT),
        controls=pse.JacobianDiagnosticControls(maximum_attempts=1),
    )
    limited_row = pa.table(limited.table()).to_pylist()[0]
    assert not limited_row["complete"]
    assert len(limited_row["attempts"]) == 1
    del native, limited
    sample_table = samples.table()
    capped_table = capped.table()
    diagnostic_table = report.table()
    finding_table = report.table("runtime.modeling_findings")
    nonfinite = package.diagnose_samples(
        declaration(220),
        pse.SolveSettings(intent=NativeSolveIntent.ROOT),
        settings,
        ((identity(234), {columns[0]: float("nan")}),),
    )
    nonfinite_report = nonfinite.result(0)
    assert nonfinite_report is not None
    assert not nonfinite_report.complete
    nonfinite_point = nonfinite_report.table()
    nonfinite_findings = nonfinite_report.table("runtime.modeling_findings")
    del nonfinite, nonfinite_report
    del samples, capped
    del package, runtime
    assert sampled.rank == 1
    assert report.complete
    assert report.profile == "synthetic"
    assert report.rank == 1
    assert report.cutoff is not None
    assert report.cutoff > 0
    assert report.statistics()["variables"] == 2
    rows, columns = report.coordinates()
    assert len(rows) == len(columns) == 2
    modes = report.singular_modes()
    assert len(modes) == 2
    assert len(modes[0][1]) == len(modes[0][2]) == 2
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
    assert native_row["complete"]
    assert len(native_row["degenerate"]) == 1
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


@pytest.mark.integration
def test_modeling_native_linear_diagnostics_preserve_scope_and_source_coordinates(
    inspection_settings: pse.EngineSettings, canonical_substrate: str
) -> None:
    runtime = pse.Runtime(inspection_settings, substrate=canonical_substrate)
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
    manifest = on_primitives(manifest)
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
        cases["bad"],
        settings,
        controls=pse.LinearDiagnosticControls(
            iis=True, rays=True, relaxation=(-1, -1, 1)
        ),
    )
    assert bad.relation == "runtime.modeling_linear_diagnostics"
    assert bad.attempts()[0].termination == "infeasible"
    row = pa.table(bad.table()).to_pylist()[0]
    bad_table = bad.table()
    assert not row["iis"]["relaxation_only"]
    assert {r["source_id"] for r in row["iis"]["rows"]} == set(row["rows"])
    assert row["relaxation"]["operation_status"] == 0
    assert row["relaxation"]["penalty"] == pytest.approx(
        1.0, rel=settings.numerics.engineering_relative_fraction, abs=0.0
    )
    assert row["relaxation"]["restored_status"]["category"] == "inconclusive"
    assert row["relaxation"]["primal"][0]["source_id"] == row["columns"][0]
    assert row["attempt"]["termination"]["category"] == "infeasible"
    with pytest.raises(pse.InspectionError, match="every source coordinate"):
        package.diagnose_linear(
            cases["bad"],
            settings,
            controls=pse.LinearDiagnosticControls(
                relaxation=(-1, -1, 1), row_penalties={}
            ),
        )
    with pytest.raises(pse.InspectionError, match="affine coefficient"):
        package.diagnose_linear(
            cases["curved"], settings, controls=pse.LinearDiagnosticControls(iis=True)
        )
    ranged = package.diagnose_linear(
        cases["good"], settings, controls=pse.LinearDiagnosticControls(ranging=True)
    )
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


@pytest.mark.integration
def test_modeling_authored_fixture_shared_checks_and_owned_tables(
    inspection_settings: pse.EngineSettings, tmp_path: Path, canonical_substrate: str
) -> None:
    runtime = pse.Runtime(inspection_settings, substrate=canonical_substrate)
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
    assert len(package.declarations()) == 9
    settings = pse.SolveSettings(intent=NativeSolveIntent.ROOT)
    result = package.solve_case(declaration(207), settings)
    assert result.accepted
    assert result.outcome_kind == "constant_evaluation"
    assert result.attempt() is None
    assert result.failure() is None
    conformance = package.conform(
        settings, controls=pse.ConformanceControls(maximum_checks=64)
    )
    checks = pa.table(conformance.table())
    assert conformance.passed, checks.to_pylist()
    assert conformance.complete, checks.to_pylist()
    assert conformance.fixtures() == (identity(207),)
    extracted = conformance.result(declaration(207))
    routes = pa.table(conformance.admission("runtime.route_decisions"))
    structure = pa.table(conformance.admission("runtime.structural_assessments"))
    assert routes.num_rows == 1
    assert structure.num_rows == 1
    with pytest.raises(pse.InspectionError, match="admission table absent"):
        conformance.admission("runtime.modeling_checks")
    assert (
        package.inspect(declaration(207), settings).execution.procedure == "initialize"
    )
    assert package.inspect(declaration(207), settings).execution.route == "simultaneous"
    with pytest.raises(pse.InspectionError, match="authored solve procedure"):
        package.prepare_solve(declaration(207), settings)
    initialized = package.initialize(
        declaration(207),
        settings,
        overrides=pse.InitializationOverrides(maximum_attempts=2),
    )
    assert initialized.completed
    assert initialized.failure is None
    history = initialized.attempts()
    assert len(history) == 1
    assert history[0].kind == "original"
    assert history[0].accepted
    assert history[0].fraction is None
    assert history[0].stage is None
    assert history[0].preparation_error is None
    assert history[0].interruption is None
    assert initialized.committed_values() is not None
    final = history[0].result()
    assert final is not None
    assert final.accepted
    with pytest.raises(pse.InspectionError, match="authored integration procedure"):
        package.simulate(
            declaration(207),
            pse.SimulationSettings(
                start=0.0,
                end=1.0,
                samples=[0.0, 1.0],
                atol=[],
                parameter_scales=[],
            ),
        )
    initialization_table = initialized.table()
    reports = pa.table(extracted.table("runtime.modeling_reports"))
    validation = pa.table(result.table("runtime.modeling_checks"))
    del extracted, conformance, result, package, runtime
    del initialized, history, final
    initial = pa.table(initialization_table).to_pylist()[0]
    assert initial["complete"]
    assert initial["attempts"][0]["kind"] == "original"
    assert reports.column("value").to_pylist() == [2.0]
    # An annotated hard range retains its closure layer and cannot select an
    # empirical evidence permission policy.
    assert any(
        row["within_validity"] is True
        and row["extrapolation_allowed"] is None
        and row["satisfied"] is True
        and row["layer"] == "closure"
        for row in validation.to_pylist()
    )
    assert all(
        (row["layer"] is None) == (row["kind"] != "validity")
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
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    assert command.returncode == 0, command.stdout + command.stderr
    assert "passed=True complete=True" in command.stdout
    with pa.ipc.open_file(report_path) as reader:
        assert reader.read_all().num_rows == checks.num_rows


@pytest.mark.integration
def test_modeling_nonlinear_explanation_retains_local_evidence(
    inspection_settings: pse.EngineSettings, canonical_substrate: str
) -> None:
    runtime = pse.Runtime(inspection_settings, substrate=canonical_substrate)
    root = Path(__file__).resolve().parents[3]
    manifest = (
        (root / "tests/fixtures/packages/minimal_explicit/package.toml")
        .read_text()
        .replace('id_policy = "explicit"', 'id_policy = "named"')
    )
    source = """package p {
 def D {
  var x:Scalar; eq lo:x*x>=4; eq hi:x*x<=1; eq spare:x*x<=100;
  annotation start x(1.5);
 }
 case run { child root:D=D(); }
}"""
    manifest = on_primitives(manifest)
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
        codec.decode_json(
            codec.encode_json(
                {
                    "nominals": {row.to_hex(): 1.0 for row in rows},
                    "penalty_tolerance": 1e-6,
                    "maximum_attempts": 4,
                    "time_limit": {"secs": 30, "nanos": 0},
                }
            ),
            pse.ModelingNonlinearPolicy,
        ),
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
        codec.decode_json(
            codec.encode_json(
                {
                    "nominals": {row.to_hex(): 1.0 for row in rows},
                    "penalty_tolerance": 1e-6,
                    "maximum_attempts": 1,
                    "time_limit": {"secs": 30, "nanos": 0},
                }
            ),
            pse.ModelingNonlinearPolicy,
        ),
    )
    assert not limited.complete
    assert limited.stop is not None
    limited_findings = pa.table(limited.findings()).to_pylist()
    assert limited_findings[0]["ordinal"] == 0
    assert limited_findings[0]["class"] == "resource_limit"
    assert limited_findings[0]["rule"] == "modeling.nonlinear.attempt_limit"
    del explanation, diagnostics, package, runtime
    durable = pa.table(explanation_table).to_pylist()[0]
    assert durable["complete"]
    assert len(durable["candidate_rows"]) == 2
    assert durable["attempts"][0]["observation"] == "local_obstruction"
    findings = {row["ordinal"]: row for row in pa.table(findings_table).to_pylist()}
    assert durable["attempts"][0]["failure_ordinal"] == 1
    assert findings[1]["rule"] == "modeling.qualification.rejected"
    assert findings[1]["sources"]
    assert findings[1]["observations"]
    # The L1 route builds no elastic model (ADR-0109 item 2a): the least-infeasible
    # point names the original rows it leaves violated, all among the candidate rows
    # (item 3).
    assert set(findings[1]["sources"]) <= set(durable["candidate_rows"])
    observations = {o["name"]: o["text"] for o in findings[1]["observations"]}
    assert observations["candidate_use"] == "diagnostic_only"
    assert len(attempts) == 4
    assert attempts[0].observation == "local_obstruction"
    assert attempts[0].failure() is not None
    assert attempts[0].penalty == pytest.approx(
        3.0, rel=settings.numerics.engineering_relative_fraction, abs=0.0
    )
    assert sum(a.observation == "feasible_witness" for a in attempts) == 2
    result = attempts[0].result()
    assert result is not None
    assert not result.accepted
    attempt = result.attempt()
    assert attempt is not None
    assert attempt.backend == "pounce"


@pytest.mark.component
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
    # Distinct budgets must be accepted even if another test already owns the process
    # Runtime.
    for threads in (1, 2):
        result = pse.ModelingConformance.pure(
            [documents],
            physical_documents,
            pse.EngineSettings(
                memory_limit_bytes=8 << 30,
                math_worker_bytes=8 << 20,
                math_workspace_bytes=8 << 20,
                threads=threads,
                spill_dir=str(tmp_path),
                max_spill_bytes=1 << 30,
                batch_size=1024,
            ),
            controls=pse.PureConformanceControls(maximum_checks=32),
        )
        assert result.passed
        assert result.complete
        inventory = pa.table(result.fixture_statuses())
        assert inventory.num_rows == 2
        assert inventory.column("status").to_pylist() == ["passed", "passed"]
        findings = pa.table(result.findings())
        assert findings.column("rule").to_pylist() == ["math.validity"]
        # Plan 23 H5: the finding carries the rejected predicate's typed lineage.
        (lineage,) = findings.column("validity").to_pylist()
        assert lineage["layer"] == "form"
        assert lineage["variables"] == [0]
        assert result.failure(0).boundary_class == "trial_rejected"
        table = result.table()
        del result
        assert pa.table(table).num_rows > 0
    limited = pse.ModelingConformance.pure(
        [documents],
        physical_documents,
        pse.EngineSettings(
            memory_limit_bytes=8 << 30,
            math_worker_bytes=8 << 20,
            math_workspace_bytes=8 << 20,
            threads=1,
            spill_dir=str(tmp_path),
            max_spill_bytes=1 << 30,
            batch_size=1024,
        ),
        controls=pse.PureConformanceControls(maximum_checks=1),
    )
    assert not limited.complete
    assert not limited.passed
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


@pytest.mark.integration
def test_knowledge_exports_admitted_values_and_bounds_projection(
    inspection_settings: pse.EngineSettings, canonical_substrate: str
) -> None:
    runtime = pse.Runtime(inspection_settings, substrate=canonical_substrate)
    root = Path(__file__).resolve().parents[3]
    manifest = on_primitives(
        (root / "tests/fixtures/packages/minimal_explicit/package.toml")
        .read_text()
        .replace('id_policy = "explicit"', 'id_policy = "named"')
    )
    source = """package knowledge {
      entity kind source provenance { attribute title:Text; }
      enum role { measured }
      entity source origin { title="synthetic inspection test" }
      entity kind sample { attribute value:Scalar; }
      entity sample a provenance(origin,role.measured) { value=2.5 ± relative(0.1) }
    }"""
    package = runtime.modeling_from_documents(
        [{"package.toml": manifest, "models/knowledge.pse": source}], physical(runtime)
    )
    sample = next(
        row.declaration_id for row in package.declarations() if row.name == "a"
    )
    knowledge = package.knowledge(
        sample, controls=pse.KnowledgeControls(maximum_cells=1)
    )
    row = pa.table(knowledge.table()).to_pylist()[0]
    assert row["owner_id"] == bytes(sample)
    assert row["value"][-1]["magnitude"] == 2.5
    assert row["value"][-1]["quantity_type_id"] is not None
    assert row["uncertainty"] == {"kind": "relative", "magnitude": 0.1}
    assert row["source_id"] is not None
    assert row["source_revision"] == bytes(knowledge.source_revision)
    assert row["slot"] == "value"
    assert row["test_only"] is False
    with pytest.raises(pse.InspectionError, match="maximum_bytes"):
        package.knowledge(sample, controls=pse.KnowledgeControls(maximum_bytes=128))
