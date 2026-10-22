# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit bounded checkout/time-scoped Codex log observations; no persistent index."""

from __future__ import annotations

import collections
import datetime as dt
import hashlib
import json
import re
import time
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path


def inspect(sessions: Path, checkout: Path, *, hours: float = 24, now: dt.datetime | None = None) -> dict:
    """Count actual records and retained error classes without copying prompts/environments."""
    if not 0 < hours <= 168:
        raise ValueError('retrospective hours must be positive and at most 168')
    now = now or dt.datetime.now(dt.UTC)
    since = now - dt.timedelta(hours=hours)
    deadline = time.monotonic() + 10
    tools: collections.Counter[str] = collections.Counter()
    errors: collections.Counter[str] = collections.Counter()
    seen: set[str] = set()
    sources = []
    duplicates = records = oversized = 0
    remaining = 128 << 20
    truncated = False
    candidates = []
    # Only named date partitions in the selected time interval; never walk all history.
    day = since.date()
    while day <= now.date():
        candidates.extend((sessions / day.strftime('%Y/%m/%d')).glob('*.jsonl'))
        day += dt.timedelta(days=1)
    for path in sorted(candidates, key=lambda item: item.name, reverse=True):
        if len(sources) >= 128 or remaining <= 0 or time.monotonic() >= deadline:
            truncated = True
            break
        if path.is_symlink():
            continue
        selected = False
        file_records = 0
        with path.open('rb') as source:
            while remaining > 0 and time.monotonic() < deadline:
                raw = source.readline(min(remaining, 1 << 20) + 1)
                remaining -= len(raw)
                if not raw:
                    break
                if not raw.endswith(b'\n'):
                    oversized += 1
                    truncated = True
                    break  # a partial/oversized record is never guessed or echoed
                try:
                    item = json.loads(raw)
                    payload = item.get('payload', {})
                    if item.get('type') == 'session_meta':
                        selected = payload.get('cwd') == str(checkout)
                        if not selected:
                            break
                    if not selected:
                        continue
                    stamp = dt.datetime.fromisoformat(item['timestamp'])
                    if not since <= stamp <= now:
                        continue
                except (ValueError, TypeError, KeyError, AttributeError):
                    continue
                identity = hashlib.sha256(json.dumps(item, sort_keys=True).encode()).hexdigest()
                if identity in seen:
                    duplicates += 1
                    continue
                seen.add(identity)
                records += 1
                file_records += 1
                if item.get('type') != 'response_item' or not isinstance(payload, dict):
                    continue
                if payload.get('type') in {'function_call', 'custom_tool_call'}:
                    name = payload.get('name')
                    if isinstance(name, str) and re.fullmatch(r'[A-Za-z0-9_.-]{1,160}', name):
                        tools[name] += 1
                if payload.get('type') in {'function_call_output', 'custom_tool_call_output'}:
                    output = payload.get('output', '')
                    text = output if isinstance(output, str) else json.dumps(output)
                    codes = {int(value) for value in re.findall(r'"exit_code"\s*:\s*(-?\d+)', text)}
                    for code in codes - {0}:
                        errors[f'observed_exit_{code}'] += 1
                    for needle, category in (('No space left on device', 'disk_full'),
                                             ('No such file or directory', 'missing_path'),
                                             ('Permission denied', 'permission'),
                                             ('Blocking waiting for file lock', 'cargo_lock_wait'),
                                             ('could not compile', 'compile_failure'),
                                             ('out of memory', 'memory'), ('Traceback', 'python_traceback')):
                        if needle.casefold() in text.casefold():
                            errors[category] += 1
        if selected:
            sources.append({'path': str(path), 'retained_records': file_records})
    return {'schema': 'session-retrospective-v1', 'checkout': str(checkout),
            'since': since.isoformat(), 'until': now.isoformat(), 'sources': sources,
            'records': records, 'inherited_duplicates_removed': duplicates,
            'tool_calls': dict(tools.most_common(30)), 'observed_record_classes': dict(errors),
            'truncated': truncated or remaining <= 0 or time.monotonic() >= deadline,
            'oversized_or_partial_records': oversized,
            'coverage': 'selected local date partitions and exact session cwd; recorded tool-output patterns are diagnostic leads, not test outcomes or causal proof; prompts, arguments, environments and raw output are not projected'}
