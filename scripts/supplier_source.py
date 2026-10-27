#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Refresh complete supplier deltas and their checked build-input receipts.

The published archive table in vendor/deltas/readme.md remains the baseline owner.
Only tracked supplier paths are copied; build artifacts cannot enter a delta.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import stat
import subprocess
import tarfile
import tempfile
import urllib.request


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def refresh(repo: Path, directory: str) -> None:
    rows = []
    for line in (repo / "vendor/deltas/readme.md").read_text().splitlines():
        cells = [cell.strip().strip("`") for cell in line.strip("|").split("|")]
        if len(cells) == 7 and cells[0] == directory and len(cells[3]) == 64:
            rows.append(cells)
    if len(rows) != 1:
        raise ValueError(f"expected one published archive row for {directory}")
    _, package, version, expected, revision, vcs_path, _ = rows[0]
    with tempfile.TemporaryDirectory(prefix="pse-supplier-source-") as temporary:
        scratch = Path(temporary)
        archive = scratch / f"{package}-{version}.crate"
        with urllib.request.urlopen(
            f"https://static.crates.io/crates/{package}/{archive.name}", timeout=60
        ) as response:
            data = response.read()
        if digest(data) != expected:
            raise ValueError(f"published archive checksum mismatch: {package}")
        archive.write_bytes(data)
        with tarfile.open(archive) as bundle:
            bundle.extractall(scratch, filter="data")
        tree = scratch / f"{package}-{version}"
        vcs = json.loads((tree / ".cargo_vcs_info.json").read_text())
        if vcs["git"]["sha1"] != revision or (vcs["path_in_vcs"] or ".") != vcs_path:
            raise ValueError(f"published VCS metadata mismatch: {package}")
        subprocess.run(["git", "init", "-q", str(tree)], check=True)
        subprocess.run(["git", "-C", str(tree), "add", "-f", "."], check=True)
        subprocess.run(
            ["git", "-C", str(tree), "-c", "user.name=source receipt", "-c",
             "user.email=source-receipt@invalid", "commit", "-qm", "published archive"],
            check=True,
        )
        prefix = f"vendor/{directory}/"
        entries = subprocess.check_output(
            ["git", "ls-files", "--stage", "-z", "--", prefix], cwd=repo
        ).decode().split("\0")
        files = {}
        for entry in filter(None, entries):
            metadata, path = entry.split("\t", 1)
            relative = path[len(prefix):]
            if relative == "PATCH.md":
                continue
            source = repo / path
            mode = metadata.split()[0]
            files[relative] = {"sha256": digest(source.read_bytes()), "mode": mode}
        for path in tree.rglob("*"):
            if path.is_file() and ".git" not in path.relative_to(tree).parts:
                if str(path.relative_to(tree)) not in files:
                    path.unlink()
        for relative, facts in files.items():
            source = repo / prefix / relative
            target = tree / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, target)
            target.chmod(0o755 if facts["mode"] == "100755" else 0o644)
        subprocess.run(["git", "-C", str(tree), "add", "-Af", "."], check=True)
        delta = subprocess.check_output(
            ["git", "-C", str(tree), "diff", "--cached", "--binary", "--full-index",
             "--no-renames", "HEAD"]
        )
        (repo / f"vendor/deltas/{directory}.patch").write_bytes(delta)
        # Verify reconstruction independently of the copied tree before publishing.
        reconstructed = scratch / "reconstructed"
        reconstructed.mkdir()
        with tarfile.open(archive) as bundle:
            bundle.extractall(reconstructed, filter="data")
        reconstructed = reconstructed / f"{package}-{version}"
        subprocess.run(
            ["git", "-C", str(reconstructed), "apply", "--whitespace=nowarn",
             str(repo / f"vendor/deltas/{directory}.patch")], check=True,
        )
        actual = {
            str(path.relative_to(reconstructed)): {
                "sha256": digest(path.read_bytes()),
                "mode": "100755" if path.stat().st_mode & stat.S_IXUSR else "100644",
            }
            for path in reconstructed.rglob("*") if path.is_file()
        }
        if actual != files:
            raise ValueError(f"complete source reconstruction mismatch: {directory}")
        receipt = {"archive_sha256": expected, "delta_sha256": digest(delta),
                   "package": package, "version": version, "files": files}
        (repo / f"vendor/deltas/{directory}.source.json").write_text(
            json.dumps(receipt, sort_keys=True, indent=2) + "\n"
        )
        print(f"{directory}: authenticated archive + complete delta; {len(files)} paths")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directories", nargs="+")
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[1]
    for directory in args.directories:
        refresh(repo, directory)


if __name__ == "__main__":
    main()
