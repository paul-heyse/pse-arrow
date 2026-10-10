# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""One strict metadata interpretation; authored Markdown bytes are never serialized."""

from __future__ import annotations

import datetime
import hashlib
import json
import re
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from ruamel.yaml import YAML

ROOT = Path(__file__).resolve().parents[1]
FIELDS = {
    "doc_role",
    "doc_owner",
    "doc_provenance",
    "doc_topics",
    "doc_retention",
    "doc_retirement_trigger",
    "doc_retention_reason",
    "doc_retention_owner",
}
ENUMS = {
    "doc_role": {
        "contract",
        "instruction",
        "decision",
        "plan",
        "review",
        "reference",
        "evidence",
        "index",
    },
    "doc_provenance": {"authored", "generated", "observed"},
    "doc_retention": {
        "while-current",
        "while-dependent",
        "owner-managed",
        "protected-unknown",
    },
    "doc_retirement_trigger": {
        "owner-release",
        "replacement-adopted",
        "producer-policy",
    },
}


@dataclass(frozen=True)
class Document:
    metadata: dict[str, object]
    prefix: bytes
    body: bytes


def _yaml() -> YAML:
    try:
        from ruamel.yaml import (  # noqa: PLC0415 -- cold bootstrap without docs dependencies
            YAML,
        )
    except ImportError as error:
        raise ValueError(
            "documentation parser missing; run just bootstrap-docs"
        ) from error
    parser = YAML(typ="safe", pure=True)
    parser.allow_duplicate_keys = False
    return parser


def _check_nodes(node: object, active: set[int], visited: set[int]) -> None:
    from ruamel.yaml.nodes import (  # noqa: PLC0415 -- docs-only parser boundary
        MappingNode,
        SequenceNode,
    )

    identity = id(node)
    if identity in active:
        raise ValueError("cyclic YAML metadata is not supported")
    if identity in visited:
        return
    active.add(identity)
    if isinstance(node, MappingNode):
        for key, value in node.value:
            if key.tag == "tag:yaml.org,2002:merge":
                raise ValueError("YAML merge keys are not supported in metadata")
            if key.tag != "tag:yaml.org,2002:str":
                raise ValueError("metadata mapping keys must be strings")
            _check_nodes(value, active, visited)
    elif isinstance(node, SequenceNode):
        for child in node.value:
            _check_nodes(child, active, visited)
    active.remove(identity)
    visited.add(identity)


def _dates(value: object) -> object:
    if isinstance(value, (datetime.datetime, datetime.date)):
        return value.isoformat()
    if isinstance(value, dict):
        return {key: _dates(item) for key, item in value.items()}
    if isinstance(value, list):
        return [_dates(item) for item in value]
    return value


def _legacy_encoding(text: str, identity: str | None, root: Path) -> str:
    """Adapt only explicitly sealed historical scalar spans, before strict YAML load."""
    if identity is None or not identity.startswith("docs/adr/"):
        return text
    policy_path = root / "docs/lifecycle.toml"
    if not policy_path.is_file():
        return text
    policy = tomllib.loads(policy_path.read_text(encoding="utf-8"))
    declarations = [
        entry for entry in policy.get("legacy_yaml", []) if entry["path"] == identity
    ]
    if len(declarations) > 1:
        raise ValueError(f"duplicate historical YAML encoding declaration: {identity}")
    if not declarations:
        return text
    scalars = declarations[0]["scalars"]
    lines = text.splitlines(keepends=True)
    for index, line in enumerate(lines):
        source = line.rstrip("\r\n")
        key, separator, literal = source.partition(":")
        if (
            separator
            and key in scalars
            and hashlib.sha256(literal.encode("utf-8")).hexdigest() == scalars[key]
        ):
            newline = line[len(source) :]
            lines[index] = (
                key + ": " + json.dumps(literal.strip(), ensure_ascii=False) + newline
            )
    return "".join(lines)


def read(
    data: bytes | str,
    where: str = "document",
    *,
    required: bool = False,
    identity: str | None = None,
    root: Path = ROOT,
) -> Document:
    raw = data.encode("utf-8") if isinstance(data, str) else data
    lines = raw.splitlines(keepends=True)
    if not lines or lines[0].rstrip(b"\r\n") != b"---":
        if required:
            raise ValueError(f"{where}: missing YAML front matter")
        return Document({}, b"", raw)
    end = next(
        (i for i, line in enumerate(lines[1:], 1) if line.rstrip(b"\r\n") == b"---"),
        None,
    )
    if end is None:
        raise ValueError(f"{where}: unterminated YAML front matter")
    text = b"".join(lines[1:end]).decode("utf-8")
    try:
        text = _legacy_encoding(text, identity, root)
        _check_nodes(_yaml().compose(text), set(), set())
        front = _metadata_mapping(_dates(_yaml().load(text)))
        validate_fields(front)
    except Exception as error:
        raise ValueError(f"{where}: {error}") from error
    return Document(front, b"".join(lines[: end + 1]), b"".join(lines[end + 1 :]))


