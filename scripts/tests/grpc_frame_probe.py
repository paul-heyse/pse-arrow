"""Bounded, unauthenticated loopback HTTP/2 control-frame diagnostic.

Uses the pinned SurrealDB GetCapabilities unary RPC with an empty protobuf request.
This is a transport probe, not an SDK query or scientific qualification. The two
legal request encodings differ only in whether END_STREAM has its own empty DATA
frame. Response header blocks remain opaque; no gRPC success claim is inferred.

Wire references: RFC 9113 (frames), RFC 7541 (literal HPACK), and
grpc/grpc doc/PROTOCOL-HTTP2.md. No packet privileges or third-party modules needed.
"""

from __future__ import annotations

import argparse
import ipaddress
import json
import socket
import struct
import time

RPC_PATH = "/surrealdb.protocol.rpc.v1.SurrealDBService/GetCapabilities"
ERROR_NAMES = {
    0: "NO_ERROR",
    1: "PROTOCOL_ERROR",
    2: "INTERNAL_ERROR",
    3: "FLOW_CONTROL_ERROR",
    4: "SETTINGS_TIMEOUT",
    5: "STREAM_CLOSED",
    6: "FRAME_SIZE_ERROR",
    7: "REFUSED_STREAM",
    8: "CANCEL",
    9: "COMPRESSION_ERROR",
    10: "CONNECT_ERROR",
    11: "ENHANCE_YOUR_CALM",
    12: "INADEQUATE_SECURITY",
    13: "HTTP_1_1_REQUIRED",
}


class ProbeError(Exception):
    """A diagnostic bound or unexpected wire condition was reached."""


def integer(value: int, prefix: int, first: int = 0) -> bytes:
    ceiling = (1 << prefix) - 1
    if value < ceiling:
        return bytes((first | value,))
    encoded = bytearray((first | ceiling,))
    value -= ceiling
    while value >= 128:
        encoded.append((value & 127) | 128)
        value >>= 7
    encoded.append(value)
    return bytes(encoded)


def string(value: str) -> bytes:
    encoded = value.encode("ascii")
    return integer(len(encoded), 7) + encoded


def frame(kind: int, flags: int, stream: int, payload: bytes = b"") -> bytes:
    return (
        len(payload).to_bytes(3, "big")
        + bytes((kind, flags))
        + struct.pack("!I", stream)
        + payload
    )


def headers(authority: str) -> bytes:
    # RFC 7541 static indices: POST=3, http=6, path=4, authority=1,
    # content-type=31. Literal fields use neither indexing nor Huffman coding.
    return (
        b"\x83\x86"
        + integer(4, 4)
        + string(RPC_PATH)
        + integer(1, 4)
        + string(authority)
        + integer(31, 4)
        + string("application/grpc")
        + b"\x00"
        + string("te")
        + string("trailers")
    )


class Channel:
    def __init__(self, host: str, port: int, deadline: float, byte_limit: int) -> None:
        self.deadline = deadline
        self.byte_limit = byte_limit
        self.received = 0
        self.sent = 0
        self.sock = socket.create_connection((host, port), timeout=self.remaining())
        self.sock.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)

    def remaining(self) -> float:
        remaining = self.deadline - time.monotonic()
        if remaining <= 0:
            raise ProbeError("original probe deadline reached")
        return remaining

    def send(self, payload: bytes) -> None:
        if self.received + self.sent + len(payload) > self.byte_limit:
            raise ProbeError("wire byte bound reached")
        self.sock.settimeout(self.remaining())
        self.sock.sendall(payload)
        self.sent += len(payload)

    def read(self, count: int) -> bytes:
        if self.received + self.sent + count > self.byte_limit:
            raise ProbeError("wire byte bound reached")
        result = bytearray()
        while len(result) < count:
            self.sock.settimeout(self.remaining())
            chunk = self.sock.recv(count - len(result))
            if not chunk:
                raise ProbeError("peer closed TCP connection")
            result.extend(chunk)
            self.received += len(chunk)
        return bytes(result)

    def receive(self) -> tuple[int, int, int, bytes]:
        head = self.read(9)
        length = int.from_bytes(head[:3], "big")
        if length > 16384:
            raise ProbeError("peer exceeded advertised default frame size")
        return (
            head[3],
            head[4],
            int.from_bytes(head[5:], "big") & 0x7FFFFFFF,
            self.read(length),
        )


