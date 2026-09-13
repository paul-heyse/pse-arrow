# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Build provenance of the compiled extension (blueprint §21, plan §5).

This is the one module allowed to import :mod:`pse._native` (ast-grep rule
``no-direct-native-import``): every other consumer goes through
:func:`build_info` or :func:`native_version`, so the extension's surface has a
single typed gate.

The extension embeds the two lockfiles as bytes (``pse-buildinfo``); this module
hashes them with :mod:`hashlib` so the sha256 twins can be checked against the
checkout without a second hasher in Rust. ``lockfile_hash`` (blake3, the manifest
form) stays empty until ``pse-ids`` wires it through the catalog in phase 1;
``build_info_matches_checkout`` only compares the fields that are populated.
"""

import hashlib

import msgspec

from pse import _native


class BuildInfo(msgspec.Struct, frozen=True, forbid_unknown_fields=True):
    """Provenance of the ``pse._native`` extension module in this environment.

    Attributes:
        version: The workspace version, which is also the wheel version.
        rustc_version: The compiler that produced the extension.
        profile: The Cargo profile the extension was built with.
        git_sha: The commit the extension was built from, empty when unknown.
        lockfile_hash: blake3 digest over both lockfiles, empty in phase 0.
        cargo_lock_sha256: sha256 of the ``Cargo.lock`` embedded at build time.
        uv_lock_sha256: sha256 of the ``uv.lock`` embedded at build time.
    """

    version: str
    rustc_version: str
    profile: str
    git_sha: str
    lockfile_hash: str
    cargo_lock_sha256: str
    uv_lock_sha256: str


def build_info() -> BuildInfo:
    """Return the compiled extension's build provenance as a frozen struct.

    Returns:
        The structured form of ``pse._native.build_info()``.

    Raises:
        msgspec.ValidationError: If the extension returns a shape this version
            of the Python package does not know, which means the wheel and the
            source tree disagree.
    """
    raw = dict(_native.build_info())
    cargo_lock = raw.pop("cargo_lock_bytes", b"")
    uv_lock = raw.pop("uv_lock_bytes", b"")
    raw["cargo_lock_sha256"] = _sha256(cargo_lock)
    raw["uv_lock_sha256"] = _sha256(uv_lock)
    return msgspec.convert(raw, BuildInfo)


def native_version() -> str:
    """Return ``pse._native.__version__``, the version the extension was built at.

    Returns:
        The extension's own version string.
    """
    return _native.__version__


def _sha256(data: bytes | str) -> str:
    """Hex sha256 of the embedded lockfile bytes, or the empty string when absent."""
    if isinstance(data, str):
        data = data.encode()
    return hashlib.sha256(data).hexdigest() if data else ""
