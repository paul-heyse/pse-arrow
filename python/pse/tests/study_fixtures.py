# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit test documents admitted through the generated study boundary."""

from pathlib import Path

import msgspec

import pse
from pse import codec
from pse.contracts.documents import StudyPoint, StudyRequest
from pse.contracts.identities import DeclarationId
from pse.contracts.reference import ReferenceQuantityTypesRow


def assignment(
    target: str,
    magnitude: float,
    quantity: str,
    unit: str,
    *,
    member: bool = False,
) -> dict[str, object]:
    return {
        "target": {"kind": "member" if member else "path", "value": target},
        "value": {"magnitude": magnitude, "quantity": quantity, "unit": unit},
    }


def physical_ids(path: Path, quantity: str, unit: str) -> tuple[str, str]:
    """Read identities from the fixture's authoritative physical document."""
    # The physical fixtures use JSON-formatted YAML with a comment header.
    content = "\n".join(
        line for line in path.read_text().splitlines() if not line.startswith("#")
    )
    document = msgspec.json.decode(content, type=dict[str, object])
    quantities = msgspec.convert(
        document["quantity_types"], type=list[dict[str, object]]
    )
    units = msgspec.convert(document["units"], type=list[dict[str, object]])
    # Physical documents may omit a quantity type's optional public name. The
    # relational row represents that unnamed semantic type with a null name.
    typed_quantities = codec.structure_rows(
        [{**row, "name": row.get("name")} for row in quantities],
        ReferenceQuantityTypesRow,
    )
    quantity_id = next(
        row.quantity_type_id.to_hex()
        for row in typed_quantities
        if row.name == quantity
    )
    unit_id = next(row["unit_id"] for row in units if row["symbol"] == unit)
    assert isinstance(quantity_id, str)
    assert isinstance(unit_id, str)
    return quantity_id, unit_id


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
    return StudyRequest(version=2, points=points)
