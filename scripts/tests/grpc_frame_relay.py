"""Bounded opaque loopback relay for an explicitly selected HTTP/2 diagnostic.

Forward HEADERS/DATA without decoding or logging their contents. Record only
GOAWAY/RST_STREAM control metadata and connection lifecycle. This is test-only;
relay-limit closures are labelled separately from peer transport failures.
"""

from __future__ import annotations

import argparse
import json
import selectors
import socket
import time
from dataclasses import dataclass, field
from pathlib import Path

from grpc_frame_probe import ERROR_NAMES

PREFACE = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n"
BUFFER_LIMIT = 256 * 1024
FRAME_LIMIT = 2_000_000
CONNECTION_LIMIT = 16
TOTAL_CONNECTION_LIMIT = 64


class RelayLimit(Exception):
    """An explicit diagnostic limit, not a server transport failure."""


class Receipt:
    def __init__(self, path: Path) -> None:
        self.file = path.open("x", encoding="utf-8")
        self.frames = 0
        self.controls = 0

    def emit(self, event: dict[str, object]) -> None:
        event["monotonic_seconds"] = round(time.monotonic(), 6)
        line = json.dumps(event, sort_keys=True)
        self.file.write(line + "\n")
        self.file.flush()
        print(line, flush=True)

    def frame(
        self, connection: int, direction: str, kind: int, stream: int, data: bytes
    ) -> None:
        self.frames += 1
        if self.frames > FRAME_LIMIT:
            raise RelayLimit("aggregate_frame_limit")
        if kind not in (3, 7):
            return
        self.controls += 1
        if self.controls > 8192:
            raise RelayLimit("control_receipt_limit")
        offset = 4 if kind == 7 else 0
        if len(data) < offset + 4:
            self.emit(
                {
                    "event": "truncated_control",
                    "connection": connection,
                    "direction": direction,
                }
            )
            return
        code = int.from_bytes(data[offset : offset + 4], "big")
        event: dict[str, object] = {
            "event": "control_frame",
            "connection": connection,
            "direction": direction,
            "frame": "GOAWAY" if kind == 7 else "RST_STREAM",
            "stream": stream,
            "error_code": code,
            "error_name": ERROR_NAMES.get(code, "UNKNOWN"),
        }
        if kind == 7:
            event["last_stream"] = int.from_bytes(data[:4], "big") & 0x7FFFFFFF
            event["debug_ascii"] = data[8:264].decode(
                "ascii", errors="backslashreplace"
            )
        self.emit(event)


class Frames:
    def __init__(
        self, receipt: Receipt, connection: int, direction: str, preface: bool
    ) -> None:
        self.receipt = receipt
        self.connection = connection
        self.direction = direction
        self.preface_left = len(PREFACE) if preface else 0
        self.preface_seen = bytearray()
        self.header = bytearray()
        self.remaining = 0
        self.kind = -1
        self.stream = 0
        self.control = bytearray()

    def feed(self, data: bytes) -> None:
        view = memoryview(data)
        while view:
            if self.preface_left:
                count = min(self.preface_left, len(view))
                self.preface_seen.extend(view[:count])
                self.preface_left -= count
                view = view[count:]
                if not self.preface_left and self.preface_seen != PREFACE:
                    raise RelayLimit("unexpected_non_http2_client_preface")
                continue
            if self.kind == -1:
                count = min(9 - len(self.header), len(view))
                self.header.extend(view[:count])
                view = view[count:]
                if len(self.header) != 9:
                    continue
                self.remaining = int.from_bytes(self.header[:3], "big")
                self.kind = self.header[3]
                self.stream = int.from_bytes(self.header[5:], "big") & 0x7FFFFFFF
                self.header.clear()
                self.control.clear()
            count = min(self.remaining, len(view))
            if self.kind in (3, 7):
                self.control.extend(view[: min(count, 264 - len(self.control))])
            view = view[count:]
            self.remaining -= count
            if not self.remaining:
                self.receipt.frame(
                    self.connection,
                    self.direction,
                    self.kind,
                    self.stream,
                    bytes(self.control),
                )
                self.kind = -1


@dataclass
class Side:
    sock: socket.socket
    parser: Frames
    connection: int
    direction: str
    peer: Side | None = None
    outgoing: bytearray = field(default_factory=bytearray)
    read_open: bool = True
    write_closed: bool = False


