# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Selective command and native plan context; discovery never executes its suggestions."""

from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path

from scripts import affected, document_lifecycle, document_metadata

ROOT = Path(__file__).resolve().parents[1]


def recipe_context(recipes: dict, query: str) -> list[dict]:
    words = query.casefold().split()
    result = []
    for name, recipe in sorted(recipes.items()):
        if recipe.get("private"):
            continue
        meaning = str(recipe.get("doc") or "")
        if not all(word in (name + " " + meaning).casefold() for word in words):
            continue
        result.append({"kind": "recipe", "name": name, "description": meaning,
                       "parameters": recipe.get("parameters", []),
                       "attributes": recipe.get("attributes", []),
                       "source": "justfile", "usage": ["just", "--usage", name],
                       "definition": ["just", "--show", name],
                       "execution": "not requested; grammar/effects are owned by the recipe"})
    return result


def describe(query: str, *, owner: str | None, scope: bool, changes: bool,
             offset: int, limit: int) -> dict:
    if offset < 0 or not 1 <= limit <= 100:
        raise ValueError("offset must be nonnegative and limit in 1..100")
    declaration = json.loads(subprocess.check_output(
        ["just", "--dump", "--dump-format", "json"], cwd=ROOT, text=True))
    items = recipe_context(declaration["recipes"], query) if owner is None else []
    if scope:
        for item in document_lifecycle.aggregate(ROOT, document_metadata.load_policy(ROOT)):
            if owner is not None and owner.casefold() not in str(item.get("owner", item["document"])).casefold():
                continue
            if all(word in json.dumps(item).casefold() for word in query.casefold().split()):
                items.append({"kind": "native-scope", **item})
    selected = items[offset:offset + limit]
    return {"schema": "workspace-context-v1", "query": query, "items": selected,
            "next_offset": offset + limit if len(items) > offset + limit else None,
            "changes": affected.changed_files("HEAD") if changes else None,
            "authority": "derived discovery; native owners retain status and authorization",
            "source_routes": ["docs/README.md", "docs/plans/README.md", "crates/pse-runtime/src/boundary_documents.rs"],
            "coverage": "recipe grammar and selected native plan rows; no inferred dynamic or cross-language reach"}


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("query", nargs="?", default="")
    parser.add_argument("--owner")
    parser.add_argument("--scope", action="store_true")
    parser.add_argument("--changed", action="store_true")
    parser.add_argument("--offset", type=int, default=0)
    parser.add_argument("--limit", type=int, default=10)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args(argv)
    try:
        result = describe(args.query, owner=args.owner, scope=args.scope or bool(args.owner),
                          changes=args.changed, offset=args.offset, limit=args.limit)
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        parser.exit(1, f"context: unavailable: {error}\n")
    if args.json:
        print(json.dumps(result, sort_keys=True))
    else:
        for item in result["items"]:
            print(json.dumps(item))
        print(f"next_offset={result['next_offset']}; {result['authority']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
