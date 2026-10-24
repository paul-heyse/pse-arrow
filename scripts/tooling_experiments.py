# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Finite, opt-in permission-confined tooling probes; ordinary work uses pse-env."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import re
import shutil
import signal
import stat
import sys
import tempfile
import time
from collections.abc import Callable
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))
from scripts import (  # noqa: E402 -- direct-script path routing
    active_diagnostics,
    resource_store,
    surreal_server,
    surreal_source,
)

SCHEMA = "tooling-experiment-v1"
BUBBLEWRAP_VERSION = "bubblewrap 0.13.0"
WITNESS = b"selected-readonly-fixture\n"
# Positional arguments contain only snapshotted input names, never shell source.
PARSER_LAUNCH = """if (printf 'forbidden-write\\n' > /inputs/readonly-witness) 2>/dev/null; then
    exit 91
fi
IFS= read -r witness < /inputs/readonly-witness
[ "$witness" = selected-readonly-fixture ] || exit 92
printf 'owned-output\\n' > /output/write-witness || exit 93
printf 'namespace-established\\n' > /output/namespace-ready || exit 94
exec /tool/surreal validate "$@"
"""
LIBRARY_PARSER_LAUNCH = PARSER_LAUNCH.replace(
    'exec /tool/surreal validate "$@"', 'exec /tool/parser surrealql-validate "$@"'
)


def capture(
    source: Path,
    target: Path,
    *,
    limit: int | None = None,
    check: Callable[[], None] | None = None,
) -> dict:
    """Freeze selected bytes and refuse a file changing during the capture."""
    descriptor = os.open(source, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC)
    digest = hashlib.sha256()
    with os.fdopen(descriptor, "rb") as original, target.open("xb") as copied:
        target.chmod(0o600)
        before = os.fstat(original.fileno())
        if not stat.S_ISREG(before.st_mode) or (
            limit is not None and before.st_size > limit
        ):
            raise ValueError(
                "selected input must be a regular file within its size bound"
            )
        total = 0
        while chunk := original.read(1 << 20):
            if check is not None:
                check()
            total += len(chunk)
            if limit is not None and total > limit:
                raise ValueError("selected input grew beyond its size bound")
            digest.update(chunk)
            copied.write(chunk)
        after = os.fstat(original.fileno())
        fields = ("st_dev", "st_ino", "st_size", "st_mtime_ns", "st_ctime_ns")
        if any(getattr(before, key) != getattr(after, key) for key in fields):
            raise ValueError("selected file changed during capture")
    return {"source": str(source), "sha256": digest.hexdigest(), "bytes": total}


def admitted_server(receipt_path: Path | None) -> tuple[Path, dict]:
    """Borrow source admission without installing, compiling or starting a store."""
    candidates = (
        [receipt_path.resolve(strict=True)]
        if receipt_path is not None
        else sorted(surreal_server.default_tool_root().glob("*/release.json"))
    )
    accepted = []
    for path in candidates:
        receipt = json.loads(path.read_text())
        if isinstance(receipt, dict) and surreal_source.admitted_receipt(receipt):
            accepted.append((path, receipt))
    if len(accepted) != 1:
        raise ValueError(
            "select exactly one installed server receipt admitted by surreal_source "
            "in this checkout; no installation or unconfined fallback is performed"
        )
    path, receipt = accepted[0]
    binary = Path(str(receipt.get("binary", ""))).resolve(strict=True)
    if not binary.is_file() or surreal_source.digest(binary) != receipt.get(
        "binary_sha256"
    ):
        raise ValueError("installed server binary differs from its admitted receipt")
    return path, receipt


def stage(
    command: list[str], output: Path, label: str, work: Path, deadline: float
) -> dict:
    remaining = deadline - time.monotonic()
    if remaining <= 0:
        return {
            "status": "timed_out",
            "exit_code": None,
            "process": {"owned_drain": "not-started"},
        }
    return active_diagnostics.collect(
        command,
        output,
        label,
        seconds=remaining,
        output_bytes=1 << 20,
        cwd=work,
        environment_remove=tuple(name for name in os.environ if name.startswith("LD_")),
    )


