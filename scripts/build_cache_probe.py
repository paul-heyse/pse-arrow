# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Exercise the selected cache parser and incremental pass-through in disposable output."""

from __future__ import annotations

import json
import os
import subprocess
import tempfile
from pathlib import Path

from scripts import build_environment, build_measurements


def main() -> None:
    env = build_environment.configure(
        build_environment.ROOT, dict(os.environ), cache="on"
    )
    with tempfile.TemporaryDirectory(prefix="pse-cache-probe-") as directory:
        root = Path(directory)
        # This server and directory belong only to this probe.
        env.update(
            SCCACHE_DIR=str(root / "cache"),
            SCCACHE_SERVER_UDS=str(root / "server.sock"),
        )
        rustc = subprocess.check_output(
            ["rustup", "which", "rustc"], env=env, text=True
        ).strip()
        source = root / "probe.rs"
        source.write_text("pub fn value(x: u64) -> u64 { x.wrapping_add(1) }\n")
        output = root / "objects"
        output.mkdir()
        command = [
            env["RUSTC_WRAPPER"],
            rustc,
            "--crate-name",
            "pse_cache_probe",
            "--crate-type",
            "rlib",
            "--emit=link,dep-info",
            "--out-dir",
            str(output),
            str(source),
        ]
        if "nightly" in rustc:
            command += [
                flag
                for flag in build_environment.effective_flags(
                    build_environment.ROOT, env
                )
                if flag.startswith("-Zthreads=")
            ]
        try:
            before = build_measurements.cache_stats(env)
            subprocess.run(command, env=env, check=True)
            subprocess.run(command, env=env, check=True)
            delta = build_measurements.counter_delta(
                before, build_measurements.cache_stats(env)
            )
            hits = delta["stats"]["cache_hits"]["counts"].get("Rust", 0)
            if hits < 1:
                raise ValueError("eligible repeated Rust compilation had no cache hit")
            subprocess.run(
                [*command, "-C", f"incremental={root / 'incremental'}"],
                env=env,
                check=True,
            )
            print(
                json.dumps(
                    {
                        "compiler": rustc,
                        "cache_delta": delta,
                        "incremental_passthrough": True,
                    },
                    indent=2,
                )
            )
        finally:
            subprocess.run(
                [env["RUSTC_WRAPPER"], "--stop-server"],
                env=env,
                check=False,
                stdout=subprocess.DEVNULL,
            )


if __name__ == "__main__":
    main()
