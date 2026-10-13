# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Deferred recording must qualify the actual mapped extension at access time."""

import os
import shutil
import subprocess
import sys
from pathlib import Path

import pytest

import pse


@pytest.mark.unit
def test_configured_local_runtime_refuses_replaced_mapped_extension(
    tmp_path: Path,
) -> None:
    """Replace only a child's copied pathname, without truncating its mapped inode."""
    package = Path(pse.__file__).resolve().parent
    imported = sys.modules["pse._native"].__file__
    assert imported is not None
    extension = Path(imported).resolve()
    isolated = tmp_path / "isolated"
    copied_package = isolated / "pse"
    shutil.copytree(
        package,
        copied_package,
        ignore=shutil.ignore_patterns("__pycache__", "tests", "_native*"),
    )
    # A real independent copy is essential: a hard link or symlink would retain
    # the installed/shared artifact's identity and give this test unsafe ownership.
    copied_extension = copied_package / extension.name
    shutil.copyfile(extension, copied_extension, follow_symlinks=True)
    copied_stat = copied_extension.stat()
    source_stat = extension.stat()
    assert (copied_stat.st_dev, copied_stat.st_ino) != (
        source_stat.st_dev,
        source_stat.st_ino,
    )
    environment = os.environ.copy()
    environment["PYTHONPATH"] = str(isolated)
    environment.pop("PSE_SURREAL_STATE", None)
    child = r"""
import os
import sys
from pathlib import Path

import pse

root = Path.cwd()
module = sys.modules["pse._native"]
extension = Path(module.__file__).resolve()
assert extension.parent == root / "pse", extension
spill = root / "spill"
spill.mkdir()
settings = pse.EngineSettings(
    memory_limit_bytes=64 << 30,
    threads=2,
    target_partitions=1,
    spill_dir=str(spill),
    max_spill_bytes=1 << 30,
    batch_size=7,
    cache=pse.CacheSettings(
        working_bytes=16 << 30,
        metadata_bytes=8 << 20,
        concurrent_loads=2,
        inflight_bytes=64 << 20,
        inspection_bytes=4 << 20,
    ),
)
state = root / "never-opened-state"
runtime = pse.Runtime(settings, ephemeral=True, substrate=str(state))
assert not runtime.durable
assert not state.exists()
# Atomic pathname replacement preserves all pages of the old mapped inode.
# Never open, truncate or rewrite the mapped file itself.
replacement = root / "replacement-image"
replacement.write_bytes(b"replacement is not the loaded native artifact\n")
os.replace(replacement, extension)
try:
    runtime.run_record("never-issued")
except pse.InspectionError as error:
    assert "actual loaded module association" in error.report.message, error.report.message
else:
    raise AssertionError("deferred access accepted a replacement for the mapped artifact")
assert not state.exists()
runtime.close()
print("refused replaced mapped artifact before store access")
"""
    completed = subprocess.run(
        [sys.executable, "-c", child],
        cwd=isolated,
        env=environment,
        capture_output=True,
        text=True,
        timeout=45,
        check=False,
    )
    assert completed.returncode == 0, completed.stdout + completed.stderr
    assert "refused replaced mapped artifact before store access" in completed.stdout
