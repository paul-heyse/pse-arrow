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


def profile_arguments(profile_state: Path | None) -> list[str]:
    if profile_state is None:
        return [
            "--memory-mib",
            "1536",
            "--server-memory-mib",
            "1024",
            "--native-workers",
            "1",
            "--native-worker-memory-mib",
            "512",
        ]
    selected = server.config_for(profile_state.resolve())
    allocation = selected["resources"]
    release = selected["server"]
    if not isinstance(allocation, dict) or not isinstance(release, dict):
        raise server.SupervisorError("Invalid selected recovery profile")
    version = ["--version", f"v{release['version']}"]
    if allocation.get("execution") is not None:
        if allocation != server.reference_resources():
            raise server.SupervisorError(
                "Recovery requires the original exact reference allocation"
            )
        receiver = server.checked_primary(selected)
        return [
            "--execution-profile",
            "plan28-reference",
            "--worker-executable",
            receiver["worker_executable"],
            *version,
        ]
    return [
        "--memory-mib",
        str(server.integer(allocation["total_memory_bytes"]) // server.MIB),
        "--server-memory-mib",
        str(server.integer(allocation["server_memory_bytes"]) // server.MIB),
        "--native-workers",
        str(server.integer(allocation["native_workers"])),
        "--native-worker-memory-mib",
        str(server.integer(allocation["native_worker_memory_bytes"]) // server.MIB),
        *version,
    ]


def recovery_identity(config: dict[str, object]) -> dict[str, object]:
    return {
        key: value
        for key, value in config.items()
        if key not in {"resources", "admission", "accepting_writes"}
    }


def refuse_reference_reduction(state: Path, config: dict, server_mib: int) -> None:
    """Exact reference capacity is immutable; this is refusal, not reconfiguration."""
    prior = server.config_for(state)
    saved = (state / "config.json").read_bytes()
    credentials = (state / "credentials.json").read_bytes()
    try:
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
    except server.SupervisorError as error:
        if "Reference execution requires" not in str(error):
            raise
    else:
        raise server.SupervisorError(
            "Exact reference server-cap reduction was not refused"
        )
    if (
        config != prior
        or server.config_for(state) != prior
        or (state / "config.json").read_bytes() != saved
        or (state / "credentials.json").read_bytes() != credentials
    ):
        raise server.SupervisorError(
            "Refused reference reduction mutated recovery state or credentials"
        )


def verify_server_placement(
    state: Path,
    allocation: dict,
    group: str,
    cgroup_root: Path = Path("/sys/fs/cgroup"),
) -> None:
    """Read actual service and selected ancestor controls, including effective limits."""
    actual = cgroup_root / group.lstrip("/")
    if (actual / "memory.max").read_text().strip() != str(
        allocation["server_memory_bytes"]
    ):
        raise server.SupervisorError(
            "Recovery server kernel MemoryMax differs from its reconfigured allocation"
        )
    execution = allocation.get("execution")
    if execution is None:
        return
    if not isinstance(execution, dict):
        raise server.SupervisorError("Invalid recovery execution allocation")
    selected = server.group_for_slice(server.execution_slice(state))
    if actual == selected or not actual.is_relative_to(selected):
        raise server.SupervisorError(
            "Recovery server is outside its selected execution ancestor"
        )
    parent_memory = (selected / "memory.max").read_text().strip()
    quota, period = (selected / "cpu.max").read_text().split()
    width = server.integer(execution["cpu_threads"])
    if (
        parent_memory != str(allocation["total_memory_bytes"])
        or quota == "max"
        or int(period) <= 0
        or int(quota) != width * int(period)
        or server.effective_limits(selected)
        != (allocation["total_memory_bytes"], width)
        or server.effective_limits(actual) != (allocation["server_memory_bytes"], width)
    ):
        raise server.SupervisorError(
            "Recovery selected ancestor memory/CPU enforcement differs from its exact profile"
        )


def journey(binary: Path, profile_state: Path | None = None) -> None:
    profile_args = profile_arguments(profile_state)
    selected = None
    if profile_state is not None:
        selected = server.config_for(profile_state.resolve())
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
        reference = allocation.get("execution") is not None
        if selected is not None and (
            allocation != selected["resources"]
            or config.get("primary_receiver") != selected.get("primary_receiver")
        ):
            raise server.SupervisorError(
                "Recovery fixture did not copy the selected allocation and receiver"
            )
        original_server_mib = (
            server.integer(allocation["server_memory_bytes"]) // server.MIB
        )
        lower_server_mib = max(512, original_server_mib // 2)
        if lower_server_mib >= original_server_mib:
            raise server.SupervisorError(
                "Recovery reconfiguration requires an original server cap above 512 MiB"
            )
        identity = recovery_identity(config)
        credentials = (state / "credentials.json").read_bytes()
        command = [str(binary.resolve())]
        try:
            server.start(state, config)
            if reference:
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
                verify_server_placement(state, allocation, group)
            subprocess.run([*command, str(state), "--seed"], check=True, timeout=120)
            with server.state_lock(state):
                server.stop(state, config, abrupt=True)
                if reference:
                    refuse_reference_reduction(state, config, lower_server_mib)
            caps = (
                (original_server_mib,)
                if reference
                else (lower_server_mib, original_server_mib)
            )
            for server_mib in caps:
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
                        original_allocation.get("execution"),
                    )
                    if (
                        config["resources"] != expected
                        or recovery_identity(config) != identity
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
                    verify_server_placement(state, expected, group)
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
            restored_allocation = restored_config["resources"]
            if not isinstance(restored_allocation, dict):
                raise server.SupervisorError("Invalid restored recovery allocation")
            if restored_config[
                "resources"
            ] != original_allocation or restored_config.get(
                "primary_receiver"
            ) != config.get("primary_receiver"):
                raise server.SupervisorError(
                    "Offline restore changed the selected allocation or receiver"
                )
            try:
                server.validate(
                    restored,
                    restored_config,
                    server.SUBSTRATE_INTERPRETATION,
                    [*command, str(restored)],
                )
                server.start(restored, restored_config)
                if reference:
                    group = server.systemctl(
                        "show",
                        "--property=ControlGroup",
                        "--value",
                        server.unit_name(restored),
                    ).stdout.strip()
                    if not server.group_populated(group):
                        raise server.SupervisorError(
                            "Restored recovery server has no populated owned cgroup"
                        )
                    verify_server_placement(restored, restored_allocation, group)
            finally:
                server.stop(restored, restored_config)
        finally:
            server.stop(state, config)
    print(
        "native canonical SIGKILL/reopen, "
        + (
            "exact reference cap-reduction refusal with unchanged state and selected ancestor enforcement, "
            if reference
            else "positive offline lower/original server-cap reconfiguration and kernel caps, "
        )
        + "protected historical source/product, exact bits, large staging and gated offline restore at the original profile passed"
    )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument(
        "--profile-state",
        type=Path,
        help="Copy released server, allocation and configured reference receiver. Exact reference tests cap-reduction refusal; numeric profiles exercise positive lower/original-cap reconfiguration.",
    )
    args = parser.parse_args()
    journey(args.binary, args.profile_state)


if __name__ == "__main__":
    main()
