# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Once-only case observation; scientific ownership remains in the case process."""

from __future__ import annotations

import json
import os
import subprocess
import time
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pse.diagnostics import CaptureSession


def _association_checkpoint(association: dict) -> str:
    checkpoint = association["checkpoint"]
    if not isinstance(checkpoint, str):
        raise TypeError("registered diagnostic checkpoint is not a path")
    return checkpoint


def _ready_handoff_identity(marker: dict, handoff: dict, pid: int) -> dict:
    if not handoff["enabled"]:
        raise ValueError("registered receiver diagnostics opted out")
    identity = handoff["identity"]
    if (
        identity["locator"]["pid"] != pid
        or identity["locator"]["start_ticks"] is None
        or not identity["process"]
        or not marker.get("nonce")
    ):
        raise ValueError("registered diagnostic locator is incomplete")
    return identity


class Observer:
    """Keep typed inspection windows and explicitly selected optional captures."""

    def __init__(self, directory: Path) -> None:
        # Ordinary benchmark collection does not import the native extension.
        from pse import codec, diagnostics  # noqa: PLC0415
        from pse.contracts.documents import (  # noqa: PLC0415
            CaptureRequest,
            CheckpointIdentity,
            ClosedDuration,
            DiagnosticListRequest,
            InspectionRequest,
            ReceiverCheckpoint,
        )
        from pse.contracts.enums import CaptureKind  # noqa: PLC0415

        self.directory = directory
        self.codec = codec
        self.diagnostics = diagnostics
        self.request = InspectionRequest
        self.identity_type = CheckpointIdentity
        self.receiver_type = ReceiverCheckpoint
        self.capture_request = CaptureRequest
        self.list_request = DiagnosticListRequest
        self.duration = ClosedDuration
        self.capture_kind = CaptureKind
        self.reads = 0
        self.failures: list[str] = []
        self.captures: list[tuple[str, CaptureSession]] = []
        self.captured = False
        self.started = time.monotonic()
        self.managed_checkpoint: Path | None = None
        self.managed_operations: list = []
        self.managed_offset = 0
        self.managed_cursor = None
        self.managed_association: dict | None = None

    def write(self, name: str, document: object) -> None:
        (self.directory / name).write_bytes(self.codec.encode_json(document))

    def inspect(self, *, terminal: bool = False) -> None:
        locator = self.directory / "case-operation.json"
        if not locator.exists():
            self.inspect_managed()
            return
        try:
            target = json.loads(locator.read_text())
            checkpoint = Path(target["checkpoint"])
            root = self.diagnostics.operation_id(target["operation"])
            image = self.diagnostics.read_checkpoint(checkpoint)
            pending = [root]
            seen = set()
            documents = []
            # Bounded navigation is deliberate. Omitted owners and unavailable
            # facts remain in Rust's documents, rather than becoming zero work.
            while pending and len(documents) < 8:
                operation = pending.pop(0)
                key = self.diagnostics.operation_text(operation)
                if key in seen:
                    continue
                seen.add(key)
                document = self.diagnostics.inspect_image(
                    image,
                    self.request(operation=operation, related_limit=32),
                    checkpoint=checkpoint,
                )
                documents.append(document)
                pending.extend(
                    related.id
                    for related in document.related
                    if terminal or related.outcome is None
                )
                self.write(
                    f"inspection-{self.reads:05d}-{len(documents):02d}.json", document
                )
            if len(documents) == 1:
                page = self.diagnostics.list_processes(
                    self.list_request(limit=1), directory=checkpoint.parent
                )
                self.write(f"process-catalog-{self.reads:05d}.json", page)
                for process in page.processes:
                    if Path(process.checkpoint) != checkpoint:
                        continue
                    for operation in process.operations:
                        if len(documents) >= 8:
                            break
                        key = self.diagnostics.operation_text(operation.id)
                        if key in seen or operation.outcome is not None:
                            continue
                        document = self.diagnostics.inspect_image(
                            image,
                            self.request(operation=operation.id, related_limit=32),
                            checkpoint=checkpoint,
                        )
                        documents.append(document)
                        self.write(
                            f"process-inspection-{self.reads:05d}-{len(documents):02d}.json",
                            document,
                        )
                # These are actual same-process owners, not invented parent links.
            self.write(f"checkpoint-{self.reads:05d}.json", image)
            active = [
                document.operation
                for document in documents
                if document.operation is not None and document.operation.outcome is None
            ]
            now = time.monotonic()
            description = "; ".join(
                f"{operation.kind}: {','.join(phase.kind for phase in operation.phases) or 'no active phase'}"
                for operation in active
            )
            print(
                f"audit {self.directory.parent.name}: {description or 'terminal or unavailable'}",
                flush=True,
            )
            if (
                not terminal
                and active
                and not self.captured
                and now - self.started >= 30
            ):
                self.captured = True
                # These are optional captures of this exact root process, not
                # a timer that cancels science or attaches to unrelated work.
                for kind in (self.capture_kind.CPU, self.capture_kind.TIMELINE):
                    name = kind.value.lower()
                    try:
                        session = self.diagnostics.capture(
                            self.capture_request(
                                checkpoint=str(checkpoint),
                                operation=root,
                                kind=kind,
                                output=str(self.directory / f"capture-{name}"),
                                duration=self.duration(secs=10, nanos=0),
                            )
                        )
                        self.captures.append((name, session))
                    except (
                        OSError,
                        ValueError,
                        RuntimeError,
                        self.diagnostics.InspectionError,
                    ) as error:
                        self.failures.append(f"{name} capture unavailable: {error}")
            self.reads += 1
        except (
            OSError,
            ValueError,
            KeyError,
            RuntimeError,
            self.diagnostics.InspectionError,
        ) as error:
            self.failures.append(f"inspection unavailable: {error}")
        self.inspect_managed()

    def inspect_managed(self) -> None:
        """Select registered receiver identity; absence never becomes caller zero."""
        target_path = self.directory / "managed-target.json"
        association = getattr(self, "managed_association", None)
        if association is None and not target_path.exists():
            return
        # Association is immutable once selected. Reading its retained file never
        # depends on another live process lookup or a replacement registration.
        try:
            target = (
                association
                if association is not None
                else json.loads(target_path.read_text())
            )
            if association is None and "caller" in target:
                association = self._managed_from_caller(target)
                if association is not None:
                    self.managed_association = association
                    self.managed_checkpoint = Path(_association_checkpoint(association))
            marker_path = Path(target["state"]) / "primary-receiver.json"
            if association is None and "caller" not in target and marker_path.exists():
                marker = json.loads(marker_path.read_text())
                if (
                    marker.get("ready")
                    and marker.get("canonical_database") == target["database"]
                ):
                    pid = int(marker["pid"])
                    handoff = marker.get("diagnostics")
                    if handoff is not None:
                        identity = _ready_handoff_identity(marker, handoff, pid)
                        association = {
                            **target,
                            "nonce": marker["nonce"],
                            "identity": identity,
                            "checkpoint": handoff["checkpoint"],
                            "writer_failure": handoff.get("writer_failure"),
                            "discovery_partial": False,
                        }
                        self.managed_association = association
                        self.managed_checkpoint = Path(
                            _association_checkpoint(association)
                        )
                        # The first checkpoint may still be pending; cache the
                        # registered association now rather than rediscover later.
                    else:
                        self._discover_managed(target, marker, pid)
                        association = getattr(self, "managed_association", None)
        except (
            OSError,
            ValueError,
            KeyError,
            TypeError,
            RuntimeError,
            self.diagnostics.InspectionError,
        ) as error:
            self.failures.append(f"managed receiver association unavailable: {error}")
        # Deliberately outside association's exception block: a missing /proc,
        # removed marker or replacement receiver cannot prevent a cached read.
        if association is None:
            return
        try:
            expected = self.codec.decode_json(
                json.dumps(association["identity"]).encode(), self.identity_type
            )
            image = self.diagnostics.read_checkpoint(
                _association_checkpoint(association), expected=expected
            )
            self.managed_operations = [operation.id for operation in image.operations]
            operations = self.managed_operations
            if operations:
                offset = self.managed_offset % len(operations)
                operations = operations[offset:] + operations[:offset]
                self.managed_offset = (offset + 8) % len(operations)
            for index, operation in enumerate(operations[:8]):
                document = self.diagnostics.inspect_image(
                    image,
                    self.request(operation=operation, related_limit=32),
                    checkpoint=_association_checkpoint(association),
                )
                self.write(
                    f"managed-inspection-{self.reads:05d}-{index:02d}.json", document
                )
            # Serialize exactly the validated decoded image. Reopening/copying
            # the path could copy a replacement that was never inspected.
            self.write(f"managed-checkpoint-{self.reads:05d}.json", image)
            (self.directory / "managed-association.json").write_text(
                json.dumps({**association, "per_read_limit": 8}, indent=2) + "\n"
            )
        except (
            OSError,
            ValueError,
            KeyError,
            TypeError,
            RuntimeError,
            self.diagnostics.InspectionError,
        ) as error:
            self.failures.append(f"managed receiver observation unavailable: {error}")

    @staticmethod
    def _receiver_chunks(sources: dict, inventory: dict, prefix: str) -> bytes:
        chunks = inventory.get(f"{prefix}_chunks", {})
        extent = inventory.get(f"{prefix}_bytes", {})
        if chunks.get("kind") != "Integer" or extent.get("kind") != "Integer":
            raise ValueError("receiver handoff chunk inventory unavailable")
        count, length = chunks["value"], extent["value"]
        if len(sources) > 32 or not 1 <= count <= 32 or not 1 <= length <= count * 256:
            raise ValueError("receiver handoff chunk inventory exceeds source bounds")
        names = [f"{prefix}.{index}" for index in range(count)]
        if (
            any(name not in sources for name in names)
            or sum(name.startswith(f"{prefix}.") for name in sources) != count
        ):
            raise ValueError("receiver handoff source chunks incomplete")
        if any(not 1 <= len(sources[name]) <= 256 for name in names):
            raise ValueError("receiver handoff source chunk exceeds byte bounds")
        payload = b"".join(bytes(sources[name]) for name in names)
        if len(payload) != length:
            raise ValueError("receiver handoff source extent differs")
        return payload

    def _managed_from_caller(self, target: dict) -> dict | None:
        """Recover the actual consumed ready handoff from its copied responsibility."""
        caller = target["caller"]
        expected = self.codec.decode_json(
            json.dumps(caller["identity"]).encode(), self.identity_type
        )
        image = self.diagnostics.read_checkpoint(
            caller["checkpoint"], expected=expected
        )
        for operation in sorted(
            image.operations, key=lambda operation: operation.id.sequence, reverse=True
        ):
            if (
                operation.summary is None
                or operation.summary.purpose != "registered receiver checkpoint"
            ):
                continue
            sources = {source.name: source.bytes for source in operation.sources}
            inventory = {
                metric.name: json.loads(self.codec.encode_json(metric.latest))
                for metric in operation.metrics
                if metric.name
                in {
                    "receiver.state_chunks",
                    "receiver.state_bytes",
                    "receiver.database_chunks",
                    "receiver.database_bytes",
                }
            }
            state = self._receiver_chunks(sources, inventory, "receiver.state")
            database = self._receiver_chunks(sources, inventory, "receiver.database")
            if (
                state != target["state"].encode()
                or database != target["database"].encode()
            ):
                continue
            # Age this same decoded image through Rust's process-clock mapping
            # and summary-expiry rules before establishing a new association.
            if image.clock is None:
                raise ValueError("receiver handoff summary clock unavailable")
            document = self.diagnostics.inspect_image(
                image,
                self.request(
                    operation=operation.id, include_related=False, related_limit=0
                ),
                checkpoint=caller["checkpoint"],
            )
            selected_operation = document.operation
            if (
                selected_operation is None
                or selected_operation.summary is None
                or selected_operation.coverage.stale
            ):
                raise ValueError(
                    "receiver handoff summary expired or freshness unavailable"
                )
            sources = {
                source.name: source.bytes for source in selected_operation.sources
            }
            inventory = {
                metric.name: json.loads(self.codec.encode_json(metric.latest))
                for metric in selected_operation.metrics
                if metric.name in {"receiver.handoff_chunks", "receiver.handoff_bytes"}
            }
            payload = self._receiver_chunks(sources, inventory, "receiver.handoff")
            copied = self.codec.decode_json(payload, self.receiver_type)
            handoff = json.loads(self.codec.encode_json(copied))
            if (
                handoff["receiver_state"] != target["state"]
                or handoff["canonical_database"] != target["database"]
            ):
                raise ValueError(
                    "receiver handoff selection differs from copied discriminator"
                )
            if (
                not handoff["nonce"]
                or handoff["identity"]["locator"]["start_ticks"] is None
            ):
                raise ValueError("receiver handoff registration identity incomplete")
            if not handoff["enabled"] or not handoff["available"]:
                raise ValueError(
                    "registered receiver diagnostics opted out or unavailable"
                )
            return {
                **target,
                "nonce": handoff["nonce"],
                "identity": handoff["identity"],
                "checkpoint": handoff["checkpoint"],
                "writer_failure": handoff["writer_failure"],
                "discovery_partial": False,
                "source": "actual caller ready responsibility",
            }
        return None

    def _discover_managed(self, target: dict, marker: dict, pid: int) -> None:
        """Bounded incomplete fallback for an observation without an exact handoff."""
        start = int(Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()[19])
        cursor = self.managed_cursor
        default_directory = (
            Path(os.environ.get("XDG_STATE_HOME", str(Path.home() / ".local/state")))
            / "pse/diagnostics"
        )
        for _ in range(4):
            page = self.diagnostics.list_processes(
                self.list_request(after=cursor, limit=32), directory=default_directory
            )
            match = next(
                (
                    process
                    for process in page.processes
                    if process.locator is not None
                    and process.locator.pid == pid
                    and process.locator.start_ticks == start
                ),
                None,
            )
            if match is not None:
                self.managed_checkpoint = Path(match.checkpoint)
                self.managed_association = {
                    **target,
                    "nonce": marker.get("nonce"),
                    "identity": {
                        "process": match.process,
                        "locator": {
                            "pid": pid,
                            "start_ticks": start,
                            "executable": None,
                        },
                    },
                    "checkpoint": match.checkpoint,
                    "discovery_partial": True,
                }
                break
            cursor = page.next
            self.managed_cursor = cursor
            if cursor is None:
                break

    def finish(self) -> dict:
        self.inspect(terminal=True)
        for name, session in self.captures:
            try:
                self.write(f"capture-{name}.json", session.stop())
            except (
                OSError,
                ValueError,
                RuntimeError,
                self.diagnostics.InspectionError,
            ) as error:
                self.failures.append(f"{name} capture join unavailable: {error}")
        return {
            "inspection_windows": self.reads,
            "managed_checkpoint": str(self.managed_checkpoint)
            if self.managed_checkpoint is not None
            else None,
            "diagnostic_failures": self.failures,
        }


