# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Read-only document inventory and native, concurrently owned plan-scope queries."""

from __future__ import annotations

import argparse
import datetime
import fnmatch
import hashlib
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING, TypedDict

from scripts import document_metadata as metadata

if TYPE_CHECKING:
    from markdown_it import MarkdownIt

ROOT = Path(__file__).resolve().parents[1]


@dataclass(frozen=True)
class NativeTable:
    heading: str
    headers: tuple[str, ...]
    rows: tuple[tuple[str, ...], ...]
    line: int
    row_lines: tuple[int, ...]
    arity_errors: tuple[int, ...] = ()


def parser() -> MarkdownIt:
    try:
        from markdown_it import (  # noqa: PLC0415 -- cold bootstrap without docs dependencies
            MarkdownIt,
        )
    except ImportError as error:
        raise ValueError("Markdown parser missing; run just bootstrap-docs") from error
    return MarkdownIt("commonmark").enable("table")


def _plain(content: str) -> str:
    tokens = parser().parseInline(content)
    return "".join(
        child.content
        for token in tokens
        for child in token.children or []
        if child.type in {"text", "code_inline"}
    )


def tables(body: str, *, strict: bool = True) -> list[NativeTable]:
    # The pinned GFM rule owns escaped-pipe semantics; its public tokens own structure.
    from markdown_it.rules_block.table import (  # noqa: PLC0415 -- docs-only parser boundary
        escapedSplit,
    )

    tokens = parser().parse(body)
    lines = body.splitlines()
    heading = ""
    result = []
    index = 0
    while index < len(tokens):
        token = tokens[index]
        if token.type == "heading_open":
            heading = tokens[index + 1].content
        if token.type != "table_open":
            index += 1
            continue
        start = token.map[0] if token.map else 0
        rows = []
        row_lines = []
        arity_errors = []
        current = []
        source_line = start
        index += 1
        while tokens[index].type != "table_close":
            token = tokens[index]
            if token.type == "tr_open":
                current = []
                source_line = token.map[0] if token.map else start
            elif token.type == "inline":
                current.append(token.content)
            elif token.type == "tr_close":
                raw = lines[source_line].strip()
                raw_cells = escapedSplit(raw)
                if raw.startswith("|"):
                    raw_cells = raw_cells[1:]
                if raw.endswith("|") and raw_cells[-1] == "":
                    raw_cells = raw_cells[:-1]
                if len(raw_cells) != len(current):
                    if strict:
                        raise ValueError(
                            f"table at line {source_line + 1}: row arity differs from header; escape literal pipes"
                        )
                    arity_errors.append(source_line + 1)
                rows.append(tuple(current))
                row_lines.append(source_line + 1)
            index += 1
        result.append(
            NativeTable(
                heading,
                tuple(_plain(cell) for cell in rows[0]),
                tuple(rows[1:]),
                start + 1,
                tuple(row_lines[1:]),
                tuple(arity_errors),
            )
        )
        index += 1
    return result


def _state(raw: str, profile: dict) -> str:
    for state in profile.get("order", []):
        if any(re.search(pattern, raw) for pattern in profile.get(state, [])):
            return state
    return "unknown"


class ScopeItem(TypedDict, total=False):
    id: str
    native_id: str | None
    label: str
    document: str
    binding: str | None
    kind: str
    source: str
    source_line: int
    raw_status: str
    state: str
    native_state: str
    status_owner: str
    status_basis: str
    owner: str
    native_cells: dict[str, str]
    relationships: list[dict[str, str]]


