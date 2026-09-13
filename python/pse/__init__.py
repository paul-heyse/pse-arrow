# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Arrow-native process systems engineering core (import name ``pse``).

The distribution is ``pse-arrow``; the import name is ``pse``. Everything that
computes lives in the Rust extension ``pse._native``; this package is the typed,
contract-enforcing boundary around it (blueprint §21).

Importing this package has three deliberate side effects and no others:

* the ten-plus ``pse.*`` ``pyarrow`` extension types of blueprint §4.4 are
  registered, idempotently, because an unregistered type degrades silently to
  its storage type on the Python side;
* every generated contract class is linted for ``typing.Any``, bare ``dict``
  and bare ``list`` fields, which are the ways a contract stops constraining
  anything;
* nothing else. In particular numpy, scipy, pyomo, pint, pandas and idaes are
  *not* imported -- a test asserts that in a subprocess. The numpy boundary is
  :mod:`pse._array`, which imports numpy lazily.

There is no ``from __future__ import annotations`` anywhere under ``python/pse``:
PEP 563 would make annotations strings and the import-time ``Any`` lint blind.
An ast-grep rule enforces that.
"""

import importlib.metadata
import platform
import re
import shutil
import subprocess

import msgspec

from pse import governance
from pse._build import BuildInfo, build_info, native_version
from pse.contracts import extension_types

__all__ = [
    "BuildInfo",
    "HostCapabilities",
    "__version__",
    "build_info",
    "probe_host",
]

#: `ipopt -v` prints e.g. "Ipopt 3.14.20 (x86_64-pc-linux-gnu), ASL(20241108)".
_IPOPT_VERSION = re.compile(r"\bIpopt\s+(\d+(?:\.\d+)*)")

#: How long to wait for `ipopt -v`; a solver that does not answer is not usable.
_PROBE_TIMEOUT_SECONDS = 30.0


class HostCapabilities(msgspec.Struct, frozen=True, forbid_unknown_fields=True):
    """What this machine can actually do, as observed rather than assumed.

    The parity pre-flight fails on this struct before a model is built, so a
    parity run reports "no solver" instead of an error thrown three layers into
    a flowsheet (blueprint §21.4). It is stdlib-only by construction: probing
    must work in an environment where the optional dependencies are absent.

    Attributes:
        python_version: The running interpreter's version, e.g. ``3.14.7``.
        platform: The platform string, e.g. ``Linux-7.0.0-x86_64-with-glibc``.
        ipopt_on_path: Whether an ``ipopt`` executable resolves on ``PATH``.
        ipopt_version: The version ``ipopt -v`` reports, or None when the
            executable is absent or did not answer.
        idaes_version: The installed ``idaes-pse`` distribution version, or
            None. Read from metadata; ``idaes`` is never imported.
        pyomo_version: The installed ``Pyomo`` distribution version, or None.
    """

    python_version: str
    platform: str
    ipopt_on_path: bool
    ipopt_version: str | None
    idaes_version: str | None
    pyomo_version: str | None


def _distribution_version(name: str) -> str | None:
    """Read an installed distribution's version without importing it.

    Args:
        name: The distribution name, e.g. ``idaes-pse``.

    Returns:
        The version string, or None when the distribution is not installed.
    """
    try:
        return importlib.metadata.version(name)
    except importlib.metadata.PackageNotFoundError:
        return None


def _ipopt_version(executable: str) -> str | None:
    """Ask an Ipopt executable for its version.

    Args:
        executable: The absolute path ``shutil.which`` resolved.

    Returns:
        The dotted version, or None when the executable did not answer with a
        recognisable banner. ``ipopt -v`` writes to stdout on some builds and
        stderr on others, so both are searched.
    """
    try:
        completed = subprocess.run(  # noqa: S603
            [executable, "-v"],
            capture_output=True,
            text=True,
            timeout=_PROBE_TIMEOUT_SECONDS,
            check=False,
        )
    except (OSError, subprocess.SubprocessError):
        return None
    match = _IPOPT_VERSION.search(f"{completed.stdout}\n{completed.stderr}")
    return match.group(1) if match else None


def probe_host() -> HostCapabilities:
    """Observe what this machine provides, without importing optional packages.

    Returns:
        A frozen :class:`HostCapabilities` describing the interpreter, the
        platform, and whether Ipopt, IDAES and Pyomo are available.
    """
    executable = shutil.which("ipopt")
    return HostCapabilities(
        python_version=platform.python_version(),
        platform=platform.platform(),
        ipopt_on_path=executable is not None,
        ipopt_version=None if executable is None else _ipopt_version(executable),
        idaes_version=_distribution_version("idaes-pse"),
        pyomo_version=_distribution_version("Pyomo"),
    )


def _resolve_version() -> str:
    """Resolve the package version, preferring installed metadata.

    Returns:
        The installed ``pse-arrow`` distribution version, or the version the
        extension was built at when the distribution metadata is absent (a
        source tree on ``sys.path`` beside a built extension).
    """
    try:
        return importlib.metadata.version("pse-arrow")
    except importlib.metadata.PackageNotFoundError:
        return native_version()


#: The one version number: the wheel's, the extension's and the workspace's.
__version__: str = _resolve_version()

# Import-time invariants (blueprint §4.4, §21.5). Both are idempotent and cheap;
# both fail loudly rather than degrade, because both defend a silent failure.
extension_types.register_all()
governance.check()
