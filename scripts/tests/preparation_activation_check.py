# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Authentic private-v3 preparation activation, interruption and refusal journey.

All states and archives remain on disk for inspection. No production state is
selected, and every native operation borrows a real supervisor reservation.
"""

from __future__ import annotations

import argparse
import base64
import ctypes
import json
import os
import selectors
import socket
import subprocess
import sys
import time
import urllib.request
from pathlib import Path
from unittest.mock import patch

from scripts import preparation_activation as activation
from scripts import surreal_server as server

LEGACY_SCHEMA = "fbd5217949165ea2e4a4c18ccb28d12a2fc2ff255819f18c6bf486676b0cc916"


def native_seed(
    state: Path, bridge: Path, *, nonpreparation: bool, history: bool = False
) -> None:
    command = [str(bridge), str(state), str(state.parent), "--private-fixture-seed"]
    if nonpreparation:
        command.append("--private-fixture-nonpreparation")
    if history:
        command.append("--private-fixture-history")
    child = subprocess.Popen(
        command,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
        bufsize=1,
    )
    complete = False
    try:
        if child.stdin is None or child.stdout is None:
            raise server.SupervisorError("Private fixture control pipe unavailable")
        with selectors.DefaultSelector() as selector:
            selector.register(child.stdout, selectors.EVENT_READ)
            while True:
                if not selector.select(timeout=0.2):
                    if child.poll() is not None:
                        break
                    continue
                raw = child.stdout.readline(65537)
                if not raw:
                    break
                request = json.loads(raw)
                selected = server.config_for(state)
                server.require_maintenance_owner(state, selected)
                if not server.listener_ready(state, selected):
                    raise server.SupervisorError(
                        "Private fixture lost its actual listener"
                    )
                if request.get("event") == "authorize":
                    child.stdin.write(
                        json.dumps(
                            {
                                "step": request["step"],
                                "operation": "private-fixture-originals",
                            }
                        )
                        + "\n"
                    )
                    child.stdin.flush()
                elif request.get("event") == "complete":
                    complete = True
                elif request.get("event") == "failed" and isinstance(
                    request.get("reason"), str
                ):
                    reason = request["reason"][-4096:]
                    for name, secret in server.read_json(
                        state / "credentials.json"
                    ).items():
                        if "password" in name and isinstance(secret, str):
                            reason = reason.replace(secret, "[REDACTED]")
                    raise server.SupervisorError(
                        "Private native seed failed: " + reason
                    )
                else:
                    raise server.SupervisorError("Unknown private fixture control")
        if child.wait(timeout=10) or not complete:
            raise server.SupervisorError("Private native seed failed")
    finally:
        if child.poll() is None:
            child.terminate()
            try:
                child.wait(timeout=10)
            except subprocess.TimeoutExpired:
                child.kill()
                child.wait()
        if child.stdin is not None:
            child.stdin.close()
        if child.stdout is not None:
            child.stdout.close()


def initialize(
    root: Path,
    bridge: Path,
    schema: str,
    *,
    nonpreparation: bool,
    history: bool = False,
) -> Path:
    state = root / "state"
    with socket.socket() as selected:
        selected.bind(("127.0.0.1", 0))
        port = selected.getsockname()[1]
    if server.main(
        [
            "setup",
            "--state",
            str(state),
            "--interpretation",
            "pse.substrate.v3",
            "--port",
            str(port),
            "--memory-mib",
            "1536",
            "--server-memory-mib",
            "1024",
            "--native-workers",
            "1",
            "--native-worker-memory-mib",
            "512",
        ]
    ):
        raise server.SupervisorError("Private activation service setup failed")
    with server.lifecycle_reservation(state):
        config = server.config_for(state)
        config.update(accepting_writes=False, admission="validation_required")
        server.write_json(state / "config.json", config)
        # Only this newly created private fixture has no persisted root account.
        # Ordinary authenticated readiness bootstraps it; maintenance deliberately
        # never bootstraps candidate credentials into an existing catalog.
        server.start(state, config, validation=True)
        server.stop(state, config)
        server.private_offline_state(state, config)
        credentials = server.read_json(state / "credentials.json")
        # The pinned CLI consumes piped input one line at a time. Submit the
        # complete generated definitions as one transaction, preserving SQL and
        # removing only the generator's standalone line comments.
        fixture_schema = " ".join(
            line
            for line in schema.splitlines()
            if not line.lstrip().startswith(("--", "//"))
        )
        server.maintenance_query(
            state,
            config,
            credentials,
            fixture_schema
            + " LET $created=(CREATE canonical_interpretations:current SET key='current',interpretation='pse.substrate.v3',schema_digest='"
            + LEGACY_SCHEMA
            + "' RETURN NONE); LET $value=true;",
        )
        native_seed(state, bridge, nonpreparation=nonpreparation, history=history)
    return state


def inspect(state: Path, body: str) -> object:
    with server.lifecycle_reservation(state):
        config = server.config_for(state)
        server.private_offline_state(state, config)
        server.systemctl("reset-failed", server.unit_name(state))
        return server.maintenance_query(
            state, config, server.read_json(state / "credentials.json"), body
        )["value"]


def activate(
    state: Path, archive: Path, bridge: Path, migration: str
) -> dict[str, object]:
    # This journey deliberately starts/stops one private service repeatedly.
    # Reset its retry counter only after the real offline/drain checks, without
    # changing service restart policy or bypassing any activation owner checks.
    with server.lifecycle_reservation(state):
        server.private_offline_state(state, server.config_for(state))
        server.systemctl("reset-failed", server.unit_name(state))
        return activation.execute(state, migration, archive, bridge)


def exit_owner_after_partial(root: Path, bridge: Path) -> None:
    """A disposable real supervisor parent exits without running cleanup."""
    state = root / "state"
    archive = root / "archive"
    authentic = activation.authorize_native

    def exit_after_effect(
        child: subprocess.Popen[str], request: dict[str, object], digest: str
    ) -> None:
        if request.get("phase") == "readback:canonical_body_contents":
            children = (
                Path(f"/proc/{os.getpid()}/task/{os.getpid()}/children")
                .read_text()
                .split()
            )
            native = [
                int(pid)
                for pid in children
                if Path(f"/proc/{pid}/exe").resolve() == bridge.resolve()
            ]
            if len(native) != 1:
                raise server.SupervisorError(
                    "Owner-loss control cannot identify its exact native child"
                )
            activation.frozen_json(
                root / "owner-exit.json",
                {
                    "parent_pid": os.getpid(),
                    "parent_start": server.native_operation.start_identity(os.getpid()),
                    "native_pid": native[0],
                    "native_start": server.native_operation.start_identity(native[0]),
                    "launch": server.read_json(state / "service-launch.json"),
                    "storage_process": server.read_json(state / "server-process.json"),
                    "operation_digest": server.file_digest(archive / "operation.json"),
                    "snapshot_digest": server.file_digest(
                        archive / "native-snapshot.json"
                    ),
                    "intent_digest": server.file_digest(archive / "intent.json"),
                },
            )
            os._exit(91)
        authentic(child, request, digest)

    with patch.object(activation, "authorize_native", side_effect=exit_after_effect):
        activate(state, archive, bridge, "private-owner-loss-migration")
    raise server.SupervisorError(
        "Owner-loss control did not reach a real partial effect"
    )


def owner_loss_journey(root: Path, bridge: Path, schema: str) -> dict[str, object]:
    root.mkdir(mode=0o700)
    state = initialize(root, bridge, schema, nonpreparation=False)
    # Reap the deliberately orphaned native child ourselves, so a container's
    # unrelated PID 1 cannot leave its zombie obscuring exact process drain.
    libc = ctypes.CDLL(None, use_errno=True)
    if libc.prctl(36, 1, 0, 0, 0):  # Linux PR_SET_CHILD_SUBREAPER
        raise OSError(
            ctypes.get_errno(), "Cannot retain native orphan reaping ownership"
        )
    child = subprocess.Popen(
        [
            sys.executable,
            "-m",
            "scripts.tests.preparation_activation_check",
            "--root",
            str(root),
            "--bridge",
            str(bridge),
            "--owner-exit",
        ],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
        text=True,
    )
    try:
        _, error = child.communicate(timeout=120)
        if child.returncode != 91:
            raise server.SupervisorError(
                "Owner-exit fixture failed before its controlled effect: "
                + error[-4096:]
            )
    finally:
        if child.poll() is None:
            child.terminate()
            child.wait(timeout=10)
    proof = server.read_json(root / "owner-exit.json")
    pid = server.integer(proof["native_pid"])
    deadline = time.monotonic() + 30
    while True:
        reaped, native_status = os.waitpid(pid, os.WNOHANG)
        if reaped == pid:
            break
        if time.monotonic() >= deadline:
            raise server.SupervisorError(
                "Exact orphaned native operation did not physically drain"
            )
        time.sleep(0.05)
    launch = server.object_mapping(proof["launch"])
    binding = server.object_mapping(launch["binding"])
    while True:
        observed = server.systemctl(
            "show",
            "--property=ActiveState,ControlGroup,InvocationID",
            server.unit_name(state),
        )
        fields = dict(
            line.split("=", 1) for line in observed.stdout.splitlines() if "=" in line
        )
        if fields.get("ActiveState") in {"inactive", "failed"} and not fields.get(
            "ControlGroup"
        ):
            break
        if time.monotonic() >= deadline:
            raise server.SupervisorError(
                "Owner-loss supervisor did not stop its exact storage invocation"
            )
        if (
            fields.get("InvocationID") != binding["invocation"]
            or fields.get("ControlGroup") != binding["group"]
        ):
            raise server.SupervisorError(
                "Owner-loss fixture storage invocation changed"
            )
        time.sleep(0.05)
    config = server.config_for(state)
    process = server.object_mapping(proof["storage_process"])
    try:
        if (
            server.native_operation.start_identity(server.integer(process["pid"]))
            == process["start"]
        ):
            raise server.SupervisorError(
                "Owner-loss storage process survives its stopped unit"
            )
    except FileNotFoundError:
        pass
    if config["accepting_writes"] or config["admission"] != "validation_required":
        raise server.SupervisorError("Owner loss reopened operational admission")
    partial = inspect(
        state,
        "LET $value={interpretation:(SELECT VALUE interpretation FROM ONLY canonical_interpretations:current),contents:array::len((SELECT key FROM canonical_body_contents))};",
    )
    if partial != {"interpretation": "pse.substrate.v3", "contents": 1}:
        raise server.SupervisorError(
            "Owner loss did not preserve its real partial activation"
        )
    resumed = activate(state, root / "archive", bridge, "private-owner-loss-migration")
    for name in ("operation", "snapshot", "intent"):
        filename = "native-snapshot.json" if name == "snapshot" else name + ".json"
        if server.file_digest(root / "archive" / filename) != proof[name + "_digest"]:
            raise server.SupervisorError(
                "Owner-loss resume replaced original immutable material"
            )
    return {
        "parent_exit": 91,
        "native_exit": os.waitstatus_to_exitcode(native_status),
        "native_process_reaped": pid,
        "storage_invocation_drained": binding,
        "interrupted_partial": partial,
        "resumed": resumed,
        "state_retained": str(state),
    }


def validate_history_fixture(state: Path) -> None:
    """An actual authenticated semantic validator, invoked by the lifecycle owner."""
    config = server.config_for(state)
    if config["accepting_writes"] or config["admission"] != "validation_required":
        raise server.SupervisorError(
            "Historical fixture validation lost closed admission"
        )
    credentials = server.read_json(state / "credentials.json")
    token = base64.b64encode(
        (str(credentials["username"]) + ":" + str(credentials["password"])).encode()
    ).decode()
    sql = "RETURN {marker:(SELECT VALUE interpretation FROM ONLY canonical_interpretations:current),admissions:array::len((SELECT key FROM canonical_historical_results)),locators:array::len((SELECT key FROM canonical_result_rows)),bytes:math::sum((SELECT VALUE bytes::len(payload) FROM canonical_result_batches)),retired:(SELECT VALUE complete FROM ONLY type::record('canonical_result_retirements','retired-original-run')),step:(SELECT VALUE <string><int>step FROM canonical_result_rows WHERE row=0dec LIMIT 1),old_runs:array::len((SELECT key FROM canonical_runs WHERE interpretation='pse.substrate.v3'))};"
    request = urllib.request.Request(
        "http://127.0.0.1:" + str(config["port"]) + "/sql",
        data=sql.encode(),
        headers={
            "Authorization": "Basic " + token,
            "Accept": "application/json",
            "surreal-ns": str(config["namespace"]),
            "surreal-db": str(config["database"]),
        },
    )
    with urllib.request.urlopen(request, timeout=30) as response:
        result = json.load(response)
    if (
        not isinstance(result, list)
        or len(result) != 1
        or result[0].get("status") != "OK"
    ):
        raise server.SupervisorError("Historical fixture semantic query failed")
    value = result[0]["result"]
    if (
        not isinstance(value, dict)
        or value.get("marker") != server.SUBSTRATE_INTERPRETATION
        or value.get("admissions") != 3
        or value.get("locators") != 82 * 2048
        or value.get("bytes", 0) <= 16 * 1024 * 1024
        or value.get("retired") is not True
        or value.get("step") != ["4000000001"]
        or value.get("old_runs") != 3
    ):
        raise server.SupervisorError("Historical fixture semantic validation differs")


def read_history_fixture(
    state: Path, archive: Path, bridge: Path
) -> list[dict[str, object]]:
    # Repeated deliberate start/stop cycles are private-fixture actions, not
    # automatic service retries. Reset only after the exact offline owner check.
    with server.lifecycle_reservation(state):
        server.private_offline_state(state, server.config_for(state))
        server.systemctl("reset-failed", server.unit_name(state))
        server.start(state, server.config_for(state))
    try:
        read = subprocess.run(
            [str(bridge), str(state), str(archive), "--private-fixture-read-history"],
            capture_output=True,
            text=True,
            timeout=120,
            check=True,
        )
        public = [
            json.loads(line)
            for line in read.stdout.splitlines()
            if line.startswith("{")
        ]
        if not any(
            item.get("historical_public_reader") == "original-ipc-verified"
            for item in public
        ):
            raise server.SupervisorError("Public historical reader did not complete")
        return public
    finally:
        with server.lifecycle_reservation(state):
            server.stop(state, server.config_for(state))


def general_history_journey(root: Path, bridge: Path, schema: str) -> dict[str, object]:
    root.mkdir(mode=0o700)
    state = initialize(root, bridge, schema, nonpreparation=False, history=True)
    original = inspect(
        state,
        "LET $value={roots:(SELECT * OMIT id FROM canonical_roots ORDER BY key),runs:(SELECT * OMIT id FROM canonical_runs ORDER BY key),receipts:(SELECT * OMIT id FROM canonical_execution_operations ORDER BY key),studies:(SELECT * OMIT id FROM canonical_studies),analyses:(SELECT * OMIT id FROM canonical_analyses),retirements:(SELECT * OMIT id FROM canonical_result_retirements),bytes:math::sum((SELECT VALUE bytes::len(payload) FROM canonical_result_batches))};",
    )
    if not isinstance(original, dict) or original["bytes"] <= 16 * 1024 * 1024:
        raise server.SupervisorError(
            "General fixture has no real paged payload pressure"
        )
    attempt = next(
        run["current_attempt"]
        for run in original["runs"]
        if run["key"] == "original-run"
    )
    # Each refusal archive retains the exact unsupported original snapshot; only
    # this explicit private fixture changes its injected premise between cases.
    for name, change, restore in [
        (
            "open",
            "LET $fixture_mutation=(UPDATE type::record('canonical_attempts','"
            + attempt
            + "') SET ingestion_open=true RETURN NONE);",
            "LET $fixture_mutation=(UPDATE type::record('canonical_attempts','"
            + attempt
            + "') SET ingestion_open=false RETURN NONE);",
        ),
        (
            "unknown",
            "LET $fixture_mutation=(UPDATE type::record('canonical_runs','original-run') SET interpretation='unknown.fixture' RETURN NONE);",
            "LET $fixture_mutation=(UPDATE type::record('canonical_runs','original-run') SET interpretation='pse.substrate.v3' RETURN NONE);",
        ),
        (
            "unknown-study",
            "LET $fixture_mutation=(UPDATE type::record('canonical_studies','original-study') SET interpretation='unknown.fixture' RETURN NONE);",
            "LET $fixture_mutation=(UPDATE type::record('canonical_studies','original-study') SET interpretation='pse.substrate.v3' RETURN NONE);",
        ),
        (
            "unknown-analysis",
            "LET $fixture_mutation=(UPDATE type::record('canonical_analyses','original-analysis') SET interpretation='unknown.fixture' RETURN NONE);",
            "LET $fixture_mutation=(UPDATE type::record('canonical_analyses','original-analysis') SET interpretation='pse.substrate.v3' RETURN NONE);",
        ),
        (
            "additional-marker",
            "LET $fixture_mutation=(CREATE type::record('canonical_interpretations','not-current') SET key='not-current',interpretation='pse.substrate.v3',schema_digest='"
            + LEGACY_SCHEMA
            + "' RETURN NONE);",
            "LET $fixture_mutation=(DELETE ONLY type::record('canonical_interpretations','not-current') RETURN NONE);",
        ),
    ]:
        inspect(state, change + " LET $value=true;")
        try:
            activate(state, root / (name + "-archive"), bridge, "private-" + name)
        except server.SupervisorError:
            pass
        else:
            raise server.SupervisorError(
                "Unsupported historical " + name + " shape was activated"
            )
        marker = inspect(
            state,
            "LET $value=(SELECT VALUE interpretation FROM ONLY canonical_interpretations:current);",
        )
        if marker != "pse.substrate.v3":
            raise server.SupervisorError("History refusal moved the original marker")
        inspect(state, restore + " LET $value=true;")
        # A different migration must use its own fresh closed fixture owner;
        # clear only this test's failed operation selection after checking refusal.
        with server.lifecycle_reservation(state):
            config = server.config_for(state)
            if (
                config["accepting_writes"]
                or config["admission"] != "validation_required"
            ):
                raise server.SupervisorError("Refusal reopened private fixture")
            config.pop("preparation_activation", None)
            server.write_json(state / "config.json", config)
    archive = root / "archive"
    authentic = activation.authorize_native

    def interrupt(
        child: subprocess.Popen[str], request: dict[str, object], digest: str
    ) -> None:
        if request.get("phase") == "readback:canonical_body_contents":
            raise server.SupervisorError("Controlled general history partial effect")
        authentic(child, request, digest)

    with patch.object(activation, "authorize_native", side_effect=interrupt):
        try:
            activate(state, archive, bridge, "private-general-history")
        except server.SupervisorError as error:
            if "Controlled general history" not in str(error):
                raise
        else:
            raise server.SupervisorError(
                "General history did not exercise partial effect"
            )
    partial = inspect(
        state,
        "LET $value={marker:(SELECT VALUE interpretation FROM ONLY canonical_interpretations:current),contents:array::len((SELECT key FROM canonical_body_contents))};",
    )
    if partial != {"marker": "pse.substrate.v3", "contents": 1}:
        raise server.SupervisorError("General history partial effect or marker differs")
    frozen = {
        name: server.file_digest(archive / name)
        for name in [
            "operation.json",
            "native-snapshot.json",
            "native-history-derived.json",
        ]
    }
    source = server.read_json(archive / "native-snapshot.json")["history"]
    if (
        sum(page["bytes"] for page in source["pages"]) <= 16 * 1024 * 1024
        or len(source["pages"]) <= 1
    ):
        raise server.SupervisorError(
            "Historical original archive was not genuinely paged"
        )
    page = archive / "native-history" / source["pages"][0]["name"]
    unchanged = page.read_bytes()
    page.write_bytes(unchanged + b"corrupt")
    try:
        activate(state, archive, bridge, "private-general-history")
    except server.SupervisorError:
        pass
    else:
        raise server.SupervisorError("Corrupt frozen historical page was admitted")
    page.write_bytes(unchanged)
    resume_conflicts = []
    original_issuer = str(
        server.read_json(archive / "intent.json")["legacy_issuing_store"]
    )
    for name, change, restore in [
        (
            "original-operation",
            "LET $fixture_mutation=(CREATE type::record('canonical_execution_operations','added-original-conflict') SET key='added-original-conflict',run='original-run',kind='fixture-conflict',request=type::bytes(''),result=type::bytes('') RETURN NONE);",
            "LET $fixture_mutation=(DELETE ONLY type::record('canonical_execution_operations','added-original-conflict') RETURN NONE);",
        ),
        (
            "open-source-stage",
            "LET $fixture_mutation=(CREATE type::record('canonical_stages','added-open-source-stage') SET key='added-open-source-stage',problem='fixture-problem',expected_head=NONE,request_digest='private-added-stage',expires_at=0,closed=false,activated=false,edit_count=0,generation=0,abandoned=false,cleanup_complete=false RETURN NONE);",
            "LET $fixture_mutation=(DELETE ONLY type::record('canonical_stages','added-open-source-stage') RETURN NONE);",
        ),
        (
            "unknown-table",
            "DEFINE TABLE private_unknown_activation_fixture SCHEMALESS; LET $fixture_mutation=(CREATE type::record('private_unknown_activation_fixture','added-unknown-original') SET key='added-unknown-original' RETURN NONE);",
            "LET $fixture_mutation=(DELETE ONLY type::record('private_unknown_activation_fixture','added-unknown-original') RETURN NONE);",
        ),
        (
            "additional-marker",
            "LET $fixture_mutation=(CREATE type::record('canonical_interpretations','not-current') SET key='not-current',interpretation='pse.substrate.v3',schema_digest='"
            + LEGACY_SCHEMA
            + "',issuing_store='"
            + original_issuer
            + "' RETURN NONE);",
            "LET $fixture_mutation=(DELETE ONLY type::record('canonical_interpretations','not-current') RETURN NONE);",
        ),
    ]:
        inspect(state, change + " LET $value=true;")
        try:
            activate(state, archive, bridge, "private-general-history")
        except server.SupervisorError:
            pass
        else:
            raise server.SupervisorError(
                "New historical " + name + " member escaped frozen inventory"
            )
        if (
            inspect(
                state,
                "LET $value=(SELECT VALUE interpretation FROM ONLY canonical_interpretations:current);",
            )
            != "pse.substrate.v3"
        ):
            raise server.SupervisorError(
                "Original " + name + " conflict moved activation marker"
            )
        if any(
            server.file_digest(archive / filename) != digest
            for filename, digest in frozen.items()
        ):
            raise server.SupervisorError(
                "Conflict replaced the immutable operation or derived manifest"
            )
        inspect(state, restore + " LET $value=true;")
        resume_conflicts.append(name)
    completed = activate(state, archive, bridge, "private-general-history")
    replay = activate(state, archive, bridge, "private-general-history")
    if any(
        server.file_digest(archive / name) != digest for name, digest in frozen.items()
    ):
        raise server.SupervisorError(
            "General history resume changed immutable original owner"
        )
    preserved = inspect(
        state,
        "LET $value={roots:(SELECT * OMIT id FROM canonical_roots ORDER BY key),runs:(SELECT * OMIT id FROM canonical_runs ORDER BY key),receipts:(SELECT * OMIT id FROM canonical_execution_operations ORDER BY key),studies:(SELECT * OMIT id FROM canonical_studies),analyses:(SELECT * OMIT id FROM canonical_analyses),retirements:(SELECT * OMIT id FROM canonical_result_retirements),bytes:math::sum((SELECT VALUE bytes::len(payload) FROM canonical_result_batches))};",
    )
    if preserved != original:
        raise server.SupervisorError(
            "General activation changed original scientific or ownership records"
        )
    try:
        inspect(
            state,
            "LET $value=fn::pse_execution_v1::claim(time::micros()+10000000,'failed-only-original-run','stale-writer',type::bytes(''), 'stale-attempt','stale-worker',1000000,'pse.substrate.v3',NONE);",
        )
    except server.SupervisorError:
        pass
    else:
        raise server.SupervisorError("Activated server accepted a stale v3 writer")
    with server.lifecycle_reservation(state):
        config = server.config_for(state)
        if (
            config["accepting_writes"]
            or config["admission"] != "validation_required"
            or config["schema_interpretation"] != "pse.substrate.v3"
        ):
            raise server.SupervisorError(
                "Adapter implicitly readmitted historical sources"
            )
        activation.frozen_json(root / "original-private-profile.json", config)
        server.maintenance_endpoint(state)
        server.validate_interpretation(state, config, server.SUBSTRATE_INTERPRETATION)
        server.stop(state, config)
        # Request the authenticated validator under closed admission. Updating
        # the requested profile is not validation; validate() owns that decision.
        config.update(
            interpretation=server.SUBSTRATE_INTERPRETATION,
            schema_interpretation=server.SUBSTRATE_INTERPRETATION,
        )
        server.write_json(state / "config.json", config)
        server.validate(
            state,
            config,
            server.SUBSTRATE_INTERPRETATION,
            [
                sys.executable,
                "-m",
                "scripts.tests.preparation_activation_check",
                "--root",
                str(root),
                "--bridge",
                str(bridge),
                "--validate-state",
                str(state),
            ],
        )
    public = read_history_fixture(state, archive, bridge)
    return {
        "bytes": original["bytes"],
        "pages": len(source["pages"]),
        "resume_conflicts": resume_conflicts,
        "partial": partial,
        "completed": completed,
        "replay": replay,
        "public_reader": public,
        "originals_preserved": True,
        "state_retained": str(state),
    }


def run(root: Path, bridge: Path) -> dict[str, object]:
    root = server.checked_directory(root, empty=True)
    repo = Path(__file__).resolve().parents[2]
    result = subprocess.run(
        [
            "git",
            "show",
            "2d6bf6ae7d03a8a9f6f109696bfe3d9cc8455466:crates/pse-operations/src/generated/surreal.surql",
        ],
        cwd=repo,
        capture_output=True,
        text=True,
        check=True,
    )
    schema = result.stdout
    (root / "original-v3-schema.surql").write_text(schema)
    supported = root / "supported"
    supported.mkdir(mode=0o700)
    state = initialize(supported, bridge, schema, nonpreparation=False)
    archive = supported / "archive"
    authentic = activation.authorize_native

    def interrupt_after_first_derived_effect(
        child: subprocess.Popen[str], request: dict[str, object], digest: str
    ) -> None:
        # Refuse the real post-create read-back, so one body exists while the
        # original marker remains. Owner/backup/page checks already ran.
        if request.get("phase") == "readback:canonical_body_contents":
            raise server.SupervisorError(
                "Controlled offline interruption after first derived effect"
            )
        authentic(child, request, digest)

    with patch.object(
        activation, "authorize_native", side_effect=interrupt_after_first_derived_effect
    ):
        try:
            activate(state, archive, bridge, "private-original-migration")
        except server.SupervisorError as error:
            if "Controlled offline interruption" not in str(error):
                raise
        else:
            raise server.SupervisorError(
                "Private interruption did not exercise a real partial activation"
            )
    original_operation = (archive / "operation.json").read_bytes()
    original_snapshot = (archive / "native-snapshot.json").read_bytes()
    original_issuer = server.read_json(archive / "intent.json")["legacy_issuing_store"]
    partial = inspect(
        state,
        "LET $value={interpretation:(SELECT VALUE interpretation FROM ONLY canonical_interpretations:current),contents:array::len((SELECT key FROM canonical_body_contents))};",
    )
    if partial != {"interpretation": "pse.substrate.v3", "contents": 1}:
        raise server.SupervisorError(
            "Interruption did not retain a derived effect beneath its original marker"
        )
    first = activate(state, archive, bridge, "private-original-migration")
    replay = activate(state, archive, bridge, "private-original-migration")
    if (
        original_operation != (archive / "operation.json").read_bytes()
        or original_snapshot != (archive / "native-snapshot.json").read_bytes()
    ):
        raise server.SupervisorError("Resume replaced original migration material")
    summary = inspect(
        state,
        "LET $value={marker:(SELECT * OMIT id FROM ONLY canonical_interpretations:current),associations:(SELECT key,revision,producer,interpretation,historical_blob FROM canonical_product_associations ORDER BY key),descriptors:(SELECT association FROM canonical_product_descriptors),originals:(SELECT key,producer,interpretation FROM canonical_products ORDER BY key),contents:array::len((SELECT key FROM canonical_body_contents)),receipts:array::len((SELECT key FROM canonical_selection_receipts))};",
    )
    if (
        not isinstance(summary, dict)
        or summary["marker"]["issuing_store"] != original_issuer
        or summary["contents"] != 1
        or summary["receipts"] != 1
        or summary["descriptors"] != [{"association": "a-original"}]
    ):
        raise server.SupervisorError(
            "Private migrated history or old rooted ACK oracle differs"
        )
    if any(
        row["interpretation"] != "pse.substrate.v3"
        or row["producer"] != "original-producer"
        or row["revision"] != "original-revision"
        for row in summary["associations"]
    ):
        raise server.SupervisorError("Private historical provenance was upgraded")
    config = server.config_for(state)
    if config["accepting_writes"] or config["admission"] != "validation_required":
        raise server.SupervisorError(
            "Preparation activation implicitly readmitted old sources"
        )
    unsupported = root / "unsupported"
    unsupported.mkdir(mode=0o700)
    other = initialize(unsupported, bridge, schema, nonpreparation=True)
    try:
        activate(
            other, unsupported / "archive", bridge, "private-unsupported-migration"
        )
    except server.SupervisorError as error:
        if "canonical_execution_operations" not in str(error):
            raise
    else:
        raise server.SupervisorError(
            "Unsupported execution history was implicitly migrated"
        )
    refusal = inspect(
        other,
        "LET $value={interpretation:(SELECT VALUE interpretation FROM ONLY canonical_interpretations:current),operations:(SELECT key FROM canonical_execution_operations)};",
    )
    if refusal != {
        "interpretation": "pse.substrate.v3",
        "operations": [{"key": "unsupported-execution-history"}],
    }:
        raise server.SupervisorError(
            "Unsupported history refusal changed the original data or marker"
        )
    owner_loss = owner_loss_journey(root / "owner-loss", bridge, schema)
    general = general_history_journey(root / "general-history", bridge, schema)
    receipt = {
        "general_history": general,
        "interrupted_partial": partial,
        "supported": first,
        "idempotent_replay": replay,
        "historical_summary": summary,
        "nonpreparation_refusal": refusal,
        "owner_loss": owner_loss,
        "admission": "closed",
        "states_retained": [str(state), str(other), owner_loss["state_retained"]],
    }
    server.write_json(root / "receipt.json", receipt)
    return receipt


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--bridge", type=Path, required=True)
    parser.add_argument("--validate-state", type=Path)
    parser.add_argument("--owner-exit", action="store_true", help=argparse.SUPPRESS)
    args = parser.parse_args()
    if args.validate_state is not None:
        validate_history_fixture(args.validate_state)
        return 0
    if args.owner_exit:
        exit_owner_after_partial(args.root.absolute(), args.bridge.absolute())
        return 1
    print(json.dumps(run(args.root.absolute(), args.bridge.absolute()), sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
