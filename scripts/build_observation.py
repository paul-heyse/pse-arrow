# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Transient facts from an actual Cargo child, never build or admission authority."""

from __future__ import annotations

import json
import os
import selectors
import subprocess
import sys
import time
import uuid
from pathlib import Path

from scripts import (
    host_admission,
    operation_lifetime,
    resource_store,
    telemetry_metrics,
)


def processes(pid: int, start: str) -> list[dict]:
    """Best-effort actual descendants; executable names describe observations only."""
    try:
        same = operation_lifetime.start_identity(pid) == start
    except OSError:
        same = False
    if not same:
        return []
    pending, seen, found = [pid], set(), []
    while pending:
        current = pending.pop()
        if current in seen:
            continue
        seen.add(current)
        directory = Path("/proc") / str(current)
        try:
            identity = operation_lifetime.start_identity(current)
            tool = (directory / "comm").read_text().strip()
            children = (directory / "task" / str(current) / "children").read_text()
            pending.extend(int(child) for child in children.split())
            stage = (
                "compiler"
                if tool in {"rustc", "clippy-driver", "rustdoc"}
                else "linker"
                if tool in {"ld", "ld.lld", "lld", "collect2", "rust-lld"}
                else "build-script"
                if tool.startswith("build-script-")
                else "cache-client"
                if tool == "sccache"
                else "cargo"
                if tool == "cargo"
                else "unknown"
            )
            found.append(
                {"pid": current, "start": identity, "tool": tool, "stage": stage}
            )
        except (OSError, ValueError):
            continue
    return found


class BuildObservation:
    def __init__(self, pid: int, environment: dict[str, str]) -> None:
        self.path = (
            host_admission.root_path(environment)
            / "observations"
            / "build"
            / (uuid.uuid4().hex + ".json")
        )
        self.value: dict = {
            "version": 1,
            "attempt": self.path.stem,
            "pid": pid,
            "start": operation_lifetime.start_identity(pid),
            "boot": host_admission.boot(),
            "invocation_declaration": environment.get("PSE_TEST_INVOCATION"),
            "allocation": environment.get(host_admission.MARKER),
            "stage": "unknown",
            "stage_source": "not yet observed",
            "cargo_lock_owner": "unknown",
            "cargo_records": 0,
            "fresh_artifacts": 0,
            "compiled_artifacts": 0,
            "cache": {
                "scope": "selected Cargo environment",
                "wrapper": environment.get("RUSTC_WRAPPER"),
                "hits": "unknown",
                "misses": "unknown",
                "bypass_reason": environment.get(
                    "PSE_COMPILER_CACHE_BYPASS_REASON", "not observed"
                ),
            },
        }
        self.last = 0.0
        self.metric_stage: str | None = None

    def message(self, line: bytes, stream: str) -> None:
        if stream == "stderr" and b"Blocking waiting for file lock" in line:
            self.value.update(
                stage="cargo-lock-wait",
                stage_source="Cargo stderr diagnostic",
                stage_observed_at=time.time(),
            )
            self.publish(force=True)
            return
        try:
            value = json.loads(line)
        except (ValueError, UnicodeError):
            return
        if not isinstance(value, dict):
            return
        reason = value.get("reason")
        if reason not in {
            "compiler-artifact",
            "compiler-message",
            "build-script-executed",
            "build-finished",
        }:
            return
        self.value["cargo_records"] += 1
        telemetry_metrics.host_metrics().build_record(reason, value.get("fresh"))
        self.value["last_cargo_record"] = {
            key: value[key]
            for key in ("reason", "package_id", "target", "fresh", "success")
            if key in value
        }
        if reason == "compiler-artifact" and type(value.get("fresh")) is bool:
            # Cargo freshness is not a cache hit: false may still use sccache.
            self.value[
                "fresh_artifacts" if value["fresh"] else "compiled_artifacts"
            ] += 1
        if reason == "build-finished":
            self.value.update(
                stage="build-finished",
                stage_source="Cargo JSON record",
                stage_observed_at=time.time(),
            )
        self.publish()

    def publish(self, *, force: bool = False) -> None:
        now = time.monotonic()
        if not force and now - self.last < 1:
            return
        try:
            self.value["processes"] = processes(self.value["pid"], self.value["start"])
            for stage in ("linker", "compiler", "build-script", "cache-client"):
                if any(item["stage"] == stage for item in self.value["processes"]):
                    if self.value["stage"] != stage:
                        self.value.update(
                            stage=stage,
                            stage_source="actual Cargo descendant /proc identity",
                            stage_observed_at=time.time(),
                        )
                    break
            for directory in (
                self.path.parents[2],
                self.path.parents[1],
                self.path.parent,
            ):
                host_admission.protected(directory)
            operation_lifetime.write_json(
                self.path, {**self.value, "observed_at": time.time()}
            )
            if self.metric_stage != self.value["stage"]:
                telemetry_metrics.host_metrics().build_stage(self.value["stage"])
                self.metric_stage = self.value["stage"]
            self.last = now
        except (OSError, ValueError):
            pass  # Diagnostics cannot change Cargo's exit or compilation policy.


