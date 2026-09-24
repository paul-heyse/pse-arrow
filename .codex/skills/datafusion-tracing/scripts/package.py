#!/usr/bin/env python3
"""Package a reproducible reader or research bundle with a file integrity manifest."""

from __future__ import annotations

import argparse
import gzip
import hashlib
import io
import json
import sys
import tarfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def selected(path: Path, mode: str) -> bool:
    parts = path.relative_to(ROOT).parts
    if any(
        p
        in {
            "__pycache__",
            ".cache",
            ".git",
            ".ruff_cache",
            "target",
            "bundles",
            "transfer",
        }
        for p in parts
    ):
        return False
    if path.suffix in {".gz", ".tar"}:
        return False
    if parts[0] in {"content", "scripts", "authoring", "queries"}:
        return True
    # Private capture contract links stay inspectable offline, even in the reader.
    if parts[:2] in {
        ("build", "acquired"),
        ("build", "fixtures"),
        ("build", "manifests"),
    }:
        return True
    if parts[0] == "build":
        return mode == "research"
    if len(parts) == 1:
        return path.suffix == ".md"
    if parts[:3] == ("skill_improvement", "evidence", "implementation"):
        return mode == "research" or parts[3] in {
            "probe-results.json",
            "probe-profile.json",
            "captures",
            "upstream-verification.json",
            "upstream-sources",
            "stream-adapter-verification.json",
            "datafusion-boundary-contract.json",
        }
    return mode == "research" and parts[0] == "skill_improvement"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=["reader", "research"])
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    files = [
        p
        for p in sorted(ROOT.rglob("*"))
        if p.is_file()
        and not p.is_symlink()
        and p.resolve() != output
        and selected(p, args.mode)
    ]
    manifest = {
        "schema_version": 1,
        "mode": args.mode,
        "files": {
            str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in files
        },
    }
    output.parent.mkdir(parents=True, exist_ok=True)
    with (
        output.open("wb") as raw,
        gzip.GzipFile(fileobj=raw, mode="wb", filename="", mtime=0) as compressed,
        tarfile.open(fileobj=compressed, mode="w") as archive,
    ):
        for name, data in [
            (str(p.relative_to(ROOT)), p.read_bytes()) for p in files
        ] + [
            (
                "BUNDLE_MANIFEST.json",
                (json.dumps(manifest, indent=2, sort_keys=True) + "\n").encode(),
            )
        ]:
            info = tarfile.TarInfo(name)
            info.size = len(data)
            info.mode = 0o644
            archive.addfile(info, io.BytesIO(data))
    sys.stdout.write(
        json.dumps(
            {
                "mode": args.mode,
                "files": len(files),
                "bytes": output.stat().st_size,
                "sha256": hashlib.sha256(output.read_bytes()).hexdigest(),
            }
        )
        + "\n"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
