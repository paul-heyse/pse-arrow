# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Command surface for native, data-authored model conformance.

One package runs ad hoc; ``--manifest`` runs every run a reference set declares (for
example ``packages/reference/conformance.toml``) once, under the manifest's run settings.
A fixture's execution policy is authored in the fixture itself; it is never
command-line data.
"""

import argparse
import re
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
    EngineSettings,
    ModelingConformance,
    ModelingLimits,
    Runtime,
    SolveControls,
    SolveSettings,
)
from pse.contracts.enums import NativeBackend, NativeSolveIntent, PresolvePolicyKind

#: The command line's word for automatic backend selection (no explicit backend).
AUTOMATIC = "auto"


class RunSettings(msgspec.Struct, frozen=True, forbid_unknown_fields=True, kw_only=True):
    """The execution settings of one conformance run, for every fixture it discovers.

    A fixture's declared execution policy replaces a setting for that fixture only.
    An absent backend selects automatically; an absent allowance is the kernel's.
    """

    memory_limit_bytes: int
    threads: int = 1
    time_limit_seconds: float = 600
    intent: NativeSolveIntent = NativeSolveIntent.ROOT
    backend: NativeBackend | None = None
    presolve: Literal["auto", "off"] = "auto"
    maximum_fixtures: int = 1024
    maximum_checks: int = 16384
    expansion_items: int | None = None
    expansion_members: int | None = None
    expansion_depth: int | None = None
    body_occurrences: int | None = None
    body_slots: int | None = None
    derivative_cells: int = 100000
    derivative_step: float = 1e-6
    derivative_tolerance: float = 1e-4


class ConformanceRun(msgspec.Struct, frozen=True, forbid_unknown_fields=True, kw_only=True):
    """One declared run: package roots relative to the manifest, and its execution.

    A pure run executes only explicit pure fixtures, without solver services.
    """

    name: str
    package: str
    physical: str
    dependencies: tuple[str, ...] = ()
    execution: Literal["native", "pure"] = "native"


class ConformanceManifest(msgspec.Struct, frozen=True, forbid_unknown_fields=True):
    """A reference set: shared run settings and every run, each executed once."""

    settings: RunSettings
    runs: tuple[ConformanceRun, ...]


class RunSummary(msgspec.Struct, frozen=True):
    """The one-line verdict of one run and its fixture inventory by status."""

    name: str
    passed: bool
    complete: bool
    checks: int
    statuses: dict[str, int]

    def line(self) -> str:
        counts = ", ".join(f"{k} {v}" for k, v in sorted(self.statuses.items()))
        return (
            f"{self.name}: passed={self.passed} complete={self.complete} "
            f"checks={self.checks} fixtures={sum(self.statuses.values())} ({counts})"
        )


def load_manifest(path: Path) -> ConformanceManifest:
    """Decode a manifest; run names are unique file stems and every root is a package."""
    manifest = msgspec.toml.decode(path.read_bytes(), type=ConformanceManifest)
    names = [run.name for run in manifest.runs]
    if (
        not names
        or len(set(names)) != len(names)
        or not all(re.fullmatch(r"[A-Za-z0-9_-]+", name) for name in names)
    ):
        message = f"{path}: runs must be nonempty and uniquely named by file stems"
        raise ValueError(message)
    for run in manifest.runs:
        for root in (run.package, run.physical, *run.dependencies):
            if not (path.parent / root / "package.toml").is_file():
                message = f"{path}: run {run.name} names {root}, which has no package.toml"
                raise ValueError(message)
    return manifest


def _documents(root: Path) -> dict[str, str]:
    if not (root / "package.toml").is_file():
        message = f"missing package.toml in {root}"
        raise ValueError(message)
    return {
        path.relative_to(root).as_posix(): path.read_text()
        for path in root.rglob("*")
        if path.is_file() and path.suffix in {".toml", ".yaml", ".yml", ".pse"}
    }


def _write(report: Path, result: ModelingConformance) -> None:
    """Write the checks, fixture inventory and findings beside each other."""
    for path, payload in (
        (report, pa.table(result.table())),
        (report.with_suffix(".fixtures.arrow"), pa.table(result.fixture_statuses())),
        (report.with_suffix(".findings.arrow"), pa.table(result.findings())),
    ):
        with (
            pa.OSFile(str(path), "wb") as destination,
            pa.ipc.new_file(destination, payload.schema) as writer,
        ):
            writer.write_table(payload)


def run_once(  # noqa: PLR0913 - one run's name, roots, execution, settings and report
    name: str,
    package: Path,
    dependencies: Sequence[Path],
    physical: Path,
    *,
    pure: bool,
    settings: RunSettings,
    report: Path | None,
) -> RunSummary:
    """Run every fixture of a package and its dependencies once and summarize it."""
    limits = ModelingLimits(
        items=settings.expansion_items,
        members=settings.expansion_members,
        depth=settings.expansion_depth,
        body_occurrences=settings.body_occurrences,
        body_slots=settings.body_slots,
    )
    with TemporaryDirectory(prefix="pse-conformance-") as spill:
        engine = EngineSettings(
            memory_limit_bytes=settings.memory_limit_bytes,
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
                maximum_fixtures=settings.maximum_fixtures,
                maximum_checks=settings.maximum_checks,
                limits=limits,
            )
        else:
            runtime = Runtime(engine)
            modeling = runtime.modeling_from_documents(
                documents, runtime.physical_from_documents(physical_documents)
            ).with_limits(limits)
            result = modeling.conform(
                SolveSettings(
                    intent=settings.intent,
                    backend=settings.backend,
                    presolve=PresolvePolicyKind(settings.presolve),
                    controls=SolveControls(time_limit=settings.time_limit_seconds),
                ),
                maximum_fixtures=settings.maximum_fixtures,
                maximum_checks=settings.maximum_checks,
                derivative_cells=settings.derivative_cells,
                derivative_step=settings.derivative_step,
                derivative_tolerance=settings.derivative_tolerance,
            )
        if report is not None:
            _write(report, result)
        statuses = pa.table(result.fixture_statuses()).column("status").to_pylist()
        return RunSummary(
            name=name,
            passed=result.passed,
            complete=result.complete,
            checks=pa.table(result.table()).num_rows,
            statuses=dict(Counter(str(s) for s in statuses)),
        )


def run_manifest(path: Path, report_dir: Path | None) -> tuple[RunSummary, ...]:
    """Run every declared run once, in order, writing its reports as ``<name>.arrow``."""
    manifest = load_manifest(path)
    if report_dir is not None:
        report_dir.mkdir(parents=True, exist_ok=True)
    summaries = []
    for run in manifest.runs:
        summary = run_once(
            run.name,
            path.parent / run.package,
            [path.parent / d for d in run.dependencies],
            path.parent / run.physical,
            pure=run.execution == "pure",
            settings=manifest.settings,
            report=None if report_dir is None else report_dir / f"{run.name}.arrow",
        )
        # One verdict line per run is the command's standard output.
        sys.stdout.write(summary.line() + "\n")
        sys.stdout.flush()
        summaries.append(summary)
    return tuple(summaries)


def main(argv: Sequence[str] | None = None) -> int:
    """Run one package or a manifest's runs; exit 0 only if every run passed completely."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("package", type=Path, nargs="?")
    parser.add_argument(
        "--manifest", type=Path, help="run every run a reference set declares"
    )
    parser.add_argument("--report-dir", type=Path)
    parser.add_argument("--physical", type=Path)
    parser.add_argument("--dependency", type=Path, action="append", default=[])
    parser.add_argument("--memory-limit-bytes", type=int)
    parser.add_argument("--threads", type=int, default=1)
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
    parser.add_argument("--intent", choices=("root", "optimize"), default="root")
    parser.add_argument("--presolve", choices=("auto", "off"), default="auto")
    parser.add_argument("--time-limit", type=float, default=600)
    parser.add_argument("--maximum-fixtures", type=int, default=1024)
    parser.add_argument("--maximum-checks", type=int, default=16384)
    parser.add_argument("--derivative-cells", type=int, default=100000)
    parser.add_argument("--derivative-step", type=float, default=1e-6)
    parser.add_argument("--derivative-tolerance", type=float, default=1e-4)
    parser.add_argument("--body-occurrences", type=int)
    parser.add_argument("--body-slots", type=int)
    parser.add_argument("--expansion-items", type=int)
    parser.add_argument("--expansion-members", type=int)
    parser.add_argument("--expansion-depth", type=int)
    parser.add_argument("--report", type=Path)
    args = parser.parse_args(argv)
    if args.manifest is not None:
        if args.package is not None or args.report is not None:
            parser.error("--manifest runs its declared packages into --report-dir")
        summaries = run_manifest(args.manifest, args.report_dir)
        return 0 if all(s.passed and s.complete for s in summaries) else 1
    if args.package is None or args.physical is None or args.memory_limit_bytes is None:
        parser.error("a package run needs a package, --physical and --memory-limit-bytes")
    settings = RunSettings(
        memory_limit_bytes=args.memory_limit_bytes,
        threads=args.threads,
        time_limit_seconds=args.time_limit,
        intent=NativeSolveIntent(args.intent),
        backend=None if args.backend == AUTOMATIC else NativeBackend(args.backend),
        presolve=args.presolve,
        maximum_fixtures=args.maximum_fixtures,
        maximum_checks=args.maximum_checks,
        expansion_items=args.expansion_items,
        expansion_members=args.expansion_members,
        expansion_depth=args.expansion_depth,
        body_occurrences=args.body_occurrences,
        body_slots=args.body_slots,
        derivative_cells=args.derivative_cells,
        derivative_step=args.derivative_step,
        derivative_tolerance=args.derivative_tolerance,
    )
    summary = run_once(
        args.package.name,
        args.package,
        args.dependency,
        args.physical,
        pure=args.pure,
        settings=settings,
        report=args.report,
    )
    # The command's one-line verdict is its standard output.
    sys.stdout.write(summary.line() + "\n")
    return 0 if summary.passed and summary.complete else 1


if __name__ == "__main__":
    raise SystemExit(main())
