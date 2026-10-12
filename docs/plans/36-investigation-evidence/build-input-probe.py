"""E-C01: same Cargo unit with the actual backend build script in a tiny workspace.

This does not compile the scientific backend or establish its rebuild latency.
All sources/targets are owned disposable copies; no shared cache is cleared.
"""
from __future__ import annotations
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parents[3]


def main():
    dependencies = tomllib.loads((ROOT / "Cargo.toml").read_text())["workspace"]["dependencies"]
    with tempfile.TemporaryDirectory(prefix="pse36-build-input-") as temporary:
        stage = Path(temporary)
        shutil.copy2(ROOT / "rust-toolchain.toml", stage / "rust-toolchain.toml")
        (stage / ".cargo").mkdir()
        shutil.copy2(ROOT / ".cargo/config.toml", stage / ".cargo/config.toml")
        pounce = dependencies["pounce-rs"]
        (stage / "Cargo.toml").write_text(
            '[workspace]\nmembers=["crates/probe","crates/downstream"]\nresolver="3"\n'
            '[workspace.dependencies.pounce-rs]\ngit=' + json.dumps(pounce["git"]) +
            '\nrev=' + json.dumps(pounce["rev"]) + '\n')
        probe = stage / "crates/probe"
        (probe / "src").mkdir(parents=True)
        (probe / "Cargo.toml").write_text(
            '[package]\nname="plan36-backend-build-probe"\nversion="0.0.1"\nedition="2024"\n'
            '[build-dependencies]\ntoml={version="' + dependencies["toml"]["version"] +
            '",default-features=false,features=["parse","serde","display"]}\ncc="' +
            dependencies["cc"] + '"\npkg-config="' + dependencies["pkg-config"] + '"\n')
        for name in ("build.rs", "build_receipt.rs"):
            shutil.copy2(ROOT / "crates/pse-backend-native" / name, probe / name)
        (probe / "src/lib.rs").write_text(
            'pub const MANIFEST: &str=include_str!(concat!(env!("OUT_DIR"),"/solver-manifest.txt"));\n'
            'pub const SOURCE: &str=include_str!(concat!(env!("OUT_DIR"),"/pounce-source.txt"));\n')
        downstream = stage / "crates/downstream"
        (downstream / "src").mkdir(parents=True)
        (downstream / "Cargo.toml").write_text(
            '[package]\nname="plan36-downstream-probe"\nversion="0.0.1"\nedition="2024"\n'
            '[dependencies]\nplan36-backend-build-probe={path="../probe"}\n')
        (downstream / "src/lib.rs").write_text('pub use plan36_backend_build_probe::{MANIFEST,SOURCE};\n')
        base = dict(os.environ)
        for name in ("IPOPT_DIR", "PKG_CONFIG_PATH", "PSE_ROOT_ISOLATION_DIR"):
            base.pop(name, None)
        sequence = []
        metadata = subprocess.run(["cargo", "metadata", "--offline", "--no-deps", "--format-version", "1"],
                                  cwd=stage, env=base, text=True, capture_output=True, check=True)
        target = Path(json.loads(metadata.stdout)["target_directory"])
        for label in ("cold-plain", "warm-plain", "native-environment", "return-plain"):
            environment = dict(base)
            if label == "native-environment":
                environment.update(IPOPT_DIR="/plan36-unused-native-prefix",
                                   PKG_CONFIG_PATH="/plan36-unused-native-prefix/lib/pkgconfig")
            result = subprocess.run(["cargo", "check", "--offline", "-p", "plan36-downstream-probe", "-vv"],
                                    cwd=stage, env=environment, text=True, capture_output=True)
            if result.returncode:
                raise RuntimeError(result.stderr)
            outputs = [p for p in target.rglob("*.txt") if p.name in ("solver-manifest.txt", "pounce-source.txt")]
            sequence.append({"mode": label,
                "build_script_ran": "cargo:rerun-if-changed=build.rs" in result.stdout + result.stderr,
                "probe_rustc": "--crate-name plan36_backend_build_probe" in result.stderr,
                "downstream_rustc": "--crate-name plan36_downstream_probe" in result.stderr,
                "dirty_reasons": [line.strip() for line in result.stderr.splitlines() if "Dirty " in line],
                "outputs": {p.name: {"sha256": hashlib.sha256(p.read_bytes()).hexdigest(),
                                     "mtime_ns": p.stat().st_mtime_ns} for p in outputs}})
        assert all(item["outputs"] for item in sequence), str(target)
        assert sequence[0]["outputs"].keys() == sequence[-1]["outputs"].keys()
        print(json.dumps({"scope": "actual current build script; miniature downstream; no native features",
                          "sequence": sequence}, indent=2))


if __name__ == "__main__":
    main()
