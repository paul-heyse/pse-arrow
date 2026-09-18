#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Reproduce the narrow Delta source override from immutable git blobs and a patch.

Default is verification; --apply rewrites only the declared generated vendor tree.
No Cargo cache or upstream checkout is edited. The library-only manifest excludes
upstream test harnesses; PSE's isolated interface probes use the selected library.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import tempfile
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEST = ROOT / "vendor/delta-rs"
PATCH = ROOT / "tooling/delta-native-seams.patch"


def git(source: Path, *args: str) -> bytes:
    return subprocess.run(
        ["git", "-C", str(source), *args], check=True, capture_output=True
    ).stdout


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "source", type=Path, help="read-only checkout containing the pinned commit"
    )
    parser.add_argument("--apply", action="store_true")
    args = parser.parse_args()
    manifest = tomllib.loads((ROOT / "Cargo.toml").read_text())
    revision = manifest["workspace"]["metadata"]["pse"]["delta-source-revision"]
    paths = (
        git(args.source, "ls-tree", "-r", "--name-only", revision).decode().splitlines()
    )
    selected = [
        p
        for p in paths
        if p
        in {
            "Cargo.toml",
            "LICENSE.txt",
            "README.md",
            "crates/core/Cargo.toml",
            "crates/derive/Cargo.toml",
        }
        or p.startswith(("crates/core/src/", "crates/derive/src/"))
    ]
    with tempfile.TemporaryDirectory(prefix="pse-delta-source-") as directory:
        scratch = Path(directory)
        originals = {}
        for name in selected:
            content = git(args.source, "show", f"{revision}:{name}")
            originals[name] = hashlib.sha256(content).hexdigest()
            path = scratch / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(content)
        # Keep upstream workspace dependency declarations and their locked kernel
        # source intact. Root Cargo.lock pins the branch's actual immutable commit.
        workspace = (scratch / "Cargo.toml").read_text()
        workspace = workspace.replace(
            'members = ["crates/*", "python"]',
            'members = ["crates/core", "crates/derive"]',
        )
        workspace = workspace[
            : workspace.index(
                "# ============================================================================"
            )
        ]
        (scratch / "Cargo.toml").write_text(workspace)
        core = scratch / "crates/core/Cargo.toml"
        text = core.read_text()
        start, end = text.index("[dev-dependencies]"), text.index("[features]")
        text = text[:start] + text[end:]
        start, end = (
            text.index("[[test]]"),
            text.index("[package.metadata.cargo-machete]"),
        )
        text = text[:start] + text[end:]
        text = text.replace(
            "[package]\n", "[package]\nautotests = false\nautobenches = false\n", 1
        )
        core.write_text(text)
        subprocess.run(
            ["git", "apply", "--unsafe-paths", str(PATCH)], cwd=scratch, check=True
        )
        result = {
            str(path.relative_to(scratch)): path.read_bytes()
            for path in scratch.rglob("*")
            if path.is_file()
        }
        provenance = {
            "upstream": "https://github.com/delta-io/delta-rs",
            "revision": revision,
            "patch_sha256": hashlib.sha256(PATCH.read_bytes()).hexdigest(),
            "upstream_sha256": originals,
            "vendored_sha256": {
                name: hashlib.sha256(data).hexdigest()
                for name, data in sorted(result.items())
            },
        }
        result["PROVENANCE.json"] = (
            json.dumps(provenance, indent=2, sort_keys=True) + "\n"
        ).encode()
        actual = {
            str(path.relative_to(DEST)): path
            for path in DEST.rglob("*")
            if path.is_file()
        }
        drift = sorted(
            name
            for name in result.keys() | actual.keys()
            if name not in result
            or name not in actual
            or actual[name].read_bytes() != result[name]
        )
        if args.apply:
            for name in drift:
                path = DEST / name
                if name in result:
                    path.parent.mkdir(parents=True, exist_ok=True)
                    path.write_bytes(result[name])
                else:
                    path.unlink()
            print(f"Delta source override regenerated: {len(drift)} files changed")
        elif drift:
            raise SystemExit("Delta source override differs: " + ", ".join(drift))
        else:
            print(f"Delta source override verified: {len(result)} files at {revision}")


if __name__ == "__main__":
    main()
