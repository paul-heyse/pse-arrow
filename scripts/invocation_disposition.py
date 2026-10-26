# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Inspection-first exact invocation abandonment through existing resource owners."""

from __future__ import annotations

import argparse
import contextlib
import json
import os
import stat
import time
from pathlib import Path

from scripts import operation_lifetime
from scripts import host_admission as host
from scripts import native_operation as operation
from scripts import run_inspection, surreal_server, test_resources


def owner_reason(record: object, deadline: float) -> str | None:
    if not isinstance(record, dict) or not all(
        key in record for key in ("pid", "start", "boot")
    ):
        return "unknown execution owner; recover exact owner association"
    if host.caller_alive(record):
        return "live execution owner; cancel through its existing command owner"
    if record.get("scope") is None:
        return (
            "unknown physical scope; authenticate exact descendants before disposition"
        )
    observation = operation_lifetime.drained(record, deadline=deadline)
    return (
        None
        if observation.complete
        else "execution scope not drained: " + observation.diagnostic()
    )


def storage_observation(
    record: test_resources.ResourceRecord,
    *,
    deadline: float,
    seen: set[tuple[int, int]],
    limit: int = 512,
) -> dict:
    """Bound exact owned file metadata; never attribute a shared database tree."""
    result = {
        "status": "measured",
        "allocated_bytes": 0,
        "logical_bytes": 0,
        "exclusive_allocated_bytes": 0,
        "shared_allocated_bytes": 0,
        "entries": 0,
        "entry_limit": limit,
        "reasons": [],
        "scope": "non-atomic exact owned-path metadata; hardlinks counted once across this request",
    }
    directory = (
        record.get("directory")
        if record.get("kind") != "evidence"
        else record.get("state")
    )
    if not isinstance(directory, str):
        return {
            "status": "unavailable",
            "reason": "resource has no independent physical file owner; shared database/namespace bytes cannot be attributed to this invocation",
        }
    path = Path(directory)
    if (
        not path.is_absolute()
        or ".." in path.parts
        or any(item.is_symlink() for item in (path, *path.parents))
    ):
        return {
            "status": "unavailable",
            "reason": "exact resource path ownership changed",
        }

    def observe() -> dict:
        with contextlib.ExitStack() as contexts:
            if record.get("kind") == "evidence":
                root = contexts.enter_context(test_resources.report_directory(record))
            else:
                state = Path(record["state"])
                if state not in path.parents:
                    raise ValueError("control path is outside its exact declared state")
                config = surreal_server.config_for(state, deadline=deadline)
                if config.get("instance_id") != record.get("service_generation"):
                    raise ValueError(
                        "service generation changed; control bytes remain unattributed"
                    )
                flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC
                root = os.open(path.anchor, flags)
                contexts.callback(os.close, root)
                for component in path.parts[1:]:
                    root = os.open(component, flags, dir_fd=root)
                    contexts.callback(os.close, root)
            information = os.fstat(root)
            if information.st_uid != os.getuid():
                raise ValueError("physical resource owner changed")
            pending = [(root, 0)]
            while pending:
                current, depth = pending.pop()
                with os.scandir(current) as entries:
                    for entry in entries:
                        if result["entries"] >= limit or time.monotonic() >= deadline:
                            result["status"] = "partial"
                            result["reasons"].append("entry/deadline bound reached")
                            return result
                        result["entries"] += 1
                        selected = entry.stat(follow_symlinks=False)
                        if (
                            selected.st_uid != os.getuid()
                            or selected.st_dev != information.st_dev
                            or stat.S_ISLNK(selected.st_mode)
                        ):
                            result["status"] = "partial"
                            result["reasons"].append(
                                "changed owner, cross-device path or symlink not followed"
                            )
                            continue
                        identity = (selected.st_dev, selected.st_ino)
                        if stat.S_ISDIR(selected.st_mode):
                            if depth >= 16:
                                result["status"] = "partial"
                                result["reasons"].append(
                                    "directory depth bound reached"
                                )
                            else:
                                descriptor = os.open(
                                    entry.name,
                                    os.O_RDONLY
                                    | os.O_DIRECTORY
                                    | os.O_NOFOLLOW
                                    | os.O_CLOEXEC,
                                    dir_fd=current,
                                )
                                contexts.callback(os.close, descriptor)
                                actual = os.fstat(descriptor)
                                if (actual.st_dev, actual.st_ino) != identity:
                                    result["status"] = "partial"
                                    result["reasons"].append(
                                        "directory changed before traversal"
                                    )
                                    continue
                                pending.append((descriptor, depth + 1))
                        elif stat.S_ISREG(selected.st_mode) and identity not in seen:
                            seen.add(identity)
                            allocated = selected.st_blocks * 512
                            result["allocated_bytes"] += allocated
                            result["logical_bytes"] += selected.st_size
                            result[
                                "shared_allocated_bytes"
                                if selected.st_nlink > 1
                                else "exclusive_allocated_bytes"
                            ] += allocated
            after = os.fstat(root)
            if after.st_mtime_ns != information.st_mtime_ns:
                result["status"] = "partial"
                result["reasons"].append(
                    "directory changed during metadata observation"
                )
        return result

    try:
        return observe()
    except (OSError, ValueError, RuntimeError) as error:
        result.update(status="unavailable", reasons=[str(error)])
    return result


