# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Once-only case observation; scientific ownership remains in the case process."""

from __future__ import annotations

import base64
import json
import math
import os
import subprocess
import time
from contextlib import contextmanager
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable, Generator

    from pse.contracts.documents import (
        AnalysisDocumentOutput,
        AnalysisRequest,
        FrozenEvidenceSetOutput,
        InspectionDocumentOutput,
        InspectionRequest,
        OperationSnapshotBodyOutput,
        PackFormat,
        PackMemberRole,
        ProcessCheckpointOutput,
        ReviewHandoffReceipt,
        SourceRecordingAcknowledgement,
        SourceRecordingRequest,
    )
    from pse.diagnostics import CaptureSession


REVIEW_ANALYSIS_MEMORY_BYTES = 256 << 20
REVIEW_TABLE_MEMORY_BYTES = 64 << 20


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
            FrozenEvidenceSetOutput,
            FrozenInspectionRequest,
            InspectionRequest,
            OperationIdBody,
            ReceiverCheckpoint,
        )
        from pse.contracts.enums import CaptureKind  # noqa: PLC0415

        self.directory = directory
        self.codec = codec
        self.diagnostics = diagnostics
        self.operation_body = OperationIdBody
        self.request = InspectionRequest
        self.evidence_type = FrozenEvidenceSetOutput
        self.frozen_request_type = FrozenInspectionRequest
        self.recorded_operations: dict[str, OperationSnapshotBodyOutput] = {}
        self.frozen_member_image: ProcessCheckpointOutput | None = None
        self.frozen_member_index = 0
        self.frozen_request_index = 0
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
        self.capture_from_start = False
        self.started = time.monotonic()
        self.managed_checkpoint: Path | None = None
        self.managed_operations: list = []
        self.managed_offset = 0
        self.managed_cursor = None
        self.managed_association: dict | None = None

    def write(self, name: str, document: object) -> None:
        (self.directory / name).write_bytes(self.codec.encode_json(document))

    def interpret_image(
        self,
        image: ProcessCheckpointOutput,
        request: InspectionRequest,
        checkpoint: Path,
        *,
        terminal: bool,
    ) -> InspectionDocumentOutput:
        """Live observation or a recorded final read over explicitly retained members."""
        if terminal:
            local = {
                self.diagnostics.operation_text(operation.id)
                for operation in image.operations
            }
            supplied = self.evidence_type(
                version=3,
                checkpoints=(image,),
                operations=tuple(
                    operation
                    for key, operation in self.recorded_operations.items()
                    if key not in local
                ),
            )
            frozen = self.diagnostics.inspect_frozen(
                supplied, self.frozen_request_type(version=1, inspection=request)
            )
            if self.frozen_member_image is not image:
                self.frozen_member_index += 1
                self.frozen_member_image = image
                self.write(
                    f"frozen-evidence-{self.frozen_member_index:03d}.json", supplied
                )
            self.frozen_request_index += 1
            self.write(
                f"frozen-selection-{self.frozen_member_index:03d}-{self.frozen_request_index:03d}.json",
                frozen,
            )
            return frozen.inspection
        document = self.diagnostics.inspect_image(image, request, checkpoint=checkpoint)
        for operation in (
            *document.related,
            *((document.operation,) if document.operation is not None else ()),
        ):
            key = self.diagnostics.operation_text(operation.id)
            self.recorded_operations[key] = operation
            if len(self.recorded_operations) > 256:
                self.recorded_operations.pop(next(iter(self.recorded_operations)))
        return document

    def inspect(self, *, terminal: bool = False) -> None:
        locator = self.directory / "case-operation.json"
        if not locator.exists():
            self.inspect_managed(terminal=terminal)
            return
        try:
            target = json.loads(locator.read_text())
            checkpoint = Path(target["checkpoint"])
            root = self.codec.convert_document(
                self.codec.document_builtins(
                    self.diagnostics.operation_id(target["operation"])
                ),
                self.operation_body,
            )
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
                document = self.interpret_image(
                    image,
                    self.request(operation=operation, related_limit=32),
                    checkpoint=checkpoint,
                    terminal=terminal,
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
            if len(documents) == 1 and not terminal:
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
                and not getattr(self, "capture_from_start", False)
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
        self.inspect_managed(terminal=terminal)

    def inspect_managed(self, *, terminal: bool = False) -> None:
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
                document = self.interpret_image(
                    image,
                    self.request(operation=operation, related_limit=32),
                    checkpoint=Path(_association_checkpoint(association)),
                    terminal=terminal,
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
            "capture_metadata": [
                str(path)
                for path in sorted(self.directory.glob("capture-*.json"))
                if path.name != "capture-launch-request.json"
            ],
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


def _read_bounded(path: Path, maximum_bytes: int) -> bytes:
    with path.open("rb") as source:
        encoded = source.read(maximum_bytes + 1)
    if len(encoded) > maximum_bytes:
        raise ValueError(f"{path.name} exceeds {maximum_bytes} byte allowance")
    return encoded


def prepare_source_recording(
    env: dict[str, str], audit: Path, *, review_artifacts: bool = False
) -> dict[str, str]:
    """Materialize the case's default selection using the shared launch protocol.

    An explicit caller envelope takes precedence. Only the actual Rust construction
    entrypoint can acknowledge it; merely supplying this variable records nothing.
    """
    if "PSE_DIAGNOSTIC_RECORD_REQUEST" in env:
        from pse import codec  # noqa: PLC0415
        from pse.contracts.documents import SourceRecordingRequest  # noqa: PLC0415

        selected = codec.decode_json(
            _read_bounded(Path(env["PSE_DIAGNOSTIC_RECORD_REQUEST"]), 64 << 10),
            SourceRecordingRequest,
        )
        if review_artifacts and selected.review is None:
            raise ValueError("explicit recording request omits selected review handoff")
        return env
    audit = audit.resolve()
    workload = json.loads(env.get("PSE_PROCESS_COST_SPEC", "{}"))
    families = workload.get("detail_families", [])
    if not isinstance(families, list) or any(
        not isinstance(family, str) for family in families
    ):
        raise ValueError("detail_families must contain source-family strings")
    if not families:
        if review_artifacts:
            raise ValueError("review artifacts require declared source detail families")
        return env
    from pse import codec  # noqa: PLC0415 -- native-backed audit only
    from pse.contracts.documents import SourceRecordingRequest  # noqa: PLC0415

    request = {
        "version": 2,
        "acknowledgement": str(audit / "recording-acknowledgement.json"),
        "detail": {
            "version": 1,
            "families": families,
            "requirement": "complete",
            "sample_every": 1,
            "boundary_thread_cpu": True,
            "managed_budget": (
                {"memory_bytes": 64 << 20, "output_bytes": 256 << 20}
                if workload.get("adapter") == "managed-durable"
                else None
            ),
            "expiry_monotonic_ns": None,
            "duration": {"secs": 60, "nanos": 0},
            "output": str(audit / "detail"),
            "budget": {"memory_bytes": 64 << 20, "output_bytes": 256 << 20},
        },
        "review": {
            "context": {
                "steps": [],
                "include_sources": True,
                "include_native_records": True,
                "maximum_bytes": 16 << 20,
            },
            "maximum_bytes": 64 << 20,
            "maximum_members": 256,
        }
        if review_artifacts
        else None,
    }
    selected = codec.decode_json(json.dumps(request).encode(), SourceRecordingRequest)
    encoded = codec.encode_json(selected)
    if len(encoded) > 64 << 10:
        raise ValueError("recording request exceeds finite protocol bound")
    path = audit / "recording-request.json"
    with path.open("xb") as destination:
        destination.write(encoded)
    return {**env, "PSE_DIAGNOSTIC_RECORD_REQUEST": str(path)}


def _append_preparation_members(index: Path, paths: list[Path]) -> None:
    document = json.loads(_read_bounded(index, 1 << 20))
    members = document["members"]
    if not isinstance(members, list) or len(members) > 256:
        raise ValueError("preparation member inventory exceeds allowance")
    for member in members:
        relative = Path(member["path"])
        if relative.is_absolute() or ".." in relative.parts:
            raise ValueError("preparation input path is not contained")
        path = index.parent / relative
        if not path.resolve().is_relative_to(index.parent.resolve()):
            raise ValueError("preparation input leaves its acknowledged group")
        if not path.is_file() or path.stat().st_size != member["bytes"]:
            raise ValueError(
                "preparation input length differs from loader acknowledgement"
            )
        paths.append(path)


def preparation_input_members(audit: Path) -> tuple[list[Path], list[str]]:
    """Select only loader-acknowledged, fully flushed original input members."""
    paths: list[Path] = []
    errors: list[str] = []
    for index in sorted((audit / "preparation-originals").glob("input-*/inputs.json")):
        paths.append(index)
        try:
            _append_preparation_members(index, paths)
        except (OSError, ValueError, KeyError, TypeError) as error:
            errors.append(f"preparation inventory {index.parent.name}: {error}")
    return paths, errors


def _read_recording_acknowledgement(
    env: dict[str, str],
    audit: Path,
    observer: Observer,
) -> tuple[SourceRecordingRequest, SourceRecordingAcknowledgement]:
    from pse.contracts import documents as d  # noqa: PLC0415

    request = observer.codec.decode_json(
        _read_bounded(Path(env["PSE_DIAGNOSTIC_RECORD_REQUEST"]), 64 << 10),
        d.SourceRecordingRequest,
    )
    acknowledgement = observer.codec.decode_json(
        _read_bounded(Path(request.acknowledgement), 64 << 10),
        d.SourceRecordingAcknowledgement,
    )
    if (
        acknowledgement.selection.families != request.detail.families
        or acknowledgement.selection.output != request.detail.output
    ):
        raise ValueError("recording acknowledgement differs from original selection")
    locator = audit / "case-operation.json"
    if (
        locator.exists()
        and acknowledgement.operation
        != observer.diagnostics.operation_id(
            json.loads(locator.read_text())["operation"]
        )
    ):
        raise ValueError("recording acknowledgement differs from actual case operation")
    return request, acknowledgement


def _read_review_receipt(
    request: SourceRecordingRequest,
    acknowledgement: SourceRecordingAcknowledgement,
    observer: Observer,
) -> tuple[Path, ReviewHandoffReceipt]:
    from pse.contracts import documents as d  # noqa: PLC0415

    review = acknowledgement.review
    if request.review is None or review is None:
        raise ValueError("review handoff admission missing")
    if review.request != request.review:
        raise ValueError("review admission differs from original selection")
    if not review.accepted:
        raise ValueError(f"review handoff refused: {review.error}")
    path = Path(review.receipt)
    receipt = observer.codec.decode_json(
        _read_bounded(path, 16 << 20), d.ReviewHandoffReceipt
    )
    if (
        receipt.operation != acknowledgement.operation
        or receipt.request != review.request
    ):
        raise ValueError("review receipt differs from actual admission")
    if (
        len(receipt.members) > 256
        or sum(member.bytes for member in receipt.members) > 64 << 20
    ):
        raise ValueError("review handoff exceeds selected aggregate/member allowances")
    return path, receipt


def _check_byte_allowance(size: int, maximum: int, message: str) -> None:
    if size > maximum:
        raise ValueError(message)


def _original_sidecar(
    path: Path,
    audit: Path,
    index: int,
    member_count: int,
    sidecar_bytes: int,
) -> tuple[Path, int]:
    if index >= 256 or member_count >= 256:
        raise ValueError("original sidecar member allowance exhausted")
    # Retain the loader's original binary bytes through the existing JSON envelope.
    source_path = path
    if path.suffix == ".bin":
        encoded = _read_bounded(path, 16 << 20)
        envelopes = audit / "preparation-pack-inputs"
        envelopes.mkdir(exist_ok=True)
        source_path = envelopes / f"input-{index:03d}.json"
        source_path.write_text(
            json.dumps(
                {
                    "encoding": "base64",
                    "original_path": str(path.relative_to(audit)),
                    "original_bytes": len(encoded),
                    "data": base64.b64encode(encoded).decode("ascii"),
                }
            )
            + "\n"
        )
    size = source_path.stat().st_size
    _check_byte_allowance(
        sidecar_bytes + size, 64 << 20, "original sidecar byte allowance exhausted"
    )
    if not path.resolve().is_relative_to(audit.resolve()):
        raise ValueError("original sidecar leaves the case diagnostics directory")
    return source_path, size


def managed_records_path(audit: Path, records: str) -> Path:
    """Read only the explicitly selected managed-child output tree."""
    path = Path(records)
    if not path.resolve().is_relative_to((audit / "detail/managed").resolve()):
        raise ValueError("managed detail reference leaves selected child output")
    return path


def analyze_case_evidence(
    observer: Observer,
    evidence: FrozenEvidenceSetOutput,
    request: AnalysisRequest,
    result: dict,
) -> AnalysisDocumentOutput:
    """Keep full copied facts when an aggregate exceeds the analyzer subject bound."""
    from msgspec.structs import replace  # noqa: PLC0415

    try:
        return observer.diagnostics.analyze_evidence(
            evidence, request, memory_bytes=REVIEW_ANALYSIS_MEMORY_BYTES
        )
    except (RuntimeError, observer.diagnostics.InspectionError) as error:
        if "analysis operation selection exceeds 64 exact subjects" not in str(error):
            raise
        reason = str(error)
    operations = [
        *evidence.operations,
        *(
            operation
            for checkpoint in evidence.checkpoints
            for operation in checkpoint.operations
        ),
    ]
    preferred = [
        invocation.operation
        for invocation in evidence.invocations
        if invocation.operation is not None
    ]
    candidates = [
        *preferred,
        *(
            operation.id
            for operation in sorted(operations, key=lambda item: -len(item.children))
            if len(operation.children) < 63
        ),
    ]
    seen = {observer.diagnostics.operation_text(request.operation)}
    for operation in candidates:
        identity = observer.diagnostics.operation_text(operation)
        if identity in seen:
            continue
        seen.add(identity)
        if len(seen) > 17:
            break
        selected = replace(request, operation=operation)
        try:
            analysis = observer.diagnostics.analyze_evidence(
                evidence, selected, memory_bytes=REVIEW_ANALYSIS_MEMORY_BYTES
            )
        except (RuntimeError, observer.diagnostics.InspectionError):
            continue
        observer.write("review-analysis-request-selected.json", selected)
        result["analysis_scope"] = {
            "requested_operation": observer.diagnostics.operation_text(
                request.operation
            ),
            "requested_refusal": reason,
            "selected_operation": identity,
            "scope": "One actual supplied operation; full original frozen evidence retained unchanged",
        }
        result.setdefault("limits", []).append(
            f"Whole-case automated analysis exceeds 64 exact subjects; selected original operation {identity}; full raw facts retained"
        )
        return analysis
    raise ValueError(f"No supplied bounded operation could be analyzed after: {reason}")


def collect_review_artifacts(
    env: dict[str, str], audit: Path, observer: Observer
) -> dict:
    """Retain actual admissions and export copied facts without scientific lookups."""
    from pse.contracts import documents as d  # noqa: PLC0415

    codec, diagnostics = observer.codec, observer.diagnostics
    result: dict = {"errors": [], "scientific_result_packs": []}
    # These original protected result packs were exported by their live owners
    # before fixture teardown. Their publication is independent of this pack.
    indexes = sorted((audit.parent / "review-results").glob("*/index.json"))
    if len(indexes) > 256:
        result["errors"].append("scientific result pack inventory exceeds 256 members")
        indexes = indexes[:256]
    for index in indexes:
        try:
            document = diagnostics.review_pack(index.parent)
            result["scientific_result_packs"].append(
                {
                    "path": str(index.parent),
                    "index": json.loads(codec.encode_json(document)),
                }
            )
        except (
            OSError,
            ValueError,
            RuntimeError,
            diagnostics.InspectionError,
        ) as error:
            result["errors"].append(
                f"scientific result pack {index.parent.name}: {error}"
            )

    try:
        request, acknowledgement = _read_recording_acknowledgement(env, audit, observer)
        observer.write("validated-recording-acknowledgement.json", acknowledgement)
        result["acknowledgement"] = json.loads(codec.encode_json(acknowledgement))
    except (
        OSError,
        ValueError,
        KeyError,
        RuntimeError,
        diagnostics.InspectionError,
    ) as error:
        result["errors"].append(f"recording acknowledgement unavailable: {error}")
        return result

    retained = d.FrozenEvidenceSetOutput(checkpoints=())
    files: list[d.PackFileInput] = []
    members: list[d.PackMemberSelection] = []

    def selected_file(
        identity: str,
        path: Path,
        role: PackMemberRole,
        format_name: PackFormat,
        prefix: int,
        *,
        scope: str | None = None,
    ) -> None:
        members.append(
            d.PackMemberSelection(
                id=identity,
                role=role,
                format=format_name,
                required=True,
                source=str(path),
                capture_scope=scope,
            )
        )
        files.append(
            d.PackFileInput(
                member=identity,
                path=str(path),
                readable_prefix_bytes=prefix,
            )
        )

    try:
        receipt_path, receipt = _read_review_receipt(request, acknowledgement, observer)
        # The native reader validates contained membership/length and codecs.
        retained = diagnostics.read_handoff(receipt_path)
        observer.write("validated-review-handoff.json", receipt)
        result["handoff"] = json.loads(codec.encode_json(receipt))
        result["handoff_path"] = str(receipt_path)
        selected_file(
            "original_handoff",
            receipt_path,
            "context",
            "json",
            receipt_path.stat().st_size,
        )
        for index, member in enumerate(receipt.members):
            selected_file(
                f"original_context_{index:03d}",
                receipt_path.parent / member.path,
                "context",
                "json",
                member.bytes,
            )
    except (OSError, ValueError, RuntimeError, diagnostics.InspectionError) as error:
        result["errors"].append(f"review handoff unavailable: {error}")

    details, detail_pages, captures = [], [], []
    try:
        detail = diagnostics.read_detail(Path(request.detail.output) / "records.jsonl")
        details.append(detail)
        observer.write("validated-detail.json", detail)
        result["detail"] = json.loads(codec.encode_json(detail))
        offset = 0
        page_bytes = 0
        for _ in range(8):
            page = diagnostics.detail_records(
                Path(detail.records), offset=offset, limit=512, maximum_bytes=2 << 20
            )
            page_bytes += len(codec.encode_json(page))
            _check_byte_allowance(
                page_bytes, 16 << 20, "selected source pages exceed 16 MiB"
            )
            detail_pages.append(page)
            if page.next_offset is None:
                break
            offset = page.next_offset
        result["detail_next_offset"] = page.next_offset
        if page.next_offset is not None:
            result["errors"].append(
                "source analysis page selection ended before original raw detail"
            )
        selected_file(
            "source_records",
            Path(detail.records),
            "detail",
            "json_lines",
            detail.readable_prefix_bytes,
        )
    except (OSError, ValueError, RuntimeError, diagnostics.InspectionError) as error:
        result["errors"].append(f"source detail unavailable: {error}")

    managed_receipt = audit / "math-originals/managed-study-diagnostics.json"
    if managed_receipt.is_file():
        try:
            managed = codec.decode_json(
                _read_bounded(managed_receipt, 16 << 20),
                d.ManagedStudyDiagnosticsDocument,
            )
            result["managed_study"] = json.loads(codec.encode_json(managed))
        except (
            OSError,
            ValueError,
            RuntimeError,
            diagnostics.InspectionError,
        ) as error:
            result["errors"].append(f"managed selection unavailable: {error}")
        else:
            page_bytes = 0
            for ordinal, reference in enumerate(managed.references):
                try:
                    if reference.detail is None:
                        continue  # Original availability/refusal is retained in the receipt.
                    records = managed_records_path(audit, reference.detail.records)
                    # The receipt was frozen while its canonical handle was live.
                    # Re-read actual final writer coverage after receiver drain.
                    detail = diagnostics.read_detail(records)
                    details.append(detail)
                    selected_file(
                        f"managed_source_{ordinal:03d}",
                        records,
                        "detail",
                        "json_lines",
                        detail.readable_prefix_bytes,
                        scope=f"Actual managed attempt {reference.run}/{reference.attempt}; coverage is per writer",
                    )
                    offset = 0
                    next_offset = 0
                    for _ in range(8):
                        page = diagnostics.detail_records(
                            records, offset=offset, limit=256, maximum_bytes=128 << 10
                        )
                        page_bytes += len(codec.encode_json(page))
                        _check_byte_allowance(
                            page_bytes, 16 << 20, "managed source pages exceed 16 MiB"
                        )
                        detail_pages.append(page)
                        next_offset = page.next_offset
                        if next_offset is None:
                            break
                        offset = next_offset
                    if next_offset is not None:
                        result["errors"].append(
                            f"managed source analysis bounded before raw prefix: {reference.run}/{reference.attempt}"
                        )
                except (
                    OSError,
                    ValueError,
                    RuntimeError,
                    diagnostics.InspectionError,
                ) as error:
                    result["errors"].append(
                        f"managed source {reference.run}/{reference.attempt}: {error}"
                    )

    # Original source-only/direct-math bytes are explanatory members, not typed
    # workflow contexts. Keep them in relocated packs as well as the raw bundle.
    sidecar_bytes = 0
    preparation_paths, preparation_errors = preparation_input_members(audit)
    result["errors"].extend(preparation_errors)
    sidecar_paths = [
        *sorted((audit / "math-originals").glob("*.json")),
        *preparation_paths,
    ]
    for index, path in enumerate(sidecar_paths):
        try:
            source_path, size = _original_sidecar(
                path, audit, index, len(members), sidecar_bytes
            )
            selected_file(
                f"original_sidecar_{index:03d}",
                source_path,
                "context",
                "json" if source_path.suffix == ".json" else "text",
                size,
                scope="Original direct mathematical/preparation bytes; no workflow invocation inferred",
            )
            sidecar_bytes += size
        except (OSError, ValueError) as error:
            result["errors"].append(f"original sidecar {path.name}: {error}")
            break

    capture_formats: dict[str, PackFormat] = {
        "Perfetto": "perfetto",
        "PerfData": "profile_binary",
        "Samply": "profile_binary",
        "Symbols": "text",
        "ToolLog": "text",
    }
    capture_paths = sorted(audit.glob("capture-*.json"))
    for path in capture_paths:
        if path.name == "capture-launch-request.json":
            continue
        try:
            capture = codec.decode_json(
                _read_bounded(path, 16 << 20), d.CaptureDocumentOutput
            )
            captures.append(capture)
            for index, output in enumerate(capture.outputs):
                if not output.readable:
                    continue
                format_name = capture_formats[output.kind.value]
                selected_file(
                    f"capture_{len(captures):02d}_{index:02d}",
                    Path(output.path),
                    "capture",
                    format_name,
                    output.readable_prefix_bytes,
                    scope=(
                        f"{capture.kind.value}: {capture.status.value}; "
                        f"original {output.kind.value} bytes ({Path(output.path).suffixes}); "
                        "Samply retains its original gzip JSON encoding; "
                        "loss/clock facts remain in frozen capture metadata"
                    ),
                )
        except (
            OSError,
            ValueError,
            KeyError,
            RuntimeError,
            diagnostics.InspectionError,
        ) as error:
            result["errors"].append(f"capture metadata {path.name}: {error}")
    result["captures"] = [
        json.loads(codec.encode_json(capture)) for capture in captures
    ]

    try:
        checkpoints = {image.process: image for image in retained.checkpoints}
        operations = {operation.id: operation for operation in retained.operations}
        encoded_bytes = 0
        for path in sorted(audit.glob("frozen-evidence-*.json")):
            encoded = _read_bounded(path, (64 << 20) - encoded_bytes)
            encoded_bytes += len(encoded)
            supplied = codec.decode_json(encoded, d.FrozenEvidenceSetOutput)
            checkpoints.update((image.process, image) for image in supplied.checkpoints)
            operations.update(
                (operation.id, operation) for operation in supplied.operations
            )
        in_checkpoints = {
            operation.id
            for image in checkpoints.values()
            for operation in image.operations
        }
        evidence = d.FrozenEvidenceSetOutput(
            checkpoints=tuple(checkpoints.values()),
            operations=tuple(
                operation
                for key, operation in operations.items()
                if key not in in_checkpoints
            ),
            invocations=retained.invocations,
            contexts=retained.contexts,
            inventories=retained.inventories,
            engine_observations=retained.engine_observations,
            records=retained.records,
            details=(*retained.details, *details),
            detail_pages=(*retained.detail_pages, *detail_pages),
            captures=(*retained.captures, *captures),
            query_results=retained.query_results,
        )
        _check_byte_allowance(
            len(codec.encode_json(evidence)),
            64 << 20,
            "joined frozen review context exceeds 64 MiB",
        )
        analysis_request = d.AnalysisRequest(
            operation=acknowledgement.operation, source_items=4096
        )
        observer.write("review-analysis-request.json", analysis_request)
        analysis = analyze_case_evidence(observer, evidence, analysis_request, result)
        table_sources: tuple[tuple[d.ReviewTableKind, tuple[object, ...]], ...] = (
            ("detail", evidence.detail_pages),
            ("invocation", evidence.invocations),
            ("capture", evidence.captures),
            ("engine", evidence.engine_observations),
            ("query", evidence.query_results),
        )
        export_request = d.PackExportRequest(
            files=tuple(files),
            tables=tuple(
                d.PackTableInput(
                    member=f"table-{kind}-{index:03d}",
                    request=d.ReviewTableRequest(
                        kind=kind, index=index, memory_bytes=REVIEW_TABLE_MEMORY_BYTES
                    ),
                )
                for kind, sources in table_sources
                for index in range(len(sources))
            ),
            selection=d.PackSelection(
                members=tuple(members),
                maximum_output_bytes=256 << 20,
                maximum_document_bytes=64 << 20,
                missing_context=(*result["errors"], *result.get("limits", [])),
            ),
        )
        observer.write("review-export-request.json", export_request)
        staging = audit / "review-pack-original"
        index = diagnostics.export_review_pack(
            staging, evidence, analysis, export_request
        )
        # Validate the actual offline index after relocation. Original receipt,
        # members and raw sources remain available independently for inspection.
        destination = audit / "review-pack"
        staging.rename(destination)
        relocated = diagnostics.review_pack(destination)
        observer.write("review-pack-index.json", relocated)
        result["pack"] = {
            "path": str(destination),
            "publication": index.publication,
            "relocated_index": json.loads(codec.encode_json(relocated)),
        }
    except (OSError, ValueError, RuntimeError, diagnostics.InspectionError) as error:
        result["errors"].append(f"offline pack export unavailable: {error}")
    return result


@contextmanager
def _launch_environment(env: dict[str, str]) -> Generator[None, None, None]:
    """The serial collector lends its process environment only during native spawn."""
    previous = dict(os.environ)
    try:
        os.environ.clear()
        os.environ.update(env)
        yield
    finally:
        os.environ.clear()
        os.environ.update(previous)


@contextmanager
def _case_command(
    command: list[str],
    *,
    cwd: Path,
    env: dict[str, str],
    directory: Path,
    observer: Observer,
    capture_from_start: bool,
) -> Generator[tuple[Callable[[], int | None], Callable[[], int]], None, None]:
    if capture_from_start:
        from pse.contracts.documents import LaunchCaptureRequest  # noqa: PLC0415

        # The shell program is constant. Paths and scientific argv stay positional
        # data, and exec retains the launcher's exact process/group ownership.
        request = LaunchCaptureRequest(
            argv=(
                "/bin/sh",
                "-c",
                'exec >"$1" 2>&1; shift; exec "$@"',
                "pse-case-output",
                str((directory / "process.log").resolve()),
                *command,
            ),
            cwd=str(cwd),
            output=str((observer.directory / "capture-cpu").resolve()),
        )
        observer.write("capture-launch-request.json", request)
        with _launch_environment(env):
            child, capture = observer.diagnostics.launch_capture(request)
        observer.captures.append(("cpu", capture))
        try:
            yield child.try_wait, child.wait
        finally:
            # Normal and cooperative-stop paths reach a terminal command first.
            # Only an unexpected orchestration exception cancels this owner.
            child.close()
    else:
        with (
            (directory / "process.log").open("w") as log,
            subprocess.Popen(
                command,
                cwd=cwd,
                env=env,
                stdout=log,
                stderr=subprocess.STDOUT,
                start_new_session=True,
            ) as child,
        ):
            yield child.poll, child.wait


def run_case(
    command: list[str],
    *,
    cwd: Path,
    env: dict[str, str],
    directory: Path,
    deadline_seconds: float | None = None,
    review_artifacts: bool = False,
    capture_from_start: bool = False,
    recording_mode: str = "selected",
) -> dict:
    """Observe one owned child. Stop requests are handled by its actual owners."""
    if deadline_seconds is not None and (
        not math.isfinite(deadline_seconds) or deadline_seconds <= 0
    ):
        raise ValueError("observation deadline must be finite and positive")
    if recording_mode not in {"ordinary", "selected"}:
        raise ValueError("recording mode must be ordinary or selected")
    if recording_mode == "ordinary" and review_artifacts:
        raise ValueError("review artifacts require selected source recording")
    directory = directory.resolve()
    audit = directory / "diagnostics"
    audit.mkdir()
    env = {
        **env,
        "PSE_CASE_AUDIT_DIR": str(audit),
        "PSE_DIAGNOSTICS_DIR": str(audit / "checkpoints"),
    }
    if capture_from_start:
        env["PSE_CASE_CAPTURE_FROM_START"] = "1"
    if review_artifacts:
        env["PSE_CASE_REVIEW_RESULTS"] = "1"
    if recording_mode == "selected":
        env = prepare_source_recording(env, audit, review_artifacts=review_artifacts)
    else:
        env.pop("PSE_DIAGNOSTIC_RECORD_REQUEST", None)
    observer = Observer(
        audit
    )  # Fail before launch if the typed interface is unavailable.
    observer.capture_from_start = capture_from_start
    started = time.monotonic()
    next_read = started
    interrupted = False
    stop_reason = None
    with _case_command(
        command,
        cwd=cwd,
        env=env,
        directory=directory,
        observer=observer,
        capture_from_start=capture_from_start,
    ) as (poll, wait):
        try:
            while True:
                try:
                    if poll() is not None:
                        returncode = wait()
                        break
                    now = time.monotonic()
                    if (
                        stop_reason is None
                        and deadline_seconds is not None
                        and now - started >= deadline_seconds
                    ):
                        stop_reason = "observation deadline reached"
                        (audit / "stop.request").write_text(stop_reason + "\n")
                    if now >= next_read:
                        observer.inspect()
                        next_read = now + (5 if now - started < 30 else 15)
                    time.sleep(0.2)
                except KeyboardInterrupt:
                    interrupted = True
                    if stop_reason is None:
                        stop_reason = "operator interrupted audit"
                        (audit / "stop.request").write_text(stop_reason + "\n")
        finally:
            observation = observer.finish()
    terminal = audit / "audit-outcome.json"
    result = reconcile_outcome(
        returncode,
        json.loads(terminal.read_text()) if terminal.exists() else None,
    )
    if review_artifacts:
        observation["review_artifacts"] = collect_review_artifacts(env, audit, observer)
    result.update(
        observation,
        elapsed_seconds=time.monotonic() - started,
        observation_deadline_seconds=deadline_seconds,
        observer_stop_reason=stop_reason,
        operator_interrupted=interrupted,
        recording_mode=recording_mode,
        capture_from_start=capture_from_start,
    )
    (audit / "observation.json").write_text(json.dumps(result, indent=2) + "\n")
    if interrupted:
        raise KeyboardInterrupt
    return result
