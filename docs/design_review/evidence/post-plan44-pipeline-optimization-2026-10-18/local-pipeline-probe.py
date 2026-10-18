# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Once-only public local preparation controls; no comparative speed claim."""

import gc
import json
import sys
import tempfile
import time
from pathlib import Path

import pyarrow as pa

import pse
from pse import codec
from pse._build import compiled_features


ROOT = Path(__file__).resolve().parents[4]
STATES = 8


def source(parameter: int = 1, changed: bool = False) -> str:
    declarations = []
    for index in range(STATES):
        factor = 2 if changed and index == 0 else 1
        declarations.append(
            f"var x{index}[i in t]:Time; "
            f"eq rate{index}[i in t]:d(x{index}[i])/di=={factor}*p; "
            f"eq initial{index}:x{index}[0{{s}}]=={index}{{s}}; "
            f'annotation report x{index}("state {index}"); '
            f"annotation check x{index}(abs(x{index}[i]-{index}{{s}}-"
            f"{factor}*p*i)<0.02{{s}});"
        )
    return (
        "package post44 { def D { domain t:Time from 0{s} to 2{s}; "
        "discretize grid on t using integrated(elements=1,order=1); "
        f"param p:Scalar={parameter}; "
        + " ".join(declarations)
        + " } case run fixture { dof 0; route integrated; procedure integrate; "
        "integrate samples(0{s},0.5{s},1{s},1.5{s},2{s}) relative(0.001) "
        "normalized_absolute(0.001) step(1e-4{s}); } { child root:D=D(); } }"
    )


def documents(text: str) -> dict[str, str]:
    manifest = (ROOT / "tests/fixtures/packages/minimal_explicit/package.toml").read_text()
    manifest = manifest.replace('id_policy = "explicit"', 'id_policy = "named"')
    manifest = manifest.replace(
        "dependencies = []",
        'dependencies = [{ package_id = "5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a", '
        'version_req = { operator = "exact", major = 1, minor = 0, patch = 0 } }]',
    )
    return {"package.toml": manifest, "models/post44.pse": text}


def emit(value: object) -> None:
    print(json.dumps(value), flush=True)


def main() -> None:
    features = compiled_features()
    assert "force-validate" in features and "native-solvers" in features, features
    emit({"kind": "scope", "features": features, "states": STATES,
          "rtol": 0.001, "sample_count": 5, "method": "diffsol",
          "comparison": "one observation per stage; source edits, not runtime overlays"})
    with tempfile.TemporaryDirectory(prefix="post44-local-spill-") as spill:
        settings = pse.EngineSettings(
            memory_limit_bytes=64 << 30, threads=2, target_partitions=1,
            spill_dir=spill, max_spill_bytes=1 << 30, batch_size=7,
            cache=pse.CacheSettings(working_bytes=16 << 30, metadata_bytes=8 << 20,
                                    concurrent_loads=2, inflight_bytes=64 << 20,
                                    inspection_bytes=4 << 20),
        )
        runtime = pse.Runtime(settings, ephemeral=True)
        diagnostics = runtime.observations()
        seen: set[int] = set()

        def observe(name: str) -> None:
            offset = 0
            while True:
                page = diagnostics.list(offset=offset, limit=128)
                emit({"kind": "operation_page", "after": name,
                      "value": json.loads(codec.encode_json(page))})
                for operation in page.operations:
                    if operation.id.sequence in seen:
                        continue
                    seen.add(operation.id.sequence)
                    reader = diagnostics.reader(operation.id)
                    if reader is not None:
                        emit({"kind": "operation", "after": name,
                              "value": json.loads(codec.encode_json(reader.snapshot(window=3600)))})
                if page.next_offset is None:
                    break
                offset = page.next_offset

        def stage(name: str, action):
            started = time.perf_counter()
            value = action()
            emit({"kind": "stage", "name": name,
                  "elapsed_seconds": time.perf_counter() - started})
            observe(name)
            return value

        try:
            primitives = ROOT / "tests/fixtures/packages/physical-primitives"
            physical_documents = {str(path.relative_to(primitives)): path.read_text()
                                  for path in primitives.rglob("*") if path.is_file()}
            physical = stage("physical_admit_first", lambda: runtime.physical_from_documents(physical_documents))
            physical_again = stage("physical_admit_same", lambda: runtime.physical_from_documents(physical_documents))
            model = stage("model_admit_first", lambda: runtime.modeling_from_documents([documents(source())], physical))
            case = next(row.declaration_id for row in model.declarations() if row.name == "run")
            simulation = pse.SimulationSettings(method="diffsol", start=0, end=2,
                samples=[0, 0.5, 1, 1.5, 2], atol=[0.001] * STATES, rtol=0.001,
                parameter_scales=[1], sensitivity="forward", time_limit=60)

            def check(prepared, name: str, parameter: int = 1, changed: bool = False):
                result = stage(name, lambda: prepared.start().wait())
                assert result.usable, result.diagnostics()
                rows = pa.table(result.table("runtime.simulation_samples")).to_pylist()
                assert len(rows) == STATES * 5, len(rows)
                # Every state has a distinct original offset. The authored check
                # verifies its own identity; independently compare the sorted values.
                for instant in [0, 0.5, 1, 1.5, 2]:
                    actual = sorted(row["value"] for row in rows if row["time"] == instant)
                    expected = sorted(index + (2 if changed and index == 0 else 1) * parameter * instant for index in range(STATES))
                    assert len(actual) == len(expected)
                    assert all(abs(left - right) < 0.02 for left, right in zip(actual, expected, strict=True)), (actual, expected)
                emit({"kind": "invocation", "name": name,
                      "evidence": json.loads(codec.encode_json(result.invocation_evidence)),
                      "summary": json.loads(codec.encode_json(result.attempt_summary))})
                return result

            prepared = stage("prepare_first", lambda: model.prepare_simulation(case, simulation))
            results = [check(prepared, "run_first"), check(prepared, "run_same_prepared")]
            prepared_again = stage("prepare_same", lambda: model.prepare_simulation(case, simulation))
            results.append(check(prepared_again, "run_equal_preparation"))
            unchanged = stage("reload_unchanged", lambda: model.reload_from_parent([documents(source())]))
            value_edit = stage("reload_parameter_default", lambda: unchanged.reload_from_parent([documents(source(2))]))
            value_prepared = stage("prepare_parameter_default", lambda: value_edit.prepare_simulation(case, simulation))
            results.append(check(value_prepared, "run_parameter_default", parameter=2))
            body_edit = stage("reload_one_equation", lambda: value_edit.reload_from_parent([documents(source(2, True))]))
            body_prepared = stage("prepare_one_equation", lambda: body_edit.prepare_simulation(case, simulation))
            results.append(check(body_prepared, "run_one_equation", parameter=2, changed=True))
            observe("all_runs_joined")
            emit({"kind": "resources_before_release", "value": json.loads(codec.encode_json(runtime.resource_usage()))})
            results.clear()
            del prepared, prepared_again, value_prepared, body_prepared
            del model, unchanged, value_edit, body_edit, physical, physical_again
            runtime.clear_program_cache()
            gc.collect()
            emit({"kind": "resources_after_release", "value": json.loads(codec.encode_json(runtime.resource_usage()))})
        finally:
            runtime.close()


if __name__ == "__main__":
    main()
