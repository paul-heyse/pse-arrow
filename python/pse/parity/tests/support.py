# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""The pse side of the parity comparisons.

A runtime, authored packages over the physical primitives or the reference libraries,
member identities by path, and typed result rows.
"""

from pathlib import Path

import pyarrow as pa

import pse
from pse.contracts.identities import DeclarationId
from pse.contracts.values import SemanticId

#: python/pse/parity/tests/support.py -> pse-arrow/
ROOT = Path(__file__).resolve().parents[4]
#: The physical primitives' neutral dimensionless and time quantity types.
SCALAR = SemanticId(bytes([0x1F]) * 16)
TIME = SemanticId(bytes([0xDE]) * 16)


def runtime(spill: Path) -> pse.Runtime:
    """The in-memory runtime the comparisons share: a process configures one budget."""
    return pse.Runtime(
        pse.EngineSettings(
            # Every live package holds its compiler workspace allowance, and the
            # comparisons keep several packages alive in one process.
            memory_limit_bytes=64 << 30,
            threads=1,
            spill_dir=str(spill),
            max_spill_bytes=1 << 30,
            batch_size=1024,
        )
    )


def package(
    runtime: pse.Runtime, text: str
) -> tuple[pse.ModelingPackage, dict[str, DeclarationId]]:
    """An authored package over the physical primitives, by declaration name.

    The package's manifest aliases `Scalar` and `Time`.
    """
    primitives = ROOT / "tests/fixtures/packages/physical-primitives"
    physical = runtime.physical_from_documents(
        {
            str(path.relative_to(primitives)): path.read_text()
            for path in primitives.rglob("*")
            if path.is_file()
        }
    )
    manifest = (
        (ROOT / "tests/fixtures/packages/minimal_explicit/package.toml")
        .read_text()
        .replace('id_policy = "explicit"', 'id_policy = "named"')
    )
    for name, quantity in (("Scalar", SCALAR), ("Time", TIME)):
        manifest += (
            f'\n[[quantity_aliases]]\nname = "{name}"\n'
            f'quantity_type_id = "{quantity.to_hex()}"\n'
        )
    authored = runtime.modeling_from_documents(
        [{"package.toml": manifest, "models/parity.pse": text}], physical
    )
    return authored, {row.name: row.declaration_id for row in authored.declarations()}


def documents(path: Path) -> dict[str, str]:
    """A package directory's documents by relative path."""
    return {
        p.relative_to(path).as_posix(): p.read_text()
        for p in path.rglob("*")
        if p.is_file() and p.suffix in {".toml", ".yaml", ".yml", ".pse"}
    }


def reference_package(runtime: pse.Runtime, text: str) -> pse.ModelingPackage:
    """An authored package that uses the reference libraries (`control`, `math`).

    The package's manifest takes the libraries' `Scalar` and `Time` aliases.
    """
    reference = ROOT / "packages/reference"
    physical = runtime.physical_from_documents(documents(reference / "physical"))
    manifest = """[package]
package_id = "5a1b2c3d4e5f60718293a4b5c6d7e8f9"
name = "parity.reference"
version = "1.0.0"
kind = "model"
id_policy = "named"
dependencies = [
  { package_id = "12104e13f2494492ba75de08955299a6", version_req = "=1.0.0" },
  { package_id = "b27409be5572b8712e47db271fae28cd", version_req = "=1.0.0" },
]
doc = "Parity comparison models."

[[quantity_aliases]]
name = "Scalar"
quantity_type_id = "dc255c612cf27e30cb835377c8dafcf4"

[[quantity_aliases]]
name = "Time"
quantity_type_id = "e2ccf6d0a394403db967f4f35b83cb7c"
"""
    return runtime.modeling_from_documents(
        [{"package.toml": manifest, "models/parity.pse": text}]
        + [
            documents(reference / name)
            for name in ("process", "thermodynamics", "methods", "domain", "physical")
        ],
        physical,
    )


def members(
    authored: pse.ModelingPackage, case: DeclarationId, settings: pse.SolveSettings
) -> dict[str, SemanticId]:
    """Every member of `case` by its lineage path."""
    inspected = authored.inspect(case, settings)["members"]
    assert isinstance(inspected, list)
    identities: dict[str, SemanticId] = {}
    for member in inspected:
        assert isinstance(member, dict)
        lineage = member["lineage"]
        assert isinstance(lineage, dict)
        path, identity = lineage["path"], member["id"]
        assert isinstance(path, str)
        assert isinstance(identity, str)
        identities[path] = SemanticId.from_hex(identity)
    return identities


def by_suffix(identities: dict[str, SemanticId], name: str) -> SemanticId:
    """The one member whose path is `name` or ends in `.name`."""
    found = [
        identity
        for path, identity in identities.items()
        if path == name or path.endswith(f".{name}")
    ]
    assert len(found) == 1, (name, sorted(identities))
    return found[0]


def rows(table: object) -> list[dict[str, object]]:
    """A published relation stream as Python rows."""
    return pa.RecordBatchReader.from_stream(table).read_all().to_pylist()


def identity(value: object) -> SemanticId:
    """A row's identity column."""
    assert isinstance(value, bytes)
    return SemanticId(value)


def real(value: object) -> float:
    """A row's finite real column."""
    assert isinstance(value, float)
    return value


def record(value: object) -> dict[str, object]:
    """A row's record column."""
    assert isinstance(value, dict)
    return value


def records(value: object) -> list[dict[str, object]]:
    """A row's list-of-records column."""
    assert isinstance(value, list)
    return [record(item) for item in value]
