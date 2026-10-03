# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Actual generated contract boundaries, including nested values and metadata."""

from collections.abc import Callable, Hashable
from typing import get_type_hints

import attrs
import cattrs
import pyarrow as pa
import pytest

from pse.codec import converter, structure_rows
from pse.contracts import runtime as runtime_contracts
from pse.contracts import values as contract_values
from pse.contracts.authored import (
    AuthoredModelingDeclarationsFieldValue,
    AuthoredPackagesRow,
)
from pse.contracts.enums import (
    CandidateRefusal,
    ModelingEnvelopeExtent,
    ModelingVariableDomain,
    PackageKind,
)
from pse.contracts.extension_types import PseEnum, PseOrdinalRef
from pse.contracts.reference import ReferenceUnitsRow
from pse.contracts.structures import (
    MemberDescriptor,
    MemberDescriptorSelection,
    ModelingEnvelopeGuard,
)
from pse.contracts.values import FIELD_NAME_METADATA, ContentHash, SemanticId


@pytest.mark.unit
def test_validator_annotations_evaluate_on_supported_python_versions() -> None:
    # Python 3.14 defers annotations that Python 3.13 evaluates during import.
    # Force evaluation here so the host also catches parity import regressions.
    for validator in (
        contract_values.exact_type,
        contract_values.integer_range,
        contract_values.exact_type(bool),
        contract_values.integer_range(0, 7),
        contract_values.finite_float,
        contract_values.utc_timestamp,
    ):
        assert get_type_hints(validator)


def package_row() -> dict[str, object]:
    return {
        "package_id": "01" * 16,
        "name": "example",
        "version": "1.0.0",
        "kind": "model",
        "id_policy": "explicit",
        "dependencies": [
            {
                "package_id": "02" * 16,
                "version_req": {
                    "operator": "exact",
                    "major": 1,
                    "minor": 0,
                    "patch": 0,
                },
            }
        ],
        "content_hash": "blake3:" + "03" * 32,
        "doc": "nested fixture",
    }


@pytest.mark.unit
def test_nested_generated_row_is_typed_and_frozen() -> None:
    row = structure_rows([package_row()], AuthoredPackagesRow)[0]
    assert row.package_id == SemanticId(bytes([1] * 16))
    assert row.dependencies[0].package_id == SemanticId(bytes([2] * 16))
    assert row.content_hash == ContentHash(bytes([3] * 32))
    assert isinstance(row.dependencies, tuple)
    with pytest.raises(attrs.exceptions.FrozenInstanceError):
        # pyrefly: ignore[read-only] -- refusal is under test
        row.name = "changed"


@pytest.mark.unit
def test_declared_dependency_collection_rejects_duplicates_and_accepts_empty() -> None:
    document = package_row()
    item = {
        "package_id": "02" * 16,
        "version_req": {"operator": "exact", "major": 1, "minor": 0, "patch": 0},
    }
    document["dependencies"] = [item, item]
    with pytest.raises((ValueError, cattrs.BaseValidationError)):
        structure_rows([document], AuthoredPackagesRow)
    document["dependencies"] = []
    assert structure_rows([document], AuthoredPackagesRow)[0].dependencies == ()


@pytest.mark.unit
def test_typed_publication_selection_has_a_payload_free_full_case() -> None:
    full: dict[str, object] = {"kind": "full", "revision": None}
    value = converter().structure(full, MemberDescriptorSelection)
    assert value.revision is None
    revision: dict[str, object] = {"column": "revision_id", "revision_id": "01" * 16}
    for invalid in (
        full | {"revision": revision},
        {"kind": "revision", "revision": None},
    ):
        with pytest.raises((ValueError, cattrs.BaseValidationError)):
            converter().structure(invalid, MemberDescriptorSelection)
    value = converter().structure(
        {"kind": "revision", "revision": revision},
        MemberDescriptorSelection,
    )
    assert value.revision is not None


