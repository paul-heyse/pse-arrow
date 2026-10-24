# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Bounded detached long poll of one exact owner; cursors confer no authority.

Snapshots are reconciled on every connection. Producers do not retain an event
log: intervening transitions may be missed, including while the observer polls.
No watcher owns, cancels, drains or resumes a workload.
"""

from __future__ import annotations

import base64
import hashlib
import json
import subprocess
import time
from collections.abc import Callable
from pathlib import Path

from scripts import (
    build_observation,
    host_admission,
    native_operation,
    run_inspection,
    test_resources,
)

MAX_WAIT = 30.0
POLL_SECONDS = 0.25


def stable(value: object) -> object:
    """Exclude observer ages/heartbeat times, retaining actual producer facts."""
    if isinstance(value, dict):
        return {
            key: stable(item)
            for key, item in value.items()
            if key not in {"age_seconds", "stage_age_seconds", "observed_at", "observed_monotonic"}
        }
    if isinstance(value, list):
        return [stable(item) for item in value]
    return value


def revision(value: object) -> str:
    return hashlib.sha256(json.dumps(stable(value), sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def selection(kind: str, handle: str, registry: Path | None = None) -> dict:
    """Bind an invocation handle to its registry, never to a similarly named run."""
    if kind == "run":
        root = registry if registry is not None else test_resources.registry()
        path = Path(handle)
        # Selection must remain representable when its owner is unavailable.
        # The result reader authenticates registry/declaration correspondence.
        if not test_resources.IDENTIFIER.fullmatch(handle) and path.absolute().parent == root / "invocations":
            handle = path.stem
        return {"kind": kind, "handle": handle, "registry": str(root)}
    if kind not in {"attempt", "unit"}:
        raise ValueError("unknown watch selection")
    result = {"kind": kind, "handle": handle}
    if kind == "attempt":
        result["admission_directory"] = str(host_admission.root_path())
    return result


def decode(token: str, selected: dict) -> dict:
    if len(token) > 16384:
        raise ValueError("watch cursor exceeds bound")
    try:
        value = json.loads(base64.b64decode(token.encode("ascii"), altchars=b"-_", validate=True))
    except (ValueError, UnicodeError) as error:
        raise ValueError("invalid watch cursor") from error
    if not isinstance(value, dict) or value.get("version") != 1 or value.get("selection") != selected:
        raise ValueError("watch cursor belongs to a different selection or version")
    if not isinstance(value.get("revision"), str) or len(value["revision"]) != 64 or any(c not in "0123456789abcdef" for c in value["revision"]):
        raise ValueError("invalid watch cursor revision")
    if not isinstance(value.get("observed_unix_ns"), int):
        raise ValueError("invalid watch cursor interval")
    return value


def snapshot(selected: dict, deadline: float) -> dict:
    kind, handle = selected["kind"], selected["handle"]
    try:
        if kind == "unit":
            fields = native_operation.unit_observation(
                handle,
                properties=("SubState", "Result", "ExecMainCode", "ExecMainStatus", "CPUUsageNSec", "MemoryCurrent"),
                deadline=deadline,
            )
            if fields["LoadState"] == "not-found":
                raise ValueError("selected unit disappeared; terminal and drain unknown")
            state = "running" if fields["ActiveState"] in {"active", "activating", "deactivating"} else "terminal-without-result"
            return {"state": state, "unit": fields, "result_reference": None}
        if kind == "attempt":
            observations = host_admission.admission_observations(Path(selected["admission_directory"]), attempt=handle)
            if len(observations) != 1:
                raise ValueError("selected admission attempt unavailable")
            attempt = observations[0]
            state = attempt["status"]
            if not attempt.get("caller_alive") and state == "waiting":
                state = "observation-lost"
            return {"state": state, "attempt": attempt, "result_reference": None}
        result = run_inspection.product_result(handle, registry=Path(selected["registry"]))
        terminal = result.get("terminal")
        state = "declared"
        if terminal is not None:
            state = "result-complete" if terminal.get("complete") else "result-incomplete"
        elif result.get("phase") in {"abandoned", "unresolved"}:
            state = "observation-lost"
        elif result.get("producer", {}).get("association") == "captured-metadata":
            state = "running" if host_admission.caller_alive(result["producer"]) else "observation-lost"
        # The result reader is authority; missing or returned Cargo observations
        # cannot turn an open product invocation into successful completion.
        value = {"state": state, "run": result, "result_reference": {
            "run_path": result["run_path"], "registry": result["registry"],
            "terminal_identity": result.get("terminal_identity"),
        } if terminal is not None else None}
        try:
            value["builds"] = [
                item for item in build_observation.observations(host_admission.root_path())
                if item.get("invocation_declaration") in {result["run_path"], result["invocation"]}
            ]
            value["build_coverage"] = "matching snapshots within owner's latest 100; no transition history"
        except (OSError, ValueError) as error:
            value["builds"] = None
            value["build_error"] = str(error)
        return value
    except (OSError, ValueError, TypeError, KeyError, RuntimeError, subprocess.SubprocessError) as error:
        return {"state": "observation-unavailable", "error": f"{type(error).__name__}: {error}", "result_reference": None}


def watch(
    selected: dict,
    *,
    cursor: str | None = None,
    max_wait: float = MAX_WAIT,
    observe: Callable[[dict, float], dict] = snapshot,
) -> dict:
    """Return once a meaningful snapshot changes, or after at most max_wait seconds.

    Source reads are bounded independently (not an event-delivery guarantee).
    KeyboardInterrupt propagates; the CLI only prints detached cancellation.
    """
    if not 0 <= max_wait <= MAX_WAIT:
        raise ValueError("watch max wait must be between 0 and 30 seconds")
    previous = decode(cursor, selected) if cursor is not None else None
    deadline = time.monotonic() + max_wait
    while True:
        # A zero-wait snapshot still permits one bounded manager read.
        current = observe(selected, max(deadline, time.monotonic() + 0.05))
        digest = revision(current)
        changed = previous is None or previous["revision"] != digest
        now = time.time_ns()
        if changed or time.monotonic() >= deadline:
            token = {"version": 1, "selection": selected, "revision": digest, "observed_unix_ns": now}
            return {
                "mode": "detached-no-write", "selection": selected,
                "changed": changed, "reason": "changed" if changed else "timeout",
                "cursor": base64.urlsafe_b64encode(json.dumps(token, sort_keys=True).encode()).decode(),
                "revision": digest, "observed_unix_ns": now,
                "reconciliation": {
                    "source": "current-owner-snapshots", "history": "unavailable",
                    "gap": "intermediate transitions may be missing; no replay guarantee",
                    "previous_observed_unix_ns": previous["observed_unix_ns"] if previous else None,
                },
                "snapshot": current,
            }
        time.sleep(min(POLL_SECONDS, max(0, deadline - time.monotonic())))