def _metadata_mapping(value: object) -> dict[str, object]:
    if not isinstance(value, dict):
        raise ValueError("front matter must contain one mapping")  # noqa: TRY004 -- invalid document input uses the reader's ValueError contract
    if not all(isinstance(key, str) for key in value):
        raise ValueError("metadata mapping keys must be strings")
    return value


def validate_fields(values: dict[str, object], topics: set[str] | None = None) -> None:
    for key, value in values.items():
        if not isinstance(key, str):
            raise ValueError("metadata mapping keys must be strings")  # noqa: TRY004 -- invalid document input uses ValueError
        if not key.startswith("doc_"):
            continue
        if key not in FIELDS:
            raise ValueError(f"unknown controlled metadata field {key}")
        if key == "doc_topics":
            if not isinstance(value, list) or any(
                not isinstance(topic, str) for topic in value
            ):
                raise ValueError("doc_topics must be a list of declared strings")
            if len(set(value)) != len(value):
                raise ValueError("doc_topics contains duplicate topics")
            if topics is not None and set(value) - topics:
                raise ValueError(
                    f"undeclared doc_topics: {sorted(set(value) - topics)}"
                )
        elif not isinstance(value, str) or not value.strip():
            raise ValueError(f"{key} must be a nonempty string")
        elif key in ENUMS and value not in ENUMS[key]:
            raise ValueError(f"invalid {key}: {value!r}")


def reference(root: Path, value: str, source: Path | None = None) -> str:
    """Canonical refs are root-relative; legacy docs/ prefixes already meant root."""
    if value.startswith(("git:", "https:")):
        match = re.fullmatch(r"git:([0-9a-f]{12,40}):([^#\s]+)(#\S+)?", value)
        permalink = re.fullmatch(
            r"https://github\.com/[\w.-]+/[\w.-]+/blob/([0-9a-f]{12,40})/([^#\s]+)(#\S+)?",
            value,
        )
        selected = match or permalink
        if (
            selected is None
            or Path(selected.group(2)).is_absolute()
            or ".." in Path(selected.group(2)).parts
        ):
            raise ValueError(f"invalid immutable owner reference: {value!r}")
        return value
    path, marker, fragment = value.partition("#")
    if not path:
        if source is None:
            raise ValueError(f"reference has no document path: {value!r}")
        candidate = source
    else:
        base = root if source is None or path.startswith("docs/") else source.parent
        candidate = base / path
    resolved = candidate.resolve()
    if not resolved.is_relative_to(root.resolve()):
        raise ValueError(f"owner reference escapes repository: {value!r}")
    if not resolved.is_file():
        raise ValueError(f"owner reference does not exist: {value!r}")
    if marker and not fragment:
        raise ValueError(f"empty owner section fragment: {value!r}")
    if marker:
        from scripts.adr import (  # noqa: PLC0415 -- avoid the metadata/ADR import cycle
            HEADING_RE,
            markdown_headings,
        )

        text = resolved.read_text(encoding="utf-8")
        anchors = set(re.findall(r'<a\s+[^>]*id=["\']([^"\']+)', text))
        for _, heading in markdown_headings(text):
            title = heading.lstrip("# ").strip()
            anchors.add(re.sub(r"\s+", "-", re.sub(r"[^\w\s-]", "", title.lower())))
            section = HEADING_RE.match(heading)
            if section:
                anchors.add(
                    "section-"
                    + (section.group(1) or section.group(2)).lower().replace(".", "-")
                )
        if fragment not in anchors:
            raise ValueError(f"owner section does not exist: {value!r}")
    return resolved.relative_to(root.resolve()).as_posix() + (
        marker + fragment if marker else ""
    )


def load_policy(root: Path = ROOT) -> dict:
    policy = tomllib.loads((root / "docs/lifecycle.toml").read_text(encoding="utf-8"))
    if type(policy.get("version")) is not int or policy["version"] != 1:
        raise ValueError("unsupported document lifecycle policy version")
    topics = policy.get("topics", [])
    if (
        not isinstance(topics, list)
        or any(not isinstance(topic, str) or not topic for topic in topics)
        or len(set(topics)) != len(topics)
    ):
        raise ValueError("lifecycle topics must be unique declared strings")
    collections = tomllib.loads((root / "docs/site.toml").read_text(encoding="utf-8"))[
        "collections"
    ]
    if set(policy.get("collections", {})) - {
        collection["root"] for collection in collections
    }:
        raise ValueError(
            "lifecycle collection defaults must use existing publication collection roots"
        )
    defaults = [policy.get("defaults", {}), *policy.get("collections", {}).values()]
    roots = []
    exceptions = []
    for bundle in policy.get("bundles", []):
        candidate = root / bundle["root"]
        if (
            Path(bundle["root"]).is_absolute()
            or not candidate.resolve().is_relative_to(root.resolve())
            or not candidate.is_dir()
        ):
            raise ValueError(f"invalid lifecycle bundle root: {bundle['root']}")
        roots.append(candidate.resolve())
        defaults.append(bundle.get("defaults", {}))
    if len(roots) != len(set(roots)):
        raise ValueError("ambiguous bundle defaults")
    for exception in policy.get("exceptions", []):
        candidate = root / exception["path"]
        if (
            Path(exception["path"]).is_absolute()
            or not candidate.resolve().is_relative_to(root.resolve())
            or not candidate.is_file()
        ):
            raise ValueError(
                f"invalid exact-file lifecycle exception: {exception['path']}"
            )
        exceptions.append(candidate.resolve())
        defaults.append(exception.get("values", {}))
    if len(exceptions) != len(set(exceptions)):
        raise ValueError("duplicate exact-file lifecycle exception")
    for values in defaults:
        if set(values) - FIELDS:
            raise ValueError(
                f"unknown lifecycle default fields: {sorted(set(values) - FIELDS)}"
            )
        validate_fields(values, set(topics))
        for key in ("doc_owner", "doc_retention_owner"):
            if key in values:
                reference(root, str(values[key]))
    for profile in policy.get("status_profiles", {}).values():
        order = profile.get("order", [])
        if (
            not isinstance(order, list)
            or len(set(order)) != len(order)
            or set(order) - {"open", "blocked", "deferred", "complete", "unknown"}
        ):
            raise ValueError("invalid native status profile ordering")
        for state in order:
            for pattern in profile.get(state, []):
                re.compile(pattern)
    for binding in policy.get("scope_tables", []):
        if binding.get("status_profile") not in policy.get("status_profiles", {}):
            raise ValueError(
                f"unknown native status profile: {binding.get('status_profile')}"
            )
    return policy


