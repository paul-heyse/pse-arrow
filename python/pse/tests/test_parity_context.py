# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Pure ownership controls for parity's session source snapshots."""

from collections.abc import Mapping, Sequence
from pathlib import Path
from typing import cast

import pytest

import pse
from pse.parity.tests import support


class PackageProbe:
    def declarations(self) -> tuple[()]:
        return ()


class RuntimeProbe:
    def __init__(self) -> None:
        self.physical_calls: list[dict[str, str | bytes]] = []
        self.model_calls: list[
            tuple[tuple[Mapping[str, str | bytes], ...], pse.PhysicalContext]
        ] = []
        self.refuse = False

    def physical_from_documents(
        self, documents: Mapping[str, str | bytes]
    ) -> pse.PhysicalContext:
        self.physical_calls.append(dict(documents))
        if self.refuse:
            raise RuntimeError("source admission refused")
        return cast("pse.PhysicalContext", object())

    def modeling_from_documents(
        self,
        documents: Sequence[Mapping[str, str | bytes]],
        physical: pse.PhysicalContext,
    ) -> pse.ModelingPackage:
        self.model_calls.append((tuple(documents), physical))
        return cast("pse.ModelingPackage", PackageProbe())


def source_tree(root: Path) -> Path:
    primitives = root / "tests/fixtures/packages/physical-primitives"
    primitives.mkdir(parents=True)
    (primitives / "package.toml").write_bytes(b"primitive\r\nexact bytes\n")
    manifest = root / "tests/fixtures/packages/minimal_explicit/package.toml"
    manifest.parent.mkdir(parents=True)
    manifest.write_text('id_policy = "explicit"\ndependencies = []\n')
    for name in (
        "data/references",
        "process",
        "thermodynamics",
        "methods",
        "domain",
        "physical",
    ):
        directory = root / "packages/reference" / name
        directory.mkdir(parents=True)
        (directory / "package.toml").write_bytes(f"{name}\r\n".encode())
    return primitives / "package.toml"


@pytest.mark.unit
def test_parity_context_reuses_admission_and_retains_exact_session_snapshot(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    source = source_tree(tmp_path)
    monkeypatch.setattr(support, "ROOT", tmp_path)
    runtime = RuntimeProbe()
    context = support.ParityContext(cast("pse.Runtime", runtime))
    assert context.runtime is runtime
    assert runtime.physical_calls == []
    first, _ = support.package(context, "first model")
    source.write_bytes(b"changed source\n")
    second, _ = support.package(context, "second model")
    assert first is not second
    assert runtime.physical_calls == [{"package.toml": b"primitive\r\nexact bytes\n"}]
    assert runtime.model_calls[0][1] is runtime.model_calls[1][1]
    assert runtime.model_calls[0][0][0]["models/parity.pse"] == "first model"
    assert runtime.model_calls[1][0][0]["models/parity.pse"] == "second model"
    another_runtime = RuntimeProbe()
    another = support.ParityContext(cast("pse.Runtime", another_runtime))
    assert another.primitives is not context.primitives
    assert another_runtime.physical_calls == [{"package.toml": b"changed source\n"}]


@pytest.mark.unit
def test_parity_reference_closure_owns_immutable_bytes_and_fresh_model_packages(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    source_tree(tmp_path)
    monkeypatch.setattr(support, "ROOT", tmp_path)
    runtime = RuntimeProbe()
    context = support.ParityContext(cast("pse.Runtime", runtime))
    original = context.reference_documents
    with pytest.raises(TypeError):
        cast("dict[str, bytes]", original[-1])["package.toml"] = b"mutated"
    first = support.reference_package(context, "first reference model")
    (tmp_path / "packages/reference/physical/package.toml").write_bytes(b"changed")
    second = support.reference_package(context, "second reference model")
    assert first is not second
    assert context.reference_documents is original
    assert runtime.physical_calls == [dict(original[-1])]
    assert runtime.model_calls[0][1] is runtime.model_calls[1][1]
    for sources, _ in runtime.model_calls:
        assert all(
            left is right for left, right in zip(sources[1:], original, strict=True)
        )
        assert sources[-1] == runtime.physical_calls[0]


@pytest.mark.unit
def test_parity_failed_physical_admission_never_publishes_a_cached_context(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    source_tree(tmp_path)
    monkeypatch.setattr(support, "ROOT", tmp_path)
    runtime = RuntimeProbe()
    runtime.refuse = True
    context = support.ParityContext(cast("pse.Runtime", runtime))
    with pytest.raises(RuntimeError, match="source admission refused"):
        _ = context.primitives
    assert runtime.model_calls == []
    runtime.refuse = False
    physical = context.primitives
    assert len(runtime.physical_calls) == 2
    assert context.primitives is physical
    assert len(runtime.physical_calls) == 2
