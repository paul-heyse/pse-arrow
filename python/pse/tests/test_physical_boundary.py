# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Public physical transport imports actual registry schemas and owns input custody."""

import gc
from collections.abc import Iterator
from pathlib import Path
from typing import cast

import msgspec
import pyarrow as pa
import pytest

import pse
from pse import codec
from pse.contracts import runtime as result_contracts
from pse.contracts.enums import (
    DynamicsMethod,
    ModelingDeclarationKind,
    NativeBackend,
    NativeSolveIntent,
    PhysicalInputKind,
)
from pse.contracts.values import SemanticId
from pse.tests.physical_input_fixture import (
    coordinate_batch,
    member,
    sample_batch,
    stage_batch,
)
from pse.tests.test_native_workflow import identity, package_documents, physical_context


@pytest.fixture
def local_runtime(inspection_settings: pse.EngineSettings) -> Iterator[pse.Runtime]:
    runtime = pse.Runtime(inspection_settings, ephemeral=True)
    try:
        yield runtime
    finally:
        runtime.clear_program_cache()
        runtime.close()


def overwrite_values(batch: pa.RecordBatch, value: float) -> None:
    """Change the actual producer buffer, retaining all Arrow field metadata."""
    if "values" in batch.schema.names:
        values = batch.column("values").values.field("value")
    else:
        values = batch.column("value")
    buffer = values.storage.field("value").buffers()[1]
    assert buffer.is_mutable
    memory = memoryview(buffer).cast("d")
    for index in range(len(memory)):
        memory[index] = value


class MutatingStream:
    """Mutate the yielded buffer when the consumer requests the end of its stream.

    The snapshot has consumed the first batch by then. This places mutation before
    downstream preparation and native execution, without a timing race or a sleep.
    """

    def __init__(self, batch: pa.RecordBatch) -> None:
        self.batch = batch
        self.mutated = False

    def __arrow_c_stream__(self, requested_schema: object = None) -> object:
        def batches() -> Iterator[pa.RecordBatch]:
            yield self.batch
            overwrite_values(self.batch, 999.0)
            self.mutated = True

        return pa.RecordBatchReader.from_batches(
            self.batch.schema, batches()
        ).__arrow_c_stream__()


@pytest.mark.unit
def test_physical_input_reflection_retains_complete_schema_metadata(
    inspection_settings: pse.EngineSettings,
) -> None:
    runtime = pse.Runtime(inspection_settings, ephemeral=True)
    identities: set[bytes] = set()
    for kind in PhysicalInputKind:
        schema = pa.schema(runtime.physical_input_schema(kind))
        assert schema.metadata is not None
        assert schema.metadata[b"pse.contract.fingerprint"]
        identities.add(schema.metadata[b"pse.contract.id"])
        roundtrip = pa.ipc.read_schema(pa.BufferReader(schema.serialize()))
        assert roundtrip.equals(schema, check_metadata=True)
    assert len(identities) == len(PhysicalInputKind)
    with pytest.raises(TypeError):
        runtime.physical_input_schema(cast("PhysicalInputKind", "state_tolerances"))


@pytest.mark.unit
def test_simulation_physical_boundary_refuses_metadata_loss_and_duplicate_keys(
    inspection_settings: pse.EngineSettings,
) -> None:
    runtime = pse.Runtime(inspection_settings, ephemeral=True)
    empty = coordinate_batch(runtime)

    def settings(atol: pa.RecordBatch) -> pse.SimulationSettings:
        return pse.SimulationSettings(
            runtime=runtime,
            start=-0.125,
            end=0.125,
            samples=[-0.125, 0.125],
            parameter_scales=empty,
            atol=atol,
        )

    with pytest.raises(pse.InspectionError):
        settings(empty.replace_schema_metadata(None))
    coordinate = SemanticId.from_hex("01010101010101010101010101010101")
    duplicate = coordinate_batch(
        runtime, [(coordinate, 1.0, "Scalar"), (coordinate, 2.0, "Scalar")]
    )
    with pytest.raises(pse.InspectionError, match="duplicate"):
        settings(duplicate)
    admitted = settings(empty)
    assert admitted.document.version == 1
    assert admitted.document.controls.start == -0.125
    assert admitted.document.controls.end == 0.125


