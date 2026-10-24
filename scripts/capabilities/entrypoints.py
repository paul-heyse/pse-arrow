# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit Just/source inspection, independent of catalogue declaration loading."""
from __future__ import annotations
import hashlib
import json
import os
import re
import selectors
import stat
import subprocess
import time
from contextlib import ExitStack
from typing import TYPE_CHECKING
from scripts import workspace_sources
from scripts.capabilities.inventory import _Family, _families, CAP05, SOURCE_INVENTORY, SOURCE_REVIEW
from scripts.capabilities.source_modules import module_sources
if TYPE_CHECKING:
    from pathlib import Path

def _recipe_family(name: str, families: tuple[_Family, ...]) -> _Family | None:
    # Exact declarations win over deliberately declared family prefixes. Unknown
    # future names remain unknown, even when their prose resembles a known family.
    for family in families:
        if name in family.recipes:
            return family
    for family in families:
        if any(name.startswith(prefix) for prefix in family.prefixes):
            return family
    return None


def _just_dump(root: Path) -> dict:
    """Read at most 4 MiB for at most 10 s from Just's non-executing dump."""
    deadline = time.monotonic() + 10
    with subprocess.Popen(
        ["just", "--dump", "--dump-format", "json"],
        cwd=root,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    ) as process:
        if process.stdout is None:
            process.kill()
            process.wait()
            raise RuntimeError("Just recipe inspection did not provide its output pipe")
        chunks: list[bytes] = []
        size = 0
        try:
            with selectors.DefaultSelector() as selector:
                selector.register(process.stdout, selectors.EVENT_READ)
                while True:
                    remaining = deadline - time.monotonic()
                    if remaining <= 0:
                        raise TimeoutError("Just recipe inspection exceeded 10 seconds")
                    if not selector.select(remaining):
                        raise TimeoutError("Just recipe inspection exceeded 10 seconds")
                    chunk = os.read(process.stdout.fileno(), 65536)
                    if not chunk:
                        break
                    size += len(chunk)
                    if size > 4 << 20:
                        raise ValueError("Just recipe inspection exceeded 4 MiB")
                    chunks.append(chunk)
            process.wait(timeout=max(0.001, deadline - time.monotonic()))
            if process.returncode:
                raise RuntimeError(
                    "Just recipe inspection failed with exit " + str(process.returncode)
                )
        finally:
            if process.poll() is None:
                process.kill()
                process.wait()
    value = json.loads(b"".join(chunks))
    if not isinstance(value, dict) or not isinstance(value.get("recipes"), dict):
        raise TypeError("Just did not return a recipe declaration object")
    return value


def _association(family: _Family | None) -> dict:
    if family is None:
        return {
            "capability": None,
            "classification": "unclassified",
            "work_owner": CAP05,
            "reason": "No exact source or declared family association; inspect before assigning semantics.",
        }
    capability = family.capability
    return {
        "capability": capability.id,
        "classification": capability.exposure,
        "work_owner": capability.work_owner,
        "reason": capability.reason,
    }


def _leaf_sources(root: Path, recipe: dict, family: _Family | None) -> list[str]:
    """Extract untrusted references; only the source-read boundary admits paths."""
    body = json.dumps(recipe.get("body", []))
    references = re.findall(
        r"scripts/[a-zA-Z0-9_./-]+\.(?:py|rs|sh)|(?:scripts|pse)(?:\.[a-zA-Z_][a-zA-Z0-9_]*)+", body
    )
    paths = []
    for reference in references:
        path = reference
        if reference.startswith(("scripts.", "pse.")):
            resolved = module_sources(root, reference, launch=True)
            if resolved:
                for source in resolved:
                    if source not in paths:
                        paths.append(source)
                continue
            # An unresolved spelling remains an attributed unavailable lead.
            path = ("python/" if reference.startswith("pse.") else "") + reference.replace(".", "/") + ".py"
        if path not in paths:
            paths.append(path)
    if family is not None and family.capability.owner not in paths:
        paths.append(family.capability.owner)
    return paths


