# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Task and alternative projections from existing owners; never a new task ledger."""

from __future__ import annotations

import hashlib
import json
import re
import subprocess
import tomllib
from pathlib import Path

from scripts import affected, document_lifecycle, document_metadata, workspace_sources

ROOT = Path(__file__).resolve().parents[1]


def _files(paths: tuple[str, ...], root: Path) -> list[dict]:
    if len(paths) > 32:
        raise ValueError("select at most 32 task sources")
    revision = workspace_sources.git(root, "rev-parse", "HEAD") or None
    result = []
    for relative in dict.fromkeys(paths):
        try:
            record = workspace_sources.source(relative, root=root, revision=revision)
        except (OSError, UnicodeError, ValueError) as error:
            result.append({"path": relative, "availability": "unavailable", "reason": str(error)})
            continue
        lines = record.pop("text").splitlines()
        # Selected detail is bounded; these are leads, not a second status parser.
        record["question_leads"] = [
            {"line": number, "text": line[:300], "basis": "text lead; inspect native owner"}
            for number, line in enumerate(lines, 1)
            if re.search(r"\b(?:unresolved|unknown|next|open question)\b", line, re.I)
        ][:8]
        record["availability"] = "current-file-read"
        result.append(record)
    return result


def task_packet(task: str | None, paths: tuple[str, ...], *, root: Path = ROOT,
                symbol: str | None = None, history_query: str | None = None,
                history_mode: str = "literal", evidence: tuple[str, ...] = ()) -> dict:
    """Select native scope rows and exact files without executing suggested work."""
    selected = list(paths)
    rows = []
    omitted = []
    if task:
        plan, separator, native_id = task.partition("#")
        fact = workspace_sources.source(plan, root=root)
        if not plan.startswith("docs/plans/"):
            raise ValueError("named task must belong to a retained plan")
        selected.extend([plan, fact["owner"]])
        try:
            native = document_lifecycle.aggregate(root, document_metadata.load_policy(root), documents={plan, fact['owner']})
            rows = [row for row in native
                    if row["document"] in {plan, fact["owner"]}
                    and (not separator or row.get("native_id") == native_id)]
            if separator and not rows:
                omitted.append("named native package not found; no state inferred from prose")
        except (OSError, ValueError) as error:
            omitted.append(f"native scope unavailable: {error}")
    facts = _files(tuple(dict.fromkeys(selected)), root)
    admitted = []
    for selected_file in facts:
        try:
            workspace_sources.selected_path(root, selected_file['path'], allow_missing=True)
            admitted.append(selected_file['path'])
        except ValueError:
            omitted.append('unadmitted requested path: '+selected_file['path'])
    queries = tuple([*admitted, *([symbol] if symbol else [])])
    relations = workspace_sources.relationships(queries, root=root) if queries else {'items':[], 'availability':'no admitted paths or symbols'}
    dirty = workspace_sources.git(root, "status", "--porcelain=v1", "--", *(':(literal)'+path for path in admitted)).splitlines() if admitted else []
    commits = []
    if admitted:
        arguments = ["git", "log", "-8", "--format=%H%x09%aI%x09%s"]
        if history_query:
            if len(history_query) > 512 or history_mode not in {"literal", "regex"}:
                raise ValueError("history query must be bounded literal or regex")
            arguments.extend(["-S" if history_mode == "literal" else "-G", history_query])
        result = subprocess.run([*arguments, "--", *(':(literal)'+path for path in admitted)], cwd=root, text=True,
                                capture_output=True, check=False, timeout=10)
        if result.returncode == 0:
            commits = [dict(zip(("commit", "date", "subject"), line.split("\t", 2), strict=True))
                       for line in result.stdout.splitlines()]
        else:
            omitted.append("selected Git rationale lookup unavailable")
    if len(evidence) > 16 or any(len(item) > 2048 for item in evidence):
        raise ValueError("select at most 16 bounded evidence handles")
    return {"schema": "workspace-task-v1", "task": task, "native_scope": rows,
            "sources": facts, "relationships": relations, "dirty_delta": dirty,
            "git_rationale": {"items": commits, "basis": "historical commit metadata; not current authorization",
                              "mode": history_mode, "query": history_query},
            "evidence": [{"handle": item, "basis": "caller-selected reference; payload not read"} for item in evidence],
            "omissions": omitted + ["dynamic consumers unresolved", "database and session history not required or opened"],
            "authority": "native source/status owners retain authority; task packet is derived"}


