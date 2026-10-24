# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit derived development-knowledge projection, with source-only operation."""

from __future__ import annotations

import argparse
import json
import math
import os
import re
import subprocess
import sys
import tempfile
import time
from pathlib import Path

from scripts import (
    evidence_store,
    resource_store,
    workspace_domain_contracts,
    workspace_domains,
    workspace_sources,
)

ROOT = Path(__file__).resolve().parents[1]


def corpus(
    paths: tuple[str, ...] = (),
    *,
    root: Path = ROOT,
    expected_active: str | None = None,
    references: dict | None = None,
    ranges: dict[str, tuple[int, int]] | None = None,
    concept: str | None = None,
    identity: str | None = None,
    excerpt_offset: int = 0,
    excerpt_limit: int | None = None,
) -> dict:
    if (
        excerpt_offset < 0
        or (excerpt_limit is not None and not 1 <= excerpt_limit <= 100)
        or (excerpt_offset and excerpt_limit is None)
    ):
        raise ValueError("excerpt offset requires a positive selected limit in 1..100")
    if (
        len(paths) > 64
        or len(set(paths)) != len(paths)
        or bool(paths) == bool(concept)
        or (concept and not identity)
        or (identity and not concept)
    ):
        raise ValueError(
            "select 1..64 distinct repository sources OR one exact concept identity"
        )
    projection = (
        workspace_domains.query(
            workspace_domain_contracts.parse_query(
                {"concept": concept, "identity": identity, "limit": 100}
            ),
            root=root,
        )
        if concept
        else workspace_domains.project(paths, root=root)
    )
    if not projection["complete"]:
        raise ValueError(
            "incomplete owner-native projection: " + "; ".join(projection["omissions"])
        )
    sources, excerpts = [], []
    excerpt_total = 0
    facts = projection["source_facts"]
    if concept:
        facts = [
            workspace_sources.source(
                fact["path"], root=root, expected_digest=fact["digest"]
            )
            for fact in facts
        ]
    ranges = dict(ranges or {})
    if set(ranges) - {fact["path"] for fact in facts}:
        raise ValueError("excerpt ranges must name selected source contributors")
    if concept in {"contract-section", "plan-package"}:
        selected_ids = {subject["id"] for subject in projection.get("items", [])}
        for extent in projection["section_ranges"]:
            if (
                extent["subject_id"] in selected_ids
                and extent["source_id"] not in ranges
            ):
                ranges[extent["source_id"]] = (
                    extent["start_line"],
                    min(extent["end_line"], extent["start_line"] + 239),
                )
    for fact in facts:
        source_id = fact["path"]
        lines = fact["text"].splitlines()
        start_line, end_line = ranges.get(source_id, (1, min(len(lines), 240)))
        if lines and not (
            1 <= start_line <= end_line <= len(lines) and end_line - start_line < 640
        ):
            raise ValueError(
                "selected excerpt range must belong to source and span at most 640 lines"
            )
        status = (
            "historical"
            if fact["scope"] == "History"
            else "current"
            if fact["scope"] == "Current"
            else "unknown"
        )
        sources.append(
            {
                "id": source_id,
                "kind": "source-file",
                "locator": "repo:" + source_id,
                "version_digest": fact["digest"],
                "basis": "declared",
                "title": source_id
                + " ["
                + fact["role"]
                + f"; selected lines {start_line}-{end_line}/{len(lines)}; chunks at most 16KiB]",
                "status": status,
            }
        )
        # Whole-file context belongs only to its document subject. A shared carrier
        # does not attribute arbitrary text to a section, declaration or package.
        document = next(
            (
                subject["id"]
                for subject in projection["subjects"]
                if subject["kind"] in {"document", "decision"}
                and subject["native_version"] == fact["digest"]
                and any(
                    link["source_id"] == source_id and link["role"] == "metadata"
                    for link in subject["sources"]
                )
            ),
            None,
        )
        selections = [(document, start_line, end_line)]
        selections.extend(
            (
                item["subject_id"],
                max(start_line, item["start_line"]),
                min(end_line, item["end_line"]),
            )
            for item in projection["section_ranges"]
            if item["source_id"] == source_id
        )
        for subject_id, first, last in selections:
            for start in range(first - 1, last, 40):
                selected_lines = lines[start : min(start + 40, last)]
                if not any(line.strip() for line in selected_lines):
                    continue
                index = excerpt_total
                excerpt_total += 1
                if (
                    excerpt_limit is not None
                    and not excerpt_offset <= index < excerpt_offset + excerpt_limit
                ):
                    continue
                encoded = "\n".join(selected_lines).encode("utf-8")
                text = encoded[:16000].decode("utf-8", errors="ignore")
                excerpts.append(
                    {
                        "id": f"{source_id}#{subject_id or 'unattributed'}:L{start + 1}",
                        "source_id": source_id,
                        "subject_id": subject_id,
                        "text": text,
                        "locator": f"repo:{source_id}#L{start + 1}",
                        "basis": "text",
                    }
                )
                if excerpt_limit is not None:
                    excerpts[-1]["text_complete"] = len(encoded) <= 16000
    families: dict[str, list[dict]] = {
        "sources": sources,
        "excerpts": excerpts,
        "subjects": projection["subjects"],
        "edges": projection["edges"],
        "productions": projection["productions"],
    }
    result: dict = {"version": 2, "expected_active": expected_active, **families}
    if excerpt_limit is not None:
        if references:
            raise ValueError(
                "selected excerpts cannot be combined with imported reference populations"
            )
        result["excerpt_page"] = {
            "offset": excerpt_offset,
            "limit": excerpt_limit,
            "total": excerpt_total,
            "next_offset": excerpt_offset + excerpt_limit
            if excerpt_offset + excerpt_limit < excerpt_total
            else None,
        }
    if references:
        if set(references) != {
            "sources",
            "subjects",
            "excerpts",
            "edges",
            "productions",
        }:
            raise ValueError(
                "explicit v2 references require all five typed families; payloads stay at original owners"
            )
        for family in references:
            families[family].extend(references[family])
    if not families["subjects"]:
        raise ValueError("selected sources declare no supported owner-native concepts")
    for family, bound in {
        "sources": 256,
        "subjects": 256,
        "excerpts": 1024,
        "edges": 2048,
        "productions": 1024,
    }.items():
        if len(families[family]) > bound or len(
            {item["id"] for item in families[family]}
        ) != len(families[family]):
            raise ValueError(
                f"{family}: native extent exceeded or duplicate identity; select a smaller exact population"
            )
    observations = freshness(result, root=root)
    if any(item["state"] not in {"current", "not-read"} for item in observations):
        raise ValueError("source cut changed while preparing corpus")
    return result


