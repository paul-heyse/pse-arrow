# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit, preservation-only preparation cutover under the real supervisor.

The native bridge owns exact codecs and derivation. This process retains the
supervisor reservation and authenticates every bridge RPC; a manifest is evidence,
never an offline permit. Admission stays closed, including after completion.
"""

from __future__ import annotations

import argparse
import json
import os
import selectors
import stat
import subprocess
import sys
import tempfile
import uuid
from pathlib import Path

if __package__ in {None, ""}:
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from scripts import surreal_server as server
from scripts import test_resources


def frozen_json(path: Path, value: dict[str, object]) -> None:
    """Compare an existing immutable file or publish it without replacement."""
    raw = (json.dumps(value, sort_keys=True, indent=2) + "\n").encode()
    if path.exists():
        if path.is_symlink() or path.read_bytes() != raw:
            raise server.SupervisorError("Immutable activation binding differs")
        return
    descriptor, pending = tempfile.mkstemp(
        prefix=".activation-pending-", dir=path.parent
    )
    with os.fdopen(descriptor, "wb") as stream:
        stream.write(raw)
        stream.flush()
        os.fsync(stream.fileno())
    try:
        os.link(pending, path, follow_symlinks=False)
    except FileExistsError:
        if path.is_symlink() or path.read_bytes() != raw:
            raise server.SupervisorError(
                "Immutable activation binding differs"
            ) from None
    descriptor = os.open(path.parent, os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def borrowers(state: Path) -> dict[str, object]:
    """Freeze this service's registered obligations, including retained evidence."""
    selected: dict[str, object] = {}
    for key, record in test_resources.resource_status().items():
        if not isinstance(record, dict):
            raise server.SupervisorError("Unresolved activation borrower inventory")
        target = Path(record.get("state", ""))
        if target == state or state / ".receivers" in target.parents:
            selected[key] = dict(record)
    contexts = state / ".contexts"
    selected["owned-context-files"] = (
        {
            str(path.relative_to(state)): server.file_digest(path)
            for path in sorted(contexts.rglob("*"))
            if path.is_file()
        }
        if contexts.exists()
        else {}
    )
    return selected


def verify_backup(archive: Path, binding: dict[str, object]) -> None:
    root = archive / "backup"
    manifest = server.read_json(root / "backup.json")
    if server.file_digest(root / "backup.json") != binding["backup_digest"]:
        raise server.SupervisorError("Original activation backup manifest changed")
    files = server.object_mapping(manifest["files"])
    actual = {str(path.relative_to(root)) for path in root.rglob("*") if path.is_file()}
    if actual != {*files, "backup.json"}:
        raise server.SupervisorError("Original activation backup inventory changed")
    for name, digest in files.items():
        path = root / name
        if (
            root not in path.parents
            or any(p.is_symlink() for p in (path, *path.parents))
            or server.file_digest(path) != digest
        ):
            raise server.SupervisorError("Original activation backup bytes changed")


def bridge_file_identity(bridge: Path) -> tuple[int, int, int, int, int, int]:
    """Pin the verified executable without re-reading its bytes on every RPC."""
    metadata = bridge.lstat()
    if not stat.S_ISREG(metadata.st_mode) or not metadata.st_mode & 0o111:
        raise server.SupervisorError("Native activation bridge identity changed")
    return (
        metadata.st_dev,
        metadata.st_ino,
        metadata.st_size,
        metadata.st_mode,
        metadata.st_mtime_ns,
        metadata.st_ctime_ns,
    )


def physical_binding(
    state: Path,
    config: dict[str, object],
    bridge: Path,
    *,
    bridge_digest: str | None = None,
) -> dict[str, object]:
    return {
        "state": str(state),
        "bridge": str(bridge),
        "bridge_digest": server.file_digest(bridge)
        if bridge_digest is None
        else bridge_digest,
        "credentials_digest": server.file_digest(state / "credentials.json"),
        **{
            key: config[key]
            for key in (
                "instance_id",
                "namespace",
                "database",
                "endpoint",
                "server",
                "resources",
            )
        },
    }


