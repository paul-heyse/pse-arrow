# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Bounded protobuf wire decoding shared by native gRPC controls and diagnostics."""

from __future__ import annotations


def fields(data: bytes) -> dict[int, list[bytes | int]]:
    """Small bounded protobuf reader for pinned gRPC/OTLP control messages."""
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
