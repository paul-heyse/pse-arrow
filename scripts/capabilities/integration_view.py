# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Read-only integration assistance from explicit claims and existing source owners."""

from __future__ import annotations

import hashlib
import json
import re
from itertools import pairwise
from pathlib import Path
from typing import Annotated, Literal

from fastmcp.tools import Tool
from mcp.types import ToolAnnotations
from pydantic import Field, TypeAdapter, model_validator

from scripts import (
    affected,
    document_lifecycle,
    document_metadata,
    test_invocation,
    workspace_sources,
    workspace_tasks,
)
from scripts.capabilities import read_contracts
from scripts.capabilities.model import Capability, Contract
from scripts.capabilities.responses import owner_read

Digest = Annotated[str, Field(pattern=r"^[a-f0-9]{64}$")]
SourcePath = Annotated[str, Field(min_length=1, max_length=1024)]
Owner = Annotated[str, Field(min_length=1, max_length=160)]
OwnershipState = Literal["owned", "unknown", "unrelated"]
PathState = Literal["owned", "unknown", "unrelated", "mixed"]


class Lines(Contract):
    start_line: int = Field(ge=1)
    end_line: int = Field(ge=1)

    @model_validator(mode="after")
    def ordered(self) -> Lines:
        if self.end_line < self.start_line:
            raise ValueError("line range must be ordered")
        return self


class OwnershipClaim(Contract):
    path: SourcePath
    owner: Owner
    source_sha256: Digest | None
    diff_sha256: Digest | None = None
    hunks: list[Lines] = Field(default_factory=list, max_length=32)
    explanation: str = Field(min_length=1, max_length=1024)


class IntegrationRequest(Contract):
    task: Owner
    claims: list[OwnershipClaim] = Field(default_factory=list, max_length=64)
    native_task: str | None = Field(default=None, min_length=1, max_length=1024)
    outcomes: list[str] = Field(default_factory=list, max_length=16)
    retiring_paths: list[SourcePath] = Field(default_factory=list, max_length=16)
    offset: int = Field(default=0, ge=0)
    limit: int = Field(default=40, ge=1, le=100)
    features: str = Field(default="", max_length=2048, pattern=r"^[A-Za-z0-9_./, -]*$")
    validation_evidence: list[read_contracts.OwnerReference] = Field(
        default_factory=list, max_length=16
    )
    validation_profile: str | None = Field(default=None, min_length=1, max_length=160)
    validation_mode: str | None = Field(default=None, min_length=1, max_length=160)

    @model_validator(mode="after")
    def bounded_references(self) -> IntegrationRequest:
        if any(not value or len(value) > 2048 for value in self.outcomes):
            raise ValueError("outcome references must be bounded")
        return self


class ClaimedRange(Contract):
    lines: Lines | None
    classification: OwnershipState
    owners: list[str]
    reason: str


class ChangedPath(Contract):
    path: str
    git_status: str
    source_sha256: str | None = None
    diff_sha256: str | None = None
    classification: PathState
    claims: list[OwnershipClaim]
    ranges: list[ClaimedRange]
    conflicts: list[str]
    generated: bool | None


class CommitSelection(Contract):
    path: str
    source_sha256: str | None
    diff_sha256: str | None
    selection: Literal["whole-path", "selected-hunks", "excluded"]
    hunks: list[Lines]
    reason: str


class TestScope(Contract):
    owner: str = "scripts/affected.py"
    changed_owned_paths: list[str]
    package_seeds: list[str]
    routes: list[list[str]]
    unknowns: list[str]
    selected_packages: list[str] = Field(default_factory=list)
    workspace_widening: list[str] = Field(default_factory=list)
    scientific_inputs: read_contracts.OwnerRecord | None = None
    exclusions: list[read_contracts.OwnerRecord] = Field(default_factory=list)
    selection_basis: str = "unknown"
    source_cut: dict[str, str | None] = Field(default_factory=dict)
    validation_evidence: list[read_contracts.OwnerRecord] = Field(default_factory=list)
    qualification: str = "Proposed dependency-reach checks only; no test execution, runtime admission or completeness certification."


