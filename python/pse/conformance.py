# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Command surface for native, data-authored model conformance.

One package runs ad hoc; ``--manifest`` runs every run a reference set declares (for
example ``packages/reference/conformance.toml``) once, under its run settings.
A fixture's execution policy is authored in the fixture itself; it is never
command-line data. Repeated ``--fixture <declaration-id>`` selects test declarations by
identity: only those run, an identity that names no authored test is refused before any
fixture runs, and the run reports ``coverage=selected`` rather than package coverage.
"""

import argparse
import os
import sys
from collections import Counter
from collections.abc import Sequence
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Literal

import msgspec
import pyarrow as pa
import pyarrow.ipc

from pse import (
    ConformanceControls,
    EngineSettings,
    ModelingConformance,
    ModelingDiagnosticPolicy,
    ModelingDiagnosticSettings,
    ModelingLimits,
    PureConformanceControls,
    Runtime,
    SolveControls,
    SolveSettings,
    codec,
)
from pse._build import _admit_conformance_manifest
from pse.contracts.documents import (
    ConformanceManifest,
    ConformanceManifestOutput,
    ModelingDiagnosticPolicyBody,
    PreparationSettings,
    PreparationSettingsBody,
    PreparationSettingsBodyOutput,
    PreparationSettingsOutput,
    RunSettings,
    RunSettingsOutput,
)
from pse.contracts.enums import (
    ConformanceExecution,
    ConformancePresolve,
    NativeBackend,
    NativeSolveIntent,
    PresolvePolicyKind,
)
from pse.contracts.identities import DeclarationId
from pse.contracts.values import SemanticId

#: The command line's word for automatic backend selection (no explicit backend).
AUTOMATIC = "auto"


class RunSummary(msgspec.Struct, frozen=True):
    """The one-line verdict of one run and its fixture inventory by status.

    ``coverage`` is ``package`` when every authored test ran and package coverage was
    assessed, and ``selected`` when a fixture selection ran, which assesses none.
    """

    name: str
    passed: bool
    complete: bool
    coverage: Literal["package", "selected"]
    checks: int
    statuses: dict[str, int]

    def line(self) -> str:
        """Render the bounded run summary."""
        counts = ", ".join(f"{k} {v}" for k, v in sorted(self.statuses.items()))
        return (
            f"{self.name}: passed={self.passed} complete={self.complete} "
            f"coverage={self.coverage} checks={self.checks} "
            f"fixtures={sum(self.statuses.values())} ({counts})"
        )


def load_manifest(path: Path) -> ConformanceManifestOutput:
    """Read generated TOML settings and admit the campaign through its Rust owner."""
    manifest = codec.decode_toml(path.read_bytes(), ConformanceManifest)
    return _admit_conformance_manifest(str(path), manifest)


def fixture_id(text: str) -> DeclarationId:
    """A fixture selection names a test declaration by its identity."""
    return DeclarationId(SemanticId.from_hex(text))


def _preparation_document(
    body: PreparationSettingsBody | PreparationSettingsBodyOutput | None,
) -> PreparationSettings | PreparationSettingsOutput | None:
    if body is None:
        return None
    if isinstance(body, PreparationSettingsBodyOutput):
        return PreparationSettingsOutput(compiler=body.compiler, limits=body.limits)
    return PreparationSettings(compiler=body.compiler, limits=body.limits)


def _diagnostic_policy(path: str) -> ModelingDiagnosticPolicyBody:
    source = Path(path).read_text(encoding="utf-8")
    # Rust retains threshold/budget validation; project the original authored fields.
    ModelingDiagnosticSettings.from_json(source)
    policy = codec.decode_json(source, ModelingDiagnosticPolicy)
    return ModelingDiagnosticPolicyBody(
        bound_violation=policy.bound_violation,
        jacobian_large=policy.jacobian_large,
        jacobian_small=policy.jacobian_small,
        matrix=policy.matrix,
        maximum_findings=policy.maximum_findings,
        near_bound_absolute=policy.near_bound_absolute,
        near_bound_relative=policy.near_bound_relative,
        profile=policy.profile,
        residual=policy.residual,
        terms=policy.terms,
        variable_large=policy.variable_large,
        variable_small=policy.variable_small,
        variable_zero=policy.variable_zero,
    )


def _documents(root: Path) -> dict[str, str | bytes]:
    if not (root / "package.toml").is_file():
        message = f"missing package.toml in {root}"
        raise ValueError(message)
    return {
        path.relative_to(root).as_posix(): path.read_bytes()
        for path in root.rglob("*")
        if path.is_file()
        and path.suffix in {".toml", ".yaml", ".yml", ".pse", ".parquet"}
    }


def _write(report: Path, result: ModelingConformance) -> None:
    """Write checks, fixture inventory, admission facts, findings and oracle parity."""
    for path, payload in (
        (report, pa.table(result.table())),
        (report.with_suffix(".fixtures.arrow"), pa.table(result.fixture_statuses())),
        (report.with_suffix(".findings.arrow"), pa.table(result.findings())),
        (report.with_suffix(".parity.arrow"), pa.table(result.parity())),
        (
            report.with_suffix(".routes.arrow"),
            pa.table(result.admission("runtime.route_decisions")),
        ),
        (
            report.with_suffix(".structure.arrow"),
            pa.table(result.admission("runtime.structural_assessments")),
        ),
    ):
        with (
            pa.OSFile(str(path), "wb") as destination,
            pa.ipc.new_file(destination, payload.schema) as writer,
        ):
            writer.write_table(payload)


def run_once(
    name: str,
    package: Path,
    dependencies: Sequence[Path],
    physical: Path,
    *,
    pure: bool,
    settings: RunSettings | RunSettingsOutput,
    report: Path | None,
    fixtures: Sequence[DeclarationId] | None = None,
) -> RunSummary:
    """Run package fixtures or the selected identities and summarize the result.

    Dependencies participate in package coverage.
    """
    limits = (
        ModelingLimits(
            items=settings.expansion_items,
            members=settings.expansion_members,
            depth=settings.expansion_depth,
            body_occurrences=settings.body_occurrences,
            body_slots=settings.body_slots,
        )
        if settings.preparation is None
        else None
    )
    with TemporaryDirectory(prefix="pse-conformance-") as spill:
        engine = EngineSettings(
            memory_limit_bytes=settings.memory_limit_bytes,
            math_worker_bytes=settings.math_worker_bytes,
            math_jobs=settings.math_jobs,
            math_admission_wait_ms=settings.math_admission_wait_ms,
            math_preparation_limit_ms=settings.math_preparation_limit_ms,
            threads=settings.threads,
            spill_dir=spill,
            max_spill_bytes=1 << 30,
            batch_size=1024,
        )
        documents = [_documents(package), *(_documents(p) for p in dependencies)]
        physical_documents = _documents(physical)
        if pure:
            result = ModelingConformance.pure(
                documents,
                physical_documents,
                engine,
                controls=PureConformanceControls(
                    maximum_fixtures=settings.maximum_fixtures,
                    maximum_checks=settings.maximum_checks,
                    fixtures=None if fixtures is None else tuple(fixtures),
                ),
                limits=limits,
                preparation=_preparation_document(settings.preparation),
            )
        else:
            runtime = Runtime(engine, substrate=os.environ["PSE_SURREAL_STATE"])
            modeling = runtime.modeling_from_documents(
                documents, runtime.physical_from_documents(physical_documents)
            )
            if limits is not None:
                modeling = modeling.with_limits(limits)
            result = modeling.conform(
                SolveSettings(
                    intent=settings.intent,
                    backend=settings.backend,
                    presolve=PresolvePolicyKind(settings.presolve.value),
                    controls=SolveControls(time_limit=settings.time_limit_seconds),
                ),
                controls=ConformanceControls(
                    maximum_fixtures=settings.maximum_fixtures,
                    maximum_checks=settings.maximum_checks,
                    derivative_cells=settings.derivative_cells,
                    derivative_step=settings.derivative_step,
                    derivative_tolerance=settings.derivative_tolerance,
                    fixtures=None if fixtures is None else tuple(fixtures),
                    diagnostics=(
                        None
                        if settings.diagnostics is None
                        else _diagnostic_policy(settings.diagnostics)
                    ),
                ),
                preparation=_preparation_document(settings.preparation),
            )
        if report is not None:
            _write(report, result)
        statuses = pa.table(result.fixture_statuses()).column("status").to_pylist()
        return RunSummary(
            name=name,
            passed=result.passed,
            complete=result.complete,
            coverage="selected" if result.selected else "package",
            checks=pa.table(result.table()).num_rows,
            statuses=dict(Counter(str(s) for s in statuses)),
        )


def run_manifest(
    path: Path,
    report_dir: Path | None,
    fixtures: Sequence[DeclarationId] | None = None,
    run_name: str | None = None,
) -> tuple[RunSummary, ...]:
    """Run every declared run once, in order, writing its reports as ``<name>.arrow``.

    ``run_name`` selects one declared run. A fixture selection needs exactly one run,
    so an identity outside that run is refused before any fixture runs.
    """
    manifest = load_manifest(path)
    runs = manifest.runs
    if run_name is not None:
        runs = tuple(run for run in runs if run.name == run_name)
        if not runs:
            message = f"{path}: unknown run {run_name!r}"
            raise ValueError(message)
    if fixtures is not None and len(runs) != 1:
        message = f"{path}: a fixture selection needs a manifest that declares one run"
        raise ValueError(message)
    if report_dir is not None:
        report_dir.mkdir(parents=True, exist_ok=True)
    summaries = []
    for run in runs:
        summary = run_once(
            run.name,
            path.parent / run.package,
            [path.parent / d for d in run.dependencies],
            path.parent / run.physical,
            pure=run.execution is ConformanceExecution.PURE,
            settings=manifest.settings,
            report=None if report_dir is None else report_dir / f"{run.name}.arrow",
            fixtures=fixtures,
        )
        # One verdict line per run is the command's standard output.
        sys.stdout.write(summary.line() + "\n")
        sys.stdout.flush()
        summaries.append(summary)
    return tuple(summaries)


def main(argv: Sequence[str] | None = None) -> int:
    """Run package conformance; exit zero only when every run passed completely."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("package", type=Path, nargs="?")
    parser.add_argument(
        "--manifest", type=Path, help="run every run a reference set declares"
    )
    parser.add_argument("--report-dir", type=Path)
    parser.add_argument("--run", help="select one named run from --manifest")
    parser.add_argument("--physical", type=Path)
    parser.add_argument("--dependency", type=Path, action="append", default=[])
    parser.add_argument("--memory-limit-bytes", type=int)
    parser.add_argument("--threads", type=int)
    parser.add_argument(
        "--pure",
        action="store_true",
        help="execute explicit pure fixtures without workflow or solver services",
    )
    parser.add_argument(
        "--backend",
        default=AUTOMATIC,
        help=f"registry backend name, or {AUTOMATIC} for automatic selection",
    )
    parser.add_argument("--intent", choices=("root", "optimize"))
    parser.add_argument(
        "--presolve", choices=tuple(choice.value for choice in ConformancePresolve)
    )
    parser.add_argument("--time-limit", type=float)
    parser.add_argument("--maximum-fixtures", type=int)
    parser.add_argument("--maximum-checks", type=int)
    parser.add_argument("--derivative-cells", type=int)
    parser.add_argument("--derivative-step", type=float)
    parser.add_argument("--derivative-tolerance", type=float)
    parser.add_argument("--body-occurrences", type=int)
    parser.add_argument("--body-slots", type=int)
    parser.add_argument("--expansion-items", type=int)
    parser.add_argument("--expansion-members", type=int)
    parser.add_argument("--expansion-depth", type=int)
    parser.add_argument("--report", type=Path)
    parser.add_argument(
        "--diagnostics",
        type=Path,
        help="numerical diagnostic threshold profile (JSON) for expected findings",
    )
    parser.add_argument(
        "--fixture",
        type=fixture_id,
        action="append",
        help="run only this test declaration (hexadecimal identity); repeatable",
    )
    args = parser.parse_args(argv)
    fixtures: list[DeclarationId] | None = args.fixture
    if args.manifest is not None:
        if args.package is not None or args.report is not None:
            parser.error("--manifest runs its declared packages into --report-dir")
        summaries = run_manifest(args.manifest, args.report_dir, fixtures, args.run)
        return 0 if all(s.passed and s.complete for s in summaries) else 1
    if args.run is not None:
        parser.error("--run requires --manifest")
    if args.package is None or args.physical is None or args.memory_limit_bytes is None:
        parser.error(
            "a package run needs a package, --physical and --memory-limit-bytes"
        )
    defaults = RunSettings(memory_limit_bytes=args.memory_limit_bytes)
    supplied: dict[str, object] = {
        "threads": args.threads,
        "time_limit_seconds": args.time_limit,
        "intent": None if args.intent is None else NativeSolveIntent(args.intent).value,
        "backend": None
        if args.backend == AUTOMATIC
        else NativeBackend(args.backend).value,
        "presolve": args.presolve,
        "maximum_fixtures": args.maximum_fixtures,
        "maximum_checks": args.maximum_checks,
        "expansion_items": args.expansion_items,
        "expansion_members": args.expansion_members,
        "expansion_depth": args.expansion_depth,
        "body_occurrences": args.body_occurrences,
        "body_slots": args.body_slots,
        "derivative_cells": args.derivative_cells,
        "derivative_step": args.derivative_step,
        "derivative_tolerance": args.derivative_tolerance,
        "diagnostics": None if args.diagnostics is None else str(args.diagnostics),
    }
    values = codec.convert_document(
        codec.document_builtins(defaults), dict[str, object]
    )
    values.update({key: value for key, value in supplied.items() if value is not None})
    settings = codec.convert_document(values, RunSettings)
    summary = run_once(
        args.package.name,
        args.package,
        args.dependency,
        args.physical,
        pure=args.pure,
        settings=settings,
        report=args.report,
        fixtures=fixtures,
    )
    # The command's one-line verdict is its standard output.
    sys.stdout.write(summary.line() + "\n")
    return 0 if summary.passed and summary.complete else 1


if __name__ == "__main__":
    raise SystemExit(main())
