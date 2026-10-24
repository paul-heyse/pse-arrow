# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Bounded projections of authentic product invocation authority and references."""

from __future__ import annotations

import contextlib
import hashlib
import os
import stat
from pathlib import Path

from scripts import resource_store, test_resources


def resolve(
    handle: str, registry: Path | None = None
) -> tuple[Path, test_resources.InvocationRecord, dict]:
    """An exact ID uses the selected registry; a declaration must belong to it."""
    root = registry if registry is not None else test_resources.registry()
    if (
        not root.is_absolute()
        or ".." in root.parts
        or any(path.is_symlink() for path in (root, *root.parents))
    ):
        raise ValueError("registry must be an absolute directory without symlinks")
    path = (
        root / "invocations" / f"{handle}.json"
        if test_resources.IDENTIFIER.fullmatch(handle)
        else Path(handle).absolute()
    )
    if path.parent != root / "invocations" or any(
        item.is_symlink() for item in (path, *path.parents)
    ):
        raise ValueError(
            "invocation declaration does not belong to the selected registry"
        )
    declaration = test_resources.read_invocation(path)
    if path.stem != declaration["nonce"]:
        raise ValueError("invocation filename differs from its exact identity")
    state = resource_store.FileStore(root).invocation_snapshot(declaration["nonce"])
    if state["declaration"] != declaration:
        raise ValueError("invocation declaration differs from registry authority")
    return root, declaration, state


def subject_identity(kind: str, selected: dict) -> str:
    return (
        selected["binary_id"] + "::" + selected["name"]
        if kind == "rust"
        else selected["name"]
    )


def owner_projection(owner: object) -> dict:
    """Project captured associations without checking or implying current liveness."""
    result: dict[str, object] = {
        "association": "captured-metadata"
        if isinstance(owner, dict)
        else "unavailable",
        "liveness": "unobserved",
    }
    if not isinstance(owner, dict):
        return result
    for name in ("pid", "start", "boot", "allocation", "operation"):
        value = owner.get(name)
        if (name == "pid" and type(value) is int and value > 0) or (
            name != "pid" and isinstance(value, str) and len(value) <= 4096
        ):
            result[name] = value
    for name, fields in (
        ("scope", ("unit", "group", "invocation")),
        ("allocation_owner", ("pid", "start", "boot")),
    ):
        value = owner.get(name)
        if isinstance(value, dict):
            selected = {}
            for key in fields:
                item = value.get(key)
                if (key == "pid" and type(item) is int and item > 0) or (
                    key != "pid" and isinstance(item, str) and len(item) <= 4096
                ):
                    selected[key] = item
            result[name] = selected
    return result


def artifact_observation(
    path: Path, digest: str, tail: int, record: test_resources.ResourceRecord | None
) -> dict:
    """Read the exact referenced descriptor through the resource owner's file reader."""
    result = {
        "path": str(path),
        "status": "available",
        "recorded_digest": digest,
        "tail": [],
    }
    try:
        with contextlib.ExitStack() as contexts:
            if record is not None:
                directory = contexts.enter_context(
                    test_resources.report_directory(record)
                )
            else:
                flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC
                directory = os.open(path.anchor, flags)
                contexts.callback(os.close, directory)
                for part in path.parent.parts[1:]:
                    directory = os.open(part, flags, dir_fd=directory)
                    contexts.callback(os.close, directory)
            _parent, _name, stream = contexts.enter_context(
                test_resources.artifact_file(directory, path.name)
            )
            before = os.fstat(stream.fileno())
            if not stat.S_ISREG(before.st_mode):
                result.update(
                    status="unavailable", error="artifact is not an ordinary file"
                )
                return result
            observed = hashlib.file_digest(stream, "sha256").hexdigest()
            result["observed_digest"] = observed
            if observed != digest:
                result["status"] = "changed"
                return result
            stream.seek(max(0, before.st_size - 65536))
            result["tail"] = (
                stream.read().decode(errors="replace").splitlines()[-tail:]
                if tail
                else []
            )
            after = os.fstat(stream.fileno())
            if (before.st_ino, before.st_size, before.st_mtime_ns) != (
                after.st_ino,
                after.st_size,
                after.st_mtime_ns,
            ):
                result.update(status="changed", tail=[])
    except (OSError, ValueError, RuntimeError) as error:
        result.update(
            status="missing" if isinstance(error, FileNotFoundError) else "unavailable",
            error=str(error),
        )
    return result


