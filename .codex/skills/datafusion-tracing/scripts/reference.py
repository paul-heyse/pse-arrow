#!/usr/bin/env python3
"""Offline, bounded capability discovery and full-contract retrieval. Standard library only."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import re
import sys
from collections.abc import Iterator
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
STOP = {
    "the",
    "a",
    "an",
    "to",
    "for",
    "of",
    "and",
    "with",
    "in",
    "from",
    "how",
    "can",
    "i",
    "by",
    "is",
    "as",
    "that",
}


def tokens(text: str) -> set[str]:
    return set(re.findall(r"[a-z0-9]+", text.lower())) - STOP


def catalog(root: Path) -> list[dict]:
    return json.loads((root / "content/capabilities/catalog.json").read_text())


def canonical(root: Path, path: str) -> list[str]:
    """Every defining path a spelling can mean, most specific first.

    Returns a LIST, because a spelling can legitimately mean two things: a module and an
    item re-exported under the same path (`salsa::input` the module and the macro;
    `tracing::instrument`; `arrow_cast::cast`). Ported from the rust-graphs sibling, where
    `petgraph::algo::dijkstra` is both a module and the re-exported function inside it. Collapsing that to one answer means silently picking
    for the reader, and the one a caller almost always wants -- the function -- is the one a
    defining-path-first rule discards.

    Two earlier versions of this got it wrong in opposite directions. Rewriting by longest
    prefix with no guard turned the already-canonical path into
    `algo::dijkstra::dijkstra::dijkstra`, because the alias key is a strict prefix of its own
    target. Preferring the defining path returned the module and dropped the function.
    """
    defined = {
        line.split("\t", 1)[0]
        for line in (root / "content/index/operations.tsv").read_text().splitlines() if line
    }
    aliases = {}
    for row in (root / "content/index/aliases.tsv").read_text().splitlines():
        alias, target, _ = row.split("\t")
        aliases[alias] = target

    found: list[str] = []
    if path in aliases and aliases[path] in defined:
        found.append(aliases[path])
    if path in defined and path not in found:
        found.append(path)
    if not found:
        parts = path.split("::")
        for end in range(len(parts), 0, -1):
            prefix = "::".join(parts[:end])
            if prefix in aliases:
                resolved = "::".join([aliases[prefix], *parts[end:]])
                if resolved in defined:
                    found.append(resolved)
                break
    return found or [path]


def operation_rows(root: Path) -> Iterator[dict]:
    with (root / "content/index/operations.tsv").open() as source:
        for line in source:
            path, op_id, kind, crate, record, page, signature = line.rstrip("\n").split(
                "\t"
            )
            yield dict(
                path=path,
                id=op_id,
                kind=kind,
                crate=crate,
                record=f"content/{record}",
                page=f"content/{page}",
                signature=signature,
            )


def brief(record: dict) -> dict:
    return {
        k: record[k]
        for k in [
            "id",
            "title",
            "brief",
            "input",
            "output",
            "choices",
            "unknowns",
            "evidence",
        ]
    } | {
        "contract": f"content/capabilities/{record['id']}.json",
        "coverage": record["coverage"],
    }


def find(root: Path, args: argparse.Namespace) -> dict:
    query = tokens(args.task or "")
    scored = []
    for record in catalog(root):
        facets = {
            "crate": record["crates"],
            "input": record["input"],
            "output": record["output"],
            "property": record["properties"],
        }
        if any(
            getattr(args, f)
            and not any(getattr(args, f).lower() in v.lower() for v in values)
            for f, values in facets.items()
        ):
            continue
        phrases = " ".join(record["task_aliases"])
        primary = tokens(phrases + " " + record["title"])
        all_terms = tokens(json.dumps(record))
        matched = sorted(query & all_terms)
        score = 4 * len(query & primary) + len(query & all_terms)
        if query and not score:
            continue
        scored.append(
            (
                score,
                record["id"],
                {
                    "id": record["id"],
                    "title": record["title"],
                    "brief": record["brief"],
                    "why": {
                        "matched_terms": matched,
                        "facets": {
                            f: getattr(args, f) for f in facets if getattr(args, f)
                        },
                    },
                    "choices": record["choices"],
                    "unknowns": record["unknowns"],
                    "contract": f"content/capabilities/{record['id']}.json",
                },
            )
        )
    scored.sort(key=lambda x: (-x[0], x[1]))
    # Exact/qualified and leaf API discovery remains available beyond reviewed cards.
    api = []
    name = (
        canonical(root, args.task)[0]
        if args.task and "::" in args.task
        else args.task or ""
    )
    seen = set()
    if name:
        needle = name.lower()
        for row in operation_rows(root):
            if args.crate and args.crate.lower() not in row["crate"].lower():
                continue
            if (needle in row["path"].lower()) and row["path"] not in seen:
                seen.add(row["path"])
                if len(api) < args.limit:
                    api.append(row)
    role_rows = list(
        csv.DictReader(
            (root / "authoring/crate-roles.tsv").read_text().splitlines(),
            delimiter="\t",
        )
    )
    roles = []
    for role in role_rows:
        if args.crate and args.crate.lower() not in role["crate"].lower():
            continue
        score = len(query & tokens(" ".join(role.values())))
        if score or (args.crate and not query):
            roles.append((score, role["crate"], role))
    roles.sort(key=lambda r: (-r[0], r[1]))
    return {
        "query": args.task,
        "capabilities": [x[2] for x in scored[: args.limit]],
        "matching_capabilities": len(scored),
        "operations": api,
        "matching_operation_paths": len(seen),
        "crate_routes": [r[2] for r in roles[: args.limit]],
        "scope": (
            "Ranked lexical/task/facet discovery, not a semantic completeness claim. "
            "Empty results do not prove absence."
        ),
        "more": (
            "Use show <id or canonical/facade path>; "
            "full indexes and crate routes remain available."
        ),
    }


def show(root: Path, args: argparse.Namespace) -> dict:
    for record in catalog(root):
        if record["id"] == args.target:
            if args.view == "brief":
                return brief(record)
            if args.view == "evidence":
                return {
                    "id": record["id"],
                    "claims": record["claims"],
                    "evidence": record["evidence"],
                    "contracts": record["resolved_operations"],
                    "unknowns": record["unknowns"],
                }
            return record
    targets = canonical(root, args.target)
    if args.member:
        targets = [f"{t}::{args.member}" for t in targets]
    wanted = set(targets)
    matches = [
        r for r in operation_rows(root) if r["path"] in wanted or r["id"] == args.target
    ]
    if not matches:
        raise ValueError(
            f"No exact record for {args.target!r}; use find or the crate/index routes"
        )
    ids = {r["id"] for r in matches}
    records = [
        r
        for file in sorted({r["record"] for r in matches})
        for r in json.loads((root / file).read_text())
        if r["id"] in ids
    ]
    if args.view == "brief":
        fields = {
            "id",
            "path",
            "kind",
            "signature",
            "docs",
            "span",
            "artifact",
            "availability",
            "impl_context",
            "aliases",
            "resolved_doc_links",
            "reader_notes",
            "reachability",
            "reached_via",
            "page",
            "anchor",
        }
        records = [{k: v for k, v in r.items() if k in fields} for r in records]
    return {
        "requested": args.target,
        # One path, or every defining path the spelling can mean (module and re-export).
        "canonical": targets[0] if len(targets) == 1 else targets,
        "matches": records,
        "note": (
            "Multiple implementation contexts are retained. Contract view includes "
            "original type_tree and artifact-scoped references."
        ),
    }


def bounded(value: dict, max_bytes: int, offset: int) -> str:
    encoded = json.dumps(value, ensure_ascii=False, indent=2).encode()
    if not offset and len(encoded) + 1 <= max_bytes:
        return encoded.decode()
    if offset < len(encoded) and (encoded[offset] & 0xC0) == 0x80:
        raise ValueError("Offset is not a UTF-8 boundary")
    if offset > len(encoded):
        raise ValueError("Offset exceeds result size")
    # Expose a lossless continuation over UTF-8 text; never present a silently clipped contract.
    size = max_bytes - 700
    while size > 0:
        fragment = encoded[offset : offset + size].decode("utf-8", errors="ignore")
        consumed = len(fragment.encode())
        result = {
            "partial": True,
            "result_sha256": hashlib.sha256(encoded).hexdigest(),
            "format": "JSON text fragment",
            "offset_bytes": offset,
            "total_bytes": len(encoded),
            "next_offset": offset + consumed
            if offset + consumed < len(encoded)
            else None,
            "continuation": "Repeat the same command and view with --offset <next_offset>.",
            "text_fragment": fragment,
        }
        text = json.dumps(result, ensure_ascii=False, indent=2)
        if len(text.encode()) + 1 <= max_bytes:
            return text
        size -= max(1, len(text.encode()) + 1 - max_bytes)
    raise ValueError("Byte budget too small to return a continuation envelope")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    common = argparse.ArgumentParser(add_help=False)
    common.add_argument("--max-bytes", type=int, default=12000)
    common.add_argument("--offset", type=int, default=0)
    lookup = sub.add_parser("find", parents=[common])
    lookup.add_argument("--task", default="")
    lookup.add_argument("--input")
    lookup.add_argument("--output")
    lookup.add_argument("--property")
    lookup.add_argument("--crate")
    lookup.add_argument("--limit", type=int, default=5)
    detail = sub.add_parser("show", parents=[common])
    detail.add_argument("target")
    detail.add_argument("--member")
    detail.add_argument(
        "--view", choices=["brief", "contract", "evidence"], default="brief"
    )
    compare = sub.add_parser("compare", parents=[common])
    compare.add_argument("targets", nargs="+", help="Two or more capability IDs")
    args = parser.parse_args(argv)
    if args.max_bytes < 1200 or args.offset < 0:
        parser.error("--max-bytes must be at least 1200; --offset must be nonnegative")
    try:
        if args.command == "find":
            if not 1 <= args.limit <= 50:
                raise ValueError("--limit must be 1..50")
            result = find(ROOT, args)
        elif args.command == "show":
            result = show(ROOT, args)
        else:
            records = {r["id"]: r for r in catalog(ROOT)}
            if len(args.targets) < 2 or any(t not in records for t in args.targets):
                raise ValueError("compare needs at least two known capability IDs")
            result = {
                "comparison": [brief(records[t]) for t in args.targets],
                "decision_axes": sorted(
                    {p for t in args.targets for p in records[t]["properties"]}
                ),
                "note": (
                    "Choices are conditional authored judgments; "
                    "inspect the decisive contract, not rank alone."
                ),
            }
        sys.stdout.write(bounded(result, args.max_bytes, args.offset) + "\n")
        return 0
    except (ValueError, OSError) as error:
        sys.stderr.write(json.dumps({"error": str(error)}) + "\n")
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