def length_context(runtime: pse.Runtime) -> pse.PhysicalContext:
    """Add an explicit centimetre representation to the real length fixture."""
    root = (
        Path(__file__).resolve().parents[3]
        / "tests/fixtures/packages/physical-primitives"
    )
    documents = {
        str(path.relative_to(root)): path.read_text()
        for path in root.rglob("*")
        if path.is_file()
    }
    text = "\n".join(
        line
        for line in documents["materials/physical.yaml"].splitlines()
        if not line.lstrip().startswith("#")
    )
    material = codec.decode_json(text, dict[str, msgspec.Raw])
    units = codec.decode_json(bytes(material["units"]), list[dict[str, msgspec.Raw]])
    centimetre = dict(units[0])
    for key, value in {
        "unit_id": identity(245).to_hex(),
        "symbol": "cm",
        "name": "centimetre",
        "scale_to_canonical": 0.01,
    }.items():
        centimetre[key] = msgspec.Raw(codec.encode_json(value))
    units.append(centimetre)
    material["units"] = msgspec.Raw(codec.encode_json(units))
    documents["materials/physical.yaml"] = codec.encode_json(material).decode()
    return runtime.physical_from_documents(documents)


def centimetres(batch: pa.RecordBatch) -> pa.RecordBatch:
    # Preserve the original list/struct/extension children: PyArrow cannot rebuild
    # these nested extension arrays from Python dictionaries.
    quantity_values = batch.column("values").values.field("value")
    units = quantity_values.storage.field("unit_id")
    buffer = units.buffers()[1]
    assert buffer.is_mutable
    memory = memoryview(buffer).cast("B")
    for index in range(len(units)):
        memory[index * 16 : (index + 1) * 16] = bytes(identity(245))
    return batch


@pytest.mark.integration
def test_initialization_and_primal_output_keep_converted_keyed_snapshots(
    local_runtime: pse.Runtime,
) -> None:
    runtime = local_runtime
    package = runtime.modeling_from_documents(
        [
            package_documents("""package stage_custody {
         def Root {param p:Length=2{m}; var x:Length;
          eq balance:x==p; annotation start x(0{m});}
        }""")
        ],
        length_context(runtime),
    )
    case = next(
        row.declaration_id for row in package.declarations() if row.name == "Root"
    )
    p, x = (member(package, case, name) for name in ("p", "x"))
    # Sparse stages, including the empty stage, remain legal and ordered by key.
    batch = centimetres(stage_batch(runtime, [(), [(p, 300.0, "Length")]]))
    producer = MutatingStream(batch.take(pa.array([1, 0], type=pa.int64())))
    strategy = package.prepare_block_initialization(
        case,
        pse.SolveSettings(
            intent=NativeSolveIntent.INITIALIZE, backend=NativeBackend.KINSOL
        ),
        producer,
    )
    assert producer.mutated
    del batch, producer
    gc.collect()
    result = strategy.run()
    initialization = result.initialization()
    assert initialization is not None
    assert initialization.completed_stages == 2
    assert initialization.solved_unknowns[x.to_hex()] == pytest.approx(3.0)
    attempts = [
        attempt.report for attempt in result.attempts() if attempt.report is not None
    ]
    assert attempts
    output = pa.table(attempts[-1].primal())
    expected = pa.schema(runtime.physical_input_schema(PhysicalInputKind.PRIMAL_VALUES))
    assert output.schema.equals(expected, check_metadata=True)
    restored = pa.RecordBatchReader.from_stream(output).read_all()
    assert restored.schema.equals(expected, check_metadata=True)
    rows = restored.to_pylist()
    assert any(
        row["coordinate_id"] == bytes(x) and row["value"]["value"] == pytest.approx(3.0)
        for row in rows
    )
    with pytest.raises(pse.InspectionError, match="unknown physical coordinate"):
        package.prepare_block_initialization(
            case,
            pse.SolveSettings(
                intent=NativeSolveIntent.INITIALIZE, backend=NativeBackend.KINSOL
            ),
            stage_batch(runtime, [[(identity(250), 3.0, "Length")]]),
        )
    with pytest.raises(pse.InspectionError, match="quantity differs"):
        package.prepare_block_initialization(
            case,
            pse.SolveSettings(
                intent=NativeSolveIntent.INITIALIZE, backend=NativeBackend.KINSOL
            ),
            stage_batch(runtime, [[(p, 3.0, "Time")]]),
        )


