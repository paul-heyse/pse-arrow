# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Read-only integration assistance from explicit claims and existing source owners."""

from __future__ import annotations

import hashlib
import re
import tomllib
from itertools import pairwise
from typing import TYPE_CHECKING, Annotated, Literal

from fastmcp.tools import Tool
from mcp.types import ToolAnnotations
from pydantic import Field, TypeAdapter, model_validator

from scripts import (
    document_lifecycle,
    document_metadata,
    workspace_sources,
    workspace_tasks,
)
from scripts.capabilities import read_contracts
from scripts.capabilities.model import Capability, Contract
from scripts.capabilities.responses import owner_read

if TYPE_CHECKING:
    from pathlib import Path


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
    qualification: str = "Proposed checks only; dependency reach and test success are not established by this view."


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


def _tests(root: Path, paths: list[str]) -> TestScope:
    seeds = set()
    routes = []
    unknowns = [
        "Feature-dependent, dynamic, cross-language and reverse-dependency coverage remains with the explicit affected/test owners; no compiler or test discovery was started."
    ]
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
        if path.endswith((".rs", "Cargo.toml")):
            parent = (root / path).parent
            while parent != root and parent.is_relative_to(root):
                manifest = parent / "Cargo.toml"
                if manifest.is_file():
                    try:
                        fact = workspace_sources.source(
                            manifest.relative_to(root).as_posix(), root=root
                        )
                        package = (
                            tomllib.loads(fact["text"]).get("package", {}).get("name")
                        )
                        if isinstance(package, str):
                            seeds.add(package)
                            routes.append(["just", "unit-package", package])
                    except (OSError, ValueError, UnicodeError):
                        unknowns.append(
                            "Selected package manifest unavailable; use the affected owner explicitly."
                        )
                    break
                parent = parent.parent
    if paths:
        routes.append(["just", "affected", "--json", "--base", "HEAD"])
    return TestScope(
        changed_owned_paths=paths,
        package_seeds=sorted(seeds),
        routes=[list(item) for item in dict.fromkeys(tuple(route) for route in routes)],
        unknowns=unknowns,
    )


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
    owned_paths = [
        change.path
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
        test_scope=_tests(root, owned_paths),
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
        effects="Read-only source/Git/producer and explicitly selected native status/lifecycle observations; no compiler, preparation, staging or mutation.",
    )
    tool.tags.update(capability.tags)
    return (capability,), {tool.name: tool}