@pytest.mark.unit
def test_member_descriptor_is_one_named_structure() -> None:
    # Every relation that lists members references the one registry structure.
    manifest = attrs.fields_dict(runtime_contracts.RuntimePublicationManifestsRow)
    release = attrs.fields_dict(runtime_contracts.RuntimeArtifactDescriptorsRow)[
        "release_members"
    ].type
    for annotation in (manifest["members"].type, manifest["inputs"].type, release):
        assert "MemberDescriptor" in str(annotation)
    assert "VersionWindow" in str(manifest["windows"].type)
    assert not hasattr(runtime_contracts, "RuntimePublicationManifestsFieldMembersItem")
    assert (
        attrs.fields_dict(MemberDescriptor)["selection"].type
        is MemberDescriptorSelection
    )


@pytest.mark.unit
def test_nested_unknown_column_is_not_ignored() -> None:
    document = package_row()
    document["dependencies"] = [
        {
            "package_id": "02" * 16,
            "version_req": {"operator": "exact", "major": 1, "minor": 0, "patch": 0},
            "unchecked": True,
        },
    ]
    with pytest.raises(cattrs.BaseValidationError):
        structure_rows([document], AuthoredPackagesRow)


@pytest.mark.unit
@pytest.mark.parametrize("value", [True, "1", 1.5])
def test_integer_codec_does_not_coerce_other_value_kinds(value: object) -> None:
    with pytest.raises(ValueError):
        converter().structure(value, int)


@pytest.mark.unit
def test_fixed_dimension_width_is_enforced_without_a_fingerprint() -> None:
    values = {
        "unit_id": "01" * 16,
        "symbol": "x",
        "name": "x",
        "dimension": [{"num": 0, "den": 1}] * 7,
        "scale_to_canonical": 1.0,
        "offset_to_canonical": 0.0,
        "is_affine": False,
        "reference_state_id": None,
        "system": "SI",
        "doc": "",
    }
    with pytest.raises(cattrs.BaseValidationError):
        structure_rows([values], ReferenceUnitsRow)


@pytest.mark.unit
def test_parameterized_metadata_preserves_actual_binding() -> None:
    binding = "01" * 16
    enum = PseEnum(binding)
    assert (
        enum.__arrow_ext_serialize__() == (f'{{"v":1,"enum_id":"{binding}"}}').encode()
    )
    assert (
        PseEnum.__arrow_ext_deserialize__(
            pa.utf8(),
            enum.__arrow_ext_serialize__(),
        ).binding_id
        == binding
    )
    ordinal = PseOrdinalRef(binding)
    assert (
        PseOrdinalRef.__arrow_ext_deserialize__(
            pa.int64(),
            ordinal.__arrow_ext_serialize__(),
        ).binding_id
        == binding
    )
    with pytest.raises(ValueError):
        PseEnum.__arrow_ext_deserialize__(
            pa.utf8(),
            (f'{{"v":1,"enum_id":"{binding}","enum_id":"{binding}"}}').encode(),
        )
    with pytest.raises(ValueError):
        PseOrdinalRef.__arrow_ext_deserialize__(pa.int64(), b'{"v":1}')


@attrs.frozen(kw_only=True)
class _KeywordCollision:
    class__: str = attrs.field(metadata={FIELD_NAME_METADATA: "class"})
    class_: str


@pytest.mark.unit
def test_declared_keyword_and_existing_suffix_remain_distinct() -> None:
    codec = converter()
    values = {"class": "keyword value", "class_": "literal suffixed field"}
    row = codec.structure(values, _KeywordCollision)
    assert row.class__ == values["class"]
    assert row.class_ == values["class_"]
    assert codec.unstructure(row) == values
    with pytest.raises(cattrs.BaseValidationError):
        codec.structure({**values, "class__": "not a wire field"}, _KeywordCollision)


class _DeclaredText(str):
    """A declared text constructor must remain active at the codec boundary."""


@pytest.mark.unit
def test_text_hook_preserves_actual_constructor_and_refuses_invalid_enum() -> None:
    codec = converter()
    assert type(codec.structure("plain", str)) is str
    text = codec.structure("declared", _DeclaredText)
    assert type(text) is _DeclaredText
    assert text == "declared"
    assert codec.structure("model", PackageKind) is PackageKind.MODEL
    with pytest.raises(ValueError):
        codec.structure("undeclared", PackageKind)
    with pytest.raises(ValueError):
        codec.structure(123, _DeclaredText)