def associated_resource(identity: str, nonce: str) -> tuple:
    """Read the detached resource snapshot and require its exact invocation owner."""
    entry = test_resources.storage().resource_snapshot(identity)
    record = test_resources.entry_facts(entry).copy()
    if record.get("invocation") != nonce:
        raise ValueError("resource membership association changed")
    return entry, record


def release_resource(identity: str, expected: dict, deadline: float) -> str:
    """Unpin only the exact inspected resource under its existing metadata owner."""
    with test_resources.resource_update(identity, deadline=deadline) as record:
        if record != expected:
            raise ValueError("resource changed after exact disposition inspection")
        record["pin"] = False
        record["explicit_release"] = True
    return test_resources.reclaim(identity, deadline=deadline)


def inspect(
    handle: str, *, deadline: float, storage: bool = False
) -> tuple[dict, dict]:
    registry, declaration, state = run_inspection.resolve(handle)
    blockers = []
    for name in ("producer", "composer"):
        record = state.get(name)
        if record is None and name == "composer":
            continue
        reason = owner_reason(record, deadline)
        if reason:
            blockers.append(f"{name}: {reason}")
    members = []
    seen: set[tuple[int, int]] = set()
    for identity, membership in list(state["members"].items())[:1000]:
        try:
            entry, record = associated_resource(identity, declaration["nonce"])
            reason = None
            if entry.phase == "disposed":
                reason = "already disposed"
            elif record.get("creation_borrower"):
                reason = (
                    "uncertain creation effects; recover through exact fixture owner"
                )
            elif record.get("manual_pin"):
                reason = "manual retention pin; release explicitly by exact resource"
            elif record.get("references") or record.get("borrows"):
                reason = "protected evidence/reference or active borrow"
            elif not record.get("drained"):
                reason = (
                    "fixture drain unrecorded; use exact resource drain/recovery owner"
                )
            elif not record.get("units"):
                reason = (
                    "unknown physical fixture owner; authenticate exact descendants"
                )
            else:
                for unit in record["units"]:
                    observation = operation_lifetime.drained({"scope": unit}, deadline=deadline)
                    if not observation.complete:
                        reason = (
                            "fixture scope not drained: " + observation.diagnostic()
                        )
                        break
            members.append(
                {
                    "resource": identity,
                    "membership": membership,
                    "kind": record["kind"],
                    "state": record["state"],
                    "role": record.get("disposition"),
                    "bytes_held": storage_observation(
                        record, deadline=deadline, seen=seen
                    )
                    if storage
                    else {
                        "status": "not_requested",
                        "route": "repeat exact inspection with --storage",
                    },
                    "reason": reason,
                    "route": "exact resource release/reclaim"
                    if reason is None
                    else "exact owner recovery",
                    "record": record,
                }
            )
        except (OSError, ValueError, RuntimeError) as error:
            members.append(
                {
                    "resource": identity,
                    "membership": membership,
                    "reason": str(error),
                    "route": "exact owner recovery",
                }
            )
    if len(state["members"]) > 1000:
        blockers.append(
            "membership exceeds bounded action page; exact owner recovery required"
        )
    result = {
        "invocation": declaration["nonce"],
        "run_path": str(registry / "invocations" / (declaration["nonce"] + ".json")),
        "registry": str(registry),
        "phase": state["phase"],
        "terminal": "available" if state.get("terminal") else "missing",
        "abandonment": state.get("abandonment"),
        "blockers": blockers,
        "resources": members,
        "capacity_release": "not requested",
        "action": "inspection",
    }
    return state, result


