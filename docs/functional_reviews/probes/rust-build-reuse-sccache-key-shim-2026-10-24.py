#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""RUSTC_WRAPPER shim for the accompanying Rust build-reuse review's miss attribution.

Records the inputs sccache 0.17/0.18 hashes for a Rust compile (src/compiler/rust.rs):
arguments minus --extern/-L/--out-dir/--check-cfg/--diagnostic-width, CARGO_* variables
minus MAKEFLAGS/REGISTRIES_*/BUILD_JOBS/ENCODED_RUSTFLAGS, and the cwd. Extern and source
contents and dep-info env-deps are hashed by sccache but not recorded here. Delegates every
call unchanged to the sccache binary named by PROBE_SCCACHE and appends one JSON line per
call to PROBE_LOG.

Use it against a probe-owned foreground server, never the managed one, and name the
wrapper file anything but `sccache` so `scripts/build_environment.py` leaves it explicit:

    PROBE_SCCACHE=$(command -v sccache) PROBE_LOG=calls.jsonl \
    RUSTC_WRAPPER=path/to/this-shim SCCACHE_SERVER_UDS=probe.sock SCCACHE_CLIENT_SIDE=0 \
    SCCACHE_LOCAL_RW_MODE=READ_ONLY scripts/pse-env -- cargo check --target-dir target/probe

Hit/miss per crate comes from the probe server's own log (`SCCACHE_LOG=
sccache::compiler::compiler=debug` on the server). Setting SCCACHE_LOG on the client
silently disables client-side mode, so attribution runs use server mode; the key
computation is the same code in both modes. Pass the target directory as `--target-dir`,
never `CARGO_TARGET_DIR`, which would itself enter the key.
"""

from __future__ import annotations

import hashlib
import json
import os
import subprocess
import sys
import time

EXCLUDED_ARGS = {"--extern", "-L", "--check-cfg", "--out-dir", "--diagnostic-width"}
EXCLUDED_ENV = {"CARGO_MAKEFLAGS", "CARGO_BUILD_JOBS", "CARGO_ENCODED_RUSTFLAGS"}


def digest(value: list[str] | dict[str, str]) -> str:
    return hashlib.sha256(json.dumps(value, sort_keys=True).encode()).hexdigest()[:12]


def hashed_arguments(arguments: list[str]) -> list[str]:
    kept: list[str] = []
    index = 0
    while index < len(arguments):
        argument = arguments[index]
        if argument.split("=", 1)[0] in EXCLUDED_ARGS:
            index += 1 if "=" in argument else 2
            continue
        kept.append(argument)
        index += 1
    return kept


def main() -> int:
    command = sys.argv[1:]
    arguments = hashed_arguments(command[1:])
    environment = {
        name: value
        for name, value in os.environ.items()
        if name.startswith("CARGO_")
        and name not in EXCLUDED_ENV
        and not name.startswith("CARGO_REGISTRIES_")
    }
    crate = (
        command[command.index("--crate-name") + 1] if "--crate-name" in command else None
    )
    started = time.monotonic()
    code = subprocess.call([os.environ["PROBE_SCCACHE"], *command])
    record = {
        "crate": crate,
        "pkg": os.environ.get("CARGO_PKG_NAME"),
        "ver": os.environ.get("CARGO_PKG_VERSION"),
        "manifest": os.environ.get("CARGO_MANIFEST_DIR"),
        "cwd": os.getcwd(),
        "seconds": round(time.monotonic() - started, 3),
        "code": code,
        "args_key": digest(arguments),
        "env_key": digest(environment),
        "env": environment,
        "args": arguments,
    }
    with open(os.environ["PROBE_LOG"], "a") as log:
        log.write(json.dumps(record) + "\n")
    return code


if __name__ == "__main__":
    sys.exit(main())