@pytest.mark.unit
def test_modeling_declaration_tag_has_one_typed_payload() -> None:
    payload: dict[str, object] = {
        field.metadata.get(FIELD_NAME_METADATA, field.name): None
        for field in attrs.fields(AuthoredModelingDeclarationsFieldValue)
    }
    payload["kind"] = "variable"
    payload["binding"] = {
        "type": [
            {
                "kind": "named",
                "path": ["Flow"],
                "name": None,
                "exponent": None,
                "children": [],
            }
        ],
        "indices": [],
        "expression": None,
        "defined_by": None,
        "domain": "binary",
    }
    value = converter().structure(payload, AuthoredModelingDeclarationsFieldValue)
    assert value.binding is not None
    assert value.binding.type is not None
    assert value.binding.type[0].path == ("Flow",)
    assert value.binding.indices == ()
    assert value.binding.domain is ModelingVariableDomain.BINARY
    for invalid in (
        payload | {"binding": None},
        payload | {"scope": {"parameters": [], "bases": [], "type_parameters": []}},
        payload | {"unchecked": True},
    ):
        with pytest.raises((ValueError, cattrs.BaseValidationError)):
            converter().structure(invalid, AuthoredModelingDeclarationsFieldValue)


@pytest.mark.unit
def test_hard_domain_interval_guards_are_typed() -> None:
    # Explicit hard-domain guards retain their declared whole interval.
    guard = converter().structure(
        {
            "carrier": "p",
            "envelope": "T",
            "extent": "interval",
            "arguments": ["T0", "T"],
        },
        ModelingEnvelopeGuard,
    )
    assert guard.extent is ModelingEnvelopeExtent.INTERVAL
    assert guard.arguments == ("T0", "T")


@pytest.mark.unit
def test_applicability_observation_retains_owner_and_scoped_permission() -> None:
    document: dict[str, object] = {
        "run_id": "01" * 16,
        "step": 0,
        "sample_index": 0,
        "time": None,
        "target_id": "02" * 16,
        "source_id": "03" * 16,
        "kind": "applicability",
        "value": 0.0,
        "tolerance": None,
        "satisfied": True,
        "within_validity": None,
        "extrapolation_allowed": False,
        "basis": "point",
        "layer": "data",
        "claim_id": "04" * 16,
        "claim_owner": "05" * 16,
        "claim_owner_lineage": ["05" * 16, "06" * 16],
        "coverage_id": None,
        "evidence_id": "07" * 16,
        "form_id": "08" * 16,
        "call_id": "09" * 16,
        "selected_records": ["0a" * 16],
        "dependencies": ["0b" * 16],
        "input_values": [{"name": "T", "value": 303.15, "quantity_type": "0c" * 16}],
        "applicability_outcome": "unknown_evidence",
        "applicability_basis": None,
        "permission_ids": ["0d" * 16],
        "unknown_allowed": True,
        "observation_instance": "0e" * 16,
        "applicability_required": True,
        "applicability_reason": "Source has no region for this use",
        "applicability_permissions": [
            {
                "permission_id": "0d" * 16,
                "scope": "0f" * 16,
                "target_kind": "families",
                "targets": ["06" * 16],
                "allow_unknown": True,
                "allow_extrapolation": False,
            }
        ],
    }
    row = structure_rows([document], runtime_contracts.RuntimeModelingChecksRow)[0]
    assert row.claim_owner_lineage == (
        SemanticId(bytes([5] * 16)),
        SemanticId(bytes([6] * 16)),
    )
    assert row.observation_instance == SemanticId(bytes([14] * 16))
    assert row.applicability_reason == "Source has no region for this use"
    assert row.input_values[0].value == 303.15
    permission = row.applicability_permissions[0]
    assert permission.permission_id == row.permission_ids[0]
    assert permission.scope == SemanticId(bytes([15] * 16))
    assert permission.targets == (SemanticId(bytes([6] * 16)),)
    assert permission.allow_unknown
    assert not permission.allow_extrapolation
    encoded = converter().unstructure(row)
    assert (
        structure_rows([encoded], runtime_contracts.RuntimeModelingChecksRow)[0] == row
    )
    with pytest.raises(cattrs.BaseValidationError):
        structure_rows(
            [document | {"claim_owner_lineage": ["invalid-id"]}],
            runtime_contracts.RuntimeModelingChecksRow,
        )


