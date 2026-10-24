# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Combination-level execution under the existing validation receipt owner.

cargo-hack owns enumeration and order. Its unstable printed display is never
parsed; the existing Cargo adapter captures literal arguments without compiling.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import signal
import subprocess
import sys
from pathlib import Path

from scripts import cargo_tools, installation_context, validation, validation_receipts
from scripts.validation_scope import input_identity, declared

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = "validation-feature-campaign-v1"
CAPTURE = "PSE_FEATURE_CAPTURE"
ATTEMPT = "PSE_FEATURE_ATTEMPT"
SELECTIONS = {
    "combinations": ["--feature-powerset", "--depth", "2"],
    "no-default": ["--no-default-features"],
}
# These identify workload bookkeeping, never executable-affecting selections.
OPERATIONAL = installation_context.OPERATIONAL | {
    "PSE_ACCEPTANCE_OUTPUT", "PSE_FEATURE_REPORT", "PSE_FEATURE_RESUME_FROM",
    "PSE_FEATURE_CAPTURE", "PSE_FEATURE_ATTEMPT", "CARGO_HACK_CARGO_SRC",
    "SHLVL", "_", "PWD", "OLDPWD", "PSE_NEXTEST_DEFAULT_THREADS",
}


def compiler_context(root: Path, env: dict[str, str]) -> dict:
    """Reuse the closed compiler/config/native capture, preserving actual env."""
    context, _ = installation_context.capture(root, env)
    context["actual_environment"] = {
        name: hashlib.sha256(value.encode()).hexdigest()
        for name, value in sorted(env.items()) if name not in OPERATIONAL
    }
    # Native solver capabilities are supplied by the existing native operation.
    context["native_selection"] = {
        name: installation_context.tree(Path(value))
        for name, value in env.items()
        if name in {"IPOPT_DIR", "SCIPOPTDIR", "UNO_DIR", "PETSC_DIR", "PSE_ROOT_ISOLATION_DIR", "SUITESPARSE_INCLUDE_DIR", "SUITESPARSE_LIBRARY_DIR"} and value
    }
    return context


def enumerate_commands(root: Path, output: Path, selection: list[str], env: dict[str, str]) -> tuple[list[list[str]], dict]:
    path = output / "feature-enumeration.jsonl"
    path.write_text("")
    tool = cargo_tools.selected_identity("cargo-hack")
    command = [tool["path"], "hack", "--keep-going", "check", "--workspace", *selection, "--locked"]
    record = validation.execute(root, output, "feature-enumeration", command, {
        **env, "CARGO_HACK_CARGO_SRC": str(ROOT / "scripts/cargo_feature_check.py"), CAPTURE: str(path),
    })
    if record["status"] != "passed":
        raise ValueError(f"cargo-hack enumeration {record['status']}; see {output / record['log']}")
    commands = [json.loads(line) for line in path.read_text().splitlines()]
    if not commands or any(not isinstance(argv, list) or not argv or argv[0] != "check" or any(not isinstance(arg, str) for arg in argv) for argv in commands):
        raise ValueError("missing or invalid literal cargo-hack enumeration")
    return commands, {"tool": tool, "command": command, "capture_sha256": validation_receipts.digest(path)}


