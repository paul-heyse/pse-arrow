# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Disposable actual-server MX/gRPC controls; never opens the selected catalog.

Run through pse-env with an explicit source-built --binary. The small raw HTTP/2
client is a test fixture: it can abruptly drop a connection without an SDK
session destructor concealing server-side cleanup defects.
"""

from __future__ import annotations

import argparse
import base64
import json
import os
import secrets
import shutil
import socket
import subprocess
import tempfile
import time
import urllib.request
from pathlib import Path


def varint(value: int) -> bytes:
    result = bytearray()
    while value > 127:
        result.append((value & 127) | 128)
        value >>= 7
    result.append(value)
    return bytes(result)


def field(tag: int, value: bytes) -> bytes:
    return varint((tag << 3) | 2) + varint(len(value)) + value


def fields(data: bytes) -> dict[int, list[bytes | int]]:
    """Small bounded protobuf reader for this fixture's pinned control messages."""
    position = 0

    def integer() -> int:
        nonlocal position
        value = 0
        for shift in range(0, 70, 7):
            if position >= len(data):
                raise RuntimeError("Truncated control protobuf integer")
            byte = data[position]
            position += 1
            if shift == 63 and byte > 1:
                raise RuntimeError("Overflowing control protobuf integer")
            value |= (byte & 127) << shift
            if not byte & 128:
                return value
        raise RuntimeError("Malformed control protobuf integer")

    result: dict[int, list[bytes | int]] = {}
    while position < len(data):
        key = integer()
        tag, wire = key >> 3, key & 7
        if tag == 0:
            raise RuntimeError("Invalid control protobuf tag")
        value: bytes | int
        if wire == 0:
            value = integer()
        elif wire in {1, 2, 5}:
            count = integer() if wire == 2 else (8 if wire == 1 else 4)
            value = data[position : position + count]
            if len(value) != count:
                raise RuntimeError("Truncated control protobuf field")
            position += count
        else:
            raise RuntimeError("Unsupported control protobuf wire kind")
        result.setdefault(tag, []).append(value)
    return result


def message(data: dict[int, list[bytes | int]], tag: int) -> bytes:
    values = data.get(tag, [])
    if len(values) != 1 or not isinstance(values[0], bytes):
        raise RuntimeError("Expected one native control protobuf message")
    return values[0]


def context(session: bytes, transaction: bytes | None = None) -> bytes:
    result = field(1, field(1, session))
    if transaction is not None:
        result += field(2, field(1, transaction))
    return field(1, result)