def _read_source_prefix(root: Path, relative: str) -> bytes:
    """Capture one bounded admitted regular source without following link swaps."""
    # The fixed root Just declaration is an explicit owner, though its
    # extensionless spelling is outside the general source suffix policy.
    root = root.resolve(strict=True)
    path = (
        root / "justfile"
        if relative == "justfile"
        else workspace_sources.selected_path(root, relative)
    )
    parts = path.relative_to(root).parts
    # Admission precedes every read. Rooted no-follow opens also prevent a
    # checked source/directory being swapped for a symlink before capture.
    with ExitStack() as stack:
        directory = os.open(root, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
        stack.callback(os.close, directory)
        for part in parts[:-1]:
            directory = os.open(
                part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=directory
            )
            stack.callback(os.close, directory)
        descriptor = os.open(
            parts[-1], os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=directory
        )
        stack.callback(os.close, descriptor)
        if not stat.S_ISREG(os.fstat(descriptor).st_mode):
            raise ValueError("source must be a regular file")
        # Captured-prefix identity is explicit if a future source exceeds
        # the bound; no outside source bytes are opened or read.
        with os.fdopen(os.dup(descriptor), "rb") as source:
            return source.read((256 << 10) + 1)


def _source_excerpt(root: Path, relative: str) -> dict:
    try:
        captured = _read_source_prefix(root, relative)
        text = captured[: 256 << 10].decode("utf-8")
    except (OSError, ValueError, RuntimeError):
        return {
            "path": relative,
            "state": "unavailable",
            "complete": False,
            "captured_sha256": None,
            "captured_bytes": 0,
            "source_truncated": False,
            "excerpt": [],
            "excerpt_complete": False,
            "reason": "Owner source is missing, inaccessible, not admitted, or invalid UTF-8; no excerpt is available.",
        }
    truncated = len(captured) > 256 << 10
    captured = captured[: 256 << 10]
    lines = text.splitlines()
    indexes: set[int] = set()
    for index, line in enumerate(lines):
        if re.search(
            r"ArgumentParser|add_argument|add_parser|add_subparsers|def main|#\[arg|#\[command|enum Cmd|struct \w*Args",
            line,
        ):
            indexes.update(range(max(0, index - 1), min(len(lines), index + 4)))
    if not indexes:
        indexes.update(range(min(35, len(lines))))
    snippets = []
    remaining = 4096
    for index in sorted(indexes):
        excerpt = lines[index]
        if len(excerpt) + 1 > remaining:
            break
        snippets.append({"line": index + 1, "text": excerpt})
        remaining -= len(excerpt) + 1
    return {
        "path": relative,
        "state": "available",
        "complete": not truncated,
        "captured_sha256": hashlib.sha256(captured).hexdigest(),
        "captured_bytes": len(captured),
        "source_truncated": truncated,
        "excerpt": snippets,
        "excerpt_complete": len(snippets) == len(lines) and not truncated,
        "meaning": "Bounded current owner-source excerpts, not an executable or complete derived argument schema.",
    }


def describe_recipe(root: Path, name: str) -> dict:
    """Describe an exact current public recipe and its actual leaf source owners."""
    root = root.resolve(strict=True)
    declaration = _just_dump(root)
    recipe = declaration["recipes"].get(name)
    if recipe is None or recipe.get("private"):
        raise ValueError("Unknown public recipe: " + name)
    family = _recipe_family(name, _families())
    sources = _leaf_sources(root, recipe, family)
    facts = [_source_excerpt(root, source) for source in sources[:8]]
    return {
        "schema": "capability-recipe-description-v1",
        "name": name,
        "description": recipe.get("doc") or "",
        "source": _source_excerpt(root, "justfile"),
        "outer_parameters": recipe.get("parameters", []),
        "attributes": recipe.get("attributes", []),
        "body": recipe.get("body", []),
        "dependencies": recipe.get("dependencies", []),
        "leaf_contracts": facts,
        "leaf_sources_omitted": max(0, len(sources) - 8),
        "leaf_schema_complete": False,
        "source_coherence": "unverified",
        "source_coherence_reason": "Just declarations and owner excerpts are separate bounded observations; no atomic common source cut is established.",
        "contract": "Live Just outer declaration and captured leaf-owner source; variadic forwarding does not establish leaf arguments/defaults. Complete parser/native binding remains owned by ADP03/ADP04.",
        "usage_route": ["just", "--usage", name],
        "definition_route": ["just", "--show", name],
        "source_inventory": SOURCE_INVENTORY,
        "source_review": SOURCE_REVIEW,
        "execution": "Only read-only Just declaration inspection and bounded owner-source reads; no recipe or leaf command ran.",
        **_association(family),
    }


def inspect_entrypoints(
    root: Path, *, offset: int = 0, limit: int = 50, kind: str | None = None
) -> dict:
    """Explicit paged reconciliation of source candidates and live public recipes.

    The only subprocess is Just's declaration dump, never a recipe/help invocation.
    Source candidates are bounded to public host files, native frontends, Python
    boundary modules, skill entrypoints and session configuration. This is not a
    recursive implementation/symbol inventory or installed-utility availability probe.
    """
    kinds = {"recipe", "script", "native", "python", "skill", "session"}
    if offset < 0 or not 1 <= limit <= 100 or (kind is not None and kind not in kinds):
        raise ValueError(
            "offset must be nonnegative, limit in 1..100, kind a known entrypoint kind"
        )
    root = root.resolve(strict=True)
    families = _families()
    by_source = {source: family for family in families for source in family.sources}
    declaration = _just_dump(root)
    items: list[dict] = []
    omissions: list[str] = []
    recipes = declaration["recipes"]
    for name, recipe in sorted(recipes.items()):
        if recipe.get("private"):
            continue
        if len(items) >= 4096:
            omissions.append("public recipe candidate bound exceeded")
            break
        family = _recipe_family(name, families)
        # Source links are extracted from the actual body, never used to guess
        # argument grammar or effect semantics. Rich leaf parsing belongs to ADP03.
        references = _leaf_sources(root, recipe, family)
        items.append(
            {
                "kind": "recipe",
                "identity": name,
                "source": "justfile",
                "description": recipe.get("doc") or "",
                "outer_parameters": recipe.get("parameters", []),
                "attributes": recipe.get("attributes", []),
                "leaf_sources": references,
                "contract": "Just outer declaration only; leaf arguments/defaults remain owned by the native parser or function.",
                "describe_route": ["just", "--show", name],
                "usage_route": ["just", "--usage", name],
                **_association(family),
            }
        )
    # Deliberately bounded globs avoid generated files, test fixtures, installed
    # packages, histories, build artifacts and native implementation forests.
    candidates = [
        ("script", path)
        for path in (root / "scripts").iterdir()
        if path.is_file() and (path.suffix in {".py", ".sh", ".rs"} or not path.suffix)
    ]
    candidates += [("native", path) for path in (root / "xtask/src/bin").glob("*.rs")]
    candidates += [("script", path) for path in (root / "scripts").glob("*/__main__.py")]
    candidates += [("native", root / "xtask/src/main.rs")]
    candidates += [("python", path) for path in (root / "python/pse").glob("*.py")]
    candidates += [
        ("skill", path) for path in (root / ".codex/skills").glob("*/SKILL.md")
    ]
    candidates += [
        ("session", root / path)
        for path in (
            ".codex/config.toml",
            ".claude/settings.json",
            ".config/library-skills.toml",
        )
    ]
    for entry_kind, path in sorted(
        candidates, key=lambda item: (item[0], item[1].as_posix())
    )[:4096]:
        if not path.is_file():
            omissions.append("source unavailable: " + path.relative_to(root).as_posix())
            continue
        relative = path.relative_to(root).as_posix()
        if entry_kind == "skill":
            association = {
                "capability": "skill-" + path.parent.name,
                "classification": "native",
                "work_owner": None,
                "reason": "Owned instruction package; retrieval does not execute its workflow."
                if path.resolve().is_relative_to(root)
                else "Retained native guidance outside checkout; its native agent skill loader owns access, with no checkout guidance resource.",
            }
        else:
            association = _association(by_source.get(relative))
        items.append(
            {
                "kind": entry_kind,
                "identity": relative,
                "source": relative,
                **association,
            }
        )
    if len(candidates) > 4096:
        omissions.append("source entrypoint candidate bound exceeded")
    if declaration.get("modules"):
        omissions.append(
            "Just module recipes are not included in this top-level inspection"
        )
    totals = {
        entry_kind: sum(item["kind"] == entry_kind for item in items)
        for entry_kind in sorted(kinds)
    }
    unclassified = {
        entry_kind: sum(
            item["kind"] == entry_kind and item["classification"] == "unclassified"
            for item in items
        )
        for entry_kind in sorted(kinds)
    }
    selected = [item for item in items if kind is None or item["kind"] == kind]
    return {
        "schema": "capability-entrypoints-v1",
        "source_inventory": SOURCE_INVENTORY,
        "source_review": SOURCE_REVIEW,
        "classification_owner": "scripts/capabilities/inventory.py",
        "items": selected[offset : offset + limit],
        "offset": offset,
        "total": len(selected),
        "next_offset": offset + limit if offset + limit < len(selected) else None,
        "totals": totals,
        "unclassified_counts": unclassified,
        "complete": not omissions and not any(unclassified.values()),
        "omissions": omissions,
        "coverage": "Top-level public Just declarations, host script candidates, xtask frontends, Python boundary modules, process/library skills and explicit session wiring; no installed availability or functional qualification.",
        "execution": "Only the bounded read-only Just declaration dump ran; no recipes, leaf commands, native imports or services were started.",
    }
