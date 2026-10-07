# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Run independent checks to completion and preserve failures, even without Cargo.

Only standard-library process/report orchestration lives here. Product-aware
inspection, generation, acceptance and native operations remain in xtask.
"""

from __future__ import annotations

import argparse
import contextlib
import hashlib
import json
import os
import platform
import re
import secrets
import shlex
import shutil
import signal
import subprocess
import sys
import tarfile
import threading
import time
import xml.etree.ElementTree as ET
from dataclasses import asdict
from datetime import UTC, datetime
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Mapping

from scripts import build_environment, producer_deployment, validation_receipts
from scripts.validation_scope import (
    EXCLUSIONS,
    FUNCTIONAL_SCOPES,
    GROUPS,
    INPUT_ENVIRONMENT,
    PRODUCER_REVIEW_INPUTS,
    Gate,
    comprehensive,
    expand,
    input_identity,
)


def command_env() -> dict[str, str]:
    # xtask can invoke just, which can invoke this runner. Cargo's parent package
    # variables are not valid inputs to nested builds (including uv/maturin).
    return build_environment.configure(
        Path(__file__).resolve().parents[1],
        {
            key: value
            for key, value in os.environ.items()
            if not key.startswith(("CARGO_PKG_", "CARGO_MANIFEST_"))
        },
    )


def relevant_environment(source: Mapping[str, str] | None = None) -> dict[str, str]:
    """Capture relevant controls, hashing credentials instead of publishing secrets."""
    source_environment = os.environ if source is None else source
    environment = {
        key: hashlib.sha256(source_environment[key].encode()).hexdigest()
        if key == "SYMBOLICA_LICENSE"
        else source_environment[key]
        for key in (
            *INPUT_ENVIRONMENT,
            "CARGO_TARGET_DIR",
            "SCCACHE_DIR",
            "CARGO_BUILD_JOBS",
        )
        if key in source_environment
    }
    local = Path(__file__).resolve().parents[1] / ".envrc.local"
    environment["LOCAL_NATIVE_ENVIRONMENT"] = hashlib.sha256(
        local.read_bytes() if local.is_file() else b"absent"
    ).hexdigest()
    for key in PRODUCER_REVIEW_INPUTS:
        if key in source_environment:
            path = Path(source_environment[key])
            if not path.is_absolute():
                path = Path(__file__).resolve().parents[1] / path
            environment[key] = json.dumps(
                {
                    "path": str(path),
                    "sha256": hashlib.sha256(path.read_bytes()).hexdigest()
                    if path.is_file()
                    else None,
                },
                sort_keys=True,
            )
    return environment


def git(root: Path, *args: str) -> bytes:
    return subprocess.check_output(["git", *args], cwd=root)


# One explicit product-input scope owns hashing, patches and source archives.
# Reference corpora and local credentials are not compiler or acceptance inputs.
SOURCE_PATHS = (
    "Cargo.toml",
    "Cargo.lock",
    "pyproject.toml",
    "uv.lock",
    "rust-toolchain.toml",
    ".python-version",
    "justfile",
    "AGENTS.md",
    "CLAUDE.md",
    "README.md",
    "clippy.toml",
    "sgconfig.yml",
    "sgrules",
    "sgutils",
    "sgtests",
    "deny.toml",
    "conftest.py",
    ".gitignore",
    ".cargo",
    ".config",
    ".github",
    ".claude",
    ".codex",
    ".agents",
    "crates",
    "xtask",
    "tests",
    "benches",
    "python",
    "scripts",
    "packages",
    "vendor",
    "docker",
    # Publishing inputs are selected by docs/site.toml; capture the whole tree
    # so new collections and root pages cannot evade the shared inventory.
    "docs",
    "REUSE.toml",
    "LICENSES",
    ".pre-commit-config.yaml",
)


def sources(root: Path) -> dict[str, str]:
    names = git(
        root,
        "ls-files",
        "-z",
        "--cached",
        "--others",
        "--exclude-standard",
        "--",
        *SOURCE_PATHS,
    )
    result = {}
    for name in sorted(set(names.decode().split("\0")) - {""}):
        path = root / name
        if path.is_symlink():
            data = b"symlink:" + os.fsencode(path.readlink())
        elif path.is_file():
            data = str(path.stat().st_mode & 0o777).encode() + b":" + path.read_bytes()
        else:
            data = b"absent"
        result[name] = hashlib.sha256(data).hexdigest()
    return result


def write_json(path: Path, value: object) -> None:
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(json.dumps(value, indent=2) + "\n")
    temporary.replace(path)


def fresh_output(root: Path, path: Path | None, label: str = "assessment") -> Path:
    """A new evidence directory; the default name is unique per invocation.

    The UTC time orders runs, the label names the selection, and the process ID plus a
    random suffix keep concurrent invocations apart. An existing directory is refused.
    """
    path = path or root / "build/assessment" / (
        datetime.now(UTC).strftime("%Y%m%dT%H%M%S.%fZ")
        + f"-{label}-{os.getpid()}-{secrets.token_hex(3)}"
    )
    path = path.resolve()
    path.relative_to(root)
    ignored = subprocess.run(
        ["git", "check-ignore", "-q", "--", str(path)], cwd=root, check=False
    )
    if ignored.returncode != 0:
        raise ValueError("evidence must be in a gitignored repository directory")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.mkdir()  # Deliberately refuse overwrite, including a prior failed run.
    return path


def mark_latest(output: Path, label: str) -> Path | None:
    """Point ``latest-<label>`` beside the run at it: a convenience nothing reads.

    The link is replaced atomically; the immutable run path is what runs print and
    record, so a concurrent run moving the link never changes anyone's evidence.
    """
    link = output.parent / f"latest-{label}"
    temporary = output.parent / f".latest-{label}.{os.getpid()}.{secrets.token_hex(3)}"
    try:
        temporary.symlink_to(output.name)
        temporary.replace(link)
    except OSError:
        temporary.unlink(missing_ok=True)
        return None
    return link


def provenance(root: Path, output: Path) -> tuple[dict[str, str], Path, list[str]]:
    snapshot = sources(root)
    write_json(output / "source-files.json", snapshot)
    write_json(
        output / "host.json",
        {
            "platform": platform.platform(),
            "machine": platform.machine(),
            "logical_cpus": os.cpu_count(),
            "cpu_affinity": sorted(os.sched_getaffinity(0))
            if hasattr(os, "sched_getaffinity")
            else None,
            "cpuinfo": Path("/proc/cpuinfo").read_text()
            if Path("/proc/cpuinfo").exists()
            else None,
            "meminfo": Path("/proc/meminfo").read_text()
            if Path("/proc/meminfo").exists()
            else None,
        },
    )
    (output / "source.diff").write_bytes(
        git(root, "diff", "--binary", "HEAD", "--", *SOURCE_PATHS)
    )
    (output / "source-status.txt").write_bytes(
        git(root, "status", "--porcelain=v1", "--", *SOURCE_PATHS)
    )
    (output / "source-revision.txt").write_bytes(git(root, "rev-parse", "HEAD"))
    untracked = git(
        root, "ls-files", "--others", "--exclude-standard", "-z", "--", *SOURCE_PATHS
    )
    with tarfile.open(output / "untracked-source.tar.gz", "w:gz") as archive:
        for name in filter(None, untracked.decode().split("\0")):
            archive.add(root / name, arcname=name, recursive=False)
    for name in (
        "Cargo.toml",
        "Cargo.lock",
        "pyproject.toml",
        "uv.lock",
        ".config/nextest.toml",
    ):
        shutil.copy2(root / name, output / Path(name).name)
    errors = []
    target = root / os.environ.get("CARGO_TARGET_DIR", "target")
    # Failure to resolve Cargo must not prevent lint, Python and report collection.
    with (output / "cargo-metadata.stderr.log").open("wb") as stderr:
        try:
            result = subprocess.run(
                ["cargo", "metadata", "--locked", "--offline", "--format-version", "1"],
                cwd=root,
                env=command_env(),
                stdout=subprocess.PIPE,
                stderr=stderr,
                check=False,
            )
        except OSError as error:
            errors.append(f"cargo metadata unavailable: {error}")
            return snapshot, target, errors
    (output / "cargo-metadata.json").write_bytes(result.stdout)
    if result.returncode == 0:
        try:
            target = Path(json.loads(result.stdout)["target_directory"])
        except (ValueError, KeyError, TypeError) as error:
            errors.append(f"invalid cargo metadata: {error}")
    else:
        errors.append(
            f"cargo metadata failed ({result.returncode}); report target defaults to {target}"
        )
    return snapshot, target, errors


def collect_report(
    source: Path, output: Path, name: str, started: float
) -> tuple[list[dict[str, str]], list[str]]:
    if not source.is_file() or source.stat().st_mtime < started:
        return [], [f"missing current report: {source}"]
    destination = output / f"{name}.xml"
    if source != destination:
        shutil.copy2(source, destination)
    try:
        tree = ET.parse(destination)  # noqa: S314 - only fresh, locally produced runner reports
    except (ET.ParseError, OSError) as error:
        return [], [f"unreadable report: {error}"]
    errors = []
    for suite in tree.iter():
        if suite.tag not in {"testsuite", "testsuites"}:
            continue
        cases = list(suite.iter("testcase"))
        counts = {
            "tests": len(cases),
            **{
                plural: sum(test.find(kind) is not None for test in cases)
                for plural, kind in (
                    ("failures", "failure"),
                    ("errors", "error"),
                    ("skipped", "skipped"),
                )
            },
        }
        for field, actual in counts.items():
            declared = suite.get(field)
            if declared is not None and (
                not declared.isdigit() or int(declared) != actual
            ):
                errors.append(
                    f"declared {field}={declared} differs from {actual} in {suite.get('name', suite.tag)}"
                )
    results = []
    identities = set()
    for test in tree.iter("testcase"):
        status = "passed"
        details = []
        for kind in ("failure", "error", "skipped"):
            for element in test.findall(kind):
                status = kind
                details.append(
                    element.get("message", "") + "\n" + "".join(element.itertext())
                )
        nodeid = next(
            (
                prop.get("value", "")
                for prop in test.findall("properties/property")
                if prop.get("name") == "nodeid"
            ),
            "",
        )
        identity = (test.get("classname", ""), test.get("name", ""), nodeid)
        if identity in identities:
            errors.append(f"duplicate reported test: {identity}")
        identities.add(identity)
        results.append(
            {
                "nodeid": nodeid,
                "name": test.get("name", ""),
                "class": test.get("classname", ""),
                "status": status,
                "details": "\n".join(details),
            }
        )
    return results, errors if results else [*errors, "report contains no test cases"]


def compose_selection(check: dict) -> None:
    """Finalize terminal evidence after the invocation's own inventory is attached."""
    validation_receipts.reconcile(check)
    if check["status"] != "interrupted" and (
        check["exit_code"] != 0
        or check["report_errors"]
        or any(result["status"] != "passed" for result in check["results"])
    ):
        check["status"] = "failed"


