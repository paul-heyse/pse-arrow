# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Exclusive beta rollout execution; operation owners separately settle and activate.

This is a maintenance foundation, not persistent rollout qualification. CLI exit,
TTL renewal and catalog parity never prove a submitted SQL step settled. The
supervisor guards surrealkit_rollout_pending until settle receives actual domain
operation evidence and drains the exact maintenance listener.
"""

from __future__ import annotations

import argparse
import contextlib
import hashlib
import json
import math
import os
import re
import signal
import subprocess
import sys
import threading
import time
import tomllib
import uuid
from collections.abc import Callable, Mapping
from pathlib import Path
from types import MappingProxyType
from typing import cast

from scripts import (
    native_operation,
    surreal_rollout_producer,
    surreal_server,
    surrealkit,
)

PENDING = "surrealkit_rollout_pending"
VERSION = "surrealkit-maintenance-v1"


class MaintenanceError(RuntimeError):
    """Exact maintenance authority or operation settlement is unavailable."""


def command(mode: str, rollout_id: str | None) -> list[str]:
    if mode == "up" and rollout_id is None:
        return ["rollout", "up", "--complete"]
    if mode == "status" and rollout_id is None:
        return ["rollout", "status"]
    if (
        mode in {"start", "complete", "repair"}
        and rollout_id
        and re.fullmatch(r"[A-Za-z0-9_-]+", rollout_id)
    ):
        return ["rollout", mode, rollout_id]
    raise MaintenanceError("Unsupported forward rollout command or identity")


def inputs(folder: Path) -> tuple[dict[str, bytes], str]:
    """Bind all selected frozen inputs; reject ambient target/config injection."""
    if not folder.is_absolute() or folder != folder.resolve(strict=True):
        raise MaintenanceError("Input requires an exact nonsymlinked directory")
    files: dict[str, bytes] = {}
    for path in sorted(folder.rglob("*")):
        if path.is_symlink() or (not path.is_dir() and not path.is_file()):
            raise MaintenanceError("Symlink or special rollout input refused")
        if path.name in {".env", ".env.local", "surrealkit.toml"}:
            raise MaintenanceError("Ambient target configuration refused")
        if path.is_file():
            files[path.relative_to(folder).as_posix()] = path.read_bytes()
    manifests = {
        name: data
        for name, data in files.items()
        if Path(name).parent == Path("rollouts") and name.endswith(".toml")
    }
    if not manifests:
        raise MaintenanceError("Frozen rollout chain is absent")
    for name, data in manifests.items():
        manifest = tomllib.loads(data.decode())
        identity = manifest.get("id")
        if identity != Path(name).stem or not isinstance(identity, str):
            raise MaintenanceError("Rollout manifest identity differs from its file")
        steps = manifest.get("steps", [])
        if not isinstance(steps, list):
            raise MaintenanceError("Malformed rollout steps")
        for step in steps:
            if not isinstance(step, dict):
                raise MaintenanceError("Malformed rollout step")
            references = step.get("files", [])
            if not isinstance(references, list):
                raise MaintenanceError("Malformed frozen file references")
            for frozen in references:
                if not isinstance(frozen, dict) or not isinstance(
                    frozen.get("path"), str
                ):
                    raise MaintenanceError("Frozen beta file identity is absent")
                relative = Path(frozen["path"])
                if relative.is_absolute() or ".." in relative.parts:
                    raise MaintenanceError("External rollout SQL refused")
                frozen_name = (Path("rollouts") / identity / relative).as_posix()
                payload = files.get(frozen_name)
                if payload is None or hashlib.sha256(payload).hexdigest() != frozen.get(
                    "hash"
                ):
                    raise MaintenanceError("Frozen SQL missing or falsified")
    digest = hashlib.sha256()
    for name, data in files.items():
        encoded = name.encode()
        digest.update(len(encoded).to_bytes(8, "big"))
        digest.update(encoded)
        digest.update(len(data).to_bytes(8, "big"))
        digest.update(data)
    return files, digest.hexdigest()


def environment(
    config: Mapping[str, object],
    credentials: Mapping[str, object],
    folder: Path,
    endpoint: str,
) -> dict[str, str]:
    result = {
        key: value
        for key, value in os.environ.items()
        if not key.startswith(("SURREAL", "DATABASE_"))
    }
    result.update(
        SURREALDB_HOST=endpoint,
        SURREALDB_NAMESPACE=str(config["namespace"]),
        SURREALDB_NAME=str(config["database"]),
        SURREALDB_USER=str(credentials["username"]),
        SURREALDB_PASSWORD=str(credentials["password"]),
        SURREALDB_AUTH_LEVEL="root",
        SURREALDB_FOLDER=str(folder),
        SURREALDB_CONNECT_TIMEOUT_SECS="15",
        SURREALDB_QUERY_TIMEOUT_SECS="30",
    )
    return result


def validated_rollout_order(output: str, inventory: list[str]) -> list[str]:
    """Consume pinned beta lint's chain order, retaining every frozen manifest."""
    matched = re.fullmatch(
        r"([1-9][0-9]*) rollout\(s\) are valid, in order: "
        r"([A-Za-z0-9_-]+(?:, [A-Za-z0-9_-]+)*)",
        output.strip(),
    )
    if matched is None:
        raise MaintenanceError("Validated beta rollout order is unavailable")
    ordered = matched[2].split(", ")
    if (
        len(ordered) != int(matched[1])
        or len(set(ordered)) != len(ordered)
        or sorted(ordered) != sorted(inventory)
    ):
        raise MaintenanceError(
            "Validated beta rollout order differs from frozen inventory"
        )
    return ordered