def verify_history_archive(archive: Path) -> None:
    """Freeze page-manifest identity and verify every bounded immutable page."""
    manifests: dict[str, object] = {}
    snapshot = archive / "native-snapshot.json"
    if snapshot.exists():
        original = server.read_json(snapshot).get("history")
        if original is not None:
            manifests["source"] = original
    derived = archive / "native-history-derived.json"
    if derived.exists():
        manifests["derived"] = server.read_json(derived)
    if not manifests:
        return
    directory = archive / "native-history"
    if directory.is_symlink() or not directory.is_dir():
        raise server.SupervisorError("Historical page directory changed")
    for role, manifest in manifests.items():
        if not isinstance(manifest, dict) or not isinstance(
            manifest.get("pages"), list
        ):
            raise server.SupervisorError("Historical page manifest malformed")
        frozen_json(archive / ("native-history-" + role + "-owner.json"), manifest)
        pages = manifest["pages"]
        if len(pages) > 131072:
            raise server.SupervisorError("Historical page manifest exceeds allocation")
        seen: set[str] = set()
        for page in pages:
            if not isinstance(page, dict):
                raise server.SupervisorError("Historical page descriptor malformed")
            name, extent, digest = (
                page.get("name"),
                page.get("bytes"),
                page.get("sha256"),
            )
            prefix = "source-" if role == "source" else "derive-"
            if (
                not isinstance(name, str)
                or len(name) != 20
                or not name.startswith(prefix)
                or not name.endswith(".json")
                or not name[7:15].isascii()
                or not name[7:15].isdigit()
                or name in seen
                or not isinstance(extent, int)
                or not 0 <= extent <= 4194304
                or not isinstance(digest, str)
                or len(digest) != 64
            ):
                raise server.SupervisorError(
                    "Historical page identity or bound malformed"
                )
            seen.add(name)
            path = directory / name
            if (
                path.is_symlink()
                or not path.is_file()
                or path.stat().st_size != extent
                or server.file_digest(path) != digest
            ):
                raise server.SupervisorError("Frozen historical page bytes changed")


def authorize_native(
    child: subprocess.Popen[str], request: dict[str, object], operation_digest: str
) -> None:
    """Acknowledge exactly one live bridge operation after all owner checks."""
    if child.stdin is None:
        raise server.SupervisorError("Native activation control pipe unavailable")
    child.stdin.write(
        json.dumps({"step": request["step"], "operation": operation_digest}) + "\n"
    )
    child.stdin.flush()


def authenticate_archive_boundary(
    archive: Path,
    binding: dict[str, object],
    request: dict[str, object],
    bridge: Path | None = None,
) -> None:
    """Hash preserved material at finite phase boundaries, never per RPC.

    The native owner authenticates each bounded page when it consumes it. The
    supervisor authenticates the complete frozen archive before derivation,
    before effects and before marker publication, and again on completion.
    Live owner, borrower and operation authorization remains outside this
    function and runs for every request.
    """
    phase = request.get("phase")
    if (
        phase in ("archive:source", "archive:derived", "archive:final")
        or request.get("event") == "complete"
    ):
        if bridge is not None:
            physical = server.object_mapping(binding["physical"])
            if server.file_digest(bridge) != physical["bridge_digest"]:
                raise server.SupervisorError("Native activation bridge bytes changed")
        verify_backup(archive, binding)
        verify_history_archive(archive)