def compose_terminal(
    check: dict, report: Path, output: Path, name: str, *, reconcile: bool = True
) -> None:
    """One JUnit parser and status composition for parented and standalone runs."""
    try:
        check["results"], errors = collect_report(
            report, output, name, check["started"]
        )
        check["report_errors"].extend(errors)
    except (OSError, ValueError) as error:
        check["report_errors"].append(str(error))
    copied = output / f"{name}.xml"
    if copied.is_file():
        check.setdefault("artifacts", {})[copied.name] = validation_receipts.digest(
            copied
        )
    if reconcile:
        compose_selection(check)


def native_report_config(
    root: Path, output: Path, name: str, profile: str = "local"
) -> Path:
    """Preserve every test setting while isolating this invocation's report."""
    config = (root / ".config/nextest.toml").read_text()
    config, count = re.subn(
        rf"(?m)(^\[profile\.{re.escape(profile)}\.junit\]\n)path\s*=\s*[^\n]+",
        lambda match: match[1] + "path = " + json.dumps(str(output / f"{name}.xml")),
        config,
    )
    if count != 1:
        raise ValueError(
            f"expected one declared {profile} JUnit path in nextest configuration"
        )
    path = output / f"{name}-nextest.toml"
    path.write_text(config)
    return path


#: Console mode for this process: stream each step's output while it is written to its
#: log (``--live`` or ``PSE_VALIDATION_LIVE=1``). The log is written either way.
LIVE = False
#: Lines of a failed step's log shown on the console when output is not live.
FAILURE_TAIL_LINES = 20