def selected(state: Path) -> dict[str, object]:
    config = surreal_server.config_for(state)
    if (
        surreal_server.service_directory(state) != state
        or surreal_server.managed_contexts(state) != [state]
        or config.get("accepting_writes") is not False
        or config.get("admission") not in {"quiesced", "validation_required"}
    ):
        raise MaintenanceError("Rollout requires its exclusive closed service owner")
    surreal_server.all_contexts_drained(state)
    if not (state / ".maintenance-listener.json").exists() and surreal_server.active(
        state
    ):
        raise MaintenanceError(
            "Stop and drain the selected service explicitly before rollout"
        )
    return config


def pending(state: Path, nonce: str) -> dict[str, object]:
    config = selected(state)
    value = config.get(PENDING)
    if (
        not isinstance(value, dict)
        or value.get("kind") != VERSION
        or value.get("nonce") != nonce
    ):
        raise MaintenanceError("Exact pending rollout identity required")
    if any(
        value.get(key) != config.get(key)
        for key in ("instance_id", "namespace", "database")
    ):
        raise MaintenanceError("Pending rollout service selection changed")
    return value


def update(state: Path, value: dict[str, object]) -> None:
    with surreal_server.state_lock(state, cleanup=True):
        config = surreal_server.config_for(state)
        current = config.get(PENDING)
        if current is not None and (
            not isinstance(current, dict) or current.get("nonce") != value["nonce"]
        ):
            raise MaintenanceError("Competing pending rollout retained")
        config[PENDING] = value
        config["accepting_writes"] = False
        surreal_server.write_json(state / "config.json", config)


def exact_owner(state: Path, owner: Mapping[str, object]) -> None:
    if surreal_server.read_json(state / "lifecycle-owner.json") != dict(
        owner
    ) or not surreal_server.reservation_live(dict(owner)):
        raise MaintenanceError("Maintenance owner changed; admission remains closed")


def require_child_drains(state: Path, value: dict[str, object]) -> None:
    """No maintenance restart may reconnect a predecessor's undrained writer."""
    # Attempts are emitted only by execute/canonical_tool for this exact pending owner.
    attempts = cast("list[dict[str, object]]", value["attempts"])
    canonical_attempts = cast(
        "list[dict[str, object]]", value.get("canonical_attempts", [])
    )
    for attempt in [*attempts, *canonical_attempts]:
        if not attempt.get("child_scope_requested"):
            continue
        if (
            cast("Mapping[str, object]", attempt.get("child", {})).get("owned_drain")
            == "complete"
        ):
            continue
        directory = Path(cast("str", attempt["directory"]))
        expected = state / ".surrealkit" / str(value["nonce"])
        if directory != directory.resolve(strict=True) or directory.parent != expected:
            raise MaintenanceError(
                "Unresolved child scope requires exact drain recovery"
            )
        result = surreal_server.read_json(directory / "child.json")
        native_record = result.get("native_record")
        if not isinstance(native_record, dict):
            raise MaintenanceError(
                "Unresolved child scope requires exact drain recovery"
            )
        drain = native_operation.drained(native_record)
        if not drain.complete:
            raise MaintenanceError(
                drain.diagnostic() + "; maintenance listener remains closed"
            )
        cast("dict[str, object]", attempt.setdefault("child", {}))["owned_drain"] = (
            "complete"
        )
        cast("dict[str, object]", attempt["child"])["scope"] = dict(drain.owner or {})
        update(state, value)


def predecessor_marker(state: Path, config: dict[str, object]) -> object:
    """Capture the immutable predecessor before any selected rollout SQL."""
    credentials = surreal_server.read_json(state / "credentials.json")
    catalog = surreal_server.maintenance_catalog(state, config, credentials, "DB")
    if "canonical_interpretations" not in surreal_server.object_mapping(
        catalog["tables"]
    ):
        return None
    result = surreal_server.maintenance_query(
        state,
        config,
        credentials,
        "LET $value=(SELECT interpretation,schema_digest,issuing_store FROM ONLY canonical_interpretations:current);",
    )
    marker = result.get("value")
    if marker is not None and (
        not isinstance(marker, dict)
        or set(marker) != {"interpretation", "schema_digest", "issuing_store"}
        or any(not isinstance(value, str) or not value for value in marker.values())
    ):
        raise MaintenanceError("Canonical predecessor marker is malformed")
    return marker


def require_empty_cutover_target(
    state: Path,
    config: dict[str, object],
    folder: Path,
    selected_rollouts: list[str],
    previous_marker: object,
) -> None:
    """Explicit design-phase schema replacement cannot operate on retained data.

    An empty uninitialized target can carry only beta's exact bookkeeping rows.
    Lifecycle exclusion and the authenticated maintenance listener already hold.
    """
    if not (folder / surreal_rollout_producer.REGISTER).exists():
        if any((folder / "rollouts").glob("*/producer.json")):
            raise MaintenanceError("Producer rollout requires immutable registration")
        return
    publication = surreal_rollout_producer.load(folder)
    releases = cast("list[dict[str, object]]", publication["rollouts"])
    if not any(
        release["id"] in selected_rollouts and "cutover" in release
        for release in releases
    ):
        return
    if previous_marker is not None or predecessor_marker(state, config) is not None:
        raise MaintenanceError(
            "Design-phase v6 cutover requires a fresh empty uninitialized target; retained-data upgrade refused"
        )
    credentials = surreal_server.read_json(state / "credentials.json")
    catalog = surreal_server.maintenance_catalog(state, config, credentials, "DB")
    for table in sorted(surreal_server.object_mapping(catalog["tables"])):
        if table in {"__entity", "__rollout"}:
            continue
        result = surreal_server.maintenance_query(
            state,
            config,
            credentials,
            "LET $rows=(SELECT id FROM "
            + surreal_server.sql_identifier(table)
            + " LIMIT 1);",
        )
        if result.get("rows") != []:
            raise MaintenanceError(
                "Design-phase v6 cutover target has retained or unknown application rows; regenerate a fresh disposable target"
            )