def relay(
    port: int, upstream: int, seconds: float, byte_limit: int, receipt: Receipt
) -> int:
    selector = selectors.DefaultSelector()
    listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    listener.bind(("127.0.0.1", port))
    listener.listen(CONNECTION_LIMIT)
    listener.setblocking(False)
    selector.register(listener, selectors.EVENT_READ)
    connections: dict[int, tuple[Side, Side]] = {}
    accepted = 0
    wire_bytes = 0
    start = time.monotonic()
    deadline = start + seconds
    last_activity = start
    outcome = "relay_deadline_limit"

    def refresh(side: Side) -> None:
        peer = side.peer
        if peer is None:
            raise RelayLimit("missing_peer")
        events = 0
        if side.read_open and len(peer.outgoing) < BUFFER_LIMIT:
            events |= selectors.EVENT_READ
        if side.outgoing:
            events |= selectors.EVENT_WRITE
        if not peer.read_open and not side.outgoing and not side.write_closed:
            side.sock.shutdown(socket.SHUT_WR)
            side.write_closed = True
        try:
            selector.unregister(side.sock)
        except KeyError:
            pass
        if events:
            selector.register(side.sock, events, side)

    def close(connection: int, reason: str) -> None:
        pair = connections.pop(connection)
        for side in pair:
            try:
                selector.unregister(side.sock)
            except KeyError:
                pass
            side.sock.close()
        receipt.emit(
            {"event": "connection_closed", "connection": connection, "reason": reason}
        )

    receipt.emit(
        {
            "event": "ready",
            "listen": f"127.0.0.1:{port}",
            "upstream": f"127.0.0.1:{upstream}",
            "seconds": seconds,
            "byte_limit": byte_limit,
            "buffer_per_direction": BUFFER_LIMIT,
            "connections_limit": CONNECTION_LIMIT,
            "total_connections_limit": TOTAL_CONNECTION_LIMIT,
            "frame_limit": FRAME_LIMIT,
            "opaque_headers_and_data": True,
        }
    )
    try:
        while time.monotonic() < deadline:
            if accepted and not connections and time.monotonic() - last_activity >= 10:
                outcome = "all_channels_closed_idle"
                break
            for key, mask in selector.select(
                min(0.2, max(0, deadline - time.monotonic()))
            ):
                if key.fileobj is listener:
                    client, _ = listener.accept()
                    if (
                        len(connections) >= CONNECTION_LIMIT
                        or accepted >= TOTAL_CONNECTION_LIMIT
                    ):
                        client.close()
                        raise RelayLimit("connection_count_limit")
                    try:
                        server = socket.create_connection(
                            ("127.0.0.1", upstream), timeout=2
                        )
                    except OSError:
                        client.close()
                        raise
                    accepted += 1
                    client_side = Side(
                        client,
                        Frames(receipt, accepted, "client_to_server", True),
                        accepted,
                        "client_to_server",
                    )
                    server_side = Side(
                        server,
                        Frames(receipt, accepted, "server_to_client", False),
                        accepted,
                        "server_to_client",
                    )
                    client_side.peer = server_side
                    server_side.peer = client_side
                    for side in (client_side, server_side):
                        side.sock.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
                        side.sock.setblocking(False)
                        refresh(side)
                    connections[accepted] = (client_side, server_side)
                    receipt.emit({"event": "connection_opened", "connection": accepted})
                    last_activity = time.monotonic()
                    continue
                side = key.data
                if not isinstance(side, Side) or side.connection not in connections:
                    continue
                peer = side.peer
                if peer is None:
                    raise RelayLimit("missing_peer")
                try:
                    if mask & selectors.EVENT_WRITE:
                        written = side.sock.send(side.outgoing)
                        del side.outgoing[:written]
                    if mask & selectors.EVENT_READ:
                        capacity = BUFFER_LIMIT - len(peer.outgoing)
                        if capacity:
                            data = side.sock.recv(min(65536, capacity))
                            if data:
                                wire_bytes += len(data)
                                if wire_bytes > byte_limit:
                                    raise RelayLimit("aggregate_wire_byte_limit")
                                side.parser.feed(data)
                                peer.outgoing.extend(data)
                                last_activity = time.monotonic()
                            else:
                                side.read_open = False
                                receipt.emit(
                                    {
                                        "event": "peer_eof",
                                        "connection": side.connection,
                                        "direction": side.direction,
                                    }
                                )
                    refresh(side)
                    refresh(peer)
                    if (
                        not side.read_open
                        and not peer.read_open
                        and not side.outgoing
                        and not peer.outgoing
                    ):
                        close(side.connection, "both_peers_eof")
                except BlockingIOError:
                    continue
                except OSError as error:
                    receipt.emit(
                        {
                            "event": "socket_error",
                            "connection": side.connection,
                            "direction": side.direction,
                            "errno": error.errno,
                        }
                    )
                    close(side.connection, "peer_socket_error")
    except RelayLimit as error:
        outcome = str(error)
    except OSError as error:
        outcome = "relay_socket_error"
        receipt.emit({"event": "relay_socket_error", "errno": error.errno})
    finally:
        for connection in list(connections):
            close(connection, "relay_limit_or_shutdown")
        selector.close()
        listener.close()
        receipt.emit(
            {
                "event": "summary",
                "outcome": outcome,
                "accepted_connections": accepted,
                "wire_bytes": wire_bytes,
                "frames": receipt.frames,
                "control_frames": receipt.controls,
                "elapsed_seconds": round(time.monotonic() - start, 3),
            }
        )
        receipt.file.close()
    return int(outcome != "all_channels_closed_idle")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--port", type=int, default=18089)
    parser.add_argument("--upstream-port", type=int, default=18088)
    parser.add_argument("--seconds", type=float, default=180)
    parser.add_argument("--byte-limit", type=int, default=4 * 1024**3)
    parser.add_argument("--receipt", type=Path, required=True)
    args = parser.parse_args()
    if (
        not 1 <= args.port <= 65535
        or not 1 <= args.upstream_port <= 65535
        or args.port == args.upstream_port
    ):
        parser.error("require distinct valid loopback ports")
    if not 0 < args.seconds <= 180 or not 0 < args.byte_limit <= 4 * 1024**3:
        parser.error("require <=180 seconds and <=4 GiB aggregate wire bound")
    return relay(
        args.port,
        args.upstream_port,
        args.seconds,
        args.byte_limit,
        Receipt(args.receipt),
    )


if __name__ == "__main__":
    raise SystemExit(main())
