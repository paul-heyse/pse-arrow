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
    workspace_sources,
    workspace_tasks,
    workspace_domains,
)

ROOT = Path(__file__).resolve().parents[1]


def boundary_leads(symbol: str, *, root: Path = ROOT, limit: int = 40) -> dict:
    """Bounded text leads across the real catalogue, generators and consumers.

    These are source matches, not inferred calls or cross-language semantic edges.
    Native LSP requests can refine each language's references independently.
    """
    if not symbol or len(symbol) > 160 or not symbol.replace("_", "").isalnum():
        raise ValueError(
            "symbol must be a simple source identifier of at most 160 characters"
        )
    catalogue = root / "crates/pse-runtime/src/boundary_documents.rs"
    paths = [catalogue]
    for directory in (
        "crates/pse-py/src",
        "python/pse",
        "crates/pse-runtime/src",
        "xtask/src",
    ):
        paths.extend(sorted((root / directory).rglob("*")))
    seen: set[Path] = set()
    items = []
    remaining = 64 << 20
    truncated = False

    def complete(partial: bool) -> dict:
        declared = workspace_sources.relationships(tuple([symbol, *(item['source'] for item in items)]), root=root)
        for item in items:
            links = [row for row in declared['items'] if row.get('output_path') == item['source']]
            if links:
                item['edit_owner'] = sorted({row.get('source_path') or row['generator_path'] for row in links})
                item['owner_basis'] = 'producer-declared contributors; unknown declaration locations remain unavailable'
        return {'symbol':symbol, 'items':items, 'truncated':partial,
                'declared_relationships':declared,
                'coverage':'bounded text leads and producer-declared contributors; language semantic references and dynamic consumers unresolved'}
    for path in paths:
        if (
            path in seen
            or path.suffix not in {".rs", ".py", ".pyi"}
            or not path.is_file()
            or path.is_symlink()
        ):
            continue
        seen.add(path)
        size = path.stat().st_size
        if size > 4 << 20 or size > remaining:
            truncated = True
            continue
        remaining -= size
        with path.open(encoding="utf-8") as source:
            for number, line in enumerate(source, 1):
                if symbol in line:
                    relative = str(path.relative_to(root))
                    items.append(
                        {
                            "source": relative,
                            "line": number,
                            "excerpt": line.strip()[:400],
                            "meaning": "catalogue declaration"
                            if path == catalogue
                            else "text reference; inspect native semantic owner",
                            "edit_owner": None
                            if "/generated/" in relative or relative.endswith("_native.pyi") or "/contracts/" in relative
                            else relative,
                            "owner_basis": "inspect producer-declared relationships for generated outputs"
                            if "/generated/" in relative or relative.endswith("_native.pyi") or "/contracts/" in relative
                            else "selected source file",
                        }
                    )
                    if len(items) == limit:
                        return complete(True)
    return complete(truncated)


def recipe_context(recipes: dict, query: str) -> list[dict]:
    words = query.casefold().split()
    result = []
    for name, recipe in sorted(recipes.items()):
        if recipe.get("private"):
            continue
        meaning = str(recipe.get("doc") or "")
        if not all(word in (name + " " + meaning).casefold() for word in words):
            continue
        result.append(
            {
                "kind": "recipe",
                "name": name,
                "description": meaning,
                "parameters": recipe.get("parameters", []),
                "attributes": recipe.get("attributes", []),
                "source": "justfile",
                "usage": ["just", "--usage", name],
                "definition": ["just", "--show", name],
                "execution": "not requested; grammar/effects are owned by the recipe",
            }
        )
    return result