@pytest.mark.integration
def test_diagnostic_samples_keep_sparse_converted_points_after_producer_mutation(
    local_runtime: pse.Runtime,
) -> None:
    runtime = local_runtime
    package = runtime.modeling_from_documents(
        [
            package_documents("""package diagnostic_custody {
         def Root {var x:Length; var y:Length;
          eq balance:x+y==0{m}; eq contrast:x-y==6{m};
          annotation start x(1{m}); annotation start y(-1{m});}
        }""")
        ],
        length_context(runtime),
    )
    case = next(
        row.declaration_id for row in package.declarations() if row.name == "Root"
    )
    x, y = (member(package, case, name) for name in ("x", "y"))
    producer = MutatingStream(
        centimetres(
            sample_batch(
                runtime,
                [
                    (identity(241), [(y, -300.0, "Length"), (x, 300.0, "Length")]),
                    (identity(242), ()),
                ],
            )
        )
    )
    profile = (
        Path(__file__).resolve().parents[3]
        / "packages/reference/diagnostics/idaes-2.13.json"
    )
    result = package.diagnose_samples(
        case,
        pse.SolveSettings(intent=NativeSolveIntent.ROOT),
        pse.ModelingDiagnosticSettings.from_json(profile.read_text()),
        producer,
    )
    assert producer.mutated
    del producer
    gc.collect()
    assert result.ids() == (identity(241), identity(242))
    for ordinal, expected in enumerate(({x: 3.0, y: -3.0}, {x: 1.0, y: -1.0})):
        report = result.result(ordinal)
        assert report is not None
        assert report.complete
        (row,) = pa.table(report.table()).to_pylist()
        point = {
            SemanticId(value["source_id"]): value["value"] for value in row["point"]
        }
        for coordinate, value in expected.items():
            assert point[coordinate] == pytest.approx(value)


