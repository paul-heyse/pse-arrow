# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Fixtures for the Python boundary tests (plan §5).

These fixtures encode invariants that no single test owns: current inspection
publications are produced by Rust, the extension types are registered,
``import pse`` stays free of the scientific stack, and the built extension
belongs to this checkout.
"""

import hashlib
import os
import subprocess
import sys
from pathlib import Path

import pyarrow as pa
import pyarrow.ipc
import pytest

from pse._build import CacheSettings, EngineSettings, build_info
from pse.contracts.extension_types import EXTENSION_NAMES

#: Modules `pse` itself must not pull in at import (blueprint §21.6, §3.1).
#:
#: pyarrow 25 imports numpy itself, so the check is differential: whatever
#: `import pyarrow` already costs is the baseline, and `import pse` must add
#: nothing to it. `pse._array` is watched alongside because it is the module
#: whose import *would* be ours, and the crisp statement of the invariant.
#:
#: Each name here answers to a live import-linter contract in `pyproject.toml`:
#: numpy/scipy to the array boundary, pyomo/pint to the adapter, idaes to the
#: parity environment. `pandas` was dropped with the dependency-hygiene contract
#: that held it (ADR-0066) -- this tuple is an import-COST check against those
#: boundaries, not a list of libraries the project may not use.
FORBIDDEN_ON_IMPORT = ("numpy", "scipy", "pyomo", "pint", "idaes")
WATCHED_ON_IMPORT = (*FORBIDDEN_ON_IMPORT, "pse._array")

#: Repository root: python/pse/tests/conftest.py -> pse-arrow/
REPO_ROOT = Path(__file__).resolve().parents[3]


@pytest.fixture(scope="session")
def native_inspection_publication() -> Path:
    """Read the fresh publication produced before workers start by ``just py-test``."""
    configured = os.environ.get("PSE_INSPECTION_PUBLICATION")
    if configured is None:
        pytest.fail(
            "Run just py-test to build a fresh native inspection publication.",
            pytrace=False,
        )
    store = Path(configured)
    if not (store / "publication-index.json").is_file():
        pytest.fail(
            f"Native inspection publication is incomplete: {store}", pytrace=False
        )
    return store


@pytest.fixture(scope="session", autouse=True)
def registered_extension_types() -> tuple[str, ...]:
    """Assert every `pse.*` extension type resolves after an IPC round trip.

    pyarrow exposes no registry lookup, and an unregistered extension degrades
    silently to its storage type, so the only honest check is to round-trip a
    schema carrying every type through IPC and read the names back.

    Returns:
        The extension names that survived the round trip, in table order.
    """
    schema = pa.schema(
        [
            pa.field(f"c{index}", _extension_type(name))
            for index, name in enumerate(EXTENSION_NAMES)
        ]
    )
    restored = _ipc_round_trip(schema)
    names = tuple(field.type.extension_name for field in restored)
    assert names == EXTENSION_NAMES, (
        "extension types did not survive an IPC round trip; "
        f"expected {EXTENSION_NAMES}, got {names}"
    )
    return names


@pytest.fixture(scope="session")
def no_numpy_on_import() -> frozenset[str]:
    """Assert that `import pse` adds nothing from the scientific stack.

    Each probe runs in a subprocess: this process has already imported numpy,
    which the tests use as an oracle. The measurement is differential because
    pyarrow -- a core dependency -- imports numpy itself; what is ours to keep
    true is that `pse` adds nothing on top and never reaches `pse._array`.

    Returns:
        The forbidden modules `pse` added over the pyarrow baseline, which is
        empty when the assertion holds.
    """
    baseline = _modules_after("import pyarrow")
    after_pse = _modules_after("import pse")
    assert "pse._array" not in after_pse, (
        "`import pse` imported pse._array; the numpy boundary is imported "
        "lazily, inside to_ndarray (blueprint §21.6)"
    )
    leaked = after_pse - baseline - {"pse._array"}
    assert not leaked, (
        f"`import pse` imported {sorted(leaked)} beyond what `import pyarrow` "
        "already costs; numpy and scipy live at the array boundary, pyomo and "
        "pint in the adapter, and idaes in the parity environment "
        "(blueprint §3.1, §21.6)"
    )
    return leaked


def _modules_after(statement: str) -> frozenset[str]:
    """Report which watched modules are loaded after running one import.

    Args:
        statement: The import statement to execute in a fresh interpreter.

    Returns:
        The subset of :data:`WATCHED_ON_IMPORT` present in ``sys.modules``.
    """
    program = (
        f"import sys\n{statement}\n"
        f"print(' '.join(sorted(m for m in {WATCHED_ON_IMPORT!r} if m in sys.modules)))"
    )
    completed = subprocess.run(
        [sys.executable, "-c", program],
        capture_output=True,
        text=True,
        check=True,
    )
    return frozenset(completed.stdout.split())


@pytest.fixture(scope="session")
def build_info_matches_checkout() -> None:
    """Assert the built extension was produced from this checkout's lockfiles.

    Only the fields the extension actually populates are compared: the lockfile
    digests are empty strings until ``pse-buildinfo`` wires them in phase 1, and
    a placeholder must not be able to fail this by looking like a mismatch.
    """
    info = build_info()
    for field_name, lockfile in (
        ("cargo_lock_sha256", REPO_ROOT / "Cargo.lock"),
        ("uv_lock_sha256", REPO_ROOT / "uv.lock"),
    ):
        recorded = getattr(info, field_name)
        if not recorded or not lockfile.is_file():
            continue
        actual = hashlib.sha256(lockfile.read_bytes()).hexdigest()
        assert recorded == actual, (
            f"the built extension records {field_name}={recorded} but "
            f"{lockfile.name} hashes to {actual}: rebuild with "
            "`uv run maturin develop --uv --release`"
        )


def _extension_type(name: str) -> pa.ExtensionType:
    """Instantiate a registered extension type by name.

    Args:
        name: A `pse.*` extension name.

    Returns:
        An instance of the registered class.
    """
    from pse.contracts import extension_types  # noqa: PLC0415

    for cls in extension_types._EXTENSION_TYPES:
        if cls._extension_name == name:
            return cls(cls._prototype_binding)
    pytest.fail(f"no extension class declares {name!r}", pytrace=False)


def _ipc_round_trip(schema: pa.Schema) -> pa.Schema:
    """Serialise a schema through the IPC stream format and read it back.

    No batches: the stream writer emits the schema message when it opens, and
    the schema is what carries the `ARROW:extension:*` keys.

    Args:
        schema: The schema to round-trip.

    Returns:
        The schema as the reader reconstructed it.
    """
    sink = pa.BufferOutputStream()
    with pa.ipc.new_stream(sink, schema):
        pass
    with pa.ipc.open_stream(sink.getvalue()) as reader:
        return reader.schema


@pytest.fixture(scope="session")
def inspection_settings(tmp_path_factory: pytest.TempPathFactory) -> EngineSettings:
    """Shared explicit budget for native publication readers in this process."""
    return EngineSettings(
        memory_limit_bytes=32 << 30,
        threads=1,
        spill_dir=str(tmp_path_factory.mktemp("inspection-spill")),
        max_spill_bytes=1 << 30,
        batch_size=7,
        cache=CacheSettings(
            working_bytes=16 << 30,
            metadata_bytes=8 << 20,
            snapshot_bytes=64 << 20,
            resident_bytes=256 << 20,
            inflight_bytes=64 << 20,
            inspection_bytes=4 << 20,
        ),
    )
