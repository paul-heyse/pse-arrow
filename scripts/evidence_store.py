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
from typing import TYPE_CHECKING

from scripts import (
    evidence_detail,
    evidence_projection,
    evidence_sources,
    host_admission,
    resource_store,
    surreal_server,
)

ROOT = Path(__file__).resolve().parents[1]

if TYPE_CHECKING:
    from typing import TextIO


def prepare(
    state: Path, *, deadline: float, replace_selection: bool = False
) -> dict[str, object]:
    """Prepare only private tooling configuration; database DDL is separate init."""
    with surreal_server.tooling_activity(state, deadline=deadline) as (
        service,
        _activity,
    ):
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
                previous = resource_store.read(target)
                if previous != config:
                    if not replace_selection:
                        raise ValueError(
                            "Existing tooling selection differs; use prepare --replace-selection explicitly"
                        )
                    fixed = {"version", "namespace", "database", "credentials_file"}
                    if set(previous) != set(config) or any(
                        previous[key] != config[key] for key in fixed
                    ):
                        raise ValueError(
                            "selection replacement cannot change tooling database or credential ownership"
                        )
                    validate_credentials(credentials)
                    resource_store.publish(target, config)
                    return {
                        "status": "reselected",
                        "previous_selection": previous,
                        "config": str(target),
                        **config,
                    }
                validate_credentials(credentials)
                return {"status": "prepared", "config": str(target), **config}
            if credentials.exists():
                # A previous preparation can have durably written credentials first.
                validate_credentials(credentials)
            else:
                resource_store.publish(
                    credentials,
                    {
                        "reader_username": "pse_evidence_reader",
                        "reader_password": secrets.token_urlsafe(36),
                        "importer_username": "pse_evidence_importer",
                        "importer_password": secrets.token_urlsafe(36),
                    },
                )
            resource_store.publish(target, config)
        return {"status": "prepared", "config": str(target), **config}


def validate_credentials(path: Path) -> None:
    value = resource_store.read(path)
    if set(value) != {
        "reader_username",
        "reader_password",
        "importer_username",
        "importer_password",
    } or any(not isinstance(item, str) or not item for item in value.values()):
        raise ValueError("Invalid private tooling credentials")


def selected_configuration(state: Path, service: dict[str, object]) -> None:
    """Require an already-prepared exact selection; observation never installs it."""
    directory = state / ".tooling" / "evidence"
    for path in (state / ".tooling", directory):
        information = path.lstat()
        if (
            not stat.S_ISDIR(information.st_mode)
            or information.st_uid != os.getuid()
            or information.st_mode & 0o077
        ):
            raise ValueError(
                "Tooling configuration directories must be private and owned"
            )
    value = resource_store.read(directory / "config.json")
    if value != {
        "version": 1,
        "service_instance_id": service["instance_id"],
        "endpoint": service["endpoint"],
        "namespace": "pse_tooling",
        "database": "evidence",
        "credentials_file": "credentials.json",
    }:
        raise ValueError("Tooling selection differs from selected service identity")
    validate_credentials(directory / "credentials.json")