@pytest.mark.unit
@pytest.mark.parametrize(
    ("key", "values"),
    [
        (contract_values.scalar_key, (0.0, -0.0)),
        (
            contract_values.tuple_key(
                (
                    contract_values.scalar_key,
                    contract_values.sequence_key(contract_values.scalar_key),
                )
            ),
            (("x", (0.0,)), ("x", (-0.0,))),
        ),
        (
            contract_values.sequence_key(contract_values.scalar_key),
            ((1.0, 0.0), (1.0, -0.0)),
        ),
        (
            contract_values.mapping_key(
                contract_values.scalar_key,
                contract_values.sequence_key(contract_values.scalar_key),
            ),
            ({"a": (0.0,), "b": (2.0,)}, {"b": (2.0,), "a": (-0.0,)}),
        ),
    ],
)
def test_collection_keys_preserve_scalar_nested_and_mapping_equality(
    key: Callable[[object], Hashable], values: tuple[object, ...]
) -> None:
    with pytest.raises(ValueError, match="uniqueness"):
        contract_values.unique(key)(None, None, values)
    contract_values.unique(key)(None, None, values[:1])


@pytest.mark.unit
def test_fixed_tuple_key_requires_exact_shape_and_keeps_position_meaning() -> None:
    key = contract_values.tuple_key(
        (contract_values.scalar_key, contract_values.scalar_key)
    )
    assert key((1, 2)) != key((2, 1))
    for invalid in ((1,), (1, 2, 3), [1, 2]):
        with pytest.raises(ValueError, match="exact arity"):
            key(invalid)
    assert contract_values.tuple_key(())(()) == (tuple, ())


@pytest.mark.unit
def test_generated_record_uniqueness_preserves_field_equality() -> None:
    first = contract_values.QuantityValue(
        value=0.0, unit_id=SemanticId(b"a" * 16), quantity_type_id=SemanticId(b"b" * 16)
    )
    second = contract_values.QuantityValue(
        value=-0.0,
        unit_id=SemanticId(b"a" * 16),
        quantity_type_id=SemanticId(b"b" * 16),
    )
    assert first == second
    with pytest.raises(ValueError, match="uniqueness"):
        contract_values.unique(contract_values.record_key)(None, None, (first, second))


@pytest.mark.unit
@pytest.mark.parametrize(
    ("state", "evidence", "artifacts", "order"),
    [
        ("refused", [], [], None),
        ("pending_evidence", ["class", "structure"], [], None),
        ("supported_pending_artifacts", [], ["derivatives", "representation"], 2),
        ("ready", [], [], None),
    ],
)
def test_contextual_route_projection_retains_readiness_and_demands(
    state: str, evidence: list[str], artifacts: list[str], order: int | None
) -> None:
    document: dict[str, object] = {
        "request_identity": "blake3:" + "01" * 32,
        "step": 0,
        "intent": "root",
        "selection": "auto",
        "requested_backend": None,
        "classes": [],
        "state": state,
        "snapshot": "blake3:" + "02" * 32,
        "pending_backend": "ipopt" if evidence else None,
        "evidence": evidence,
        "artifacts": artifacts,
        "evidence_classes": [],
        "required_order": order,
        "artifact_representations": ["nlp"] if artifacts else [],
        "eligibility": [],
        "selected": None,
        "backend": None,
        "representation": None,
        "lexicographic": None,
        "refusal": None,
        "detail": None,
    }
    conversion = converter()
    row = conversion.structure(document, runtime_contracts.RuntimeRouteDecisionsRow)
    assert row.state.value == state
    assert [value.value for value in row.evidence] == evidence
    assert [value.value for value in row.artifacts] == artifacts
    assert row.required_order == order
    assert row.snapshot == ContentHash(bytes([2] * 32))
    assert (
        conversion.structure(
            conversion.unstructure(row), runtime_contracts.RuntimeRouteDecisionsRow
        )
        == row
    )


@pytest.mark.unit
def test_unavailable_feasibility_refusal_is_distinct_in_current_codec() -> None:
    conversion = converter()
    unavailable = conversion.structure("feasibility_unavailable", CandidateRefusal)
    violating = conversion.structure("infeasible", CandidateRefusal)
    assert unavailable is CandidateRefusal.FEASIBILITY_UNAVAILABLE
    assert unavailable is not violating
    assert conversion.unstructure(unavailable) == "feasibility_unavailable"
