# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Source-bound previews shared by language and structural edit producers.

A preview is review material, never an authorization inferred from a language server.
Application rechecks every input first; reconciliation records partial writes truthfully.
"""

from __future__ import annotations

import hashlib
import json
import os
import tempfile
from itertools import pairwise
from pathlib import Path
from typing import TypedDict
from urllib.parse import unquote, urlsplit


class Edit(TypedDict):
    start: int
    end: int
    text: str


def digest(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def owned_path(root: Path, name: str, *, allow_missing: bool = False) -> Path:
    root = root.resolve()
    uri = urlsplit(name)
    if uri.scheme:
        if (
            uri.scheme != "file"
            or uri.netloc not in {"", "localhost"}
            or uri.query
            or uri.fragment
        ):
            raise ValueError("edit requires a local file URI")
        path = Path(unquote(uri.path))
    else:
        path = Path(name)
        if not path.is_absolute():
            path = root / path
    if ".." in path.parts or any(p.is_symlink() for p in (path, *path.parents)):
        raise ValueError("edit path traverses a symlink or parent component")
    try:
        relative = path.relative_to(root)
    except ValueError as error:
        raise ValueError("edit path is outside the selected workspace") from error
    parts = relative.parts
    protected = (
        not parts
        or parts[0] in {".git", "external", "build", "target"}
        or "generated" in parts
        or relative.as_posix().startswith(
            (
                "docs/authoritative_design/",
                "docs/adr/",
                "crates/pse-operations-queries/",
                "python/pse/contracts/",
            )
        )
        or relative.as_posix()
        in {"python/pse/_native.pyi", "crates/pse-ipopt-sys/src/bindings.rs"}
    )
    if protected:
        raise ValueError(
            "edit path is generated or protected; follow its declaration owner"
        )
    if not path.is_file() and not (allow_missing and not path.exists()):
        raise ValueError("edit requires an existing regular owned source file")
    return path


def capture(root: Path, paths: list[str]) -> dict[str, str]:
    """Capture explicit owned inputs before a request, including cross-file consumers."""
    result = {}
    for name in paths:
        path = owned_path(root, name)
        if path.stat().st_size > 10 << 20:
            raise ValueError("source file exceeds preview allowance")
        # Preserve CRLF and reject undecodable source rather than normalizing it.
        result[path.relative_to(root.resolve()).as_posix()] = path.read_bytes().decode(
            "utf-8"
        )
    return result


def require_complete(value: object) -> None:
    if isinstance(value, dict):
        for key, nested in value.items():
            if (
                key in {"dropped", "truncated", "isError", "positions_degraded"}
                and nested
            ):
                raise ValueError(f"incomplete server result: {key}")
            require_complete(nested)
    elif isinstance(value, list):
        for nested in value:
            require_complete(nested)


def _offset(text: str, position: dict, encoding: str) -> int:
    line, character = position["line"], position["character"]
    if type(line) is not int or type(character) is not int or min(line, character) < 0:
        raise ValueError("invalid source position")
    lines = text.splitlines(keepends=True)
    if not lines or text.endswith(("\n", "\r")):
        lines.append("")
    if line >= len(lines):
        raise ValueError("stale source line")
    content = lines[line].rstrip("\r\n")
    consumed = 0
    for index, char in enumerate(content):
        if consumed == character:
            return sum(map(len, lines[:line])) + index
        consumed += (
            len(char.encode("utf-16-le")) // 2
            if encoding == "utf-16"
            else len(char.encode("utf-8"))
        )
        if consumed > character:
            raise ValueError("position splits an encoded character")
    if consumed != character:
        raise ValueError("stale source character")
    return sum(map(len, lines[:line])) + len(content)


def preview(
    root: Path,
    source_cut: dict[str, str],
    result: dict,
    *,
    producer: str,
    encoding: str = "utf-16",
) -> dict:
    """Normalize mcpls WorkspaceEdits; incomplete/unknown edits are never dropped."""
    if encoding not in {"utf-8", "utf-16"}:
        raise ValueError("unsupported position encoding")
    require_complete(result)
    for name, content in source_cut.items():
        if owned_path(root, name).read_bytes().decode("utf-8") != content:
            raise ValueError("source changed while the edit request was running")
    changes = result.get("changes")
    if not isinstance(changes, list) or not changes:
        raise ValueError("result does not contain a nonempty complete edit list")
    if result.get("documentChanges"):
        raise ValueError("untranslated document/resource changes")
    grouped: dict[str, list] = {}
    for change in changes:
        if not isinstance(change, dict) or set(change) - {"uri", "edits", "version"}:
            raise ValueError("unknown edit shape")
        if change.get("version") is not None:
            raise ValueError(
                "document version cannot be reconciled with this disk source cut"
            )
        path = owned_path(root, change["uri"])
        relative = path.relative_to(root.resolve()).as_posix()
        if relative not in source_cut:
            raise ValueError(
                "edit affects a source absent from the pre-request owned cut"
            )
        if path.read_bytes().decode("utf-8") != source_cut[relative]:
            raise ValueError("source changed while the edit request was running")
        edits = change.get("edits")
        if not isinstance(edits, list) or not edits:
            raise ValueError("empty or malformed file edit list")
        grouped.setdefault(relative, []).extend(edits)
    files = []
    for name, edits in sorted(grouped.items()):
        before = source_cut[name]
        normalized: list[Edit] = []
        for edit in edits:
            if set(edit) - {"range", "new_text", "newText", "annotationId"} or edit.get(
                "annotationId"
            ):
                raise ValueError("unsupported annotated edit")
            if "new_text" in edit and "newText" in edit:
                raise ValueError("ambiguous replacement text keys")
            replacement = edit.get("new_text", edit.get("newText"))
            if not isinstance(replacement, str):
                raise ValueError("edit lacks replacement text")  # noqa: TRY004 -- malformed native edit retains the existing value-refusal contract.
            start = _offset(before, edit["range"]["start"], encoding)
            end = _offset(before, edit["range"]["end"], encoding)
            if start > end:
                raise ValueError("reversed edit range")
            normalized.append({"start": start, "end": end, "text": replacement})
        normalized.sort(key=lambda e: (e["start"], e["end"]))
        for left, right in pairwise(normalized):
            if left["end"] > right["start"]:
                raise ValueError("overlapping or ambiguous edits")
        after = before
        for edit in reversed(normalized):
            after = after[: edit["start"]] + edit["text"] + after[edit["end"] :]
        if after == before:
            raise ValueError("edit result does not change source")
        files.append(
            {
                "path": name,
                "before_sha256": digest(before),
                "after_sha256": digest(after),
                "before": before,
                "after": after,
                "edits": normalized,
            }
        )
    return {
        "schema": 1,
        "root": str(root.resolve()),
        "producer": producer,
        "position_encoding": encoding,
        "basis": {name: digest(content) for name, content in source_cut.items()},
        "files": files,
    }


def replacements(
    root: Path, source_cut: dict[str, str], outputs: dict[str, str], *, producer: str
) -> dict:
    changes = []
    for name, after in outputs.items():
        before = source_cut[name]
        if before == after:
            continue
        end_line = before.count("\n")
        end_character = len(before.rsplit("\n", 1)[-1].encode("utf-16-le")) // 2
        changes.append(
            {
                "uri": (root.resolve() / name).as_uri(),
                "edits": [
                    {
                        "range": {
                            "start": {"line": 0, "character": 0},
                            "end": {"line": end_line, "character": end_character},
                        },
                        "new_text": after,
                    }
                ],
            }
        )
    return preview(root, source_cut, {"changes": changes}, producer=producer)


def validate(selected: dict) -> Path:
    if (
        selected.get("schema") != 1
        or not selected.get("files")
        or not isinstance(selected.get("basis"), dict)
        or not selected["basis"]
    ):
        raise ValueError("invalid edit preview")
    root = Path(selected["root"])
    names = set()
    for name in selected.get("basis", {}):
        owned_path(root, name, allow_missing=True)
    for file in selected["files"]:
        owned_path(root, file["path"], allow_missing=True)
        if str(file["path"]) in names:
            raise ValueError("duplicate file in preview")
        names.add(file["path"])
        if selected["basis"].get(file["path"]) != file["before_sha256"]:
            raise ValueError("edited source differs from the preview basis")
        if (
            digest(file["before"]) != file["before_sha256"]
            or digest(file["after"]) != file["after_sha256"]
        ):
            raise ValueError("preview content digest mismatch")
        calculated = file["before"]
        if not file["edits"]:
            raise ValueError("preview file has no edits")
        previous = len(calculated)
        for edit in reversed(file["edits"]):
            if not 0 <= edit["start"] <= edit["end"] <= previous:
                raise ValueError("invalid preview edit offsets")
            previous = edit["start"]
            calculated = (
                calculated[: edit["start"]] + edit["text"] + calculated[edit["end"] :]
            )
        if calculated != file["after"]:
            raise ValueError("preview edits differ from approved output")
    return root


def reconcile(selected: dict) -> dict:
    root = validate(selected)
    files = []
    for file in selected["files"]:
        path = owned_path(root, file["path"], allow_missing=True)
        current = (
            hashlib.sha256(path.read_bytes()).hexdigest() if path.is_file() else None
        )
        state = (
            "missing"
            if current is None
            else "applied"
            if current == file["after_sha256"]
            else "unchanged"
            if current == file["before_sha256"]
            else "diverged"
        )
        files.append(
            {
                "path": file["path"],
                "state": state,
                "sha256": current,
                "edits": len(file["edits"]) if state == "applied" else 0,
            }
        )
    edited_names = {file["path"] for file in files}
    basis = []
    for name, expected in selected["basis"].items():
        if name in edited_names:
            continue
        path = owned_path(root, name, allow_missing=True)
        current = (
            hashlib.sha256(path.read_bytes()).hexdigest() if path.is_file() else None
        )
        basis.append(
            {
                "path": name,
                "state": "missing"
                if current is None
                else "unchanged"
                if current == expected
                else "diverged",
                "sha256": current,
            }
        )
    complete = all(file["state"] == "applied" for file in files) and all(
        item["state"] == "unchanged" for item in basis
    )
    return {
        "complete": complete,
        "status": "complete" if complete else "incomplete",
        "preview_sha256": digest(json.dumps(selected, sort_keys=True)),
        "files": files,
        "basis": basis,
    }


def _write(path: Path, content: bytes) -> None:
    with tempfile.NamedTemporaryFile(
        dir=path.parent, prefix=".semantic-edit-", delete=False
    ) as target:
        temporary = Path(target.name)
        try:
            target.write(content)
            target.flush()
            os.fsync(target.fileno())
            temporary.chmod(path.stat().st_mode)
            temporary.replace(path)
        finally:
            temporary.unlink(missing_ok=True)


def apply(selected: dict, receipt: Path) -> dict:
    """Apply an explicitly selected reviewed preview, retaining interrupted reconciliation."""
    root = validate(selected)
    for name, expected in selected.get("basis", {}).items():
        if digest(owned_path(root, name).read_bytes().decode("utf-8")) != expected:
            raise ValueError("stale preview basis; rebuild and review before applying")
    for file in selected["files"]:
        if (
            owned_path(root, file["path"]).read_bytes().decode("utf-8")
            != file["before"]
        ):
            raise ValueError("stale preview; rebuild and review before applying")
    source_paths = {
        owned_path(root, file["path"]).resolve() for file in selected["files"]
    }
    source_paths.update(
        owned_path(root, name).resolve() for name in selected.get("basis", {})
    )
    if receipt.resolve() in source_paths:
        raise ValueError("receipt must not overwrite an edited source")
    if receipt.is_symlink():
        raise ValueError("receipt must not be a symlink")
    # Establish the reconciliation destination before any source mutation.
    receipt.write_text(json.dumps(reconcile(selected), indent=2) + "\n")
    try:
        for file in selected["files"]:
            path = owned_path(root, file["path"])
            if path.read_bytes().decode("utf-8") != file["before"]:
                raise ValueError("source drift during application")
            _write(path, file["after"].encode("utf-8"))
    finally:
        result = reconcile(selected)
        receipt.write_text(json.dumps(result, indent=2) + "\n")
    return result
