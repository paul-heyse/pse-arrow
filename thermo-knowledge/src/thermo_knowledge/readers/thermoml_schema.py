# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Reader of the ThermoML schema (`sources/thermoml_schema.toml`, `ThermoML.xsd`).

The XSD is read as data: every declaration becomes a row, in document order, with the schema's
own names and attribute values as written. Nothing is resolved: a `type` or `ref` is kept as
written (`tml:CitationType`, `xsd:integer`) with its prefix and local name split out, and the
namespaces the prefixes denote are in `schema_namespaces`.

Every row carries a scope, the path of the declaration and a locator `ThermoML.xsd#L<line>:<column>`
(the position of its start tag):

- `scope_kind` and `scope_name` name the top-level declaration the node sits in: a global
  `element`, a named `complexType` or a named `simpleType`;
- `path` is the scope name followed by the names of the elements down to the node, so the element
  `ePropName` of the property group `Criticals` has the path
  `PureOrMixtureData/Property/Property-MethodID/PropertyGroup/Criticals/ePropName`; an element that
  is a reference contributes the local part of its `ref`.

| Table | Rows |
|---|---|
| `schema_document` | the `schema` element's own attributes |
| `schema_namespaces` | the namespace declarations |
| `schema_elements` | every element declaration (global, local, by reference) |
| `schema_compositors` | every `sequence` and `choice` with its cardinality |
| `schema_complex_types` | every `complexType`, named or anonymous |
| `schema_simple_types` | every `simpleType` with its restriction base |
| `schema_enumerations` | every enumeration value, in document order |
| `schema_attributes` | every attribute declaration |
| `schema_annotations` | every `documentation` and `appinfo` text, verbatim |
| `schema_comments` | every XML comment, verbatim |

