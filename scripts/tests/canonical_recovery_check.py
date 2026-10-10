# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Released-server recovery with generated native schema and protected source/products."""

from __future__ import annotations

import argparse
import os
import shutil
import socket
import stat
import subprocess
import sys
import tempfile
from contextlib import nullcontext
from pathlib import Path
from typing import TYPE_CHECKING

from scripts import surreal_server as server
from scripts.tests import canonical_recovery_science as science

if TYPE_CHECKING:
    from collections.abc import Mapping


def science_command(state: Path, inputs: Path, receipt: Path, phase: str) -> list[str]:
    return [
        sys.executable,
        "-m",
        "scripts.tests.canonical_recovery_science",
        "--state",
        str(state),
        "--inputs",
        str(inputs),
        "--receipt",
        str(receipt),
        "--phase",
        phase,
    ]


def refuse_old_credentials(state: Path, credentials: dict[str, object]) -> None:
    config = server.config_for(state)
    with server.lifecycle_reservation(state):
        server.private_offline_state(state, config)
        current = server.read_json(state / "credentials.json")
        if current == credentials:
            raise server.SupervisorError("Restore retained original root credentials")
        server.maintenance_query(state, config, current, "LET $value=true;")
        try:
            server.maintenance_query(state, config, credentials, "LET $value=true;")
        except server.SupervisorError:
            server.maintenance_query(state, config, current, "LET $value=true;")
            return
    raise server.SupervisorError(
        "Original root credentials admitted on restored backend"
    )


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
        receiver = reference_recovery_receiver(selected)
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


def reference_recovery_receiver(config: dict[str, object]) -> dict[str, str]:
    """Recovery selects the original reference allocation and frozen receiver."""
    if config.get("resources") != server.reference_resources():
        raise server.SupervisorError(
            "Recovery requires the original exact reference allocation"
        )
    return server.checked_primary(config)


def receiver_equivalence(
    original: Mapping[str, object],
    restored: Mapping[str, object],
    original_root: Path,
    restored_root: Path,
    *,
    primary: bool,
) -> None:
    """Compare immutable content and its exact installation association separately."""
    fields = {"supervisor_executable", "supervisor_script", "supervisor_sha256"}
    if primary:
        fields |= {"worker_executable", "worker_sha256"}
    if set(original) != fields or set(restored) != fields:
        raise server.SupervisorError("Recovery receiver has unsupported fields")
    # The interpreter is an external qualification, never a relocated artifact.
    if original["supervisor_executable"] != restored["supervisor_executable"]:
        raise server.SupervisorError("Recovery external interpreter changed")
    interpreter = Path(str(restored["supervisor_executable"]))
    if (
        not interpreter.is_absolute()
        or not interpreter.is_file()
        or not os.access(interpreter, os.X_OK)
    ):
        raise server.SupervisorError("Recovery external interpreter is unavailable")
    for root, receiver in ((original_root, original), (restored_root, restored)):
        generation = Path(str(receiver["supervisor_script"])).parents[1]
        if generation.parent != root / ".generations":
            raise server.SupervisorError(
                "Recovery receiver is outside its exact installation"
            )
        server.verify_generation(generation)
        if Path(
            str(receiver["supervisor_script"])
        ) != generation / "scripts/surreal_server.py" or (
            primary
            and Path(str(receiver["worker_executable"]))
            != generation / "bin/pse-worker"
        ):
            raise server.SupervisorError(
                "Recovery receiver does not name its declared artifacts"
            )
        manifest = server.read_json(generation / "generation.json")
        for name in ["generation.json", *server.object_mapping(manifest["files"])]:
            artifact = generation / name
            mode = 0o700 if name in {"scripts/sccache", "bin/pse-worker"} else 0o600
            if (
                stat.S_IMODE(artifact.stat().st_mode) != mode
                or artifact.stat().st_uid != os.getuid()
            ):
                raise server.SupervisorError("Recovery generation permissions changed")
            for parent in (artifact.parent, *artifact.parent.parents):
                if parent == root:
                    break
                if parent.stat().st_uid != os.getuid():
                    raise server.SupervisorError(
                        "Recovery generation directory permissions changed"
                    )
    mapped = server.reroot_restored(dict(original), str(original_root), restored_root)
    if mapped != restored:
        raise server.SupervisorError(
            "Recovery receiver content or installation association changed"
        )
    for key in (
        ("supervisor_script", "worker_executable")
        if primary
        else ("supervisor_script",)
    ):
        before, after = Path(str(original[key])), Path(str(restored[key]))
        digest_key = (
            "worker_sha256" if key == "worker_executable" else "supervisor_sha256"
        )
        if (
            server.file_digest(before) != original[digest_key]
            or server.file_digest(after) != restored[digest_key]
        ):
            raise server.SupervisorError("Recovery receiver bytes changed")
        if stat.S_IMODE(before.stat().st_mode) != stat.S_IMODE(after.stat().st_mode):
            raise server.SupervisorError("Recovery receiver permissions changed")
    before = Path(str(original["supervisor_script"])).parents[1]
    after = Path(str(restored["supervisor_script"])).parents[1]
    manifest = server.read_json(before / "generation.json")
    if server.read_json(after / "generation.json") != manifest:
        raise server.SupervisorError("Recovery generation manifest changed")
    for name in ["generation.json", *server.object_mapping(manifest["files"])]:
        source, target = before / name, after / name
        if stat.S_IMODE(source.stat().st_mode) != stat.S_IMODE(target.stat().st_mode):
            raise server.SupervisorError("Recovery generation permissions changed")
        for parent in (source.parent, *source.parent.parents):
            if parent == original_root:
                break
            mapped_parent = Path(
                str(
                    server.reroot_restored(
                        str(parent), str(original_root), restored_root
                    )
                )
            )
            original_mode = stat.S_IMODE(parent.stat().st_mode)
            restored_mode = stat.S_IMODE(mapped_parent.stat().st_mode)
            # Restore may remove group/other access while retaining owner access.
            if (
                original_mode & 0o700 != 0o700
                or restored_mode & 0o700 != 0o700
                or original_mode & 0o7000 != restored_mode & 0o7000
                or restored_mode & 0o077 & ~original_mode
            ):
                raise server.SupervisorError(
                    "Recovery generation directory permissions changed"
                )


