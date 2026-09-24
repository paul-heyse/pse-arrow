#!/usr/bin/env python3
"""Run locked public-consumer assertions in an external capsule and retain exact receipts."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
from datetime import UTC, datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "build/fixtures/probe-crate"
EVIDENCE = ROOT / "skill_improvement/evidence/implementation"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--capsule", type=Path, required=True)
    parser.add_argument("--target-cache", type=Path)
    parser.add_argument("--jobs", type=int, default=4)
    args = parser.parse_args()
    capsule = args.capsule.expanduser().resolve()
    if capsule.is_relative_to(ROOT) or any(
        (p / ".git").exists() for p in [capsule, *capsule.parents]
    ):
        parser.error("Probe capsule must be outside working repositories and the skill")
    if args.jobs < 1:
        parser.error("--jobs must be positive")
    EVIDENCE.mkdir(parents=True, exist_ok=True)
    capture_dir = EVIDENCE / "captures"
    capture_dir.mkdir(exist_ok=True)
    for previous in capture_dir.glob("*.json"):
        previous.unlink()
    source_files = [p for p in FIXTURE.rglob("*") if p.is_file()] + [
        Path(__file__).resolve()
    ]
    sources = {
        str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
        for p in source_files
    }
    receipt = {
        "state": "not_run",
        "started_at": datetime.now(UTC).isoformat(),
        "sources": sources,
        "passed_tests": [],
        "commands": [],
        "scope": "Public consumer contracts; SDK export is local, not OTLP receipt.",
    }
    if shutil.which("cargo") is None:
        receipt.update(state="blocked", reason="cargo is not on PATH")
    else:
        capsule.mkdir(parents=True, exist_ok=True)
        destination = capsule / "probe-crate"
        shutil.copytree(FIXTURE, destination, dirs_exist_ok=True)
        environment = dict(os.environ)
        for key in ("RUSTFLAGS", "RUSTDOCFLAGS", "CARGO_BUILD_TARGET_DIR"):
            environment.pop(key, None)
        environment.update(
            CARGO_TARGET_DIR=str((args.target_cache or capsule / "target").resolve()),
            CARGO_BUILD_JOBS=str(args.jobs),
            TRACE_PROBE_OUTPUT=str(capture_dir),
        )

        def execute(label: str, command: list[str]) -> subprocess.CompletedProcess[str]:
            done = subprocess.run(
                command,
                cwd=destination,
                env=environment,
                capture_output=True,
                text=True,
                check=False,
            )
            (EVIDENCE / f"{label}.log").write_text(
                "Command: " + " ".join(command) + "\n" + done.stdout + done.stderr
            )
            receipt["commands"].append(
                {
                    "command": command,
                    "exit_code": done.returncode,
                    "log": f"{label}.log",
                }
            )
            return done

        toolchain = json.loads(
            (ROOT / "build/manifests/datafusion-tracing.json").read_text()
        )["tools"]["probe_toolchain"]["toolchain"]
        compiler = execute(
            "probe-compiler", ["rustc", f"+{toolchain}", "--version", "--verbose"]
        )
        if compiler.returncode:
            receipt.update(state="blocked", reason=f"rustc +{toolchain} is unavailable")
        else:
            test = execute(
                "contract-probes",
                [
                    "cargo",
                    f"+{toolchain}",
                    "test",
                    "--release",
                    "--locked",
                    "--test",
                    "contracts",
                    "--",
                    "--test-threads=1",
                ],
            )
            passed = re.findall(r"^test (\S+) \.\.\. ok$", test.stdout, re.MULTILINE)
            receipt.update(
                state="passed" if test.returncode == 0 and passed else "failed",
                passed_tests=passed,
            )
            metadata = execute(
                "probe-metadata",
                [
                    "cargo",
                    f"+{toolchain}",
                    "metadata",
                    "--locked",
                    "--format-version",
                    "1",
                ],
            )
            if metadata.returncode == 0:
                data = json.loads(metadata.stdout)
                features = {n["id"]: n["features"] for n in data["resolve"]["nodes"]}
                profile = {
                    "compiler": compiler.stdout,
                    "lock_sha256": hashlib.sha256(
                        (FIXTURE / "Cargo.lock").read_bytes()
                    ).hexdigest(),
                    "packages": [
                        {
                            "name": p["name"],
                            "version": p["version"],
                            "source": p["source"],
                            "features": features.get(p["id"], []),
                        }
                        for p in data["packages"]
                    ],
                }
                (EVIDENCE / "probe-profile.json").write_text(
                    json.dumps(profile, indent=2) + "\n"
                )
            else:
                receipt.update(
                    state="failed", reason="Cannot record resolved dependency profile"
                )
            receipt["captures"] = {
                str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
                for p in sorted(capture_dir.glob("*.json"))
            }
    receipt["finished_at"] = datetime.now(UTC).isoformat()
    (EVIDENCE / "probe-results.json").write_text(json.dumps(receipt, indent=2) + "\n")
    sys.stdout.write(json.dumps(receipt, indent=2) + "\n")
    return 0 if receipt["state"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
