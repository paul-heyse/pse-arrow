# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Accuracy request version boundaries and immutable historical result reads."""

from pathlib import Path

import msgspec
import pytest

import pse
from pse import codec
from pse.contracts import documents, enums
from pse.contracts.identities import AccuracyGoalId, RunId
from pse.contracts.values import ContentHash, SemanticId
from pse.tests.physical_input_fixture import coordinate_batch


@pytest.mark.unit
def test_contextual_accuracy_current_request_leaves_refuse_obsolete_versions() -> None:
    assert documents.NumericalPolicy().version == 1
    assert pse.SolveSettings().version == 5
    for document, obsolete in (
        (documents.NumericalPolicy, 0),
        (documents.NumericalPolicy, 2),
        (documents.SolveSettings, 3),
        (documents.PhysicalSimulationRequest, 0),
        (documents.PhysicalSimulationRequest, 2),
        (documents.StudyOperation, 4),
    ):
        with pytest.raises(msgspec.ValidationError):
            codec.decode_json(codec.encode_json({"version": obsolete}), document)


@pytest.mark.unit
def test_contextual_accuracy_document_versions_precede_defaults() -> None:
    for document in (
        documents.NumericalPolicy,
        documents.SolveSettings,
        documents.PhysicalSimulationRequest,
    ):
        with pytest.raises(msgspec.ValidationError, match="version"):
            codec.decode_json(b"{}", document)
    # A malformed current body cannot precede a missing interpretation header.
    with pytest.raises(
        msgspec.ValidationError, match="Missing required document version"
    ):
        codec.decode_json(b'{"integrality":"not a number"}', documents.NumericalPolicy)
    for invalid in (b"0", b"2", b"true", b"1.0", b"null", b'"1"'):
        with pytest.raises(msgspec.ValidationError, match="version"):
            codec.decode_json(
                b'{"version":' + invalid + b"}", documents.NumericalPolicy
            )


@pytest.mark.unit
def test_contextual_accuracy_explicit_nested_policy_cannot_be_defaulted_on_read(
    inspection_settings: pse.EngineSettings,
) -> None:
    current = codec.decode_json(b'{"version":5}', documents.SolveSettings)
    assert current.numerics.version == 1  # Omission uses the owning current default.
    wire = codec.decode_json(codec.encode_json(current), dict[str, msgspec.Raw])
    wire["numerics"] = msgspec.Raw(b"{}")
    with pytest.raises(msgspec.ValidationError, match=r"\$\.numerics\.version"):
        codec.decode_json(codec.encode_json(wire), documents.SolveSettings)
    wire["numerics"] = msgspec.Raw(b'{"integrality":"not a number"}')
    with pytest.raises(
        msgspec.ValidationError, match="Missing required document version"
    ):
        codec.decode_json(codec.encode_json(wire), documents.SolveSettings)
    # These are ordinary explicitly current typed constructions, not historical reads.
    policy = documents.NumericalPolicy()
    assert policy.version == 1
    assert pse.SolveSettings(numerics=policy).version == 5
    assert (
        codec.decode_json(codec.encode_json(policy), documents.NumericalPolicy)
        == policy
    )
    # The native typed admission accepts that same explicitly current policy.
    runtime = pse.Runtime(inspection_settings, ephemeral=True)
    settings = pse.SimulationSettings(
        runtime=runtime,
        start=0.0,
        end=1.0,
        samples=[0.0, 1.0],
        atol=coordinate_batch(runtime),
        parameter_scales=coordinate_batch(runtime),
        numerics=policy,
    )
    assert codec.document_builtins(
        settings.document.controls.numerics
    ) == codec.document_builtins(policy)


@pytest.mark.unit
def test_contextual_accuracy_versions_are_checked_inside_typed_containers() -> None:
    for payload, document in (
        (b"[{}]", list[documents.NumericalPolicy]),
        (b"[{}]", tuple[documents.NumericalPolicy, ...]),
        (b"[{}]", tuple[documents.NumericalPolicy]),
        (b'{"policy":{}}', dict[str, documents.NumericalPolicy]),
    ):
        with pytest.raises(msgspec.ValidationError, match="version"):
            codec.decode_json(payload, document)
    assert codec.decode_json(b"null", documents.NumericalPolicy | None) is None
    assert codec.decode_json(b"[]", list[documents.NumericalPolicy]) == []
    # Uninterpreted raw documents remain raw; later typed reads own admission.
    assert (
        bytes(codec.decode_json(b'{"policy":{}}', dict[str, msgspec.Raw])["policy"])
        == b"{}"
    )


