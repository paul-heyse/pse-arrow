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
                    server.SUBSTRATE_INTERPRETATION,
                    *profile_args,
                ]
            )
        )
        config = server.config_for(state)
        allocation = config["resources"]
        if not isinstance(allocation, dict):
            raise server.SupervisorError("Invalid recovery fixture resource allocation")
        original_allocation = dict(allocation)
        original_server_mib = (
            server.integer(allocation["server_memory_bytes"]) // server.MIB
        )
        lower_server_mib = max(512, original_server_mib // 2)
        if lower_server_mib >= original_server_mib:
            raise server.SupervisorError(
                "Recovery reconfiguration requires an original server cap above 512 MiB"
            )
        identity = {
            key: value
            for key, value in config.items()
            if key not in {"resources", "admission", "accepting_writes"}
        }
        credentials = (state / "credentials.json").read_bytes()
        command = [str(binary.resolve())]
        try:
            server.start(state, config)
            subprocess.run([*command, str(state), "--seed"], check=True, timeout=120)
            with server.state_lock(state):
                server.stop(state, config, abrupt=True)
            for server_mib in (lower_server_mib, original_server_mib):
                with server.state_lock(state):
                    config = server.config_for(state)
                    server.reconfigure(
                        state,
                        config,
                        server.parser().parse_args(
                            [
                                "reconfigure",
                                "--state",
                                str(state),
                                "--server-memory-mib",
                                str(server_mib),
                            ]
                        ),
                    )
                    expected = server.resources(
                        server.integer(original_allocation["total_memory_bytes"]),
                        server_mib * server.MIB,
                        server.integer(original_allocation["native_workers"]),
                        server.integer(
                            original_allocation["native_worker_memory_bytes"]
                        ),
                    )
                    if (
                        config["resources"] != expected
                        or {
                            key: value
                            for key, value in config.items()
                            if key not in {"resources", "admission", "accepting_writes"}
                        }
                        != identity
                        or (state / "credentials.json").read_bytes() != credentials
                    ):
                        raise server.SupervisorError(
                            "Resource reconfiguration changed recovery identity, credentials or joint profile"
                        )
                    if config["accepting_writes"] or config["admission"] != "quiesced":
                        raise server.SupervisorError(
                            "Reconfiguration reopened recovery admission"
                        )
                    server.start(state, config)
                    group = server.systemctl(
                        "show",
                        "--property=ControlGroup",
                        "--value",
                        server.unit_name(state),
                    ).stdout.strip()
                    if not server.group_populated(group):
                        raise server.SupervisorError(
                            "Started recovery server has no populated owned cgroup"
                        )
                    kernel_cap = (
                        (Path("/sys/fs/cgroup") / group.lstrip("/") / "memory.max")
                        .read_text()
                        .strip()
                    )
                    if kernel_cap != str(expected["server_memory_bytes"]):
                        raise server.SupervisorError(
                            "Recovery server kernel MemoryMax differs from its reconfigured allocation"
                        )
                    config["accepting_writes"] = False
                    config["admission"] = "quiesced"
                    server.write_json(state / "config.json", config)
                subprocess.run([*command, str(state)], check=True, timeout=120)
                if server_mib == lower_server_mib:
                    with server.state_lock(state):
                        server.stop(state, config)
            if config["resources"] != original_allocation:
                raise server.SupervisorError(
                    "Recovery backup did not regain its original selected profile"
                )
            with server.state_lock(state):
                server.backup(state, config, directory / "backup")
            server.restore(
                directory / "backup", restored, server.SUBSTRATE_INTERPRETATION
            )
            restored_config = server.config_for(restored)
            try:
                server.validate(
                    restored,
                    restored_config,
                    server.SUBSTRATE_INTERPRETATION,
                    [*command, str(restored)],
                )
                server.start(restored, restored_config)
            finally:
                server.stop(restored, restored_config)
        finally:
            server.stop(state, config)
    print(
        "native canonical SIGKILL/reopen, offline resource reconfigure/kernel caps/original-profile recovery, protected historical source/product, exact bits, large staging and gated offline restore passed"
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