def reconcile_outcome(returncode: int, outcome: dict | None) -> dict:
    """A child exit never supplies missing scientific or cleanup evidence."""
    if outcome is None:
        return {
            "status": "failed",
            "exit_code": returncode,
            "reason": "audit outcome missing",
            "cleanup": "unobserved",
        }
    status = outcome.get("status", "failed")
    if status == "completed" and returncode != 0:
        status = "failed"
    return {**outcome, "status": status, "exit_code": returncode}


def run_case(
    command: list[str], *, cwd: Path, env: dict[str, str], directory: Path
) -> dict:
    """Observe one owned child. Stop requests are handled by its actual owners."""
    audit = directory / "diagnostics"
    audit.mkdir()
    env = {
        **env,
        "PSE_CASE_AUDIT_DIR": str(audit),
        "PSE_DIAGNOSTICS_DIR": str(audit / "checkpoints"),
    }
    observer = Observer(
        audit
    )  # Fail before launching if the typed interface is unavailable.
    started = time.monotonic()
    next_read = started
    interrupted = False
    with (directory / "process.log").open("w") as log:
        with subprocess.Popen(
            command,
            cwd=cwd,
            env=env,
            stdout=log,
            stderr=subprocess.STDOUT,
            start_new_session=True,
        ) as child:
            try:
                while child.poll() is None:
                    now = time.monotonic()
                    if now >= next_read:
                        observer.inspect()
                        next_read = now + (5 if now - started < 30 else 15)
                    time.sleep(0.2)
            except KeyboardInterrupt:
                # No signal/kill fallback. Native drain and fixture release
                # remain the child owner's responsibility and are awaited.
                (audit / "stop.request").write_text("operator interrupted audit\n")
                child.wait()
                interrupted = True
            finally:
                observation = observer.finish()
        terminal = audit / "audit-outcome.json"
        result = reconcile_outcome(
            child.returncode,
            json.loads(terminal.read_text()) if terminal.exists() else None,
        )
        result.update(observation, elapsed_seconds=time.monotonic() - started)
        (audit / "observation.json").write_text(json.dumps(result, indent=2) + "\n")
        if interrupted:
            raise KeyboardInterrupt
        return result
