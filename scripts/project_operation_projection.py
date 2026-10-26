# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Render the finite owned public facade from its actual typed receiver glue AST.

No scientific package is imported. Bodies remain in their typed owners; this
projection publishes only the original declared methods and argument forwarding.
"""

from __future__ import annotations

import ast
import copy
import inspect
import re
import sys
import textwrap
import tomllib
from pathlib import Path

from pydantic import JsonValue, TypeAdapter

FAMILIES = {
    "project": (
        "pse._project_operations_glue",
        "ProjectRuntimeGlue",
        "ProjectRuntimeOperations",
        "Runtime",
        "pse._workflow",
    ),
    "research": (
        "pse._research",
        "ResearchRuntimeGlue",
        "ResearchRuntime",
        "ResearchRuntime",
        "pse._research",
    ),
    "study": (
        "pse._study_workbench",
        "StudyWorkbenchGlue",
        "StudyWorkbenchMixin",
        "StudyWorkbenchMixin",
        "pse._study_workbench",
    ),
}
BUILTINS = {
    "self",
    "str",
    "int",
    "float",
    "bool",
    "bytes",
    "list",
    "dict",
    "tuple",
    "set",
    "frozenset",
    "object",
    "None",
}
ROWS = TypeAdapter(list[dict[str, JsonValue]])


def validate_python_floor(root: Path, source: str) -> None:
    """Check publication against the package's declared interpreter floor."""
    declared = tomllib.loads((root / "pyproject.toml").read_text())["project"][
        "requires-python"
    ]
    if not isinstance(declared, str):
        raise TypeError("project requires-python must declare its interpreter floor")
    floor = re.fullmatch(r">=(\d+)\.(\d+)(?:\.\d+)?", declared)
    if floor is None:
        raise ValueError(f"unsupported package interpreter floor {declared!r}")
    ast.parse(
        source,
        filename="<owned-project-operation-projection>",
        feature_version=(int(floor[1]), int(floor[2])),
    )


def text(value: JsonValue) -> str:
    if not isinstance(value, str):
        raise TypeError("project projection identity must be text")
    return value


def annotation_names(node: ast.AST) -> set[str]:
    names = {item.id for item in ast.walk(node) if isinstance(item, ast.Name)}
    for item in ast.walk(node):
        if isinstance(item, ast.Constant) and isinstance(item.value, str):
            try:
                parsed = ast.parse(item.value, mode="eval")
            except SyntaxError:
                continue
            names.update(
                sub.id for sub in ast.walk(parsed) if isinstance(sub, ast.Name)
            )
    return names - BUILTINS


def owned_defaults(signature: JsonValue) -> dict[str, ast.expr]:
    """Lower only the finite calling defaults supplied by the Rust declaration."""
    if signature is None:
        return {}
    value = text(signature)
    body = value[value.index("(") + 1 : value.rindex(")")]
    defaults: dict[str, ast.expr] = {}
    for match in re.finditer(r"([A-Za-z_]\w*)\s*=\s*([^,]+)", body):
        name, expression = match.groups()
        expression = expression.strip()
        if expression == "None":
            converted = "None"
        elif expression in {"false", "true"}:
            converted = "False" if expression == "false" else "True"
        elif expression == "pse_runtime::host_operations::project::DEFAULT_BRANCH":
            converted = "DEFAULT_BRANCH"
        else:
            raise ValueError(f"unsupported owned public project default {expression}")
        defaults[name] = ast.parse(converted, mode="eval").body
    return defaults


def documented_effects(value: JsonValue) -> str:
    """Publish exact structured owner facts through the existing help consumer."""
    if not isinstance(value, dict):
        raise TypeError("project operation requires structured effect facts")
    rows = [f"    scope: {text(value['scope'])}"]
    for field in ("read_only", "idempotent", "destructive"):
        flag = value[field]
        if not isinstance(flag, bool):
            raise TypeError(f"project effect {field} must be boolean")
        rows.append(f"    {field}: {'true' if flag else 'false'}")
    return "Effects:\n" + "\n".join(rows)