def aggregate(root: Path, policy: dict) -> list[ScopeItem]:
    result: list[ScopeItem] = []
    seen = set()
    cache = {}
    bound_documents = set()
    for binding in policy.get("scope_tables", []):
        path = root / binding["path"]
        if not path.resolve().is_relative_to((root / "docs/plans").resolve()):
            raise ValueError("scope table must belong to retained plan content")
        if path not in cache:
            document = metadata.read(
                path.read_bytes(), binding["path"], identity=binding["path"], root=root
            )
            cache[path] = (
                document,
                tables(document.body.decode("utf-8"), strict=False),
            )
        document, candidates = cache[path]
        matching = [
            table
            for table in candidates
            if table.heading == binding["heading"]
            and list(table.headers) == binding["headers"]
        ]
        occurrence = binding.get("occurrence")
        if (occurrence is None and len(matching) != 1) or (
            occurrence is not None
            and (
                not isinstance(occurrence, int)
                or occurrence < 1
                or occurrence > len(matching)
            )
        ):
            raise ValueError(
                f"{binding['path']}::{binding['name']}: scope selector must select exactly one native table"
            )
        table = matching[occurrence - 1] if occurrence is not None else matching[0]
        if table.arity_errors:
            raise ValueError(
                f"{binding['path']}::{binding['name']}: bound table row arity differs at lines {table.arity_errors}; escape literal pipes"
            )
        profile = policy.get("status_profiles", {}).get(
            binding.get("status_profile", ""), {}
        )
        offset = len(document.prefix.splitlines())
        for row, line in zip(table.rows, table.row_lines, strict=True):
            label = _plain(row[binding["id_column"]])
            match = re.search(binding.get("id_pattern", r"^(.+)$"), label)
            if match is None:
                raise ValueError(
                    f"{binding['path']}:{line}: native ID does not match binding"
                )
            native_id = match.group(1)
            identity = f"{binding['path']}::{binding['name']}::{native_id}"
            if identity in seen:
                raise ValueError(f"duplicate native scope ID: {identity}")
            seen.add(identity)
            status_column = binding.get("status_column")
            raw = row[status_column] if status_column is not None else ""
            owner_column = binding.get("owner_column")
            result.append(
                {
                    "id": identity,
                    "native_id": native_id,
                    "label": label,
                    "document": binding["path"],
                    "binding": binding["name"],
                    "kind": binding["kind"],
                    "source": binding["path"]
                    + "#"
                    + re.sub(
                        r"\s+", "-", re.sub(r"[^\w\s-]", "", table.heading.lower())
                    ),
                    "source_line": line + offset,
                    "raw_status": raw,
                    "state": _state(_plain(raw), profile),
                    "status_basis": "bound native column"
                    if status_column is not None
                    else "no authoritative status column",
                    "owner": row[owner_column]
                    if owner_column is not None
                    else binding["path"],
                    "native_cells": dict(zip(table.headers, row, strict=True)),
                    "relationships": [],
                }
            )
        bound_documents.add(binding["path"])
    by_id = {item["id"]: item for item in result}
    status_owners = {}
    for relationship in policy.get("relationships", []):
        source, target = relationship["from"], relationship["to"]
        if source not in by_id or target not in by_id:
            raise ValueError(
                f"scope relationship names an unknown native item: {source} -> {target}"
            )
        if relationship["kind"] not in {
            "depends-on",
            "superseded-by",
            "status-owned-by",
        }:
            raise ValueError("unsupported scope relationship kind")
        if source == target:
            raise ValueError("scope relationship cannot point to itself")
        by_id[source]["relationships"].append(
            {"kind": relationship["kind"], "target": target}
        )
        if relationship["kind"] == "status-owned-by":
            if source in status_owners and status_owners[source] != target:
                raise ValueError(f"conflicting canonical scope status owners: {source}")
            status_owners[source] = target

    def canonical(identity: str, visited: set[str]) -> str:
        if identity in visited:
            raise ValueError(f"cyclic canonical scope status ownership: {identity}")
        if identity not in status_owners:
            return identity
        return canonical(status_owners[identity], visited | {identity})

    for item in result:
        item["native_state"] = item["state"]
        item["status_owner"] = canonical(item["id"], set())
    for item in result:
        if item["status_owner"] != item["id"]:
            item["state"] = by_id[item["status_owner"]]["native_state"]
            item["status_basis"] = (
                "canonical owner through explicit status-owned-by relationship"
            )
    for path in sorted((root / "docs/plans").glob("*.md")):
        if (
            re.match(r"(?:2[89]|[3-9][0-9])[a-z]?-", path.name)
            and path.relative_to(root).as_posix() not in bound_documents
        ):
            result.append(
                {
                    "id": path.relative_to(root).as_posix(),
                    "document": path.relative_to(root).as_posix(),
                    "binding": None,
                    "native_id": None,
                    "kind": "unbound-document",
                    "state": "unknown",
                    "raw_status": "",
                    "status_basis": "no configured authoritative native table",
                    "relationships": [],
                }
            )
    return sorted(result, key=lambda item: item["id"])


