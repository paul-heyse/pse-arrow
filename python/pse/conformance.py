# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Command surface for native, data-authored model conformance."""

import argparse
from collections.abc import Sequence
from pathlib import Path
from tempfile import TemporaryDirectory

import pyarrow as pa
import pyarrow.ipc

from pse import (
    EngineSettings,
    ModelingConformance,
    ModelingFixturePolicy,
    ModelingLimits,
    Runtime,
    SolveSettings,
)
from pse.contracts.values import SemanticId

#: The command line's word for automatic backend selection (no explicit backend).
AUTOMATIC = "auto"


def _selected(backend: str) -> str | None:
    """Map the command-line backend word to an explicit registry backend or none."""
    return None if backend == AUTOMATIC else backend


def _documents(root: Path) -> dict[str, str]:
    if not (root / "package.toml").is_file():
        raise ValueError(f"missing package.toml in {root}")
    return {
        path.relative_to(root).as_posix(): path.read_text()
        for path in root.rglob("*")
        if path.is_file() and path.suffix in {".toml", ".yaml", ".yml", ".pse"}
    }


def main(argv: Sequence[str] | None = None) -> int:
    """Run the Rust harness and optionally retain its generated Arrow report."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("package", type=Path)
    parser.add_argument("--physical", type=Path, required=True)
    parser.add_argument("--dependency", type=Path, action="append", default=[])
    parser.add_argument("--memory-limit-bytes", type=int, required=True)
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
    parser.add_argument(
        "--fixture-solver",
        nargs=3,
        action="append",
        default=[],
        metavar=("ID", "INTENT", "BACKEND"),
    )
    parser.add_argument(
        "--fixture-presolve",
        nargs=2,
        action="append",
        default=[],
        metavar=("ID", "MODE"),
    )
    parser.add_argument(
        "--fixture-derivatives",
        nargs=4,
        action="append",
        default=[],
        metavar=("ID", "STEP", "TOLERANCE", "CELLS"),
    )
    parser.add_argument("--maximum-fixtures", type=int, default=1024)
    parser.add_argument("--maximum-checks", type=int, default=16384)
    parser.add_argument("--derivative-cells", type=int, default=100000)
    parser.add_argument("--body-occurrences", type=int)
    parser.add_argument("--expansion-items", type=int)
    parser.add_argument("--expansion-members", type=int)
    parser.add_argument("--expansion-depth", type=int)
    parser.add_argument("--report", type=Path)
    args = parser.parse_args(argv)
    selections: dict[SemanticId, tuple[str, str]] = {}
    presolves: dict[SemanticId, str] = {}
    derivatives: dict[SemanticId, tuple[float, float, int]] = {}
    for key, intent, backend in args.fixture_solver:
        fixture = SemanticId.from_hex(key)
        if fixture in selections:
            parser.error("duplicate fixture solver policy")
        selections[fixture] = (intent, backend)
    for key, mode in args.fixture_presolve:
        fixture = SemanticId.from_hex(key)
        if fixture in presolves:
            parser.error("duplicate fixture presolve policy")
        if mode not in {"auto", "off"}:
            parser.error("fixture presolve must be auto or off")
        presolves[fixture] = mode
    solvers = {
        fixture: SolveSettings(
            intent=selections.get(fixture, (args.intent, args.backend))[0],
            backend=_selected(selections.get(fixture, (args.intent, args.backend))[1]),
            presolve=presolves.get(fixture, args.presolve),
            time_limit=args.time_limit,
        )
        for fixture in selections.keys() | presolves.keys()
    }
    for key, step, tolerance, cells in args.fixture_derivatives:
        fixture = SemanticId.from_hex(key)
        if fixture in derivatives:
            parser.error("duplicate fixture derivative policy")
        derivatives[fixture] = (float(step), float(tolerance), int(cells))
    if args.pure and (solvers or derivatives):
        parser.error("pure conformance does not consume native fixture policies")
    fixture_policies = {
        fixture: ModelingFixturePolicy(
            solvers.get(fixture),
            derivative_step=derivatives[fixture][0] if fixture in derivatives else None,
            derivative_tolerance=derivatives[fixture][1]
            if fixture in derivatives
            else None,
            derivative_cells=derivatives[fixture][2]
            if fixture in derivatives
            else None,
        )
        for fixture in solvers.keys() | derivatives.keys()
    }
    limits = ModelingLimits(
        items=args.expansion_items,
        members=args.expansion_members,
        depth=args.expansion_depth,
        body_occurrences=args.body_occurrences,
    )
    with TemporaryDirectory(prefix="pse-conformance-") as spill:
        settings = EngineSettings(
            memory_limit_bytes=args.memory_limit_bytes,
            threads=args.threads,
            spill_dir=spill,
            max_spill_bytes=1 << 30,
            batch_size=1024,
        )
        documents = [
            _documents(args.package),
            *(_documents(path) for path in args.dependency),
        ]
        physical_documents = _documents(args.physical)
        if args.pure:
            result = ModelingConformance.pure(
                documents,
                physical_documents,
                settings,
                maximum_fixtures=args.maximum_fixtures,
                maximum_checks=args.maximum_checks,
                limits=limits,
            )
        else:
            runtime = Runtime(settings)
            physical = runtime.physical_from_documents(physical_documents)
            package = runtime.modeling_from_documents(documents, physical).with_limits(
                limits
            )
            result = package.conform(
                SolveSettings(
                    backend=_selected(args.backend),
                    intent=args.intent,
                    presolve=args.presolve,
                    time_limit=args.time_limit,
                ),
                maximum_fixtures=args.maximum_fixtures,
                maximum_checks=args.maximum_checks,
                derivative_cells=args.derivative_cells,
                fixture_policies=fixture_policies,
            )
        table = pa.table(result.table())
        fixtures = pa.table(result.fixture_statuses())
        if args.report is not None:
            for path, payload in (
                (args.report, table),
                (args.report.with_suffix(".fixtures.arrow"), fixtures),
                (
                    args.report.with_suffix(".findings.arrow"),
                    pa.table(result.findings()),
                ),
            ):
                with pa.OSFile(str(path), "wb") as destination:
                    with pa.ipc.new_file(destination, payload.schema) as writer:
                        writer.write_table(payload)
        print(
            f"passed={result.passed} complete={result.complete} checks={table.num_rows} fixtures={fixtures.num_rows}"
        )
        return 0 if result.passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
