# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Once-only case observation; scientific ownership remains in the case process."""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import time
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pse.diagnostics import CaptureSession


class Observer:
    """Keep typed inspection windows and explicitly selected optional captures."""

    def __init__(self, directory: Path) -> None:
        # Ordinary benchmark collection does not import the native extension.
        from pse import codec, diagnostics  # noqa: PLC0415
        from pse.contracts.documents import (  # noqa: PLC0415
            CaptureRequest,
            ClosedDuration,
            DiagnosticListRequest,
            InspectionRequest,
        )
        from pse.contracts.enums import CaptureKind  # noqa: PLC0415

        self.directory = directory
        self.codec = codec
        self.diagnostics = diagnostics
        self.request = InspectionRequest
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

    def write(self, name: str, document: object) -> None:
        (self.directory / name).write_bytes(self.codec.encode_json(document))

    def inspect(self, *, terminal: bool = False) -> None:
        locator = self.directory / "case-operation.json"
        if not locator.exists():
            return
        try:
            target = json.loads(locator.read_text())
            checkpoint = Path(target["checkpoint"])
            root = self.diagnostics.operation_id(target["operation"])
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
                document = self.diagnostics.inspect_checkpoint(
                    checkpoint,
                    self.request(operation=operation, related_limit=32),
                )
                documents.append(document)
                pending.extend(
                    related.id
                    for related in document.related
                    if related.outcome is None
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
                        document = self.diagnostics.inspect_checkpoint(
                            checkpoint,
                            self.request(operation=operation.id, related_limit=32),
                        )
                        documents.append(document)
                        self.write(
                            f"process-inspection-{self.reads:05d}-{len(documents):02d}.json",
                            document,
                        )
                # These are actual same-process owners, not invented parent links.
            if checkpoint.exists():
                shutil.copyfile(
                    checkpoint, self.directory / f"checkpoint-{self.reads:05d}.json"
                )
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
            self.inspect_managed()
        except (
            OSError,
            ValueError,
            KeyError,
            RuntimeError,
            self.diagnostics.InspectionError,
        ) as error:
            self.failures.append(f"inspection unavailable: {error}")

    def inspect_managed(self) -> None:
        """Select registered receiver identity; absence never becomes caller zero."""
        target_path = self.directory / "managed-target.json"
        if not target_path.exists():
            return
        try:
            target = json.loads(target_path.read_text())
            marker_path = Path(target["state"]) / "primary-receiver.json"
            if marker_path.exists():
                marker = json.loads(marker_path.read_text())
                if (
                    marker.get("ready")
                    and marker.get("canonical_database") == target["database"]
                ):
                    pid = int(marker["pid"])
                    start = int(
                        Path(f"/proc/{pid}/stat")
                        .read_text()
                        .rsplit(")", 1)[1]
                        .split()[19]
                    )
                    cursor = None
                    # Read-only default discovery includes older/dead prefixes.
                    # Only this exact registered live receiver can match.
                    for _ in range(4):
                        default_directory = (
                            Path(
                                os.environ.get(
                                    "XDG_STATE_HOME", str(Path.home() / ".local/state")
                                )
                            )
                            / "pse/diagnostics"
                        )
                        page = self.diagnostics.list_processes(
                            self.list_request(after=cursor, limit=32),
                            directory=default_directory,
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
                            self.managed_operations = [
                                operation.id
                                for operation in match.operations
                                if operation.outcome is None
                            ][:8]
                            (self.directory / "managed-association.json").write_text(
                                json.dumps(
                                    {
                                        **target,
                                        "pid": pid,
                                        "start_ticks": start,
                                        "checkpoint": match.checkpoint,
                                        "partial": match.partial,
                                    },
                                    indent=2,
                                )
                                + "\n"
                            )
                            break
                        cursor = page.next
                        if cursor is None:
                            break
            if self.managed_checkpoint is not None:
                for index, operation in enumerate(self.managed_operations):
                    document = self.diagnostics.inspect_checkpoint(
                        self.managed_checkpoint,
                        self.request(operation=operation, related_limit=32),
                    )
                    self.write(
                        f"managed-inspection-{self.reads:05d}-{index:02d}.json",
                        document,
                    )
                if self.managed_checkpoint.exists():
                    shutil.copyfile(
                        self.managed_checkpoint,
                        self.directory / f"managed-checkpoint-{self.reads:05d}.json",
                    )
        except (
            OSError,
            ValueError,
            KeyError,
            RuntimeError,
            self.diagnostics.InspectionError,
        ) as error:
            self.failures.append(f"managed receiver observation unavailable: {error}")

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