class IntegrationPayload(Contract):
    task: str
    checkout: str
    revision: str
    ownership_basis: str = "Explicit caller claims bound to current source bytes; claims do not prove authorship. Git state never grants task ownership."
    changes: list[ChangedPath]
    total_changes: int
    next_offset: int | None
    relationships: read_contracts.RelationshipsPayload | None
    changed_consumers: list[str]
    regeneration_routes: list[list[str]]
    test_scope: TestScope
    native_task: read_contracts.TaskContextPayload | None
    retirement_preflight: list[read_contracts.OwnerRecord]
    outcome_references: list[str]
    commit_selection: list[CommitSelection]
    remaining_verification: list[str]
    effects: Literal[
        "read-only; no staging, merge, deletion, retirement, execution or completion certification"
    ] = "read-only; no staging, merge, deletion, retirement, execution or completion certification"


class IntegrationResult(read_contracts.ReadAnswer):
    operation: Literal["workspace_integration_view"] = "workspace_integration_view"
    complete: bool
    omissions: list[str]
    answer: read_contracts.AnswerState
    payload: IntegrationPayload


ANSWER = TypeAdapter(IntegrationResult | read_contracts.DetailRefused)


def _git(root: Path, *arguments: str, maximum: int = 4 << 20) -> bytes:
    result = workspace_sources._command(  # noqa: SLF001 -- reuse the original bounded source owner
        root, ["git", *arguments], seconds=5, max_bytes=maximum
    )
    if not result["complete"] or result["returncode"]:
        raise ValueError("selected Git observation unavailable or exceeds its bound")
    return result["stdout"]


def _span(claim: OwnershipClaim, start: int, end: int) -> bool:
    return not claim.hunks or any(
        item.start_line <= start and item.end_line >= end for item in claim.hunks
    )


def _change(
    root: Path, status: str, path: str, claims: list[OwnershipClaim], task: str
) -> ChangedPath:
    conflicts = []
    source_digest = None
    diff_digest = None
    hunks: list[tuple[int, int, bool]] = []
    valid: list[OwnershipClaim] = []
    if claims:
        try:
            selected = workspace_sources.selected_path(root, path, allow_missing=True)
            raw = (
                workspace_sources._read(root, path, 4 << 20)  # noqa: SLF001 -- reuse the original bounded source owner
                if selected.exists()
                else None
            )
            source_digest = hashlib.sha256(raw).hexdigest() if raw is not None else None
            difference = _git(
                root,
                "--literal-pathspecs",
                "diff",
                "--no-ext-diff",
                "--no-textconv",
                "--no-renames",
                "--unified=0",
                "HEAD",
                "--",
                path,
                maximum=1 << 20,
            )
            diff_digest = hashlib.sha256(difference).hexdigest()
            for match in re.finditer(
                rb"^@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? @@", difference, re.MULTILINE
            ):
                start, count = int(match[1]), int(match[2]) if match[2] else 1
                hunks.append(
                    (max(1, start), max(1, start) + max(1, count) - 1, count == 0)
                )
            line_count = max(1, len(raw.splitlines())) if raw is not None else 0
            if status == "??":
                hunks = [(1, max(1, line_count), False)]
            for claim in claims:
                if (
                    claim.source_sha256 != source_digest
                    or (
                        raw is None
                        and (claim.hunks or claim.diff_sha256 != diff_digest)
                    )
                    or (
                        claim.diff_sha256 is not None
                        and claim.diff_sha256 != diff_digest
                    )
                ):
                    conflicts.append(
                        "source or deletion/diff witness differs for owner "
                        + claim.owner
                    )
                elif any(item.end_line > line_count for item in claim.hunks):
                    conflicts.append(
                        "current line range unavailable for owner " + claim.owner
                    )
                else:
                    valid.append(claim)
        except (OSError, ValueError, UnicodeError):
            conflicts.append(
                "claimed source or Git difference is unavailable or unadmitted"
            )
    ranges: list[ClaimedRange] = []
    for start, end, deletion in hunks or [(1, 1, False)]:
        endpoints = {start, end + 1}
        for claim in valid:
            for lines in claim.hunks:
                endpoints.update(
                    (
                        max(start, min(end + 1, lines.start_line)),
                        max(start, min(end + 1, lines.end_line + 1)),
                    )
                )
        points = sorted(endpoints)
        for lower, upper in pairwise(points):
            selected = [
                claim
                for claim in valid
                if _span(claim, lower, upper - 1)
                and (not deletion or not claim.hunks)
                and (bool(hunks) or not claim.hunks)
            ]
            owners = sorted({claim.owner for claim in selected})
            range_classification: OwnershipState
            if len(owners) > 1:
                range_classification = "unknown"
                reason = "conflicting explicit owners overlap this change"
                conflicts.append(
                    f"overlapping owners at current lines {lower}-{upper - 1}: "
                    + ", ".join(owners)
                )
            elif not owners or conflicts:
                range_classification = "unknown"
                reason = "ownership or current source witness is unestablished"
            else:
                range_classification = "owned" if owners == [task] else "unrelated"
                reason = (
                    "matching explicit task claim"
                    if range_classification == "owned"
                    else "matching explicit other-owner claim"
                )
            ranges.append(
                ClaimedRange(
                    lines=Lines(start_line=lower, end_line=upper - 1)
                    if hunks
                    else None,
                    classification=range_classification,
                    owners=owners,
                    reason=reason,
                )
            )
    if "U" in status or status in {"AA", "DD"}:
        conflicts.append(
            "Git reports an unresolved index conflict; a clean text merge would not establish semantic acceptance"
        )
    states = {item.classification for item in ranges}
    path_classification: PathState = (
        "unknown" if conflicts else next(iter(states)) if len(states) == 1 else "mixed"
    )
    return ChangedPath(
        path=path,
        git_status=status,
        source_sha256=source_digest,
        diff_sha256=diff_digest,
        classification=path_classification,
        claims=claims,
        ranges=ranges,
        conflicts=list(dict.fromkeys(conflicts)),
        generated=None,
    )


