# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Bounded, resumable local Codex history observations and selected recall.

The caller may retain the JSON checkpoint as a disposable discovery/read index. It
contains metadata, hashes and locators, never prompts, arguments or output text.
No Codex process, thread, database or global memory is opened for writing.
"""

from __future__ import annotations

import collections
import copy
import datetime as dt
import hashlib
import json
import os
import re
import time
from pathlib import Path
from typing import NotRequired, TypedDict, Unpack, cast


class HistorySelection(TypedDict):
    roots: list[str]
    checkouts: list[str]
    since: str
    until: str
    query_digest: str | None
    record_limit: int


class HistoryDirectory(TypedDict):
    root: int
    path: str
    position: int
    identity: NotRequired[list[int]]


class HistoryRecord(TypedDict, total=False):
    offset: int
    end: int
    timestamp: str
    role: str
    record_type: str | None
    digest: str
    content_digest: str
    message_role: str
    call_id: str
    explicit_origin: str
    tool: str
    classes: list[str]
    output_digest: str
    match: bool
    match_character_offset: int
    root: int
    relative: str
    identity: list[int]
    session_id: str
    originating_session_id: str
    inherited_sources: list[HistoryRecord]


class HistoryHeader(TypedDict):
    session_id: str
    parent_session_id: str | None
    direct: bool


class HistorySource(TypedDict):
    root: int
    relative: str
    identity: list[int]
    offset: int
    records: list[HistoryRecord]
    gaps: list[str]
    done: bool
    generation: int
    session_id: NotRequired[str]
    parent_session_id: NotRequired[str | None]
    direct: NotRequired[bool]
    skipping: NotRequired[bool]


class HistoryCheckpoint(TypedDict):
    schema: str
    selection: HistorySelection
    until: str
    frontier: list[HistoryDirectory]
    sources: list[HistorySource]
    discovery_complete: bool
    read_complete: bool
    generation: int
    discovery_gaps: list[str]


class InspectOptions(TypedDict, total=False):
    hours: float
    now: dt.datetime | None
    related_checkouts: tuple[Path, ...]
    archived_sessions: Path | None
    continuation: dict | None
    max_entries: int
    max_files: int
    max_bytes: int
    max_records: int
    max_record_bytes: int
    seconds: float


def _read_header(raw: bytes, maximum: int, checkouts: set[str], relative: str) -> HistoryHeader:
    if not raw.endswith(b"\n") or len(raw) > maximum:
        raise ValueError("oversized_or_partial_header")
    return _header(json.loads(raw), checkouts, relative)


_SCHEMA = "session-retrospective-v2"
_ERRORS = (
    ("No space left on device", "disk_full"),
    ("No such file or directory", "missing_path"),
    ("Permission denied", "permission"),
    ("Blocking waiting for file lock", "cargo_lock_wait"),
    ("could not compile", "compile_failure"),
    ("out of memory", "memory"),
    ("Traceback", "python_traceback"),
)


def _identity(path: Path) -> list[int]:
    stat = path.stat()
    return [stat.st_dev, stat.st_ino, stat.st_size, stat.st_mtime_ns, stat.st_ctime_ns]


def _path(roots: list[Path], root: int, relative: str) -> Path:
    """A saved cursor never grants access outside its selected roots or via links."""
    candidate = roots[root] / relative
    if Path(relative).is_absolute() or ".." in Path(relative).parts:
        raise ValueError("history locator escapes selected root")
    current = roots[root]
    for part in Path(relative).parts:
        current /= part
        if current.is_symlink():
            raise ValueError("history source symlink refused")
    if not candidate.resolve().is_relative_to(roots[root]):
        raise ValueError("history source escapes selected root")
    return candidate


def _stamp(value: object) -> dt.datetime | None:
    try:
        stamp = dt.datetime.fromisoformat(str(value))
    except ValueError:
        return None
    else:
        return stamp if stamp.tzinfo else None


def _header(item: dict, checkouts: set[str], fallback: str) -> HistoryHeader:
    payload = item.get("payload", {})
    if item.get("type") != "session_meta" or not isinstance(payload, dict):
        raise ValueError("missing session metadata")
    parent = payload.get("parent_thread_id")
    source = payload.get("source")
    if not parent and isinstance(source, dict):
        subagent = source.get("subagent", {})
        if isinstance(subagent, dict):
            spawn = subagent.get("thread_spawn", {})
            if isinstance(spawn, dict):
                parent = spawn.get("parent_thread_id")
    workspace = payload.get("runtime_workspace_roots", [])
    return {
        "session_id": str(payload.get("id") or payload.get("session_id") or fallback),
        "parent_session_id": parent if isinstance(parent, str) else None,
        "direct": payload.get("cwd") in checkouts
        or (
            isinstance(workspace, list)
            and any(root in checkouts for root in workspace if isinstance(root, str))
        ),
    }


def _association(sources: list[HistorySource]) -> dict[str, str]:
    selected = {
        source["session_id"]: "explicit_checkout_or_workspace"
        for source in sources
        if source.get("direct")
    }
    while True:
        additions = {
            source["session_id"]: "parent_session"
            for source in sources
            if source.get("parent_session_id") in selected
            and source["session_id"] not in selected
        }
        if not additions:
            return selected
        selected.update(additions)


def _origin(source: HistorySource, parents: dict[str, str | None]) -> str:
    origin = source["session_id"]
    seen = set()
    parent = parents.get(origin)
    while parent and origin not in seen:
        seen.add(origin)
        origin = parent
        parent = parents.get(origin)
    return origin


def _observation(
    item: dict, _source: HistorySource, start: int, end: int, query: str | None
) -> HistoryRecord:
    payload = item.get("payload", {})
    if not isinstance(payload, dict):
        payload = {}
    role = str(payload.get("type") or item.get("type"))
    record: HistoryRecord = {
        "offset": start,
        "end": end,
        "timestamp": item["timestamp"],
        "role": role,
        "record_type": item.get("type"),
        "digest": hashlib.sha256(json.dumps(item, sort_keys=True).encode()).hexdigest(),
        "content_digest": hashlib.sha256(
            json.dumps(payload, sort_keys=True).encode()
        ).hexdigest(),
    }
    if payload.get("role") in {"assistant", "user", "system", "developer", "tool"}:
        record["message_role"] = payload["role"]
    call = payload.get("call_id")
    if isinstance(call, str):
        record["call_id"] = call
    origin = payload.get("origin_session_id") or payload.get("origin_thread_id")
    if isinstance(origin, str):
        record["explicit_origin"] = origin
    if role in {"function_call", "custom_tool_call"}:
        name = payload.get("name")
        if isinstance(name, str) and re.fullmatch(r"[A-Za-z0-9_.-]{1,160}", name):
            record["tool"] = name
    if role in {"function_call_output", "custom_tool_call_output"}:
        output = payload.get("output", "")
        text = output if isinstance(output, str) else json.dumps(output)
        codes = {
            int(value) for value in re.findall(r'"exit_code"\s*:\s*(-?\d+)', text)
        } - {0}
        record["classes"] = [f"observed_exit_{code}" for code in sorted(codes)] + [
            category
            for needle, category in _ERRORS
            if needle.casefold() in text.casefold()
        ]
        # Several distinct outputs/polls for one call remain distinct observations.
        record["output_digest"] = hashlib.sha256(text.encode()).hexdigest()
    if query:
        # Only an explicitly supplied query inspects conversational text for recall.
        text = json.dumps(payload, ensure_ascii=False)
        match = re.search(re.escape(query), text, re.IGNORECASE)
        record["match"] = match is not None
        if match:
            record["match_character_offset"] = match.start()
    return record


def inspect(
    sessions: Path,
    checkout: Path,
    *,
    hours: float = 24,
    now: dt.datetime | None = None,
    related_checkouts: tuple[Path, ...] = (),
    archived_sessions: Path | None = None,
    continuation: dict | None = None,
    query: str | None = None,
    max_entries: int = 4096,
    max_files: int = 128,
    max_bytes: int = 128 << 20,
    max_records: int = 10000,
    max_record_bytes: int = 1 << 20,
    seconds: float = 10,
) -> dict:
    """Observe activity, regardless of session creation date, with resumable bounds.

    Resume with ``continuation=result['continuation']`` and the same selection;
    results are cumulative. ``checkpoint`` can also refresh a completed scan,
    reusing unchanged reads and invalidating changed/deleted sources. A complete
    negative requires discovery_complete AND read_complete AND no source gaps.
    Paths/worktrees must be explicitly supplied; parent links admit descendants.
    """
    if not 0 < hours <= 168:
        raise ValueError("retrospective hours must be positive and at most 168")
    if (
        min(max_entries, max_files, max_records, max_record_bytes) <= 0
        or max_bytes < max_record_bytes + 1
        or seconds <= 0
    ):
        raise ValueError("invalid history scan budget (bytes must exceed record limit)")
    if query is not None and (not query.strip() or len(query) > 1000):
        raise ValueError("recall query must contain 1 to 1000 characters")
    sessions = sessions.resolve()
    archived = archived_sessions or (
        sessions.parent / "archived_sessions" if sessions.name == "sessions" else None
    )
    roots = [sessions] + ([archived.resolve()] if archived else [])
    checkouts = {
        str(checkout.resolve()),
        *(str(path.resolve()) for path in related_checkouts),
    }
    if continuation and now is None:
        now = _stamp(continuation.get("until"))
    now = now or dt.datetime.now(dt.UTC)
    if now.tzinfo is None:
        raise ValueError("history interval needs timezone-aware timestamps")
    since = now - dt.timedelta(hours=hours)
    selection: HistorySelection = {
        "roots": [str(root) for root in roots],
        "checkouts": sorted(checkouts),
        "since": since.isoformat(),
        "until": now.isoformat(),
        "query_digest": hashlib.sha256(query.encode()).hexdigest() if query else None,
        "record_limit": max_record_bytes,
    }
    if continuation:
        if (
            continuation.get("schema") != _SCHEMA
            or continuation.get("selection") != selection
        ):
            raise ValueError("history continuation belongs to another selection")
        state: HistoryCheckpoint = cast("HistoryCheckpoint", copy.deepcopy(continuation))
        if state["discovery_complete"] and state["read_complete"]:
            state["frontier"] = [
                {"root": i, "path": ".", "position": 0} for i in range(len(roots))
            ]
            state["discovery_complete"] = False
            state["read_complete"] = False
            state["generation"] += 1
            state["discovery_gaps"] = []
    else:
        state = {
            "schema": _SCHEMA,
            "selection": selection,
            "until": now.isoformat(),
            "frontier": [
                {"root": i, "path": ".", "position": 0} for i in range(len(roots))
            ],
            "sources": [],
            "discovery_complete": False,
            "read_complete": False,
            "generation": 1,
            "discovery_gaps": [],
        }
    deadline = time.monotonic() + seconds
    entries = files = read_bytes = read_records = 0
    sources = state["sources"]
    by_path = {(source["root"], source["relative"]): source for source in sources}
    while (
        state["frontier"]
        and entries < max_entries
        and files < max_files
        and read_bytes < max_bytes
        and time.monotonic() < deadline
    ):
        directory = state["frontier"][0]
        try:
            path = _path(roots, directory["root"], directory["path"])
            stat = _identity(path)
            if directory.get("identity") and directory["identity"] != stat:
                directory["position"] = 0
                state["discovery_gaps"].append("directory_changed_during_discovery")
            directory["identity"] = stat
            exhausted = True
            with os.scandir(path) as children:
                for position, entry in enumerate(children):
                    if position < directory["position"]:
                        continue
                    if (
                        entries >= max_entries
                        or files >= max_files
                        or read_bytes + max_record_bytes + 1 > max_bytes
                        or time.monotonic() >= deadline
                    ):
                        exhausted = False
                        break
                    entries += 1
                    directory["position"] = position + 1
                    relative = str(Path(directory["path"]) / entry.name)
                    if entry.is_symlink():
                        state["discovery_gaps"].append("symlink_refused")
                        continue
                    if entry.is_dir(follow_symlinks=False):
                        state["frontier"].append(
                            {"root": directory["root"], "path": relative, "position": 0}
                        )
                    elif entry.is_file(follow_symlinks=False) and entry.name.endswith(
                        ".jsonl"
                    ):
                        files += 1
                        source_path = _path(roots, directory["root"], relative)
                        identity = _identity(source_path)
                        key = (directory["root"], relative)
                        existing = by_path.get(key)
                        if existing and existing["identity"] == identity:
                            existing["generation"] = state["generation"]
                            continue
                        source: HistorySource = {
                            "root": key[0],
                            "relative": key[1],
                            "identity": identity,
                            "offset": 0,
                            "records": [],
                            "gaps": [],
                            "done": False,
                            "generation": state["generation"],
                        }
                        try:
                            with source_path.open("rb") as stream:
                                raw = stream.readline(max_record_bytes + 1)
                            read_bytes += len(raw)
                            header = _read_header(raw, max_record_bytes, checkouts, relative)
                            source.update(session_id=header["session_id"], parent_session_id=header["parent_session_id"], direct=header["direct"])
                        except (ValueError, TypeError, AttributeError) as error:
                            source["gaps"] = [
                                "oversized_or_partial_header"
                                if str(error) == "oversized_or_partial_header"
                                else "invalid_header"
                            ]
                            source["done"] = True
                        if existing:
                            sources[sources.index(existing)] = source
                        else:
                            sources.append(source)
                        by_path[key] = source
            if exhausted:
                state["frontier"].pop(0)
            else:
                break
        except FileNotFoundError:
            # An absent optional archive is a known empty source, not failed access.
            if directory["root"] == 0 or directory["path"] != ".":
                state["discovery_gaps"].append("unavailable_directory")
            state["frontier"].pop(0)
        except (OSError, ValueError):
            state["discovery_gaps"].append("unavailable_directory")
            state["frontier"].pop(0)
    if not state["frontier"]:
        state["discovery_complete"] = True
        sources[:] = [
            source for source in sources if source["generation"] == state["generation"]
        ]
    current_sources = [
        source for source in sources if source["generation"] == state["generation"]
    ]
    selected = _association(current_sources)
    # Population must be known before a foreign-cwd child can be ruled out.
    if state["discovery_complete"]:
        for source in current_sources:
            if source.get("session_id") not in selected or source["done"]:
                continue
            if (
                read_bytes + max_record_bytes + 1 > max_bytes
                or read_records >= max_records
                or time.monotonic() >= deadline
            ):
                break
            files += 1
            try:
                path = _path(roots, source["root"], source["relative"])
                if _identity(path) != source["identity"]:
                    source["gaps"].append("source_changed_during_scan")
                    source["done"] = True
                    continue
                with path.open("rb") as stream:
                    stream.seek(source["offset"])
                    while (
                        read_bytes + max_record_bytes + 1 <= max_bytes
                        and read_records < max_records
                        and time.monotonic() < deadline
                    ):
                        start = stream.tell()
                        raw = stream.readline(max_record_bytes + 1)
                        read_bytes += len(raw)
                        if not raw:
                            source["done"] = True
                            break
                        source["offset"] = stream.tell()
                        if source.get("skipping"):
                            if raw.endswith(b"\n"):
                                source["skipping"] = False
                            continue
                        if not raw.endswith(b"\n") or len(raw) > max_record_bytes:
                            source["gaps"].append("oversized_or_partial_record")
                            source["skipping"] = not raw.endswith(b"\n")
                            continue
                        read_records += 1
                        try:
                            item = json.loads(raw)
                            stamp = _stamp(item.get("timestamp"))
                            if stamp and since <= stamp <= now:
                                source["records"].append(
                                    _observation(
                                        item, source, start, stream.tell(), query
                                    )
                                )
                        except (ValueError, TypeError, AttributeError, KeyError):
                            source["gaps"].append("invalid_record")
                    if _identity(path) != source["identity"]:
                        source["records"] = []
                        source["gaps"].append("source_changed_during_scan")
                        source["done"] = True
            except (OSError, ValueError):
                source["gaps"].append("unavailable_source")
                source["done"] = True
        state["read_complete"] = all(
            source["done"] for source in sources if source.get("session_id") in selected
        )
    tools: collections.Counter[str] = collections.Counter()
    classes: collections.Counter[str] = collections.Counter()
    seen: dict[tuple[object, ...], HistoryRecord] = {}
    duplicates = records = 0
    hits: list[HistoryRecord] = []
    projected: list[dict[str, object]] = []
    # A new child call originates in that child. Only a matching call actually
    # present in an ancestor (or explicit record provenance) proves inheritance.
    calls: dict[tuple[str, str], set[str]] = {}
    parents = {
        source["session_id"]: source.get("parent_session_id")
        for source in current_sources
        if "session_id" in source
    }
    for source in current_sources:
        for record in source["records"]:
            if record.get("call_id"):
                calls.setdefault(
                    (_origin(source, parents), record["call_id"]), set()
                ).add(source["session_id"])

    def call_origin(source: HistorySource, record: HistoryRecord) -> str:
        if record.get("explicit_origin"):
            return record["explicit_origin"]
        origin = source["session_id"]
        call = record.get("call_id")
        if call is None:
            return origin
        candidates = calls.get((_origin(source, parents), call), set())
        ancestor: str | None = origin
        visited = set()
        while ancestor and ancestor not in visited:
            visited.add(ancestor)
            if ancestor in candidates:
                origin = ancestor
            ancestor = parents.get(ancestor)
        return origin

    def ancestry_depth(source: HistorySource) -> int:
        session = source.get("session_id")
        ancestors = set()
        while session and session not in ancestors:
            ancestors.add(session)
            session = parents.get(session)
        return len(ancestors)

    for source in sorted(current_sources, key=ancestry_depth):
        session = source.get("session_id")
        if session not in selected:
            continue
        projected.append(
            {
                "path": str(roots[source["root"]] / source["relative"]),
                "session_id": session,
                "parent_session_id": source.get("parent_session_id"),
                "association": selected[session],
                "identity": source["identity"],
                "read_position": source["offset"],
                "read_complete": source["done"],
                "gaps": sorted(set(source["gaps"])),
                "retained_records": len(source["records"]),
            }
        )
        for record in source["records"]:
            call = record.get("call_id")
            origin = (
                call_origin(source, record)
                if call
                else record.get("explicit_origin", session)
            )
            # No call ID means no evidence of inheritance: identical text is not identity.
            key = (
                (origin, call, record["role"], record["content_digest"])
                if call
                else (session, source["root"], source["relative"], record["offset"])
            )
            locator: HistoryRecord = {
                "root": source["root"],
                "relative": source["relative"],
                "identity": source["identity"],
                "session_id": session,
                "originating_session_id": origin,
                **record,
            }
            if key in seen:
                duplicates += 1
                seen[key]["inherited_sources"].append(
                    {
                        "session_id": session,
                        "root": source["root"],
                        "relative": source["relative"],
                        "offset": record["offset"],
                    }
                )
                continue
            locator["inherited_sources"] = []
            seen[key] = locator
            records += 1
            if "tool" in record:
                tools[record["tool"]] += 1
            classes.update(record.get("classes", []))
            if record.get("match"):
                hits.append(locator)
    gaps = sorted(
        set(
            state["discovery_gaps"]
            + [gap for source in sources for gap in source["gaps"]]
        )
    )
    complete = state["discovery_complete"] and state["read_complete"]
    return {
        "schema": _SCHEMA,
        "checkout": str(checkout),
        "since": since.isoformat(),
        "until": now.isoformat(),
        "sources": projected,
        "records": records,
        "inherited_duplicates_removed": duplicates,
        "tool_calls": dict(tools.most_common(30)),
        "observed_record_classes": dict(classes),
        "truncated": not complete or bool(gaps),
        "oversized_or_partial_records": sum(
            source["gaps"].count("oversized_or_partial_record") for source in sources
        ),
        "recall_matches": hits[:100],
        "recall_matches_omitted": max(0, len(hits) - 100),
        "discovery": {
            "complete": state["discovery_complete"],
            "roots": selection["roots"],
            "sources_discovered": len(sources),
            "entries_this_page": entries,
            "gaps": gaps,
        },
        "read": {
            "complete": state["read_complete"],
            "bytes_this_page": read_bytes,
            "records_this_page": read_records,
        },
        "continuation": None if complete else state,
        "checkpoint": state,
        "coverage": "All date directories and selected archive roots; metadata never proves inactivity. Record timestamps and explicit checkout/workspace/parent associations select activity. Patterns, retries and polls are observations, not incidents, productivity totals or current authorization. Raw text requires an explicit selected excerpt read.",
    }


def recall(sessions: Path, checkout: Path, query: str, **options: Unpack[InspectOptions]) -> dict:
    """Find selected literal rationale text, returning exact attribution, not text."""
    return inspect(sessions, checkout, query=query, **options)


def read_excerpt(
    sessions: Path,
    locator: dict,
    *,
    archived_sessions: Path | None = None,
    max_characters: int = 4000,
) -> dict:
    """Explicitly read one selected record, refusing changed or escaped sources.

    This returns raw historical content and can contain sensitive material. The
    caller selects the locator and controls where its excerpt is displayed/stored.
    """
    if not 0 < max_characters <= 16000:
        raise ValueError("excerpt character limit must be between 1 and 16000")
    sessions = sessions.resolve()
    archive = archived_sessions or (
        sessions.parent / "archived_sessions" if sessions.name == "sessions" else None
    )
    roots = [sessions] + ([archive.resolve()] if archive else [])
    try:
        path = _path(roots, locator["root"], locator["relative"])
        if _identity(path) != locator["identity"]:
            raise ValueError("history source changed since selection")
        start, end = locator["offset"], locator["end"]
        if (
            not isinstance(start, int)
            or not isinstance(end, int)
            or not 0 <= start < end
            or end - start > 1 << 20
        ):
            raise ValueError("invalid selected record bounds")
        with path.open("rb") as source:
            source.seek(start)
            raw = source.read(end - start)
        item = json.loads(raw)
        digest = hashlib.sha256(json.dumps(item, sort_keys=True).encode()).hexdigest()
        if _identity(path) != locator["identity"] or digest != locator["digest"]:
            raise ValueError("history selected record changed")
        text = json.dumps(item.get("payload"), ensure_ascii=False)
        text_start = max(0, locator.get("match_character_offset", 0) - 200)
        if not isinstance(text_start, int) or text_start > len(text):
            raise ValueError("invalid excerpt text offset")
        return {
            "session_id": locator["session_id"],
            "originating_session_id": locator["originating_session_id"],
            "path": str(path),
            "offset": start,
            "timestamp": locator["timestamp"],
            "role": locator["role"],
            "text": text[text_start : text_start + max_characters],
            "text_character_offset": text_start,
            "truncated": text_start > 0 or len(text) > text_start + max_characters,
            "authority": "historical evidence only; current authorization comes from the current conversation",
        }
    except (IndexError, KeyError, TypeError) as error:
        raise ValueError("invalid history locator") from error