def run_native(
    state: Path,
    tool: Path,
    arguments: list[str],
    *,
    deadline: float,
    stdout: TextIO | None = None,
) -> int:
    """Run only the explicit built tool; never install, compile or invoke a shell."""
    information = tool.stat()
    if not stat.S_ISREG(information.st_mode) or not os.access(tool, os.X_OK):
        raise ValueError(
            "Evidence tool is unavailable; run just evidence-build explicitly"
        )
    with surreal_server.tooling_activity(state, deadline=deadline) as (
        service,
        activity,
    ):
        selected_configuration(state, service)
        if not surreal_server.owns_listener(state, service, deadline=deadline):
            raise ValueError(
                "Selected tooling service listener ownership is unavailable; inspect its existing service owner"
            )
        milliseconds = max(1, int(surreal_server.remaining(deadline) * 1000))
        command = [
            str(tool),
            "evidence",
            "--state",
            str(state),
            "--timeout-ms",
            str(milliseconds),
            *arguments,
        ]
        environment = {
            key: value
            for key, value in os.environ.items()
            if key not in {"SURREAL_USER", "SURREAL_PASS"}
        }
        environment.update(
            PSE_TOOLING_ACTIVITY_FD=str(activity),
            PSE_TOOLING_SERVICE_INSTANCE_ID=str(service["instance_id"]),
        )
        # The child retains its exact kernel lifetime if its wrapper is killed.
        completed = subprocess.run(
            command,
            check=False,
            timeout=surreal_server.remaining(deadline),
            pass_fds=(activity,),
            env=environment,
            stdout=stdout,
        )
        return completed.returncode


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser(description=__doc__)
    result.add_argument(
        "--state", type=Path, default=os.environ.get("PSE_SURREAL_STATE")
    )
    result.add_argument("--tool", type=Path, default=ROOT / "target/debug/pse-evidence")
    result.add_argument("--timeout", type=float, default=30)
    commands = result.add_subparsers(dest="command", required=True)
    commands.add_parser("prepare").add_argument(
        "--replace-selection",
        action="store_true",
        help="explicitly bind existing private tooling credentials to the selected live service; no database mutation",
    )
    for command in ("init", "status", "rebuild", "drop"):
        commands.add_parser(command)
    for command in ("snapshot", "import"):
        selected = commands.add_parser(command)
        selected.add_argument("invocation")
        selected.add_argument("--registry", type=Path)
        selected.add_argument(
            "--source-kind",
            choices=(
                "invocation",
                "assessment",
                "diagnostic",
                "measurement",
                "host",
                "storage-health",
            ),
            default="invocation",
        )
        selected.add_argument("--inventory", type=Path)
        selected.add_argument(
            "--criterion", action="append", default=[], metavar="CASE=CSV"
        )
        selected.add_argument(
            "--invocations", action="append", default=[], metavar="CASE=JSONL"
        )
    for command in ("compare-offline", "compare-indexed"):
        comparison = commands.add_parser(command)
        comparison.add_argument("--request", type=Path, required=True)
        comparison.add_argument("--observations", type=Path)
        if command == "compare-offline":
            comparison.add_argument(
                "--input", type=Path, action="append", required=True
            )
    for command in ("offline", "analyze"):
        selected = commands.add_parser(command)
        selected.add_argument(
            "--operation",
            choices=(
                "subjects",
                "failures",
                "missing",
                "distribution",
                "diagnostics",
                "measurements",
            ),
            default="subjects",
        )
        selected.add_argument("--crate-name")
        selected.add_argument("--context-id")
        selected.add_argument("--status")
        selected.add_argument("--limit", type=int, default=1024)
        selected.add_argument("--history", action="store_true")
        selected.add_argument("--observations", type=Path)
        if command == "offline":
            selected.add_argument("--input", type=Path, action="append", default=[])
            selected.add_argument("--invocation", action="append", default=[])
            selected.add_argument("--registry", type=Path)
            selected.add_argument("--inventory", type=Path)
            selected.add_argument(
                "--assessment", type=Path, action="append", default=[]
            )
            selected.add_argument(
                "--diagnostic", type=Path, action="append", default=[]
            )
            selected.add_argument(
                "--measurement", type=Path, action="append", default=[]
            )
            selected.add_argument("--host", type=Path, action="append", default=[])
            selected.add_argument(
                "--storage-health", type=Path, action="append", default=[]
            )
            selected.add_argument(
                "--criterion", action="append", default=[], metavar="CASE=CSV"
            )
            selected.add_argument(
                "--invocations", action="append", default=[], metavar="CASE=JSONL"
            )
        else:
            selected.add_argument("--generation", action="append", required=True)
    detail = commands.add_parser("detail")
    kind = detail.add_mutually_exclusive_group(required=True)
    kind.add_argument("--diagnostic", type=Path)
    kind.add_argument("--copy", type=Path)
    kind.add_argument("--invocation")
    detail.add_argument("--registry", type=Path)
    detail.add_argument("--subject")
    detail.add_argument("--member")
    detail.add_argument("--tail", type=int, default=20)
    detail.add_argument("--expected-digest")
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


