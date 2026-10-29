# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Read generated native evidence schemas for explicit offline report validation."""

from __future__ import annotations

import copy
import hashlib
import os
from typing import TYPE_CHECKING

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


class EvidenceSchemas:
    """One selected Rust-owned evidence schema publication; no execution authority."""

    def __init__(self, root: Path) -> None:
        self.root = root
        self.schema_root = "scripts/generated/evidence_schema"
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
        self._schemas: dict[tuple[str, str], dict[str, JsonValue]] = {}

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
            message = "Generated schema source unavailable after interface load; reload the selected schema publication."
            raise ValueError(message) from error
        if hashlib.sha256(raw).hexdigest() != expected_sha256:
            raise ValueError(
                "Generated schema source disagrees with its catalogue digest; reload the selected schema publication."
            )
        return raw

    def _captured_source(self, path: Path) -> bytes:
        expected = self._sources[path]
        message = "Generated schema source changed after interface load; reload the selected schema publication."
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
