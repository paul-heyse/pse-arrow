# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit test documents admitted through the generated study boundary."""

import msgspec

import pse
from pse import codec
from pse.contracts.documents import (
    StudyInputsExplicit,
    StudyOutputDemandFull,
    StudyPoint,
    StudyRequest,
)
from pse.contracts.identities import DeclarationId
from pse.contracts.values import SemanticId


def assignment(
    target: str | SemanticId,
    magnitude: float,
    quantity: str,
    unit: str,
    *,
    member: bool = False,
) -> dict[str, object]:
    return {
        "target": {
            "kind": "member" if member else "path",
            "value": target.to_hex() if isinstance(target, SemanticId) else target,
        },
        "value": {"magnitude": magnitude, "quantity": quantity, "unit": unit},
    }


def physical_ids(
    physical: pse.PhysicalContext, quantity: str, unit: str
) -> tuple[str, str]:
    """Resolve input addresses through the actual selected physical owner."""
    return physical.resolve_quantity(quantity).to_hex(), physical.resolve_unit(
        unit
    ).to_hex()


def point(
    case: DeclarationId,
    settings: pse.SolveSettings,
    key: int,
    *,
    assignments: tuple[dict[str, object], ...] = (),
    predecessor: int | None = None,
    continuation: bool = False,
) -> StudyPoint:
    # These are deliberately small, explicit fixture budgets, rather than another
    # authority for the production preparation defaults.
    start: dict[str, object] = {"kind": "fresh"}
    if continuation:
        assert predecessor is not None
        start = {
            "kind": "continuation",
            "predecessor": predecessor,
            "role": "primal_solution",
            "permission": "require_usable",
            "unavailable": "refuse",
        }
    return codec.decode_json(
        msgspec.json.encode(
            {
                "operation": {
                    "kind": "declared_case",
                    "request": {
                        "case": case.to_hex(),
                        "route": "steady",
                        "settings": msgspec.to_builtins(settings),
                    },
                },
                "preparation": {
                    "compiler": {
                        "class_proof_work": 1_000_000,
                        "assembly": {
                            "contributions": 100_000,
                            "native_index": 100_000,
                            "worker_bytes": 64 * 1024 * 1024,
                        },
                        "optimization": {
                            "cores": 1,
                            "horner_iterations": 2,
                            "cpe_iterations": 1,
                            "evaluator": {"kind": "interpreted"},
                        },
                        "evaluation": {
                            "derivative_components": 4096,
                            "operations": 1_000_000,
                            "scratch_bytes": 64 * 1024 * 1024,
                            "provider_calls": 4096,
                        },
                    },
                    "limits": {
                        "depth": 64,
                        "items": 100_000,
                        "members": 100_000,
                        "body_occurrences": None,
                        "body_slots": None,
                    },
                },
                "overlay": {"assignments": assignments},
                "policy": {
                    "key": key,
                    "dependencies": []
                    if predecessor is None or continuation
                    else [{"kind": "usable_result", "predecessor": predecessor}],
                    "start": start,
                    "attempt_limit": 1,
                },
            }
        ),
        StudyPoint,
    )


def request(*points: StudyPoint) -> StudyRequest:
    return StudyRequest(
        output=StudyOutputDemandFull(),
        inputs=StudyInputsExplicit(value=points),
    )
