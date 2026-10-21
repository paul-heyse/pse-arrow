#!/usr/bin/env python3
"""Extract borrowed protobuf field shapes from pinned protocol 0.13.1 prost sources.

Usage: python tools/update_grpc_wire_schema.py /path/to/surrealdb-protocol-0.13.1
No network or Rust code execution. This is vendor integration output, not a
canonical registry declaration or a second semantic model of database values.
"""
from pathlib import Path
import re
import sys

root = Path(sys.argv[1])
base = root / "gen/rust/proto/surrealdb/protocol"
files = [base / "v1/surrealdb.protocol.v1.rs", base / "rpc/v1/surrealdb.protocol.rpc.v1.rs"]
def declarations(text):
    for match in re.finditer(r"pub (struct|enum) (\w+) \{", text):
        start = match.end()
        depth = 1
        for end in range(start, len(text)):
            if text[end] == "{":
                depth += 1
            elif text[end] == "}":
                depth -= 1
                if depth == 0:
                    yield match, text[start:end]
                    break

messages = {}
for source in files:
    text = source.read_text()
    owners = {}
    for declaration, body in declarations(text):
        kind, owner = declaration.groups()
        if kind != "struct":
            continue
        for module, enum in re.findall(r'oneof = "(\w+)::(\w+)"', body):
            owners[(module, enum)] = owner
    for match, body in declarations(text):
        declaration, name = match.groups()
        if declaration == "enum":
            modules = list(re.finditer(r"pub mod (\w+) \{", text[:match.start()]))
            if not modules:
                continue
            name = owners.get((modules[-1].group(1), name))
            if name is None:
                continue
        fields = []
        for field in re.finditer(r'#\[prost\((.*?)\)\]\s*(?:#\[[^\n]*\]\s*)?(?:pub (\w+): ([^\n]+)|\w+\(([^\n]+)\),)', body, re.S):
            attrs, _, ty, variant = field.groups()
            if "oneof" in attrs:
                continue
            tag = re.search(r'tag = "(\d+)"', attrs)
            if not tag:
                continue
            if "message" in attrs:
                target = re.findall(r"\b[A-Z]\w*\b", ty or variant)[-1]
                kind = f'Message("{target}")'
            elif "string" in attrs:
                kind = "String"
            elif "bytes" in attrs:
                kind = "FixedUuid" if name == "Uuid" else "Blob"
            elif any(width in attrs for width in ("double", "fixed64", "sfixed64")):
                kind = "Fixed64"
            elif any(width in attrs for width in ("float", "fixed32", "sfixed32")):
                raise ValueError("Pinned wire schema gained fixed32; extend the borrowed validator")
            else:
                kind = "Varint"
            fields.append((int(tag.group(1)), kind, "repeated" in attrs))
        if fields:
            # Value's nested enum is intentionally merged with its empty shell.
            messages.setdefault(name, []).extend(fields)

rpc = files[1].read_text()
out = ['//! Pinned protocol 0.13.1 borrowed wire shapes. Regenerate with tools/update_grpc_wire_schema.py.', 'use super::grpc_wire::Kind;', 'use Kind::*;', 'pub(super) fn field(message: &str, tag: u32) -> Option<(Kind, bool)> {', '    Some(match (message, tag) {']
for message, fields in sorted(messages.items()):
    for tag, kind, repeated in sorted(set(fields)):
        out.append(f'        ("{message}", {tag}) => ({kind}, {str(repeated).lower()}),')
out.extend(['        _ => return None,', '    })', '}', 'pub(super) fn response(path: &str) -> Option<&\'static str> {', '    Some(match path.rsplit(\'/\').next()? {'])
tonic = (base / "rpc/v1/surrealdb.protocol.rpc.v1.tonic.rs").read_text()
for chunk in tonic.split('pub async fn ')[1:]:
    response = re.search(r'tonic::Response<(?:tonic::codec::Streaming<)?super::(\w+)', chunk)
    path = re.search(r'"/surrealdb.protocol.rpc.v1.SurrealDBService/(\w+)"', chunk)
    if response and path:
        out.append(f'        "{path.group(1)}" => "{response.group(1)}",')
out.extend(['        _ => return None,', '    })', '}', ''])
target = Path(__file__).resolve().parents[1] / "src/engine/remote/grpc_wire_schema.rs"
target.write_text("\n".join(out))
