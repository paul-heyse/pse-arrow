#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
set -euo pipefail

review_root="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
review_source="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
review_work="$(mktemp -d "${TMPDIR:-/tmp}/pse-full-capability-review.XXXXXX")"
trap 'rm -rf -- "$review_work"' EXIT

"$review_root/.venv/bin/python" - "$review_root" "$review_source" "$review_work" <<'PY'
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tomllib

root, source, scratch = map(Path, sys.argv[1:])
workspace = tomllib.loads((root / "Cargo.toml").read_text())
toolchain = tomllib.loads((root / "rust-toolchain.toml").read_text())["toolchain"]["channel"]
dependencies = workspace["workspace"]["dependencies"]
selected = [
    name for name in dependencies
    if name in ("arrow", "parquet", "tokio", "blake3", "object_store")
    or name.startswith("arrow-") or name.startswith("datafusion")
]

def toml_value(value):
    if isinstance(value, str):
        return json.dumps(value)
    if isinstance(value, bool):
        return str(value).lower()
    if isinstance(value, list):
        return "[" + ", ".join(toml_value(item) for item in value) + "]"
    if isinstance(value, dict):
        return "{" + ", ".join(key + " = " + toml_value(item) for key, item in value.items()) + "}"
    return str(value)

lines = [
    "[package]", 'name = "pse-full-capability-review-probe"', 'version = "0.0.0"',
    'edition = "2024"', "[workspace]", "[dependencies]",
]
lines += [name + " = " + toml_value(dependencies[name]) for name in selected]
lines += [
    "[features]", 'force-validate = ["arrow/force_validate"]',
    "[profile.dev]", 'debug = "line-tables-only"',
    '[profile.dev.package."*"]', "opt-level = 2", "debug = false",
]
(scratch / "Cargo.toml").write_text("\n".join(lines) + "\n")
(scratch / "Cargo.lock").write_bytes((root / "Cargo.lock").read_bytes())
(scratch / "toolchain").write_text(toolchain + "\n")
(scratch / "src").mkdir()
(scratch / "src/main.rs").write_bytes((source / "probe.rs").read_bytes())

# Adding the scratch root changes the copied lockfile. Allow that local change,
# then assert that every resolved third-party version already exists in the
# repository lock. The execution below is locked and offline.
command = [
    "rustup", "run", toolchain, "cargo", "metadata", "--offline",
    "--format-version", "1", "--manifest-path", str(scratch / "Cargo.toml"),
    "--features", "force-validate",
]
metadata = json.loads(subprocess.check_output(command, text=True))
original = {
    (package["name"], package["version"])
    for package in tomllib.loads((root / "Cargo.lock").read_text())["package"]
}
new_versions = [
    (package["name"], package["version"])
    for package in metadata["packages"]
    if package["source"] is not None
    and (package["name"], package["version"]) not in original
]
if new_versions:
    raise SystemExit(f"Probe resolved versions absent from repository lock: {new_versions}")
print("Repository HEAD:", subprocess.check_output(["git", "-C", str(root), "rev-parse", "HEAD"], text=True).strip())
print("Cargo.lock SHA-256:", hashlib.sha256((root / "Cargo.lock").read_bytes()).hexdigest())
print("Probe SHA-256:", hashlib.sha256((source / "probe.rs").read_bytes()).hexdigest())
print("Rust toolchain:", toolchain)
print("Mode: dev; arrow/force_validate enabled; offline; workspace version changes: 0; failure baseline: 0")
print("Resolved package count:", len(metadata["packages"]))
for name in ("arrow", "datafusion", "object_store", "blake3"):
    print(name + ":", sorted({package["version"] for package in metadata["packages"] if package["name"] == name}))
PY

IFS= read -r review_toolchain < "$review_work/toolchain"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$review_root/target}"
rustup run "$review_toolchain" cargo run --offline --locked \
    --manifest-path "$review_work/Cargo.toml" --features force-validate