def execute(
    binary: Path,
    state: Path,
    folder: Path,
    mode: str,
    *,
    rollout_id: str | None = None,
    pending_nonce: str | None = None,
    timeout: float = 120,
) -> dict[str, object]:
    """Execute one fixed command, preserving SQL uncertainty until explicit settlement."""
    if not math.isfinite(timeout) or timeout <= 0:
        raise MaintenanceError("Tool execution needs a finite positive clock")
    args = command(mode, rollout_id)
    provenance = surrealkit.verify(binary)
    files, input_digest = inputs(folder)
    rollout_inventory = sorted(
        Path(name).stem
        for name in files
        if Path(name).parent == Path("rollouts") and name.endswith(".toml")
    )
    with surreal_server.lifecycle_reservation(state):
        config = selected(state)
        owner = surreal_server.read_json(state / "lifecycle-owner.json")
        continuing = mode == "complete" and pending_nonce is not None
        if mode in {"status", "repair"} or continuing:
            if pending_nonce is None:
                raise MaintenanceError("Recovery requires the exact pending nonce")
            value = pending(state, pending_nonce)
            if value["tool"] != provenance:
                raise MaintenanceError(
                    "Recovery tool provenance differs from submitted rollout"
                )
            require_child_drains(state, value)
            if value["input_sha256"] != input_digest:
                raise MaintenanceError(
                    "Recovery frozen input differs from submitted rollout"
                )
            if mode == "repair":
                if rollout_id not in cast("list[str]", value["rollouts"]):
                    raise MaintenanceError("Repair does not name a submitted rollout")
                if value.get("sql_outcome") != "settled":
                    raise MaintenanceError(
                        "Repair requires exact SQL settlement before metadata repair"
                    )
            if continuing:
                if (
                    value.get("phase") != "start_settled"
                    or value.get("command") != ["rollout", "start", rollout_id]
                    or value.get("rollouts") != [rollout_id]
                    or cast("Mapping[str, object]", value.get("settlement", {})).get(
                        "phase"
                    )
                    != "start"
                ):
                    raise MaintenanceError(
                        "Completion requires the exact settled start phase"
                    )
                cast(
                    "list[dict[str, object]]", value.setdefault("phase_history", [])
                ).append(
                    {
                        "command": value["command"],
                        "settlement": value.pop("settlement"),
                    }
                )
                value.update(
                    command=args, phase="prepared", sql_outcome="not_submitted"
                )
        else:
            if config.get(PENDING) is not None or pending_nonce is not None:
                raise MaintenanceError(
                    "Pending SQL requires exact settlement; blind replay refused"
                )
            nonce = uuid.uuid4().hex
            value: dict[str, object] = {
                "kind": VERSION,
                "nonce": nonce,
                "instance_id": config["instance_id"],
                "namespace": config["namespace"],
                "database": config["database"],
                "input_sha256": input_digest,
                "command": args,
                "phase": "prepared",
                "sql_outcome": "not_submitted",
                "rollouts": rollout_inventory,
                "tool": provenance,
                "owner": owner,
                "attempts": [],
                "phase_history": [],
            }
            if rollout_id is not None and rollout_id not in cast(
                "list[str]", value["rollouts"]
            ):
                raise MaintenanceError("Selected rollout is absent from frozen chain")
            if rollout_id is not None:
                value["rollouts"] = [rollout_id]
            update(state, value)
        owner = surreal_server.read_json(state / "lifecycle-owner.json")
        nonce = str(value["nonce"])
        directory = surreal_server.checked_directory(state / ".surrealkit" / nonce)
        attempt = directory / uuid.uuid4().hex
        attempt.mkdir(mode=0o700)
        snapshot = attempt / "input"
        snapshot.mkdir(mode=0o700)
        for name, data in files.items():
            path = snapshot / name
            path.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
            with path.open("xb") as output:
                output.write(data)
            path.chmod(0o400)
        credentials = surreal_server.read_json(state / "credentials.json")
        env = environment(config, credentials, snapshot, "http://127.0.0.1:1")
        # Beta owns chain, fork/order, and frozen-manifest validation. Lint does
        # not connect, and runs before any maintenance listener or SQL submission.
        lint = subprocess.run(
            [str(binary), "rollout", "lint"],
            env=env,
            cwd=snapshot,
            capture_output=True,
            text=True,
            timeout=15,
            check=False,
        )
        (attempt / "lint.stdout").write_text(lint.stdout)
        (attempt / "lint.stderr").write_text(lint.stderr)
        if lint.returncode:
            raise MaintenanceError(
                "Frozen chain lint failed; pending selection retained"
            )
        ordered = validated_rollout_order(lint.stdout, rollout_inventory)
        if mode == "up":
            # Filenames carry timestamps plus digest suffixes; lexical order
            # can differ from beta's validated source-to-target chain order.
            value["rollouts"] = ordered
            update(state, value)
        record: dict[str, object] = {
            "command": args,
            "directory": str(attempt),
            "phase": "prepared",
            "tool": provenance,
        }
        cast("list[dict[str, object]]", value["attempts"]).append(record)
        update(state, value)
        observation: dict[str, object] = {}
        try:
            if mode in {"status", "repair"}:
                surreal_server.drain_maintenance_listener(state)
            endpoint = surreal_server.maintenance_endpoint(state)
            surreal_server.require_maintenance_owner(
                state, surreal_server.config_for(state)
            )
            if "previous_marker" not in value and mode not in {"status", "repair"}:
                if continuing:
                    raise MaintenanceError(  # noqa: TRY301 -- exact pending/CLI refusal boundary
                        "Original predecessor marker unavailable; activation refused"
                    )
                value["previous_marker"] = predecessor_marker(state, config)
            if mode not in {"status", "repair"}:
                require_empty_cutover_target(
                    state,
                    config,
                    snapshot,
                    cast("list[str]", value["rollouts"]),
                    value["previous_marker"],
                )
            value["initializer"] = owner
            update(state, value)
            record["phase"] = "submitted"
            # Recovery inspection/repair cannot erase the original uncertain SQL.
            if mode not in {"status", "repair"}:
                value.update(phase="submitted", sql_outcome="unknown")
            update(state, value)
            env = environment(config, credentials, snapshot, endpoint)
            surreal_server.write_json(
                attempt / "authority.json",
                {
                    "owner": owner,
                    "listener": surreal_server.read_json(
                        state / ".maintenance-listener.json"
                    ),
                    "endpoint": endpoint,
                },
            )
            record["child_scope_requested"] = True
            update(state, value)
            status = native_operation.run_child_scope(
                [
                    sys.executable,
                    "-m",
                    "scripts.surrealkit_maintenance",
                    "_child",
                    "--state",
                    str(state),
                    "--binary",
                    str(binary),
                    "--folder",
                    str(snapshot),
                    "--attempt",
                    str(attempt),
                    "--mode",
                    mode,
                    "--timeout",
                    str(timeout),
                    *(["--rollout-id", rollout_id] if rollout_id else []),
                ],
                env,
                cwd=surrealkit.ROOT,
                observation=observation,
            )
            record.update(exit_code=status, child=observation)
            child_result = surreal_server.read_json(attempt / "child.json")
            surreal_server.require_maintenance_owner(
                state, surreal_server.config_for(state)
            )
            record["result"] = child_result
            if (
                status == 0
                and child_result.get("reason") == "exited"
                and observation.get("owned_drain") == "complete"
            ):
                record["phase"] = "cli_succeeded"
                if mode not in {"status", "repair"}:
                    value["phase"] = "cli_succeeded"
            else:
                record["phase"] = "unknown"
                if mode not in {"status", "repair"}:
                    value["phase"] = "unknown"
        except BaseException as error:
            record.update(
                phase="unknown",
                error=f"{type(error).__name__}: {error}",
                child=observation,
            )
            if mode not in {"status", "repair"}:
                value["phase"] = "unknown"
            raise
        finally:
            update(state, value)
            try:
                surreal_server.drain_maintenance_listener(state)
                record["listener_drain"] = "complete"
                exact_owner(state, owner)
            except BaseException as error:
                record.update(
                    listener_drain="unresolved",
                    drain_error=f"{type(error).__name__}: {error}",
                )
                update(state, value)
                raise
            update(state, value)
        return value