def render(root: Path, declarations: list[dict[str, JsonValue]]) -> str:
    line_length = tomllib.loads((root / "pyproject.toml").read_text())["tool"]["ruff"][
        "line-length"
    ]
    if not isinstance(line_length, int) or isinstance(line_length, bool):
        raise TypeError("project Ruff line-length must be an integer")
    if line_length <= 11:
        raise ValueError(
            "project Ruff line-length must accommodate the generated docstring indentation"
        )
    imports: dict[str, set[str]] = {}
    blocks: list[str] = []
    published: set[str] = set()
    for family, (
        module,
        glue_name,
        public_class,
        qualifier,
        public_module,
    ) in FAMILIES.items():
        source = root / "python" / (module.replace(".", "/") + ".py")
        tree = ast.parse(source.read_text())
        glue = next(
            (
                node
                for node in tree.body
                if isinstance(node, ast.ClassDef) and node.name == glue_name
            ),
            None,
        )
        if glue is None:
            raise ValueError(
                f"original project receiver glue is missing: {module}.{glue_name}"
            )
        functions = {
            node.name: node for node in glue.body if isinstance(node, ast.FunctionDef)
        }
        owner_imports = {
            alias.asname or alias.name: (node.module, alias.name)
            for node in ast.walk(tree)
            if isinstance(node, ast.ImportFrom)
            and node.module is not None
            and node.level == 0
            for alias in node.names
        }
        imports.setdefault(module, set()).add(glue_name)
        class_doc = ast.get_docstring(glue, clean=False)
        class_node = ast.ClassDef(
            name=public_class,
            bases=[ast.Name(id=glue_name, ctx=ast.Load())],
            keywords=[],
            body=[ast.Expr(value=ast.Constant(value=class_doc))] if class_doc else [],
            decorator_list=[],
        )
        family_methods = 0
        for declaration in declarations:
            if text(declaration["family"]) != family:
                continue
            name = text(declaration["name"])
            if not name.isidentifier() or name in published:
                raise ValueError("project operation identity must be unique and valid")
            published.add(name)
            family_methods += 1
            helper_name = "_host_" + name
            helper = functions.get(helper_name)
            if helper is None:
                raise ValueError(
                    f"typed receiver helper is missing: {module}.{helper_name}"
                )
            wrapper = copy.deepcopy(helper)
            wrapper.name = name
            wrapper.decorator_list = []
            defaults = owned_defaults(declaration.get("pyo3_signature"))
            positional = [*wrapper.args.posonlyargs, *wrapper.args.args]
            first_default = next(
                (index for index, arg in enumerate(positional) if arg.arg in defaults),
                len(positional),
            )
            if any(arg.arg not in defaults for arg in positional[first_default:]):
                raise ValueError(
                    "owned project defaults would precede a required positional argument"
                )
            wrapper.args.defaults = [
                copy.deepcopy(defaults[arg.arg]) for arg in positional[first_default:]
            ]
            wrapper.args.kw_defaults = [
                copy.deepcopy(defaults[arg.arg]) if arg.arg in defaults else None
                for arg in wrapper.args.kwonlyargs
            ]
            doc = ast.get_docstring(helper, clean=False)
            if doc is None:
                doc = text(declaration["receiver_description"])
            if not doc:
                raise ValueError(
                    f"original typed receiver documentation is missing: {module}.{helper_name}"
                )
            doc = "\n".join(
                textwrap.fill(
                    line,
                    width=line_length - 11,
                    break_long_words=False,
                    break_on_hyphens=False,
                )
                for line in inspect.cleandoc(doc).splitlines()
            )
            doc += "\n\n" + documented_effects(declaration["effect_facts"])
            for parameter in [
                *wrapper.args.posonlyargs,
                *wrapper.args.args,
                *wrapper.args.kwonlyargs,
            ]:
                if isinstance(parameter.annotation, ast.Constant) and isinstance(
                    parameter.annotation.value, str
                ):
                    parameter.annotation = ast.parse(
                        parameter.annotation.value, mode="eval"
                    ).body
            if isinstance(wrapper.returns, ast.Constant) and isinstance(
                wrapper.returns.value, str
            ):
                wrapper.returns = ast.parse(wrapper.returns.value, mode="eval").body
            parameters = [*helper.args.posonlyargs, *helper.args.args]
            if not parameters or parameters[0].arg != "self":
                raise ValueError("project receiver glue must be an instance method")
            forwarded: list[ast.expr] = [
                ast.Name(id=arg.arg, ctx=ast.Load()) for arg in parameters[1:]
            ]
            if helper.args.vararg is not None:
                forwarded.append(
                    ast.Starred(
                        value=ast.Name(id=helper.args.vararg.arg, ctx=ast.Load()),
                        ctx=ast.Load(),
                    )
                )
            keywords = [
                ast.keyword(arg=arg.arg, value=ast.Name(id=arg.arg, ctx=ast.Load()))
                for arg in helper.args.kwonlyargs
            ]
            if helper.args.kwarg is not None:
                keywords.append(
                    ast.keyword(
                        arg=None,
                        value=ast.Name(id=helper.args.kwarg.arg, ctx=ast.Load()),
                    )
                )
            statements: list[ast.stmt] = [
                ast.Expr(value=ast.Constant(value=doc)),
                ast.Return(
                    value=ast.Call(
                        func=ast.Attribute(
                            value=ast.Name(id="self", ctx=ast.Load()),
                            attr=helper_name,
                            ctx=ast.Load(),
                        ),
                        args=forwarded,
                        keywords=keywords,
                    )
                ),
            ]
            wrapper.body = statements
            # Types/defaults are the actual helper's interface. Only forwarding is generated.
            interface = ast.FunctionDef(
                name=name,
                args=copy.deepcopy(wrapper.args),
                body=[ast.Pass()],
                decorator_list=[],
                returns=copy.deepcopy(helper.returns),
                type_comment=None,
            )
            for reference in annotation_names(interface):
                origin, imported = (
                    ("pse.generated.project_operation_defaults", reference)
                    if reference in {"DEFAULT_BRANCH", "DEFAULT_DOCUMENT_BYTES"}
                    else owner_imports.get(reference, (module, reference))
                )
                imports.setdefault(origin, set()).add(
                    imported if imported == reference else f"{imported} as {reference}"
                )
            class_node.body.append(wrapper)
            class_node.body.extend(
                ast.parse(
                    f"_public_metadata({name}, {glue_name}, {helper_name!r}, {qualifier!r}, {public_module!r})"
                ).body
            )
        if not family_methods:
            raise ValueError(f"owned public project family is empty: {family}")
        blocks.append(ast.unparse(ast.fix_missing_locations(class_node)))
    if len(published) != len(declarations):
        raise ValueError("project operation has an unsupported facade family")
    source = '''# @generated from Rust project operations and typed helper interfaces.
# Do not edit.
"""Public project operations projected from their Rust owner and typed receiver glue."""

from types import FunctionType
'''
    for module, names in sorted(imports.items()):
        source += f"from {module} import (" + ", ".join(sorted(names)) + ")\n"
    source += """
def _public_metadata(public: object, glue: type[object], helper: str, owner: str, module: str) -> None:
    original = vars(glue)[helper]
    if not isinstance(public, FunctionType) or not isinstance(original, FunctionType):
        raise TypeError((helper, type(public), type(original)))
    public.__qualname__ = owner + "." + public.__name__
    public.__module__ = module
    public.__annotations__ = dict(original.__annotations__)

"""
    source += "\n\n".join(blocks) + "\n"
    source += '\n__all__ = ("ProjectRuntimeOperations", "ResearchRuntime", "StudyWorkbenchMixin")\n'
    validate_python_floor(root, source)
    return source


def main() -> None:
    root = Path(sys.argv[1]).resolve()
    if sys.argv[2:] == ["--validate-floor"]:
        validate_python_floor(root, sys.stdin.read())
        return
    declarations = ROWS.validate_json(sys.stdin.buffer.read(), strict=True)
    sys.stdout.write(render(root, declarations))


if __name__ == "__main__":
    main()