def copy_selected_receiver(
    config: dict[str, object], source: Path, state: Path
) -> dict[str, object]:
    """Copy only the admitted closure, retaining its declared bytes and directory modes."""
    receiver = server.checked_primary(config)
    receiver_equivalence(receiver, receiver, source, source, primary=True)
    generation = Path(receiver["supervisor_script"]).parents[1]
    manifest = server.read_json(generation / "generation.json")
    destination = state / ".generations" / generation.name
    directories = {generation}
    for name in ["generation.json", *server.object_mapping(manifest["files"])]:
        original = generation / name
        target = destination / name
        target.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
        shutil.copy2(original, target)
        directories.update(
            parent for parent in original.parents if parent.is_relative_to(generation)
        )
    for original in directories:
        shutil.copystat(original, destination / original.relative_to(generation))
    mapped = server.object_mapping(server.reroot_restored(receiver, str(source), state))
    receiver_equivalence(receiver, mapped, source, state, primary=True)
    return mapped


def recovery_equivalence(
    original: dict[str, object],
    restored: dict[str, object],
    original_root: Path,
    restored_root: Path,
    *,
    fresh: bool = False,
    expected_resources: dict[str, object] | None = None,
) -> None:
    immutable = {
        "owner",
        "profile_version",
        "port",
        "endpoint",
        "schema_interpretation",
        "interpretation",
        "max_message_bytes",
        "websocket_max_message_bytes",
        "log_max_bytes",
        "log_backups",
        "service_class",
        "resident",
    }
    installation = {
        "credentials_file",
        "server",
        "service_supervisor",
        "primary_receiver",
    }
    incarnation = {
        "instance_id",
        "namespace",
        "database",
        "unit_materialized",
        "restart_qualified",
        "admission",
        "accepting_writes",
        "parked",
    }
    if (set(original) | set(restored)) - (
        immutable | installation | incarnation | {"resources"}
    ):
        raise server.SupervisorError(
            "Recovery profile has unclassified consequential fields"
        )
    if any(original.get(key) != restored.get(key) for key in immutable):
        raise server.SupervisorError("Recovery immutable profile changed")
    if restored["resources"] != (
        original["resources"] if expected_resources is None else expected_resources
    ):
        raise server.SupervisorError("Recovery exact resource profile changed")
    for key in ("credentials_file", "server"):
        if (
            server.reroot_restored(original[key], str(original_root), restored_root)
            != restored[key]
        ):
            raise server.SupervisorError("Recovery installation association changed")
    for key, primary in (("service_supervisor", False), ("primary_receiver", True)):
        if original.get(key) is None and restored.get(key) is None:
            continue
        receiver_equivalence(
            server.object_mapping(original[key]),
            server.object_mapping(restored[key]),
            original_root,
            restored_root,
            primary=primary,
        )
    for key in ("instance_id", "namespace", "database"):
        if fresh == (original[key] == restored[key]):
            raise server.SupervisorError(
                "Recovery current incarnation was not re-established"
            )
    # Closed restore maintenance may materialize this fresh incarnation's unit;
    # that file does not establish restart qualification or scientific admission.
    if fresh and (
        restored.get("restart_qualified")
        or restored["accepting_writes"]
        or restored["admission"] != "validation_required"
    ):
        raise server.SupervisorError("Recovery copied current readiness or admission")