def invocation_resource(
    store: resource_store.FileStore, identity: str, nonce: str
) -> tuple[resource_store.Entry, test_resources.ResourceRecord]:
    entry = store.resource_snapshot(identity)
    record = test_resources.entry_facts(entry)
    if record.get("invocation") != nonce:
        raise ValueError("resource association differs from invocation")
    return entry, record


def evidence_owner(store: resource_store.FileStore, directory: Path) -> str:
    resource = store.identity_lookup([str(directory), "", "evidence"])
    if resource is None:
        raise ValueError("artifact has no current registered evidence owner")
    return resource


def product_result(
    handle: str,
    *,
    registry: Path | None = None,
    subject: str | None = None,
    failures: bool = False,
    offset: int = 0,
    limit: int = 100,
    tail: int = 0,
) -> dict:
    root, declaration, state = resolve(handle, registry)
    frozen = state.get("terminal") if state["phase"] == "frozen" else None
    evidence = frozen.get("evidence", {}) if frozen else {}
    outcomes = frozen.get("outcomes", {}) if frozen else {}
    if not isinstance(evidence, dict) or not isinstance(outcomes, dict):
        raise TypeError("invalid frozen evidence or outcomes")
    selected = declaration["selected"]
    catalog = [subject_identity(declaration["kind"], item) for item in selected]
    if len(catalog) != len(set(catalog)) or set(outcomes) - set(catalog):
        raise ValueError("frozen results differ from selected catalog")
    if subject is not None and subject not in catalog:
        raise ValueError("subject is not in the selected catalog")
    subjects = [
        {
            "identity": identity,
            "selection": item,
            "outcome": outcomes.get(identity, "unreported"),
            "requirements": evidence.get(
                "requirements", state.get("requirements", {})
            ).get(identity),
        }
        for identity, item in zip(catalog, selected, strict=True)
        if (subject is None or subject == identity)
        and (not failures or outcomes.get(identity) != "pass")
    ]
    # Metadata references are intentionally not hydrated by the ordinary view.
    artifacts = evidence.get("artifacts", {})
    if not isinstance(artifacts, dict):
        raise TypeError("invalid frozen artifact references")
    resources = []
    store = resource_store.FileStore(root)
    for identity, member in list(state["members"].items())[offset : offset + limit]:
        try:
            entry, record = invocation_resource(store, identity, declaration["nonce"])
            resources.append(
                {
                    "resource": identity,
                    "status": "available",
                    "phase": entry.phase,
                    "record": record,
                    "retention_reason": test_resources.cleanup_reason(
                        record, check_units=False
                    ),
                }
            )
        except (OSError, ValueError, TypeError) as error:
            resources.append(
                {
                    "resource": identity,
                    "status": "unavailable",
                    "member": member,
                    "error": str(error),
                }
            )
    payloads = []
    if tail:
        # Only explicit artifact references from the authentic terminal may be read.
        # Protect report directories through their current owner for the full read.
        with contextlib.ExitStack() as borrows:
            borrowed: dict[
                Path, tuple[test_resources.ResourceRecord | None, str | None]
            ] = {}
            for name, reference in artifacts.items():
                if not isinstance(name, str) or not isinstance(reference, str):
                    continue  # binary/extension association is metadata, not a log
                path = Path(name)
                if not path.is_absolute():
                    base = evidence.get("artifact_base")
                    if not isinstance(base, str) or not Path(base).is_absolute():
                        continue  # no authenticated base is recorded for this reference
                    path = Path(base) / path
                    if not path.is_relative_to(Path(base)) or ".." in path.parts:
                        raise ValueError("artifact reference escapes its recorded base")
                if ".." in path.parts or any(
                    item.is_symlink() for item in (path.parent, *path.parent.parents)
                ):
                    payloads.append(
                        {
                            "path": str(path),
                            "status": "unavailable",
                            "recorded_digest": reference,
                            "tail": [],
                            "error": "artifact reference traverses a changed path",
                        }
                    )
                    continue
                directory = path.parent
                if directory not in borrowed:
                    # Explicit registry selection applies to protections as well.
                    try:
                        resource = evidence_owner(store, directory)
                        borrows.enter_context(
                            test_resources.borrow(resource, store=store)
                        )
                        borrowed[directory] = (
                            test_resources.entry_facts(
                                store.resource_snapshot(resource)
                            ),
                            None,
                        )
                    except (OSError, ValueError) as error:
                        borrowed[directory] = (None, str(error))
                record, error = borrowed[directory]
                if record is None:
                    payloads.append(
                        {
                            "path": str(path),
                            "status": "unavailable",
                            "recorded_digest": reference,
                            "tail": [],
                            "error": f"protected payload read unavailable: {error}",
                        }
                    )
                else:
                    payloads.append(
                        artifact_observation(
                            directory / path.name, reference, tail, record
                        )
                    )
    unreported = [name for name in catalog if name not in outcomes]
    return {
        "kind": "product-test",
        "registry": str(root),
        "invocation": declaration["nonce"],
        "run_kind": declaration["kind"],
        "run_path": str(root / "invocations" / (declaration["nonce"] + ".json")),
        "observation_mode": "protected-payload" if tail else "detached-metadata",
        "phase": state["phase"],
        "terminal_status": "available" if frozen else "missing",
        "terminal_identity": state.get("terminal_identity"),
        "terminal": {
            key: value
            for key, value in frozen.items()
            if key not in {"outcomes", "evidence"}
        }
        if frozen
        else None,
        "lifecycle": state.get("lifecycle"),
        "producer": owner_projection(state.get("producer")),
        "composer": owner_projection(state.get("composer")),
        "diagnostic_actions": state.get("diagnostic_actions", [])[
            offset : offset + limit
        ],
        "selected_count": len(selected),
        "selection_state": evidence.get(
            "selection_state",
            "collected"
            if selected or declaration.get("collection_owner")
            else "uncollected",
        ),
        "refusal": evidence.get("refusal"),
        "subjects": subjects[offset : offset + limit],
        "selected_unreported": unreported[offset : offset + limit],
        "selected_unreported_count": len(unreported),
        "report_rows": evidence.get("results", [])[offset : offset + limit],
        "report_errors": evidence.get("report_errors", []),
        "workload_exit": evidence.get("workload_exit"),
        "assurance": evidence.get("assurance", "unknown"),
        "peer_invocations": {
            key: value
            for key, value in evidence.get("peer_invocations", {}).items()
            if key in {item["identity"] for item in subjects[offset : offset + limit]}
        },
        "cleanup_observations": {
            key: value
            for key, value in evidence.get("cleanup_observations", {}).items()
            if key in {item["identity"] for item in subjects[offset : offset + limit]}
        },
        "setup_failures": {
            key: value
            for key, value in evidence.get("setup_failures", {}).items()
            if key in {item["identity"] for item in subjects[offset : offset + limit]}
        },
        "process": evidence.get("process", {}),
        "owned_drain": evidence.get("owned_drain", "unobserved"),
        "artifacts": artifacts,
        "payloads": payloads,
        "resources": resources,
        "page": {
            "offset": offset,
            "limit": limit,
            "subject_count": len(subjects),
            "resource_count": len(state["members"]),
        },
    }