def _tests(
    root: Path,
    changes: list[ChangedPath],
    *,
    request: IntegrationRequest,
    revision: str,
) -> TestScope:
    paths = [change.path for change in changes]
    selected = affected.preview(
        [(change.git_status, change.path) for change in changes],
        root=root,
        base=revision,
        features=request.features,
        prepare=False,
    )
    routes = []
    for path in paths:
        if path.startswith("scripts/tests/test_") and path.endswith(".py"):
            routes.append(
                [
                    "scripts/pse-env",
                    "--",
                    ".venv/bin/python",
                    "-m",
                    "unittest",
                    path[:-3].replace("/", "."),
                ]
            )
        elif path.startswith(("scripts/", "python/")) or path in {
            "pyproject.toml",
            "uv.lock",
            "conftest.py",
        }:
            routes.append(
                ["just", "setup-test"]
                if path.startswith("scripts/")
                else ["just", "py-unit"]
            )
    routes.extend(
        ["just", "unit-package", package] for package in selected["changed_packages"]
    )
    if selected["command"]:
        routes.append(selected["command"])
    if paths:
        routes.append(
            [
                "just",
                "affected",
                "--json",
                "--passive",
                "--base",
                revision,
                *(["--features", request.features] if request.features else []),
                "--changes-json",
                json.dumps(
                    [(change.git_status, change.path) for change in changes],
                    separators=(",", ":"),
                ),
            ]
        )
    unknowns = [
        "The change population is exactly the currently claimed paths, including the full file for a claimed hunk; other dirty paths are excluded.",
        "No compiler, scientific capture or test discovery was started. The execution receiver still owns actual feature/force-validation admission.",
        "Dynamic/cross-language consumers not declared by original owners remain unknown.",
        *(
            ["Cargo graph unavailable; nearest package names are source leads only."]
            if selected["graph_status"] == "unavailable"
            else []
        ),
        *(
            [str(selected["scientific_inputs"]["reason"])]
            if selected["scientific_inputs"].get("reason")
            else []
        ),
    ]
    return TestScope(
        changed_owned_paths=paths,
        package_seeds=selected["changed_packages"],
        selected_packages=selected["selected_packages"],
        workspace_widening=selected["workspace_widening"],
        scientific_inputs=read_contracts.OwnerRecord.model_validate(
            selected["scientific_inputs"]
        ),
        exclusions=[
            read_contracts.OwnerRecord.model_validate(item)
            for item in selected["exclusions"]
        ],
        selection_basis=selected["graph_status"] + "; " + selected["validation"],
        source_cut={change.path: change.source_sha256 for change in changes},
        validation_evidence=_validation_evidence(root, changes, request),
        routes=[list(item) for item in dict.fromkeys(tuple(route) for route in routes)],
        unknowns=unknowns,
    )


