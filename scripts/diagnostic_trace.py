#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit pinned Trace Processor installation and finite owned SQL export.

Raw traces stay with the original capture owner. This adapter neither observes
nor controls the target process and never installs tools during a query.
"""

from __future__ import annotations

import argparse
import hashlib
import os
import shutil
import tempfile
import tomllib
import urllib.parse
import urllib.request
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def tool_spec() -> dict[str, str]:
    with (ROOT / ".config/diagnostic-tools.toml").open("rb") as source:
        return tomllib.load(source)["trace-processor"]


def binary_path() -> Path:
    spec = tool_spec()
    base = Path(os.environ.get("XDG_DATA_HOME", Path.home() / ".local/share"))
    return base / "pse/tools/perfetto" / spec["version"] / "trace_processor_shell"


def install() -> Path:
    """Install only the declared binary from a bounded authenticated release archive."""
    target = binary_path()
    if target.is_file():
        return target
    spec = tool_spec()
    target.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=".install-", dir=target.parent) as scratch:
        archive = Path(scratch) / "release.zip"
        digest = hashlib.sha256()
        size = 0
        if urllib.parse.urlsplit(spec["url"]).scheme != "https":
            raise ValueError("Trace Processor release URL must use HTTPS")
        # The declaration selects HTTPS; the bounded archive digest authenticates bytes.
        request = urllib.request.Request(  # noqa: S310
            spec["url"], headers={"User-Agent": "pse-arrow-diagnostics"}
        )
        with (
            urllib.request.urlopen(request, timeout=60) as response,  # noqa: S310
            archive.open("xb") as output,
        ):
            while chunk := response.read(1 << 20):
                size += len(chunk)
                if size > 256 << 20:
                    raise ValueError(
                        "Trace Processor archive exceeds installation allowance"
                    )
                digest.update(chunk)
                output.write(chunk)
        if digest.hexdigest() != spec["sha256"]:
            raise ValueError("Trace Processor release archive checksum differs")
        with zipfile.ZipFile(archive) as bundle:
            members = [
                info
                for info in bundle.infolist()
                if Path(info.filename).name == spec["member"]
            ]
            if len(members) != 1 or members[0].file_size > 256 << 20:
                raise ValueError(
                    "Trace Processor binary member missing, ambiguous or oversized"
                )
            staged = Path(scratch) / "tool"
            with bundle.open(members[0]) as source, staged.open("xb") as output:
                shutil.copyfileobj(source, output, 1 << 20)
            staged.chmod(0o755)
            # A racing install may already have completed. Never replace another tool.
            try:
                os.link(staged, target)
            except FileExistsError:
                if staged.read_bytes() != target.read_bytes():
                    raise ValueError(
                        "An incompatible tool already occupies the selected installation path"
                    ) from None
    return target


def query(
    trace: Path,
    question: str,
    output: Path,
    timeout: float,
    output_bytes: int,
    binary: Path | None = None,
    *,
    request_file: Path | None = None,
) -> int:
    """Compose the shared typed query product; retain every actual failure prefix."""
    from pse import (  # noqa: PLC0415 -- install needs no native extension
        InspectionError,
        codec,
    )
    from pse.contracts.documents import (  # noqa: PLC0415 -- install needs no native extension
        ReviewQuery,
        ReviewQueryRequest,
    )
    from pse.diagnostics import (  # noqa: PLC0415 -- install needs no native extension
        query_trace,
    )

    if request_file is None:
        request = ReviewQueryRequest(
            version=1,
            input_member=str(trace),
            question=codec.decode_json(codec.encode_json(question), ReviewQuery),
            clock_scope=f"capture:{trace.name}",
            interval=None,
            timeout_seconds=timeout,
            output_bytes=output_bytes,
        )
    else:
        with request_file.open("rb") as source:
            encoded = source.read((1 << 20) + 1)
        if len(encoded) > 1 << 20:
            raise ValueError("query request exceeds protocol allowance")
        request = codec.decode_json(encoded, ReviewQueryRequest)
    try:
        result = query_trace(request, trace, output, binary or binary_path())
    except InspectionError as error:
        raise ValueError(str(error)) from error
    if result.status == "completed":
        return 0
    if result.status == "timed_out":
        return 124
    return result.exit_code or 1


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("install")
    inspect = commands.add_parser("query")
    inspect.add_argument("trace", type=Path)
    inspect.add_argument("question", choices=("timeline", "numerical", "coverage"))
    inspect.add_argument("--output", type=Path, required=True)
    inspect.add_argument("--timeout", type=float, default=60.0)
    inspect.add_argument("--output-bytes", type=int, default=16 << 20)
    inspect.add_argument("--binary", type=Path)
    inspect.add_argument(
        "--request",
        type=Path,
        help="explicit ReviewQueryRequest JSON overrides leaf query parameters",
    )
    args = parser.parse_args()
    try:
        if args.command == "install":
            print(install())
            return 0
        return query(
            args.trace,
            args.question,
            args.output,
            args.timeout,
            args.output_bytes,
            args.binary,
            request_file=args.request,
        )
    except (OSError, ValueError, zipfile.BadZipFile) as error:
        parser.exit(1, f"diagnostic trace unavailable: {error}\n")


if __name__ == "__main__":
    raise SystemExit(main())