def freshness(page: dict, *, root: Path = ROOT) -> list[dict]:
    """Observe only admitted current repo locators; never reopen historical absolute paths."""
    output = []
    for selected in page.get("sources", []):
        locator = selected.get("locator", "")
        if selected.get("kind") != "source-file" or not locator.startswith("repo:"):
            output.append(
                {
                    "id": selected["id"],
                    "state": "not-read",
                    "basis": "original owner must resolve selected external reference",
                }
            )
            continue
        try:
            fact = workspace_sources.source(locator[5:], root=root)
            status = (
                "historical"
                if fact["scope"] == "History"
                else "current"
                if fact["scope"] == "Current"
                else "unknown"
            )
            state = (
                "current"
                if fact["digest"] == selected["version_digest"]
                and status == selected["status"]
                else "stale"
            )
        except (OSError, UnicodeError, ValueError):
            state = "missing-or-unavailable"
        output.append(
            {
                "id": selected["id"],
                "state": state,
                "basis": "current selected-file digest versus imported cut",
            }
        )
    return output


def native(state: Path, tool: Path, arguments: list[str], *, deadline: float) -> dict:
    with tempfile.TemporaryFile(mode="w+", encoding="utf-8") as output:
        code = evidence_store.run_native(
            state, tool, arguments, deadline=deadline, stdout=output
        )
        if code:
            raise ValueError(
                f"native knowledge operation failed with exit {code}; no source fallback was substituted"
            )
        output.seek(0)
        text = output.read((8 << 20) + 1)
        if len(text) > 8 << 20:
            raise ValueError("native knowledge response exceeds 8MiB")
        return json.loads(text)