class RawGrpc:
    def __init__(self, port: int) -> None:
        self.socket = socket.create_connection(("127.0.0.1", port), timeout=3)
        self.socket.settimeout(3)
        self.next_stream = 1
        self.socket.sendall(b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n")
        self.frame(4, 0, 0, b"")

    def close(self) -> None:
        self.socket.close()

    def frame(self, kind: int, flags: int, stream: int, data: bytes) -> None:
        self.socket.sendall(
            len(data).to_bytes(3, "big")
            + bytes((kind, flags))
            + stream.to_bytes(4, "big")
            + data
        )

    def receive(self, count: int) -> bytes:
        result = bytearray()
        while len(result) < count:
            chunk = self.socket.recv(count - len(result))
            if not chunk:
                raise RuntimeError("Control socket closed before terminal response")
            result.extend(chunk)
        return bytes(result)

    def responses(self, method: str, request: bytes = b"") -> list[bytes]:
        path = ("/surrealdb.protocol.rpc.v1.SurrealDBService/" + method).encode()
        # HPACK static indices for method POST, scheme http, and literal names.
        # Fixture values are short ASCII and deliberately never use Huffman.
        headers = b"\x83\x86\x04" + bytes((len(path),)) + path
        headers += (
            b"\x01\x09localhost\x0f\x10\x10application/grpc\x00\x02te\x08trailers"
        )
        stream = self.next_stream
        self.next_stream += 2
        self.frame(1, 4, stream, headers)
        self.frame(0, 1, stream, b"\x00" + len(request).to_bytes(4, "big") + request)
        data = bytearray()
        while True:
            header = self.receive(9)
            length = int.from_bytes(header[:3], "big")
            if length > 65536:
                raise RuntimeError("Control response exceeds its finite frame bound")
            kind, flags = header[3:5]
            selected = int.from_bytes(header[5:9], "big")
            payload = self.receive(length)
            if kind == 4 and not flags & 1:
                self.frame(4, 1, 0, b"")
            elif kind == 6 and not flags & 1:
                self.frame(6, 1, 0, payload)
            elif kind == 7:
                raise RuntimeError("Unexpected HTTP/2 GOAWAY in control")
            if selected != stream:
                continue
            if kind == 3:
                raise RuntimeError("Unexpected HTTP/2 reset in control")
            if kind == 0:
                data.extend(payload)
                if len(data) > 65536:
                    raise RuntimeError(
                        "Control response exceeds its finite aggregate bound"
                    )
            if kind in {0, 1} and flags & 1:
                break
        messages = []
        position = 0
        while position < len(data):
            if len(data) - position < 5 or data[position] != 0:
                raise RuntimeError("Malformed gRPC control response framing")
            count = int.from_bytes(data[position + 1 : position + 5], "big")
            position += 5
            payload = bytes(data[position : position + count])
            if len(payload) != count:
                raise RuntimeError("Truncated gRPC control response")
            messages.append(payload)
            position += count
        return messages

    def call(self, method: str, request: bytes = b"") -> bytes | None:
        messages = self.responses(method, request)
        if len(messages) > 1:
            raise RuntimeError("Unary control received multiple protobuf messages")
        return messages[0] if messages else None

    def attach(self, session: bytes | None = None) -> bytes | None:
        request = b"" if session is None else b"\x0a\x14\x0a\x12\x0a\x10" + session
        response = self.call("AttachSession", request)
        if response is None:
            return None
        if len(response) < 20 or response[:4] != b"\x0a\x12\x0a\x10":
            raise RuntimeError("Unexpected attached-session identity encoding")
        return response[4:20]

    def authenticate(self, session: bytes, username: str, password: str) -> None:
        credentials = field(3, username.encode()) + field(4, password.encode())
        if (
            self.call("Signin", context(session) + field(2, field(1, credentials)))
            is None
        ):
            raise RuntimeError("Native held-reader authentication was refused")
        selection = field(2, field(2, b"mx")) + field(3, field(2, b"controls"))
        if self.call("Use", context(session) + selection) is None:
            raise RuntimeError("Native held-reader selection was refused")

    def begin(self, session: bytes) -> bytes:
        response = self.call("BeginTransaction", context(session))
        if response is None:
            raise RuntimeError("Native held-reader transaction was refused")
        identity = message(fields(message(fields(response), 1)), 1)
        if len(identity) != 16:
            raise RuntimeError("Invalid native held-reader transaction identity")
        return identity

    def read_integer(self, session: bytes, transaction: bytes) -> int:
        responses = self.responses(
            "Query",
            context(session, transaction)
            + field(2, b"RETURN (SELECT VALUE value FROM ONLY mx_stats:head);"),
        )
        values: list[bytes | int] = []
        began = ended = False
        for response in responses:
            frame = fields(response)
            if 4 in frame:
                raise RuntimeError("Native held-reader query reported a stream failure")
            if 1 in frame:
                if began:
                    raise RuntimeError("Native held-reader repeated its Begin frame")
                began = True
            if 2 in frame:
                batch = fields(message(frame, 2))
                if 6 in batch:
                    raise RuntimeError("Native held-reader statement failed")
                if 7 in batch:
                    values.extend(fields(message(batch, 7)).get(1, []))
            if 3 in frame:
                ended = True
        if (
            not began
            or not ended
            or len(values) != 1
            or not isinstance(values[0], bytes)
        ):
            raise RuntimeError(
                "Native held-reader result did not complete exactly once"
            )
        value = fields(values[0]).get(4)
        if value is None or len(value) != 1 or not isinstance(value[0], int):
            raise RuntimeError(
                "Native held-reader did not return the expected integer kind"
            )
        return value[0]


def run(binary: Path, strace: Path | None = None) -> None:
    binary = binary.resolve(strict=True)
    with tempfile.TemporaryDirectory(prefix="pse-mx-live-") as scratch:
        state = Path(scratch)
        with socket.socket() as reservation:
            reservation.bind(("127.0.0.1", 0))
            port = reservation.getsockname()[1]
        environment = {
            key: value
            for key, value in os.environ.items()
            if not key.startswith("SURREAL_")
        }
        environment.pop("BASH_ENV", None)
        environment.update(
            SURREAL_USER="mx-control",
            SURREAL_PASS=secrets.token_urlsafe(32),
            SURREAL_GRPC_MAX_MESSAGE_SIZE="4194304",
            SURREAL_GRPC_MAX_ATTACHED_SESSIONS="4",
            SURREAL_RUNTIME_WORKER_THREADS="2",
            TOKIO_WORKER_THREADS="2",
            RAYON_NUM_THREADS="2",
        )
        endpoint = f"grpc://127.0.0.1:{port}"
        uri = (
            f"mem://{state / 'database'}?aol=sync&sync=every&snapshot=31s&versioned=false"
            "&memory_limit=268435456&max_transaction_age=90s&max_async_queue_depth=32&max_async_queue_bytes=16777216"
        )

        def query(sql: str) -> list[object]:
            result = subprocess.run(
                [
                    str(binary),
                    "sql",
                    "--endpoint",
                    endpoint,
                    "--json",
                    "--hide-welcome",
                ],
                input=sql + "\n",
                env=environment,
                capture_output=True,
                text=True,
                timeout=20,
                check=False,
            )
            if result.returncode:
                detail = (result.stdout + result.stderr).replace(
                    environment["SURREAL_PASS"], "[redacted]"
                )
                raise RuntimeError(
                    f"Native gRPC CLI control failed ({result.returncode}): {detail[-1800:]}"
                )
            lines = [
                line.removeprefix("> ").strip() for line in result.stdout.splitlines()
            ]
            payload = next((line for line in lines if line.startswith("[")), None)
            if payload is None:
                raise RuntimeError("Native gRPC CLI did not return its JSON result")
            value = json.loads(payload)
            if not isinstance(value, list):
                raise TypeError("Native gRPC CLI result is not a statement list")
            return value

        metric_names = {
            "live_keys",
            "live_versions",
            "approx_data_bytes",
            "active_readers",
            "oldest_pinned_version",
            "gc_candidate_keys",
            "commit_ring_occupancy",
            "async_queue_depth",
            "async_queue_bytes",
            "async_dropped_batches",
            "poisoned",
        }
        metric_prefix = "surrealdb_storage_surrealmx_"

        def metrics(*, authenticated: bool) -> dict[str, int]:
            request = urllib.request.Request(f"http://127.0.0.1:{port}/metrics")
            if authenticated:
                credentials = (
                    environment["SURREAL_USER"] + ":" + environment["SURREAL_PASS"]
                )
                request.add_header(
                    "Authorization",
                    "Basic " + base64.b64encode(credentials.encode()).decode(),
                )
            with urllib.request.urlopen(request, timeout=3) as response:  # noqa: S310 -- fixture-owned loopback with explicit root auth
                payload = response.read((1 << 20) + 1)
            if len(payload) > 1 << 20:
                raise RuntimeError(
                    "Owned metrics snapshot exceeds its finite probe envelope"
                )
            text = payload.decode()
            if not authenticated and "surrealmx" in text:
                raise RuntimeError(
                    "Unauthenticated metrics exposed storage-engine statistics"
                )
            result = {}
            for line in text.splitlines():
                if not line.startswith(metric_prefix):
                    continue
                name = (
                    line.split("{", 1)[0].split(" ", 1)[0].removeprefix(metric_prefix)
                )
                number = float(line.rsplit(" ", 1)[1])
                if not number.is_integer() or number < 0:
                    raise RuntimeError(
                        "Native engine statistics are not nonnegative integers"
                    )
                result[name] = int(number)
            if (
                authenticated
                and not metric_names - {"oldest_pinned_version"} <= result.keys()
            ):
                raise RuntimeError(
                    "Authenticated metrics lack declared native MX gauges: "
                    + repr(sorted(result))
                )
            return result

        def wait_readers(expected: int) -> dict[str, int]:
            deadline = time.monotonic() + 5
            while True:
                snapshot = metrics(authenticated=True)
                if snapshot["active_readers"] == expected:
                    return snapshot
                if time.monotonic() >= deadline:
                    raise RuntimeError(
                        "Native reader statistics did not physically drain to baseline"
                    )
                time.sleep(0.02)

        def held_reader_statistics() -> None:
            query("USE NS mx DB controls; CREATE ONLY mx_stats:head SET value=0;")
            metrics(authenticated=False)
            baseline = wait_readers(0)
            observations: list[dict[str, object]] = []
            for cancel in (True, False):
                drained: dict[str, int] | None = None
                reader = RawGrpc(port)
                try:
                    session = reader.attach()
                    if session is None:
                        raise RuntimeError("Native held-reader session was refused")
                    reader.authenticate(
                        session,
                        environment["SURREAL_USER"],
                        environment["SURREAL_PASS"],
                    )
                    transaction = reader.begin(session)
                    before_value = reader.read_integer(session, transaction)
                    held = metrics(authenticated=True)
                    if (
                        held["active_readers"] < baseline["active_readers"] + 1
                        or set(held) != metric_names
                    ):
                        raise RuntimeError(
                            "Held native transaction lacks its full11-gauge reader/pin snapshot"
                        )
                    pinned = held["oldest_pinned_version"]
                    statements = ["USE NS mx DB controls;"]
                    statements.extend(["UPDATE ONLY mx_stats:head SET value += 1;"] * 8)
                    statements.append(
                        "RETURN (SELECT VALUE value FROM ONLY mx_stats:head);"
                    )
                    written = query(" ".join(statements))
                    if written[-1] != before_value + 8:
                        raise RuntimeError(
                            "Independent native control writes were not acknowledged: "
                            + repr(written)
                        )
                    grown = metrics(authenticated=True)
                    # GC can reclaim versions and commit entries older than
                    # this reader between snapshots. Require the fixed pin and
                    # the eight new unretired commits, not monotone totals.
                    if (
                        grown["commit_ring_occupancy"] < 8
                        or grown["oldest_pinned_version"] != pinned
                    ):
                        raise RuntimeError(
                            f"Independent acknowledged commits lost their pinned snapshot/ring accounting: held={held}, grown={grown}"
                        )
                    if reader.read_integer(session, transaction) != before_value:
                        raise RuntimeError(
                            "The native transaction did not preserve its original read snapshot"
                        )
                    if cancel:
                        if (
                            reader.call(
                                "CancelTransaction", context(session, transaction)
                            )
                            is None
                        ):
                            raise RuntimeError(
                                "Native reader cancellation was not acknowledged"
                            )
                        # Prove cancellation alone drains the snapshot while
                        # its physical connection and session are still alive.
                        drained = wait_readers(baseline["active_readers"])
                finally:
                    reader.close()  # Exercise physical loss without cancellation in the second case.
                if drained is None:
                    drained = wait_readers(baseline["active_readers"])
                if "oldest_pinned_version" in drained:
                    raise RuntimeError(
                        "An absent reader fabricated or retained an oldest snapshot pin"
                    )
                observations.append(
                    {
                        "release": "cancel" if cancel else "physical_close",
                        "held": held,
                        "writes": grown,
                        "drained": drained,
                    }
                )
                metrics(authenticated=False)
            print(
                "native_mx_statistics="
                + json.dumps(
                    {"baseline": baseline, "observations": observations}, sort_keys=True
                )
            )
            print("authenticated_full_native_mx_metrics_and_public_filter=passed")
            print("held_snapshot_versions_and_cancel_or_crash_reader_drain=passed")

        def start() -> subprocess.Popen[bytes]:
            command = [
                str(binary),
                "start",
                "--no-banner",
                "--no-defaults",
                "--log=warn",
                "--deny-guests",
                "--deny-net",
                "--bind",
                f"127.0.0.1:{port}",
                uri,
            ]
            with (state / "server.log").open("ab") as log:
                process = subprocess.Popen(
                    command,
                    env=environment,
                    stdout=log,
                    stderr=log,
                )

            def wait_ready() -> None:
                deadline = time.monotonic() + 15
                while time.monotonic() < deadline:
                    if process.poll() is not None:
                        raise RuntimeError("Source server exited before readiness")
                    try:
                        with urllib.request.urlopen(
                            f"http://127.0.0.1:{port}/health", timeout=0.2
                        ) as response:
                            if response.status == 200:
                                return
                    except OSError:
                        pass
                    time.sleep(0.05)
                detail = (
                    (state / "server.log")
                    .read_text()[-1800:]
                    .replace(environment["SURREAL_PASS"], "[redacted]")
                )
                raise RuntimeError("Source server readiness timed out: " + detail)

            try:
                wait_ready()
            except BaseException:
                process.kill()
                process.wait()
                raise
            else:
                return process

        def stop(process: subprocess.Popen[bytes]) -> None:
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=20)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()
                    raise RuntimeError(
                        "Source server failed graceful shutdown"
                    ) from None
            if process.returncode != 0:
                raise RuntimeError(
                    f"Source server exited unsuccessfully ({process.returncode})"
                )

        def attach_trace(
            process: subprocess.Popen[bytes], injection: str, name: str
        ) -> subprocess.Popen[bytes]:
            if strace is None:
                raise RuntimeError(
                    "Fault control requires an explicitly selected tracer"
                )
            with (state / "tracer.log").open("ab") as log:
                tracer = subprocess.Popen(
                    [
                        str(strace),
                        "-f",
                        "-qq",
                        "-yy",
                        "-o",
                        str(state / name),
                        "-e",
                        "trace=fsync",
                        "-e",
                        "inject=" + injection,
                        "-p",
                        str(process.pid),
                    ],
                    stdout=log,
                    stderr=log,
                )

            def wait_attached() -> None:
                deadline = time.monotonic() + 5
                while time.monotonic() < deadline:
                    if tracer.poll() is not None:
                        raise RuntimeError(
                            "Explicit tracer could not attach to the owned server"
                        )
                    tasks = list(Path(f"/proc/{process.pid}/task").glob("*/status"))
                    try:
                        attached = tasks and all(
                            f"TracerPid:\t{tracer.pid}\n" in task.read_text()
                            for task in tasks
                        )
                    except FileNotFoundError:
                        attached = False
                    if attached:
                        return
                    time.sleep(0.01)
                raise RuntimeError("Owned server tracer attachment did not complete")

            try:
                wait_attached()
            except BaseException:
                tracer.terminate()
                tracer.wait(timeout=5)
                raise
            else:
                return tracer

        def crash_traced(
            process: subprocess.Popen[bytes], tracer: subprocess.Popen[bytes]
        ) -> None:
            if process.poll() is None:
                process.kill()
                process.wait(timeout=10)
            try:
                tracer.wait(timeout=10)
            except subprocess.TimeoutExpired:
                tracer.kill()
                tracer.wait(timeout=5)
                raise RuntimeError("Fault tracer failed physical drain") from None

        process = start()
        try:
            result = query(
                "DEFINE NAMESPACE mx; USE NS mx; DEFINE DATABASE controls; USE DB controls; CREATE ONLY mx_receipts:ack SET value=41; RETURN (SELECT VALUE value FROM ONLY mx_receipts:ack);"
            )
            if result[-1] != 41:
                raise RuntimeError("Acknowledged control write returned another value")
            print("native_cli_grpc_acknowledged_write=passed")
            held_reader_statistics()
            # A tiny process-global cap makes leaked crash sessions revealing.
            for _ in range(8):
                owner, other = RawGrpc(port), RawGrpc(port)
                try:
                    identities = [owner.attach() for _ in range(4)]
                    if (
                        any(identity is None for identity in identities)
                        or owner.attach() is not None
                    ):
                        raise RuntimeError(
                            "Finite global attached-session cap was not enforced"
                        )
                    if owner.attach(identities[0]) != identities[0]:
                        raise RuntimeError(
                            "Same-connection acknowledged session reuse failed"
                        )
                    if other.attach(identities[0]) is not None:
                        raise RuntimeError(
                            "A different physical connection adopted another session"
                        )
                    owner.close()  # No DetachSession; emulate caller loss.
                    deadline = time.monotonic() + 3
                    while other.attach() is None:
                        if time.monotonic() >= deadline:
                            raise RuntimeError(
                                "Crashed connection sessions were not physically reclaimed"
                            )
                        time.sleep(0.02)
                finally:
                    owner.close()
                    other.close()
                time.sleep(0.03)
            print("connection_crash_cleanup_cap_and_cross_connection_refusal=passed")
            deadline = time.monotonic() + 35
            while not (state / "database/snapshot.bin").is_file():
                if time.monotonic() >= deadline:
                    raise RuntimeError("Periodic snapshot was not physically published")
                time.sleep(0.05)
            print("periodic_snapshot_publication=passed")
        finally:
            stop(process)

        process = start()
        try:
            result = query(
                "USE NS mx DB controls; RETURN (SELECT VALUE value FROM ONLY mx_receipts:ack);"
            )
            if result[-1] != 41:
                raise RuntimeError(
                    "Acknowledged synchronous append did not survive reopen"
                )
            print("sync_everyappend_reopen=passed")
            query(
                'USE NS mx DB controls; BEGIN TRANSACTION; UPDATE ONLY mx_receipts:ack SET value=42; THROW "control rollback"; COMMIT TRANSACTION;'
            )
            result = query(
                "USE NS mx DB controls; RETURN (SELECT VALUE value FROM ONLY mx_receipts:ack);"
            )
            if result[-1] != 41:
                raise RuntimeError(
                    "Failed statement left a partially applied transaction"
                )
            print("failed_transaction_atomicity=passed")
            result = query(
                "USE NS mx DB controls; UPDATE ONLY mx_receipts:ack SET value=43; RETURN (SELECT VALUE value FROM ONLY mx_receipts:ack);"
            )
            if result[-1] != 43:
                raise RuntimeError("Crash control write was not acknowledged")
            process.kill()
            process.wait(timeout=5)
        finally:
            if process.poll() is None:
                stop(process)
        # Emulate an interrupted next frame while the owned process is gone.
        # Existing acknowledged data is before this incomplete header.
        with (state / "database/aol.bin").open("ab") as aol:
            aol.write(b"\x20\x00\x00")
            aol.flush()
            os.fsync(aol.fileno())
        process = start()
        try:
            result = query(
                "USE NS mx DB controls; RETURN (SELECT VALUE value FROM ONLY mx_receipts:ack);"
            )
            if result[-1] != 43:
                raise RuntimeError(
                    "Acknowledged commit lost across crash/torn-tail recovery"
                )
            print("acknowledged_sigkill_and_torn_tail_recovery=passed")
        finally:
            stop(process)

        if strace is not None:
            process = start()
            tracer = None
            try:
                # Attach after genuine readiness: startup and node registration
                # also write synchronously, so counter guesses at process launch
                # do not identify an application commit boundary.
                tracer = attach_trace(
                    process, "fsync:error=EIO:when=1", "fsync-fault.trace"
                )
                result = query(
                    "USE NS mx DB controls; CREATE ONLY mx_receipts:uncertain SET operation='fsync-control',value=44; CREATE ONLY mx_receipts:after_poison SET value=45;"
                )
                if "PSE_MX_COMMIT_UNKNOWN" not in json.dumps(result):
                    raise RuntimeError(
                        "Fsync failure was not reported as an uncertain commit: "
                        + repr(result)
                    )
                trace = (state / "fsync-fault.trace").read_text()
                if not any(
                    "/database/aol.bin>" in line
                    and "EIO" in line
                    and "INJECTED" in line
                    for line in trace.splitlines()
                ):
                    raise RuntimeError(
                        "No injected fsync failure on the owned AOL was observed"
                    )
                # Both statements share the already-authenticated RPC. A new
                # CLI's signin deliberately redacts underlying store failures.
                if "PSE_MX_DATABASE_POISONED" not in json.dumps(result):
                    raise RuntimeError(
                        "Unrecoverable persistence failure did not retain poisoned-store meaning"
                    )
                print("fsync_eio_unknown_commit_and_poison=passed")
            finally:
                # Kill only this tracer's exact server child. Avoid a graceful
                # persistence attempt concealing the injected failure model.
                if tracer is not None:
                    crash_traced(process, tracer)
                elif process.poll() is None:
                    stop(process)
            process = start()
            try:
                result = query(
                    "USE NS mx DB controls; RETURN (SELECT VALUE value FROM ONLY mx_receipts:ack); RETURN (SELECT operation,value FROM ONLY mx_receipts:uncertain);"
                )
                if result[-2] != 43 or result[-1] not in (
                    None,
                    {"operation": "fsync-control", "value": 44},
                ):
                    raise RuntimeError(
                        "Post-fsync recovery lost an acknowledged receipt or changed the original uncertain operation"
                    )
                print("fsync_unknown_original_receipt_settlement=passed")
            finally:
                stop(process)

            previous_snapshot = (state / "database/snapshot.bin").read_bytes()
            process = start()
            tracer = None
            try:
                tracer = attach_trace(
                    process,
                    "fsync:delay_enter=3s:when=1",
                    "snapshot-interruption.trace",
                )
                deadline = time.monotonic() + 40
                while not list(
                    (state / "database").glob(f"snapshot.bin.{process.pid}_*.tmp")
                ):
                    if time.monotonic() >= deadline:
                        raise RuntimeError("No in-progress owned snapshot was observed")
                    time.sleep(0.01)
                if (state / "database/snapshot.bin").read_bytes() != previous_snapshot:
                    raise RuntimeError(
                        "Snapshot was already replaced before interruption"
                    )
                crash_traced(process, tracer)
                tracer = None
                print("snapshot_interrupted_before_atomic_replacement=passed")
            finally:
                if tracer is not None:
                    crash_traced(process, tracer)
                elif process.poll() is None:
                    stop(process)
            process = start()
            try:
                result = query(
                    "USE NS mx DB controls; RETURN (SELECT VALUE value FROM ONLY mx_receipts:ack);"
                )
                if result[-1] != 43:
                    raise RuntimeError(
                        "Interrupted snapshot lost an acknowledged log receipt"
                    )
                print("interrupted_snapshot_and_log_recovery=passed")
            finally:
                stop(process)

        # A separate fresh engine makes exhaustion independent of accumulated
        # recovery data. Keep the authenticated RPC alive across the refusal:
        # a failed new signin deliberately conceals native store diagnostics.
        uri = uri.replace(str(state / "database"), str(state / "capacity"))
        uri = uri.replace("memory_limit=268435456", "memory_limit=8388608")
        process = start()
        try:
            result = query(
                "DEFINE NAMESPACE capacity; USE NS capacity; DEFINE DATABASE controls; USE DB controls; "
                "DEFINE TABLE payload SCHEMALESS; "
                "BEGIN TRANSACTION; "
                "FOR $i IN 0..128 { CREATE type::record('payload', $i) "
                "SET payload=string::repeat('a',262144) RETURN NONE; }; "
                "COMMIT TRANSACTION; "
                "RETURN count(SELECT id FROM payload);"
            )
            if "PSE_MX_MEMORY_LIMIT" not in json.dumps(result):
                raise RuntimeError(
                    "Finite catalog exhaustion did not retain its capacity meaning: "
                    + repr(result)[-1800:]
                )
            # The entire FOR statement is one failed transaction, not a stream
            # of independently acknowledged writes.
            if result[-1] != 0:
                raise RuntimeError(
                    "Capacity refusal published partial transaction data: "
                    + repr(result)[-1800:]
                )
            print("finite_catalog_exhaustion_and_atomic_refusal=passed")
        finally:
            stop(process)

        # A malformed framed-log magic is corruption, not an incomplete tail.
        # Damage only a fresh copy after every process using the source drained.
        corrupt = state / "corrupt"
        shutil.copytree(state / "database", corrupt)
        aol = corrupt / "aol.bin"
        contents = bytearray(aol.read_bytes())
        if contents[:8] != b"\xffSMXAOL\x01":
            raise RuntimeError(
                "Corruption control did not find the native framed-log header"
            )
        contents[1] ^= 1
        aol.write_bytes(contents)
        uri = uri.replace(str(state / "capacity"), str(corrupt))
        uri = uri.replace("memory_limit=8388608", "memory_limit=268435456")
        offset = (state / "server.log").stat().st_size
        try:
            process = start()
        except RuntimeError:
            diagnostic = (state / "server.log").read_bytes()[offset:]
            if b"PSE_MX_CORRUPTED" not in diagnostic:
                raise RuntimeError(
                    "Corrupt-log startup lost its distinct native meaning"
                ) from None
            print("corrupt_framed_log_startup_refuses=passed")
        else:
            stop(process)
            raise RuntimeError("Corrupt framed log was admitted as a ready store")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument(
        "--strace",
        type=Path,
        help="Explicit Linux tracer for fsync EIO and poison controls",
    )
    args = parser.parse_args()
    run(
        args.binary,
        args.strace.resolve(strict=True) if args.strace is not None else None,
    )


if __name__ == "__main__":
    main()