def interpreted(
    root: Path, path: Path, document: Document, policy: dict, site: dict
) -> dict[str, object]:
    relative = path.resolve().relative_to(root.resolve()).as_posix()
    values = dict(policy.get("defaults", {}))
    origins = dict.fromkeys(values, "global default")
    collection_matches = []
    for collection in site["collections"]:
        base = (root / "docs" / collection["root"]).resolve()
        if path.resolve().is_relative_to(base) and (
            collection.get("recursive", True) or path.parent.resolve() == base
        ):
            collection_matches.append((len(base.parts), collection["root"]))
    if collection_matches:
        depth = max(item[0] for item in collection_matches)
        nearest = [name for size, name in collection_matches if size == depth]
        if len(nearest) != 1:
            raise ValueError(f"{relative}: ambiguous collection defaults")
        defaults = policy.get("collections", {}).get(nearest[0], {})
        values.update(defaults)
        origins.update(dict.fromkeys(defaults, f"collection:{nearest[0]}"))
    bundles = [
        bundle
        for bundle in policy.get("bundles", [])
        if Path(relative).is_relative_to(Path(bundle["root"]))
    ]
    if bundles:
        depth = max(len(Path(bundle["root"]).parts) for bundle in bundles)
        nearest = [
            bundle for bundle in bundles if len(Path(bundle["root"]).parts) == depth
        ]
        if len(nearest) != 1:
            raise ValueError(f"{relative}: ambiguous bundle defaults")
        defaults = nearest[0].get("defaults", {})
        values.update(defaults)
        origins.update(dict.fromkeys(defaults, f"bundle:{nearest[0]['root']}"))
    explicit = {
        key: value for key, value in document.metadata.items() if key.startswith("doc_")
    }
    aliases = [
        reference(root, str(document.metadata[key]), path)
        for key in ("disposition_owner", "disposition-owner")
        if document.metadata.get(key)
    ]
    if len(set(aliases)) > 1:
        raise ValueError(f"{relative}: conflicting legacy owners")
    if aliases:
        historical = aliases[0].startswith(("git:", "https:"))
        if historical:
            values["historical_owner"] = aliases[0]
        elif (
            "doc_owner" in explicit
            and reference(root, str(explicit["doc_owner"])) != aliases[0]
        ):
            raise ValueError(f"{relative}: conflicting canonical and legacy owners")
        else:
            explicit["doc_owner"] = aliases[0]
    for key in ("doc_owner", "doc_retention_owner"):
        if key in explicit:
            explicit[key] = reference(root, str(explicit[key]))
    exceptions = [
        entry for entry in policy.get("exceptions", []) if entry["path"] == relative
    ]
    if len(exceptions) > 1:
        raise ValueError(f"{relative}: duplicate exact-file exception")
    if exceptions:
        overrides = dict(exceptions[0].get("values", {}))
        for key in ("doc_owner", "doc_retention_owner"):
            if key in overrides:
                overrides[key] = reference(root, str(overrides[key]))
        for key in explicit.keys() & overrides.keys():
            if explicit[key] != overrides[key]:
                raise ValueError(f"{relative}: conflicting explicit {key}")
        values.update(overrides)
        origins.update(dict.fromkeys(overrides, "exact-file exception"))
    values.update(explicit)
    origins.update(dict.fromkeys(explicit, "authored metadata"))
    validate_fields(values, set(policy.get("topics", [])))
    for key in ("doc_owner", "doc_retention_owner"):
        if key in values:
            values[key] = reference(root, str(values[key]))
    if values.get("doc_owner", "").startswith(("git:", "https:")):
        values["historical_owner"] = values.pop("doc_owner")
    if "doc_owner" not in values:
        values["doc_retention"] = "protected-unknown"
    values["metadata_origins"] = origins
    return values