def probe(
    host: str,
    port: int,
    requests: int,
    concurrency: int,
    separate_final: bool,
    deadline: float,
    byte_limit: int,
) -> dict[str, object]:
    started = time.monotonic()
    result: dict[str, object] = {
        "rpc": RPC_PATH,
        "concurrency": concurrency,
        "request_encoding": "separate_empty_final"
        if separate_final
        else "message_final",
        "requests_sent": 0,
        "response_streams_closed": 0,
        "responses_with_grpc_message": 0,
        "response_headers_decoded": False,
        "control_frames": [],
    }
    controls: list[dict[str, object]] = []
    result["control_frames"] = controls
    channel: Channel | None = None
    sent = 0
    closed = 0
    messages = 0
    frame_count = 0
    try:
        channel = Channel(host, port, deadline, byte_limit)
        channel.send(b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n" + frame(4, 0, 0))
        block = headers(f"{host}:{port}")
        active: dict[int, bytearray] = {}
        while closed < requests:
            while sent < requests and len(active) < concurrency:
                stream = 2 * sent + 1
                active[stream] = bytearray()
                channel.send(frame(1, 4, stream, block))
                channel.send(frame(0, 0 if separate_final else 1, stream, b"\0" * 5))
                if separate_final:
                    channel.send(frame(0, 1, stream))
                sent += 1
            kind, flags, stream, payload = channel.receive()
            frame_count += 1
            if frame_count > 64 * requests + 1024:
                raise ProbeError("received frame count bound reached")
            if kind == 4 and not flags & 1:
                channel.send(frame(4, 1, 0))
            elif kind == 6 and not flags & 1:
                channel.send(frame(6, 1, 0, payload))
            elif kind in (3, 7):
                offset = 4 if kind == 7 else 0
                if len(payload) < offset + 4:
                    raise ProbeError("truncated reset/goaway frame")
                code = int.from_bytes(payload[offset : offset + 4], "big")
                control: dict[str, object] = {
                    "direction": "server_to_probe",
                    "frame": "GOAWAY" if kind == 7 else "RST_STREAM",
                    "stream": stream,
                    "error_code": code,
                    "error_name": ERROR_NAMES.get(code, "UNKNOWN"),
                }
                if kind == 7:
                    control["last_stream"] = (
                        int.from_bytes(payload[:4], "big") & 0x7FFFFFFF
                    )
                    control["debug_ascii"] = payload[8:264].decode(
                        "ascii", errors="backslashreplace"
                    )
                controls.append(control)
                raise ProbeError("peer emitted " + str(control["frame"]))
            if kind == 0:
                if stream not in active:
                    raise ProbeError("DATA for an unowned stream")
                data = payload
                if flags & 8:
                    if not payload or payload[0] >= len(payload):
                        raise ProbeError("invalid DATA padding")
                    data = payload[1 : len(payload) - payload[0]]
                if len(active[stream]) + len(data) > 65536:
                    raise ProbeError("per-response byte bound reached")
                active[stream].extend(data)
                if payload:
                    increment = struct.pack("!I", len(payload))
                    channel.send(frame(8, 0, 0, increment))
                    if not flags & 1:
                        channel.send(frame(8, 0, stream, increment))
            if kind in (0, 1) and flags & 1:
                if stream not in active:
                    raise ProbeError("END_STREAM for an unowned stream")
                body = active.pop(stream)
                if len(body) >= 5 and len(body) >= 5 + int.from_bytes(body[1:5], "big"):
                    messages += 1
                closed += 1
        result["outcome"] = "bounded_requests_drained"
    except (ProbeError, OSError) as error:
        result["outcome"] = "stopped"
        result["error"] = str(error)
    finally:
        result["requests_sent"] = sent
        result["response_streams_closed"] = closed
        result["responses_with_grpc_message"] = messages
        result["frames_received"] = frame_count
        result["elapsed_seconds"] = round(time.monotonic() - started, 3)
        if channel is not None:
            result["wire_bytes_sent"] = channel.sent
            result["wire_bytes_received"] = channel.received
            channel.sock.close()
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=18088)
    parser.add_argument("--requests", type=int, default=1024)
    parser.add_argument("--seconds", type=float, default=45)
    parser.add_argument("--byte-limit", type=int, default=32 * 1024 * 1024)
    args = parser.parse_args()
    if not ipaddress.ip_address(args.host).is_loopback:
        parser.error("only literal loopback addresses are permitted")
    if not 1 <= args.port <= 65535 or not 100 <= args.requests <= 4096:
        parser.error("require valid port and 100..4096 requests per case")
    if not 0 < args.seconds <= 60 or not 1024 <= args.byte_limit <= 64 * 1024 * 1024:
        parser.error("require deadline <=60 seconds and byte bound <=64 MiB")
    deadline = time.monotonic() + args.seconds
    remaining_bytes = args.byte_limit
    failed = False
    for concurrency in (1, 16):
        for separate_final in (False, True):
            result = probe(
                args.host,
                args.port,
                args.requests,
                concurrency,
                separate_final,
                deadline,
                remaining_bytes,
            )
            print(json.dumps(result, sort_keys=True), flush=True)
            remaining_bytes -= int(str(result.get("wire_bytes_sent", 0)))
            remaining_bytes -= int(str(result.get("wire_bytes_received", 0)))
            failed |= result["outcome"] != "bounded_requests_drained"
    return int(failed)


if __name__ == "__main__":
    raise SystemExit(main())
