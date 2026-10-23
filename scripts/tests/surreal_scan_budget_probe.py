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


def git_environment() -> dict[str, str]:
    environment = dict(os.environ)
    for name in ("GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_COMMON_DIR"):
        environment.pop(name, None)
    return environment


def git(source: Path, *arguments: str) -> bytes:
    return subprocess.check_output(["git", "-C", str(source), *arguments], env=git_environment())


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
    # Private empty metadata prevents Git from discovering an ancestor checkout
    # when scratch is below its build/. Never copy the installed Git metadata.
    subprocess.run(["git", "init", "--quiet", "--template=", str(copied)],
                   env=git_environment(), check=True)
    fixture_patch = scratch / "test-fixture.patch"
    shutil.copy2(PATCH, fixture_patch)
    subprocess.run(["git", "apply", "--check", str(fixture_patch)], cwd=copied, env=git_environment(), check=True)
    subprocess.run(["git", "apply", str(fixture_patch)], cwd=copied, env=git_environment(), check=True)
    receipt: dict[str, object] = {
        "source": str(source),
        "copy": str(copied),
        "revision": REVISION,
        "source_files_sha256": digest(scratch / "source-files.json"),
        "selected_source_patch_sha256": digest(scratch / "selected-source.patch"),
        "fixture_patch_sha256": digest(fixture_patch),
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
            "65-entry cap and 1001-entry cap; only 1001 crosses native batch boundary; no timing claim",
        ],
    }
    (scratch / "probe.json").write_text(json.dumps(receipt, sort_keys=True, indent=2) + "\n")
    return receipt


def verify_instrumentation(copied: Path) -> None:
    text = (copied / "surrealdb/core/src/exec/index/iterator/btree.rs").read_text()
    required = (
        "#[cfg(test)]\nmod tool08_probe",
        "#[cfg(test)]\n\ttool08_observe: bool",
        "tool08_observe: ix.name.as_str() == tool08_probe::INDEX",
        "async fn tool08_compound_range_scan_stops_at_entry_budget()",
    )
    if any(fragment not in text for fragment in required):
        raise ValueError("disposable source lacks the exact test-only TOOL08 instrumentation")
    iterator = text.split("impl CompoundRangeForwardIterator {", 1)[1].split(
        "/// Direction-dispatching wrapper", 1
    )[0]
    if not (iterator.index("let res = scan(")
            < iterator.index("tool08_probe::SCANS.lock()")
            < iterator.index("decode_record_ids(res)")):
        raise ValueError("TOOL08 observer is not at the raw KV boundary before decode")


def selected_test_passed(output: str) -> bool:
    return ("running 1 test\n" in output
            and f"test {TEST} ..." in output
            and "test result: ok. 1 passed; 0 failed;" in output)


def run(scratch: Path, receipt: dict[str, object]) -> int:
    verify_instrumentation(scratch / "source")
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
    output = (scratch / "probe.log").read_text()
    exit_code = result.returncode or (0 if selected_test_passed(output) else 1)
    receipt["cargo_exit_code"] = result.returncode
    receipt["exit_code"] = exit_code
    receipt["result"] = "passed" if exit_code == 0 else "failed"
    (scratch / "probe.json").write_text(json.dumps(receipt, sort_keys=True, indent=2) + "\n")
    print(output[-20000:], flush=True)
    return exit_code


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