def immutable(value: object) -> object:
    if isinstance(value, dict):
        return MappingProxyType({key: immutable(item) for key, item in value.items()})
    if isinstance(value, list):
        return tuple(immutable(item) for item in value)
    return value


def settle(
    state: Path,
    nonce: str,
    operation_settler: Callable[
        [Path, str, Mapping[str, object]], Mapping[str, object]
    ],
) -> dict[str, object]:
    """Join actual operation settlement under authority, then drain; never activate.

    The integration supplies its domain owner, which must inspect actual SQL
    effects and semantic obligations, including uncertain data steps. Its proof
    identifies the selected nonce/input, operation owner/identity and phase/steps.
    A command exit, beta repair result or arbitrary boolean is not that evidence.
    """
    with surreal_server.lifecycle_reservation(state):
        value = pending(state, nonce)
        require_child_drains(state, value)
        owner = surreal_server.read_json(state / "lifecycle-owner.json")
        value["initializer"] = owner
        update(state, value)
        surreal_server.drain_maintenance_listener(state)
        endpoint = surreal_server.maintenance_endpoint(state)
        try:
            surreal_server.require_maintenance_owner(
                state, surreal_server.config_for(state)
            )
            evidence = operation_settler(
                state, endpoint, cast("Mapping[str, object]", immutable(value))
            )
            if (
                not isinstance(evidence, Mapping)
                or evidence.get("outcome") != "settled"
                or evidence.get("nonce") != nonce
                or evidence.get("input_sha256") != value["input_sha256"]
                or not isinstance(evidence.get("operation"), Mapping)
                or not all(
                    isinstance(
                        cast("Mapping[str, object]", evidence["operation"]).get(key),
                        str,
                    )
                    and cast("Mapping[str, object]", evidence["operation"])[key]
                    for key in ("owner", "identity")
                )
                or not isinstance(evidence.get("phase"), str)
                or evidence["phase"]
                != (
                    "start"
                    if cast("list[str]", value["command"])[1] == "start"
                    else "complete"
                )
                or not isinstance(evidence.get("steps"), list)
                or any(
                    not isinstance(step, Mapping)
                    or not isinstance(step.get("id"), str)
                    or not step["id"]
                    or step.get("outcome")
                    not in {"acknowledged", "reconciled", "not_submitted"}
                    for step in cast("list[object]", evidence["steps"])
                )
            ):
                raise MaintenanceError(
                    "Actual structured operation settlement evidence required"
                )
            surreal_server.require_maintenance_owner(
                state, surreal_server.config_for(state)
            )
            current = pending(state, nonce)
            if (
                current["command"] != value["command"]
                or current["input_sha256"] != value["input_sha256"]
            ):
                raise MaintenanceError("Pending phase changed during domain settlement")
            value["canonical_attempts"] = current.get("canonical_attempts", [])
            value["settlement"] = dict(evidence)
            update(state, value)
        finally:
            surreal_server.drain_maintenance_listener(state)
        exact_owner(state, owner)
        with surreal_server.state_lock(state, cleanup=True):
            config = surreal_server.config_for(state)
            current = config.get(PENDING)
            if not isinstance(current, dict) or current.get("nonce") != nonce:
                raise MaintenanceError("Pending selection changed during settlement")
            value["phase"] = (
                "sql_settled"
                if evidence.get("phase_complete") is False
                else "start_settled"
                if cast("list[str]", value["command"])[1] == "start"
                else "settled"
            )
            value["sql_outcome"] = "settled"
            surreal_server.write_json(
                state / ".surrealkit" / nonce / "settled.json", value
            )
            config[PENDING] = value
            config["accepting_writes"] = False
            surreal_server.write_json(state / "config.json", config)
        return value


