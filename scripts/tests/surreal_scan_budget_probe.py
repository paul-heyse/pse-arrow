# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Run only TOOL08's instrumented native scanner test in a private source copy.

Invoke through scripts/pse-env --resource-class build -- .venv/bin/python -m
scripts.tests.surreal_scan_budget_probe --source /selected/patched/source
--scratch /new/owned/directory. No server or canonical database is opened. The
source's own target/ is excluded; this copy builds in its own default target/.
The 1001-entry case crosses a native batch boundary, not a performance benchmark.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
REVISION = "1ac2b1dba7cd177621a4cf5d507caebeb2414395"
PATCH = ROOT / "scripts/tests/fixtures/surreal-source/tool08-compound-range-budget.patch"
NATIVE_PATHS = (
    "surrealdb/core/src/exec",
    "surrealdb/core/src/kvs/util.rs",
    "surrealdb/idx/src",
)
TEST = "exec::index::iterator::btree::tool08_probe::tool08_compound_range_scan_stops_at_entry_budget"


def digest(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def git(source: Path, *arguments: str) -> bytes:
    return subprocess.check_output(["git", "-C", str(source), *arguments])


def source_files(source: Path) -> list[Path]:
    tracked = [Path(os.fsdecode(name)) for name in git(source, "ls-files", "-z").split(b"\0") if name]
    dependencies = source / ".pse-deps"
    if not dependencies.is_dir():
        raise ValueError("selected patched source must retain its .pse-deps inputs")
    return sorted(set(tracked) | {path.relative_to(source) for path in dependencies.rglob("*") if path.is_file()})


def snapshot(source: Path, paths: list[Path]) -> dict[str, str]:
    result = {}
    for relative in paths:
        path = source / relative
        if path.is_symlink() or not path.is_file():
            raise ValueError(f"source input must be a regular file: {relative}")
        result[relative.as_posix()] = digest(path)
    return result


def prepare(source: Path, scratch: Path) -> dict[str, object]:
    source = source.resolve(strict=True)
    scratch = scratch.resolve()
    if scratch.is_relative_to(source) or source.is_relative_to(scratch):
        raise ValueError("probe source and disposable scratch must be disjoint")
    if git(source, "rev-parse", "HEAD").decode().strip() != REVISION:
        raise ValueError("selected source is not the exact TOOL08 upstream revision")
    manifest = (source / "Cargo.toml").read_text()
    lock = (source / "Cargo.lock").read_text()
    if 'surrealmx = "=1.0.0"' not in manifest or 'name = "surrealmx"\nversion = "1.0.0"' not in lock:
        raise ValueError("selected source must use the patched MX 1.0.0 engine")
    if git(source, "diff", "HEAD", "--", *NATIVE_PATHS):
        raise ValueError("native planner/index/scanner inputs differ from pinned upstream")
    paths = source_files(source)
    hashes = snapshot(source, paths)
    scratch.mkdir(parents=True, exist_ok=False)
    copied = scratch / "source"
    for relative in paths:
        target = copied / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source / relative, target)
    if snapshot(copied, paths) != hashes or snapshot(source, paths) != hashes:
        raise ValueError("source changed while copying; refuse mixed snapshot")
    (scratch / "selected-source.patch").write_bytes(git(source, "diff", "--binary", "HEAD"))
    (scratch / "source-files.json").write_text(json.dumps(hashes, sort_keys=True, indent=2) + "\n")
    # git apply works outside a Git repository; the copy has no .git, links or
    # installed artifacts. Only this disposable copy receives instrumentation.
    subprocess.run(["git", "apply", "--check", str(PATCH)], cwd=copied, check=True)
    subprocess.run(["git", "apply", str(PATCH)], cwd=copied, check=True)
    receipt: dict[str, object] = {
        "source": str(source),
        "copy": str(copied),
        "revision": REVISION,
        "source_files_sha256": digest(scratch / "source-files.json"),
        "selected_source_patch_sha256": digest(scratch / "selected-source.patch"),
        "fixture_patch_sha256": digest(PATCH),
        "lockfile_sha256": digest(copied / "Cargo.lock"),
        "native_paths_equal_pinned_upstream": list(NATIVE_PATHS),
        "native_file_sha256": {name: value for name, value in hashes.items()
                               if any(name == prefix or name.startswith(prefix + "/") for prefix in NATIVE_PATHS)},
        "equivalence_scope": "native planner/index/scanner source only; exact dirty snapshot retained, not whole deployed binary equivalence",
        "repository_toolchain_sha256": digest(ROOT / "rust-toolchain.toml"),
        "premises": [
            "scalar owner:string and k:int compound index; no array fanout",
            "root OWNER and VIEWER; no conditional row permissions",
            "named WITH INDEX; equality prefix, exclusive cursor, ORDER BY k, literal LIMIT",
            "2400 eligible entries after cursor and 300 unrelated-prefix entries",
            "observer records requested raw KV cap and returned pairs before decode/fetch/filter",
            "65 entries and 1001 entries cross native batch boundary; no timing claim",
        ],
    }
    (scratch / "probe.json").write_text(json.dumps(receipt, sort_keys=True, indent=2) + "\n")
    return receipt


def run(scratch: Path, receipt: dict[str, object]) -> int:
    command = [
        "cargo", "test", "--manifest-path", str(scratch / "source/Cargo.toml"),
        "--locked", "--no-default-features", "--features", "kv-mem",
        "--package", "surrealdb-core", "--lib", "--jobs", "16",
        "--config", "profile.dev.debug=0", "--config", "profile.test.debug=0",
        TEST, "--", "--exact", "--nocapture",
    ]
    environment = dict(os.environ)
    environment.pop("BASH_ENV", None)
    if environment.get("CARGO_TARGET_DIR"):
        raise ValueError("probe requires its own default target; unset CARGO_TARGET_DIR")
    if environment.get("RUSTUP_TOOLCHAIN"):
        raise ValueError("probe requires repository-pinned toolchain without override")
    environment["CARGO_BUILD_JOBS"] = "16"
    environment["RUST_MIN_STACK"] = "8388608"
    receipt["command"] = command
    receipt["cargo_cwd"] = str(ROOT)
    receipt["test_log"] = str(scratch / "probe.log")
    print(json.dumps({"receipt": str(scratch / "probe.json"), "command": command, "test_log": receipt["test_log"]}, indent=2), flush=True)
    with (scratch / "probe.log").open("w") as log:
        result = subprocess.run(command, cwd=ROOT, env=environment, stdout=log, stderr=subprocess.STDOUT, check=False)
    receipt["exit_code"] = result.returncode
    receipt["result"] = "passed" if result.returncode == 0 else "failed"
    (scratch / "probe.json").write_text(json.dumps(receipt, sort_keys=True, indent=2) + "\n")
    print((scratch / "probe.log").read_text()[-20000:], flush=True)
    return result.returncode


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--scratch", type=Path, required=True)
    parser.add_argument("--prepare-only", action="store_true")
    args = parser.parse_args()
    receipt = prepare(args.source, args.scratch)
    if args.prepare_only:
        print(json.dumps(receipt, sort_keys=True, indent=2))
        return 0
    return run(args.scratch.resolve(), receipt)


if __name__ == "__main__":
    raise SystemExit(main())
