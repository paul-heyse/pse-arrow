# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Released-server recovery with generated native schema and protected source/products."""

from __future__ import annotations

import argparse
import os
import socket
import subprocess
import tempfile
from pathlib import Path

from scripts import surreal_server as server


def journey(binary: Path, profile_state: Path | None = None) -> None:
    profile_args = [
        "--memory-mib",
        "1536",
        "--server-memory-mib",
        "1024",
        "--native-workers",
        "1",
        "--native-worker-memory-mib",
        "512",
    ]
    if profile_state is not None:
        selected = server.config_for(profile_state.resolve())
        allocation = selected["resources"]
        release = selected["server"]
        if not isinstance(allocation, dict) or not isinstance(release, dict):
            raise server.SupervisorError("Invalid selected recovery profile")
        profile_args = [
            "--memory-mib",
            str(server.integer(allocation["total_memory_bytes"]) // server.MIB),
            "--server-memory-mib",
            str(server.integer(allocation["server_memory_bytes"]) // server.MIB),
            "--native-workers",
            str(server.integer(allocation["native_workers"])),
            "--native-worker-memory-mib",
            str(server.integer(allocation["native_worker_memory_bytes"]) // server.MIB),
            "--version",
            f"v{release['version']}",
        ]
    root = (
        Path(os.environ.get("XDG_STATE_HOME", str(Path.home() / ".local/state")))
        / "pse-arrow/fixtures"
    )
    root.mkdir(parents=True, exist_ok=True, mode=0o700)
    with tempfile.TemporaryDirectory(prefix="canonical-recovery-", dir=root) as scratch:
        directory = Path(scratch)
        state, restored = directory / "live", directory / "restored"
        with socket.socket() as probe:
            probe.bind(("127.0.0.1", 0))
            port = probe.getsockname()[1]
        server.setup(
            server.parser().parse_args(
                [
                    "setup",
                    "--state",
                    str(state),
                    "--port",
                    str(port),
                    "--interpretation",
                    "pse.substrate.v1",
                    *profile_args,
                ]
            )
        )
        config = server.config_for(state)
        command = [str(binary.resolve())]
        try:
            server.start(state, config)
            subprocess.run([*command, str(state), "--seed"], check=True, timeout=120)
            server.stop(state, config, abrupt=True)
            server.start(state, config)
            config["accepting_writes"] = False
            config["admission"] = "quiesced"
            server.write_json(state / "config.json", config)
            subprocess.run([*command, str(state)], check=True, timeout=120)
            server.backup(state, config, directory / "backup")
            server.restore(directory / "backup", restored, "pse.substrate.v1")
            restored_config = server.config_for(restored)
            try:
                server.validate(
                    restored,
                    restored_config,
                    "pse.substrate.v1",
                    [*command, str(restored)],
                )
                server.start(restored, restored_config)
            finally:
                server.stop(restored, restored_config)
        finally:
            server.stop(state, config)
    print(
        "native canonical SIGKILL/reopen, protected historical source/product, exact bits, large staging and gated offline restore passed"
    )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument(
        "--profile-state",
        type=Path,
        help="Copy the selected deployment's released-server and resource profile into the isolated recovery fixture",
    )
    args = parser.parse_args()
    journey(args.binary, args.profile_state)


if __name__ == "__main__":
    main()