def publication(folder: Path) -> dict[str, object]:
    # The publication producer owns interpretation of its immutable files.
    from scripts import surreal_rollout_producer as producer  # noqa: PLC0415

    try:
        return producer.load(folder, current=True)
    except producer.ProducerError as error:
        raise MaintenanceError(str(error)) from error


def generated_settler(
    folder: Path, binary: Path
) -> Callable[[Path, str, Mapping[str, object]], Mapping[str, object]]:
    """Fixed schema-only settlement; data SQL needs a separate domain owner."""

    def operation(
        state: Path, endpoint: str, selected: Mapping[str, object]
    ) -> Mapping[str, object]:
        declarations = publication(folder)
        if inputs(folder)[1] != selected["input_sha256"]:
            raise MaintenanceError(
                "Selected frozen publication changed before settlement"
            )
        # Producer registration validates the entire immutable rollout inventory.
        ordered = cast("list[dict[str, object]]", declarations["rollouts"])
        selected_command = cast("tuple[str, ...]", selected["command"])
        selected_rollouts = cast("tuple[str, ...]", selected["rollouts"])
        if selected_command[1] == "up" and list(selected_rollouts) != [
            item["id"] for item in ordered
        ]:
            raise MaintenanceError(
                "Whole-chain submission differs from full producer publication"
            )
        chosen = [item for item in ordered if item["id"] in selected_rollouts]
        if not chosen or [item["id"] for item in chosen] != list(selected_rollouts):
            raise MaintenanceError(
                "Submitted rollout identities differ from producer publication"
            )
        phase = "start" if selected_command[1] == "start" else "complete"
        config = surreal_server.config_for(state)
        credentials = surreal_server.read_json(state / "credentials.json")
        response = surreal_server.maintenance_query(
            state,
            config,
            credentials,
            "LET $value=(SELECT record::id(id) AS rollout_id,manifest_checksum,source_schema_hash,target_schema_hash,status,steps,last_error FROM __rollout);",
        )
        rows = response.get("value")
        if not isinstance(rows, list) or any(not isinstance(row, dict) for row in rows):
            raise MaintenanceError("Actual beta rollout records are unavailable")
        if any(not isinstance(row.get("rollout_id"), str) for row in rows):
            raise MaintenanceError("Malformed beta rollout identity")
        by_id = {row["rollout_id"]: row for row in rows}
        if not set(by_id).issubset({item["id"] for item in ordered}) or len(
            by_id
        ) != len(rows):
            raise MaintenanceError("Ambiguous beta rollout identities")
        steps = []
        phase_complete = True
        for index, item in enumerate(chosen):
            row = by_id.get(item["id"])
            if not isinstance(row, dict) or any(
                row.get(name) != item[name]
                for name in (
                    "manifest_checksum",
                    "source_schema_hash",
                    "target_schema_hash",
                )
            ):
                raise MaintenanceError(
                    "Actual beta manifest identity differs from frozen producer"
                )
            expected_status = "ready_to_complete" if phase == "start" else "completed"
            if row.get("status") != expected_status:
                if index != len(chosen) - 1 or row.get("status") not in {
                    "running_start",
                    "running_complete",
                    "failed",
                }:
                    raise MaintenanceError(
                        "Beta phase cannot be reconciled against its exact postcondition"
                    )
                phase_complete = False
            recorded = row.get("steps")
            if not isinstance(recorded, list):
                raise MaintenanceError(
                    "Actual beta step acknowledgements are unavailable"
                )
            expected = {
                step["id"]: step
                for step in cast("list[dict[str, object]]", item["steps"])
            }
            if len(expected) != len(cast("list[object]", item["steps"])):
                raise MaintenanceError("Producer step identities are ambiguous")
            observed = {}
            for step in recorded:
                if (
                    not isinstance(step, dict)
                    or step.get("step_id") not in expected
                    or step["step_id"] in observed
                ):
                    raise MaintenanceError("Unexpected or duplicated beta step record")
                target = expected[step["step_id"]]
                if any(
                    step.get(name) != target[name]
                    for name in ("phase", "kind", "checksum")
                ):
                    raise MaintenanceError(
                        "Actual beta step identity differs from frozen producer"
                    )
                observed[step["step_id"]] = step
            for step in cast("list[dict[str, object]]", item["steps"]):
                if step["phase"] == "rollback" or (
                    phase == "start" and step["phase"] != "start"
                ):
                    continue
                if step["kind"] not in {
                    "apply_files",
                    "remove_entities",
                    "restore_definitions",
                }:
                    raise MaintenanceError(
                        "Data SQL requires its own exact domain settlement"
                    )
                actual = observed.get(step["id"], {})
                acknowledged = (
                    actual.get("status") == "completed" and actual.get("error") is None
                )
                if index != len(chosen) - 1 and not acknowledged:
                    raise MaintenanceError(
                        "Earlier phase lacks its original SQL acknowledgement"
                    )
                steps.append(
                    {
                        "id": step["id"],
                        "rollout": item["id"],
                        "phase": step["phase"],
                        "kind": step["kind"],
                        "checksum": step["checksum"],
                        "outcome": "acknowledged" if acknowledged else "reconciled",
                        "recorded_status": actual.get("status"),
                        "recorded_error": actual.get("error"),
                    }
                )
        target = chosen[-1]
        # Intermediate/expansion witnesses can settle SQL but never activate.
        final = phase == "complete" and target["id"] == ordered[-1]["id"]
        schemas = (
            ()
            if final
            else tuple(
                folder / cast("str", name)
                for name in (
                    [target["source_sql"], target["target_sql"]]
                    if phase == "start"
                    else [target["target_sql"]]
                )
            )
        )
        value = pending(state, str(selected["nonce"]))
        witness = canonical_tool(
            state, endpoint, value, binary, expected_schemas=schemas
        )
        if witness.get("previous_marker") != selected.get("previous_marker"):
            raise MaintenanceError(
                "Canonical predecessor changed during rollout settlement"
            )
        return {
            "outcome": "settled",
            "nonce": selected["nonce"],
            "input_sha256": selected["input_sha256"],
            "operation": {
                "owner": "canonical-frozen-rollout-v1",
                "identity": str(selected["nonce"]) + ":" + phase,
            },
            "phase": phase,
            "phase_complete": phase_complete,
            "steps": steps,
            "canonical": witness,
        }

    return operation


