# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Disposable released-server lifecycle check; no existing application state is used.

Run `python -m scripts.tests.surreal_fixture_check` for the journey or add
`--validate-only` to check the state passed in PSE_SURREAL_STATE during restore.
This HTTP control exercises the server lifecycle. The Rust SDK owns gRPC checks.
"""

from __future__ import annotations

import argparse
import base64
import json
import os
import socket
import subprocess
import sys
import tempfile
import time
import urllib.request
from pathlib import Path

from scripts import surreal_server as server

INTERPRETATION = server.SUBSTRATE_INTERPRETATION
FIXTURE = {"operation": "ack-fixed-1", "exact_bits": "8000000000000000"}


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
    with urllib.request.urlopen(request, timeout=30) as response:  # noqa: S310 -- Literal authenticated loopback endpoint; config_for validates the recorded port.
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


def journey() -> None:
    root = (
        Path(os.environ.get("XDG_STATE_HOME", str(Path.home() / ".local/state")))
        / "pse-arrow/fixtures"
    )
    root.mkdir(parents=True, exist_ok=True, mode=0o700)
    with tempfile.TemporaryDirectory(prefix="surreal-a1-", dir=root) as scratch:
        directory = Path(scratch)
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
            query(
                state,
                "DEFINE NAMESPACE IF NOT EXISTS pse; USE NS pse; DEFINE DATABASE IF NOT EXISTS canonical; USE DB canonical; "
                "DEFINE TABLE IF NOT EXISTS canonical_interpretations SCHEMALESS; "
                "UPSERT canonical_interpretations:current CONTENT {key:'current',interpretation:'pse.substrate.v1'}; "
                "DEFINE TABLE IF NOT EXISTS supervisor_probe SCHEMALESS; "
                "UPSERT supervisor_probe:ack CONTENT {operation:'ack-fixed-1',exact_bits:'8000000000000000'};",
            )
            check(state)
            managed_worker_check(state, config, directory)
            print(f"fixture abrupt stop on {port}", flush=True)
            server.stop(state, config, abrupt=True)
            print(
                f"fixture restart on {port}; unit active={server.active(state)}",
                flush=True,
            )
            server.start(state, config)
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
                check(restored)
            finally:
                server.stop(restored, restored_config)
        finally:
            server.stop(state, config)
        print(
            "released server lifecycle: authenticated acknowledgement, SIGKILL/reopen, offline backup/restore and validation gate passed"
        )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--validate-only", action="store_true")
    args = parser.parse_args()
    if args.validate_only:
        check(Path(os.environ["PSE_SURREAL_STATE"]))
    else:
        journey()


if __name__ == "__main__":
    main()
