# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Run independent checks to completion and preserve failures, even without Cargo.

Only standard-library process/report orchestration lives here. Product-aware
inspection, generation, acceptance and native operations remain in xtask.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import re
import shutil
import signal
import subprocess
import sys
import tarfile
import time
import xml.etree.ElementTree as ET
from dataclasses import asdict
from datetime import UTC, datetime
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Mapping

from scripts import build_environment, validation_receipts
from scripts.validation_scope import (
    EXCLUSIONS,
    FUNCTIONAL_SCOPES,
    GROUPS,
    INPUT_ENVIRONMENT,
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
        if key in {"SYMBOLICA_LICENSE", "PSE_DATABASE_URL"}
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


def fresh_output(root: Path, path: Path | None) -> Path:
    path = path or root / "build/assessment" / datetime.now(UTC).strftime(
        "%Y%m%dT%H%M%S.%fZ"
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


def native_report_config(root: Path, output: Path, name: str) -> Path:
    """Preserve every test setting while isolating this invocation's report."""
    config = (root / ".config/nextest.toml").read_text()
    config, count = re.subn(
        r"(?m)(^\[profile\.ci\.junit\]\n)path\s*=\s*[^\n]+",
        lambda match: match[1] + "path = " + json.dumps(str(output / f"{name}.xml")),
        config,
    )
    if count != 1:
        raise ValueError("expected one declared ci JUnit path in nextest configuration")
    path = output / f"{name}-nextest.toml"
    path.write_text(config)
    return path


def execute(
    root: Path, output: Path, name: str, command: list[str], env: dict[str, str]
) -> dict:
    started = time.time()
    print(f"validation: {name}: {' '.join(command)}", flush=True)
    with (output / f"{name}.log").open("wb") as log:
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
                os.killpg(process.pid, signal.SIGTERM)
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid, signal.SIGKILL)
                    process.wait()
                code = -signal.SIGINT
            error = None
        except OSError as failure:
            code = None
            error = str(failure)
            log.write((error + "\n").encode())
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
        # Fixture consumers use the retained fixture's actual directory on continuation.
        fixture = previous.get("inspection-fixture", {})
        gate_env = dict(env)
        if recipe in {"native-test", "native-python", "feature-absence"}:
            gate_env["PSE_NATIVE_PROVENANCE"] = str(output / f"{gate.name}-native.json")
            if recipe == "native-test":
                gate_env["PSE_NATIVE_SELECTION"] = str(
                    output / f"{gate.name}-selected.json"
                )
            elif recipe == "native-python":
                command.append("--terminal-owner=assessment")
            else:
                gate_env.update(
                    OMP_NUM_THREADS="1", OPENBLAS_NUM_THREADS="1", MKL_NUM_THREADS="1"
                )
        if fixture.get("origin"):
            gate_env["PSE_INSPECTION_PUBLICATION"] = str(
                Path(fixture["origin"]) / "inspection"
            )
        if gate.report and "nextest/ci/junit.xml" in gate.report:
            config = native_report_config(root, output, gate.name)
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
        # Retained setup is reusable only with the exact produced fixture bytes.
        if gate.name == "inspection-fixture":
            fixture_root = output / "inspection"
            files = sorted(path for path in fixture_root.rglob("*") if path.is_file())
            if record["status"] == "passed" and not files:
                record["status"] = "failed"
                record["report_errors"].append("fixture completed without artifacts")
            for artifact in files:
                record["artifacts"][str(artifact.relative_to(output))] = (
                    validation_receipts.digest(artifact)
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
        print(
            f"validation: {gate.name}: {record['status']} ({record['elapsed_seconds']:.1f}s); {output / record['log']}",
            flush=True,
        )
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


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    selector = parser.add_mutually_exclusive_group()
    selector.add_argument("--group", choices=sorted(GROUPS))
    selector.add_argument(
        "--functional-scope", action="append", choices=sorted(FUNCTIONAL_SCOPES)
    )
    parser.add_argument("--output", type=Path)
    parser.add_argument("--list", action="store_true")
    parser.add_argument("--advisory", action="store_true")
    parser.add_argument("--reuse-from", type=Path)
    parser.add_argument("--reuse", action="append", default=[])
    parser.add_argument("--transfer", action="append", default=[])
    parser.add_argument("--change-reason")
    args = parser.parse_args()
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
        else comprehensive()
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
    output = fresh_output(root, args.output)
    print(f"validation evidence: {output}", flush=True)
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
    print(
        f"validation complete: exit {code}; baseline zero; {output / 'summary.md'}",
        flush=True,
    )
    # Advisory findings are already classified; tool failures must remain failures.
    return code


if __name__ == "__main__":
    sys.exit(main())
