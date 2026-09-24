# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""The explicit and dynamic contract-type lint (blueprint §21.5, §24.1)."""

import subprocess
import sys
from pathlib import Path
from typing import Any

import attrs
import pytest

import pse
import pse.contracts
from pse import governance
from pse._build import _check_native_compatibility
from pse.codec import converter
from pse.governance import ContractTypeError


@attrs.frozen
class _Good:
    name: str
    ordinals: tuple[int, ...]
    labels: dict[str, str]


@attrs.frozen
class _AnyField:
    payload: Any


@attrs.frozen
class _BareDict:
    payload: dict


@attrs.frozen
class _BareList:
    payload: list


@pytest.mark.unit
def test_specific_types_pass() -> None:
    governance.check_class(_Good)


@pytest.mark.unit
@pytest.mark.parametrize(
    ("cls", "reason"),
    [
        (_AnyField, "typing.Any"),
        (_BareDict, "a bare dict"),
        (_BareList, "a bare list"),
    ],
)
def test_unconstrained_types_fail(cls: type, reason: str) -> None:
    with pytest.raises(ContractTypeError) as caught:
        governance.check_class(cls)
    assert caught.value.reason == reason
    assert "payload" in str(caught.value)


@pytest.mark.unit
def test_generated_contracts_pass_the_lint() -> None:
    governance.check()


@attrs.frozen
class _NestedAny:
    payload: list[dict[str, Any]]


@attrs.frozen
class _NestedBare:
    payload: tuple[list, ...]


@attrs.frozen
class _Recursive:
    children: "list[_Recursive]"


@pytest.mark.unit
@pytest.mark.parametrize("cls", [_NestedAny, _NestedBare])
def test_nested_annotations_fail_before_converter_hook_generation(cls: type) -> None:
    with pytest.raises(ContractTypeError):
        governance.check_class(cls)
    with pytest.raises(ContractTypeError):
        converter().get_structure_hook(cls)


@pytest.mark.unit
def test_recursive_specific_annotations_terminate() -> None:
    governance.check_class(_Recursive)


@pytest.mark.unit
def test_import_checks_compatibility_without_exhaustive_class_scan() -> None:
    subprocess.run(
        [
            sys.executable,
            "-c",
            (
                "import sys, pse; assert 'pse.governance' not in sys.modules; "
                "assert 'pse.contracts.authored' not in sys.modules"
            ),
        ],
        check=True,
    )


@pytest.mark.unit
def test_native_compatibility_refuses_version_or_fingerprint_mismatch(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    with pytest.raises(ImportError, match="versions differ"):
        _check_native_compatibility("incompatible")
    monkeypatch.setattr(pse.contracts, "REGISTRY_FINGERPRINT", "incompatible")
    with pytest.raises(ImportError, match="registry differ"):
        _check_native_compatibility(pse.__version__)


@pytest.mark.unit
def test_generation_lints_the_candidate_instead_of_installed_contracts(
    tmp_path: Path,
) -> None:
    candidate = tmp_path / "python/pse/contracts"
    candidate.mkdir(parents=True)
    (candidate / "__init__.py").write_text("", encoding="utf-8")
    (candidate / "invalid.py").write_text(
        (
            "import attrs, typing\n@attrs.frozen\nclass Invalid:\n"
            "    value: list[typing.Any]\n"
        ),
        encoding="utf-8",
    )
    script = Path(__file__).resolve().parents[3] / "scripts/check_python_contracts.py"
    result = subprocess.run(
        [sys.executable, str(script), "--root", str(tmp_path)],
        check=False,
        capture_output=True,
        text=True,
    )
    assert result.returncode != 0
    assert "contracts.invalid.Invalid.value" in result.stderr