class _Follower(threading.Thread):
    """Copy a log to standard output as it grows; the child still writes only the file."""

    def __init__(self, path: Path) -> None:
        super().__init__(daemon=True)
        self.path = path
        self.finished = threading.Event()

    def run(self) -> None:
        with self.path.open("rb") as stream:
            while True:
                done = self.finished.is_set()
                chunk = stream.read(1 << 16)
                if chunk:
                    sys.stdout.buffer.write(chunk)
                    sys.stdout.buffer.flush()
                elif done:
                    return
                else:
                    self.finished.wait(0.2)


def tail(path: Path, lines: int = FAILURE_TAIL_LINES) -> list[str]:
    """The last lines of a log, decoded leniently; a missing log has none."""
    try:
        with path.open("rb") as stream:
            stream.seek(0, os.SEEK_END)
            stream.seek(max(0, stream.tell() - 64 * 1024))
            data = stream.read()
    except OSError:
        return []
    return data.decode(errors="replace").splitlines()[-lines:]


def execute(
    root: Path, output: Path, name: str, command: list[str], env: dict[str, str]
) -> dict:
    started = time.time()
    path = output / f"{name}.log"
    print(f"validation: {name}: {shlex.join(command)}; log {path}", flush=True)
    with path.open("wb") as log:
        follower = _Follower(path) if LIVE else None
        if follower is not None:
            follower.start()
        try:
            process = subprocess.Popen(
                command,
                cwd=root,
                env=env,
                stdout=log,
                stderr=subprocess.STDOUT,
                start_new_session=True,
            )
            try:
                code = process.wait()
            except KeyboardInterrupt:
                # The child has its own session; terminate its whole group and wait,
                # so no owned descendant outlives the retained "interrupted" record.
                os.killpg(process.pid, signal.SIGTERM)
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid, signal.SIGKILL)
                    process.wait()
                with contextlib.suppress(ProcessLookupError):
                    os.killpg(process.pid, signal.SIGKILL)
                code = -signal.SIGINT
            error = None
        except OSError as failure:
            code = None
            error = str(failure)
            log.write((error + "\n").encode())
        finally:
            if follower is not None:
                log.flush()
                follower.finished.set()
                follower.join(timeout=5)
    return {
        "gate": name,
        "command": command,
        "exit_code": code,
        "status": "passed"
        if code == 0
        else "interrupted"
        if code is not None and code < 0
        else "failed",
        "spawn_error": error,
        "started": started,
        "ended": time.time(),
        "elapsed_seconds": time.time() - started,
        "log": f"{name}.log",
        "results": [],
        "report_errors": [],
    }


