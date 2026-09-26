#!/usr/bin/env python3
"""Verify the handoff package without running or changing its archived tests."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import sys


def main() -> int:
    root = Path(__file__).resolve().parent
    errors: list[str] = []
    try:
        manifest = json.loads((root / "MANIFEST.json").read_text(encoding="utf-8"))
        for item in manifest["files"]:
            relative = Path(item["path"])
            if relative.is_absolute() or ".." in relative.parts:
                errors.append(f"Unsafe manifest path: {relative}")
                continue
            p = root / relative
            if not p.is_file():
                errors.append(f"Missing file: {relative}")
                continue
            payload = p.read_bytes()
            if len(payload) != item["bytes"] or hashlib.sha256(payload).hexdigest() != item["sha256"]:
                errors.append(f"Content mismatch: {relative}")
        for line in (root / "CHECKSUMS.sha256").read_text(encoding="utf-8").splitlines():
            digest, name = line.split("  ", 1)
            relative = Path(name)
            if relative.is_absolute() or ".." in relative.parts:
                errors.append(f"Unsafe checksum path: {relative}")
                continue
            p = root / relative
            if not p.is_file() or hashlib.sha256(p.read_bytes()).hexdigest() != digest:
                errors.append(f"Checksum mismatch: {relative}")
    except (OSError, ValueError, KeyError, TypeError) as exc:
        errors.append(f"Cannot verify package: {exc}")
    if errors:
        print("VERIFICATION FAILED")
        print("\n".join(errors))
        return 1
    print(f"PASS: {len(manifest['files'])} payload files and checksum manifest verified.")
    print("This is a byte-integrity check, not a thermodynamics or provider test.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
