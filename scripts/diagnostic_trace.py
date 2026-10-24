#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit pinned diagnostic tool installation and finite owned SQL export.

Raw traces stay with the original capture owner. This adapter neither observes
nor controls the target process and never installs tools during a query.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import stat
import tarfile
import tempfile
import tomllib
import urllib.parse
import urllib.request
import zipfile
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Mapping

ROOT = Path(__file__).resolve().parents[1]


def tool_spec(name: str = "trace-processor") -> dict[str, str]:
    with (ROOT / ".config/diagnostic-tools.toml").open("rb") as source:
        return tomllib.load(source)[name]


def binary_path(
    name: str = "trace-processor", environment: Mapping[str, str] | None = None
) -> Path:
    spec = tool_spec(name)
    env = os.environ if environment is None else environment
    home = Path(env.get("HOME", str(Path.home())))
    base = Path(env.get("XDG_DATA_HOME") or home / ".local/share")
    return (
        base
        / "pse/tools"
        / ("perfetto" if name == "trace-processor" else name)
        / spec["version"]
        / spec["member"]
    )


def verify_binary(
    name: str = "trace-processor", environment: Mapping[str, str] | None = None
) -> Path:
    """Verify an installed default without installing or executing it."""
    target = binary_path(name, environment)
    spec = tool_spec(name)
    if target.is_symlink() or not target.is_file():
        raise ValueError(f"{name} managed executable missing or nonregular: {target}")
    if not os.access(target, os.X_OK):
        raise ValueError(f"{name} managed executable is not executable: {target}")
    if spec.get("archive") == "portable-tar":
        verify_bundle(target.parents[1], spec)
    if expected := spec.get("binary_sha256"):
        with target.open("rb") as source:
            observed = hashlib.file_digest(source, "sha256").hexdigest()
        if observed != expected:
            raise ValueError(f"{name} managed executable checksum differs: {target}")
    return target


def verify_bundle(root: Path, spec: dict[str, str]) -> None:
    """Verify the selected immutable layout; presence of bin/rr alone is insufficient."""
    receipt = root / "bundle.json"
    if any(path.is_symlink() for path in (receipt, root, *root.parents)):
        raise ValueError("portable tool installation must not traverse symlinks")
    record = json.loads(receipt.read_text())
    if (
        record.get("archive_sha256") != spec["sha256"]
        or record.get("version") != spec["version"]
    ):
        raise ValueError("portable tool receipt differs from the pinned declaration")
    members = record.get("members", {})
    if not isinstance(members, dict) or not 1 <= len(members) <= 200:
        raise ValueError("invalid portable tool member receipt")
    for name, expected in members.items():
        selected = Path(name)
        if selected.is_absolute() or ".." in selected.parts:
            raise ValueError("invalid portable tool member path")
        path = root / selected
        if (
            any(item.is_symlink() for item in (path, *path.parents))
            or not path.is_file()
        ):
            raise ValueError("portable tool member disappeared or became a symlink")
        with path.open("rb") as stream:
            observed = hashlib.file_digest(stream, "sha256").hexdigest()
        if observed != expected:
            raise ValueError("portable tool member changed")
    for required in (
        spec["member"],
        "lib/rr/librrpreload.so",
        "lib/rr/librrpage.so",
        "bin/rr_exec_stub",
    ):
        if required not in members:
            raise ValueError("portable rr runtime member missing")


def stage_bundle(archive: Path, directory: Path, spec: dict[str, str]) -> Path:
    """Extract only bounded regular files/directories below the declared release prefix."""
    root = directory / "bundle"
    root.mkdir()
    hashes = {}
    with tarfile.open(archive, "r:gz") as bundle:
        names = set()
        expanded = 0
        for count, item in enumerate(bundle, start=1):
            expanded += item.size
            if count > 200 or expanded > 64 << 20:
                raise ValueError("portable tool archive exceeds expanded allowance")
            path = Path(item.name)
            if (
                path.is_absolute()
                or ".." in path.parts
                or not path.parts
                or path.parts[0] != spec["prefix"]
            ):
                raise ValueError(
                    "portable archive member escapes selected release prefix"
                )
            if item.name in names or not (item.isdir() or item.isfile()):
                raise ValueError("duplicate or nonregular portable archive member")
            names.add(item.name)
            relative = Path(*path.parts[1:])
            selected = root / relative
            if item.isdir():
                selected.mkdir(parents=True, exist_ok=True)
                continue
            selected.parent.mkdir(parents=True, exist_ok=True)
            source = bundle.extractfile(item)
            if source is None:
                raise ValueError("portable tool archive member unavailable")
            with source, selected.open("xb") as destination:
                shutil.copyfileobj(source, destination, 1 << 20)
            selected.chmod(0o755 if item.mode & 0o111 else 0o644)
            with selected.open("rb") as stream:
                hashes[str(relative)] = hashlib.file_digest(
                    stream, "sha256"
                ).hexdigest()
    (root / "bundle.json").write_text(
        json.dumps(
            {
                "version": spec["version"],
                "archive_sha256": spec["sha256"],
                "source": spec["url"],
                "members": hashes,
            },
            sort_keys=True,
        )
        + "\n"
    )
    verify_bundle(root, spec)
    return root


