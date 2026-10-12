# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""The pse side of the parity comparisons.

A runtime, authored packages over the physical primitives or the reference libraries,
member identities by path, and typed result rows.
"""

import os
from collections.abc import Mapping
from pathlib import Path
from types import MappingProxyType

import pyarrow as pa

import pse
from pse.contracts import documents as contract_documents
from pse.contracts.identities import DeclarationId
from pse.contracts.values import SemanticId

#: python/pse/parity/tests/support.py -> pse-arrow/
ROOT = Path(__file__).resolve().parents[4]

# The generated document derives this value from the Rust numerical policy.
ENGINEERING_RELATIVE_FRACTION = (
    contract_documents.NumericalPolicy().engineering_relative_fraction
)


def scalar_canonical_allowance() -> float:
    """Default engineering budget for canonical Scalar without declared context.

    Scalar is dimensionless and its canonical fallback is one canonical unit.
    These PID outputs have no authored or quantity engineering nominal; the
    budget therefore uses the generated global fraction times that same floor.
    No reference answer chooses the scale. Dimensional quantities and covariance
    entries require their admitted context instead of this Scalar-only helper.
    """
    return ENGINEERING_RELATIVE_FRACTION * 1.0


def runtime(spill: Path) -> pse.Runtime:
    """The configured canonical runtime; one process configures one budget."""
    return pse.Runtime(
        pse.EngineSettings(
            # Every live package holds its compiler workspace allowance, and the
            # comparisons keep several packages alive in one process.
            memory_limit_bytes=64 << 30,
            threads=1,
            spill_dir=str(spill),
            max_spill_bytes=1 << 30,
            batch_size=1024,
        ),
        substrate=os.environ["PSE_SURREAL_STATE"],
    )


class ParityContext:
    """Session owner of one Runtime and exact immutable admitted source snapshots.

    Only physical admission and source bytes survive between comparisons. Each
    authored model package and every scientific operation remain independently built.
    """

    def __init__(self, runtime: pse.Runtime) -> None:
        self._runtime = runtime
        self._primitives: tuple[Mapping[str, bytes], pse.PhysicalContext] | None = None
        self._reference: tuple[Mapping[str, bytes], pse.PhysicalContext] | None = None
        self._reference_documents: tuple[Mapping[str, bytes], ...] | None = None

    @property
    def runtime(self) -> pse.Runtime:
        """The actual admission owner, fixed for this session."""
        return self._runtime

    @property
    def primitives(self) -> pse.PhysicalContext:
        """Lazily admit and retain the physical primitives' exact byte snapshot."""
        if self._primitives is None:
            source = documents(
                ROOT / "tests/fixtures/packages/physical-primitives", all_files=True
            )
            physical = self._runtime.physical_from_documents(source)
            self._primitives = source, physical
        return self._primitives[1]

    @property
    def reference_physical(self) -> pse.PhysicalContext:
        """Lazily admit the same physical source used by the reference closure."""
        if self._reference is None:
            source = self.reference_documents[-1]
            physical = self._runtime.physical_from_documents(source)
            self._reference = source, physical
        return self._reference[1]

    @property
    def reference_documents(self) -> tuple[Mapping[str, bytes], ...]:
        """Immutable library source bytes; no admitted model packages or results."""
        if self._reference_documents is None:
            reference = ROOT / "packages/reference"
            self._reference_documents = tuple(
                documents(reference / name)
                for name in (
                    "data/references",
                    "process",
                    "thermodynamics",
                    "methods",
                    "domain",
                    "physical",
                )
            )
        return self._reference_documents


def package(
    context: ParityContext, text: str
) -> tuple[pse.ModelingPackage, dict[str, DeclarationId]]:
    """An authored package over the physical primitives, by declaration name.

    The package's manifest depends on the primitives, whose physical document names
    `Scalar` and `Time`.
    """
    physical = context.primitives
    manifest = (
        (ROOT / "tests/fixtures/packages/minimal_explicit/package.toml")
        .read_text()
        .replace('id_policy = "explicit"', 'id_policy = "named"')
        .replace(
            "dependencies = []",
            'dependencies = [{ package_id = "5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a", '
            'version_req = { operator = "exact", major = 1, minor = 0, patch = 0 } }]',
        )
    )
    authored = context.runtime.modeling_from_documents(
        [{"package.toml": manifest, "models/parity.pse": text}], physical
    )
    return authored, {row.name: row.declaration_id for row in authored.declarations()}


def documents(path: Path, *, all_files: bool = False) -> Mapping[str, bytes]:
    """An immutable snapshot of original document bytes, by relative path."""
    return MappingProxyType(
        {
            p.relative_to(path).as_posix(): p.read_bytes()
            for p in sorted(path.rglob("*"))
            if p.is_file()
            and (
                all_files or p.suffix in {".toml", ".yaml", ".yml", ".pse", ".parquet"}
            )
        }
    )


def reference_package(context: ParityContext, text: str) -> pse.ModelingPackage:
    """An authored package that uses the reference libraries (`control`, `math`).

    The package's manifest depends on `pse.physical`, whose physical document names
    `Scalar` and `Time`.
    """
    physical = context.reference_physical
    manifest = """[package]
package_id = "5a1b2c3d4e5f60718293a4b5c6d7e8f9"
name = "parity.reference"
version = "1.0.0"
kind = "model"
id_policy = "named"
dependencies = [
  { package_id = "12104e13f2494492ba75de08955299a6", version_req = {
      operator = "exact", major = 1, minor = 0, patch = 0 } },
  { package_id = "b27409be5572b8712e47db271fae28cd", version_req = {
      operator = "exact", major = 1, minor = 0, patch = 0 } },
]
doc = "Parity comparison models."
"""
    return context.runtime.modeling_from_documents(
        [{"package.toml": manifest, "models/parity.pse": text}]
        + list(context.reference_documents),
        physical,
    )


def members(
    authored: pse.ModelingPackage, case: DeclarationId, settings: pse.SolveSettings
) -> dict[str, SemanticId]:
    """Every member of `case` by its lineage path."""
    inspected = authored.inspect(case, settings).members
    identities: dict[str, SemanticId] = {}
    for member in inspected:
        identities[member.lineage.path] = SemanticId.from_hex(member.id)
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
