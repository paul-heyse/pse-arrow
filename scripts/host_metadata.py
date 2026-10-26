# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Private host metadata mechanics without admission or workload policy."""

from __future__ import annotations

import contextlib
import fcntl
import os
import stat
import subprocess
import tempfile
import time
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence


class AdmissionError(ValueError):
    """The requested supervised placement could not be established."""



def root_path(env: Mapping[str, str] | None = None) -> Path:
    source = os.environ if env is None else env
    home = Path(source.get("XDG_STATE_HOME", str(Path.home() / ".local/state")))
    return home / "pse-arrow/host-admission"



def protected(directory: Path) -> None:
    if any(path.is_symlink() for path in (directory, *directory.parents)):
        raise AdmissionError("Admission state must not traverse symlinks")
    directory.mkdir(mode=0o700, parents=True, exist_ok=True)
    information = directory.stat()
    if information.st_uid != os.getuid() or stat.S_IMODE(information.st_mode) & 0o077:
        raise AdmissionError("Admission state must be owned with mode0700")



def remaining(deadline: float | None, maximum: float) -> float:
    """A shared absolute clock; ordinary calls retain their existing timeouts."""
    if deadline is None:
        return maximum
    budget = min(maximum, deadline - time.monotonic())
    if budget <= 0:
        raise AdmissionError("Control operation deadline exhausted")
    return budget



def control_run(
    command: Sequence[str],
    *,
    env: Mapping[str, str],
    deadline: float,
    maximum: float = 10,
) -> subprocess.CompletedProcess[str]:
    """Bound every process wait, including cancellation, without descendant pipe waits."""
    remaining(deadline, maximum)
    with tempfile.TemporaryFile() as output, tempfile.TemporaryFile() as errors:
        child = subprocess.Popen(
            command,
            env=dict(env),
            stdout=output,
            stderr=errors,
        )
        try:
            budget = remaining(deadline, maximum)
            # Reserve part of this same clock for killing/reaping the direct
            # client. The actual scope remains charged independently of it.
            status = child.wait(timeout=budget - min(0.05, budget / 4))
        except BaseException:
            with contextlib.suppress(ProcessLookupError):
                child.kill()
            # The scope stays charged if its client or actual descendants outlive
            # this clock. Never wait indefinitely for their output or their exit.
            with contextlib.suppress(subprocess.TimeoutExpired):
                child.wait(timeout=max(0, deadline - time.monotonic()))
            raise
        output.seek(0)
        errors.seek(0)
        return subprocess.CompletedProcess(
            command,
            status,
            output.read().decode(errors="replace"),
            errors.read().decode(errors="replace"),
        )



def lock_until(fd: int, kind: int, deadline: float | None) -> None:
    if deadline is None:
        fcntl.flock(fd, kind)
        return
    while True:
        remaining(deadline, 1)
        try:
            fcntl.flock(fd, kind | fcntl.LOCK_NB)
        except BlockingIOError:
            time.sleep(remaining(deadline, 0.01))
        else:
            return



def validate_private_file(information: os.stat_result, label: str) -> None:
    if (
        not stat.S_ISREG(information.st_mode)
        or information.st_uid != os.getuid()
        or information.st_mode & 0o077
    ):
        raise AdmissionError(f"Unsafe admission {label}")

