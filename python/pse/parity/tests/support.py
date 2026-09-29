# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""The pse side of the parity comparisons: a runtime, authored packages over the
physical primitives, member identities by path and result rows."""

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
            memory_limit_bytes=8 << 30,
            threads=1,
            spill_dir=str(spill),
            max_spill_bytes=1 << 30,
            batch_size=1024,
        )
    )


def package(
    runtime: pse.Runtime, text: str
) -> tuple[pse.ModelingPackage, dict[str, DeclarationId]]:
    """An authored package over the physical primitives, with `Scalar` and `Time`
    aliases, and its declarations by name."""
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
        assert isinstance(path, str) and isinstance(identity, str)
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