def install(
    name: str = "trace-processor", environment: Mapping[str, str] | None = None
) -> Path:
    """Install the declared binary or complete portable runtime from its pinned archive."""
    target = binary_path(name, environment)
    spec = tool_spec(name)
    archive_allowance = int(spec.get("max_archive_bytes", str(256 << 20)))
    binary_allowance = int(spec.get("max_binary_bytes", str(256 << 20)))
    if archive_allowance <= 0 or binary_allowance <= 0:
        raise ValueError("tool installation allowances must be positive")
    portable = spec.get("archive") == "portable-tar"
    selected_root = target.parents[1] if portable else target.parent
    if target.exists() or target.is_symlink():
        return verify_binary(name, environment)
    staging_parent = selected_root.parent if portable else target.parent
    staging_parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=".install-", dir=staging_parent) as scratch:
        archive = Path(scratch) / "release.archive"
        digest = hashlib.sha256()
        size = 0
        if urllib.parse.urlsplit(spec["url"]).scheme != "https":
            raise ValueError("tool release URL must use HTTPS")
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
                if size > archive_allowance:
                    raise ValueError("tool archive exceeds installation allowance")
                digest.update(chunk)
                output.write(chunk)
        if digest.hexdigest() != spec["sha256"]:
            raise ValueError("tool release archive checksum differs")
        if portable:
            staged_root = stage_bundle(archive, Path(scratch), spec)
            try:
                staged_root.rename(selected_root)
            except FileExistsError:
                verify_bundle(selected_root, spec)
            return target
        staged = Path(scratch) / "tool"
        if spec.get("archive") == "tar.gz":
            with tarfile.open(archive, "r:gz") as bundle:
                members = [
                    item
                    for item in bundle.getmembers()
                    if Path(item.name).name == spec["member"]
                ]
                if (
                    len(members) != 1
                    or not members[0].isfile()
                    or members[0].size > binary_allowance
                ):
                    raise ValueError(
                        "tool binary member missing, ambiguous or oversized"
                    )
                source = bundle.extractfile(members[0])
                if source is None:
                    raise ValueError("tool binary member unavailable")
                with source, staged.open("xb") as output:
                    shutil.copyfileobj(source, output, 1 << 20)
        else:
            with zipfile.ZipFile(archive) as bundle:
                members = [
                    info
                    for info in bundle.infolist()
                    if Path(info.filename).name == spec["member"]
                ]
                if (
                    len(members) != 1
                    or members[0].file_size > binary_allowance
                    or members[0].is_dir()
                    or stat.S_IFMT(members[0].external_attr >> 16)
                    not in (0, stat.S_IFREG)
                ):
                    raise ValueError(
                        "tool binary member missing, ambiguous or oversized"
                    )
                with bundle.open(members[0]) as source, staged.open("xb") as output:
                    shutil.copyfileobj(source, output, 1 << 20)
        staged.chmod(0o755)
        if expected := spec.get("binary_sha256"):
            with staged.open("rb") as source:
                observed = hashlib.file_digest(source, "sha256").hexdigest()
            if observed != expected:
                raise ValueError("tool executable checksum differs")
        # A racing install may already have completed. Never replace another tool.
        try:
            os.link(staged, target)
        except FileExistsError:
            verify_binary(name, environment)
            if staged.read_bytes() != target.read_bytes():
                raise ValueError(
                    "An incompatible tool already occupies the selected installation path"
                ) from None
    return verify_binary(name, environment)


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
    commands.add_parser("install").add_argument(
        "tool",
        nargs="?",
        choices=("trace-processor", "mcpls", "rr", "sccache", "otelcol-contrib", "openobserve"),
        default="trace-processor",
    )
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
            print(install(args.tool))
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
    except (OSError, ValueError, zipfile.BadZipFile, tarfile.TarError) as error:
        parser.exit(1, f"diagnostic trace unavailable: {error}\n")


if __name__ == "__main__":
    raise SystemExit(main())