def checkpoint(output: Path, receipt: dict) -> None:
    write_json(output / "checks.json", receipt)
    failures = [
        {"gate": check["gate"], **result}
        for check in receipt["checks"]
        for result in check["results"]
        if result["status"] != "passed"
    ]
    write_json(output / "test-findings.json", failures)
    unsuccessful = [c for c in receipt["checks"] if c["status"] != "passed"]
    write_json(output / "failures.json", unsuccessful)
    lines = [
        "# Validation assessment",
        "",
        f"Baseline: zero failures. Mode: {receipt['mode']}; command results do not constitute architecture review.",
        "",
        f"Attempted {len(receipt['checks'])}/{len(receipt['scope'])} checks; {len(unsuccessful)} unsuccessful checks.",
        f"All checks attempted: {receipt['complete']}. Source unchanged: {receipt['source_unchanged']}.",
        "",
        "| Check | Status | Exit | Seconds | Log |",
        "| --- | --- | --- | --- | --- |",
    ]
    for check in receipt["checks"]:
        log = (
            str(Path(check["origin"]) / check["log"])
            if check.get("origin")
            else check["log"]
        )
        lines.append(
            f"| {check['gate']} | {check['status']} | {check['exit_code']} | {check['elapsed_seconds']:.1f} | [{check['log']}]({log}) |"
        )
    lines.extend(
        (
            "",
            "Native failures, errors and skips: [test-findings.json](test-findings.json).",
            "Command/report failures: [failures.json](failures.json). Full commands and source drift: [checks.json](checks.json).",
            "",
        )
    )
    (output / "summary.md").write_text("\n".join(lines))


def observed_deployment_attestation(path: Path) -> dict[str, str]:
    observed = json.loads(path.read_text())
    if (
        not isinstance(observed, list)
        or len(observed) != 2
        or not all(isinstance(value, str) for value in observed)
    ):
        raise ValueError("invalid independently observed deployment header")
    return {"source": observed[0], "build": observed[1]}


def report_gate(output: Path, record: dict) -> None:
    """One console line per finished gate; an unqualified gate adds its log's tail."""
    exit_code = record["exit_code"]
    detail = f"exit {exit_code}, " if exit_code not in (0, None) else ""
    log = output / record["log"]
    print(
        f"validation: {record['gate']}: {record['status']} ({detail}{record['elapsed_seconds']:.1f}s); {log}",
        flush=True,
    )
    if validation_receipts.qualified(record):
        return
    for error in record["report_errors"][:5]:
        print(f"    report: {error}", flush=True)
    if len(record["report_errors"]) > 5:
        print(f"    report: {len(record['report_errors']) - 5} more in checks.json", flush=True)
    if not LIVE:
        for line in tail(log):
            print(f"    | {line}", flush=True)


