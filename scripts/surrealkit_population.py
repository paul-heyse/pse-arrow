# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Named, deterministic database specimens, never scientific result producers.

The rows exercise generated storage contracts. Opaque bytes explicitly identify
synthetic fixture evidence; they are not admitted project/scientific documents.
Changing this population's logical content requires a new versioned name.
"""

from __future__ import annotations

import base64
import hashlib
import json

NAME = "readiness-graph-v2"
DATABASE = "representative_v2"
READINESS = (
    "SELECT VALUE ordinal FROM canonical_study_readiness {hint} "
    "WHERE study='fixture_alpha' AND eligible=true AND ordinal>=4dec "
    "ORDER BY ordinal LIMIT 4"
)
READINESS_ANSWER = ["4", "6", "10", "12"]


def rows(name: str = NAME) -> dict[str, list[dict[str, object]]]:
    if name != NAME:
        raise ValueError("Unknown named population: " + name)
    result: dict[str, list[dict[str, object]]] = {
        "canonical_projects": [],
        "canonical_problems": [],
        "canonical_revisions": [],
        "canonical_project_branches": [],
        "canonical_project_contexts": [],
        "canonical_study_points": [],
        "canonical_study_readiness": [],
        "canonical_analysis_nodes": [],
        "canonical_analysis_edges": [],
    }
    for project, revisions in (("alpha", 3), ("beta", 2), ("gamma", 1)):
        problem = "fixture_" + project
        result["canonical_projects"].append(
            {
                "key": problem,
                "title": "Synthetic fixture " + project,
                "deleted": False,
                "generation": revisions,
            }
        )
        result["canonical_problems"].append(
            {
                "key": problem,
                "head": problem + "_r" + str(revisions - 1),
                "sequence": revisions - 1,
            }
        )
        result["canonical_project_branches"].append(
            {
                "key": problem,
                "project": problem,
                "problem": problem,
                "name": "main",
                "origin_revision": problem + "_r0",
                "generation": revisions,
            }
        )
        for revision in range(revisions):
            key = problem + "_r" + str(revision)
            result["canonical_revisions"].append(
                {
                    "key": key,
                    "problem": problem,
                    "sequence": revision,
                    "parent": None
                    if revision == 0
                    else problem + "_r" + str(revision - 1),
                    "operation": "synthetic-fixture-" + key,
                    "request": b"fixture-only-v1",
                    "interpretation": "unactivated-fixture",
                }
            )
            result["canonical_project_contexts"].append(
                {
                    "key": key,
                    "project": problem,
                    "scope": "fixture",
                    "revision": key,
                    "source_identity": "fixture-source-v1",
                    "physical_context": "fixture",
                    "physical_revision": "fixture-r0",
                    "physical_source_identity": "fixture-physical-v1",
                    "effective_from_us": revision,
                    "effective_to_us": None,
                    "recorded_sequence": revision,
                    "corrects": None,
                    "deleted": False,
                    "request": b"fixture-only-v1",
                }
            )
    # Uneven project and eligibility distributions expose filter/order mistakes.
    for study, count in (
        ("fixture_alpha", 19),
        ("fixture_beta", 5),
        ("fixture_gamma", 2),
    ):
        for ordinal in range(count):
            key = study + "_p" + str(ordinal).zfill(2)
            eligible = ordinal % 2 == 0 and ordinal % 5 != 3
            scenario = ("accepted", "refused", "unobserved")[ordinal % 3]
            marker = json.dumps(
                {"synthetic_fixture_evidence": scenario}, sort_keys=True
            ).encode()
            result["canonical_study_points"].append(
                {
                    "key": key,
                    "study": study,
                    "ordinal": ordinal,
                    "occurrence": ordinal,
                    "seed_solution": None,
                    "policy": b"fixture-only-v1",
                    "descriptor": b"fixture-only-v1",
                    "admission": None,
                    "facts": marker,
                    "outcome": None if scenario == "unobserved" else marker,
                    "run": key + "_unexecuted",
                    "revision": ordinal % 3,
                    "assigned": False,
                    "settled": scenario != "unobserved",
                    "attempt": None,
                    "start": None,
                }
            )
            result["canonical_study_readiness"].append(
                {
                    "key": key,
                    "point": key,
                    "study": study,
                    "ordinal": ordinal,
                    "source_revision": ordinal % 3,
                    "pending": 0 if eligible else 1,
                    "eligible": eligible,
                    "premises": None,
                    "dirty": not eligible,
                    "notify": False,
                    "notify_revision": 0,
                    "notify_after": "",
                }
            )
    for n in range(10):
        result["canonical_analysis_nodes"].append(
            {
                "key": "fixture_n" + str(n),
                "analysis": "fixture_unexecuted",
                "semantic": "synthetic-fixture",
                "kind": "fixture",
            }
        )
    # A hub with seven outgoing relations, a small chain and an isolated node.
    for n, (source, target) in enumerate([(0, n) for n in range(1, 8)] + [(7, 8)]):
        result["canonical_analysis_edges"].append(
            {
                "key": "fixture_e" + str(n),
                "analysis": "fixture_unexecuted",
                "source": "fixture_n" + str(source),
                "target": "fixture_n" + str(target),
                "kind": "synthetic-fixture",
                "evidence_artifact": None,
                "evidence_block": None,
                "evidence_row": None,
                "evidence_field": None,
            }
        )
    return result


DECIMALS = frozenset(
    {
        "generation",
        "sequence",
        "recorded_sequence",
        "ordinal",
        "occurrence",
        "revision",
        "source_revision",
        "pending",
        "notify_revision",
        "evidence_block",
        "evidence_row",
    }
)


def literal(field: str, value: object) -> str:
    if value is None:
        return "NONE"
    if isinstance(value, bytes):
        return (
            "encoding::base64::decode("
            + json.dumps(base64.b64encode(value).decode())
            + ")"
        )
    if field in DECIMALS and isinstance(value, int) and not isinstance(value, bool):
        return str(value) + "dec"
    return json.dumps(value)


def sql(name: str = NAME) -> str:
    statements = ["BEGIN TRANSACTION;"]
    for table, records in rows(name).items():
        for row in records:
            target = (
                "type::record(" + json.dumps(table) + "," + json.dumps(row["key"]) + ")"
            )
            if table == "canonical_analysis_edges":
                source = "canonical_analysis_nodes:" + str(row["source"])
                dest = "canonical_analysis_nodes:" + str(row["target"])
                command = (
                    "RELATE "
                    + source
                    + "->canonical_analysis_edges:"
                    + str(row["key"])
                    + "->"
                    + dest
                )
            else:
                command = "CREATE " + target
            statements.append(
                command
                + " SET "
                + ",".join(
                    field + "=" + literal(field, value) for field, value in row.items()
                )
                + ";"
            )
    statements.append("COMMIT TRANSACTION;")
    return " ".join(statements)


def identity(name: str = NAME) -> str:
    return hashlib.sha256(sql(name).encode()).hexdigest()


def checks(name: str = NAME) -> list[tuple[str, list[dict[str, object]]]]:
    """Independent ordered readbacks include every declared field and endpoint.

    JSON normalizations make decimal, byte, NONE and record identities explicit.
    Selecting the whole table also detects unexpected/missing rows.
    """
    result: list[tuple[str, list[dict[str, object]]]] = []
    for table, records in rows(name).items():
        expected: list[dict[str, object]] = []
        columns = []
        for field in records[0]:
            value = next(
                (row[field] for row in records if row[field] is not None), None
            )
            expr = field
            if (
                field in DECIMALS
                and isinstance(value, int)
                and not isinstance(value, bool)
            ):
                expr = "<string>" + field
            elif isinstance(value, bytes):
                expr = "encoding::base64::encode(" + field + ")"
            columns.append(
                "IF " + field + "=NONE {NULL} ELSE {" + expr + "} AS " + field
            )
        if table == "canonical_analysis_edges":
            columns.extend(
                ("record::id(in) AS source_record", "record::id(out) AS target_record")
            )
        for row in sorted(records, key=lambda item: str(item["key"])):
            normalized: dict[str, object] = {}
            for field, value in row.items():
                normalized[field] = (
                    None
                    if value is None
                    else str(value)
                    if field in DECIMALS
                    and isinstance(value, int)
                    and not isinstance(value, bool)
                    else base64.b64encode(value).decode().rstrip("=")
                    if isinstance(value, bytes)
                    else value
                )
            if table == "canonical_analysis_edges":
                normalized.update(
                    source_record=row["source"], target_record=row["target"]
                )
            expected.append(normalized)
        result.append(
            (
                "SELECT " + ",".join(columns) + " FROM " + table + " ORDER BY key;",  # noqa: S608 -- identifiers/expressions come only from the fixed generated fixture vocabulary
                expected,
            )
        )
    result.append(("SELECT * FROM canonical_interpretations;", []))
    return result
