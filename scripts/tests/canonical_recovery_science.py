# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Public-API scientific child for the actual canonical maintenance journey.

The explicit authored and physical documents are the reconstruction authority.
Receipts identify retained sources/results; they never supply numerical inputs.
"""

from __future__ import annotations

import argparse
import math
import shutil
import sys
import traceback
from pathlib import Path
from typing import TYPE_CHECKING

from scripts import surreal_server as server

if TYPE_CHECKING:
    import pyarrow as pa

SOURCE = """package recovery { def LengthBalance {
    var x:Length;
    eq length:x==2{m};
    annotation start x(1{m});
    annotation bounds x(0{m},10{m});
    annotation report x("length");
    annotation check x(x>1{m});
} }"""
PRIMITIVES = (
    'dependencies = [{ package_id = "5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a", '
    'version_req = { operator = "exact", major = 1, minor = 0, patch = 0 } }]'
)


def prepare_inputs(destination: Path) -> dict[str, str]:
    root = Path(__file__).resolve().parents[2]
    fixtures = root / "tests/fixtures/packages"
    shutil.copytree(fixtures / "physical-primitives", destination / "physical")
    model = destination / "model"
    (model / "models").mkdir(parents=True)
    manifest = (fixtures / "minimal_named/package.toml").read_text()
    (model / "package.toml").write_text(
        manifest.replace("dependencies = []", PRIMITIVES)
    )
    (model / "models/length.pse").write_text(SOURCE)
    return server.input_inventory(destination)


def documents(directory: Path) -> dict[str, str]:
    return {
        str(path.relative_to(directory)): path.read_text()
        for path in sorted(directory.rglob("*"))
        if path.is_file()
    }


def check_solution(
    usable: bool,
    backend: str | None,
    values: list[float],
    satisfied: list[bool],
    relative: float,
) -> None:
    if (
        not usable
        or backend != "ipopt"
        or len(values) != 1
        or not math.isfinite(values[0])
        or abs(values[0] - 2.0) > 2.0 * relative
        or not satisfied
        or not all(satisfied)
    ):
        raise RuntimeError("Original dimensional/native scientific acceptance failed")


def result_snapshot(
    manifest: pa.Table, variables: pa.Table, checks: pa.Table
) -> dict[str, str]:
    """Retain complete current typed rows, including exact float and binary bits.

    IPC retains the complete schema and values without JSON's float or binary
    conversions. Comparison normalizes transport chunks and metadata map order.
    """
    snapshot: dict[str, str] = {}
    (header,) = manifest.to_pylist()
    snapshot["manifest_digest"] = header["digest"]
    for name, table in (
        ("manifest", manifest),
        ("solve_variables", variables),
        ("modeling_checks", checks),
    ):
        snapshot[name] = _table_ipc(table, table.schema).hex()
    return snapshot


def _table_ipc(table: pa.Table, schema: pa.Schema) -> bytes:
    import pyarrow as pa  # noqa: PLC0415

    sink = pa.BufferOutputStream()
    with pa.ipc.new_stream(sink, schema) as writer:
        writer.write_table(table.combine_chunks())
    return sink.getvalue().to_pybytes()


def check_retained_snapshot(saved: object, current: dict[str, str]) -> None:
    import pyarrow as pa  # noqa: PLC0415

    failure = "Acknowledged scientific manifest or retained result contents changed"
    if (
        not isinstance(saved, dict)
        or saved.keys() != current.keys()
        or saved.get("manifest_digest") != current["manifest_digest"]
    ):
        raise RuntimeError(failure)
    for name in ("manifest", "solve_variables", "modeling_checks"):
        previous = saved.get(name)
        if not isinstance(previous, str):
            raise TypeError(failure)
        original = pa.ipc.open_stream(bytes.fromhex(previous)).read_all()
        observed = pa.ipc.open_stream(bytes.fromhex(current[name])).read_all()
        # Arrow compares metadata maps by content, including nested fields. Only
        # after that full schema check may both encodings use the same schema.
        # This preserves exact float/binary bits without treating metadata map
        # insertion order or transport chunking as scientific changes.
        if not original.schema.equals(observed.schema, check_metadata=True) or (
            _table_ipc(original, original.schema)
            != _table_ipc(observed, original.schema)
        ):
            raise RuntimeError(failure)


def run(state: Path, inputs: Path, receipt: Path, phase: str) -> None:
    # Keep the pure orchestration/unit controls independent of the installed extension.
    import pyarrow as pa  # noqa: PLC0415

    import pse  # noqa: PLC0415
    from pse.contracts.enums import (  # noqa: PLC0415
        NativeBackend,
        NativeSolveIntent,
        PresolvePolicyKind,
    )

    config = server.config_for(state)
    allocation = server.object_mapping(config["resources"])
    execution = allocation.get("execution")
    if execution is None:
        raise server.SupervisorError(
            "Scientific recovery requires the original plan28-reference profile"
        )
    if allocation != server.reference_resources():
        raise server.SupervisorError("Scientific recovery allocation changed")
    execution = server.object_mapping(execution)
    spill = inputs.parent / "science-spill"
    spill.mkdir(parents=True, exist_ok=True)
    settings = pse.EngineSettings(
        memory_limit_bytes=server.integer(execution["pool_memory_bytes"]),
        threads=server.integer(execution["cpu_threads"]),
        target_partitions=server.integer(execution["cpu_threads"]),
        spill_dir=str(spill),
        max_spill_bytes=1 << 30,
        batch_size=7,
        math_worker_bytes=server.integer(execution["worker_bytes"]),
        math_jobs=server.integer(execution["math_jobs"]),
        math_admission_wait_ms=server.integer(execution["admission_wait_ms"]),
    )
    if phase == "rebuild":
        pse.Runtime.initialize_database(
            settings, substrate=str(state), database=str(config["database"])
        )
    runtime = pse.Runtime(settings, substrate=str(state))
    physical = package = result = variable_table = check_table = None
    try:
        physical = runtime.physical_from_documents(documents(inputs / "physical"))
        if phase in {"seed", "rebuild"}:
            package = runtime.modeling_from_documents(
                [documents(inputs / "model")], physical
            )
        else:
            saved = server.read_json(receipt)
            package = runtime.modeling_revision(str(saved["revision"]), physical)
        case = next(
            row.declaration_id
            for row in package.declarations()
            if row.name == "LengthBalance"
        )
        identity = {
            "revision": package.canonical_revision,
            "problem": package.canonical_problem,
            "physical": physical.identity.to_prefixed(),
            "source": package.knowledge().source_revision.to_prefixed(),
            "case": case.to_hex(),
            "interpretation": server.SUBSTRATE_INTERPRETATION,
            "inputs": server.input_inventory(inputs),
        }
        if phase != "seed":
            saved = server.read_json(receipt)
            for key, value in identity.items():
                if phase == "rebuild" and key in {"revision", "problem"}:
                    continue
                if saved[key] != value:
                    raise RuntimeError("Scientific recovery identity changed: " + key)
            if phase != "rebuild":
                (record,) = pa.table(
                    runtime.attempt_record(str(saved["attempt"]))
                ).to_pylist()
                if record["run"] != saved["run"] or record["outcome"] != "succeeded":
                    raise RuntimeError("Acknowledged scientific attempt changed")
                check_retained_snapshot(
                    saved["results"],
                    result_snapshot(
                        pa.table(runtime.result_manifest(str(saved["attempt"]))),
                        pa.table(
                            runtime.results(
                                str(saved["run"]),
                                str(saved["attempt"]),
                                "runtime.solve_variables",
                            )
                        ),
                        pa.table(
                            runtime.results(
                                str(saved["run"]),
                                str(saved["attempt"]),
                                "runtime.modeling_checks",
                            )
                        ),
                    ),
                )
        # Fresh solve from the reopened/rebuilt source, never a recorded-value replay.
        solve = pse.SolveSettings(
            backend=NativeBackend.IPOPT,
            intent=NativeSolveIntent.FEASIBLE_POINT,
            presolve=PresolvePolicyKind.OFF,
        )
        result = package.prepare_solve(case, solve).start().wait()
        variable_table = pa.table(result.table("runtime.solve_variables"))
        check_table = pa.table(result.table("runtime.modeling_checks"))
        variables = variable_table.to_pylist()
        checks = check_table.to_pylist()
        check_solution(
            result.usable,
            result.completion.solves[0].backend,
            [row["value"] for row in variables],
            [row["satisfied"] for row in checks],
            solve.numerics.engineering_relative_fraction,
        )
        if result.canonical_run_key is None or result.canonical_attempt_key is None:
            raise RuntimeError(
                "Scientific execution did not acknowledge durable results"
            )
        if phase in {"seed", "rebuild"}:
            snapshot = result_snapshot(
                pa.table(runtime.result_manifest(result.canonical_attempt_key)),
                pa.table(
                    runtime.results(
                        result.canonical_run_key,
                        result.canonical_attempt_key,
                        "runtime.solve_variables",
                    )
                ),
                pa.table(
                    runtime.results(
                        result.canonical_run_key,
                        result.canonical_attempt_key,
                        "runtime.modeling_checks",
                    )
                ),
            )
            server.write_json(
                receipt if phase == "seed" else receipt.with_suffix(".rebuilt.json"),
                {
                    **identity,
                    "run": result.canonical_run_key,
                    "attempt": result.canonical_attempt_key,
                    "results": snapshot,
                },
            )
        print("scientific public-API " + phase + " passed", flush=True)
    finally:
        primary_error = sys.exception()
        # Depart actual local borrowers before evicting optional source/program
        # owners. A partially constructed journey follows the same drain path.
        variable_table = check_table = None
        result = None
        package = None
        physical = None
        try:
            runtime.clear_program_cache()
            runtime.close()
        except BaseException as cleanup_error:
            if primary_error is None:
                raise
            primary_error.add_note(
                f"Scientific recovery cleanup also failed: {cleanup_error}"
            )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--state", type=Path, required=True)
    parser.add_argument("--inputs", type=Path, required=True)
    parser.add_argument("--receipt", type=Path, required=True)
    parser.add_argument(
        "--phase",
        choices=("seed", "reopen", "restored", "rebuild", "verify-rebuild"),
        required=True,
    )
    args = parser.parse_args()
    try:
        run(args.state, args.inputs, args.receipt, args.phase)
    except Exception:
        # The maintenance owner suppresses initializer output. Keep its concrete
        # cause beside the private identity receipts for the integrating operator.
        (args.receipt.parent / f"science-{args.phase}-error.txt").write_text(
            traceback.format_exc()
        )
        raise


if __name__ == "__main__":
    main()
