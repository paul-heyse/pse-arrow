# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Publish synthetic assessments through the actual current report codec."""

from __future__ import annotations

from typing import TYPE_CHECKING

from scripts import assessment_store

if TYPE_CHECKING:
    from pathlib import Path


def publish(directory: Path, receipt: dict) -> dict:
    for key, value in {
        "version": 6, "mode": "local", "baseline_failures": 0,
        "complete": False, "source_unchanged": True, "provenance_errors": [],
        "scope": [{"name": check["gate"]} for check in receipt.get("checks", [])],
    }.items():
        receipt.setdefault(key, value)
    for check in receipt.get("checks", []):
        check.setdefault("log", check["gate"] + ".log")
    assessment_store.publish(directory, receipt)
    return receipt