def run_gates(
    root: Path,
    output: Path,
    gates: list[Gate],
    *,
    capture: bool = True,
    reuse_from: Path | None = None,
    reuse: tuple[str, ...] = (),
    transfer: tuple[str, ...] = (),
    change_reason: str | None = None,
) -> int:
    if (reuse or transfer) and reuse_from is None:
        raise ValueError("reuse and transfer require an origin report")
    deployment_retained = set(reuse + transfer) & {
        "producer-deployment",
        "python-deployment-association",
    }
    if deployment_retained:
        raise ValueError(
            "Deployment capture and imported association require fresh execution: "
            + ", ".join(sorted(deployment_retained))
        )
    snapshot, target, errors = (
        provenance(root, output) if capture else ({}, root / "target", [])
    )
    context = {
        "output": str(output),
        "target": str(target),
        "relative": str(output.relative_to(root)),
    }
    receipt: dict = {
        "version": 5,
        "mode": "local",
        "baseline_failures": 0,
        "scope": [asdict(gate) for gate in gates],
        "checks": [],
        "complete": False,
        "source_unchanged": True,
        "provenance_errors": errors,
        "evidence": "Proposed",
        "source_files": snapshot,
        "input_coverage": capture,
        "parent": None,
        "environment": relevant_environment(),
    }
    selected = {gate.name for gate in gates}
    if reuse_from:
        parent = reuse_from.resolve()
        receipt["parent"] = {
            "path": str(parent),
            "digest": validation_receipts.digest(parent / "checks.json"),
        }
        receipt["checks"] = validation_receipts.reuse_checks(
            parent,
            snapshot,
            receipt["environment"],
            receipt["scope"],
            set(reuse),
            set(transfer),
            change_reason,
        )
    write_json(
        output / "scope.json", {"checks": receipt["scope"], "exclusions": EXCLUSIONS}
    )
    checkpoint(output, receipt)
    env = command_env()
    env["PSE_ACCEPTANCE_OUTPUT"] = str(output)
    if "producer-fixture" in selected:
        fixture = next(
            (
                check
                for check in receipt["checks"]
                if check["gate"] == "producer-fixture"
            ),
            None,
        )
        fixture_origin = Path(fixture["origin"]) if fixture is not None else output
        env["PSE_PRODUCER_FIXTURE_RECEIPT"] = str(
            fixture_origin / "producer-fixture.json"
        )
    deployment_env = {}
    if "producer-deployment" in selected:
        deployment_env = producer_deployment.deployment_environment(
            root, output / "deployment"
        )
    # Nested just aggregates use their own evidence folder; never overwrite ours.
    env.pop("PSE_VALIDATION_OUTPUT", None)
    # Selection belongs to this exact gate, never an enclosing pytest/assessment.
    env.pop("PSE_TEST_ENUMERATION", None)
    interrupted = False
    for gate in gates:
        if any(check["gate"] == gate.name for check in receipt["checks"]):
            continue
        previous = {check["gate"]: check for check in receipt["checks"]}
        dependencies = [
            name
            for name in gate.dependencies
            if name in selected
            and (
                name not in previous
                or not validation_receipts.qualified(previous[name])
            )
        ]
        recipe = gate.recipe or gate.name
        command = ["just", recipe, *(arg.format_map(context) for arg in gate.args)]
        report = Path(gate.report.format_map(context)) if gate.report else None
        gate_env = dict(env)
        if recipe in {"native-test", "native-python"}:
            gate_env.update(deployment_env)
        receipt_variable = {
            "native-test": "PSE_WORKER_PRODUCER_RECEIPT",
            "native-python": "PSE_PYTHON_PRODUCER_RECEIPT",
        }.get(recipe)
        if receipt_variable and gate_env.get(receipt_variable):
            gate_env["PSE_PRODUCER_RECEIPT"] = gate_env[receipt_variable]
        if gate.name == "python-deployment-association":
            # The observed header is this gate's output, then an input to the
            # native cross-role and ordinary Python gates. Do not hash it before
            # the imported extension has produced it.
            gate_env["PSE_PYTHON_DEPLOYMENT_OBSERVATION_OUTPUT"] = gate_env.pop(
                "PSE_PYTHON_DEPLOYMENT_ATTESTATION"
            )
        if recipe in {"native-test", "native-python", "feature-absence"}:
            # Receipts require the single-thread native budget; an ambient caller
            # choice must not reach a qualification gate.
            gate_env.update(
                OMP_NUM_THREADS="1", OPENBLAS_NUM_THREADS="1", MKL_NUM_THREADS="1"
            )
            gate_env["PSE_NATIVE_PROVENANCE"] = str(output / f"{gate.name}-native.json")
            if recipe == "native-test":
                gate_env["PSE_NATIVE_SELECTION"] = str(
                    output / f"{gate.name}-selected.json"
                )
            elif recipe == "native-python":
                command.append("--terminal-owner=assessment")
        if gate.report and f"nextest/{gate.profile}/junit.xml" in gate.report:
            config = native_report_config(root, output, gate.name, gate.profile)
            command.extend(("--config-file", str(config)))
            report = output / f"{gate.name}.xml"
        record: dict = {
            "evidence_kind": "not-run",
            "invocation": asdict(gate),
            "gate": gate.name,
            "command": command,
            "exit_code": None,
            "status": "not_run"
            if interrupted
            else "blocked"
            if dependencies
            else "running",
            "dependencies": dependencies,
            "role": gate.role,
            "phase": gate.phase,
            "native_required": recipe
            in {"native-test", "native-python", "feature-absence"},
            "mode": gate.mode,
            "profile": gate.profile,
            "elapsed_seconds": 0,
            "log": f"{gate.name}.log",
            "results": [],
            "report_errors": [],
            "artifacts": {},
            "changed_source": [],
            "inputs": input_identity(
                gate.input_scope, snapshot, receipt["environment"]
            ),
        }
        receipt["checks"].append(record)
        checkpoint(output, receipt)
        if interrupted or dependencies:
            print(
                f"validation: {gate.name}: {record['status']}"
                + (f" by {', '.join(dependencies)}" if dependencies and not interrupted else ""),
                flush=True,
            )
            continue
        if report and recipe == "native-python":
            gate_env["PSE_TEST_ENUMERATION"] = str(output / f"{gate.name}-selected.txt")
        if gate.enumerate_native:
            listing_command = [
                arg
                for i, arg in enumerate(command)
                if arg != "--success-output"
                and (i == 0 or command[i - 1] != "--success-output")
            ]
            listing = execute(
                root,
                output,
                f"{gate.name}-enumeration",
                listing_command,
                {**gate_env, "PSE_NEXTEST_ACTION": "list --message-format json"},
            )
            record["enumeration"] = listing
            record["invocation_started"] = listing["started"]
            record["artifacts"][listing["log"]] = validation_receipts.digest(
                output / listing["log"]
            )
            try:
                record["selected"] = validation_receipts.enumeration_selection(
                    output, listing
                )
            except (ValueError, KeyError, TypeError) as error:
                record["status"] = (
                    listing["status"] if listing["status"] != "passed" else "failed"
                )
                record["report_errors"].append(str(error))
                interrupted = record["status"] == "interrupted"
                checkpoint(output, receipt)
                continue
            if recipe == "feature-absence":
                try:
                    # The native wrapper imports this collector; defer to avoid a cycle.
                    from scripts.native_tests import native_provenance  # noqa: PLC0415

                    data = validation_receipts.native_inventory(
                        (output / listing["log"]).read_text()
                    )
                    binaries = [
                        suite["binary-path"]
                        for suite in data["rust-suites"].values()
                        if any(
                            test["filter-match"]["status"] == "matches"
                            for test in suite["testcases"].values()
                        )
                    ]
                    write_json(
                        Path(gate_env["PSE_NATIVE_PROVENANCE"]),
                        native_provenance(
                            {
                                "features": ["pse-relations/force-validate"],
                                "cargo_profile": "dev",
                            },
                            binaries,
                            environment=gate_env,
                        ),
                    )
                except (
                    OSError,
                    ValueError,
                    KeyError,
                    TypeError,
                    subprocess.CalledProcessError,
                ) as error:
                    record["status"] = "failed"
                    record["report_errors"].append(
                        f"default graph provenance unavailable: {error}"
                    )
                    checkpoint(output, receipt)
                    continue
            checkpoint(output, receipt)
        record.update(execute(root, output, gate.name, command, gate_env))
        record["evidence_kind"] = "executed"
        interrupted = record["status"] == "interrupted"
        receipt["evidence"] = "Tested"
        checkpoint(output, receipt)  # Command survives collector or source errors.
        if report:
            compose_terminal(record, report, output, gate.name, reconcile=False)
        if gate.name == "setup-test" and gate.recipe == "setup-test-report":
            selected_path = output / "setup-test-selected.json"
            try:
                record["selected"] = json.loads(selected_path.read_text())
                record["artifacts"][selected_path.name] = validation_receipts.digest(
                    selected_path
                )
            except (OSError, ValueError, TypeError) as error:
                record["report_errors"].append(f"setup selection unavailable: {error}")
        if gate.name in {"producer-deployment", "python-deployment-association"}:
            artifacts = (
                [
                    output / "deployment" / f"{role}.json"
                    for role in (
                        "runtime",
                        "worker",
                        "python",
                        "outer-observation",
                        "observed-artifacts",
                    )
                ]
                if gate.name == "producer-deployment"
                else [Path(deployment_env["PSE_PYTHON_DEPLOYMENT_ATTESTATION"])]
            )
            for artifact in artifacts:
                if artifact.is_file():
                    record["artifacts"][str(artifact.relative_to(output))] = (
                        validation_receipts.digest(artifact)
                    )
                elif record["status"] == "passed":
                    record["status"] = "failed"
                    record["report_errors"].append(
                        f"deployment step completed without {artifact.name}"
                    )
            if (
                gate.name == "python-deployment-association"
                and record["status"] == "passed"
            ):
                try:
                    observed_deployment_attestation(artifacts[0])
                    producer_deployment.validate_receipts(
                        output / "deployment",
                    )
                except (OSError, ValueError, KeyError, TypeError) as error:
                    record["status"] = "failed"
                    record["report_errors"].append(
                        f"deployment association refused: {error}"
                    )
        if gate.name == "producer-fixture" and record["status"] == "passed":
            artifact = output / "producer-fixture.json"
            if artifact.is_file():
                record["artifacts"][artifact.name] = validation_receipts.digest(
                    artifact
                )
            else:
                record["status"] = "failed"
                record["report_errors"].append(
                    "producer fixture completed without receipt"
                )
        if recipe == "native-test":
            raw_selection = output / f"{gate.name}-selected.json"
            try:
                validation_receipts.require_fresh(
                    raw_selection, record["started"], "native"
                )
                record["selected"] = validation_receipts.native_selection(
                    raw_selection.read_text()
                )
                record["artifacts"][raw_selection.name] = validation_receipts.digest(
                    raw_selection
                )
            except (OSError, ValueError, KeyError, TypeError) as error:
                record["report_errors"].append(f"native selection unavailable: {error}")
        selected_path = output / f"{gate.name}-selected.txt"
        if gate_env.get("PSE_TEST_ENUMERATION"):
            try:
                record["selected"] = validation_receipts.python_selection(selected_path)
                record["artifacts"][selected_path.name] = validation_receipts.digest(
                    selected_path
                )
            except (OSError, ValueError) as error:
                record["report_errors"].append(f"missing current collection: {error}")
        compose_selection(record)
        validation_receipts.classify(record, output)
        record["artifacts"][record["log"]] = validation_receipts.digest(
            output / record["log"]
        )
        record["totals"] = {
            status: sum(r["status"] == status for r in record["results"])
            for status in ("passed", "failure", "error", "skipped", "not_run")
        }
        checkpoint(output, receipt)
        report_gate(output, record)
    if capture:
        try:
            current = sources(root)
            receipt["contextual_changes"] = validation_receipts.changed(
                snapshot, current
            )
            receipt["source_unchanged"] = not receipt["contextual_changes"]
            after_environment = relevant_environment()
            for record in receipt["checks"]:
                scope = next(g.input_scope for g in gates if g.name == record["gate"])
                after = input_identity(scope, current, after_environment)
                record["changed_source"] = validation_receipts.changed(
                    record["inputs"]["files"], after["files"]
                )
                record["changed_environment"] = validation_receipts.changed(
                    record["inputs"]["environment"], after["environment"]
                )
                if record["changed_source"] or record["changed_environment"]:
                    record["report_errors"].append(
                        "relevant inputs changed during assessment"
                    )
                    if record["status"] == "passed":
                        record["status"] = "failed"
        except OSError as error:
            receipt["provenance_errors"].append(str(error))
    receipt["complete"] = not interrupted and all(
        c["status"] not in {"running", "not_run", "blocked", "interrupted"}
        for c in receipt["checks"]
    )
    receipt["required_checks_covered"] = (
        receipt["complete"]
        and not receipt["provenance_errors"]
        and all(validation_receipts.qualified(c) for c in receipt["checks"])
    )
    checkpoint(output, receipt)
    return int(not receipt["required_checks_covered"])