def execute(
    state: Path, migration: str, archive: Path, bridge: Path
) -> dict[str, object]:
    """Resume only the original operation; never dispose of historical material."""
    if not migration or len(migration) > 4096:
        raise server.SupervisorError(
            "A bounded immutable migration identity is required"
        )
    state = server.checked_directory(state)
    archive = server.checked_directory(archive)
    bridge = bridge.absolute()
    if state == archive or state in archive.parents or archive in state.parents:
        raise server.SupervisorError(
            "Activation archive must be outside the owned state"
        )
    if bridge.is_symlink() or not bridge.is_file() or not os.access(bridge, os.X_OK):
        raise server.SupervisorError(
            "Activation requires the built native codec bridge"
        )
    with server.lifecycle_reservation(state):
        config = server.config_for(state)
        server.private_offline_state(state, config)
        pending = {"migration": migration, "archive": str(archive)}
        previous = config.get("preparation_activation")
        if previous is not None and previous != pending:
            raise server.SupervisorError(
                "A different preparation activation remains closed"
            )
        if previous is None and any(archive.iterdir()):
            raise server.SupervisorError("Unknown activation history retained")
        binding_path = archive / "operation.json"
        intent_path = archive / "intent.json"
        intent: dict[str, object]
        if intent_path.exists():
            intent = server.read_json(intent_path)
            if intent.get("migration") != migration or intent.get(
                "physical"
            ) != physical_binding(state, config, bridge):
                raise server.SupervisorError("Original migration intent differs")
            try:
                uuid.UUID(str(intent["legacy_issuing_store"]))
            except (KeyError, ValueError) as error:
                raise server.SupervisorError(
                    "Original legacy issuer binding is invalid"
                ) from error
        else:
            intent = {
                "version": 1,
                "migration": migration,
                "physical": physical_binding(state, config, bridge),
                "legacy_issuing_store": str(uuid.uuid4()),
            }
        if binding_path.exists():
            if not intent_path.exists():
                raise server.SupervisorError("Original migration intent is missing")
            binding = server.read_json(binding_path)
            if (
                binding.get("migration") != migration
                or binding.get("physical") != physical_binding(state, config, bridge)
                or binding.get("legacy_issuing_store") != intent["legacy_issuing_store"]
            ):
                raise server.SupervisorError(
                    "Original migration or physical binding differs"
                )
            frozen_json(archive / "borrowers.json", borrowers(state))
            verify_backup(archive, binding)
        else:
            # Incomplete private staging is retained as non-authority. The
            # immutable intent precedes backup; native effects require the later
            # operation.json and native snapshot to have been durably published.
            allowed = {"intent.json", "borrowers.json", "backup"}
            if any(
                path.name not in allowed
                and not path.name.startswith(
                    (".activation-pending-", ".backup-pending-")
                )
                for path in archive.iterdir()
            ):
                raise server.SupervisorError("Unknown activation history retained")
            if any(archive.iterdir()) and previous != pending:
                raise server.SupervisorError("Unknown activation history retained")
            # Closing admission precedes backup and every native effect. Even an
            # interrupted snapshot remains closed for explicit operator recovery.
            config.update(
                accepting_writes=False,
                admission="validation_required",
                preparation_activation=pending,
            )
            server.write_json(state / "config.json", config)
            frozen_json(intent_path, intent)
            frozen_json(archive / "borrowers.json", borrowers(state))
            if not (archive / "backup").exists():
                staged = archive / (".backup-pending-" + uuid.uuid4().hex)
                # No target effect can exist before operation publication. A
                # fresh complete backup of this still-stopped original store is
                # safe; partial predecessors stay untouched and non-authoritative.
                server._backup(state, config, staged)  # noqa: SLF001 -- retain the enclosing authentic reservation
                staged.rename(archive / "backup")
                descriptor = os.open(archive, os.O_DIRECTORY)
                try:
                    os.fsync(descriptor)
                finally:
                    os.close(descriptor)
            binding = {
                "version": 1,
                "migration": migration,
                "legacy_issuing_store": intent["legacy_issuing_store"],
                "physical": physical_binding(state, config, bridge),
                "backup_digest": server.file_digest(archive / "backup" / "backup.json"),
                "borrowers_digest": server.file_digest(archive / "borrowers.json"),
            }
            frozen_json(binding_path, binding)
            verify_backup(archive, binding)
        config.update(
            accepting_writes=False,
            admission="validation_required",
            preparation_activation=pending,
        )
        server.write_json(state / "config.json", config)
        server.maintenance_endpoint(state)
        operation_digest = server.file_digest(binding_path)
        intent_digest = server.file_digest(intent_path)
        bridge_identity = bridge_file_identity(bridge)
        physical = server.object_mapping(binding["physical"])
        bridge_digest = str(physical["bridge_digest"])
        verify_history_archive(archive)
        child = subprocess.Popen(
            [str(bridge), str(state), str(archive)],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
            bufsize=1,
        )
        completed: dict[str, object] | None = None
        failure: str | None = None
        try:
            if child.stdout is None or child.stdin is None:
                raise server.SupervisorError(
                    "Native activation control pipe unavailable"
                )
            with selectors.DefaultSelector() as selector:
                selector.register(child.stdout, selectors.EVENT_READ)
                while True:
                    if not selector.select(timeout=0.2):
                        if child.poll() is not None:
                            break
                        continue
                    raw = child.stdout.readline(65537)
                    if not raw or len(raw) > 65536:
                        break
                    request = json.loads(raw)
                    if not isinstance(request, dict):
                        raise server.SupervisorError(
                            "Invalid native activation control"
                        )
                    current = server.config_for(state)
                    server.require_maintenance_owner(state, current)
                    if not server.listener_ready(state, current):
                        raise server.SupervisorError(
                            "Exact maintenance invocation is unavailable"
                        )
                    if (
                        bridge_file_identity(bridge) != bridge_identity
                        or physical_binding(
                            state, current, bridge, bridge_digest=bridge_digest
                        )
                        != binding["physical"]
                        or current.get("preparation_activation") != pending
                        or server.file_digest(binding_path) != operation_digest
                        or server.file_digest(intent_path) != intent_digest
                        or server.file_digest(archive / "borrowers.json")
                        != binding["borrowers_digest"]
                        or server.read_json(archive / "borrowers.json")
                        != borrowers(state)
                    ):
                        raise server.SupervisorError(
                            "Activation owner or frozen borrower binding changed"
                        )
                    authenticate_archive_boundary(archive, binding, request, bridge)
                    if request.get("event") == "authorize" and isinstance(
                        request.get("step"), int
                    ):
                        authorize_native(child, request, operation_digest)
                    elif request.get("event") == "complete":
                        completed = request
                    elif request.get("event") == "failed":
                        reason = request.get("reason")
                        if not isinstance(reason, str):
                            raise server.SupervisorError(
                                "Invalid native activation failure reason"
                            )
                        failure_detail: str = reason[-4096:]
                        for name, secret in server.read_json(
                            state / "credentials.json"
                        ).items():
                            if "password" in name and isinstance(secret, str):
                                failure_detail = failure_detail.replace(
                                    secret, "[REDACTED]"
                                )
                        failure = failure_detail
                    else:
                        raise server.SupervisorError(
                            "Unknown native activation control request"
                        )
            status = child.wait(timeout=10)
            if status or completed is None:
                raise server.SupervisorError(
                    "Native activation interrupted; original archive retained and admission closed"
                    + (": " + failure if failure else "")
                )
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
        # The reservation's exit physically drains the listener. Do not use
        # finish_maintenance: activation grants neither disposal nor readmission.
        return {
            "migration": migration,
            "archive": str(archive),
            "admission": "validation_required",
            "native": completed,
        }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--state", type=Path, required=True)
    parser.add_argument("--migration", required=True)
    parser.add_argument("--archive", type=Path, required=True)
    parser.add_argument("--bridge", type=Path, required=True)
    args = parser.parse_args()
    try:
        result = execute(args.state, args.migration, args.archive, args.bridge)
    except (
        server.SupervisorError,
        OSError,
        ValueError,
        subprocess.SubprocessError,
    ) as error:
        print(str(error), file=sys.stderr)
        return 1
    print(json.dumps(result, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
