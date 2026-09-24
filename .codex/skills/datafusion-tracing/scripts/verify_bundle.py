#!/usr/bin/env python3
"""Verify a copied bundle offline; no cache, network, Cargo or host project imports."""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# Each reader subprocess can access only its bundle and Python's runtime. The check
# also rejects network and child-process attempts rather than merely omitting credentials.
ISOLATED_READER = r"""
import os, pathlib, runpy, sys
root = pathlib.Path(sys.argv[1]).resolve()
allowed = (root, pathlib.Path(sys.base_prefix).resolve())
def audit(event, args):
    if event == "open" or event in ("os.listdir", "os.scandir"):
        name = args[0]
        if isinstance(name, (str, bytes, os.PathLike)):
            path = pathlib.Path(os.fsdecode(name)).resolve()
            if not any(path.is_relative_to(base) for base in allowed):
                raise PermissionError(f"Reader access outside bundle/runtime: {path}")
    if event.startswith("socket.") or event in ("subprocess.Popen", "os.system"):
        raise PermissionError(f"Reader external execution/network denied: {event}")
sys.addaudithook(audit)
sys.argv = [str(root / "scripts/reference.py"), *sys.argv[2:]]
runpy.run_path(sys.argv[0], run_name="__main__")
"""


def main() -> int:
    manifest = json.loads((ROOT / "BUNDLE_MANIFEST.json").read_text())
    for name, expected in manifest["files"].items():
        path = ROOT / name
        if (
            not path.is_file()
            or hashlib.sha256(path.read_bytes()).hexdigest() != expected
        ):
            raise ValueError(f"Missing or changed bundle file: {name}")
    for args in [
        ["find", "--task", "stdout works collector empty", "--limit", "1"],
        ["show", "tracing.preview"],
        [
            "show",
            "datafusion_tracing::InstrumentationOptions",
            "--member",
            "preview_fn",
        ],
        ["compare", "tracing.rules.phase", "tracing.rules.full"],
    ]:
        done = subprocess.run(
            [sys.executable, "-I", "-S", "-c", ISOLATED_READER, str(ROOT), *args],
            cwd="/",
            capture_output=True,
            text=True,
            check=True,
        )
        result = json.loads(done.stdout)
        if "error" in result:
            raise ValueError(result)
    # Validate content separately from the transport manifest.
    provenance = json.loads((ROOT / "content/PROVENANCE.json").read_text())
    for name, expected in provenance["files"].items():
        assert (
            hashlib.sha256((ROOT / "content" / name).read_bytes()).hexdigest()
            == expected
        )
    sys.stdout.write(
        json.dumps(
            {
                "state": "passed",
                "mode": manifest["mode"],
                "manifest_files": len(manifest["files"]),
                "lookup_commands": 4,
                "cwd": "/",
                "network_required": False,
                "reader_access": "audit hook restricts file reads to bundle and Python runtime",
            }
        )
        + "\n"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