def settle_generated(
    state: Path, nonce: str, folder: Path, binary: Path
) -> dict[str, object]:
    return settle(state, nonce, generated_settler(folder, binary))


def canonical_tool(
    state: Path,
    endpoint: str,
    value: dict[str, object],
    binary: Path,
    *,
    activate: bool = False,
    expected_schemas: tuple[Path, ...] = (),
) -> dict[str, object]:
    """Run the fixed canonical owner under exact child and lifecycle ownership."""
    owner = surreal_server.read_json(state / "lifecycle-owner.json")
    exact_owner(state, owner)
    surreal_server.require_maintenance_owner(state, surreal_server.config_for(state))
    if value.get("initializer") != owner:
        raise MaintenanceError(
            "Canonical tool requires this reservation's initializer authority"
        )
    directory = surreal_server.checked_directory(
        state / ".surrealkit" / str(value["nonce"])
    )
    attempt = directory / uuid.uuid4().hex
    attempt.mkdir(mode=0o700)
    surreal_server.write_json(
        attempt / "authority.json",
        {
            "owner": owner,
            "listener": surreal_server.read_json(state / ".maintenance-listener.json"),
            "endpoint": endpoint,
            "definition_inputs": [
                {"path": str(path), "sha256": surrealkit.digest(path)}
                for path in expected_schemas
            ],
        },
    )
    record: dict[str, object] = {
        "directory": str(attempt),
        "phase": "prepared",
        "child_scope_requested": True,
    }
    cast("list[dict[str, object]]", value.setdefault("canonical_attempts", [])).append(
        record
    )
    update(state, value)
    observation: dict[str, object] = {}
    try:
        status = native_operation.run_child_scope(
            [
                sys.executable,
                "-m",
                "scripts.surrealkit_maintenance",
                "_canonical-child",
                "--state",
                str(state),
                "--binary",
                str(binary),
                "--folder",
                str(surrealkit.ROOT),
                "--attempt",
                str(attempt),
                "--mode",
                "canonical-activate" if activate else "canonical-validate",
                "--timeout",
                "120",
            ],
            {
                **os.environ,
                "SURREALDB_HOST": endpoint,
                "PSE_CANONICAL_INITIALIZER": str(owner["nonce"]),
            },
            cwd=surrealkit.ROOT,
            observation=observation,
        )
        record.update(exit_code=status, child=observation)
        result = surreal_server.read_json(attempt / "child.json")
        exact_owner(state, owner)
        surreal_server.require_maintenance_owner(
            state, surreal_server.config_for(state)
        )
        if (
            status
            or result.get("reason") != "exited"
            or observation.get("owned_drain") != "complete"
        ):
            raise MaintenanceError(  # noqa: TRY301 -- exact pending/CLI refusal boundary
                "Canonical operation lacks acknowledged success and owned drain; pending retained"
            )
        witness = json.loads((attempt / "stdout").read_text())
        if (
            not isinstance(witness, dict)
            or any(
                witness.get(name) != value.get(name)
                for name in ("namespace", "database", "instance_id")
            )
            or witness.get("initializer_nonce") != owner["nonce"]
        ):
            raise MaintenanceError(  # noqa: TRY301 -- exact pending/CLI refusal boundary
                "Canonical witness differs from selected maintenance authority"
            )
        if not activate and (
            witness.get("scope")
            != ("rollout-phase" if expected_schemas else "generated-target")
            or witness.get("definition_inputs")
            != [
                surrealkit.digest(path)
                for path in (
                    expected_schemas or (surrealkit.ROOT / surrealkit.GENERATED_SCHEMA,)
                )
            ]
        ):
            raise MaintenanceError(  # noqa: TRY301 -- exact pending/CLI refusal boundary
                "Canonical witness differs from exact definition inputs"
            )
        record["phase"] = "acknowledged"
        return witness  # noqa: TRY300 -- interruptions during ACK bookkeeping retain unknown phase
    except BaseException:
        record.update(phase="unknown", child=observation)
        raise
    finally:
        update(state, value)