def _validation_evidence(
    root: Path, changes: list[ChangedPath], request: IntegrationRequest
) -> list[read_contracts.OwnerRecord]:
    if not request.validation_evidence:
        return []
    from scripts import (  # noqa: PLC0415 -- explicit original evidence only
        run_inspection,
        validation,
        validation_result,
    )

    results = []
    current_stamps = {
        change.path: validation.source_stamp(root / change.path) for change in changes
    }
    for reference in request.validation_evidence:
        value: dict[str, object] = {
            "reference": reference.model_dump(mode="json"),
            "applicability": "unknown",
            "qualification": "Historical evidence only; no automatic reuse or narrowed test selection.",
        }
        try:
            if isinstance(reference, read_contracts.AssessmentReference):
                directory = Path(reference.directory)
                if not directory.is_absolute():
                    directory = root / directory
                observed = validation_result.read_assessment(
                    directory,
                    gate=reference.gate,
                    source_detail=reference.gate is not None,
                    full_export=False,
                )
                original = [item["record"] for item in observed["checks"]]
                if reference.gate is None:
                    value.update(
                        availability="available",
                        execution_explanation=observed.get("execution_explanation"),
                        reason="Select an exact original assessment gate before hydrating source/population context.",
                    )
                    results.append(read_contracts.OwnerRecord.model_validate(value))
                    continue
                mismatches = []
                missing = []
                contexts = []
                for record in original:
                    inputs = record.get("inputs", {})
                    files = inputs.get("files", {}) if isinstance(inputs, dict) else {}
                    for change in changes:
                        actual = current_stamps[change.path]
                        expected = (
                            files.get(change.path) if isinstance(files, dict) else None
                        )
                        if expected is None:
                            missing.append(change.path)
                        elif expected != actual:
                            mismatches.append(change.path)
                    if (
                        request.validation_profile is not None
                        and record.get("profile") != request.validation_profile
                    ):
                        mismatches.append("profile:" + str(record["gate"]))
                    if (
                        request.validation_mode is not None
                        and record.get("mode") != request.validation_mode
                    ):
                        mismatches.append("mode:" + str(record["gate"]))
                    if request.features:
                        invocation = record.get("invocation", {})
                        arguments = (
                            invocation.get("args")
                            if isinstance(invocation, dict)
                            else None
                        )
                        if isinstance(arguments, list) and all(
                            isinstance(arg, str) for arg in arguments
                        ):
                            try:
                                groups = test_invocation.rust(arguments).values(
                                    "--features"
                                )
                                selected_features = {
                                    feature
                                    for group in groups
                                    for feature in re.split(r"[ ,]+", group)
                                    if feature
                                }
                                requested_features = set(
                                    filter(None, re.split(r"[ ,]+", request.features))
                                )
                                if not groups:
                                    missing.append(
                                        "feature-context:" + str(record["gate"])
                                    )
                                elif selected_features != requested_features:
                                    mismatches.append("features:" + str(record["gate"]))
                            except (ValueError, TypeError):
                                missing.append("feature-context:" + str(record["gate"]))
                        else:
                            missing.append("feature-context:" + str(record["gate"]))
                    contexts.append(
                        {
                            "gate": record["gate"],
                            "mode": record.get("mode"),
                            "profile": record.get("profile"),
                            "invocation": record.get("invocation"),
                            "evidence_kind": record.get("evidence_kind"),
                            "status": record.get("status"),
                            "native_recorded": record.get("native") is not None,
                            "selected_count": len(record["selected"])
                            if isinstance(record.get("selected"), list)
                            else None,
                            "environment_keys": sorted(inputs.get("environment", {}))
                            if isinstance(inputs, dict)
                            else [],
                        }
                    )
                value.update(
                    availability="available",
                    applicability="inapplicable" if mismatches else "unknown",
                    source_mismatches=sorted(set(mismatches)),
                    missing_source_context=sorted(set(missing)),
                    original_contexts=contexts,
                    execution_explanation=observed.get("execution_explanation"),
                    reason="Current selected sources/profile/mode differ."
                    if mismatches
                    else "Selected source equality alone cannot establish feature, native environment, population and execution eligibility; inspect original context before choosing validation.",
                )
            elif isinstance(reference, read_contracts.InvocationReference):
                observed = run_inspection.product_result(
                    reference.invocation,
                    registry=Path(reference.registry) if reference.registry else None,
                    limit=1,
                )
                value.update(
                    availability="available",
                    execution_explanation=observed.get("execution_explanation"),
                    reason="Original invocation has no complete selected-source/feature comparison; its outcomes are not current acceptance.",
                )
            else:
                value.update(
                    availability="unsupported",
                    reason="Native diagnostic/result references remain with their exact original reader; no inferred validation correspondence.",
                )
        except (OSError, RuntimeError, TypeError, ValueError) as error:
            value.update(
                availability="unavailable",
                reason=type(error).__name__
                + ": original selected evidence unavailable",
            )
        results.append(read_contracts.OwnerRecord.model_validate(value))
    return results