def apply(handle: str, *, deadline: float, storage: bool = False) -> dict:
    state, result = inspect(handle, deadline=deadline, storage=storage)
    if result["blockers"]:
        result["action"] = "blocked"
        return result
    store = test_resources.storage()
    disposition = store.abandon(result["invocation"], state, deadline=deadline)
    result.update(action="membership fenced", abandonment=disposition)
    # Re-observe every exact resource after durable closure. Its owner compares
    # generation, references and physical scopes again during reclaim.
    _fresh, current = inspect(handle, deadline=deadline, storage=storage)
    for resource in current["resources"]:
        identity = resource["resource"]
        status = resource.get("reason")
        if status is None:
            try:
                status = release_resource(identity, resource["record"], deadline)
            except (OSError, ValueError, RuntimeError) as error:
                status = f"retained: {error}"
        with store.invocation_edit(result["invocation"], deadline=deadline) as image:
            image["abandonment"]["resources"][identity] = status
        resource["disposition_result"] = status
    result["resources"] = current["resources"]
    # Only an authenticated captured allocation can request owner reconciliation.
    producer = state.get("producer", {})
    selected = producer.get("allocation")
    if isinstance(selected, str):
        path = Path(selected)
        try:
            snapshot = host.readonly_snapshot(path.parent, deadline=deadline)
            record = snapshot["owners"].get(path.name)
            if record is None:
                result["capacity_release"] = "already absent"
            elif not isinstance(producer.get("allocation_owner"), dict) or any(
                record.get(key) != producer["allocation_owner"].get(key)
                for key in ("pid", "start", "boot")
            ):
                result["capacity_release"] = "blocked: allocation owner changed"
            elif not host.drained(record, allocation=path, deadline=deadline):
                result["capacity_release"] = (
                    "blocked: owner drain or terminal guard unresolved; recover existing owner"
                )
            else:
                profile = host.Profile(
                    record["class"],
                    record["memory"],
                    record["lane"],
                    record["slots"],
                    tuple(record["cores"]),
                    record["exclusive"],
                )
                allocation = host.Allocation(
                    path.parent, path.name, profile, record["deadline"]
                )
                result["capacity_release"] = (
                    "released" if allocation.release(deadline=deadline) else "retained"
                )
        except (OSError, ValueError, RuntimeError) as error:
            result["capacity_release"] = f"unavailable: {error}"
    else:
        result["capacity_release"] = "unknown allocation; no release attempted"
    with store.invocation_edit(result["invocation"], deadline=deadline) as image:
        image["abandonment"].update(
            phase="observed", capacity_release=result["capacity_release"]
        )
    return result


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("invocation")
    parser.add_argument(
        "--apply",
        action="store_true",
        help="explicitly fence and dispose eligible exact members",
    )
    parser.add_argument(
        "--storage",
        action="store_true",
        help="bounded exact owned-path byte observation; shared stores stay unattributed",
    )
    parser.add_argument("--seconds", type=float, default=30)
    args = parser.parse_args(argv)
    if not 0 < args.seconds <= 90:
        parser.error("--seconds must be in (0, 90]")
    deadline = time.monotonic() + args.seconds
    try:
        result = (
            apply(args.invocation, deadline=deadline, storage=args.storage)
            if args.apply
            else inspect(args.invocation, deadline=deadline, storage=args.storage)[1]
        )
    except (OSError, ValueError, RuntimeError) as error:
        print(json.dumps({"error": str(error), "invocation": args.invocation}))
        return 1
    print(json.dumps(result, indent=2))
    return int(args.apply and result["action"] == "blocked")


if __name__ == "__main__":
    raise SystemExit(main())