def activate(state: Path, nonce: str, binary: Path) -> dict[str, object]:
    """Publish only the exact settled operation, then drain before releasing its fence."""
    with surreal_server.lifecycle_reservation(state):
        value = pending(state, nonce)
        if (
            value.get("phase") != "settled"
            or value.get("sql_outcome") != "settled"
            or cast("Mapping[str, object]", value.get("settlement", {})).get(
                "phase_complete"
            )
            is False
            or cast(
                "Mapping[str, object]",
                cast("Mapping[str, object]", value.get("settlement", {})).get(
                    "canonical", {}
                ),
            ).get("scope")
            != "generated-target"
        ):
            raise MaintenanceError(
                "Activation requires final canonical operation settlement"
            )
        require_child_drains(state, value)
        owner = surreal_server.read_json(state / "lifecycle-owner.json")
        value["initializer"] = owner
        update(state, value)
        # Historical settlement evidence is immutable; the refreshed live initializer
        # belongs to this attempt, not to the earlier validator invocation.
        surreal_server.write_json(state / ".surrealkit" / nonce / "settled.json", value)
        surreal_server.drain_maintenance_listener(state)
        endpoint = surreal_server.maintenance_endpoint(state)
        try:
            witness = canonical_tool(state, endpoint, value, binary, activate=True)
            value["activation"] = witness
            update(state, value)
        finally:
            surreal_server.drain_maintenance_listener(state)
        exact_owner(state, owner)
        with surreal_server.state_lock(state, cleanup=True):
            config = surreal_server.config_for(state)
            current = config.get(PENDING)
            if (
                not isinstance(current, dict)
                or current.get("nonce") != nonce
                or current.get("settlement") != value["settlement"]
            ):
                raise MaintenanceError(
                    "Pending selection changed after activation; fence retained"
                )
            surreal_server.write_json(
                state / ".surrealkit" / nonce / "activated.json", value
            )
            config.pop(PENDING)
            config["accepting_writes"] = False
            surreal_server.write_json(state / "config.json", config)
        return value