@pytest.mark.integration
def test_simulation_snapshots_all_coordinate_controls_and_precise_signed_time(
    local_runtime: pse.Runtime,
) -> None:
    runtime = local_runtime
    source = """package signed_time {
     def D {
      domain t:Time from -0.0000002{s} to 0.0000002{s};
      discretize grid on t using integrated(elements=1,order=1);
      param p:Scalar=1; var x[i in t]:Time;
      eq rate[i in t]:d(x[i])/di==p;
      eq initial:x[-0.0000002{s}]==1{s};
      let area:Time=integral(i in t | p);
      annotation report area("integral");
     }
     case run fixture {dof 0; route integrated; procedure integrate;
      integrate samples(-0.0000002{s},0{s},0.0000002{s}) relative(1e-8)
       normalized_absolute(1e-8) step(0.000000001{s})
       quadrature_relative(1e-8) quadrature_absolute(root.area=1e-12{s});
     } {child root:D=D();}
    }"""
    package = runtime.modeling_from_documents(
        [package_documents(source)], physical_context(runtime)
    )
    case = next(
        row.declaration_id for row in package.declarations() if row.name == "run"
    )
    x, p = (member(package, case, name) for name in ("x", "p"))
    inspection = package.inspect(case, pse.SolveSettings())
    area_members = tuple(
        value
        for value in inspection.members
        if value.lineage.path.rsplit(".", 1)[-1] == "area"
    )
    # An integral has an authored let, a flux let and a generated parameter result.
    # Physical output tolerances address that result, which owns the quadrature.
    (area_coordinate,) = (
        value
        for value in area_members
        if value.role is ModelingDeclarationKind.PARAMETER
    )
    area = area_coordinate.id
    state = coordinate_batch(runtime, [(x, 1e-10, "Time")])
    parameters = coordinate_batch(runtime, [(p, 2.0, "Scalar")])
    outputs = coordinate_batch(runtime, [(area, 1e-12, "Time")])
    times = [-2e-7, -1e-7, 0.0, 2e-7]
    settings = pse.SimulationSettings(
        runtime=runtime,
        start=times[0],
        end=times[-1],
        samples=times,
        initial_step=1e-9,
        method=DynamicsMethod.DIFFSOL,
        atol=state,
        parameter_scales=parameters,
        out_atol=outputs,
        out_rtol=1e-8,
    )
    before = codec.encode_json(settings.document)
    for batch in (state, parameters, outputs):
        overwrite_values(batch, 999.0)
    del state, parameters, outputs, batch
    gc.collect()
    assert codec.encode_json(settings.document) == before
    controls = settings.document.controls
    assert controls.start == -2e-7
    assert controls.initial_step == 1e-9
    assert controls.samples == tuple(times)
    assert controls.atol[0].value.value == 1e-10
    assert controls.parameter_scales[0].value.value == 2.0
    assert controls.out_atol[0].value.value == 1e-12
    for field in ("atol", "parameter_scales", "out_atol"):
        incomplete = msgspec.structs.replace(
            settings.document,
            controls=msgspec.structs.replace(controls, **{field: ()}),
        )
        with pytest.raises(pse.InspectionError, match="every prepared coordinate"):
            package.prepare_simulation(
                case, pse.SimulationSettings.from_document(runtime, incomplete)
            )
    prepared = package.prepare_simulation(case, settings)
    del settings
    gc.collect()
    result = prepared.start().wait()
    assert result.usable
    rows = codec.structure_rows(
        pa.table(result.table("runtime.simulation_samples")).to_pylist(),
        result_contracts.RuntimeSimulationSamplesRow,
    )
    assert {row.symbol_id for row in rows} == {x}
    assert [row.time for row in rows] == times
    for row in rows:
        assert row.value == pytest.approx(1.0 + row.time - times[0], rel=0, abs=1e-10)
    assert rows[-1].value - rows[0].value == pytest.approx(
        times[-1] - times[0], rel=0, abs=1e-10
    )
    # Whole-domain integrals are terminal products, excluded from sample series.
    (integral,) = codec.structure_rows(
        pa.table(result.table("runtime.modeling_reports")).to_pylist(),
        result_contracts.RuntimeModelingReportsRow,
    )
    assert integral.label == "integral"
    (reported_area,) = (
        value for value in area_members if value.id == integral.target_id
    )
    assert reported_area.role is ModelingDeclarationKind.LET
    assert reported_area.lineage.declaration == area_coordinate.lineage.declaration
    assert integral.value == pytest.approx(times[-1] - times[0], rel=0, abs=1e-12)


@pytest.mark.unit
def test_wall_budget_rounds_to_nanoseconds_and_refuses_invalid_extent(
    local_runtime: pse.Runtime,
) -> None:
    runtime = local_runtime

    def settings(budget: float) -> pse.SimulationSettings:
        return pse.SimulationSettings(
            runtime=runtime,
            start=-1e-9,
            end=1e-9,
            samples=[-1e-9, 1e-9],
            atol=coordinate_batch(runtime),
            parameter_scales=coordinate_batch(runtime),
            time_limit=budget,
        )

    admitted = settings(1.25e-9)
    assert admitted.document.controls.time_limit.secs == 0
    assert admitted.document.controls.time_limit.nanos == 1
    for budget in (-1.0, float("nan"), float("inf"), 1e100):
        with pytest.raises(pse.InspectionError):
            settings(budget)
