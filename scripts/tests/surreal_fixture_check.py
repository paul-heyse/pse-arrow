# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Disposable released-server lifecycle check; no existing application state is used.

Run `python -m scripts.tests.surreal_fixture_check` for the journey or add
`--validate-only` to check the state passed in PSE_SURREAL_STATE during restore.
This administrative HTTP control exercises lifecycle; Rust owns native WebSocket checks.
"""

from __future__ import annotations

import argparse
import base64
import contextlib
import json
import os
import shutil
import socket
import subprocess
import sys
import tempfile
import time
import urllib.request
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Generator

from scripts import host_admission, native_operation
from scripts import surreal_server as server

INTERPRETATION = server.SUBSTRATE_INTERPRETATION
FIXTURE = {"operation": "ack-fixed-1", "exact_bits": "8000000000000000"}


def record_ownership(directory: Path, unit: str) -> None:
    observed = native_operation.unit_observation(unit)
    if not observed.get("ControlGroup") or not observed.get("InvocationID"):
        raise server.SupervisorError(
            "Fixture unit did not expose actual owned kernel membership"
        )
    path = directory / "fixture-ownership.json"
    record: dict[str, object] = (
        server.read_json(path) if path.exists() else {"scopes": []}
    )
    scopes = record["scopes"]
    if not isinstance(scopes, list):
        raise server.SupervisorError("Invalid fixture scope inventory")
    scopes.append(
        {
            "unit": unit,
            "group": observed["ControlGroup"],
            "invocation": observed["InvocationID"],
            "inode": host_admission.group_identity(observed["ControlGroup"]),
            "boot": host_admission.boot(),
        }
    )
    server.write_json(path, record)


def finish_fixture(directory: Path, evidence: Path | None) -> dict[str, object] | None:
    """Verify successful terminal state before saving proof and retiring new inputs."""
    proof: dict[str, object] | None = None
    for state in (directory / "live", directory / "restored"):
        if (state / "config.json").exists():
            server.all_contexts_drained(state)
            if server.active(state):
                raise server.SupervisorError(
                    "Successful fixture storage remains active"
                )
    record = server.read_json(directory / "fixture-ownership.json")
    scopes = record["scopes"]
    if not isinstance(scopes, list):
        raise server.SupervisorError("Invalid fixture scope inventory")
    scopes = [server.object_mapping(value) for value in scopes]
    if not all(
        host_admission.drained(
            {
                "boot": scope["boot"],
                "released": True,
                "units": {scope["unit"]: scope},
            }
        )
        for scope in scopes
    ):
        raise server.SupervisorError(
            "Fixture kernel lifetime remains undrained; retained"
        )
    if evidence is not None:
        evidence = evidence.resolve()
        if evidence.exists():
            raise server.SupervisorError(
                "Positive evidence destination already exists; refusing replacement"
            )
        evidence.mkdir(mode=0o700, parents=True)
        selected = server.config_for(directory / "live")
        controls: dict[str, object] = {}
        proof = {
            "version": 1,
            "outcome": "passed",
            "scope": "bounded owned process lifecycle and toy receiver artifact controls; no scientific or performance qualification",
            "fixture": str(directory),
            "fixture_retired": False,
            "fixture_source_sha256": server.file_digest(Path(__file__)),
            "supervisor_source_sha256": server.file_digest(server.SCRIPT),
            "server": selected["server"],
            "service_generation": selected["instance_id"],
            "service_supervisor": selected["service_supervisor"],
            "all_contexts_and_kernel_scopes_drained": True,
            "owned_scopes": record["scopes"],
            "controls": controls,
        }
        for name, path in {
            "receiver_closures": directory / "receiver-closures.json",
            "restart_exhaustion": directory / "restart-exhaustion.json",
            "recovery_qualification": directory / "live/recovery-qualification.json",
        }.items():
            if path.is_file():
                controls[name] = server.read_json(path)
        server.write_json(evidence / "passed.json", proof)
    return proof


@contextlib.contextmanager
def owned_fixture(
    root: Path, evidence: Path | None = None
) -> Generator[Path, None, None]:
    """Retain every incomplete/new fixture; clean only positively drained success."""
    directory = Path(tempfile.mkdtemp(prefix="surreal-b1-", dir=root))
    proof: dict[str, object] | None = None
    print(f"owned lifecycle fixture: {directory}", flush=True)
    try:
        yield directory
        proof = finish_fixture(directory, evidence)
    except BaseException as error:
        server.write_json(
            directory / "fixture-outcome.json",
            {"outcome": "failed", "reason": type(error).__name__, "retained": True},
        )
        print(f"incomplete lifecycle fixture retained: {directory}", flush=True)
        raise
    else:
        shutil.rmtree(directory)
        if evidence is not None and proof is not None:
            proof["fixture_retired"] = True
            server.write_json(evidence / "passed.json", proof)
            print(
                f"compact positive lifecycle evidence: {evidence / 'passed.json'}",
                flush=True,
            )


def query(state: Path, text: str) -> list[dict[str, object]]:
    config = server.config_for(state)
    credentials = server.read_json(state / "credentials.json")
    token = base64.b64encode(
        f"{credentials['username']}:{credentials['password']}".encode()
    ).decode()
    request = urllib.request.Request(
        f"http://127.0.0.1:{config['port']}/sql",
        data=text.encode(),
        headers={
            "Authorization": f"Basic {token}",
            "Accept": "application/json",
            "surreal-ns": "pse",
            "surreal-db": "canonical",
        },
    )
    with urllib.request.urlopen(request, timeout=30) as response:  # noqa: S310 -- literal authenticated loopback endpoint; config_for validates port
        result = json.load(response)
    if not isinstance(result, list) or any(
        not isinstance(row, dict) or row.get("status") != "OK" for row in result
    ):
        raise server.SupervisorError("Fixture query failed")
    return result


def check(state: Path) -> None:
    server.validate_interpretation(state, server.config_for(state), INTERPRETATION)
    rows = query(state, "SELECT operation, exact_bits FROM supervisor_probe:ack;")
    if rows[0]["result"] != [FIXTURE]:
        raise server.SupervisorError(
            "Acknowledged exact fixture did not survive lifecycle operation"
        )


def managed_worker_check(
    state: Path, config: dict[str, object], directory: Path
) -> None:
    """Lightweight real-scope control; no scientific worker or claim service."""
    observation, release = directory / "worker-cap.json", directory / "worker-release"
    code = (
        "import json,pathlib,sys,time; "
        "group=next(line.split(':',2)[2] for line in pathlib.Path('/proc/self/cgroup').read_text().splitlines() if line.startswith('0:')); "
        "root=pathlib.Path('/sys/fs/cgroup')/group.lstrip('/'); "
        "pathlib.Path(sys.argv[1]).write_text(json.dumps({'memory_max':(root/'memory.max').read_text().strip(),'swap_max':(root/'memory.swap.max').read_text().strip()})); "
        'exec("while not pathlib.Path(sys.argv[2]).exists():\\n time.sleep(0.02)")'
    )
    launcher = subprocess.Popen(
        [
            sys.executable,
            str(server.SCRIPT),
            "worker",
            "--state",
            str(state),
            "--worker-capabilities",
            "",
            "--worker-command",
            sys.executable,
            "-c",
            code,
            str(observation),
            str(release),
        ]
    )
    try:
        deadline = time.monotonic() + 15
        while not observation.exists():
            if launcher.poll() is not None or time.monotonic() >= deadline:
                raise server.SupervisorError(
                    "Managed worker did not enter its capped slot"
                )
            time.sleep(0.02)
        observed = server.read_json(observation)
        allocation = config["resources"]
        if not isinstance(allocation, dict):
            raise server.SupervisorError(
                "Managed worker lacks its recorded resource allocation"
            )
        if observed != {
            "memory_max": str(allocation["native_worker_memory_bytes"]),
            "swap_max": "0",
        }:
            raise server.SupervisorError(
                "Managed worker kernel cap differs from its configured allocation"
            )
        record_ownership(directory, server.worker_unit(state, 0))
        launcher.kill()
        launcher.wait(timeout=10)
        # A dead launcher is not a free slot: the named scope owns the survivor.
        try:
            server.worker(state, [sys.executable, "-c", "raise SystemExit(99)"])
        except server.SupervisorError as error:
            if "occupied" not in str(error):
                raise
        else:
            raise server.SupervisorError("A live orphaned worker slot was reused")
        try:
            server.backup(state, config, directory / "blocked-worker-backup")
        except server.SupervisorError as error:
            if "remain active" not in str(error):
                raise
        else:
            raise server.SupervisorError(
                "Offline backup bypassed the managed worker drain"
            )
        if (directory / "blocked-worker-backup").exists():
            raise server.SupervisorError(
                "A refused offline copy created its destination"
            )
        release.write_text("drain")
        deadline = time.monotonic() + 15
        while True:
            try:
                with server.state_lock(state):
                    server.workers_drained(state, config)
                break
            except server.SupervisorError:
                if time.monotonic() >= deadline:
                    raise server.SupervisorError(
                        "Managed worker failed to drain its scope"
                    ) from None
                time.sleep(0.02)
    finally:
        release.touch()
        server.systemctl("stop", server.worker_unit(state, 0), check=False)
        if launcher.poll() is None:
            launcher.terminate()
            launcher.wait(timeout=10)
    print(
        "managed worker: kernel budget, finite slot, launcher-death fence and offline drain passed",
        flush=True,
    )


def restart_limit_denial(
    unit: str, since: float, deadline: float
) -> dict[str, object] | None:
    """Read the actual unit-specific denial; Result may retain the last signal."""
    result = subprocess.run(
        [
            "journalctl",
            "--user",
            "--unit",
            unit,
            "--output=json",
            "--lines=128",
            "--no-pager",
        ],
        env=server.systemd_environment(),
        capture_output=True,
        text=True,
        timeout=server.remaining(deadline),
        check=False,
    )
    if result.returncode or len(result.stdout.encode()) > server.MESSAGE_BYTES:
        raise server.SupervisorError(
            "Cannot inspect bounded actual restart-denial evidence"
        )
    for line in reversed(result.stdout.splitlines()):
        row = json.loads(line)
        if row.get("MESSAGE") == f"{unit}: Start request repeated too quickly." and int(
            row.get("__REALTIME_TIMESTAMP", "0")
        ) >= int(since * 1_000_000):
            return {
                "unit": unit,
                "message": row["MESSAGE"],
                "realtime_timestamp_us": row["__REALTIME_TIMESTAMP"],
            }
    return None


def restart_exhaustion(state: Path, config: dict[str, object], directory: Path) -> None:
    """Exercise actual finite recovery, exhausted admission, and deliberate stop."""
    unit = server.unit_name(state)
    burst = server.systemctl(
        "show", "--property=StartLimitBurst", "--value", unit
    ).stdout.strip()
    interval = server.systemctl(
        "show", "--property=RestartUSec", "--value", unit
    ).stdout.strip()
    start_window = server.systemctl(
        "show", "--property=StartLimitIntervalUSec", "--value", unit
    ).stdout.strip()
    if burst != "3" or interval != "5s" or start_window not in {"5min", "300s"}:
        raise server.SupervisorError(
            "Actual restart policy differs from the admitted finite contract"
        )
    # Three actual failure/stop/start transitions consume their declared finite
    # lifecycle budgets; this control establishes behavior, not startup speed.
    deadline = time.monotonic() + 300
    started = time.time()
    server.stop(state, config)
    server.reset_failure_window(state)
    server.start(state, config, deadline=deadline)
    record_ownership(directory, unit)
    invocations = {native_operation.unit_observation(unit)["InvocationID"]}
    exhausted = False
    denial: dict[str, object] | None = None
    for _ in range(3):
        previous = native_operation.unit_observation(unit)["InvocationID"]
        server.systemctl("kill", "--kill-whom=all", "--signal=SIGKILL", unit)
        while True:
            server.remaining(deadline)
            observed = native_operation.unit_observation(unit)
            if observed.get("ActiveState") == "failed":
                denial = restart_limit_denial(unit, started, deadline)
                if denial is not None:
                    exhausted = True
                    break
            if (
                observed.get("ActiveState") == "active"
                and observed.get("InvocationID") != previous
                and server.listener_ready(state, config)
            ):
                server.establish_protocol_readiness(state, config, deadline)
                record_ownership(directory, unit)
                invocations.add(observed["InvocationID"])
                check(state)
                break
            time.sleep(0.05)
        if exhausted:
            break
    if not exhausted or len(invocations) != 3:
        raise server.SupervisorError(
            "Actual restart budget did not exhaust after three started generations"
        )
    restart_counter = server.systemctl(
        "show", "--property=NRestarts", "--value", unit
    ).stdout.strip()
    if restart_counter != "3":
        raise server.SupervisorError(
            "Actual restart counter differs from three bounded restart attempts"
        )
    terminal_result = server.systemctl(
        "show", "--property=Result", "--value", unit
    ).stdout.strip()
    terminal = native_operation.unit_observation(unit)
    wait_until = time.monotonic() + 6
    while time.monotonic() < wait_until:
        server.remaining(deadline)
        observed = native_operation.unit_observation(unit)
        if (
            observed.get("ActiveState") != "failed"
            or observed.get("InvocationID") != terminal.get("InvocationID")
            or observed.get("ControlGroup")
        ):
            raise server.SupervisorError(
                "Exhausted service restarted or retained live processes"
            )
        time.sleep(0.05)
    # The explicit administrative recovery begins a new bounded startup window.
    server.reset_failure_window(state)
    server.start(state, config, deadline=deadline)
    record_ownership(directory, unit)
    check(state)
    server.stop(state, config)
    stopped = native_operation.unit_observation(unit)
    wait_until = time.monotonic() + 6
    while time.monotonic() < wait_until:
        server.remaining(deadline)
        observed = native_operation.unit_observation(unit)
        if (
            observed.get("ActiveState") != "inactive"
            or observed.get("InvocationID") != stopped.get("InvocationID")
            or observed.get("ControlGroup")
        ):
            raise server.SupervisorError(
                "Intentional stop triggered automatic recovery"
            )
        time.sleep(0.05)
    server.write_json(
        directory / "restart-exhaustion.json",
        {
            "outcome": "passed",
            "generation": server.recovery_generation(state, config),
            "started_invocations": sorted(invocations),
            "burst": 3,
            "restart_seconds": 5,
            "start_limit_interval_seconds": 300,
            "restart_counter": 3,
            "actual_denial": denial,
            "terminal_result": terminal_result,
            "exhaustion_observed_seconds": 6,
            "intentional_stop_observed_seconds": 6,
            "scope": "small owned storage fixture; no scientific or timing-performance qualification",
        },
    )
    print(
        "bounded restart: three starts, actual start-limit denial, no restart after exhaustion/intentional stop, explicit recovery and exact readback passed",
        flush=True,
    )


def receiver_closures(state: Path, directory: Path) -> None:
    """Actual artifact/context lifetime control; no scientific primary claim."""
    source, producer = (
        directory / "selected-worker",
        directory / "selected-producer.json",
    )
    shutil.copyfile(Path(sys.executable).resolve(), source)
    source.chmod(0o700)
    server.write_json(
        producer, {"fixture_generation": 1, "scope": "toy producer association only"}
    )
    launchers: list[subprocess.Popen[bytes]] = []
    contexts: list[Path] = []
    releases: list[Path] = []
    proofs: list[dict[str, object]] = []
    storage = native_operation.unit_observation(server.unit_name(state))["InvocationID"]
    original = {
        key: os.environ.get(key)
        for key in ("PSE_WORKER_BINARY", "PSE_WORKER_PRODUCER_RECEIPT")
    }
    code = (
        "import hashlib,json,os,pathlib,sys,time; "
        "receipt=pathlib.Path(os.environ['PSE_PRODUCER_RECEIPT']); "
        "pathlib.Path(sys.argv[1]).write_text(json.dumps({'pid':os.getpid(),'producer_receipt_path':str(receipt),'producer_receipt_sha256':hashlib.sha256(receipt.read_bytes()).hexdigest(),'producer_bytes':receipt.read_text()})); "
        'exec("while not pathlib.Path(sys.argv[2]).exists():\\n time.sleep(0.02)")'
    )
    try:
        for index in (1, 2):
            os.environ.update(
                PSE_WORKER_BINARY=str(source), PSE_WORKER_PRODUCER_RECEIPT=str(producer)
            )
            database = f"owned_generation_{index}"
            descriptor = server.register_context(state, database, "functional", source)
            context = server.receiver_context(state, database)
            contexts.append(context)
            selected = server.config_for(context)
            receiver = server.checked_primary(selected)
            environment, _ = server.primary_environment(
                context, server.object_mapping(selected["resources"]), receiver
            )
            # A relocated standalone interpreter needs its original standard library.
            environment["PYTHONHOME"] = sys.base_prefix
            environment["LD_LIBRARY_PATH"] = str(Path(sys.base_prefix) / "lib") + (
                ":" + environment["LD_LIBRARY_PATH"]
                if environment.get("LD_LIBRARY_PATH")
                else ""
            )
            observation, release = (
                directory / f"receiver-{index}.json",
                directory / f"receiver-{index}.release",
            )
            releases.append(release)
            launcher = subprocess.Popen(
                [
                    sys.executable,
                    str(server.SCRIPT),
                    "worker",
                    "--state",
                    str(context),
                    "--worker-capabilities",
                    "",
                    "--worker-command",
                    receiver["worker_executable"],
                    "-c",
                    code,
                    str(observation),
                    str(release),
                ],
                env=environment,
            )
            launchers.append(launcher)
            deadline = time.monotonic() + 15
            while not observation.exists():
                if launcher.poll() is not None or time.monotonic() >= deadline:
                    raise server.SupervisorError(
                        "Toy admitted receiver did not establish its actual artifact observation"
                    )
                time.sleep(0.02)
            marker = server.read_json(observation)
            process = Path(f"/proc/{marker['pid']}")
            actual = process / "exe"
            if (
                str(actual.readlink()) != receiver["worker_executable"]
                or server.file_digest(actual) != receiver["worker_sha256"]
            ):
                raise server.SupervisorError(
                    "Toy receiver actual executable differs from immutable admission"
                )
            producer_path = marker["producer_receipt_path"]
            if not isinstance(producer_path, str):
                raise server.SupervisorError(
                    "Toy receiver producer path is not a string"
                )
            if not server.primary_receipt_ready(
                process,
                marker,
                {"PSE_WORKER_PRODUCER_RECEIPT": producer_path},
            ):
                raise server.SupervisorError(
                    "Toy receiver did not retain its exact admitted producer bytes"
                )
            unit = server.worker_unit(context, 0)
            record_ownership(directory, unit)
            proofs.append(
                {
                    "context": str(context),
                    "generation": descriptor["receiver_generation"],
                    "pid": marker["pid"],
                    "worker_sha256": receiver["worker_sha256"],
                    "producer_sha256": marker["producer_receipt_sha256"],
                    "producer_path": marker["producer_receipt_path"],
                    "unit": unit,
                }
            )
            if index == 1:
                # Mutate only newly owned build-path inputs; admitted copies stay immutable.
                with source.open("ab") as stream:
                    stream.write(b"\0owned-second-generation\0")
                server.write_json(
                    producer,
                    {"fixture_generation": 2, "scope": "toy producer association only"},
                )
                try:
                    server.primary_environment(
                        context, server.object_mapping(selected["resources"]), receiver
                    )
                except server.SupervisorError as error:
                    if "differs" not in str(error):
                        raise
                else:
                    raise server.SupervisorError(
                        "Changed selected executable reused stale admitted authority"
                    )
                if server.file_digest(actual) != receiver["worker_sha256"]:
                    raise server.SupervisorError(
                        "Changing build-path inputs changed the old running receiver"
                    )
        if (
            proofs[0]["generation"] == proofs[1]["generation"]
            or proofs[0]["unit"] == proofs[1]["unit"]
            or proofs[0]["producer_sha256"] == proofs[1]["producer_sha256"]
        ):
            raise server.SupervisorError(
                "Distinct receiver artifacts failed to establish independent generations"
            )
        if native_operation.unit_observation(server.unit_name(state))[
            "InvocationID"
        ] != storage or not all(
            native_operation.unit_observation(str(proof["unit"]))["ActiveState"]
            == "active"
            for proof in proofs
        ):
            raise server.SupervisorError(
                "Receiver generations did not coexist over one unchanged storage invocation"
            )
        releases[0].touch()
        if launchers[0].wait(timeout=15):
            raise server.SupervisorError(
                "First toy receiver failed during independent release"
            )
        server.workers_drained(contexts[0], server.config_for(contexts[0]))
        if (
            native_operation.unit_observation(str(proofs[1]["unit"]))["ActiveState"]
            != "active"
        ):
            raise server.SupervisorError(
                "Draining the first generation interrupted the second"
            )
        releases[1].touch()
        if launchers[1].wait(timeout=15):
            raise server.SupervisorError("Second toy receiver failed during release")
        server.all_contexts_drained(state)
        check(state)
        server.write_json(
            directory / "receiver-closures.json",
            {
                "outcome": "passed",
                "storage_invocation": storage,
                "receivers": proofs,
                "scope": "actual managed toy artifact/producer/context lifetime, selected-byte stale-authority refusal and independent drain; excludes scientific primary startup",
            },
        )
        print(
            "receiver closures: two actual frozen generations coexist, changed build inputs preserve admitted bytes, stale selected authority refused and independent drain passed",
            flush=True,
        )
    finally:
        for key, value in original.items():
            if value is None:
                os.environ.pop(key, None)
            else:
                os.environ[key] = value
        for release in releases:
            release.touch()
        for context in contexts:
            server.systemctl("stop", server.worker_unit(context, 0), check=False)
        for launcher in launchers:
            if launcher.poll() is None:
                launcher.terminate()
                launcher.wait(timeout=15)


def journey(
    *,
    restart_exhaustion_only: bool = False,
    receiver_closures_only: bool = False,
    bounded_b_controls: bool = False,
    evidence: Path | None = None,
) -> None:
    root = (
        Path(os.environ.get("XDG_STATE_HOME", str(Path.home() / ".local/state")))
        / "pse-arrow/fixtures"
    )
    root.mkdir(parents=True, exist_ok=True, mode=0o700)
    with owned_fixture(root, evidence) as directory:
        state, restored = directory / "live", directory / "restored"
        with socket.socket() as probe:
            probe.bind(("127.0.0.1", 0))
            port = probe.getsockname()[1]
        args = server.parser().parse_args(
            [
                "setup",
                "--state",
                str(state),
                "--port",
                str(port),
                "--no-resident",
                "--interpretation",
                INTERPRETATION,
                "--memory-mib",
                "1536",
                "--server-memory-mib",
                "1024",
                "--native-workers",
                "1",
                "--native-worker-memory-mib",
                "512",
            ]
        )
        server.setup(args)
        config = server.config_for(state)
        try:
            server.start(state, config)
            record_ownership(directory, server.unit_name(state))
            query(
                state,
                "DEFINE NAMESPACE IF NOT EXISTS pse; USE NS pse; DEFINE DATABASE IF NOT EXISTS canonical; USE DB canonical; "
                "DEFINE TABLE IF NOT EXISTS canonical_interpretations SCHEMALESS; "
                f"UPSERT canonical_interpretations:current CONTENT {{key:'current',interpretation:{json.dumps(INTERPRETATION)}}}; "
                "DEFINE TABLE IF NOT EXISTS supervisor_probe SCHEMALESS; "
                "UPSERT supervisor_probe:ack CONTENT {operation:'ack-fixed-1',exact_bits:'8000000000000000'};",
            )
            check(state)
            if receiver_closures_only or bounded_b_controls:
                receiver_closures(state, directory)
                if not bounded_b_controls:
                    return
            if restart_exhaustion_only or bounded_b_controls:
                server.qualify_recovery(state)
                config.update(server.config_for(state))
                restart_exhaustion(state, config, directory)
                return
            managed_worker_check(state, config, directory)
            print(f"fixture abrupt stop on {port}", flush=True)
            server.stop(state, config, abrupt=True)
            print(
                f"fixture restart on {port}; unit active={server.active(state)}",
                flush=True,
            )
            server.start(state, config)
            record_ownership(directory, server.unit_name(state))
            check(state)
            server.qualify_recovery(state)
            config.update(server.config_for(state))
            policy = server.systemctl(
                "show", "--property=Restart", "--value", server.unit_name(state)
            ).stdout.strip()
            if policy != "on-failure":
                raise server.SupervisorError(
                    "Positive recovery proof did not materialize bounded restart"
                )
            server.start(state, config)
            original = native_operation.unit_observation(server.unit_name(state))
            server.systemctl(
                "kill", "--kill-whom=all", "--signal=SIGKILL", server.unit_name(state)
            )
            deadline = time.monotonic() + 30
            while True:
                observed = native_operation.unit_observation(server.unit_name(state))
                if (
                    observed.get("InvocationID") != original.get("InvocationID")
                    and observed.get("ActiveState") == "active"
                    and server.listener_ready(state, config)
                ):
                    server.establish_protocol_readiness(state, config, deadline)
                    break
                server.remaining(deadline)
                time.sleep(0.05)
            record_ownership(directory, server.unit_name(state))
            check(state)
            server.backup(state, config, directory / "backup")
            server.restore(directory / "backup", restored, INTERPRETATION)
            restored_config = server.config_for(restored)
            try:
                try:
                    server.start(restored, restored_config)
                except server.SupervisorError:
                    pass
                else:
                    raise server.SupervisorError(
                        "Restore unexpectedly admitted normal writes before validation"
                    )
                server.validate(
                    restored,
                    restored_config,
                    INTERPRETATION,
                    [
                        sys.executable,
                        "-m",
                        "scripts.tests.surreal_fixture_check",
                        "--validate-only",
                    ],
                )
                server.start(restored, restored_config)
                record_ownership(directory, server.unit_name(restored))
                check(restored)
            finally:
                server.stop(restored, restored_config)
        finally:
            server.stop(state, config)
        print(
            "released server lifecycle: authenticated native-WS readiness, fixed acknowledgment/unknown-outcome readback, SIGKILL/reopen, bounded automatic restart, offline backup/restore and validation gate passed"
        )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--validate-only", action="store_true")
    parser.add_argument("--restart-exhaustion-only", action="store_true")
    parser.add_argument("--receiver-closures-only", action="store_true")
    parser.add_argument("--bounded-b-controls", action="store_true")
    parser.add_argument(
        "--evidence",
        type=Path,
        help="New private destination for compact positive controls after actual drain",
    )
    args = parser.parse_args()
    if (
        sum(
            (
                args.validate_only,
                args.restart_exhaustion_only,
                args.receiver_closures_only,
                args.bounded_b_controls,
            )
        )
        > 1
    ):
        parser.error("select one fixture control")
    if args.validate_only:
        if args.evidence is not None:
            parser.error("validate-only does not create owned lifecycle evidence")
        check(Path(os.environ["PSE_SURREAL_STATE"]))
    else:
        journey(
            restart_exhaustion_only=args.restart_exhaustion_only,
            receiver_closures_only=args.receiver_closures_only,
            bounded_b_controls=args.bounded_b_controls,
            evidence=args.evidence,
        )


if __name__ == "__main__":
    main()