def alternatives(document: dict, *, root: Path = ROOT) -> dict:
    if set(document) != {"question", "alternatives"} or not isinstance(document["question"], str):
        raise ValueError("alternative input requires question and alternatives only")
    arrangements = document["alternatives"]
    if not isinstance(arrangements, list) or len(arrangements) != 2:
        raise ValueError("compare exactly two explicit alternatives")
    if not document["question"].strip() or len(document["question"]) > 4096:
        raise ValueError("bounded design question required")
    manifests = []
    for path in root.glob("crates/*/Cargo.toml"):
        data = tomllib.loads(path.read_text())
        package = data.get("package", {}).get("name")
        if isinstance(package, str):
            manifests.append((path.parent.relative_to(root), package, data))
    workspace = tomllib.loads((root / "Cargo.toml").read_text()).get("workspace", {}).get("dependencies", {}) if (root / "Cargo.toml").is_file() else {}
    dependents = {name: set() for _, name, _ in manifests}
    for _, name, data in manifests:
        sections = [data, *data.get("target", {}).values()]
        for section in sections:
            for kind in ("dependencies", "dev-dependencies", "build-dependencies"):
                for key, value in section.get(kind, {}).items():
                    declared = workspace.get(key, value) if isinstance(value, dict) and value.get("workspace") else value
                    dependency = declared.get("package", key) if isinstance(declared, dict) else key
                    if dependency in dependents:
                        dependents[dependency].add(name)
    result = []
    for arrangement in arrangements:
        if not isinstance(arrangement, dict) or set(arrangement) != {"name", "proposal", "paths", "removals", "unknowns"}:
            raise ValueError("each alternative requires name/proposal/paths/removals/unknowns")
        if any(not isinstance(arrangement[key], str) or not arrangement[key].strip() or len(arrangement[key]) > 4096
               for key in ("name", "proposal")):
            raise ValueError("bounded named proposal required")
        for field in ("paths", "removals", "unknowns"):
            if not isinstance(arrangement[field], list) or len(arrangement[field]) > 32 or any(
                not isinstance(item, str) or len(item) > 4096 for item in arrangement[field]
            ):
                raise ValueError("bounded explicit paths/removals/unknowns required")
        packet = task_packet(None, tuple(arrangement["paths"]), root=root)
        # Manifest membership gives conservative reach without resolving/building Cargo.
        seeds = {name for directory, name, _ in manifests
                 if any(Path(source).is_relative_to(directory) for source in arrangement["paths"])}
        if any(source == prefix or prefix.endswith('/') and source.startswith(prefix)
               for source in arrangement['paths'] for prefix in affected.WORKSPACE_WIDE):
            seeds = {name for _, name, _ in manifests}
        reach = set(seeds)
        while True:
            widened = reach | {dependent for name in reach for dependent in dependents[name]}
            if widened == reach:
                break
            reach = widened
        result.append({**arrangement, "proposal_basis": "caller proposal, not implemented design",
                       "current": packet, "package_reach": sorted(reach),
                       "reach_basis": "conservative all-target manifest dependency reach; optional dependencies included, dynamic/string consumers unresolved",
                       "decision_route": "inspect .claude/rules/decisions.md against proposed contract changes; no automatic approval"})
    return {"schema": "workspace-alternatives-v1", "question": document["question"],
            "alternatives": result, "selection": "operator/agent judgment required; no architectural score or auto approval"}


def line_history(selection: str, *, root: Path = ROOT) -> dict:
    match = re.fullmatch(r"([1-9]\d*),([1-9]\d*):(.+)", selection)
    if match is None:
        raise ValueError("line history needs START,END:repository-relative-path")
    start, end = int(match[1]), int(match[2])
    if end < start or end - start > 40:
        raise ValueError("line history selects at most 41 source lines")
    fact = workspace_sources.source(match[3], root=root)
    result = subprocess.run(["git", "log", "-3", "--no-patch", "--format=%H%x09%aI%x09%s", "-L", selection],
                            cwd=root, capture_output=True, text=True, check=False, timeout=10)
    return {"selection": selection, "current_digest": fact["digest"],
            "commits": result.stdout.splitlines()[:6], "available": result.returncode == 0,
            "error": result.stderr[:400] if result.returncode else None,
            "basis": "selected historical line provenance; dirty lines may not map to HEAD"}