def inspect(
    root: Path, request: IntegrationRequest
) -> IntegrationResult | read_contracts.DetailRefused:
    revision = _git(root, "rev-parse", "HEAD", maximum=128).decode().strip()
    status = _git(
        root, "status", "--porcelain=v1", "-z", "--untracked-files=all", "--no-renames"
    )
    entries = [
        (item[:2].decode(), item[3:].decode("utf-8"))
        for item in status.split(b"\0")
        if item
    ]
    selected = entries[request.offset : request.offset + request.limit]
    omissions = (
        ["Only the selected dirty-path page is shown."]
        if request.offset or len(entries) > request.limit
        else []
    )
    claims: dict[str, list[OwnershipClaim]] = {}
    for claim in request.claims:
        workspace_sources.selected_path(root, claim.path, allow_missing=True)
        claims.setdefault(claim.path, []).append(claim)
    changes = [
        _change(root, state, path, claims.get(path, []), request.task)
        for state, path in selected
    ]
    admitted = []
    for change in changes:
        try:
            workspace_sources.selected_path(root, change.path, allow_missing=True)
            admitted.append(change.path)
        except ValueError:
            pass
    relationships = None
    consumers = []
    if admitted:
        relationships = read_contracts.RelationshipsPayload.model_validate(
            workspace_sources.relationships(
                root=root, exact_paths=tuple(admitted), limit=100
            )
        )
        if relationships.truncated or relationships.availability != "available":
            omissions.append(
                "Producer relationships are unavailable or partial; dynamic consumers are never inferred."
            )
        for record in relationships.items:
            row = record.model_dump(mode="json")
            output = row.get("output_path")
            if isinstance(output, str):
                consumers.append(output)
                for index, change in enumerate(changes):
                    if change.path == output:
                        changes[index] = change.model_copy(update={"generated": True})
    proposals = []
    for change in changes:
        owned = [
            item.lines
            for item in change.ranges
            if item.classification == "owned" and item.lines is not None
        ]
        excluded = (
            change.generated
            or bool(change.conflicts)
            or change.classification in {"unknown", "unrelated"}
        )
        proposals.append(
            CommitSelection(
                path=change.path,
                source_sha256=change.source_sha256,
                diff_sha256=change.diff_sha256,
                selection="excluded"
                if excluded
                else "whole-path"
                if change.classification == "owned"
                and any(
                    not claim.hunks and claim.owner == request.task
                    for claim in change.claims
                )
                else "selected-hunks",
                hunks=[] if excluded else owned,
                reason="Generated consumer: edit its producer and explicitly regenerate before reviewing inclusion."
                if change.generated
                else "Unestablished, conflicting or explicitly unrelated ownership is excluded."
                if excluded
                else "Review only this explicitly claimed selection against its source/diff witnesses; nothing was staged.",
            )
        )
    native = (
        read_contracts.TaskContextPayload.model_validate(
            workspace_tasks.task_packet(
                request.native_task, (), root=root, evidence=tuple(request.outcomes)
            )
        )
        if request.native_task
        else None
    )
    if native is not None and not native.population_complete:
        omissions.append(
            "Selected native task context is partial; its original omissions and status owners are retained."
        )
    retirement = []
    if request.retiring_paths:
        for path in request.retiring_paths:
            workspace_sources.selected_path(root, path, allow_missing=True)
        snapshot = document_lifecycle.snapshot(
            root, "retire-plan", document_metadata.load_policy(root)
        )
        retirement = [
            read_contracts.OwnerRecord.model_validate(item)
            for item in snapshot["candidates"]
            if item["path"] in request.retiring_paths
        ]
        if snapshot["errors"] or len(retirement) != len(set(request.retiring_paths)):
            omissions.append(
                "Selected lifecycle preflight has unavailable inputs or candidates; retirement remains unestablished."
            )
    owned_changes = [
        change
        for change in changes
        if any(item.classification == "owned" for item in change.ranges)
        and not change.conflicts
    ]
    payload = IntegrationPayload(
        task=request.task,
        checkout=str(root.resolve()),
        revision=revision,
        changes=changes,
        total_changes=len(entries),
        next_offset=request.offset + request.limit
        if request.offset + request.limit < len(entries)
        else None,
        relationships=relationships,
        changed_consumers=sorted(set(consumers)),
        regeneration_routes=[["just", "codegen"], ["just", "codegen-check"]]
        if relationships and relationships.items
        else [],
        test_scope=_tests(root, owned_changes, request=request, revision=revision),
        native_task=native,
        retirement_preflight=retirement,
        outcome_references=request.outcomes,
        commit_selection=proposals,
        remaining_verification=[
            "Resolve unknown/overlapping ownership and review the selected source/diff witnesses immediately before any staging.",
            "Explicitly run required generation and the affected owner's selected functional checks; no outcomes were executed or certified here.",
            "Run scope-end verification at the owning plan's boundary; selected outcome references were not read and do not establish a pass.",
            "Keep native plan status, owner release and surviving references authoritative; this view grants no retirement or completion.",
        ],
    )
    for change in changes:
        if change.claims and not change.conflicts:
            selected_path = workspace_sources.selected_path(
                root, change.path, allow_missing=True
            )
            raw = (
                workspace_sources._read(root, change.path, 4 << 20)  # noqa: SLF001 -- reuse the original bounded source owner
                if selected_path.exists()
                else None
            )
            if (
                hashlib.sha256(raw).hexdigest() if raw is not None else None
            ) != change.source_sha256:
                raise ValueError(
                    "Selected source changed during integration composition"
                )
    if (
        _git(
            root,
            "status",
            "--porcelain=v1",
            "-z",
            "--untracked-files=all",
            "--no-renames",
        )
        != status
        or _git(root, "rev-parse", "HEAD", maximum=128).decode().strip() != revision
    ):
        raise ValueError("Git state changed during selected integration composition")
    return read_contracts.finish_read(
        IntegrationResult,
        payload,
        population="partial" if omissions else "complete",
        reasons=omissions,
    )