def run(root: Path, output: Path, mode: str, *, resume_from: Path | None = None, environment: dict[str, str] | None = None) -> int:
    env = dict(os.environ) if environment is None else environment.copy()
    if CAPTURE in env or ATTEMPT in env:
        raise ValueError("campaign refuses inherited adapter capture/execution controls")
    output.mkdir(parents=True, exist_ok=True)
    receipt_path = output / "feature-campaign.json"
    if receipt_path.exists():
        raise ValueError("new attempt output must not overwrite a campaign receipt")
    snapshot = validation.sources(root)
    commands, enumeration = enumerate_commands(root, output, SELECTIONS[mode], env)
    frozen = {
        "mode": mode, "enumeration": enumeration,
        "ordered_literal_argv": commands,
        "adapter_sha256": validation_receipts.digest(ROOT / "scripts/cargo_feature_check.py"),
        "inputs": input_identity("rust-product", snapshot, validation.relevant_environment(env)),
        "compiler_context": compiler_context(root, env),
        "force_validation": "arrow-unit-graph-force-validation-v1",
    }
    receipt = {"schema": SCHEMA, "baseline_failures": 0, "frozen": frozen, "combinations": [
        {"index": index, "literal_argv": argv, "status": "not_run", "attempts": []}
        for index, argv in enumerate(commands)
    ], "complete": False, "required_checks_covered": False, "source_unchanged": True, "evidence_kind": "executed", "parent": None}
    if resume_from is not None:
        parent = resume_from.resolve()
        receipt["parent"] = {"path": str(parent), "sha256": validation_receipts.digest(parent / "feature-campaign.json")}
        previous = validation_receipts.feature_campaign(parent)
        receipt["reuse_applicable"] = previous["frozen"] == frozen
        for current, old in zip(receipt["combinations"], previous["combinations"], strict=False):
            # Preserve failed/interrupted history even after a changed-input retry.
            current["history"] = {"campaign": str(parent), "index": old["index"], "status": old["status"]}
            if receipt["reuse_applicable"] and old["status"] == "passed":
                origin = old.get("origin", {"path": str(parent), "index": old["index"], "sha256": receipt["parent"]["sha256"]})
                validation_receipts.feature_combination_origin(origin, frozen, current["index"])
                current.update(status="passed", evidence_kind="unchanged-input-reuse", origin=origin)
                receipt["evidence_kind"] = "composed-execution-and-unchanged-input-reuse"
    validation.write_json(receipt_path, receipt)
    interrupted = False
    for combination in receipt["combinations"]:
        if combination["status"] == "passed" or interrupted:
            continue
        index = combination["index"]
        execution_path = output / f"feature-{index:05d}-execution.json"
        command = [str(ROOT / ".venv/bin/python"), str(ROOT / "scripts/cargo_feature_check.py"), *combination["literal_argv"]]
        combination["status"] = "running"
        validation.write_json(receipt_path, receipt)
        record = validation.execute(root, output, f"feature-{index:05d}", command, {**env, ATTEMPT: str(execution_path)})
        record["artifacts"] = {record["log"]: validation_receipts.digest(output / record["log"])}
        try:
            actual = json.loads(execution_path.read_text())
            validation_receipts.feature_execution(actual, combination["literal_argv"])
            record["execution"] = actual
            record["artifacts"][execution_path.name] = validation_receipts.digest(execution_path)
        except (OSError, ValueError, KeyError, TypeError) as error:
            record["report_errors"].append(f"missing/invalid correctness execution: {error}")
            if record["status"] == "passed":
                record["status"] = "failed"
        combination["attempts"].append(record)
        combination["status"] = record["status"]
        combination["evidence_kind"] = "executed"
        interrupted = record["status"] == "interrupted"
        validation.write_json(receipt_path, receipt)
    # A repair invalidates the entire applicable Rust closure, including passes
    # reached before the repair. Retained origin receipts stay unchanged.
    after = input_identity("rust-product", validation.sources(root), validation.relevant_environment(env))
    after_context = compiler_context(root, env)
    receipt["source_unchanged"] = frozen["inputs"] == after and frozen["compiler_context"] == after_context
    receipt["complete"] = all(combination["status"] in {"passed", "failed"} for combination in receipt["combinations"])
    receipt["required_checks_covered"] = receipt["source_unchanged"] and receipt["complete"] and all(combination["status"] == "passed" for combination in receipt["combinations"])
    validation.write_json(receipt_path, receipt)
    return 130 if interrupted else int(not receipt["required_checks_covered"])


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=SELECTIONS)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--resume-from", type=Path)
    args = parser.parse_args(argv)
    selected_report = os.environ.get("PSE_FEATURE_REPORT")
    output = args.output or (Path(selected_report) if selected_report else None)
    resume = args.resume_from or (Path(os.environ["PSE_FEATURE_RESUME_FROM"]) if os.environ.get("PSE_FEATURE_RESUME_FROM") else None)
    for name in (signal.SIGTERM, signal.SIGHUP):
        signal.signal(name, validation.interrupt)
    try:
        if selected_report is None:
            # Direct recipes use the existing assessment/resource owner. Its
            # nested recipe receives PSE_FEATURE_REPORT and executes once below.
            selected = validation.fresh_output(ROOT, output, "features-" + args.mode)
            gate = declared("features-combinations" if args.mode == "combinations" else "features-no-default")
            print(f"validation evidence: {selected}", flush=True)
            return validation.run_gates(ROOT, selected, [gate], reuse_from=resume)
        if output is None:
            raise ValueError("assessment feature output missing")
        return run(ROOT, output.resolve(), args.mode, resume_from=resume)
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"feature campaign refused: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
