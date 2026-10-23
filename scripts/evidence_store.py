# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit evidence preparation and bounded use of an already-built native tool."""

from __future__ import annotations

import argparse
import json
import math
import os
import secrets
import stat
import subprocess
import sys
import tempfile
import time
from pathlib import Path

from scripts import evidence_projection, host_admission, resource_store, surreal_server

ROOT = Path(__file__).resolve().parents[1]


def prepare(state: Path, *, deadline: float) -> dict[str, object]:
    """Prepare only private tooling configuration; database DDL is separate init."""
    with surreal_server.tooling_activity(state, deadline=deadline) as (service, _activity):
        directory = state / ".tooling" / "evidence"
        host_admission.protected(directory)
        config = {
            "version": 1,
            "service_instance_id": service["instance_id"],
            "endpoint": service["endpoint"],
            "namespace": "pse_tooling",
            "database": "evidence",
            "credentials_file": "credentials.json",
        }
        target = directory / "config.json"
        credentials = directory / "credentials.json"
        with surreal_server.state_lock(state, deadline=deadline):
            if target.exists():
                if resource_store.read(target) != config:
                    raise ValueError("Existing tooling selection differs; explicit fresh selection is required")
                validate_credentials(credentials)
                return {"status": "prepared", "config": str(target), **config}
            if credentials.exists():
                # A previous preparation can have durably written credentials first.
                validate_credentials(credentials)
            else:
                resource_store.publish(credentials, {
                    "reader_username": "pse_evidence_reader",
                    "reader_password": secrets.token_urlsafe(36),
                    "importer_username": "pse_evidence_importer",
                    "importer_password": secrets.token_urlsafe(36),
                })
            resource_store.publish(target, config)
        return {"status": "prepared", "config": str(target), **config}


def validate_credentials(path: Path) -> None:
    value = resource_store.read(path)
    if set(value) != {"reader_username", "reader_password", "importer_username", "importer_password"} or any(
        not isinstance(item, str) or not item for item in value.values()
    ):
        raise ValueError("Invalid private tooling credentials")


def selected_configuration(state: Path, service: dict[str, object]) -> None:
    """Require an already-prepared exact selection; observation never installs it."""
    directory = state / ".tooling" / "evidence"
    for path in (state / ".tooling", directory):
        information = path.lstat()
        if not stat.S_ISDIR(information.st_mode) or information.st_uid != os.getuid() or information.st_mode & 0o077:
            raise ValueError("Tooling configuration directories must be private and owned")
    value = resource_store.read(directory / "config.json")
    if value != {
        "version": 1, "service_instance_id": service["instance_id"],
        "endpoint": service["endpoint"], "namespace": "pse_tooling",
        "database": "evidence", "credentials_file": "credentials.json",
    }:
        raise ValueError("Tooling selection differs from selected service identity")
    validate_credentials(directory / "credentials.json")


def run_native(state: Path, tool: Path, arguments: list[str], *, deadline: float) -> int:
    """Run only the explicit built tool; never install, compile or invoke a shell."""
    information = tool.stat()
    if not stat.S_ISREG(information.st_mode) or not os.access(tool, os.X_OK):
        raise ValueError("Evidence tool is unavailable; run just evidence-build explicitly")
    with surreal_server.tooling_activity(state, deadline=deadline) as (service, activity):
        selected_configuration(state, service)
        if not surreal_server.owns_listener(state, service, deadline=deadline):
            raise ValueError("Selected tooling service listener ownership is unavailable; inspect its existing service owner")
        milliseconds = max(1, int(surreal_server.remaining(deadline) * 1000))
        command = [str(tool), "evidence", "--state", str(state), "--timeout-ms", str(milliseconds), *arguments]
        environment = {
            key: value for key, value in os.environ.items()
            if key not in {"SURREAL_USER", "SURREAL_PASS"}
        }
        environment.update(PSE_TOOLING_ACTIVITY_FD=str(activity), PSE_TOOLING_SERVICE_INSTANCE_ID=str(service["instance_id"]))
        # The child retains its exact kernel lifetime if its wrapper is killed.
        completed = subprocess.run(command, check=False, timeout=surreal_server.remaining(deadline), pass_fds=(activity,), env=environment)
        return completed.returncode


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser(description=__doc__)
    result.add_argument("--state", type=Path, default=os.environ.get("PSE_SURREAL_STATE"))
    result.add_argument("--tool", type=Path, default=ROOT / "target/debug/pse-evidence")
    result.add_argument("--timeout", type=float, default=30)
    commands = result.add_subparsers(dest="command", required=True)
    for command in ("prepare", "init", "status", "rebuild", "drop"):
        commands.add_parser(command)
    for command in ("snapshot", "import"):
        selected = commands.add_parser(command)
        selected.add_argument("invocation")
        selected.add_argument("--registry", type=Path)
    query = commands.add_parser("query")
    query.add_argument("--limit", type=int)
    query.add_argument("--kind")
    query.add_argument("--status")
    query.add_argument("--cursor", type=Path)
    query.add_argument("--generation")
    generations = commands.add_parser("generations")
    generations.add_argument("--limit", type=int)
    generations.add_argument("--after")
    return result


def main(argv: list[str] | None = None) -> int:
    cli = parser()
    args = cli.parse_args(argv)
    if not math.isfinite(args.timeout) or args.timeout <= 0:
        cli.error("--timeout must be finite and positive")
    deadline = time.monotonic() + args.timeout
    try:
        if args.command == "snapshot":
            print(json.dumps(evidence_projection.invocation_snapshot(args.invocation, registry=args.registry), sort_keys=True, allow_nan=False))
            return 0
        if args.state is None:
            cli.error("--state or PSE_SURREAL_STATE is required for service operations")
        state = args.state.absolute()
        if args.command == "prepare":
            print(json.dumps(prepare(state, deadline=deadline), sort_keys=True))
            return 0
        arguments = [args.command]
        if args.command == "import":
            snapshot = evidence_projection.invocation_snapshot(args.invocation, registry=args.registry)
            # Private temporary supplied input is deleted after the native process
            # exits. It carries compact metadata, never live payload credentials.
            with tempfile.TemporaryDirectory(prefix="pse-evidence-") as directory:
                source = Path(directory) / "snapshot.json"
                resource_store.publish(source, snapshot)
                return run_native(state, args.tool.absolute(), ["import", "--input", str(source)], deadline=deadline)
        if args.command in {"query", "generations"}:
            if args.limit is not None:
                arguments += ["--limit", str(args.limit)]
            for name in (("kind", "status", "cursor", "generation") if args.command == "query" else ("after",)):
                value = getattr(args, name)
                if value is not None:
                    arguments += ["--" + name, str(value)]
        return run_native(state, args.tool.absolute(), arguments, deadline=deadline)
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        print(f"evidence-store: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