def bindings(root: Path) -> tuple[tuple[Capability, ...], dict[str, Tool]]:
    def workspace_integration_view(
        request: IntegrationRequest,
    ) -> IntegrationResult | read_contracts.DetailRefused:
        """Read a selected dirty-path page using explicit task/worker ownership claims bound to current source_sha256 and optional diff_sha256. Empty hunks explicitly claim the whole path; hunk coordinates refer to current bytes. Deletions require absence plus the current Git diff digest. Git state never establishes authorship. Preserve unknown, unrelated and overlapping claims, producer relationships, proposed checks and native lifecycle preflight. No staging, merge, generation, test execution, deletion or completion certification."""
        return owner_read(
            "workspace_integration_view",
            lambda: inspect(root, request).model_dump(mode="json"),
            project=ANSWER.validate_python,
        )

    tool = Tool.from_function(
        workspace_integration_view,
        annotations=ToolAnnotations(
            read_only_hint=True,
            destructive_hint=False,
            idempotent_hint=True,
            open_world_hint=False,
        ),
    )
    capability = Capability(
        id=tool.name,
        title="Ownership-aware integration view",
        description=tool.description or tool.name,
        domain="workspace",
        family="source-context",
        guides=("root", "tooling", "decisions"),
        owner="scripts/capabilities/integration_view.py",
        source="scripts/capabilities/integration_view.py",
        exposure="tool",
        invocation_lifetime="single",
        effects="Read-only source/Git/producer and explicitly selected native status/lifecycle observations; no compiler, preparation, staging or mutation.",
    )
    tool.tags.update(capability.tags)
    return (capability,), {tool.name: tool}
