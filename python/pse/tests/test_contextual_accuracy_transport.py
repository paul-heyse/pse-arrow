# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Accuracy request version boundaries and immutable historical result reads."""

from pathlib import Path

import msgspec
import pytest

import pse
from pse import codec
from pse.contracts import documents, enums


@pytest.mark.unit
def test_contextual_accuracy_current_request_leaves_refuse_obsolete_versions() -> None:
    assert documents.NumericalPolicy().version == 1
    assert pse.SolveSettings().version == 4
    for document, obsolete in (
        (documents.NumericalPolicy, 0),
        (documents.NumericalPolicy, 2),
        (documents.SolveSettings, 3),
        (documents.Profile, 0),
        (documents.Profile, 2),
        (documents.StudyOperation, 4),
    ):
        with pytest.raises(msgspec.ValidationError):
            codec.decode_json(codec.encode_json({"version": obsolete}), document)


@pytest.mark.unit
def test_contextual_accuracy_document_versions_precede_defaults() -> None:
    for document in (
        documents.NumericalPolicy,
        documents.SolveSettings,
        documents.Profile,
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
def test_contextual_accuracy_explicit_nested_policy_cannot_be_defaulted_on_read() -> (
    None
):
    current = codec.decode_json(b'{"version":4}', documents.SolveSettings)
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
    assert pse.SolveSettings(numerics=policy).version == 4
    assert (
        codec.decode_json(codec.encode_json(policy), documents.NumericalPolicy)
        == policy
    )
    # The native typed admission accepts that same explicitly current policy.
    settings = pse.SimulationSettings(
        start=0.0,
        end=1.0,
        samples=[0.0, 1.0],
        atol=[0.001],
        parameter_scales=[],
        numerics=policy,
    )
    assert codec.decode_json(settings.to_json(), documents.Profile).numerics == policy


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
def test_contextual_accuracy_simulation_version_precedes_policy() -> None:
    settings = pse.SimulationSettings(
        start=0.0, end=1.0, samples=[0.0, 1.0], atol=[0.001], parameter_scales=[]
    )
    wire = codec.decode_json(settings.to_json(), dict[str, msgspec.Raw])
    assert bytes(wire["version"]) == b"1"
    restored = pse.SimulationSettings.from_json(settings.to_json())
    assert codec.decode_json(restored.to_json(), documents.Profile).version == 1
    for obsolete in (b"0", b"2", None):
        historical = dict(wire)
        if obsolete is None:
            historical.pop("version")
        else:
            historical["version"] = msgspec.Raw(obsolete)
        with pytest.raises(pse.InspectionError, match="version"):
            pse.SimulationSettings.from_json(codec.encode_json(historical).decode())
    # A current enclosing version cannot launder an obsolete accuracy policy.
    wire["numerics"] = msgspec.Raw(b'{"version":0}')
    with pytest.raises(pse.InspectionError, match="version"):
        pse.SimulationSettings.from_json(codec.encode_json(wire).decode())
    wire["numerics"] = msgspec.Raw(b"{}")
    with pytest.raises(pse.InspectionError, match="version"):
        pse.SimulationSettings.from_json(codec.encode_json(wire).decode())


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
        goal_id="01" * 16,
        run_id="02" * 16,
        target_id="03" * 16,
        quantity_id="04" * 16,
        unit_id="05" * 16,
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
        dependencies=("blake3:" + "06" * 32,),
        product="blake3:" + "07" * 32,
        validity="blake3:" + "08" * 32,
        limitation="Local estimate on the retained stationary branch.",
    )
    completion = documents.Completion(
        accuracy_goals=(row,), assessments=(), diagnostics=(), lineage=(), solves=()
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
    assert restored.accuracy_goals[0].dependencies == ("blake3:" + "06" * 32,)
    assert codec.encode_json(restored) == encoded