def criterion_selection(values: list[str]) -> dict[str, Path]:
    selected: dict[str, Path] = {}
    for value in values:
        case, separator, path = value.partition("=")
        if not separator or not case or not path or case in selected:
            raise ValueError("Criterion selection requires unique CASE=CSV inputs")
        selected[case] = Path(path)
    return selected


def selected_snapshot(args: argparse.Namespace) -> evidence_projection.EvidenceSnapshot:
    if args.source_kind == "invocation":
        return evidence_projection.invocation_snapshot(
            args.invocation, registry=args.registry, inventory=args.inventory
        )
    path = Path(args.invocation)
    if args.source_kind == "assessment":
        return evidence_sources.assessment_snapshot(path)
    if args.source_kind == "diagnostic":
        return evidence_sources.diagnostic_snapshot(path)
    if args.source_kind == "measurement":
        return evidence_sources.measurement_snapshot(
            path,
            criterion=criterion_selection(args.criterion),
            invocations=criterion_selection(args.invocations),
        )
    if args.source_kind == "storage-health":
        return evidence_sources.storage_health_snapshot(path)
    return evidence_sources.host_snapshot(path)


def run_offline(
    tool: Path, arguments: list[str], *, deadline: float, operation: str = "offline", stdout: TextIO | None = None
) -> int:
    """Dispatch an explicitly built tool without configuration, service or build effects."""
    if operation not in {"offline", "compare-offline"}:
        raise ValueError("unsupported supplied-only evidence operation")
    information = tool.stat()
    if not stat.S_ISREG(information.st_mode) or not os.access(tool, os.X_OK):
        raise ValueError(
            "Evidence tool unavailable; use independent snapshot/detail inspection or explicitly run just evidence-build"
        )
    environment = {
        key: value
        for key, value in os.environ.items()
        if key
        not in {
            "SURREAL_USER",
            "SURREAL_PASS",
            "PSE_TOOLING_ACTIVITY_FD",
            "PSE_TOOLING_SERVICE_INSTANCE_ID",
        }
    }
    return subprocess.run(
        [str(tool), "evidence", operation, *arguments],
        stdout=stdout,
        check=False,
        timeout=surreal_server.remaining(deadline),
        env=environment,
    ).returncode


def analyze(args: argparse.Namespace, *, deadline: float) -> int:
    question = {
        "operation": args.operation,
        "crate_name": args.crate_name,
        "context_id": args.context_id,
        "status": args.status,
        "limit": args.limit,
    }
    with tempfile.TemporaryDirectory(prefix="pse-evidence-query-") as directory:
        root = Path(directory)
        question_path = root / "question.json"
        resource_store.publish(question_path, question)
        arguments = ["--question", str(question_path)]
        if args.history:
            arguments += ["--history"]
        if args.observations:
            arguments += ["--observations", str(args.observations.absolute())]
        if args.command == "analyze":
            for generation in args.generation:
                arguments += ["--generation", generation]
            return run_native(
                args.state.absolute(),
                args.tool.absolute(),
                ["analyze", *arguments],
                deadline=deadline,
            )
        snapshots = [
            evidence_projection.invocation_snapshot(
                handle, registry=args.registry, inventory=args.inventory
            )
            for handle in args.invocation
        ]
        for path in args.assessment:
            snapshots.append(evidence_sources.assessment_snapshot(path))
        for path in args.diagnostic:
            snapshots.append(evidence_sources.diagnostic_snapshot(path))
        for path in args.measurement:
            snapshots.append(
                evidence_sources.measurement_snapshot(
                    path,
                    criterion=criterion_selection(args.criterion),
                    invocations=criterion_selection(args.invocations),
                )
            )
        for path in args.host:
            snapshots.append(evidence_sources.host_snapshot(path, deadline=deadline))
        for path in args.storage_health:
            snapshots.append(evidence_sources.storage_health_snapshot(path))
        inputs = list(args.input)
        for offset, snapshot in enumerate(snapshots):
            path = root / f"snapshot-{offset}.json"
            resource_store.publish(path, snapshot)
            inputs.append(path)
        if not inputs:
            raise ValueError("select explicit snapshots or authentic owner sources")
        for path in inputs:
            arguments += ["--input", str(path.absolute())]
        return run_offline(args.tool.absolute(), arguments, deadline=deadline)