def run_command(command: list[str], *, env: dict[str, str]) -> int:
    """Preserve both byte streams and the command's exit while observing records."""
    child = subprocess.Popen(
        command, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE
    )
    started = time.monotonic()
    telemetry_metrics.host_metrics().build_started()
    try:
        with child:
            try:
                observation = BuildObservation(child.pid, env)
            except (OSError, ValueError):
                observation = None
            with selectors.DefaultSelector() as selected:
                buffers = {"stdout": b"", "stderr": b""}
                for stream, name in (
                    (child.stdout, "stdout"),
                    (child.stderr, "stderr"),
                ):
                    if stream is None:
                        raise RuntimeError("Cargo output stream was not created")
                    selected.register(stream, selectors.EVENT_READ, name)
                while selected.get_map():
                    for key, _event in selected.select(0.2):
                        name = key.data
                        data = os.read(key.fd, 65536)
                        if not data:
                            selected.unregister(key.fileobj)
                            continue
                        destination = sys.stdout if name == "stdout" else sys.stderr
                        if hasattr(destination, "buffer"):
                            destination.buffer.write(data)
                        else:
                            destination.write(data.decode(errors="replace"))
                        destination.flush()
                        buffers[name] += data
                        while b"\n" in buffers[name]:
                            line, buffers[name] = buffers[name].split(b"\n", 1)
                            if observation is not None:
                                observation.message(line, name)
                        buffers[name] = buffers[name][-1024 * 1024 :]
                    if observation is not None:
                        observation.publish()
            code = child.wait()
            if observation is not None:
                observation.value["exit_code"] = code
                observation.value["status"] = "returned"
                observation.publish(force=True)
            return code
    finally:
        # Popen owns child settlement. An exceptional exit can leave a live child;
        # never claim completion or a terminal duration without a settled exit.
        code = child.poll()
        if code is not None:
            telemetry_metrics.host_metrics().build_finished(
                time.monotonic() - started, code
            )


def observations(directory: Path) -> list[dict]:
    area = directory / "observations" / "build"
    result = []
    for path in sorted(
        area.glob("*.json"), key=lambda path: path.stat().st_mtime, reverse=True
    )[:100]:
        value = resource_store.read(path)
        if value.get("version") != 1 or value.get("attempt") != path.stem:
            raise ValueError("invalid build observation")
        value["age_seconds"] = max(0, time.time() - value["observed_at"])
        value["caller_alive"] = host_admission.caller_alive(value)
        if "stage_observed_at" in value:
            value["stage_age_seconds"] = max(
                0, time.time() - value["stage_observed_at"]
            )
        result.append(value)
    return result
