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
from pathlib import Path
import resource
import shutil
import subprocess
import tempfile
import tomllib
import urllib.request
import zipfile

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
        request = urllib.request.Request(spec["url"], headers={"User-Agent": "pse-arrow-diagnostics"})
        with urllib.request.urlopen(request, timeout=60) as response, archive.open("xb") as output:
            while chunk := response.read(1 << 20):
                size += len(chunk)
                if size > 256 << 20:
                    raise ValueError("Trace Processor archive exceeds installation allowance")
                digest.update(chunk)
                output.write(chunk)
        if digest.hexdigest() != spec["sha256"]:
            raise ValueError("Trace Processor release archive checksum differs")
        with zipfile.ZipFile(archive) as bundle:
            members = [info for info in bundle.infolist() if Path(info.filename).name == spec["member"]]
            if len(members) != 1 or members[0].file_size > 256 << 20:
                raise ValueError("Trace Processor binary member missing, ambiguous or oversized")
            staged = Path(scratch) / "tool"
            with bundle.open(members[0]) as source, staged.open("xb") as output:
                shutil.copyfileobj(source, output, 1 << 20)
            staged.chmod(0o755)
            # A racing install may already have completed. Never replace another tool.
            try:
                os.link(staged, target)
            except FileExistsError:
                if staged.read_bytes() != target.read_bytes():
                    raise ValueError("An incompatible tool already occupies the selected installation path") from None
    return target


def query(trace: Path, question: str, output: Path, timeout: float, output_bytes: int, binary: Path | None = None) -> int:
    """Run one declared SQL question, retaining finite output/error prefixes on failure."""
    if not (0 < timeout < float("inf")) or output_bytes < 2:
        raise ValueError("query needs a finite positive clock and output allowance >= 2")
    tool = binary or binary_path()
    if not tool.is_file():
        raise ValueError("Trace Processor unavailable; run `just diagnostic-tools` explicitly")
    sql = ROOT / "scripts/diagnostic_sql" / f"{question}.sql"
    if question not in {"timeline", "numerical", "coverage"}:
        raise ValueError("unknown diagnostic SQL question")
    if not trace.is_file():
        raise ValueError("selected raw trace is unavailable")
    output.mkdir(parents=True, exist_ok=False)
    # RLIMIT_FSIZE applies only to this owned child; each output gets half the allowance.
    def limits() -> None:
        resource.setrlimit(resource.RLIMIT_FSIZE, (output_bytes // 2, output_bytes // 2))
    with (output / "rows.csv").open("xb") as rows, (output / "stderr.txt").open("xb") as errors:
        child = subprocess.Popen([str(tool), "query", "-f", str(sql), str(trace)], stdin=subprocess.DEVNULL, stdout=rows, stderr=errors, preexec_fn=limits)
        try:
            return child.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            child.kill()
            child.wait()  # The exact owned process is joined before returning.
            return 124
        except BaseException:
            if child.poll() is None:
                child.kill()
            child.wait()
            raise


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
    args = parser.parse_args()
    try:
        if args.command == "install":
            print(install())
            return 0
        return query(args.trace, args.question, args.output, args.timeout, args.output_bytes, args.binary)
    except (OSError, ValueError, zipfile.BadZipFile) as error:
        parser.exit(1, f"diagnostic trace unavailable: {error}\n")


if __name__ == "__main__":
    raise SystemExit(main())
