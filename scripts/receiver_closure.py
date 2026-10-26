# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Derive and probe an immutable receiver's actual owned Python import closure."""

from __future__ import annotations

import ast
import hashlib
import importlib.util
import json
import os
import subprocess
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence


class ClosureError(ValueError):
    """The selected receiver has an unresolved owned import or frozen dependency."""


def _module_file(root: Path, module: str) -> Path | None:
    path = root.joinpath(*module.split("."))
    for selected in (path.with_suffix(".py"), path / "__init__.py"):
        if selected.is_file():
            return selected
    return None


def source_files(
    root: Path,
    entrypoints: Sequence[str],
    *,
    dynamic: Mapping[str, Sequence[str]] | None = None,
) -> tuple[str, ...]:
    """Follow owned imports, including regular-package parents and declared dynamics.

    Computed imports require their owner's explicit finite declaration. Literal
    module imports are followed normally. External packages retain the selected
    interpreter's installed distribution authority and are checked by the probe.
    """
    root = root.resolve(strict=True)
    dynamic = {} if dynamic is None else dynamic
    pending = list(entrypoints)
    selected: set[str] = set()

    def include(module: str, *, required: bool) -> None:
        if module != "scripts" and not module.startswith("scripts."):
            return
        source = _module_file(root, module)
        namespace = root.joinpath(*module.split("."))
        if source is None:
            if namespace.is_dir():
                return  # An existing namespace package has no synthetic initializer.
            if required:
                raise ClosureError(f"Owned receiver import is missing: {module}")
            return
        relative = source.relative_to(root).as_posix()
        pending.append(relative)
        for parent in source.relative_to(root).parents:
            if parent == Path():
                continue
            initializer = root / parent / "__init__.py"
            if initializer.is_file():
                pending.append(initializer.relative_to(root).as_posix())

    while pending:
        relative = pending.pop()
        if relative in selected:
            continue
        path = root / relative
        if (
            not path.is_file()
            or path.is_symlink()
            or not path.resolve().is_relative_to(root)
            or Path(relative).is_absolute()
            or ".." in Path(relative).parts
        ):
            raise ClosureError(f"Receiver source is missing or aliased: {relative}")
        selected.add(relative)
        module = relative.removesuffix(".py").replace("/", ".")
        package = (
            module.removesuffix(".__init__")
            if path.name == "__init__.py"
            else module.rpartition(".")[0]
        )
        include(package, required=True)
        try:
            tree = ast.parse(path.read_bytes(), filename=relative)
        except (SyntaxError, ValueError) as error:
            raise ClosureError(
                f"Receiver source cannot be parsed: {relative}"
            ) from error
        computed = False
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                for imported in node.names:
                    include(imported.name, required=True)
            elif isinstance(node, ast.ImportFrom):
                imported = node.module or ""
                if node.level:
                    try:
                        imported = importlib.util.resolve_name(
                            "." * node.level + imported, package
                        )
                    except (ImportError, ValueError) as error:
                        raise ClosureError(
                            f"Invalid receiver relative import: {relative}"
                        ) from error
                include(imported, required=True)
                if (
                    imported == "scripts"
                    or (root.joinpath(*imported.split("."))).is_dir()
                ):
                    for name in node.names:
                        if name.name == "*":
                            raise ClosureError(
                                f"Owned wildcard receiver import: {relative}"
                            )
                        provider = _module_file(root, imported)
                        declared = set()
                        if provider is not None:
                            for statement in ast.parse(provider.read_bytes()).body:
                                if isinstance(
                                    statement,
                                    (
                                        ast.ClassDef,
                                        ast.FunctionDef,
                                        ast.AsyncFunctionDef,
                                    ),
                                ):
                                    declared.add(statement.name)
                                elif isinstance(
                                    statement, (ast.Import, ast.ImportFrom)
                                ):
                                    declared.update(
                                        alias.asname or alias.name.split(".")[0]
                                        for alias in statement.names
                                    )
                                elif isinstance(statement, (ast.Assign, ast.AnnAssign)):
                                    targets = (
                                        statement.targets
                                        if isinstance(statement, ast.Assign)
                                        else [statement.target]
                                    )
                                    declared.update(
                                        target.id
                                        for target in targets
                                        if isinstance(target, ast.Name)
                                    )
                        include(
                            imported + "." + name.name,
                            required=name.name not in declared,
                        )
            elif isinstance(node, ast.Call) and (
                (isinstance(node.func, ast.Name) and node.func.id == "__import__")
                or (
                    isinstance(node.func, ast.Attribute)
                    and node.func.attr == "import_module"
                )
            ):
                argument = node.args[0] if node.args else None
                if isinstance(argument, ast.Constant) and isinstance(
                    argument.value, str
                ):
                    include(argument.value, required=True)
                else:
                    computed = True
        if computed and relative not in dynamic:
            raise ClosureError(
                f"Computed receiver import needs an explicit owner declaration: {relative}"
            )
        for imported in dynamic.get(relative, ()):
            include(imported, required=True)
    return tuple(sorted(selected))


