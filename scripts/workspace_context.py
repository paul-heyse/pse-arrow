# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Selective command and native plan context; discovery never executes its suggestions."""

from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path

from scripts import (
    affected,
    document_lifecycle,
    document_metadata,
    session_retrospective,
)

ROOT = Path(__file__).resolve().parents[1]


def boundary_leads(symbol: str, *, root: Path = ROOT, limit: int = 40) -> dict:
    """Bounded text leads across the real catalogue, generators and consumers.

    These are source matches, not inferred calls or cross-language semantic edges.
    Native LSP requests can refine each language's references independently.
    """
    if not symbol or len(symbol) > 160 or not symbol.replace('_', '').isalnum():
        raise ValueError('symbol must be a simple source identifier of at most 160 characters')
    catalogue = root / 'crates/pse-runtime/src/boundary_documents.rs'
    paths = [catalogue]
    for directory in ('crates/pse-py/src', 'python/pse', 'crates/pse-runtime/src', 'xtask/src'):
        paths.extend(sorted((root / directory).rglob('*')))
    seen: set[Path] = set()
    items = []
    remaining = 64 << 20
    truncated = False
    for path in paths:
        if path in seen or path.suffix not in {'.rs', '.py', '.pyi'} or not path.is_file() or path.is_symlink():
            continue
        seen.add(path)
        size = path.stat().st_size
        if size > 4 << 20 or size > remaining:
            truncated = True
            continue
        remaining -= size
        with path.open(encoding='utf-8') as source:
            for number, line in enumerate(source, 1):
                if symbol in line:
                    relative = str(path.relative_to(root))
                    items.append({'source': relative, 'line': number, 'excerpt': line.strip()[:400],
                                  'meaning': 'catalogue declaration' if path == catalogue else 'text reference; inspect native semantic owner',
                                  'edit_owner': 'crates/pse-runtime/src/boundary_documents.rs and its generator' if '/generated/' in relative or relative.endswith('_native.pyi') or '/contracts/' in relative else relative})
                    if len(items) == limit:
                        return {'symbol': symbol, 'items': items, 'truncated': True,
                                'coverage': 'bounded source leads; dynamic consumers and semantic correspondence remain unresolved'}
    return {'symbol': symbol, 'items': items, 'truncated': truncated,
            'coverage': 'catalogue, PyO3, Python/stubs, runtime and xtask text leads; no claimed dynamic or whole-workspace call graph'}


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
            if owner is not None and owner.casefold() not in item.get("owner", item["document"]).casefold():
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
    parser.add_argument("--symbol", help="bounded catalogue/native/facade/stub/test source leads")
    parser.add_argument("--retrospective", action="store_true", help="explicit bounded checkout/time log observation")
    parser.add_argument("--hours", type=float, default=24)
    parser.add_argument("--sessions", type=Path, default=Path.home() / '.codex/sessions')
    parser.add_argument("--offset", type=int, default=0)
    parser.add_argument("--limit", type=int, default=10)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args(argv)
    try:
        result = describe(args.query, owner=args.owner, scope=args.scope or bool(args.owner),
                          changes=args.changed, offset=args.offset, limit=args.limit)
        if args.symbol:
            result['boundary_leads'] = boundary_leads(args.symbol, limit=args.limit)
        if args.retrospective:
            result['retrospective'] = session_retrospective.inspect(args.sessions, ROOT, hours=args.hours)
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
