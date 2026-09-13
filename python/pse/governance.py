# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""The import-time contract-type lint (blueprint §21.5, plan §5).

Every generated contract class is fully typed: a field whose resolved type is
``typing.Any``, a bare ``dict`` or a bare ``list`` would let an unvalidated
payload through the boundary the contract layer exists to defend, so
:func:`check` walks every attrs class defined under :mod:`pse.contracts` when
``pse`` is imported and raises :class:`ContractTypeError` on the first offence.

The lint runs at import, not only in CI, because a generated file that drifts is
a production hazard, not a review nit. ``ruff``'s banned-api rule catches the
same mistake in hand-written code; this catches it in generated code, where the
fix is to the generator.
"""

import importlib
import pkgutil
from types import ModuleType

import attrs

# `typing.Any` cannot be named here: the ruff banned-api rule that forbids it in
# contracts has no exemption for this module, and an exemption would be a hole in
# exactly the lint this module implements. It is identified structurally instead.
_ANY_MODULE = "typing"
_ANY_NAME = "Any"


class ContractTypeError(TypeError):
    """A contract class carries a field whose type does not constrain anything.

    Raised by :func:`check` and :func:`check_class`. The message names the class,
    the field and the offending type so the fix lands in the registry the
    generator reads, never in the generated file.
    """

    def __init__(self, cls: type, field_name: str, reason: str) -> None:
        """Build the error from the offending class, field and reason.

        Args:
            cls: The contract class that failed the lint.
            field_name: The attrs field whose type is not specific enough.
            reason: What was found, in the vocabulary of blueprint §21.5.
        """
        super().__init__(
            f"{cls.__module__}.{cls.__qualname__}.{field_name} is typed {reason}; "
            "contracts are fully typed (blueprint §21.5). Fix the registry the "
            "generator reads, then `just codegen`."
        )
        self.contract_class = cls
        self.field_name = field_name
        self.reason = reason


def _is_any(annotation: object) -> bool:
    """Report whether an annotation is ``typing.Any``.

    Args:
        annotation: The resolved annotation object.

    Returns:
        True when the annotation is the ``Any`` special form.
    """
    if getattr(annotation, "__module__", None) == _ANY_MODULE and (
        getattr(annotation, "__name__", None) == _ANY_NAME
    ):
        return True
    return repr(annotation) == f"{_ANY_MODULE}.{_ANY_NAME}"


def _describe_offence(annotation: object) -> str | None:
    """Classify an annotation against the §21.5 ban list.

    Args:
        annotation: The resolved annotation object.

    Returns:
        A short description of the offence, or None when the annotation is
        specific enough.
    """
    if _is_any(annotation):
        return "typing.Any"
    if annotation is dict:
        return "a bare dict"
    if annotation is list:
        return "a bare list"
    return None


def check_class(cls: type) -> None:
    """Assert that every attrs field of ``cls`` carries a specific type.

    Args:
        cls: An attrs class to lint.

    Raises:
        ContractTypeError: If a field resolves to ``typing.Any``, a bare
            ``dict`` or a bare ``list``.
    """
    resolved = attrs.resolve_types(cls)
    for field in attrs.fields(resolved):
        offence = _describe_offence(field.type)
        if offence is not None:
            raise ContractTypeError(cls, field.name, offence)


def _contract_classes(module: ModuleType) -> list[type]:
    """Collect the attrs classes a module defines itself.

    Args:
        module: The module to inspect.

    Returns:
        The attrs classes whose ``__module__`` is this module, so a re-export
        is linted once, where it is defined.
    """
    found: list[type] = []
    for name in dir(module):
        if name.startswith("_"):
            continue
        candidate = getattr(module, name)
        if (
            isinstance(candidate, type)
            and attrs.has(candidate)
            and candidate.__module__ == module.__name__
        ):
            found.append(candidate)
    return found


def check(module: ModuleType | None = None) -> None:
    """Lint every attrs contract class reachable from ``module``.

    Args:
        module: The package to walk. Defaults to :mod:`pse.contracts`, which is
            what ``import pse`` checks.

    Raises:
        ContractTypeError: If any field of any contract class is typed
            ``typing.Any``, a bare ``dict`` or a bare ``list``.
    """
    root = importlib.import_module("pse.contracts") if module is None else module
    modules = [root]
    search_path = getattr(root, "__path__", None)
    if search_path is not None:
        prefix = f"{root.__name__}."
        modules.extend(
            importlib.import_module(info.name)
            for info in pkgutil.walk_packages(search_path, prefix=prefix)
        )
    for candidate_module in modules:
        for cls in _contract_classes(candidate_module):
            check_class(cls)