def modules(files: Sequence[str]) -> tuple[str, ...]:
    return tuple(
        sorted(
            {
                name.removesuffix(".py").replace("/", ".").removesuffix(".__init__")
                for name in files
                if name.endswith(".py")
            }
        )
    )


def probe(
    directory: Path, files: Sequence[str], interpreter: Path, *, timeout: float = 30
) -> dict[str, str]:
    """Import every frozen owned module in a fresh isolated interpreter."""
    program = r"""
import importlib, importlib.util, json, os, pathlib, sys, sysconfig
root = pathlib.Path(sys.argv[1]).resolve(strict=True)
sys.path.insert(0, str(root))
for name in json.loads(sys.argv[2]):
    importlib.import_module(name)
providers = {str(pathlib.Path(sys.executable).absolute()), str(pathlib.Path(sys.executable).resolve())}
configuration = pathlib.Path(sys.prefix) / "pyvenv.cfg"
if configuration.is_file(): providers.add(str(configuration))
library = pathlib.Path(sysconfig.get_config_var("LIBDIR") or "") / (sysconfig.get_config_var("LDLIBRARY") or "")
if library.is_file(): providers.add(str(library.absolute()))
for name, module in tuple(sys.modules.items()):
    if name == "scripts" or name.startswith("scripts."):
        source = getattr(module, "__file__", None)
        if source is not None and not pathlib.Path(source).resolve().is_relative_to(root):
            raise RuntimeError("owned module escaped frozen receiver: " + name)
        for path in getattr(module, "__path__", ()):
            if not pathlib.Path(path).resolve().is_relative_to(root):
                raise RuntimeError("owned namespace escaped frozen receiver: " + name)
    else:
        source = getattr(module, "__file__", None)
        if source and pathlib.Path(source).is_file():
            if source.endswith(".pyc"):
                candidate = importlib.util.source_from_cache(source)
                if pathlib.Path(candidate).is_file(): source = candidate
            providers.add(str(pathlib.Path(source).absolute()))
print(json.dumps(sorted(providers)))
"""
    environment = {
        key: value
        for key, value in os.environ.items()
        if key not in {"PYTHONPATH", "PYTHONHOME"}
    }
    result = subprocess.run(
        [
            str(interpreter),
            "-I",
            "-B",
            "-c",
            program,
            str(directory.resolve()),
            json.dumps(modules(files)),
        ],
        cwd=directory,
        env=environment,
        capture_output=True,
        timeout=timeout,
        check=False,
    )
    if result.returncode:
        detail = result.stderr.decode(errors="replace")[-4096:]
        raise ClosureError(f"Frozen receiver import probe refused: {detail}")
    if len(result.stdout) > 1024 * 1024:
        raise ClosureError("Frozen receiver provider inventory exceeds its bound")
    try:
        paths = json.loads(result.stdout)
        if not isinstance(paths, list) or not all(
            isinstance(path, str) for path in paths
        ):
            raise ValueError("invalid provider paths")
        return {path: digest(Path(path)) for path in paths}
    except (OSError, ValueError, TypeError) as error:
        raise ClosureError("Frozen receiver provider inventory is malformed") from error


def digest(path: Path) -> str:
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()