#: Recipes whose gates need the native environment, and those that also need a serving
#: canonical server; the assessment checks them before running anything.
NATIVE_RECIPES = frozenset({"native-test", "native-python", "feature-absence"})
STORE_RECIPES = frozenset({"native-test", "native-python"})

EPILOG = """\
output:
  Each run writes a new build/assessment/<UTC time>-<selection>-<pid>-<random>/ (or
  --output) holding <gate>.log per gate, checks.json, failures.json and summary.md, and
  prints that immutable path. build/assessment/latest-<selection> points at the newest
  default run, for convenience only. The console shows each gate's command and log path
  when it starts (follow it with tail -f) and one result line when it ends; an unqualified
  gate adds the last 20 lines of its log. --live (or PSE_VALIDATION_LIVE=1) streams each
  gate's output to the console as well, still writing the logs.

exit status:
  0 when every selected gate qualified (passed, advisory findings, or deferred and
  unsupported); 1 when any gate failed, was blocked or provenance failed (later gates still
  run); 125 with a pse-env: line when a prerequisite of the selection is missing (nothing
  runs); 128+N when signal N interrupted the run (SIGINT, SIGTERM or SIGHUP: the running
  gate's process group is terminated and later gates are recorded not_run).
"""


def preflight_kinds(gates: list[Gate]) -> list[str]:
    """Prerequisites the selected gates are known to need before they can mean anything."""
    recipes = {gate.recipe or gate.name for gate in gates}
    return [
        kind
        for kind, needed in (("native", NATIVE_RECIPES), ("store", STORE_RECIPES))
        if recipes & needed
    ]