A construct or attribute the reader has no column for is an error, so nothing is silently dropped.
"""

from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass, field
from pathlib import Path
from xml.parsers import expat

import pyarrow as pa

from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.schema import (
    INT64,
    STRING,
    column,
    table_schema,
)
from thermo_knowledge.staging.writer import Writer

READER_VERSION = "1"

ARTIFACT = "ThermoML.xsd"
XS = "http://www.w3.org/2001/XMLSchema"
NOT_APPLICABLE = "not applicable"


def _text(
    name: str, source_name: str | None = None, *, nullable: bool = True, note: str | None = None
) -> pa.Field:
    return column(
        name, STRING, source_name=source_name, unit=NOT_APPLICABLE, note=note, nullable=nullable
    )


def _int(name: str, source_name: str | None = None, *, nullable: bool = True) -> pa.Field:
    return column(name, INT64, source_name=source_name, unit=NOT_APPLICABLE, nullable=nullable)


_LINE = _int("line", "line of the start tag", nullable=False)
_COLUMN = _int("column", "column of the start tag (0-based)", nullable=False)
_SCOPE = (
    _text("scope_kind", "element, complexType or simpleType", nullable=False),
    _text("scope_name", "name of the top-level declaration that holds the node", nullable=False),
)
_PATH = _text("path", "scope name and element names down to the node", nullable=False)


def _type_columns(attribute: str) -> tuple[pa.Field, ...]:
    return (
        _text(attribute, attribute, note="as written"),
        _text(f"{attribute}_prefix", f"{attribute} (before the colon)"),
        _text(f"{attribute}_local", f"{attribute} (after the colon)"),
    )


SCHEMA_DOCUMENT = table_schema(
    _LINE,
    _COLUMN,
    _text("target_namespace", "targetNamespace"),
    _text("element_form_default", "elementFormDefault"),
    _text("attribute_form_default", "attributeFormDefault"),
)

SCHEMA_NAMESPACES = table_schema(
    _LINE,
    _COLUMN,
    _text("prefix", "xmlns:<prefix> (null for the default namespace)"),
    _text("uri", "namespace URI", nullable=False),
)

SCHEMA_ELEMENTS = table_schema(
    _LINE,
    _COLUMN,
    *_SCOPE,
    _PATH,
    _text("parent_path", "path of the enclosing element or type (null for a global element)"),
    _text("name", "name"),
    _text("ref", "ref", note="as written"),
    _text("ref_prefix", "ref (before the colon)"),
    _text("ref_local", "ref (after the colon)"),
    *_type_columns("type"),
    _text(
        "min_occurs", "minOccurs", note="as written; null when not written (the XSD default is 1)"
    ),
    _text(
        "max_occurs",
        "maxOccurs",
        note="as written, number or unbounded; null when not written (the XSD default is 1)",
    ),
    _text("compositor", "kind of the enclosing sequence or choice (null for a global element)"),
    _int("compositor_line", "line of the enclosing sequence or choice"),
    _int("compositor_column", "column of the enclosing sequence or choice"),
    _int("position", "index among the element and compositor children of the enclosing compositor"),
    _text(
        "inline_type", "kind of the anonymous type the element declares: complexType or simpleType"
    ),
    column(
        "is_global",
        pa.bool_(),
        source_name="a direct child of the schema element",
        unit=NOT_APPLICABLE,
        nullable=False,
    ),
)

SCHEMA_COMPOSITORS = table_schema(
    _LINE,
    _COLUMN,
    *_SCOPE,
    _PATH,
    _text("kind", "sequence or choice", nullable=False),
    _text("min_occurs", "minOccurs", note="as written; null when not written"),
    _text("max_occurs", "maxOccurs", note="as written; null when not written"),
    _int("parent_compositor_line", "line of the enclosing compositor (null at the top of a type)"),
    _int("parent_compositor_column", "column of the enclosing compositor"),
    _int("position", "index among the element and compositor children of the enclosing compositor"),
)

SCHEMA_COMPLEX_TYPES = table_schema(
    _LINE,
    _COLUMN,
    *_SCOPE,
    _PATH,
    _text("name", "name (null for an anonymous type)"),
    _text("content", "kind of the type's top compositor: sequence, choice or none"),
    _int("attribute_count", "number of attribute declarations", nullable=False),
)

SCHEMA_SIMPLE_TYPES = table_schema(
    _LINE,
    _COLUMN,
    *_SCOPE,
    _PATH,
    _text("name", "name (null for an anonymous type)"),
    *_type_columns("base"),
    _int("enumeration_count", "number of enumeration values", nullable=False),
)

SCHEMA_ENUMERATIONS = table_schema(
    _LINE,
    _COLUMN,
    *_SCOPE,
    _PATH,
    _text("type_name", "name of the named simpleType that holds the value (null when anonymous)"),
    _text("element_name", "name of the element whose anonymous type holds the value"),
    _text(
        "property_group",
        "the element below PropertyGroup on the path (null elsewhere)",
        note="a segment of the path, not a source attribute",
    ),
    _int("enum_index", "position of the value in its restriction", nullable=False),
    _text("value", "value", nullable=False, note="verbatim, including any unit text"),
    *_type_columns("base"),
)

SCHEMA_ATTRIBUTES = table_schema(
    _LINE,
    _COLUMN,
    *_SCOPE,
    _PATH,
    _text("name", "name", nullable=False),
    *_type_columns("type"),
    _text("use", "use"),
)

SCHEMA_ANNOTATIONS = table_schema(
    _LINE,
    _COLUMN,
    _text("owner_kind", "tag of the annotated node", nullable=False),
    _text("owner_name", "name, ref or value of the annotated node"),
    _int("owner_line", "line of the annotated node's start tag", nullable=False),
    _int("owner_column", "column of the annotated node's start tag", nullable=False),
    _text("scope_kind", "element, complexType, simpleType or schema", nullable=False),
    _text("scope_name", "name of the top-level declaration (null for the schema element)"),
    _text("kind", "documentation or appinfo", nullable=False),
    _int("position", "index among the annotation's children", nullable=False),
    _text("text", "text content", nullable=False, note="verbatim"),
)

SCHEMA_COMMENTS = table_schema(
    _LINE,
    _COLUMN,
    _text("text", "comment text", nullable=False, note="verbatim, between <!-- and -->"),
)

TABLES: dict[str, pa.Schema] = {
    "schema_document": SCHEMA_DOCUMENT,
    "schema_namespaces": SCHEMA_NAMESPACES,
    "schema_elements": SCHEMA_ELEMENTS,
    "schema_compositors": SCHEMA_COMPOSITORS,
    "schema_complex_types": SCHEMA_COMPLEX_TYPES,
    "schema_simple_types": SCHEMA_SIMPLE_TYPES,
    "schema_enumerations": SCHEMA_ENUMERATIONS,
    "schema_attributes": SCHEMA_ATTRIBUTES,
    "schema_annotations": SCHEMA_ANNOTATIONS,
    "schema_comments": SCHEMA_COMMENTS,
}


# -- the syntax tree ----------------------------------------------------------------------------


@dataclass
class Node:
    tag: str
    attributes: dict[str, str]
    line: int
    column: int
    children: list[Node] = field(default_factory=list)
    text: str = ""


def parse(document: bytes) -> tuple[Node, list[tuple[str, int, int]], list[tuple[str | None, str]]]:
    """The document's element tree (tags are local names of the XML Schema namespace), its
    comments and its namespace declarations."""
    parser = expat.ParserCreate(namespace_separator=" ")
    parser.buffer_text = True
    stack: list[Node] = []
    roots: list[Node] = []
    comments: list[tuple[str, int, int]] = []
    namespaces: list[tuple[str | None, str]] = []

    def declared(prefix: str | None, uri: str) -> None:
        namespaces.append((prefix, uri))

    def start(name: str, attributes: dict[str, str]) -> None:
        namespace, _, local = name.rpartition(" ")
        if namespace != XS:
            raise StagingError(
                f"{ARTIFACT}#L{parser.CurrentLineNumber}: element {name!r} is not in the XML "
                "Schema namespace"
            )
        node = Node(local, dict(attributes), parser.CurrentLineNumber, parser.CurrentColumnNumber)
        if stack:
            stack[-1].children.append(node)
        else:
            roots.append(node)
        stack.append(node)

    def end(_: str) -> None:
        stack.pop()

    def characters(data: str) -> None:
        stack[-1].text += data

    def comment(data: str) -> None:
        comments.append((data, parser.CurrentLineNumber, parser.CurrentColumnNumber))

    parser.StartNamespaceDeclHandler = declared
    parser.StartElementHandler = start
    parser.EndElementHandler = end
    parser.CharacterDataHandler = characters
    parser.CommentHandler = comment
    try:
        parser.Parse(document, True)
    except expat.ExpatError as error:
        raise StagingError(f"{ARTIFACT}: not well-formed XML: {error}") from error
    if len(roots) != 1 or roots[0].tag != "schema":
        raise StagingError(f"{ARTIFACT}: the root element is not an XML Schema 'schema'")
    return roots[0], comments, namespaces


# -- walking the tree -----------------------------------------------------------------------------


def locator(line: int, column_number: int) -> str:
    return f"{ARTIFACT}#L{line}:{column_number}"


def split_name(value: str | None) -> tuple[str | None, str | None]:
    if value is None:
        return None, None
    prefix, separator, local = value.rpartition(":")
    return (prefix if separator else None), local


@dataclass(frozen=True)
class Scope:
    kind: str
    name: str


@dataclass(frozen=True)
class Enclosing:
    """The compositor an element or compositor sits in."""

    kind: str
    line: int
    column: int


class Walker:
    def __init__(self, emit: Callable[[str, dict[str, object]], None]) -> None:
        self.emit = emit

    def row(self, node: Node, **values: object) -> dict[str, object]:
        return {
            "_artifact": ARTIFACT,
            "_locator": locator(node.line, node.column),
            "line": node.line,
            "column": node.column,
            **values,
        }

    @staticmethod
    def check(node: Node, allowed: tuple[str, ...], where: str) -> None:
        for name in node.attributes:
            if name not in allowed:
                raise StagingError(
                    f"{locator(node.line, node.column)}: {where} has the attribute {name!r}, "
                    "which the reader has no column for; extend it and bump READER_VERSION"
                )

    @staticmethod
    def unexpected(node: Node, parent: str) -> StagingError:
        return StagingError(
            f"{locator(node.line, node.column)}: a {node.tag!r} node inside {parent}, which the "
            "reader has no rule for; extend it and bump READER_VERSION"
        )

    # -- top level -----------------------------------------------------------------------------

    def schema(self, root: Node) -> None:
        self.check(
            root, ("targetNamespace", "elementFormDefault", "attributeFormDefault"), "schema"
        )
        self.emit(
            "schema_document",
            self.row(
                root,
                target_namespace=root.attributes.get("targetNamespace"),
                element_form_default=root.attributes.get("elementFormDefault"),
                attribute_form_default=root.attributes.get("attributeFormDefault"),
            ),
        )
        for child in root.children:
            if child.tag == "annotation":
                self.annotation(child, root, "schema", None)
            elif child.tag == "element":
                name = child.attributes.get("name")
                if name is None:
                    raise StagingError(
                        f"{locator(child.line, child.column)}: a global element without name"
                    )
                scope = Scope("element", name)
                self.element(child, scope, name, None, None, is_global=True)
            elif child.tag == "complexType":
                name = child.attributes.get("name")
                if name is None:
                    raise StagingError(
                        f"{locator(child.line, child.column)}: a global complexType without name"
                    )
                self.complex_type(child, Scope("complexType", name), name)
            elif child.tag == "simpleType":
                name = child.attributes.get("name")
                if name is None:
                    raise StagingError(
                        f"{locator(child.line, child.column)}: a global simpleType without name"
                    )
                self.simple_type(child, Scope("simpleType", name), name)
            else:
                raise self.unexpected(child, "schema")

    # -- annotations ---------------------------------------------------------------------------

    def annotation(self, node: Node, owner: Node, scope_kind: str, scope_name: str | None) -> None:
        self.check(node, (), "annotation")
        owner_name = (
            owner.attributes.get("name")
            or owner.attributes.get("ref")
            or owner.attributes.get("value")
        )
        for position, child in enumerate(node.children):
            if child.tag not in ("documentation", "appinfo"):
                raise self.unexpected(child, "annotation")
            self.check(child, (), child.tag)
            if child.children:
                raise self.unexpected(child.children[0], child.tag)
            self.emit(
                "schema_annotations",
                self.row(
                    child,
                    owner_kind=owner.tag,
                    owner_name=owner_name,
                    owner_line=owner.line,
                    owner_column=owner.column,
                    scope_kind=scope_kind,
                    scope_name=scope_name,
                    kind=child.tag,
                    position=position,
                    text=child.text,
                ),
            )

    # -- types ---------------------------------------------------------------------------------

    def complex_type(self, node: Node, scope: Scope, path: str) -> None:
        self.check(node, ("name",), "complexType")
        content = "none"
        attributes = 0
        for child in node.children:
            if child.tag == "annotation":
                self.annotation(child, node, scope.kind, scope.name)
            elif child.tag in ("sequence", "choice"):
                if content != "none":
                    raise self.unexpected(child, "complexType (a second compositor)")
                content = child.tag
                self.compositor(child, scope, path, None, 0)
            elif child.tag == "attribute":
                attributes += 1
                self.attribute(child, scope, path)
            else:
                raise self.unexpected(child, "complexType")
        self.emit(
            "schema_complex_types",
            self.row(
                node,
                scope_kind=scope.kind,
                scope_name=scope.name,
                path=path,
                name=node.attributes.get("name"),
                content=content,
                attribute_count=attributes,
            ),
        )

    def simple_type(self, node: Node, scope: Scope, path: str) -> None:
        self.check(node, ("name",), "simpleType")
        name = node.attributes.get("name")
        restrictions = [child for child in node.children if child.tag == "restriction"]
        for child in node.children:
            if child.tag == "annotation":
                self.annotation(child, node, scope.kind, scope.name)
            elif child.tag != "restriction":
                raise self.unexpected(child, "simpleType")
        if len(restrictions) != 1:
            raise StagingError(
                f"{locator(node.line, node.column)}: a simpleType needs exactly one restriction"
            )
        restriction = restrictions[0]
        self.check(restriction, ("base",), "restriction")
        base = restriction.attributes.get("base")
        prefix, local = split_name(base)
        count = 0
        for child in restriction.children:
            if child.tag != "enumeration":
                raise self.unexpected(child, "restriction")
            self.check(child, ("value",), "enumeration")
            value = child.attributes.get("value")
            if value is None:
                raise StagingError(
                    f"{locator(child.line, child.column)}: an enumeration without value"
                )
            element_name = None if name is not None else path.rsplit("/", 1)[-1]
            segments = path.split("/")
            group = None
            if "PropertyGroup" in segments[:-1]:
                below = segments.index("PropertyGroup") + 1
                group = segments[below] if below < len(segments) - 1 else None
            self.emit(
                "schema_enumerations",
                self.row(
                    child,
                    scope_kind=scope.kind,
                    scope_name=scope.name,
                    path=path,
                    type_name=name,
                    element_name=element_name,
                    property_group=group,
                    enum_index=count,
                    value=value,
                    base=base,
                    base_prefix=prefix,
                    base_local=local,
                ),
            )
            for annotated in child.children:
                if annotated.tag != "annotation":
                    raise self.unexpected(annotated, "enumeration")
                self.annotation(annotated, child, scope.kind, scope.name)
            count += 1
        self.emit(
            "schema_simple_types",
            self.row(
                node,
                scope_kind=scope.kind,
                scope_name=scope.name,
                path=path,
                name=name,
                base=base,
                base_prefix=prefix,
                base_local=local,
                enumeration_count=count,
            ),
        )

    def attribute(self, node: Node, scope: Scope, path: str) -> None:
        self.check(node, ("name", "type", "use"), "attribute")
        if node.children:
            raise self.unexpected(node.children[0], "attribute")
        name = node.attributes.get("name")
        if name is None:
            raise StagingError(f"{locator(node.line, node.column)}: an attribute without name")
        type_value = node.attributes.get("type")
        prefix, local = split_name(type_value)
        self.emit(
            "schema_attributes",
            self.row(
                node,
                scope_kind=scope.kind,
                scope_name=scope.name,
                path=path,
                name=name,
                type=type_value,
                type_prefix=prefix,
                type_local=local,
                use=node.attributes.get("use"),
            ),
        )

    # -- compositors and elements --------------------------------------------------------------

    def compositor(
        self,
        node: Node,
        scope: Scope,
        path: str,
        parent: Enclosing | None,
        position: int,
    ) -> None:
        self.check(node, ("minOccurs", "maxOccurs"), node.tag)
        self.emit(
            "schema_compositors",
            self.row(
                node,
                scope_kind=scope.kind,
                scope_name=scope.name,
                path=path,
                kind=node.tag,
                min_occurs=node.attributes.get("minOccurs"),
                max_occurs=node.attributes.get("maxOccurs"),
                parent_compositor_line=None if parent is None else parent.line,
                parent_compositor_column=None if parent is None else parent.column,
                position=None if parent is None else position,
            ),
        )
        here = Enclosing(node.tag, node.line, node.column)
        index = 0
        for child in node.children:
            if child.tag == "element":
                self.element(child, scope, path, here, index)
                index += 1
            elif child.tag in ("sequence", "choice"):
                self.compositor(child, scope, path, here, index)
                index += 1
            else:
                raise self.unexpected(child, node.tag)

    def element(
        self,
        node: Node,
        scope: Scope,
        parent_path: str,
        compositor: Enclosing | None,
        position: int | None,
        *,
        is_global: bool = False,
    ) -> None:
        self.check(node, ("name", "ref", "type", "minOccurs", "maxOccurs"), "element")
        name = node.attributes.get("name")
        ref = node.attributes.get("ref")
        if (name is None) == (ref is None):
            raise StagingError(
                f"{locator(node.line, node.column)}: an element needs exactly one of name and ref"
            )
        own = name if name is not None else (split_name(ref)[1] or "")
        path = own if is_global else f"{parent_path}/{own}"
        inline: str | None = None
        for child in node.children:
            if child.tag == "annotation":
                self.annotation(child, node, scope.kind, scope.name)
            elif child.tag == "complexType":
                inline = self._inline(inline, child)
                self.complex_type(child, scope, path)
            elif child.tag == "simpleType":
                inline = self._inline(inline, child)
                self.simple_type(child, scope, path)
            else:
                raise self.unexpected(child, "element")
        type_value = node.attributes.get("type")
        if inline is not None and type_value is not None:
            raise StagingError(
                f"{locator(node.line, node.column)}: an element with both a type and an anonymous type"
            )
        type_prefix, type_local = split_name(type_value)
        ref_prefix, ref_local = split_name(ref)
        self.emit(
            "schema_elements",
            self.row(
                node,
                scope_kind=scope.kind,
                scope_name=scope.name,
                path=path,
                parent_path=None if is_global else parent_path,
                name=name,
                ref=ref,
                ref_prefix=ref_prefix,
                ref_local=ref_local,
                type=type_value,
                type_prefix=type_prefix,
                type_local=type_local,
                min_occurs=node.attributes.get("minOccurs"),
                max_occurs=node.attributes.get("maxOccurs"),
                compositor=None if compositor is None else compositor.kind,
                compositor_line=None if compositor is None else compositor.line,
                compositor_column=None if compositor is None else compositor.column,
                position=position,
                inline_type=inline,
                is_global=is_global,
            ),
        )

    @staticmethod
    def _inline(current: str | None, child: Node) -> str:
        if current is not None:
            raise StagingError(
                f"{locator(child.line, child.column)}: an element with two anonymous types"
            )
        return child.tag


def read(tree: Path, writer: Writer) -> None:
    """Decompose `ThermoML.xsd`."""
    try:
        document = (tree / ARTIFACT).read_bytes()
    except OSError as error:
        raise StagingError(f"{ARTIFACT}: cannot be read: {error}") from error
    root, comments, namespaces = parse(document)
    buffers: dict[str, list[dict[str, object]]] = {name: [] for name in TABLES}

    def emit(table: str, row: dict[str, object]) -> None:
        buffers[table].append(row)

    Walker(emit).schema(root)
    for prefix, uri in namespaces:
        buffers["schema_namespaces"].append(
            {
                "_artifact": ARTIFACT,
                "_locator": locator(root.line, root.column) + f"/xmlns:{prefix or ''}",
                "line": root.line,
                "column": root.column,
                "prefix": prefix,
                "uri": uri,
            }
        )
    for data, line, column_number in comments:
        buffers["schema_comments"].append(
            {
                "_artifact": ARTIFACT,
                "_locator": locator(line, column_number),
                "line": line,
                "column": column_number,
                "text": data,
            }
        )
    for table, rows in buffers.items():
        rows.sort(key=lambda row: (row["line"], row["column"]))
        writer.rows(table, rows)
