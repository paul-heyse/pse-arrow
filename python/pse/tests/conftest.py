# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Fixtures for the Python boundary tests (plan §5).

The four fixtures here encode invariants that no single test owns: the golden
stores are read-only and produced by Rust, the extension types are registered,
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

from pse._build import build_info
from pse.contracts.extension_types import EXTENSION_NAMES

#: Modules `pse` itself must not pull in at import (blueprint §21.6, §3.1).
#:
#: pyarrow 25 imports numpy itself, so the check is differential: whatever
#: `import pyarrow` already costs is the baseline, and `import pse` must add
#: nothing to it. `pse._array` is watched alongside because it is the module
#: whose import *would* be ours, and the crisp statement of the invariant.
FORBIDDEN_ON_IMPORT = ("numpy", "scipy", "pyomo", "pint", "idaes", "pandas")
WATCHED_ON_IMPORT = (*FORBIDDEN_ON_IMPORT, "pse._array")

#: Repository root: python/pse/tests/conftest.py -> pse-arrow/
REPO_ROOT = Path(__file__).resolve().parents[3]


@pytest.fixture(scope="session")
def golden_root() -> Path:
    """Locate the golden stores.

    Returns:
        The directory ``cargo xtask golden`` writes to, honouring
        ``PSE_GOLDEN_DIR`` from ``.envrc``.
    """
    configured = os.environ.get("PSE_GOLDEN_DIR")
    return Path(configured) if configured else REPO_ROOT / "tests" / "golden"


@pytest.fixture
def golden(request: pytest.FixtureRequest, golden_root: Path) -> Path:
    """Open the golden store named by the test's ``golden(name)`` marker.

    Golden stores are produced by ``cargo xtask golden`` and are read-only from
    Python: a Python test that could rewrite the oracle is not an oracle.

    Args:
        request: The requesting test, whose ``golden`` marker names the store.
        golden_root: Where the stores live.

    Returns:
        The store's directory.
    """
    marker = request.node.get_closest_marker("golden")
    if marker is None or not marker.args:
        pytest.fail(
            "the `golden` fixture requires @pytest.mark.golden(<name>) on the test",
            pytrace=False,
        )
    name = str(marker.args[0])
    store = golden_root / name
    if not store.is_dir():
        pytest.fail(
            f"golden store {name!r} is missing at {store}. Golden stores are "
            "produced by `cargo xtask golden` (Rust writes them, Python only "
            "reads them); none exist in phase 0.",
            pytrace=False,
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
        "already costs; numpy, scipy, pyomo, pint, idaes and pandas are not "
        "platform dependencies (blueprint §3.1, §21.6)"
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
            return cls()
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