def successful(record: dict) -> bool:
    return (
        record.get("status") == "returned"
        and record.get("exit_code") == 0
        and record.get("process", {}).get("owned_drain") == "complete"
        and not record.get("errors")
    )


def dependencies(text: str, code: int) -> list[Path]:
    """Match actual ldd loader paths, preserving their in-namespace spelling."""
    if "not found" in text or code not in {0, 1}:
        raise ValueError("unresolved parser runtime; namespace execution refused")
    paths = list(
        dict.fromkeys(Path(name) for name in re.findall(r"(?<!\S)(/[^\s]+)", text))
    )
    if not paths and not any(
        marker in text for marker in ("statically linked", "not a dynamic executable")
    ):
        raise ValueError("unrecognized parser runtime dependency observation")
    if any(not path.is_file() for path in paths):
        raise ValueError("observed parser runtime file is unavailable")
    return paths


def namespace_command(
    bwrap: Path,
    tools: dict[str, Path],
    libraries: dict[Path, Path],
    inputs: list[Path],
    witness: Path,
    output: Path,
    *,
    library_parser: bool = False,
) -> list[str]:
    command = [
        str(bwrap),
        "--unshare-user",
        "--unshare-ipc",
        "--unshare-pid",
        "--unshare-net",
        "--unshare-uts",
        "--unshare-cgroup",
        "--disable-userns",
        "--die-with-parent",
        "--new-session",
        "--cap-drop",
        "ALL",
        "--clearenv",
        "--proc",
        "/proc",
        "--remount-ro",
        "/proc",
        "--dev",
        "/dev",
        "--tmpfs",
        "/tmp",
        "--bind",
        str(output),
        "/output",
        "--setenv",
        "HOME",
        "/tmp",
        "--setenv",
        "TMPDIR",
        "/tmp",
        "--setenv",
        "LC_ALL",
        "C",
        "--setenv",
        "PATH",
        "/tool",
    ]
    for name, path in tools.items():
        command += ["--ro-bind", str(path), f"/tool/{name}"]
    for destination, source in libraries.items():
        command += ["--ro-bind", str(source), str(destination)]
    selected = []
    for index, path in enumerate(inputs):
        destination = f"/inputs/input-{index:03d}.surql"
        selected.append(destination)
        command += ["--ro-bind", str(path), destination]
    command += ["--ro-bind", str(witness), "/inputs/readonly-witness"]
    return [
        *command,
        "--remount-ro",
        "/",
        "--chdir",
        "/output",
        "--",
        "/tool/sh",
        "-c",
        LIBRARY_PARSER_LAUNCH if library_parser else PARSER_LAUNCH,
        "surrealql-validate",
        *selected,
    ]


def library_report(text: str, count: int, code: int) -> dict:
    """Keep the selected binary's own metadata and per-file library errors."""
    report = json.loads(text)
    if (
        not isinstance(report, dict)
        or report.get("schema") != "surrealql-validation-v1"
    ):
        raise ValueError(
            "selected parser did not return its structured validation report"
        )
    metadata = report.get("parser")
    if not isinstance(metadata, dict) or (
        metadata.get("library") != "surrealdb-syn"
        or not isinstance(metadata.get("version"), str)
        or not metadata["version"]
        or metadata.get("syntax_profile") != "all-features"
        or metadata.get("scope") != "parse-only"
    ):
        raise ValueError(
            "selected parser did not report the expected parse-only syntax profile"
        )
    files = report.get("files")
    if not isinstance(files, list) or len(files) != count:
        raise ValueError(
            "selected parser file results do not match the captured input population"
        )
    for index, entry in enumerate(files):
        if (
            not isinstance(entry, dict)
            or entry.get("input") != f"/inputs/input-{index:03d}.surql"
            or not isinstance(entry.get("valid"), bool)
        ):
            raise ValueError("selected parser file result has unknown correspondence")
        error = entry.get("error")
        if (entry["valid"] and error is not None) or (
            not entry["valid"]
            and (
                not isinstance(error, dict)
                or error.get("kind") not in {"syntax", "input"}
                or not isinstance(error.get("message"), str)
                or not error["message"]
            )
        ):
            raise ValueError(
                "selected parser did not preserve a per-file validation outcome"
            )
    valid = all(entry["valid"] for entry in files)
    if report.get("valid") is not valid or code != int(not valid):
        raise ValueError(
            "selected parser terminal status contradicts its file outcomes"
        )
    return report


