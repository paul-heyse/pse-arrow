# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Exercise the selected cache parser and incremental pass-through in disposable output."""

from __future__ import annotations

import json
import os
import random
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
            # An explicit experiment owns its foreground server. The managed
            # wrapper selects a shared admitted endpoint for ordinary builds.
            RUSTC_WRAPPER=env["PSE_SCCACHE_BINARY"],
            SCCACHE_DIR=str(root / "cache"),
            SCCACHE_SERVER_UDS=str(root / "server.sock"),
        )
        rustc = subprocess.check_output(
            ["rustup", "which", "rustc"], env=env, text=True
        ).strip()
        # In client-side mode, publication sends the compressed artifact over
        # daemon IPC. A tiny probe cannot detect the upstream 8 MiB frame limit.
        payload = random.Random(54).randbytes(4 * 1024**2)
        (root / "payload.bin").write_bytes(payload)
        source = root / "probe.rs"
        source.write_text(
            'pub static DATA: &[u8] = include_bytes!("payload.bin");\n'
            "pub fn value(x: u64) -> u64 { x.wrapping_add(1) }\n"
        )
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
        server = build_measurements.start_cache_server(env, root)
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
            consumer = root / "consumer.rs"
            consumer.write_text(
                "fn main() {\n"
                " assert_eq!(pse_cache_probe::value(41), 42);\n"
                f" assert_eq!(pse_cache_probe::DATA.len(), {len(payload)});\n"
                " assert_eq!(pse_cache_probe::DATA.iter().map(|&b| u64::from(b))"
                f".sum::<u64>(), {sum(payload)});\n"
                "}\n"
            )
            executable = root / "consumer"
            subprocess.run(
                [rustc, "--edition=2024", str(consumer), "--extern",
                 f"pse_cache_probe={output / 'libpse_cache_probe.rlib'}",
                 "-o", str(executable)],
                env=env, check=True,
            )
            subprocess.run([str(executable)], env=env, check=True)
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
                        "restored_payload_bytes": len(payload),
                        "restored_payload_checksum_verified": True,
                    },
                    indent=2,
                )
            )
        finally:
            build_measurements.stop_cache_server(server, env)


if __name__ == "__main__":
    main()