def selected_detail(args: argparse.Namespace) -> dict:
    """Validate exact detail coordinates and dispatch to their original reader."""
    if args.tail < 0:
        raise ValueError("nonnegative detail tail required")
    if args.invocation is not None:
        return evidence_detail.invocation_detail(
            args.invocation,
            registry=args.registry,
            subject=args.subject,
            tail=args.tail,
        )
    if args.member is None:
        raise ValueError("--member is required for diagnostic or copied detail")
    if args.diagnostic is not None:
        return evidence_detail.diagnostic_detail(
            args.diagnostic,
            args.member,
            tail=args.tail,
            expected_digest=args.expected_digest,
        )
    return evidence_detail.copied_detail(
        args.copy,
        args.member,
        tail=args.tail,
        expected_digest=args.expected_digest,
    )


def main(argv: list[str] | None = None) -> int:
    cli = parser()
    args = cli.parse_args(argv)
    if not math.isfinite(args.timeout) or args.timeout <= 0:
        cli.error("--timeout must be finite and positive")
    deadline = time.monotonic() + args.timeout
    try:
        if args.command == "snapshot":
            print(
                json.dumps(
                    selected_snapshot(args),
                    sort_keys=True,
                    allow_nan=False,
                )
            )
            return 0
        if args.command == "detail":
            value = selected_detail(args)
            print(json.dumps(value, sort_keys=True, allow_nan=False))
            return 0
        if args.command == "compare-offline":
            arguments = ["--request", str(args.request.absolute())]
            for source in args.input:
                arguments += ["--input", str(source.absolute())]
            if args.observations:
                arguments += ["--observations", str(args.observations.absolute())]
            return run_offline(
                args.tool.absolute(),
                arguments,
                deadline=deadline,
                operation="compare-offline",
            )
        if args.command == "offline":
            return analyze(args, deadline=deadline)
        if args.state is None:
            cli.error("--state or PSE_SURREAL_STATE is required for service operations")
        state = args.state.absolute()
        if args.command == "prepare":
            print(
                json.dumps(
                    prepare(
                        state,
                        deadline=deadline,
                        replace_selection=args.replace_selection,
                    ),
                    sort_keys=True,
                )
            )
            return 0
        if args.command == "compare-indexed":
            arguments = ["compare-indexed", "--request", str(args.request.absolute())]
            if args.observations:
                arguments += ["--observations", str(args.observations.absolute())]
            return run_native(state, args.tool.absolute(), arguments, deadline=deadline)
        if args.command == "analyze":
            return analyze(args, deadline=deadline)
        arguments = [args.command]
        if args.command == "import":
            snapshot = selected_snapshot(args)
            # Private temporary supplied input is deleted after the native process
            # exits. It carries compact metadata, never live payload credentials.
            with tempfile.TemporaryDirectory(prefix="pse-evidence-") as directory:
                source = Path(directory) / "snapshot.json"
                resource_store.publish(source, snapshot)
                return run_native(
                    state,
                    args.tool.absolute(),
                    ["import", "--input", str(source)],
                    deadline=deadline,
                )
        if args.command in {"query", "generations"}:
            if args.limit is not None:
                arguments += ["--limit", str(args.limit)]
            for name in (
                ("kind", "status", "cursor", "generation")
                if args.command == "query"
                else ("after",)
            ):
                value = getattr(args, name)
                if value is not None:
                    arguments += ["--" + name, str(value)]
        return run_native(state, args.tool.absolute(), arguments, deadline=deadline)
    except (
        OSError,
        ValueError,
        TypeError,
        KeyError,
        RuntimeError,
        subprocess.SubprocessError,
    ) as error:
        print(f"evidence-store: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