def selection_label(args: argparse.Namespace) -> str:
    if args.group:
        return args.group
    if args.functional_scope:
        return "functional-" + "-".join(dict.fromkeys(args.functional_scope))
    return "assessment"


#: The signal that interrupted this run; Ctrl-C unless a handler below recorded another.
RECEIVED_SIGNAL = signal.SIGINT


class Interrupted(KeyboardInterrupt):
    """A termination signal delivered to the runner, handled like Ctrl-C."""


def interrupt(signum: int, _frame: object) -> None:
    global RECEIVED_SIGNAL  # noqa: PLW0603 -- one process-wide signal record
    RECEIVED_SIGNAL = signum
    raise Interrupted


def main(argv: list[str] | None = None) -> int:
    global LIVE  # noqa: PLW0603 -- process-wide console mode, chosen once here
    parser = argparse.ArgumentParser(
        description=__doc__,
        epilog=EPILOG,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    selector = parser.add_mutually_exclusive_group()
    selector.add_argument("--group", choices=sorted(GROUPS))
    selector.add_argument(
        "--functional-scope", action="append", choices=sorted(FUNCTIONAL_SCOPES)
    )
    parser.add_argument("--output", type=Path)
    parser.add_argument("--list", action="store_true")
    parser.add_argument(
        "--live",
        action="store_true",
        default=os.environ.get("PSE_VALIDATION_LIVE") == "1",
        help="stream every gate's output to the console as well as its log",
    )
    parser.add_argument(
        "--preflight",
        action="store_true",
        help="only check the selection's prerequisites (native solver, canonical store) and exit",
    )
    parser.add_argument("--reuse-from", type=Path)
    parser.add_argument("--reuse", action="append", default=[])
    parser.add_argument("--transfer", action="append", default=[])
    parser.add_argument("--change-reason")
    parser.add_argument(
        "--python-profile",
        default="dev",
        choices=("dev", "producer"),
        help="Cargo profile installed for the linked Python assessment",
    )
    args = parser.parse_args(argv)
    if (
        args.functional_scope
        and "native" in args.functional_scope
        and len(set(args.functional_scope)) != 1
    ):
        parser.error(
            "native is the explicit covering functional scope; select it alone"
        )
    gates = (
        [FUNCTIONAL_SCOPES[name] for name in dict.fromkeys(args.functional_scope)]
        if args.functional_scope
        else expand((args.group,))
        if args.group
        else comprehensive(args.python_profile)
    )
    if args.list:
        print(
            json.dumps(
                {
                    "checks": [asdict(g) for g in gates],
                    "exclusions": EXCLUSIONS,
                },
                indent=2,
            )
        )
        return 0
    root = Path(__file__).resolve().parents[1]
    kinds = preflight_kinds(gates)
    if kinds:
        # Deferred: preflight composes the checkout environment through pse_env.
        from scripts import preflight  # noqa: PLC0415

        if code := preflight.main(kinds):
            return code
    if args.preflight:
        return 0
    LIVE = args.live
    label = selection_label(args)
    output = fresh_output(root, args.output, label)
    print(f"validation evidence: {output}", flush=True)
    if args.output is None:
        mark_latest(output, label)
    for name in (signal.SIGTERM, signal.SIGHUP):
        signal.signal(name, interrupt)
    try:
        code = run_gates(
            root,
            output,
            gates,
            capture=not args.group,
            reuse_from=args.reuse_from,
            reuse=tuple(args.reuse),
            transfer=tuple(args.transfer),
            change_reason=args.change_reason,
        )
    except KeyboardInterrupt:
        # Outside a gate's process wait; checks.json keeps the last checkpoint.
        print(f"validation interrupted; partial evidence: {output}", flush=True)
        return 128 + RECEIVED_SIGNAL
    checks = json.loads((output / "checks.json").read_text())["checks"]
    unsuccessful = [
        check["gate"]
        for check in checks
        if not validation_receipts.qualified(check)
    ]
    if any(check["status"] == "interrupted" for check in checks):
        code = 128 + RECEIVED_SIGNAL
    print(
        f"validation {label}: exit {code}; "
        + (
            f"{len(unsuccessful)} unsuccessful: {' '.join(unsuccessful)} (re-run one with just <gate>); "
            if unsuccessful
            else "all qualified; "
        )
        + f"baseline zero; {output / 'summary.md'}",
        flush=True,
    )
    return code


if __name__ == "__main__":
    sys.exit(main())
