# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Local composition exercises only the native model's actual dependencies."""

from pathlib import Path

import msgspec
import pytest

import pse
from pse import codec
from pse.contracts import documents
from pse.contracts.documents import SourceDelta
from pse.contracts.enums import NativeSolveIntent
from pse.tests.test_native_workflow import (
    declaration,
    identity,
    package_documents,
    revision,
)


def physical_documents() -> dict[str, str]:
    root = (
        Path(__file__).resolve().parents[3]
        / "tests/fixtures/packages/physical-primitives"
    )
    return {
        str(path.relative_to(root)): path.read_text()
        for path in root.rglob("*")
        if path.is_file()
    }


@pytest.mark.unit
def test_local_runtime_owns_models_and_exact_edits_without_a_substrate(
    inspection_settings: pse.EngineSettings,
) -> None:
    runtime = pse.Runtime(inspection_settings, ephemeral=True)
    assert not runtime.durable
    physical = runtime.physical_from_documents(physical_documents())
    sources = package_documents("package local {param a:Scalar=1;}")
    package = runtime.modeling_from_documents([sources], physical)
    assert package.revision_id
    assert package.canonical_revision is None
    assert package.canonical_problem is None
    assert (
        package.with_delta(SourceDelta(parent=package.revision_id)).revision_id
        == package.revision_id
    )
    original = package.declarations()
    changed = package.reload_from_parent(
        [package_documents("package local {param a:Scalar=2;}")]
    )
    assert changed.revision_id != package.revision_id
    assert changed.canonical_revision is None
    assert package.declarations() == original
    with pytest.raises(pse.InspectionError):
        changed.with_delta(SourceDelta(parent=package.revision_id))
    with pytest.raises(pse.InspectionError):
        runtime.run_record("unavailable")
    with pytest.raises(pse.InspectionError):
        runtime.source_settlement("original-problem", "original-operation")
    runtime.close()
    with pytest.raises(pse.InspectionError):
        package.declarations()


@pytest.mark.unit
def test_configured_ephemeral_runtime_does_not_open_its_substrate(
    inspection_settings: pse.EngineSettings,
    tmp_path: Path,
) -> None:
    missing = tmp_path / "never-opened-state"
    runtime = pse.Runtime(inspection_settings, ephemeral=True, substrate=str(missing))
    physical = runtime.physical_from_documents(physical_documents())
    package = runtime.modeling_from_documents(
        [package_documents("package local {param a:Scalar=1;}")], physical
    )
    assert package.canonical_revision is None
    assert not missing.exists()
    runtime.close()
    with pytest.raises(pse.InspectionError):
        runtime.run_record("unavailable")
    assert not missing.exists(), "closed caller cannot start lazy connection"
    with pytest.raises(pse.InspectionError):
        runtime.physical_from_documents(physical_documents())
    with pytest.raises(pse.InspectionError):
        runtime.modeling_from_documents(
            [package_documents("package closed {param a:Scalar=1;}")], physical
        )


@pytest.mark.unit
@pytest.mark.parametrize(("ephemeral", "database"), [(False, None), (True, "orphan")])
def test_local_composition_requires_explicit_ephemeral_and_valid_sink_selection(
    inspection_settings: pse.EngineSettings,
    ephemeral: bool,
    database: str | None,
) -> None:
    with pytest.raises(pse.InspectionError):
        pse.Runtime(inspection_settings, ephemeral=ephemeral, database=database)


@pytest.mark.unit
def test_retained_attempt_summary_and_replay_use_original_inputs(
    inspection_settings: pse.EngineSettings,
) -> None:
    runtime = pse.Runtime(inspection_settings, ephemeral=True)
    physical = runtime.physical_from_documents(physical_documents())
    prepared = revision(runtime, physical).prepare_solve(
        declaration(101), pse.SolveSettings(intent=NativeSolveIntent.ROOT)
    )
    handle = prepared.start()
    result = handle.wait()
    summary = result.attempt_summary
    assert summary.usable == result.usable
    assert summary.requested_steps == 1
    assert summary.omitted_steps == 0
    assert result.attempt_summary == summary
    # This original root has no established equivalent convex optimization cohort.
    assert prepared.advice_inputs is None
    with pytest.raises(pse.InspectionError, match="equivalence"):
        prepared.freeze_advice("native_execution_nanoseconds", ())
    replay = result.replay()
    document = replay.document(bytes=128 * 1024 * 1024)
    assert document.coverage.usable == result.usable
    assert not document.native_continuation
    assert document.sources
    assert document.case_constructors
    assert (
        codec.decode_json(codec.encode_json(document), documents.ReplayDocument)
        == document
    )
    with pytest.raises(pse.InspectionError, match="checkpoint"):
        replay.native_continuation()
    with pytest.raises(pse.InspectionError, match="allocation"):
        replay.document(bytes=1)
    repeated = replay.rerun().wait()
    assert repeated.run_id != result.run_id
    assert repeated.usable == result.usable
    assert result.attempt_summary == summary
    runtime.close()


@pytest.mark.unit
def test_frozen_advice_uses_actual_native_cohort_and_survives_replay(
    inspection_settings: pse.EngineSettings,
) -> None:
    runtime = pse.Runtime(inspection_settings, ephemeral=True)
    physical = runtime.physical_from_documents(physical_documents())
    source = (
        f'package advice {{ @id("{identity(202).to_hex()}") def Root {{ '
        "var x:Scalar; annotation bounds x(0,2); annotation start x(1); "
        "let f:Scalar=x; annotation objective f(minimize); eq floor:x>=0.5; } }"
    )
    package = runtime.modeling_from_documents([package_documents(source)], physical)
    settings = pse.SolveSettings()
    settings = msgspec.structs.replace(
        settings,
        composition=msgspec.structs.replace(
            settings.composition, limits=documents.WorkLimits(attempts=4)
        ),
    )
    prepared = package.prepare_solve(declaration(202), settings)
    inputs = prepared.advice_inputs
    assert inputs is not None
    assert inputs.candidates
    # Scripted observations exercise materialization, without claiming measured benefit.
    observations = tuple(
        documents.AdviceObservation(
            attempt=candidate,
            candidate=candidate,
            cohort=inputs.cohort,
            matched=inputs.candidates[0],
            outcome="complete",
            value=float(index + 1),
        )
        for index, candidate in enumerate(inputs.candidates)
    )
    frozen = prepared.freeze_advice("native_execution_nanoseconds", observations)
    assert isinstance(frozen.support, documents.AdviceSupportCompleteMatched)
    assert frozen.support.pairs == 1
    assert (
        codec.decode_json(codec.encode_json(frozen), documents.FrozenAdvice) == frozen
    )
    tied = prepared.freeze_advice("native_execution_nanoseconds", ())
    assert isinstance(tied.support, documents.AdviceSupportBaselineTie)
    assert tied.ordered == tied.baseline
    settings = msgspec.structs.replace(
        settings,
        composition=msgspec.structs.replace(settings.composition, advice=frozen),
    )
    advised = package.prepare_solve(declaration(202), settings)
    assert advised.advice_inputs == inputs
    assert advised.strategy_profile != prepared.strategy_profile
    result = advised.start().wait()
    assert result.usable
    summary = result.attempt_summary
    assert (
        codec.decode_json(codec.encode_json(summary), documents.AttemptSummary)
        == summary
    )
    exported = result.replay().document(bytes=128 * 1024 * 1024)
    assert exported.case_constructors[0].settings.composition.advice == frozen
    runtime.close()
