#!/usr/bin/env python3
"""Exercise a packaged reader from / with an empty environment and trace filesystem/network use."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
import tarfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "skill_improvement/evidence/implementation"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", type=Path, required=True)
    parser.add_argument("--destination", type=Path, required=True)
    args = parser.parse_args()
    archive = args.archive.resolve()
    destination = args.destination.resolve()
    if destination.is_relative_to(ROOT):
        parser.error("Transfer destination must be outside the skill")
    destination.mkdir(parents=True)
    with tarfile.open(archive) as bundle:
        bundle.extractall(destination, filter="data")
    assert not (destination / "build/.cache").exists()
    mode = json.loads((destination / "BUNDLE_MANIFEST.json").read_text())["mode"]
    trace = EVIDENCE / f"transfer-{mode}-syscalls.log"
    interpreter = str(Path(sys.executable).resolve())
    command = [
        "strace",
        "-f",
        "-qq",
        "-s",
        "4096",
        "-e",
        "trace=file,network",
        "-o",
        str(trace),
        interpreter,
        "-I",
        "-S",
        str(destination / "scripts/verify_bundle.py"),
    ]
    done = subprocess.run(
        command,
        cwd="/",
        env={"PATH": "/usr/bin:/bin", "LANG": "C.UTF-8"},
        capture_output=True,
        text=True,
        check=True,
    )
    log = trace.read_text()
    socket_calls = [
        line
        for line in log.splitlines()
        if re.search(r"\b(socket|connect|sendto|recvfrom)\(", line)
    ]
    connections = [line for line in socket_calls if "AF_INET" in line]
    local_name_cache = [line for line in socket_calls if "/var/run/nscd/socket" in line]
    other_connects = [
        line
        for line in socket_calls
        if "connect(" in line and line not in local_name_cache
    ]
    unexpected = [
        line
        for line in log.splitlines()
        if re.search(r"\b(open|openat|execve)\(", line)
        and str(ROOT.parents[2]) in line
        and str(destination) not in line
    ]
    assert not connections, connections
    assert not other_connects, other_connects
    assert not unexpected, unexpected
    assert "build/.cache" not in log
    result = json.loads(done.stdout)

    def invalidation() -> dict:
        run = subprocess.run(
            [interpreter, "-I", "-S", str(destination / "scripts/invalidation.py")],
            cwd="/",
            env={"PATH": "/usr/bin:/bin"},
            capture_output=True,
            text=True,
            check=True,
        )
        return json.loads(run.stdout)

    assert not invalidation()["affected"]
    authored = destination / "authoring/capabilities/tracing.preview.json"
    original = authored.read_bytes()
    try:
        authored.write_bytes(original + b"\n")
        assert set(invalidation()["affected"]) == {"tracing.preview"}
    finally:
        authored.write_bytes(original)
    index = destination / "content/index/operations.tsv"
    original_index = index.read_bytes()
    try:
        index.write_bytes(
            original_index
            + b"datafusion_tracing::qualification_control\tfixture\tfunction\tdatafusion-tracing\t"
            b"fixture.json\tfixture.md\tfn fixture()\n"
        )
        changed = invalidation()
        assert changed["changed_crates"] == ["datafusion-tracing"]
        assert changed["affected"]["tracing.preview"][
            "candidate_set_review_crates"
        ] == ["datafusion-tracing"]
    finally:
        index.write_bytes(original_index)
    assert not invalidation()["affected"]
    result.update(
        archive_sha256=hashlib.sha256(archive.read_bytes()).hexdigest(),
        network_syscalls=len(connections),
        system_name_cache_attempts=len(local_name_cache),
        outside_skill_accesses=len(unexpected),
        source_cache_present=False,
        reader_file_access_restricted=True,
        environment="PATH and LANG only; resolved base Python -I -S",
        invalidation_controls="passed: changed authoring and newly added crate operation",
        scope=(
            "No original host-project files opened; "
            "Python/system runtime reads and ancestor path metadata are expected."
        ),
    )
    (EVIDENCE / f"transfer-{mode}-qualification.json").write_text(
        json.dumps(result, indent=2) + "\n"
    )
    sys.stdout.write(json.dumps(result, indent=2) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