def inventory(root: Path, policy: dict) -> tuple[list[dict], list[str]]:
    from scripts import (  # noqa: PLC0415 -- lifecycle/publication import cycle
        docs,
    )

    site = docs.configuration(root)
    pages = docs.discover(root, site)
    records = []
    errors = []
    for page in pages:
        path = root / "docs" / page.path
        try:
            raw = path.read_bytes()
            document = metadata.read(
                raw,
                str(path.relative_to(root)),
                identity=path.relative_to(root).as_posix(),
                root=root,
            )
            interpreted = metadata.interpreted(root, path, document, policy, site)
            records.append(
                {
                    "path": path.relative_to(root).as_posix(),
                    "scope": page.scope,
                    "native_status": document.metadata.get("status"),
                    "native_decision": document.metadata.get("decision"),
                    "sha256": hashlib.sha256(raw).hexdigest(),
                    "apparent_bytes": len(raw),
                    **interpreted,
                }
            )
        except (ValueError, OSError) as error:
            errors.append(str(error))
    return records, errors


def snapshot(root: Path, command: str, policy: dict) -> dict:
    observed = datetime.datetime.now(datetime.UTC).isoformat()
    if command == "scope":
        return {
            "version": 1,
            "observed_at": observed,
            "operation": command,
            "authority": "derived native status; no scheduling or authorization",
            "items": aggregate(root, policy),
        }
    records, errors = inventory(root, policy)
    result = {
        "version": 1,
        "observed_at": observed,
        "operation": command,
        "documents": records,
        "errors": errors,
        "exclusions": [
            "production and test bodies",
            ".venv*",
            "build and target payloads",
            "persistent state",
            "shared external resources",
        ],
        "deletion_authorized": False,
    }
    if command == "validate":
        try:
            items = aggregate(root, policy)
            result["scope_items_validated"] = len(items)
        except (ValueError, OSError) as error:
            errors.append(str(error))
    if command == "retire-plan":
        references = {}
        for record in records:
            text = (root / record["path"]).read_text(encoding="utf-8")
            for target in re.findall(r"\]\(([^)]+)\)", text):
                if "://" in target or target.startswith("#"):
                    continue
                candidate = (
                    (root / record["path"]).parent / target.split("#", 1)[0]
                ).resolve()
                if candidate.is_relative_to(root.resolve()):
                    references.setdefault(
                        candidate.relative_to(root.resolve()).as_posix(), set()
                    ).add(record["path"])
        result["candidates"] = [
            {
                "path": record["path"],
                "surviving_meaning_owner": record.get("doc_owner"),
                "references_to_repair": sorted(references.get(record["path"], set())),
                "blockers": [
                    "owner release not established",
                    *(
                        ["unknown canonical owner"]
                        if not record.get("doc_owner")
                        else []
                    ),
                ],
                "native_status": record["native_status"],
            }
            for record in records
            if record.get("doc_role") == "plan"
        ]
    return result


def main(argv: list[str] | None = None) -> int:
    arguments = argparse.ArgumentParser(description=__doc__)
    arguments.add_argument(
        "command", choices=("inventory", "validate", "retire-plan", "scope")
    )
    arguments.add_argument("--path", default="*")
    arguments.add_argument("--role")
    arguments.add_argument("--topic")
    arguments.add_argument("--scope", choices=("Current", "Reference", "History"))
    arguments.add_argument(
        "--state", choices=("open", "blocked", "deferred", "complete", "unknown")
    )
    args = arguments.parse_args(argv)
    if args.command == "scope" and any((args.role, args.topic, args.scope)):
        arguments.error(
            "scope query supports --path/--state; --role/--topic/--scope select documents"
        )
    if args.command != "scope" and args.state is not None:
        arguments.error("--state selects native scope items; use the scope command")
    try:
        result = snapshot(ROOT, args.command, metadata.load_policy(ROOT))
        if "items" in result:
            result["items"] = [
                item
                for item in result["items"]
                if fnmatch.fnmatchcase(item["document"], args.path)
                and (args.state is None or item["state"] == args.state)
            ]
            result["counts"] = {
                state: sum(item["state"] == state for item in result["items"])
                for state in ("open", "blocked", "deferred", "complete", "unknown")
            }
        else:
            result["documents"] = [
                record
                for record in result["documents"]
                if fnmatch.fnmatchcase(record["path"], args.path)
                and (args.role is None or record.get("doc_role") == args.role)
                and (args.topic is None or args.topic in record.get("doc_topics", []))
                and (args.scope is None or record["scope"] == args.scope)
            ]
            if "candidates" in result:
                selected = {record["path"] for record in result["documents"]}
                result["candidates"] = [
                    candidate
                    for candidate in result["candidates"]
                    if candidate["path"] in selected
                ]
        print(json.dumps(result, indent=2, sort_keys=True))
        return 1 if result.get("errors") else 0
    except (ValueError, OSError) as error:
        print(f"document-lifecycle: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
