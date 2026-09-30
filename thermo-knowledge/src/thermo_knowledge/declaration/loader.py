# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Read a declaration from `model/` and `forms/`, validate it and resolve it (sections 1 to 6).

`load_declaration` collects every refusal: structural ones from all documents first; when
those are clean, the semantic ones, then the differences from the pipeline's shape contract
(meta-model section 5), those of the form expressions (expressions.md section 4) and the ones that
arise from projecting to PostgreSQL (identifier length and collisions).
"""

from __future__ import annotations

import tomllib
from dataclasses import dataclass
from pathlib import Path

from thermo_knowledge import config
from thermo_knowledge import pipeline_contract
from thermo_knowledge.declaration import contract_check
from thermo_knowledge.declaration import schema as s
from thermo_knowledge.declaration.diagnostics import Code, DeclarationError, Diagnostic
from thermo_knowledge.declaration.model import Declaration
from thermo_knowledge.declaration.resolve import ModuleDoc, Resolver
from thermo_knowledge.pipeline_contract import PipelineContract

MANIFEST_NAME = "manifest.toml"


def default_model_dir() -> Path:
    """`model/` of this tree."""
    return config.TREE_DIR / "model"


def default_forms_dir() -> Path:
    """`forms/` of this tree."""
    return config.TREE_DIR / "forms"


@dataclass(frozen=True)
class LoadResult:
    """A loaded declaration, or the diagnostics that refused it."""

    declaration: Declaration | None
    diagnostics: tuple[Diagnostic, ...]

    def require(self) -> Declaration:
        """The declaration; raises `DeclarationError` when it was refused."""
        if self.declaration is None or self.diagnostics:
            raise DeclarationError(self.diagnostics)
        return self.declaration


def _document_name(directory: Path, path: Path) -> str:
    return f"{directory.name}/{path.relative_to(directory).as_posix()}"


def _toml_files(directory: Path) -> list[Path]:
    if not directory.is_dir():
        return []
    return sorted(
        directory.rglob("*.toml"), key=lambda path: path.relative_to(directory).as_posix()
    )


def _parse(path: Path, document: str, diagnostics: list[Diagnostic]) -> dict[str, object] | None:
    try:
        with path.open("rb") as handle:
            return tomllib.load(handle)
    except tomllib.TOMLDecodeError as error:
        diagnostics.append(
            Diagnostic(document=document, construct="", code=Code.TOML_SYNTAX, message=str(error))
        )
    except UnicodeDecodeError as error:
        diagnostics.append(
            Diagnostic(document=document, construct="", code=Code.TOML_SYNTAX, message=str(error))
        )
    return None


def _decode_module(
    data: dict[str, object], document: str, diagnostics: list[Diagnostic]
) -> ModuleDoc | None:
    failed = False
    header_data = {key: data[key] for key in s.HEADER_KEYS if key in data}
    header = s.decode(s.ModuleHeader, header_data, "module header", document, diagnostics)
    if header is None:
        failed = True
    sections: dict[str, dict[str, s.Construct]] = {}
    entities: dict[str, dict[str, tuple[s.EntityDecl, dict[str, object]]]] = {}
    for key, value in data.items():
        if key in s.HEADER_KEYS:
            continue
        if key in s.SECTIONS:
            if not isinstance(value, dict):
                diagnostics.append(
                    Diagnostic(
                        document=document,
                        construct=key,
                        code=Code.INVALID_VALUE,
                        message="expected a table of constructs",
                    )
                )
                failed = True
                continue
            table: dict[str, s.Construct] = {}
            for name, body in value.items():
                built = s.decode(s.SECTIONS[key], body, f"{key}.{name}", document, diagnostics)
                if built is None:
                    failed = True
                else:
                    table[name] = built
            sections[key] = table
        elif key == "entities":
            if not isinstance(value, dict):
                diagnostics.append(
                    Diagnostic(
                        document=document,
                        construct=key,
                        code=Code.INVALID_VALUE,
                        message="expected tables of entities, one per kind",
                    )
                )
                failed = True
                continue
            for kind, by_name in value.items():
                if not isinstance(by_name, dict):
                    diagnostics.append(
                        Diagnostic(
                            document=document,
                            construct=f"entities.{kind}",
                            code=Code.INVALID_VALUE,
                            message="expected a table of entities",
                        )
                    )
                    failed = True
                    continue
                for name, body in by_name.items():
                    marks = (
                        {k: v for k, v in body.items() if k in s.ENTITY_MARKS}
                        if isinstance(body, dict)
                        else body
                    )
                    decl = s.decode(
                        s.EntityDecl, marks, f"entities.{kind}.{name}", document, diagnostics
                    )
                    if decl is None:
                        failed = True
                        continue
                    assert isinstance(body, dict)
                    values = {k: v for k, v in body.items() if k not in s.ENTITY_MARKS}
                    entities.setdefault(kind, {})[name] = (decl, values)
        else:
            diagnostics.append(
                Diagnostic(
                    document=document,
                    construct=key,
                    code=Code.UNKNOWN_KEY,
                    message=f"unknown key `{key}`",
                )
            )
            failed = True
    if failed or header is None:
        return None
    return ModuleDoc(document=document, header=header, sections=sections, entities=entities)


def load_declaration(
    model_dir: Path | None = None,
    forms_dir: Path | None = None,
    *,
    contract: PipelineContract | None = pipeline_contract.PACKAGED,
) -> LoadResult:
    """Load the declaration under `model_dir` (default `model/`) and `forms_dir` (default
    `forms/`), both of this tree unless given.

    The loaded declaration is checked against `contract`, the shape the stage code depends on
    (by default the contract shipped with the package). `contract=None` loads a declaration
    that is not this pipeline's model, such as a fixture of the declaration language itself."""
    model = model_dir if model_dir is not None else default_model_dir()
    forms = forms_dir if forms_dir is not None else default_forms_dir()
    diagnostics: list[Diagnostic] = []
    manifest: s.ManifestDecl | None = None
    manifest_path = model / MANIFEST_NAME
    manifest_document = f"{model.name}/{MANIFEST_NAME}"
    if manifest_path.is_file():
        data = _parse(manifest_path, manifest_document, diagnostics)
        if data is not None:
            manifest = s.decode(s.ManifestDecl, data, "manifest", manifest_document, diagnostics)
    else:
        diagnostics.append(
            Diagnostic(
                document=manifest_document,
                construct="",
                code=Code.MISSING_MANIFEST,
                message="the model directory has no manifest.toml",
            )
        )
    docs: list[ModuleDoc] = []
    for directory in (model, forms):
        for path in _toml_files(directory):
            if directory == model and path.parent == model and path.name == MANIFEST_NAME:
                continue
            document = _document_name(directory, path)
            data = _parse(path, document, diagnostics)
            if data is None:
                continue
            module = _decode_module(data, document, diagnostics)
            if module is not None:
                docs.append(module)
    if diagnostics or manifest is None:
        return LoadResult(None, tuple(sorted(diagnostics)))
    resolver = Resolver(manifest=manifest, docs=docs)
    declaration = resolver.run()
    if declaration is not None and not resolver.diagnostics and contract is not None:
        resolver.diagnostics.extend(contract_check.check(declaration, contract))
    if declaration is not None and not resolver.diagnostics:
        # Imported here: the expression checker reads the loaded declaration, like the
        # projection below.
        from thermo_knowledge.expression.check import check_declaration

        resolver.diagnostics.extend(check_declaration(declaration))
    if declaration is not None and not resolver.diagnostics:
        # Imported here: the projection builds on the loaded declaration, so the loader asks
        # it, once everything else is clean, which identifiers it would emit.
        from thermo_knowledge.generate.plan import projection_diagnostics

        resolver.diagnostics.extend(projection_diagnostics(declaration))
    found = tuple(sorted(set(resolver.diagnostics)))
    return LoadResult(None if found else declaration, found)