def describe(
    query: str,
    *,
    owner: str | None,
    scope: bool,
    changes: bool,
    offset: int,
    limit: int,
    recipes: bool = True,
) -> dict:
    if offset < 0 or not 1 <= limit <= 100:
        raise ValueError("offset must be nonnegative and limit in 1..100")
    declaration = json.loads(
        subprocess.check_output(
            ["just", "--dump", "--dump-format", "json"], cwd=ROOT, text=True
        )
    ) if recipes and owner is None else {"recipes": {}}
    items = recipe_context(declaration["recipes"], query) if owner is None else []
    if scope:
        for item in document_lifecycle.aggregate(
            ROOT, document_metadata.load_policy(ROOT)
        ):
            if (
                owner is not None
                and owner.casefold()
                not in item.get("owner", item["document"]).casefold()
            ):
                continue
            if all(
                word in json.dumps(item).casefold() for word in query.casefold().split()
            ):
                items.append({"kind": "native-scope", **item})
    selected = items[offset : offset + limit]
    return {
        "schema": "workspace-context-v1",
        "query": query,
        "items": selected,
        "next_offset": offset + limit if len(items) > offset + limit else None,
        "changes": affected.changed_files("HEAD") if changes else None,
        "authority": "derived discovery; native owners retain status and authorization",
        "source_routes": [
            "docs/README.md",
            "docs/plans/README.md",
            "crates/pse-runtime/src/boundary_documents.rs",
        ],
        "coverage": "recipe grammar and selected native plan rows; no inferred dynamic or cross-language reach",
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("query", nargs="?", default="")
    parser.add_argument("--owner")
    parser.add_argument("--concept", choices=tuple(workspace_domains.CONCEPTS), help="primary exact owner-native concept query")
    parser.add_argument("--identity", help="exact owner-native concept identity")
    parser.add_argument("--topic", action="append", default=[], help="exact native metadata topic; repeated values are ANDed")
    parser.add_argument("--role", help="exact interpreted document role")
    parser.add_argument("--document", help="exact declaring document facet")
    parser.add_argument("--publication-scope", choices=("Current", "Reference", "History"), help="exact publication scope, independent of native status")
    parser.add_argument("--native-status", help="exact owner-native document/package status")
    parser.add_argument("--origin", choices=("user-question", "agent-interpretation"), help="explicit research-question origin")
    parser.add_argument("--native-reference", type=Path, help="explicit JSON selecting an original evidence adapter and member")
    parser.add_argument("--scope", action="store_true")
    parser.add_argument("--changed", action="store_true")
    parser.add_argument(
        "--symbol", help="bounded catalogue/native/facade/stub/test source leads"
    )
    parser.add_argument(
        "--retrospective",
        action="store_true",
        help="explicit bounded checkout/time log observation",
    )
    parser.add_argument("--hours", type=float, default=24)
    parser.add_argument(
        "--sessions", type=Path, default=Path.home() / ".codex/sessions"
    )
    parser.add_argument("--offset", type=int, default=0)
    parser.add_argument("--limit", type=int, default=10)
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--task", help="retained plan path, optionally #NATIVE_PACKAGE")
    parser.add_argument("--path", action="append", default=[], help="selected repository-relative task source")
    parser.add_argument("--evidence", action="append", default=[], help="explicit evidence handle; does not read payload")
    parser.add_argument("--history-query", help="selected-file Git pickaxe rationale")
    parser.add_argument("--history-mode", choices=("literal", "regex"), default="literal")
    parser.add_argument("--line-history", help="selected Git history START,END:repository-relative-path")
    parser.add_argument("--compare", type=Path, help="explicit JSON with two named proposed arrangements")
    parser.add_argument("--search", action="store_true", help="source-fresh lexical query")
    parser.add_argument("--source-root", action="append", default=[], help="narrow lexical source membership")
    parser.add_argument("--library", help="exact dependency research packet, no install or resolution")
    parser.add_argument("--reference", action="append", default=[])
    parser.add_argument("--feature", action="append", default=[])
    parser.add_argument("--target", help="selected target for supplied library build evidence")
    parser.add_argument("--cargo-metadata", help="repository-relative selected Cargo observation JSON")
    parser.add_argument("--rustdoc", help="repository-relative selected rustdoc observation JSON")
    parser.add_argument("--source-role", action="append", default=[])
    parser.add_argument("--literal", action="store_true", help="literal lexical query")
    parser.add_argument("--recall", help="explicit historical text query; returns locators only")
    parser.add_argument("--history-checkpoint", type=Path, help="resume an explicit metadata-only history checkpoint")
    parser.add_argument("--save-history-checkpoint", type=Path, help="write a disposable metadata-only checkpoint")
    parser.add_argument("--related-checkout", action="append", type=Path, default=[])
    parser.add_argument("--archived-sessions", type=Path)
    parser.add_argument("--history-excerpt", type=Path, help="explicit selected locator JSON; reads bounded raw historical detail")
    args = parser.parse_args(argv)
    if args.concept:
        if any((args.search, args.symbol, args.retrospective, args.recall, args.history_excerpt, args.library, args.compare, args.scope, args.changed, args.path, args.line_history, args.history_query)):
            parser.error("exact concept queries are separate from explicit lexical/history/task discovery")
        try:
            reference = None
            if args.native_reference:
                if args.native_reference.stat().st_size > 16 << 10:
                    raise ValueError("native reference selection exceeds 16KiB")
                reference = json.loads(args.native_reference.read_bytes())
            identity = args.identity or (args.task if args.concept == "plan-package" else None)
            result = workspace_domains.query(args.concept, identity, root=ROOT, topics=tuple(args.topic), role=args.role, owner=args.owner,
                document=args.document, origin=args.origin, question=args.query if args.concept == "research-question" else None,
                selected_reference=reference, publication_scope=args.publication_scope, native_status=args.native_status, limit=args.limit, offset=args.offset)
        except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
            parser.exit(1, f"context: exact concept unavailable: {error}\n")
        print(json.dumps(result, sort_keys=True, indent=None if args.json else 2))
        return 0
    if args.identity or args.topic or args.role or args.document or args.origin or args.native_reference or args.publication_scope or args.native_status:
        parser.error("exact identity/topic/role/document/origin/reference facets require --concept")
    if args.history_query and not (args.task or args.path):
        parser.error("--history-query requires --task or --path")
    if (args.history_checkpoint or args.save_history_checkpoint) and not (args.retrospective or args.recall):
        parser.error("history checkpoints require --retrospective or --recall")
    try:
        result = describe(
            args.query,
            owner=args.owner,
            scope=args.scope or bool(args.owner),
            changes=args.changed,
            offset=args.offset,
            limit=args.limit,
            recipes=not any((args.task, args.path, args.compare, args.search, args.library, args.changed, args.scope, args.owner,
                             args.symbol, args.retrospective, args.recall, args.history_excerpt, args.line_history)),
        )
        if args.task or args.path:
            result["task"] = workspace_tasks.task_packet(
                args.task, tuple(args.path), symbol=args.symbol, history_query=args.history_query,
                history_mode=args.history_mode, evidence=tuple(args.evidence))
        if args.search:
            result["search"] = workspace_sources.search(args.query, roots=tuple(args.source_root), roles=tuple(args.source_role), literal=args.literal,
                                                         limit=args.limit, offset=args.offset)
        if args.library:
            result["library"] = workspace_sources.library_packet(args.library,
                references=tuple(args.reference), features=tuple(args.feature), target=args.target,
                cargo_metadata=args.cargo_metadata, rustdoc=args.rustdoc)
        if args.compare:
            if args.compare.stat().st_size > 64 << 10:
                raise ValueError("alternative document exceeds 64KiB")
            result["alternatives"] = workspace_tasks.alternatives(json.loads(args.compare.read_bytes()))
            if args.library:
                result['alternatives']['selected_research'] = result['library']
        if args.line_history:
            result["line_history"] = workspace_tasks.line_history(args.line_history)
        if args.symbol:
            result["boundary_leads"] = boundary_leads(args.symbol, limit=args.limit)
            result["declared_paths"] = workspace_sources.relationships(args.symbol, limit=args.limit)
        if args.retrospective or args.recall:
            continuation = None
            if args.history_checkpoint:
                if args.history_checkpoint.stat().st_size > 16 << 20:
                    raise ValueError("history checkpoint exceeds 16MiB")
                continuation = json.loads(args.history_checkpoint.read_bytes())
            result["retrospective"] = session_retrospective.inspect(
                args.sessions, ROOT, hours=args.hours, related_checkouts=tuple(args.related_checkout),
                archived_sessions=args.archived_sessions, continuation=continuation, query=args.recall,
            )
            if args.save_history_checkpoint:
                args.save_history_checkpoint.write_text(json.dumps(result["retrospective"]["checkpoint"], sort_keys=True))
        if args.history_excerpt:
            if args.history_excerpt.stat().st_size > 16 << 10:
                raise ValueError("history locator exceeds 16KiB")
            result["history_excerpt"] = session_retrospective.read_excerpt(args.sessions,
                json.loads(args.history_excerpt.read_bytes()), archived_sessions=args.archived_sessions)
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        parser.exit(1, f"context: unavailable: {error}\n")
    if args.json:
        print(json.dumps(result, sort_keys=True))
    else:
        # All sections are visible in both renderings, including partial/unavailable observations.
        print(json.dumps(result, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