def experiment(
    args: argparse.Namespace,
    result: dict,
    output: Path,
    work: Path,
    cancelled: list[int],
) -> int:
    deadline = time.monotonic() + args.seconds

    def check() -> None:
        if cancelled:
            result["outcome"] = "cancelled"
            result["exit_code"] = 128 + cancelled[0]
            raise ValueError("experiment preparation cancelled")
        if time.monotonic() >= deadline:
            result["outcome"] = "timed_out"
            raise ValueError("original experiment deadline expired")

    check()
    expected_digest = None
    if args.parser_binary is not None:
        binary = args.parser_binary.resolve(strict=True)
        if not binary.is_file() or not binary.stat().st_mode & 0o111:
            raise ValueError("select an executable xtask built with surrealql-tools")
        parser_tool = "parser"
        result["parser"] = {
            "selection": "library-parser",
            "binary": str(binary),
            "syntax_profile": "all-features",
            "scope": "parse-only",
        }
    else:
        receipt_path, receipt = admitted_server(args.server_receipt)
        binary = Path(receipt["binary"])
        expected_digest = receipt["binary_sha256"]
        parser_tool = "surreal"
        result["server"] = {
            "receipt": str(receipt_path),
            "receipt_sha256": surreal_source.digest(receipt_path),
            "effective_key": receipt["effective_key"],
            "version": receipt["version"],
            "binary_sha256": expected_digest,
        }
        result["parser"] = {
            "selection": "server-cli",
            "binary": str(binary),
            "scope": "parse-only",
            "syntax_profile": "server-cli-experimental-grammar-disabled",
        }
        captured_receipt = capture(
            receipt_path, output / "server-receipt.json", check=check
        )
        if (
            captured_receipt["sha256"] != result["server"]["receipt_sha256"]
            or json.loads((output / "server-receipt.json").read_text()) != receipt
        ):
            raise ValueError("admitted server receipt changed during capture")
    bwrap_name = shutil.which("bwrap")
    if bwrap_name is None:
        raise ValueError("Bubblewrap 0.13.0 is unavailable; namespace probe refused")
    bwrap = Path(bwrap_name).resolve(strict=True)
    result["bubblewrap"] = {"path": str(bwrap), "sha256": surreal_source.digest(bwrap)}
    stages = result["stages"]

    def run(command: list[str], label: str) -> dict:
        check()
        observed = stage(command, output, label, work, deadline)
        stages.append({"phase": label, **observed})
        if observed.get("status") == "timed_out":
            result["outcome"] = "timed_out"
            raise ValueError(f"{label}: original experiment deadline expired")
        code = observed.get("exit_code")
        if isinstance(code, int) and code >= 128:
            result["outcome"] = "cancelled"
            result["exit_code"] = code
            raise ValueError(f"{label}: cancelled")
        return observed

    version = run([str(bwrap), "--version"], "bubblewrap-version")
    if (
        not successful(version)
        or (output / "bubblewrap-version.stdout.txt").read_text().strip()
        != BUBBLEWRAP_VERSION
    ):
        raise ValueError("namespace probe requires exactly Bubblewrap 0.13.0")
    result["bubblewrap"]["version"] = BUBBLEWRAP_VERSION
    selected_inputs = []
    for index, source in enumerate(args.input):
        target = work / f"input-{index:03d}.surql"
        observed = capture(
            source.resolve(strict=True), target, limit=16 << 20, check=check
        )
        result["inputs"].append({**observed, "member": target.name})
        selected_inputs.append(target)
    tools = {}
    libraries = {}
    for name, source in ((parser_tool, binary), ("sh", Path("/bin/sh"))):
        source = source.resolve(strict=True)
        target = work / name
        observed = capture(source, target, check=check)
        target.chmod(0o500)
        if name == parser_tool:
            if expected_digest is not None and observed["sha256"] != expected_digest:
                raise ValueError("admitted parser bytes changed during capture")
            result["parser"]["binary_sha256"] = observed["sha256"]
        result["runtime"].append({"kind": name, **observed})
        tools[name] = target
        ldd = run(["/usr/bin/ldd", str(target)], f"{name}-runtime")
        if (
            ldd.get("status") != "returned"
            or ldd.get("process", {}).get("owned_drain") != "complete"
            or ldd.get("errors")
            or ldd.get("truncated")
        ):
            raise ValueError("runtime observation failed or did not drain")
        listing = (output / f"{name}-runtime.stdout.txt").read_text() + (
            output / f"{name}-runtime.stderr.txt"
        ).read_text()
        for destination in dependencies(listing, ldd["exit_code"]):
            if destination in libraries:
                continue
            target = work / f"library-{len(libraries):03d}"
            observed = capture(destination.resolve(strict=True), target, check=check)
            target.chmod(0o500)
            result["runtime"].append(
                {"kind": "library", "destination": str(destination), **observed}
            )
            libraries[destination] = target
    witness = work / "readonly-witness"
    witness.write_bytes(WITNESS)
    payload = output / "payload-output"
    payload.mkdir(mode=0o700)
    observed = run(
        namespace_command(
            bwrap,
            tools,
            libraries,
            selected_inputs,
            witness,
            payload,
            library_parser=args.parser_binary is not None,
        ),
        "parser",
    )
    ready = payload / "namespace-ready"
    confined = ready.is_file() and ready.read_bytes() == b"namespace-established\n"
    result["confinement"] = {
        "namespace_started": confined,
        "readonly_witness_unchanged": witness.read_bytes() == WITNESS,
        "owned_output_witness": (payload / "write-witness").is_file()
        and (payload / "write-witness").read_bytes() == b"owned-output\n",
    }
    if not confined or not all(result["confinement"].values()):
        raise ValueError(
            "namespace setup or permission witness refused; see parser diagnostics"
        )
    if observed.get("status") != "returned" or observed.get("errors"):
        raise ValueError("parser outcome unavailable; see owned process observation")
    code = observed["exit_code"]
    if not isinstance(code, int) or not 0 <= code <= 255:
        raise ValueError("parser has no valid terminal exit status")
    if args.parser_binary is not None:
        if observed.get("truncated"):
            raise ValueError(
                "selected parser report exceeded the retained output bound"
            )
        report = library_report(
            (output / "parser.stdout.txt").read_text(), len(selected_inputs), code
        )
        result["parser"]["reported"] = report["parser"]
        result["validation"] = report
    result["outcome"] = "valid" if code == 0 else "invalid"
    result["exit_code"] = code
    return code


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("case", choices=("surrealql-validate",))
    parser.add_argument("--input", action="append", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    selection = parser.add_mutually_exclusive_group()
    selection.add_argument("--server-receipt", type=Path)
    selection.add_argument(
        "--parser-binary",
        type=Path,
        help="xtask built with surrealql-tools; select all optional grammar",
    )
    parser.add_argument("--seconds", type=float, default=60)
    args = parser.parse_args(argv)
    if (
        not math.isfinite(args.seconds)
        or not 0 < args.seconds <= 300
        or len(args.input) > 32
    ):
        parser.error("select 1..32 inputs and a finite duration in (0,300]")
    output = args.output.absolute()
    result = {
        "schema": SCHEMA,
        "case": args.case,
        "output": str(output),
        "outcome": "unavailable",
        "exit_code": None,
        "stages": [],
        "inputs": [],
        "runtime": [],
        "environment": {
            "default": "owned private directories plus pse-env",
            "selection": "optional Bubblewrap namespace probe",
            "payload": {
                "HOME": "/tmp",
                "TMPDIR": "/tmp",
                "LC_ALL": "C",
                "PATH": "/tool",
            },
            "inherited_payload_environment": False,
            "network": "new isolated network namespace",
            "filesystem": "read-only captured inputs, executable and selected loader files; owned output and private tmpfs",
            "scope": "existing native_operation child of pse-env allocation",
            "qualification": "parse-only; no database, solver or product qualification",
        },
        "cleanup": "not-started",
        "drain": "not-started",
    }
    work = None
    identity = None
    code = 125
    created = False
    cancelled: list[int] = []
    previous_handlers = {}

    def cancel(signum: int, _frame: object) -> None:
        if not cancelled:
            cancelled.append(signum)

    try:
        if ".." in output.parts or any(
            path.is_symlink() for path in (output, *output.parents)
        ):
            raise ValueError("select a new output directory without symlink traversal")
        output.mkdir(mode=0o700)
        created = True
        resource_store.protected(output)
        work = Path(tempfile.mkdtemp(prefix=".probe-", dir=output))
        info = work.stat()
        identity = (info.st_dev, info.st_ino)
        result["private_workspace"] = str(work)
        # A durable initial owner survives interruption before terminal publication.
        resource_store.publish(output / "result.json", result)
        for signum in (signal.SIGINT, signal.SIGTERM):
            previous_handlers[signum] = signal.signal(signum, cancel)
        code = experiment(args, result, output, work, cancelled)
    except (OSError, ValueError, RuntimeError) as error:
        result["error"] = f"{type(error).__name__}: {error}"
        code = (
            result["exit_code"]
            if result["outcome"] == "cancelled"
            else 124
            if result["outcome"] == "timed_out"
            else 125
        )
    finally:
        drains = [
            item.get("process", {}).get("owned_drain", "unobserved")
            for item in result["stages"]
        ]
        result["drain"] = (
            "complete"
            if drains and all(item in {"complete", "not-started"} for item in drains)
            else "unresolved"
            if drains
            else "not-started"
        )
        if work is not None:
            if result["drain"] != "unresolved":
                try:
                    info = work.lstat()
                    if (
                        not stat.S_ISDIR(info.st_mode)
                        or (info.st_dev, info.st_ino) != identity
                    ):
                        raise ValueError(
                            "private workspace identity changed; disposal refused"
                        )
                    shutil.rmtree(work)
                    result["cleanup"] = "complete"
                except (OSError, ValueError) as error:
                    result["cleanup"] = "unresolved"
                    result["cleanup_error"] = str(error)
            else:
                result["cleanup"] = "retained-pending-owned-drain"
        if result["drain"] == "unresolved" or result["cleanup"] in {
            "unresolved",
            "retained-pending-owned-drain",
        }:
            code = 125 if code == 0 else code
        if cancelled:
            result["cancellation_signal"] = cancelled[0]
            code = 128 + cancelled[0]
        result["command_exit_code"] = code
        try:
            if created:
                result["artifacts"] = [
                    {
                        "member": str(path.relative_to(output)),
                        "sha256": surreal_source.digest(path),
                    }
                    for path in sorted(output.rglob("*"))
                    if path.is_file()
                    and not path.is_symlink()
                    and path.name != "result.json"
                    and (work is None or work not in path.parents)
                ]
                resource_store.publish(output / "result.json", result)
        finally:
            for signum, handler in previous_handlers.items():
                signal.signal(signum, handler)
        print(json.dumps(result, sort_keys=True))
    return code


if __name__ == "__main__":
    raise SystemExit(main())