def owned_child(
    binary: Path,
    state: Path,
    folder: Path,
    attempt: Path,
    mode: str,
    rollout_id: str | None,
    timeout: float,
) -> int:
    """Internal admitted child: watch exact owner, stop/reap only this new session."""
    if native_operation.scope_owner() is None:
        raise MaintenanceError("Internal tool child requires its admitted native scope")
    if not math.isfinite(timeout) or timeout <= 0:
        raise MaintenanceError("Invalid internal child clock")
    owner_path = native_operation.current()
    if owner_path is None:
        raise MaintenanceError("Internal child lacks its exact native record")
    native_record = surreal_server.read_json(owner_path)
    surreal_server.write_json(
        attempt / "child.json", {"reason": "prepared", "native_record": native_record}
    )
    authority = surreal_server.read_json(attempt / "authority.json")

    def require_bound_owner() -> None:
        config = surreal_server.config_for(state)
        exact_owner(state, cast("Mapping[str, object]", authority["owner"]))
        surreal_server.require_maintenance_owner(state, config)
        if (
            surreal_server.read_json(state / ".maintenance-listener.json")
            != authority["listener"]
            or authority["endpoint"] != "http://127.0.0.1:" + str(config["port"])
            or os.environ.get("SURREALDB_HOST") != authority["endpoint"]
        ):
            raise MaintenanceError("Original maintenance listener binding changed")

    owner_lost = threading.Event()

    def watch_bound_owner(child: subprocess.Popen[bytes]) -> None:
        while child.poll() is None:
            try:
                require_bound_owner()
            except (
                MaintenanceError,
                surreal_server.SupervisorError,
                OSError,
                KeyError,
                ValueError,
            ):
                owner_lost.set()
                return
            time.sleep(0.1)

    interrupted = threading.Event()
    old_handlers = {
        sig: signal.signal(sig, lambda _sig, _frame: interrupted.set())
        for sig in (signal.SIGTERM, signal.SIGINT)
    }
    if mode == "canonical-validate":
        arguments = ["canonical-validate-definitions", "--state", str(state)]
        for definition in cast(
            "list[dict[str, str]]", authority.get("definition_inputs", [])
        ):
            path = Path(definition["path"])
            if surrealkit.digest(path) != definition["sha256"]:
                raise MaintenanceError("Exact phase definition input changed")
            arguments.extend(["--schema", str(path)])
    elif mode == "canonical-activate":
        current = pending(
            state,
            str(
                cast("Mapping[str, object]", surreal_server.config_for(state)[PENDING])[
                    "nonce"
                ]
            ),
        )
        arguments = [
            "canonical-activate",
            "--state",
            str(state),
            "--settlement",
            str(state / ".surrealkit" / str(current["nonce"]) / "settled.json"),
        ]
    else:
        arguments = command(mode, rollout_id)
    child: subprocess.Popen[bytes] | None = None
    child_pidfd: int | None = None
    reason = "not_started"
    status = 125
    try:
        require_bound_owner()
        with (
            (attempt / "stdout").open("xb") as stdout,
            (attempt / "stderr").open("xb") as stderr,
        ):
            child = subprocess.Popen(
                [str(binary), *arguments],
                env=dict(os.environ),
                cwd=folder,
                stdout=stdout,
                stderr=stderr,
                start_new_session=True,
            )
            # The unreaped Popen child pins its PID until this fd is captured.
            child_pidfd = os.pidfd_open(child.pid)
            threading.Thread(
                target=watch_bound_owner, args=(child,), daemon=True
            ).start()
            deadline = time.monotonic() + timeout
            while True:
                try:
                    require_bound_owner()
                except (
                    MaintenanceError,
                    surreal_server.SupervisorError,
                    OSError,
                    KeyError,
                    ValueError,
                ):
                    reason = "owner_lost"
                    break
                if owner_lost.is_set():
                    reason = "owner_lost"
                    break
                if interrupted.is_set():
                    reason = "interrupted"
                    break
                if time.monotonic() >= deadline:
                    reason = "timeout"
                    break
                try:
                    status = child.wait(timeout=min(0.1, deadline - time.monotonic()))
                    try:
                        require_bound_owner()
                    except (
                        MaintenanceError,
                        surreal_server.SupervisorError,
                        OSError,
                        KeyError,
                        ValueError,
                    ):
                        reason = "owner_lost"
                    else:
                        reason = "exited"
                    break
                except subprocess.TimeoutExpired:
                    continue
    finally:
        try:
            if child is not None:
                if reason != "exited":
                    with contextlib.suppress(ProcessLookupError):
                        if child_pidfd is not None:
                            signal.pidfd_send_signal(child_pidfd, signal.SIGTERM)
                    try:
                        child.wait(timeout=2)
                    except subprocess.TimeoutExpired:
                        with contextlib.suppress(ProcessLookupError):
                            if child_pidfd is not None:
                                signal.pidfd_send_signal(child_pidfd, signal.SIGKILL)
                        child.wait(timeout=8)
                # Process reap is distinct from the outer native scope drain.
                status = child.returncode if child.returncode is not None else 125
            surreal_server.write_json(
                attempt / "child.json",
                {"reason": reason, "exit_code": status, "native_record": native_record},
            )
        finally:
            if child_pidfd is not None:
                os.close(child_pidfd)
            for sig, handler in old_handlers.items():
                signal.signal(sig, handler)
    return status if reason == "exited" and status >= 0 else 125


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "mode",
        choices=[
            "up",
            "start",
            "complete",
            "status",
            "repair",
            "settle",
            "activate",
            "_child",
            "_canonical-child",
        ],
    )
    parser.add_argument("--state", type=Path, required=True)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--folder", type=Path, required=True)
    parser.add_argument("--rollout-id")
    parser.add_argument("--pending-nonce")
    parser.add_argument("--timeout", type=float, default=120)
    parser.add_argument("--attempt", type=Path)
    parser.add_argument("--mode", dest="child_mode")
    args = parser.parse_args()
    try:
        if args.mode in {"_child", "_canonical-child"}:
            if args.attempt is None or args.child_mode is None:
                raise MaintenanceError("Internal child identity missing")  # noqa: TRY301 -- exact pending/CLI refusal boundary
            return owned_child(
                args.binary,
                args.state,
                args.folder,
                args.attempt,
                args.child_mode,
                args.rollout_id,
                args.timeout,
            )
        if args.mode == "settle":
            if args.pending_nonce is None:
                raise MaintenanceError("Settlement requires the exact pending nonce")  # noqa: TRY301 -- exact pending/CLI refusal boundary
            print(
                json.dumps(
                    settle_generated(
                        args.state, args.pending_nonce, args.folder, args.binary
                    ),
                    indent=2,
                )
            )
            return 0
        if args.mode == "activate":
            if args.pending_nonce is None:
                raise MaintenanceError("Activation requires the exact pending nonce")  # noqa: TRY301 -- exact pending/CLI refusal boundary
            print(
                json.dumps(
                    activate(args.state, args.pending_nonce, args.binary), indent=2
                )
            )
            return 0
        result = execute(
            args.binary,
            args.state,
            args.folder,
            args.mode,
            rollout_id=args.rollout_id,
            pending_nonce=args.pending_nonce,
            timeout=args.timeout,
        )
        print(json.dumps(result, indent=2))
        return (
            0
            if cast("list[dict[str, object]]", result["attempts"])[-1]["phase"]
            == "cli_succeeded"
            else 1
        )
    except (
        MaintenanceError,
        surreal_rollout_producer.ProducerError,
        surrealkit.ToolError,
        surreal_server.SupervisorError,
        OSError,
        ValueError,
        subprocess.SubprocessError,
    ) as error:
        print(f"surrealkit-maintenance: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
