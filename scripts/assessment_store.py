# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Atomic assessment roots and demand-shaped immutable detail.

Publication belongs to the existing report lifetime. Readers must hold that
owner's borrow while hydrating detail. This codec grants no independent custody.
"""

from __future__ import annotations

import json
import os
import re
import uuid
from pathlib import Path
from typing import TYPE_CHECKING

from scripts import resource_store

if TYPE_CHECKING:
    from collections.abc import Collection

VERSION = 6
SUMMARY = frozenset(
    {
        "version",
        "mode",
        "baseline_failures",
        "complete",
        "required_checks_covered",
        "source_unchanged",
        "provenance_errors",
        "input_coverage",
        "evidence",
        "parent",
    }
)
CHECK_SUMMARY = frozenset(
    {
        "gate",
        "status",
        "role",
        "log",
        "exit_code",
        "evidence_kind",
        "resource_invocation",
        "process",
        "origin",
        "origin_digest",
    }
)


def immutable(run: Path, value: dict) -> dict[str, str]:
    value = json.loads(json.dumps(value, allow_nan=False))
    identity = resource_store.digest(value)
    path = run / "details" / (identity + ".json")
    if path.exists():
        if detail(run, {"sha256": identity}) != value:
            raise ValueError("immutable assessment detail changed")
        resource_store.settle(path)
    else:
        resource_store.publish(path, value)
    return {"sha256": identity}


def detail(run: Path, reference: object) -> dict:
    if not isinstance(reference, dict) or set(reference) != {"sha256"}:
        raise ValueError("invalid assessment detail reference")
    identity = reference["sha256"]
    if not isinstance(identity, str) or not re.fullmatch(r"[a-f0-9]{64}", identity):
        raise ValueError("invalid assessment detail digest")
    from scripts import test_resources  # noqa: PLC0415 -- reciprocal report owner

    data, _ = test_resources.read_report_file(run, "details/" + identity + ".json")
    value = json.loads(data)
    if not isinstance(value, dict) or resource_store.digest(value) != identity:
        raise ValueError("assessment detail digest mismatch")
    return value


def publish(run: Path, receipt: dict) -> None:
    if receipt.get("version") != VERSION:
        raise ValueError("new assessments require the current owner codec")
    root = {key: value for key, value in receipt.items() if key in SUMMARY}
    root["scope"] = receipt["scope"]
    root["source_cut"] = immutable(
        run,
        {
            key: value
            for key, value in receipt.items()
            if key not in SUMMARY | {"scope", "checks"}
        },
    )
    root["checks"] = []
    for check in receipt["checks"]:
        summary = {key: value for key, value in check.items() if key in CHECK_SUMMARY}
        summary["detail"] = immutable(run, {"gate": check["gate"], "record": check})
        summary["population"] = {
            "selected": len(check["selected"])
            if isinstance(check.get("selected"), list)
            else None,
            "reported": len(check["results"])
            if isinstance(check.get("results"), list)
            else None,
            "report_errors": len(check.get("report_errors", [])),
        }
        root["checks"].append(summary)
    # Detail is fsynced, then the sole progress root is replaced and fsynced.
    # Report directories predate this codec and use the producer's umask.
    # Their files remain private; metadata authority directories use FileStore's
    # stricter 0700 rule. Never chmod an existing report to obtain publication.
    information = run.stat()
    if run.is_symlink() or information.st_uid != os.getuid():
        raise ValueError("assessment directory ownership is unsafe")
    temporary = run / (".checks-" + uuid.uuid4().hex + ".tmp")
    descriptor = os.open(
        temporary,
        os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW | os.O_CLOEXEC,
        0o600,
    )
    try:
        with os.fdopen(descriptor, "w") as stream:
            json.dump(root, stream, sort_keys=True)
            stream.write("\n")
            stream.flush()
            os.fsync(stream.fileno())
        temporary.replace(run / "checks.json")
        resource_store.sync_directory(run)
    finally:
        temporary.unlink(missing_ok=True)


def root(run: Path) -> dict:
    from scripts import test_resources  # noqa: PLC0415 -- reciprocal report owner

    with test_resources.borrow_report(run):
        data, custody = test_resources.read_report_root(run)
    value = json.loads(data)
    if not isinstance(value, dict):
        raise ValueError("invalid assessment root")
    if value.get("version") != VERSION:
        raise ValueError(
            "historical assessment requires explicit original export; automatic reuse refused"
        )
    for field, kind in (
        ("scope", list),
        ("checks", list),
        ("complete", bool),
        ("source_unchanged", bool),
        ("baseline_failures", int),
        ("provenance_errors", list),
    ):
        if type(value.get(field)) is not kind:
            raise ValueError("invalid or missing assessment " + field)
    names = [item.get("name") for item in value["scope"] if isinstance(item, dict)]
    if (
        len(names) != len(value["scope"])
        or any(not isinstance(name, str) for name in names)
        or len(set(names)) != len(names)
    ):
        raise ValueError("invalid or duplicate scope gate")
    checks = value["checks"]
    recorded = [check.get("gate") for check in checks if isinstance(check, dict)]
    if (
        len(recorded) != len(checks)
        or len(set(recorded)) != len(recorded)
        or set(recorded) - set(names)
    ):
        raise ValueError("undeclared or duplicate recorded gate")
    if any(
        not isinstance(check.get(field), str)
        for check in checks
        for field in ("gate", "status", "log")
    ):
        raise ValueError("invalid assessment gate summary")
    value["reader_custody"] = custody
    return value


def selected(
    run: Path, image: dict, *, gates: Collection[str] = (), source: bool = False
) -> dict:
    """Select before acquiring detail; an empty gate set is a summary-only read."""
    result = {key: value for key, value in image.items() if key != "source_cut"}
    result["recorded_gates"] = [check["gate"] for check in image["checks"]]
    result["source_cut_ref"] = image["source_cut"]
    result["checks"] = []
    if source:
        result.update(detail(run, image["source_cut"]))
    for check in image["checks"]:
        if check["gate"] not in gates:
            result["checks"].append(dict(check))
            continue
        payload = detail(run, check["detail"])
        record = payload.get("record")
        if (
            payload.get("gate") != check["gate"]
            or not isinstance(record, dict)
            or any(
                record.get(key) != value
                for key, value in check.items()
                if key in CHECK_SUMMARY
            )
        ):
            raise ValueError("assessment detail differs from its gate summary")
        result["checks"].append(record)
    return result


def export(run: Path) -> dict:
    image = root(run)
    result = selected(
        run, image, gates={check["gate"] for check in image["checks"]}, source=True
    )
    result.pop("recorded_gates")
    result.pop("source_cut_ref")
    result.pop("reader_custody")
    return result


def original_export(run: Path) -> dict:
    """Inspect protected v5 bytes without promoting them to current reuse authority."""
    from scripts import test_resources  # noqa: PLC0415 -- reciprocal report owner

    with test_resources.borrow_report(run):
        data, _ = test_resources.read_report_root(run)
    value = json.loads(data)
    if not isinstance(value, dict):
        raise ValueError("invalid historical assessment root")
    if value.get("version") != 5:
        raise ValueError(
            "original export expects an unchanged historical v5 assessment"
        )
    return value


def artifact_roles(run: Path) -> dict[str, str]:
    image = root(run)
    roles = {"details/" + image["source_cut"]["sha256"] + ".json": "provenance"}
    for check in image["checks"]:
        roles["details/" + check["detail"]["sha256"] + ".json"] = "evidence"
    return roles
