# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Real Cargo content-freshness controls in an exclusively owned standalone fixture.

Run through scripts/pse-env with --output pointing to a new directory. This does not
compile workspace code or touch its target, sources, shared cache, or lockfile.
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def probe(output: Path) -> dict:
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    source = output / "source"
    source.mkdir()
    (source / "src").mkdir()
    (source / ".cargo").mkdir()
    (source / "value/src").mkdir(parents=True)
    shutil.copyfile(ROOT / "rust-toolchain.toml", source / "rust-toolchain.toml")
    shutil.copyfile(ROOT / ".cargo/config.toml", source / ".cargo/config.toml")
    (source / "Cargo.toml").write_text(
        '[package]\nname="pse-freshness-probe"\nversion="0.0.0"\nedition="2024"\n'
        '[workspace]\nmembers=["value"]\n'
        '[dependencies]\nlocal-value={path="value"}\n'
    )
    (source / "value/Cargo.toml").write_text(
        '[package]\nname="local-value"\nversion="0.0.0"\nedition="2024"\n'
    )
    ordinary = source / "src/ordinary.rs"
    ordinary.write_text("pub const VALUE: u32 = 11;\n")
    included = source / "generated.rs"
    included.write_text("const INCLUDED: u32 = 21;\n")
    dependency = source / "value/src/lib.rs"
    dependency.write_text("pub const VALUE: u32 = 51;\n")
    declared = source / "input.txt"
    declared.write_text("41\n")
    (source / "src/main.rs").write_text(
        'mod ordinary;\ninclude!("../generated.rs");\n'
        'include!(concat!(env!("OUT_DIR"), "/generated.rs"));\n'
        'fn main() { println!("{}:{}:{}:{}:{}", ordinary::VALUE, INCLUDED, '
        "BUILD_FILE, BUILD_ENV, local_value::VALUE); }\n"
    )
    (source / "build.rs").write_text(
        "use std::{env, fs, io::Write, path::PathBuf};\n"
        "fn main() {\n"
        ' println!("cargo::rerun-if-changed=input.txt");\n'
        ' println!("cargo::rerun-if-env-changed=PSE_FRESHNESS_VALUE");\n'
        " let mut log = fs::OpenOptions::new().create(true).append(true)"
        '.open(env::var("PSE_FRESHNESS_RUNS").unwrap()).unwrap();\n'
        ' writeln!(log, "ran").unwrap();\n'
        ' let input = fs::read_to_string("input.txt").unwrap();\n'
        ' let value = env::var("PSE_FRESHNESS_VALUE").unwrap();\n'
        ' let bytes = format!("const BUILD_FILE: u32 = {};\\nconst BUILD_ENV: u32 = {};\\n", '
        "input.trim(), value);\n"
        ' let dest = PathBuf::from(env::var("OUT_DIR").unwrap()).join("generated.rs");\n'
        " if fs::read_to_string(&dest).ok().as_deref() != Some(bytes.as_str()) { "
        "fs::write(dest, bytes).unwrap(); }\n}\n"
    )
    wrapper = output / "rustc-wrapper"
    wrapper.write_text(
        f"#!{sys.executable}\n"
        "import json, os, sys\n"
        "args = sys.argv[1:]\n"
        "if '--crate-name' in args and any(arg.startswith('--emit') for arg in args):\n"
        " with open(os.environ['PSE_FRESHNESS_RUSTC'], 'a') as log:\n"
        "  log.write(json.dumps(args) + '\\n')\n"
        "os.execv(args[0], args)\n"
    )
    wrapper.chmod(0o755)
    rustc_log = output / "rustc.jsonl"
    build_log = output / "build-script.txt"
    env = dict(os.environ)
    for key in (
        "CARGO_TARGET_DIR",
        "PSE_CARGO_TARGET_DIR",
        "CARGO_BUILD_TARGET_DIR",
        "CARGO_BUILD_BUILD_DIR",
        "CARGO_BUILD_FINGERPRINT",
        "RUSTC_WORKSPACE_WRAPPER",
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "RUSTDOCFLAGS",
        "CARGO_ENCODED_RUSTDOCFLAGS",
        "CARGO_INCREMENTAL",
    ):
        env.pop(key, None)
    env.update(
        RUSTC_WRAPPER=str(wrapper),
        PSE_FRESHNESS_RUSTC=str(rustc_log),
        PSE_FRESHNESS_RUNS=str(build_log),
        PSE_FRESHNESS_VALUE="31",
    )
    commands = []

    def run(command: list[str]) -> subprocess.CompletedProcess:
        result = subprocess.run(
            command, cwd=source, env=env, text=True, capture_output=True, check=False
        )
        commands.append(
            {
                "argv": command,
                "exit": result.returncode,
                "stdout": result.stdout,
                "stderr": result.stderr,
            }
        )
        (output / "commands.json").write_text(json.dumps(commands, indent=2) + "\n")
        result.check_returncode()
        return result

    def lines(path: Path) -> list[str]:
        return path.read_text().splitlines() if path.exists() else []

    run(["cargo", "generate-lockfile", "--offline"])
    metadata = json.loads(
        run(
            [
                "cargo",
                "metadata",
                "--offline",
                "--locked",
                "--no-deps",
                "--format-version=1",
            ]
        ).stdout
    )
    target = source / "target"
    if Path(metadata["target_directory"]).resolve() != target:
        raise ValueError("Cargo target escaped the private fixture")
    cases = []

    def build(
        name: str,
        expected: str,
        *,
        recompile: bool | None,
        script: bool,
        clean_compare: bool = False,
    ) -> None:
        rustc_before, script_before = len(lines(rustc_log)), len(lines(build_log))
        result = run(
            ["cargo", "build", "--offline", "--locked", "--message-format=json"]
        )
        launches = [json.loads(row) for row in lines(rustc_log)[rustc_before:]]
        script_runs = len(lines(build_log)) - script_before
        observed = run([str(target / "debug/pse-freshness-probe")]).stdout.strip()
        artifacts = [
            item
            for line in result.stdout.splitlines()
            if (item := json.loads(line)).get("reason") == "compiler-artifact"
        ]
        case = {
            "name": name,
            "rustc_crates": [args[args.index("--crate-name") + 1] for args in launches],
            "build_script_runs": script_runs,
            "output": observed,
            "fresh": [
                {"name": item["target"]["name"], "fresh": item["fresh"]}
                for item in artifacts
            ],
        }
        cases.append(case)
        if observed != expected:
            raise AssertionError(f"{name}: output {observed!r} != {expected!r}")
        if recompile is not None and bool(launches) != recompile:
            raise AssertionError(
                f"{name}: unexpected rustc launches {case['rustc_crates']}"
            )
        if bool(script_runs) != script:
            raise AssertionError(f"{name}: unexpected build script runs {script_runs}")
        if clean_compare:
            run(["cargo", "clean", "--offline", "--quiet"])
            run(["cargo", "build", "--offline", "--locked", "--message-format=json"])
            clean = run([str(target / "debug/pse-freshness-probe")]).stdout.strip()
            case["clean_output"] = clean
            if observed != clean:
                raise AssertionError(f"{name}: clean recomputation differs")
        (output / "cases.json").write_text(json.dumps(cases, indent=2) + "\n")

    def touch(path: Path) -> None:
        time.sleep(0.05)
        os.utime(path, None)

    def replace(
        path: Path, before: str, after: str, *, preserve_mtime: bool = False
    ) -> None:
        original_stat = path.stat()
        original = path.read_text()
        if before not in original:
            raise AssertionError(f"Missing replacement input in {path}")
        path.write_text(original.replace(before, after))
        if preserve_mtime:
            os.utime(path, ns=(original_stat.st_atime_ns, original_stat.st_mtime_ns))

    build("initial", "11:21:41:31:51", recompile=True, script=True)
    build("unchanged", "11:21:41:31:51", recompile=False, script=False)
    touch(ordinary)
    build("ordinary-identical-touch", "11:21:41:31:51", recompile=False, script=False)
    ordinary.write_bytes(ordinary.read_bytes())
    build("ordinary-identical-restore", "11:21:41:31:51", recompile=False, script=False)
    touch(included)
    build("included-identical-touch", "11:21:41:31:51", recompile=False, script=False)
    replace(ordinary, "= 11", "= 12", preserve_mtime=True)
    build(
        "ordinary-same-length-preserved-mtime-edit",
        "12:21:41:31:51",
        recompile=True,
        script=False,
        clean_compare=True,
    )
    replace(ordinary, "= 12", "= 123")
    build(
        "ordinary-real-edit",
        "123:21:41:31:51",
        recompile=True,
        script=False,
        clean_compare=True,
    )
    replace(included, "= 21", "= 22", preserve_mtime=True)
    build(
        "included-same-length-preserved-mtime-edit",
        "123:22:41:31:51",
        recompile=True,
        script=False,
        clean_compare=True,
    )
    replace(dependency, "= 51", "= 52", preserve_mtime=True)
    build(
        "dependency-same-length-preserved-mtime-edit",
        "123:22:41:31:52",
        recompile=True,
        script=False,
        clean_compare=True,
    )
    touch(declared)
    build(
        "declared-input-identical-touch-mtime-exception",
        "123:22:41:31:52",
        recompile=None,
        script=True,
        clean_compare=True,
    )
    replace(declared, "41", "42")
    build(
        "declared-input-same-length-edit",
        "123:22:42:31:52",
        recompile=True,
        script=True,
        clean_compare=True,
    )
    env["PSE_FRESHNESS_VALUE"] = "32"
    build(
        "declared-environment-edit",
        "123:22:42:32:52",
        recompile=True,
        script=True,
        clean_compare=True,
    )
    return {
        "status": "passed",
        "source": str(source),
        "target": str(target),
        "compiler": run(["rustc", "-vV"]).stdout,
        "cargo": run(["cargo", "-vV"]).stdout,
        "cases": cases,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--output",
        type=Path,
        required=True,
        help="New directory for private source/target and retained evidence",
    )
    args = parser.parse_args()
    result = probe(args.output)
    (args.output / "receipt.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
