# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""M22 public native workflow; shared authored fixtures and independent references."""

import gc
import subprocess
import sys
from pathlib import Path

import pyarrow as pa
import pytest

import pse
from pse import codec, modeling as w
from pse.contracts import authored
from pse.contracts.values import SemanticId

FIXTURE = Path(__file__).resolve().parents[3] / "tests/fixtures/plan14"


@pytest.mark.integration
def test_public_native_process_and_exact_results(
    inspection_settings: pse.EngineSettings,
) -> None:
    runtime = pse.Runtime(inspection_settings)
    package = FIXTURE / "package"
    docs = {
        str(p.relative_to(package)): p.read_text()
        for p in package.rglob("*")
        if p.is_file()
    }
    physical = runtime.physical_from_documents(docs)
    declaration = codec.converter().structure(
        codec.decode_json((FIXTURE / "model.json").read_bytes(), dict[str, object]),
        w.ModelDeclaration,
    )
    draft = runtime.from_declaration(declaration, physical)
    for row in codec.decode_json(
        (FIXTURE / "providers.json").read_bytes(), list[dict[str, object]]
    ):
        draft.native_provider(
            codec.converter().structure(row, authored.AuthoredNativeProvidersRow)
        )
    for row in codec.decode_json(
        (FIXTURE / "balances.json").read_bytes(), list[dict[str, object]]
    ):
        draft.balance(
            codec.converter().structure(row, authored.AuthoredPhysicalBalancesRow)
        )
    revision = draft.freeze()
    selected = next(c for c in declaration.cases if c.name == "heater-recycle")
    prepared = revision.prepare(
        selected.case_id,
        pse.SolveSettings(
            backend="ipopt",
            intent="feasible_point",
            variable_tolerances=[1e-6] * 3,
            row_tolerances=[1e-4] * 3,
        ),
    )
    result = prepared.start().wait()
    table = pa.RecordBatchReader.from_stream(
        result.table("runtime.solve_variables")
    ).read_all()
    rows = table.to_pylist()
    for source, expected in zip(
        selected.variables, [76.85, 34.565566349336066, 5.0], strict=True
    ):
        actual = next(
            row["value"]
            for row in rows
            if SemanticId(row["symbol_id"]) == source.port.symbol_id
        )
        assert actual == pytest.approx(expected, abs=1e-5)
    source = pa.RecordBatchReader.from_stream(
        result.table("authored.computation_models")
    ).read_all()
    assert source.num_rows == 1
    arrays = table.column("value").chunks
    del table, result, prepared, revision, draft, runtime
    gc.collect()
    assert any(a.null_count < len(a) for a in arrays)


@pytest.mark.integration
def test_import_without_pyomo() -> None:
    program = """
import importlib.abc, sys
class NoPyomo(importlib.abc.MetaPathFinder):
    def find_spec(self, fullname, path=None, target=None):
        if fullname == 'pyomo' or fullname.startswith('pyomo.'):
            raise AssertionError('production attempted to import Pyomo')
sys.meta_path.insert(0, NoPyomo())
import pse
assert pse.Runtime and pse.ModelBuilder and pse.SolveSettings
assert not any(n == 'pyomo' or n.startswith('pyomo.') for n in sys.modules)
"""
    subprocess.run(
        [sys.executable, "-c", program], check=True, capture_output=True, text=True
    )


@pytest.mark.unit
def test_shared_source_contracts() -> None:
    """Pure generated declaration decoding, without runtime construction or execution."""
    declaration = codec.converter().structure(
        codec.decode_json((FIXTURE / "model.json").read_bytes(), dict[str, object]),
        w.ModelDeclaration,
    )
    assert {c.name for c in declaration.cases} == {
        "heater-recycle",
        "heater-optimization",
        "flash",
        "separator-0",
        "separator-1",
        "separator-2",
    }
    assert declaration.domains and declaration.groups
    assert any(c.objective is not None for c in declaration.cases)
    for row in codec.decode_json(
        (FIXTURE / "providers.json").read_bytes(), list[dict[str, object]]
    ):
        codec.converter().structure(row, authored.AuthoredNativeProvidersRow)
    for row in codec.decode_json(
        (FIXTURE / "balances.json").read_bytes(), list[dict[str, object]]
    ):
        codec.converter().structure(row, authored.AuthoredPhysicalBalancesRow)
    malformed = codec.decode_json(
        (FIXTURE / "model.json").read_bytes(), dict[str, object]
    )
    malformed["obsolete_math_ir"] = []
    with pytest.raises(ExceptionGroup):
        codec.converter().structure(malformed, w.ModelDeclaration)
