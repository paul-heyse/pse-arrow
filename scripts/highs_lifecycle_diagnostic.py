# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Run the actual HiGHS callback lifecycle under a selected, identified diagnostic build."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import time
import subprocess
from pathlib import Path

from scripts import native_operation, native_tests

ROOT = Path(__file__).resolve().parents[1]
NORMAL = "highs::mip_tests::callback_binding_normal_and_interrupted_teardown_keeps_context_until_native_return"


def digest(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def run(command: list[str], environment: dict[str, str], output: Path, name: str, timeout: float) -> dict:
    """Use the existing child-scope owner and retain its actual drain observation."""
    observation={}
    with (output / f"{name}.stdout.txt").open("wb") as stdout, (output / f"{name}.stderr.txt").open("wb") as stderr:
        try:
            code=native_operation.run_child_scope(command,environment,cwd=ROOT,observation=observation,deadline=time.monotonic()+timeout,stdout=stdout.fileno(),stderr=stderr.fileno())
            return {"command":command,"exit_code":code,"timed_out":False,"ownership":observation}
        except subprocess.TimeoutExpired:
            return {"command":command,"exit_code":None,"timed_out":True,"ownership":observation}



def qualify_asan(normal: dict, archive_instrumented: bool, rust_instrumented: bool) -> bool:
    """Qualify actual lifecycle execution only, with no injected-fault sensitivity claim."""
    return archive_instrumented and rust_instrumented and not normal["timed_out"] and normal["exit_code"] == 0



def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    parser.add_argument("--mode", choices=("asan", "gdb"), default="asan")
    parser.add_argument("--build-timeout", type=float, default=1800)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    environment = dict(os.environ)
    record = {"schema": "highs-lifecycle-diagnostic-v1", "mode": args.mode, "status": "incomplete", "scope": "one actual CallbackBinding normal/interrupted teardown", "baseline_failures": 0, "sources": {path: digest(ROOT / path) for path in ("Cargo.lock", "rust-toolchain.toml", "crates/pse-backend-native/src/highs.rs", "crates/pse-backend-native/src/highs/mip_tests.rs", "crates/pse-backend-native/src/highs_lifecycle.rs")}, "limits": ["not a solver-fleet or ThreadSanitizer qualification", "standard library and unrelated opaque native libraries are outside selected instrumentation", "deterministic schedules are separate ordinary tests, not sanitizer interleaving coverage"]}
    try:
        target = subprocess.check_output(["rustc", "-vV"], text=True).split("host: ")[1].splitlines()[0]
        arguments = ["cargo", "test", "--locked", "-p", "pse-backend-native", "--lib", "--features", "highs", "--no-run", "--message-format=json"]
        if args.mode == "asan":
            # The isolated target prevents mixing ordinary and instrumented artifacts.
            environment["PSE_CARGO_TARGET_DIR"] = str(ROOT / "target" / "diagnostics" / "highs-asan")
            environment.pop("CARGO_TARGET_DIR", None)
            environment.pop("CARGO_ENCODED_RUSTFLAGS", None)
            environment["RUSTFLAGS"] = "-Zsanitizer=address -Cforce-frame-pointers=yes -Clink-arg=-fuse-ld=mold"
            environment["CFLAGS"] = "-fsanitize=address -fno-omit-frame-pointer"
            environment["CXXFLAGS"] = "-fsanitize=address -fno-omit-frame-pointer"
            environment["ASAN_OPTIONS"] = "detect_leaks=1:halt_on_error=1"
            arguments += ["--target", target]
        # Keep the existing correctness resolver and environment/native build owners.
        # Resolve inside that environment: its unit graph is built with the actual flags.
        command = [str(ROOT / "scripts/pse-env"), "--resource-class", "build", "--native=solver", "--", str(ROOT / ".venv/bin/python"), "-m", "scripts.arrow_validation", *arguments[1:]]
        record["build"] = run(command, environment, output, "build", args.build_timeout)
        record["instrumentation_environment"] = {key: environment.get(key) for key in ("PSE_CARGO_TARGET_DIR", "RUSTFLAGS", "CFLAGS", "CXXFLAGS", "ASAN_OPTIONS", "CARGO_BUILD_JOBS")}
        if record["build"]["exit_code"] != 0:
            record["status"] = "unsupported-build" if not record["build"]["timed_out"] else "build-timeout"
            return 1
        messages = [json.loads(line) for line in (output / "build.stdout.txt").read_text().splitlines() if line.startswith("{")]
        binaries = [Path(message["executable"]) for message in messages if message.get("reason") == "compiler-artifact" and message.get("executable") and message["target"]["name"] == "pse_backend_native" and message["profile"]["test"]]
        if len(binaries) != 1:
            raise ValueError("expected one actual selected native test executable")
        binary = binaries[0]
        record["native_identity"] = native_tests.native_provenance({"selection": "highs lifecycle", "target": target}, [str(binary)], environment=environment)
        record["binary"] = {"path": str(binary), "sha256": digest(binary)}
        inventory=subprocess.check_output([str(binary),"--list","--format","terse"],env=environment,text=True)
        if f"{NORMAL}: test" not in inventory:
            raise ValueError("actual executable does not contain the selected lifecycle control")
        (output/"test-inventory.txt").write_text(inventory)
        if args.mode == "gdb":
            # Reuse the existing active-diagnostic GDB owner, including inferior status.
            command = [str(ROOT / "scripts/pse-env"), "--resource-class", "build", "--native=solver", "--", str(ROOT / ".venv/bin/python"), "-m", "scripts.active_diagnostics", "gdb-launch", "--output", str(output / "gdb"), "--", str(binary), "--exact", NORMAL, "--nocapture"]
            record["normal"] = run(command, environment, output, "normal", 120)
            record["status"] = "ordinary-gdb-route" if record["normal"]["exit_code"] == 0 else "gdb-failed"
            return int(record["normal"]["exit_code"] != 0)
        native = [message for message in messages if message.get("reason") == "build-script-executed" and "highs-sys" in message.get("package_id", "")]
        if len(native) != 1:
            raise ValueError("expected one actual Cargo-owned HiGHS supplier")
        archive = Path(native[0]["out_dir"]) / "lib/libhighs.a"
        cache = Path(native[0]["out_dir"]) / "build/CMakeCache.txt"
        cache_text = cache.read_text()
        symbols = subprocess.check_output(["nm", "-u", str(archive)], text=True, stderr=subprocess.DEVNULL)
        rust_symbols = subprocess.check_output(["nm", str(binary)], text=True, stderr=subprocess.DEVNULL)
        native_instrumented = "__asan_" in symbols and any("CMAKE_CXX_FLAGS:" in line and "-fsanitize=address" in line for line in cache_text.splitlines())
        rust_instrumented = "__asan_init" in rust_symbols
        record["highs_supplier"] = {"package": native[0]["package_id"], "archive": str(archive), "sha256": digest(archive), "cmake_cache": str(cache), "cmake_cache_sha256": digest(cache), "instrumented": native_instrumented}
        record["rust_instrumented"] = rust_instrumented
        if not native_instrumented or not rust_instrumented:
            raise ValueError("selected Rust/native instrumentation was not observed")
        record["normal"] = run([str(binary), "--exact", NORMAL, "--nocapture"], environment, output, "normal", 120)
        adopted = qualify_asan(record["normal"], native_instrumented, rust_instrumented)
        record["sensitivity_control"] = "not_run: deliberately invalid memory control withdrawn after content-safeguard denial; maintainer selected ordinary lifecycle coverage"
        record["status"] = "tested-normal-lifecycle" if adopted else "not-qualified"
        return 0 if adopted else 1
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        record["status"] = "unavailable"
        record["error"] = str(error)
        return 1
    finally:
        (output / "lifecycle.json").write_text(json.dumps(record, indent=2) + "\n")
        print(output / "lifecycle.json")


if __name__ == "__main__":
    raise SystemExit(main())
