# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Prepared published Rust contracts, without importing the native extension."""

from __future__ import annotations

import copy
import hashlib
import os
import re
from typing import TYPE_CHECKING, Literal

if TYPE_CHECKING:
    from pathlib import Path

from pydantic import JsonValue, TypeAdapter

JSON_OBJECT = TypeAdapter(dict[str, JsonValue])


def object_value(value: JsonValue) -> dict[str, JsonValue]:
    if not isinstance(value, dict):
        raise TypeError("native schema metadata requires an object")
    return value


def text_value(value: JsonValue) -> str:
    if not isinstance(value, str):
        raise TypeError("native schema metadata requires text")
    return value


SourceIdentity = tuple[int, int, int, int, int, int]


def source_identity(information: os.stat_result) -> SourceIdentity:
    return (
        information.st_dev,
        information.st_ino,
        information.st_size,
        information.st_mtime_ns,
        information.st_ctime_ns,
        information.st_mode,
    )


class SchemaSet:
    """One immutable loaded interface revision from the compiled catalogue producer."""

    def __init__(self, root: Path, *, family: Literal["product", "evidence"] = "product") -> None:
        if family not in {"product", "evidence"}:
            raise ValueError("unknown responsibility-owned schema family")
        self.root = root
        self.schema_root = "docs/generated/schema" if family == "product" else "scripts/generated/evidence_schema"
        self.manifest = JSON_OBJECT.validate_json(
            (root / self.schema_root / "catalogue.json").read_bytes(), strict=True
        )
        if self.manifest.get("version") != 1:
            raise ValueError("unsupported native schema catalogue version")
        self.fingerprint = text_value(self.manifest["schema_set_fingerprint"])
        documents = self.manifest["documents"]
        if not isinstance(documents, list):
            raise TypeError("native schema catalogue requires document rows")
        self._documents: dict[tuple[str, str], dict[str, JsonValue]] = {}
        for item in documents:
            row = object_value(item)
            key = (text_value(row["slug"]), text_value(row["direction"]))
            if key in self._documents:
                raise ValueError("duplicate native schema catalogue identity")
            self._documents[key] = row
        self._sources = {
            self._path(text_value(row["schema_path"])): source_identity(
                self._path(text_value(row["schema_path"])).stat()
            )
            for row in self._documents.values()
        }
        self._authoring_row = object_value(self.manifest["authoring"]) if self.manifest.get("authoring") is not None else None
        if self._authoring_row is not None:
            authoring = self._path(text_value(self._authoring_row["schema_path"]))
            self._sources[authoring] = source_identity(authoring.stat())
        self._schemas: dict[tuple[str, str], dict[str, JsonValue]] = {}
        self._authoring: dict[str, JsonValue] | None = None
        self._authoring_identity: dict[str, JsonValue] | None = None

    def _path(self, relative: str) -> Path:
        path = self.root / relative
        if (
            path.is_symlink()
            or path.resolve().parent != (self.root / self.schema_root).resolve()
        ):
            raise ValueError("native schema path escapes its generated owner")
        return path

    def _source(self, path: Path, expected_sha256: str) -> bytes:
        try:
            raw = self._captured_source(path)
        except OSError as error:
            message = "Generated schema source unavailable after interface load; restart required."
            raise ValueError(message) from error
        if hashlib.sha256(raw).hexdigest() != expected_sha256:
            raise ValueError(
                "Generated schema source disagrees with its catalogue digest; restart required."
            )
        return raw

    def _captured_source(self, path: Path) -> bytes:
        expected = self._sources[path]
        message = (
            "Generated schema source changed after interface load; restart required."
        )
        if path.is_symlink() or source_identity(path.stat()) != expected:
            raise ValueError(message)
        with path.open("rb") as source:
            if source_identity(os.fstat(source.fileno())) != expected:
                raise ValueError(message)
            raw = source.read(expected[2] + 1)
            after = source_identity(os.fstat(source.fileno()))
        if after != expected or source_identity(path.stat()) != expected:
            raise ValueError(message)
        return raw

    def _prepared(self, slug: str, direction: str) -> dict[str, JsonValue]:
        key = (slug, direction)
        if key not in self._schemas:
            row = self._documents[key]
            path = self.root / text_value(row["schema_path"])
            schema = JSON_OBJECT.validate_json(
                self._source(path, text_value(row["schema_sha256"])), strict=True
            )
            if schema["title"] != row["exported_name"]:
                raise ValueError("native catalogue/schema title disagreement")
            self._schemas[key] = schema
        return self._schemas[key]

    def document(self, slug: str, direction: str = "input") -> dict[str, JsonValue]:
        return copy.deepcopy(self._prepared(slug, direction))

    def identity(self, slug: str, direction: str = "input") -> dict[str, JsonValue]:
        return {
            **self._documents[(slug, direction)],
            "schema_set_fingerprint": self.fingerprint,
        }

    def input_for_type(self, exported_name: str) -> dict[str, JsonValue] | None:
        """Find a generated input by its catalogue-owned exported type name."""
        for (slug, direction), row in self._documents.items():
            if direction == "input" and row["exported_name"] == exported_name:
                return self.document(slug)
        return None

    def read(
        self, slug: str, direction: str | None, pointer: str = ""
    ) -> dict[str, JsonValue]:
        """Read exact schema detail, retaining the original root for local refs."""
        root_arguments: dict[str, JsonValue] = {"slug": slug}
        if slug == "authoring":
            if self._authoring_row is None:
                raise ValueError("This private document family has no scientific authoring guidance")
            if direction is not None:
                raise ValueError(
                    "authoring guidance has no native input/output direction"
                )
            if self._authoring is None:
                path = self.root / text_value(self._authoring_row["schema_path"])
                raw = self._source(path, text_value(self._authoring_row["sha256"]))
                self._authoring = JSON_OBJECT.validate_json(raw, strict=True)
                self._authoring_identity = {
                    "kind": "authoring_guidance",
                    "slug": "authoring",
                    "schema_path": self._authoring_row["schema_path"],
                    "sha256": self._authoring_row["sha256"],
                    "registration_source": "crates/pse-codegen/src/codegen/jsonschema.rs",
                    "applicability": "Logical authored source sections before hydration only. Raw TOML, .pse and Parquet bytes are admitted by their native loaders, not as JSON-schema objects.",
                }
            root = self._authoring
            identity = self._authoring_identity
        else:
            if direction is None or direction not in {"input", "output"}:
                raise ValueError(
                    "native schema reads require exact input/output direction"
                )
            try:
                root = self._prepared(slug, direction)
                identity = self.identity(slug, direction)
            except KeyError as error:
                raise ValueError(
                    "unknown exact native schema slug/direction"
                ) from error
            root_arguments["direction"] = direction
        if pointer and not pointer.startswith("/"):
            raise ValueError(
                "schema detail uses an empty root or an absolute JSON Pointer"
            )
        selected: JsonValue = root
        for encoded in pointer.split("/")[1:] if pointer else ():
            # RFC 6901 permits only ~0 and ~1 escapes. Do not reinterpret paths.
            if re.search(r"~(?![01])", encoded):
                raise ValueError("invalid JSON Pointer escape")
            key = encoded.replace("~1", "/").replace("~0", "~")
            if isinstance(selected, dict) and key in selected:
                selected = selected[key]
            elif (
                isinstance(selected, list)
                and key.isdigit()
                and str(int(key)) == key
                and int(key) < len(selected)
            ):
                selected = selected[int(key)]
            else:
                raise ValueError(
                    "schema pointer does not select an existing schema value"
                )
        return {
            "identity": identity,
            "pointer": pointer,
            "root_ref": {
                "capability": "native_contract_schema",
                "arguments": root_arguments,
            },
            "reference_scope": "Local $ref values resolve against the original schema root, not this selected subtree. Select their original JSON Pointer through root_ref.",
            "schema": copy.deepcopy(selected),
        }

    def envelope(
        self,
        host_schema: dict[str, JsonValue],
        documents: dict[str, str],
        required: tuple[str, ...],
    ) -> dict[str, JsonValue]:
        """Embed complete schema resources; local refs keep their original scope.

        A resource's $id establishes its own root. No scientific definitions or
        references are rewritten or merged into another document's namespace.
        """
        result = copy.deepcopy(host_schema)
        definitions = object_value(result.setdefault("$defs", {}))
        fields = object_value(result["properties"])
        for field, slug in documents.items():
            schema = self.document(slug)
            identity = f"urn:pse:boundary:{self.fingerprint.removeprefix('blake3:')}:{slug}:input"
            schema["$id"] = identity
            definitions[slug] = schema
            original = object_value(fields[field])
            alternatives = original.get("anyOf")
            nullable = (
                isinstance(alternatives, list) and {"type": "null"} in alternatives
            )
            if original.get("type") == "array":
                fields[field] = {**original, "items": {"$ref": identity}}
                continue
            fields[field] = (
                {"anyOf": [{"$ref": identity}, {"type": "null"}], "default": None}
                if nullable
                else {"$ref": identity}
            )
        result["$schema"] = "https://json-schema.org/draft/2020-12/schema"
        result["required"] = list(required)
        return result
