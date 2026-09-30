# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The ThermoML schema reader: a synthetic schema in the source's format, then the real
acquired `ThermoML.xsd`, counted with `xml.etree` (the reader uses expat directly)."""

from __future__ import annotations

import xml.etree.ElementTree as ET
from collections import Counter
from pathlib import Path

import pytest
from r3_reader_support import (
    field_metadata,
    run_reader,
    stage_real,
    staged_manifest_of,
    staged_table,
)

from thermo_knowledge import config
from thermo_knowledge.acquire import store
from thermo_knowledge.readers import thermoml_schema
from thermo_knowledge.staging.errors import StagingError

PIN = "5c9945ce07c2"
TREE = store.pin_dir(config.raw_dir(), "thermoml_schema", PIN) / store.TREE_DIR_NAME
XSD = "{http://www.w3.org/2001/XMLSchema}"

SYNTHETIC = """<?xml version="1.0" encoding="UTF-8"?>
<!-- a synthetic schema -->
<xsd:schema targetNamespace="urn:demo" xmlns:d="urn:demo" xmlns:xsd="http://www.w3.org/2001/XMLSchema" elementFormDefault="qualified" attributeFormDefault="unqualified">
\t<xsd:annotation>
\t\t<xsd:documentation>demo version 1</xsd:documentation>
\t</xsd:annotation>
\t<xsd:element name="Report" type="d:ReportType"/>
\t<xsd:element name="Data">
\t\t<xsd:complexType>
\t\t\t<xsd:sequence>
\t\t\t\t<xsd:element name="Property" minOccurs="0" maxOccurs="unbounded">
\t\t\t\t\t<xsd:complexType>
\t\t\t\t\t\t<xsd:sequence>
\t\t\t\t\t\t\t<xsd:element name="PropertyGroup">
\t\t\t\t\t\t\t\t<xsd:complexType>
\t\t\t\t\t\t\t\t\t<xsd:choice>
\t\t\t\t\t\t\t\t\t\t<xsd:element name="Volumetric">
\t\t\t\t\t\t\t\t\t\t\t<xsd:complexType>
\t\t\t\t\t\t\t\t\t\t\t\t<xsd:sequence>
\t\t\t\t\t\t\t\t\t\t\t\t\t<xsd:element name="ePropName">
\t\t\t\t\t\t\t\t\t\t\t\t\t\t<xsd:simpleType>
\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t<xsd:restriction base="xsd:string">
\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t<xsd:enumeration value="Mass density, kg/m3">
\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t<xsd:annotation><xsd:documentation>density</xsd:documentation></xsd:annotation>
\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t</xsd:enumeration>
\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t<xsd:enumeration value="Specific volume, m3/kg"/>
\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t</xsd:restriction>
\t\t\t\t\t\t\t\t\t\t\t\t\t\t</xsd:simpleType>
\t\t\t\t\t\t\t\t\t\t\t\t\t</xsd:element>
\t\t\t\t\t\t\t\t\t\t\t\t</xsd:sequence>
\t\t\t\t\t\t\t\t\t\t\t</xsd:complexType>
\t\t\t\t\t\t\t\t\t\t</xsd:element>
\t\t\t\t\t\t\t\t\t</xsd:choice>
\t\t\t\t\t\t\t\t</xsd:complexType>
\t\t\t\t\t\t\t</xsd:element>
\t\t\t\t\t\t</xsd:sequence>
\t\t\t\t\t</xsd:complexType>
\t\t\t\t</xsd:element>
\t\t\t\t<xsd:element ref="d:Report" minOccurs="0"/>
\t\t\t</xsd:sequence>
\t\t</xsd:complexType>
\t</xsd:element>
\t<!-- types -->
\t<xsd:complexType name="ReportType">
\t\t<xsd:sequence>
\t\t\t<xsd:element name="nCount" type="xsd:integer"/>
\t\t\t<xsd:choice minOccurs="0">
\t\t\t\t<xsd:element name="sNote" type="xsd:string">
\t\t\t\t\t<xsd:annotation>
\t\t\t\t\t\t<xsd:documentation>free text</xsd:documentation>
\t\t\t\t\t\t<xsd:appinfo>SMILES notation, string</xsd:appinfo>
\t\t\t\t\t</xsd:annotation>
\t\t\t\t</xsd:element>
\t\t\t\t<xsd:element name="sOther" type="xsd:string"/>
\t\t\t</xsd:choice>
\t\t</xsd:sequence>
\t\t<xsd:attribute name="kind" type="d:Kind"/>
\t\t<xsd:attribute name="basis" type="d:Kind" use="optional"/>
\t</xsd:complexType>
\t<xsd:simpleType name="Kind">
\t\t<xsd:restriction base="xsd:string">
\t\t\t<xsd:enumeration value="alloy"/>
\t\t\t<xsd:enumeration value="mole fraction"/>
\t\t</xsd:restriction>
\t</xsd:simpleType>
</xsd:schema>
"""


def schema_with(body: str) -> str:
    return (
        '<xsd:schema xmlns:xsd="http://www.w3.org/2001/XMLSchema" targetNamespace="urn:x">'
        f"{body}</xsd:schema>"
    )


def test_every_declared_column_documents_name_and_unit() -> None:
    for name, schema in thermoml_schema.TABLES.items():
        for field in schema:
            metadata = field_metadata(schema, field.name)
            assert metadata.get("source_name"), (name, field.name)
            assert metadata.get("unit"), (name, field.name)


def test_synthetic_schema_rows(tmp_path: Path) -> None:
    run = run_reader(thermoml_schema, tmp_path, {"ThermoML.xsd": SYNTHETIC})
    (document,) = run.rows("schema_document")
    assert document["target_namespace"] == "urn:demo"
    assert document["element_form_default"] == "qualified"
    assert {(n["prefix"], n["uri"]) for n in run.rows("schema_namespaces")} == {
        ("d", "urn:demo"),
        ("xsd", "http://www.w3.org/2001/XMLSchema"),
    }
    assert [c["text"] for c in run.rows("schema_comments")] == [" a synthetic schema ", " types "]

    elements = run.rows("schema_elements")
    by_path = {(e["scope_kind"], e["path"]): e for e in elements}
    report = by_path[("element", "Report")]
    assert report["is_global"] and report["type"] == "d:ReportType"
    assert (report["type_prefix"], report["type_local"]) == ("d", "ReportType")
    assert report["parent_path"] is None and report["compositor"] is None
    prop = by_path[("element", "Data/Property")]
    assert (prop["min_occurs"], prop["max_occurs"]) == ("0", "unbounded")
    assert prop["inline_type"] == "complexType" and prop["type"] is None
    reference = by_path[("element", "Data/Report")]
    assert reference["name"] is None and reference["ref"] == "d:Report"
    assert reference["ref_local"] == "Report" and reference["position"] == 1
    note = by_path[("complexType", "ReportType/sNote")]
    assert note["compositor"] == "choice" and note["position"] == 0
    assert by_path[("complexType", "ReportType/nCount")]["max_occurs"] is None  # not written
    assert by_path[("complexType", "ReportType/nCount")]["position"] == 0

    compositors = run.rows("schema_compositors")
    choices = [c for c in compositors if c["kind"] == "choice" and c["scope_name"] == "ReportType"]
    assert len(choices) == 1 and choices[0]["min_occurs"] == "0"
    assert choices[0]["position"] == 1 and choices[0]["parent_compositor_line"] is not None
    top = [c for c in compositors if c["scope_name"] == "ReportType" and c["kind"] == "sequence"]
    assert top[0]["parent_compositor_line"] is None and top[0]["position"] is None

    enums = run.rows("schema_enumerations")
    assert [(e["scope_name"], e["value"], e["enum_index"]) for e in enums] == [
        ("Data", "Mass density, kg/m3", 0),
        ("Data", "Specific volume, m3/kg", 1),
        ("Kind", "alloy", 0),
        ("Kind", "mole fraction", 1),
    ]
    first = enums[0]
    assert first["path"] == "Data/Property/PropertyGroup/Volumetric/ePropName"
    assert first["property_group"] == "Volumetric" and first["element_name"] == "ePropName"
    assert first["type_name"] is None and enums[2]["type_name"] == "Kind"
    assert enums[2]["property_group"] is None and enums[2]["element_name"] is None
    simple = {s["path"]: s for s in run.rows("schema_simple_types")}
    assert simple["Kind"]["enumeration_count"] == 2 and simple["Kind"]["base_local"] == "string"

    attributes = run.rows("schema_attributes")
    assert [(a["name"], a["type"], a["use"]) for a in attributes] == [
        ("kind", "d:Kind", None),
        ("basis", "d:Kind", "optional"),
    ]
    complex_types = {(c["scope_name"], c["path"]): c for c in run.rows("schema_complex_types")}
    assert complex_types[("ReportType", "ReportType")]["attribute_count"] == 2
    assert complex_types[("ReportType", "ReportType")]["name"] == "ReportType"
    assert complex_types[("Data", "Data")]["name"] is None

    annotations = run.rows("schema_annotations")
    kinds = [(a["owner_kind"], a["owner_name"], a["kind"], a["text"]) for a in annotations]
    assert ("schema", "urn:x", "documentation", "demo version 1") not in kinds
    assert ("schema", None, "documentation", "demo version 1") in kinds
    assert ("enumeration", "Mass density, kg/m3", "documentation", "density") in kinds
    assert ("element", "sNote", "appinfo", "SMILES notation, string") in kinds
    assert all(a["_locator"].startswith("ThermoML.xsd#L") for a in annotations)
    (record,) = run.result.payload
    assert record.status == "read"


@pytest.mark.parametrize(
    ("document", "message"),
    [
        (schema_with('<xsd:key name="k"/>'), r"a 'key' node inside schema"),
        (schema_with('<xsd:element name="A" nillable="true"/>'), r"the attribute 'nillable'"),
        (schema_with('<xsd:element type="xsd:string"/>'), r"a global element without name"),
        (
            schema_with('<xsd:complexType name="T"><xsd:all/></xsd:complexType>'),
            r"a 'all' node inside complexType",
        ),
        (
            schema_with(
                '<xsd:simpleType name="S"><xsd:restriction base="xsd:string">'
                '<xsd:maxLength value="3"/></xsd:restriction></xsd:simpleType>'
            ),
            r"a 'maxLength' node inside restriction",
        ),
        (
            schema_with(
                '<xsd:complexType name="T"><xsd:sequence>'
                '<xsd:element name="A" ref="x:B"/></xsd:sequence></xsd:complexType>'
            ),
            r"exactly one of name and ref",
        ),
        ('<schema xmlns="urn:other"/>', r"not in the XML Schema namespace"),
        ("<xsd:schema", r"not well-formed XML"),
    ],
)
def test_unknown_constructs_and_malformed_documents_are_refused(
    tmp_path: Path, document: str, message: str
) -> None:
    with pytest.raises(StagingError, match=message):
        run_reader(thermoml_schema, tmp_path, {"ThermoML.xsd": document})


def test_a_located_error_names_the_line(tmp_path: Path) -> None:
    document = schema_with("\n\n<xsd:key name='k'/>")
    with pytest.raises(StagingError, match=r"ThermoML\.xsd#L3:\d+: a 'key' node"):
        run_reader(thermoml_schema, tmp_path, {"ThermoML.xsd": document})


# -- the real acquired schema -------------------------------------------------------------------

real = pytest.mark.skipif(
    not TREE.is_dir(),
    reason=f"the acquired ThermoML schema {TREE} is absent (run `tk acquire thermoml_schema`)",
)


@pytest.fixture(scope="module")
def staged(tmp_path_factory: pytest.TempPathFactory) -> Path:
    return stage_real("thermoml_schema", tmp_path_factory.mktemp("staged"))


def tag(node: ET.Element) -> str:
    return node.tag.replace(XSD, "") if isinstance(node.tag, str) else "#comment"


def expected_elements(root: ET.Element) -> list[tuple]:
    """(scope_kind, path, name, ref, type, minOccurs, maxOccurs) of every element declaration in
    document order, by a recursive walk that tracks the scope itself."""
    out: list[tuple] = []

    def visit(node: ET.Element, scope_kind: str, path: str | None) -> None:
        for child in node:
            name = tag(child)
            if name == "element":
                own = child.get("name") or child.get("ref", "").split(":")[-1]
                full = f"{path}/{own}"
                out.append(
                    (scope_kind, full, child.get("name"), child.get("ref"), child.get("type"), child.get("minOccurs"), child.get("maxOccurs"))
                )
                visit(child, scope_kind, full)
            elif name in ("complexType", "simpleType", "sequence", "choice", "annotation", "restriction"):
                visit(child, scope_kind, path)

    for top in root:
        name = tag(top)
        if name == "element":
            out.append(("element", top.get("name"), top.get("name"), top.get("ref"), top.get("type"), top.get("minOccurs"), top.get("maxOccurs")))
            visit(top, "element", top.get("name"))
        elif name in ("complexType", "simpleType"):
            visit(top, name, top.get("name"))
    return out


@real
def test_counts_match_an_independent_parse(staged: Path) -> None:
    root = ET.parse(TREE / "ThermoML.xsd").getroot()
    counts = Counter(tag(node) for node in root.iter())
    manifest = staged_manifest_of(staged)
    rows = {name: table.rows for name, table in manifest.tables.items()}
    assert rows["schema_elements"] == counts["element"] == 524
    assert rows["schema_complex_types"] == counts["complexType"] == 94
    assert rows["schema_simple_types"] == counts["simpleType"] == 64
    assert rows["schema_enumerations"] == counts["enumeration"] == 743
    assert rows["schema_attributes"] == counts["attribute"] == 2
    assert rows["schema_compositors"] == counts["sequence"] + counts["choice"] == 168
    assert rows["schema_annotations"] == counts["documentation"] + counts["appinfo"] == 101
    assert counts["appinfo"] == 6 and counts["documentation"] == 95
    text = (TREE / "ThermoML.xsd").read_text(encoding="utf-8")
    assert rows["schema_comments"] == text.count("<!--") == 34
    assert text.count("<xsd:enumeration ") == rows["schema_enumerations"]
    assert rows["schema_document"] == 1 and rows["schema_namespaces"] == 2


@real
def test_payload_is_the_one_schema_file(staged: Path) -> None:
    manifest = staged_manifest_of(staged)
    assert [(r.path, r.status) for r in manifest.payload] == [("ThermoML.xsd", "read")]
    assert set(manifest.tables) == set(thermoml_schema.TABLES)


@real
def test_elements_round_trip_in_document_order(staged: Path) -> None:
    root = ET.parse(TREE / "ThermoML.xsd").getroot()
    rows = sorted(
        staged_table(staged, "schema_elements").to_pylist(), key=lambda r: (r["line"], r["column"])
    )
    staged_form = [
        (r["scope_kind"], r["path"], r["name"], r["ref"], r["type"], r["min_occurs"], r["max_occurs"])
        for r in rows
    ]
    assert staged_form == expected_elements(root)
    assert len({r["_locator"] for r in rows}) == len(rows)
    # the survey's counts of types and of elements by kind
    assert Counter(r["type_prefix"] for r in rows if r["type"]) == Counter(
        node.get("type").split(":")[0] for node in root.iter(f"{XSD}element") if node.get("type")
    )
    assert sum(r["ref"] is not None for r in rows) == 64
    assert sum(r["is_global"] for r in rows) == 14


@real
def test_enumerations_round_trip_with_their_property_groups(staged: Path) -> None:
    root = ET.parse(TREE / "ThermoML.xsd").getroot()
    expected = [
        (node.get("value"))
        for node in root.iter(f"{XSD}enumeration")
    ]
    rows = sorted(
        staged_table(staged, "schema_enumerations").to_pylist(),
        key=lambda r: (r["line"], r["column"]),
    )
    assert [r["value"] for r in rows] == expected
    groups: dict[tuple[str, str], list[str]] = {}
    for row in rows:
        if row["property_group"]:
            groups.setdefault((row["element_name"], row["property_group"]), []).append(row["value"])
    by_group = {
        (element, group): len(values) for (element, group), values in groups.items()
    }
    assert by_group[("ePropName", "Criticals")] == 10
    assert by_group[("eMethodName", "Criticals")] == 10
    assert by_group[("ePropName", "VaporPBoilingTAzeotropTandP")] == 5
    assert by_group[("ePropName", "CompositionAtPhaseEquilibrium")] == 31
    assert len({group for element, group in groups if element == "ePropName"}) == 13
    # a value keeps its unit text and its spelling
    assert any(value.endswith(", K") for value in groups[("ePropName", "Criticals")])
    # every enumeration of a simple type is indexed from zero in order
    by_owner: dict[tuple[int, int], list[int]] = {}
    for row in rows:
        by_owner.setdefault((row["scope_name"], row["path"]), []).append(row["enum_index"])  # type: ignore[arg-type]
    for indexes in by_owner.values():
        assert indexes == list(range(len(indexes)))


@real
def test_types_compositors_attributes_and_annotations_round_trip(staged: Path) -> None:
    root = ET.parse(TREE / "ThermoML.xsd").getroot()
    attributes = staged_table(staged, "schema_attributes").to_pylist()
    expected = [(a.get("name"), a.get("type"), a.get("use")) for a in root.iter(f"{XSD}attribute")]
    assert [(a["name"], a["type"], a["use"]) for a in attributes] == expected
    compositors = sorted(
        staged_table(staged, "schema_compositors").to_pylist(), key=lambda r: (r["line"], r["column"])
    )
    nodes = [n for n in root.iter() if tag(n) in ("sequence", "choice")]
    assert [(c["kind"], c["min_occurs"], c["max_occurs"]) for c in compositors] == [
        (tag(n), n.get("minOccurs"), n.get("maxOccurs")) for n in nodes
    ]
    annotations = sorted(
        staged_table(staged, "schema_annotations").to_pylist(), key=lambda r: (r["line"], r["column"])
    )
    texts = [
        (tag(n), n.text or "")
        for n in root.iter()
        if tag(n) in ("documentation", "appinfo")
    ]
    assert [(a["kind"], a["text"]) for a in annotations] == texts
    simple = sorted(
        staged_table(staged, "schema_simple_types").to_pylist(), key=lambda r: (r["line"], r["column"])
    )
    restrictions = [n for n in root.iter(f"{XSD}restriction")]
    assert [(s["base"], s["enumeration_count"]) for s in simple] == [
        (r.get("base"), len(r.findall(f"{XSD}enumeration"))) for r in restrictions
    ]
    (document,) = staged_table(staged, "schema_document").to_pylist()
    assert document["target_namespace"] == root.get("targetNamespace")
    comments = [row["text"] for row in staged_table(staged, "schema_comments").to_pylist()]
    assert comments[0].strip().startswith("edited with XML Spy")
