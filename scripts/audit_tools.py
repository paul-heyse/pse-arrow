# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Preserve audit tool exits separately from nonblocking dependency findings."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import time
from pathlib import Path

from scripts.validation import command_env, write_json

COMMANDS = {
    "audit-dependencies": [
        "cargo",
        "deny",
        "--format",
        "json",
        "--all-features",
        "--locked",
        "check",
    ],
    "audit-advisories": ["cargo", "audit", "--json", "--deny", "warnings"],
    "audit-shear": ["cargo", "shear", "--format", "json", "--locked"],
    "audit-machete": ["cargo", "machete", "--with-metadata"],
}


def finding_status(gate: str, code: int, stdout: str, stderr: str) -> str:
    if code == 0:
        return "passed"
    try:
        if gate == "audit-dependencies":
            # cargo-deny check uses a four-bit error mask, not a Boolean exit.
            records = [json.loads(line) for line in stderr.splitlines() if line.strip()]
            summaries = [r["fields"] for r in records if r.get("type") == "summary"]
            if len(summaries) != 1 or any(
                r.get("type") not in {"diagnostic", "summary"} for r in records
            ):
                return "failed"
            summary = summaries[0]
            mask = sum(
                bit
                for name, bit in (
                    ("advisories", 1),
                    ("bans", 2),
                    ("licenses", 4),
                    ("sources", 8),
                )
                if summary[name]["errors"] > 0
            )
            return "findings" if mask == code and mask > 0 else "failed"
        if code != 1:
            return "failed"
        if gate == "audit-machete":
            # Documented exit 1 is unused dependencies; exit 2 is a tool error.
            return "findings"
        if gate == "audit-advisories":
            report = json.loads(stdout)
            if report.get("vulnerabilities", {}).get("found") or report.get("warnings"):
                return "findings"
        elif gate == "audit-shear":
            report = json.loads(stdout)
            findings = report["findings"]
            summary = report["summary"]
            if (
                findings
                and all(
                    isinstance(item.get("code"), str)
                    and item["code"].startswith("shear/")
                    and item.get("severity") in {"warning", "error"}
                    for item in findings
                )
                and all(
                    summary[plural]
                    == sum(item["severity"] == singular for item in findings)
                    for plural, singular in (
                        ("errors", "error"),
                        ("warnings", "warning"),
                    )
                )
            ):
                return "findings"
    except (ValueError, KeyError, TypeError, AttributeError):
        pass
    return "failed"


def invoke(command: list[str], root: Path) -> dict:
    started = time.time()
    env = command_env()
    if command[:2] == ["cargo", "geiger"]:
        # Geiger invokes cargo clean before discovering compiler inputs. Never
        # give that operation the development, coverage or measurement target.
        env["CARGO_TARGET_DIR"] = str(root / "build/audits/geiger-target")
    try:
        result = subprocess.run(
            command,
            cwd=root,
            env=env,
            capture_output=True,
            text=True,
            check=False,
        )
        record = {
            "command": command,
            "target_directory": env.get("CARGO_TARGET_DIR"),
            "exit_code": result.returncode,
            "stdout": result.stdout,
            "stderr": result.stderr,
        }
    except OSError as error:
        record = {
            "command": command,
            "exit_code": None,
            "stdout": "",
            "stderr": str(error),
        }
    print(record["stdout"], end="", flush=True)
    print(record["stderr"], end="", flush=True)
    return {**record, "started": started, "elapsed_seconds": time.time() - started}


def unsafe_members(root: Path) -> list[list[str]]:
    metadata = subprocess.check_output(
        ["cargo", "metadata", "--no-deps", "--format-version", "1", "--locked"],
        cwd=root,
        env=command_env(),
    )
    value = json.loads(metadata)
    members = set(value["workspace_members"])
    return [
        [
            "cargo",
            "geiger",
            "--manifest-path",
            package["manifest_path"],
            "--package",
            package["name"],
            "--all-features",
            "--locked",
            "--output-format",
            "Json",
        ]
        for package in sorted(value["packages"], key=lambda package: package["name"])
        if package["id"] in members
    ]


def run(root: Path, gate: str, output: Path) -> int:
    started = time.time()
    records = []
    try:
        commands = (
            unsafe_members(root) if gate == "unsafe-surface" else [COMMANDS[gate]]
        )
        if not commands:
            records.append(
                {"status": "failed", "error": "no workspace members enumerated"}
            )
        for command in commands:
            record = invoke(command, root)
            if gate == "unsafe-surface":
                try:
                    parsed = json.loads(record["stdout"])
                    valid = bool(parsed) and isinstance(parsed, (dict, list))
                except ValueError:
                    valid = False
                record["status"] = (
                    "passed" if record["exit_code"] == 0 and valid else "failed"
                )
            else:
                record["status"] = finding_status(
                    gate, record["exit_code"], record["stdout"], record["stderr"]
                )
            records.append(record)
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        records.append({"status": "failed", "error": str(error)})
    status = (
        "failed"
        if any(r["status"] == "failed" for r in records)
        else "findings"
        if any(r["status"] == "findings" for r in records)
        else "passed"
    )
    output.mkdir(parents=True, exist_ok=True)
    write_json(
        output / f"{gate}-tool.json",
        {"gate": gate, "started": started, "status": status, "invocations": records},
    )
    # Preserve findings as a nonzero strict recipe result; the assessment owns policy.
    return 2 if status == "failed" else int(status == "findings")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("gate", choices=(*COMMANDS, "unsafe-surface"))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    output = Path(os.environ.get("PSE_ACCEPTANCE_OUTPUT", root / "build/audits"))
    return run(root, args.gate, output)


if __name__ == "__main__":
    raise SystemExit(main())