def refuse_reduced_recovery_profile(state: Path, config: dict, server_mib: int) -> None:
    """Reject budget-valid reduction for recovery without changing admitted state."""
    prior = server.config_for(state)
    saved = (state / "config.json").read_bytes()
    credentials = (state / "credentials.json").read_bytes()
    allocation = config["resources"]
    candidate = dict(config)
    candidate["resources"] = server.resources(
        server.integer(allocation["total_memory_bytes"]),
        server_mib * server.MIB,
        server.integer(allocation["native_workers"]),
        server.integer(allocation["native_worker_memory_bytes"]),
        allocation.get("execution"),
    )
    try:
        reference_recovery_receiver(candidate)
    except server.SupervisorError as error:
        if "Recovery requires the original exact reference allocation" not in str(
            error
        ):
            raise
    else:
        raise server.SupervisorError(
            "Reduced reference recovery profile was not refused"
        )
    if (
        config != prior
        or server.config_for(state) != prior
        or (state / "config.json").read_bytes() != saved
        or (state / "credentials.json").read_bytes() != credentials
    ):
        raise server.SupervisorError(
            "Refused reduced profile mutated recovery state or credentials"
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


def journey(
    binary: Path,
    profile_state: Path | None = None,
    *,
    scientific: bool = False,
    output_directory: Path | None = None,
) -> None:
    profile_args = profile_arguments(profile_state)
    selected = None
    if profile_state is not None:
        selected = server.config_for(profile_state.resolve())
    if scientific and (
        selected is None or selected.get("resources") != server.reference_resources()
    ):
        raise server.SupervisorError(
            "Scientific recovery requires an explicit original plan28-reference profile"
        )
    root = (
        Path(os.environ.get("XDG_STATE_HOME", str(Path.home() / ".local/state")))
        / "pse-arrow/fixtures"
    )
    root.mkdir(parents=True, exist_ok=True, mode=0o700)
    if output_directory is not None:
        output_directory = output_directory.resolve()
        output_directory.mkdir(parents=True, mode=0o700, exist_ok=False)
    retained = (
        nullcontext(str(output_directory))
        if output_directory is not None
        else tempfile.TemporaryDirectory(prefix="canonical-recovery-", dir=root)
    )
    with retained as scratch:
        directory = Path(scratch)
        state, restored = directory / "live", directory / "restored"
        inputs, receipt = directory / "authored", directory / "science.json"
        input_hashes = science.prepare_inputs(inputs) if scientific else None
        intent = directory / "creation-intent.json"
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
        if selected is not None:
            if allocation != selected["resources"]:
                raise server.SupervisorError(
                    "Recovery fixture changed the selected allocation"
                )
            if selected.get("primary_receiver") is not None:
                if profile_state is None:
                    raise server.SupervisorError(
                        "Selected receiver has no original installation"
                    )
                config["primary_receiver"] = copy_selected_receiver(
                    selected, profile_state.resolve(), state
                )
                server.write_json(state / "config.json", config)
        original_server_mib = (
            server.integer(allocation["server_memory_bytes"]) // server.MIB
        )
        lower_server_mib = max(512, original_server_mib // 2)
        if lower_server_mib >= original_server_mib:
            raise server.SupervisorError(
                "Recovery reconfiguration requires an original server cap above 512 MiB"
            )
        identity = dict(config)
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
            subprocess.run(
                [*command, str(state), "--seed", "--creation-intent", str(intent)],
                check=True,
                timeout=120,
            )
            if scientific:
                subprocess.run(
                    science_command(state, inputs, receipt, "seed"), check=True
                )
            with server.lifecycle_reservation(state):
                server.stop(state, config, abrupt=True)
                if reference:
                    refuse_reduced_recovery_profile(state, config, lower_server_mib)
            caps = (
                (original_server_mib,)
                if reference
                else (lower_server_mib, original_server_mib)
            )
            for server_mib in caps:
                with server.lifecycle_reservation(state):
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
                    recovery_equivalence(
                        identity, config, state, state, expected_resources=expected
                    )
                    if (state / "credentials.json").read_bytes() != credentials:
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
                if scientific:
                    subprocess.run(
                        science_command(state, inputs, receipt, "reopen"), check=True
                    )
                with server.state_lock(state):
                    config["accepting_writes"] = False
                    config["admission"] = "quiesced"
                    server.write_json(state / "config.json", config)
                subprocess.run([*command, str(state)], check=True, timeout=120)
                if server_mib == lower_server_mib:
                    with server.lifecycle_reservation(state):
                        server.stop(state, config)
            if config["resources"] != original_allocation:
                raise server.SupervisorError(
                    "Recovery backup did not regain its original selected profile"
                )
            with server.lifecycle_reservation(state):
                server.backup(state, config, directory / "backup")
            server.restore(
                directory / "backup", restored, server.SUBSTRATE_INTERPRETATION
            )
            restored_config = server.config_for(restored)
            restored_allocation = restored_config["resources"]
            if not isinstance(restored_allocation, dict):
                raise server.SupervisorError("Invalid restored recovery allocation")
            recovery_equivalence(config, restored_config, state, restored, fresh=True)
            refuse_old_credentials(
                restored, server.read_json(state / "credentials.json")
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
                subprocess.run(
                    [
                        *command,
                        str(restored),
                        "--creation-intent",
                        str(intent),
                        "--check-creation",
                    ],
                    check=True,
                    timeout=120,
                )
                if scientific:
                    subprocess.run(
                        science_command(restored, inputs, receipt, "restored"),
                        check=True,
                    )
                    server.stop(restored, restored_config)
                    preserved = directory / "preserved"
                    initializer = science_command(
                        restored, preserved / "inputs", receipt, "rebuild"
                    )
                    server.rebuild(restored, inputs, preserved, initializer)
                    if not (
                        server.input_inventory(inputs)
                        == input_hashes
                        == server.input_inventory(preserved / "inputs")
                    ):
                        raise server.SupervisorError(
                            "Scientific authored/physical inputs changed during rebuild"
                        )
                    restored_config = server.config_for(restored)
                    if restored_config["resources"] != original_allocation:
                        raise server.SupervisorError(
                            "Scientific rebuild changed the reference allocation"
                        )
                    server.start(restored, restored_config)
                    subprocess.run(
                        science_command(
                            restored,
                            preserved / "inputs",
                            receipt.with_suffix(".rebuilt.json"),
                            "verify-rebuild",
                        ),
                        check=True,
                    )
            finally:
                server.stop(restored, restored_config)
        finally:
            server.stop(state, config)
    print(
        "native canonical SIGKILL/reopen, "
        + (
            "reduced reference recovery-profile refusal with unchanged state and selected ancestor enforcement, "
            if reference
            else "positive offline lower/original server-cap reconfiguration and kernel caps, "
        )
        + "protected historical source/product, exact bits, large staging, stale credentials/creation-intent refusal and fresh creation after gated offline restore at the original profile passed"
        + (
            "; local public-API authored/physical reconstruction, native Ipopt solve/checks and exact retained scientific reopen passed"
            if scientific
            else ""
        )
    )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument(
        "--directory",
        type=Path,
        help="Retain explicit inputs, backup, source/result identity receipts and stopped disposable profiles in a fresh directory.",
    )
    parser.add_argument(
        "--scientific",
        action="store_true",
        help="Also rebuild/import/reopen explicit authored and physical documents and run native Ipopt with original scientific checks; requires the original reference profile and linked Python.",
    )
    parser.add_argument(
        "--profile-state",
        type=Path,
        help="Copy released server, allocation and configured reference receiver. Exact reference tests reduced-profile recovery refusal; numeric profiles exercise positive lower/original-cap reconfiguration.",
    )
    args = parser.parse_args()
    journey(
        args.binary,
        args.profile_state,
        scientific=args.scientific,
        output_directory=args.directory,
    )


if __name__ == "__main__":
    main()