@pytest.mark.unit
def test_contextual_accuracy_simulation_version_precedes_policy(
    inspection_settings: pse.EngineSettings,
) -> None:
    runtime = pse.Runtime(inspection_settings, ephemeral=True)
    settings = pse.SimulationSettings(
        runtime=runtime,
        start=0.0,
        end=1.0,
        samples=[0.0, 1.0],
        atol=coordinate_batch(runtime),
        parameter_scales=coordinate_batch(runtime),
    )
    document = settings.document
    assert document.version == 1
    assert pse.SimulationSettings.from_document(runtime, document).document == document
    historical_policy = msgspec.structs.replace(document.controls.numerics, version=0)
    for obsolete in (0, 2, 3):
        # Both interpretations are obsolete. The enclosing header must refuse
        # before the current body can interpret its nested accuracy policy.
        historical = msgspec.structs.replace(
            document,
            version=obsolete,
            controls=msgspec.structs.replace(
                document.controls, numerics=historical_policy
            ),
        )
        with pytest.raises(pse.InspectionError) as refused:
            pse.SimulationSettings.from_document(runtime, historical)
        report = refused.value.report
        assert report.reported_code == "validation.invariant"
        location = report.decode_location
        assert isinstance(location, documents.DecodeLocation)
        assert (location.stage, location.category) == ("header", "data")
    # A current enclosing version cannot launder an obsolete accuracy policy.
    with pytest.raises(pse.InspectionError) as refused:
        pse.SimulationSettings.from_document(
            runtime,
            msgspec.structs.replace(
                document,
                controls=msgspec.structs.replace(
                    document.controls, numerics=historical_policy
                ),
            ),
        )
    report = refused.value.report
    assert report.reported_code == "validation.invariant"
    location = report.decode_location
    assert isinstance(location, documents.DecodeLocation)
    assert (location.stage, location.category) == ("body", "data")
    assert location.path == (
        documents.DecodePathSegmentMap(key="controls"),
        documents.DecodePathSegmentMap(key="numerics"),
        documents.DecodePathSegmentMap(key="version"),
    )


@pytest.mark.unit
def test_contextual_accuracy_historical_completion_preserves_interpretation() -> None:
    products = codec.decode_json(
        (
            Path(__file__).parent / "fixtures/generated-native-boundaries/products.json"
        ).read_bytes(),
        dict[str, msgspec.Raw],
    )
    for name in ("qualified_incumbent", "event_ended_trajectory"):
        current_bytes = bytes(products[name])
        wire = codec.decode_json(current_bytes, dict[str, msgspec.Raw])
        # The old product omitted this collection. Its retained numerical/native
        # observations remain unchanged; reading it grants no accuracy claim.
        wire.pop("accuracy_goals", None)
        historical_bytes = codec.encode_json(wire)
        historical = codec.decode_json(historical_bytes, documents.Completion)
        retained = codec.decode_json(current_bytes, documents.Completion)
        assert historical.accuracy_goals == ()
        assert historical.assessments == retained.assessments
        assert historical.solves == retained.solves
        assert historical.computation == retained.computation
        assert historical.diagnostics == retained.diagnostics
        assert historical.lineage == retained.lineage
        assert historical_bytes == codec.encode_json(wire)


@pytest.mark.unit
def test_contextual_accuracy_completion_preserves_estimated_violated_decision() -> None:
    """Export retains a resolved violation and its estimated evidence unchanged."""
    row = documents.RuntimeAccuracyGoalAssessmentsRow(
        goal_id=AccuracyGoalId(SemanticId.from_hex("01" * 16)),
        run_id=RunId(SemanticId.from_hex("02" * 16)),
        target_id=SemanticId.from_hex("03" * 16),
        quantity_id=SemanticId.from_hex("04" * 16),
        unit_id=SemanticId.from_hex("05" * 16),
        step=7,
        subject=enums.AccuracyGoalSubject.SELECTED_OUTPUT,
        target_kind=enums.NumericalTarget.OBSERVABLE,
        observation=enums.AccuracyObservation.STEADY,
        required_class=enums.NumericalAccuracyClass.ESTIMATED,
        accuracy_class=enums.NumericalAccuracyClass.ESTIMATED,
        use_policy=enums.AccuracyGoalUse.ASSESS,
        refine=True,
        resolution=0.1,
        value=301.0,
        error=0.02,
        interval_lower=300.98,
        interval_upper=301.02,
        criterion_upper=300.0,
        status=enums.AccuracyGoalStatus.VIOLATED,
        resolution_status=enums.AccuracyResolutionStatus.MET,
        criterion_status=enums.AccuracyCriterionStatus.VIOLATED,
        interpretation=enums.AccuracyEvidenceInterpretation.OUTPUT_ERROR,
        method=enums.AccuracyEvidenceMethod.SQUARE_CORRECTION,
        dependencies=(ContentHash.from_prefixed("blake3:" + "06" * 32),),
        product=ContentHash.from_prefixed("blake3:" + "07" * 32),
        validity=ContentHash.from_prefixed("blake3:" + "08" * 32),
        limitation="Local estimate on the retained stationary branch.",
    )
    completion = documents.Completion(
        accuracy_goals=(row,),
        assessments=(),
        diagnostics=(),
        lineage=(),
        solves=(),
        products=(),
        step_permissions=(),
    )
    encoded = codec.encode_json(completion)
    restored = codec.decode_json(encoded, documents.Completion)
    assert restored == completion
    assert restored.accuracy_goals[0].status == enums.AccuracyGoalStatus.VIOLATED
    assert restored.accuracy_goals[0].criterion_upper == 300.0
    assert restored.accuracy_goals[0].error == 0.02
    assert (
        restored.accuracy_goals[0].accuracy_class
        == enums.NumericalAccuracyClass.ESTIMATED
    )
    assert restored.accuracy_goals[0].dependencies == (
        ContentHash.from_prefixed("blake3:" + "06" * 32),
    )
    assert codec.encode_json(restored) == encoded