def _run(args: argparse.Namespace) -> int:
    if not math.isfinite(args.timeout) or args.timeout <= 0:
        raise ValueError("finite positive timeout required")
    deadline = time.monotonic() + args.timeout
    selection: dict | None = None
    if args.command in {"build", "import"}:
        references = None
        if args.references:
            if args.references.stat().st_size > 4 << 20:
                raise ValueError("explicit references exceed 4MiB")
            references = json.loads(args.references.read_bytes())
        ranges = {}
        for value in args.range:
            path, separator, extent = value.rpartition(":")
            match = re.fullmatch(r"([1-9]\d*)-([1-9]\d*)", extent)
            if not separator or not match or path in ranges:
                raise ValueError("unique excerpt ranges require PATH:START-END")
            ranges[path] = (int(match[1]), int(match[2]))
        selection = corpus(
            tuple(args.path),
            expected_active=args.expected_active,
            references=references,
            ranges=ranges,
            concept=args.concept,
            identity=args.identity,
        )
        if args.command == "build":
            print(json.dumps(selection, sort_keys=True))
            return 0
    if args.state is None:
        raise ValueError(
            "--state or PSE_SURREAL_STATE required; source-only build/context search remain available"
        )
    with tempfile.TemporaryDirectory(prefix="pse-knowledge-") as directory:
        arguments = ["knowledge-" + args.command]
        if args.command in {"query", "explain", "import"}:
            if args.command in {"query", "explain"}:
                cursor = None
                if args.cursor:
                    if args.cursor.stat().st_size > 16 << 10:
                        raise ValueError("cursor exceeds 16KiB")
                    cursor = json.loads(args.cursor.read_bytes())
                selection = {
                    field: getattr(args, field)
                    for field in (
                        "generation",
                        "mode",
                        "text",
                        "kind",
                        "owner",
                        "native_id",
                        "document_role",
                        "native_state",
                        "publication_scope",
                        "subject_id",
                        "declaration_id",
                        "generator_id",
                        "artifact_id",
                        "relation",
                        "limit",
                    )
                }
                selection.update(topics=args.topic, cursor=cursor)
            input_path = Path(directory) / "input.json"
            if selection is None:
                raise RuntimeError(
                    "selected input is required for this knowledge operation"
                )
            resource_store.publish(input_path, selection)
            arguments.extend(["--input", str(input_path)])
        result = native(
            args.state.absolute(),
            args.tool.absolute(),
            arguments,
            deadline=deadline,
        )
    if args.command == "query":
        result["source_observations"] = freshness(result)
    print(json.dumps(result, sort_keys=True))
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--state", type=Path, default=os.environ.get("PSE_SURREAL_STATE")
    )
    parser.add_argument("--tool", type=Path, default=ROOT / "target/debug/pse-evidence")
    parser.add_argument("--timeout", type=float, default=30)
    commands = parser.add_subparsers(dest="command", required=True)
    for command in ("build", "import"):
        selected = commands.add_parser(command)
        selected.add_argument("--path", action="append", default=[])
        selected.add_argument(
            "--concept",
            choices=workspace_domain_contracts.SOURCE_CONCEPTS,
        )
        selected.add_argument(
            "--identity", help="exact native identity; mutually exclusive with --path"
        )
        selected.add_argument("--expected-active")
        selected.add_argument(
            "--range",
            action="append",
            default=[],
            metavar="PATH:START-END",
            help="explicit excerpt range; default first240 lines/source, each chunk at most16KiB",
        )
        selected.add_argument(
            "--references",
            type=Path,
            help="explicit typed v2 references; raw payloads stay at original owners",
        )
    for operation in ("query", "explain"):
        query = commands.add_parser(operation)
        query.add_argument(
            "--mode", choices=("subjects", "graph", "productions"), default="subjects"
        )
        query.add_argument("--text")
        for field in (
            "kind",
            "owner",
            "native-id",
            "document-role",
            "native-state",
            "publication-scope",
            "subject-id",
            "declaration-id",
            "generator-id",
            "artifact-id",
        ):
            query.add_argument("--" + field)
        query.add_argument("--topic", action="append", default=[])
        query.add_argument("--relation")
        query.add_argument("--generation")
        query.add_argument("--limit", type=int, default=10)
        query.add_argument("--cursor", type=Path)
    commands.add_parser("status")
    commands.add_parser("reset")
    args = parser.parse_args(argv)
    try:
        code = _run(args)
    except (
        OSError,
        ValueError,
        TypeError,
        KeyError,
        RuntimeError,
        subprocess.SubprocessError,
    ) as error:
        print(f"knowledge-store: {error}", file=sys.stderr)
        return 1
    else:
        return code


if __name__ == "__main__":
    raise SystemExit(main())
