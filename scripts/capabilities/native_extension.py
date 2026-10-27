# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Consume an owned extension requirement and the actual installed receipt selection.

Selection consumes the published constructor contract and prepares nothing. A consumer
never guesses dev mode or an omitted build profile.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

from scripts import python_extension

if TYPE_CHECKING:
    from collections.abc import Mapping
    from pathlib import Path

    from pydantic import JsonValue


class ExtensionSelectionError(ValueError):
    """Missing or malformed owner selection before any product import or preparation."""

    def __init__(self, reason: str) -> None:
        super().__init__("installed extension selection is unavailable")
        self.reason = reason

    def detail(self) -> dict[str, JsonValue]:
        return {
            "outcome": "prerequisite_unavailable",
            "code": "pse.native.installed_extension_selection_unavailable",
            "selection": {"state": "unknown", "reason": self.reason},
            "remedy": "Inspect the existing Python preparation receipt and explicitly select its supported correctness mode/profile; no import or preparation was attempted.",
        }


@dataclass(frozen=True)
class InstalledExtensionSelection:
    mode: python_extension.Mode
    profile: str
    validated: bool


def selected_extension(
    environment: Mapping[str, str],
    requirement_mode: str,
    *,
    validated: bool,
    explicit_mode: python_extension.Mode | None = None,
    explicit_profile: str | None = None,
    root: Path | None = None,
) -> InstalledExtensionSelection:
    if requirement_mode not in {"installed_correctness", "native_correctness"}:
        raise ExtensionSelectionError("unknown_owned_mode_requirement")
    if requirement_mode == "native_correctness" and explicit_mode == "dev":
        raise ExtensionSelectionError("explicit_mode_conflicts_with_owner")
    profile = explicit_profile
    mode = explicit_mode
    if requirement_mode == "native_correctness":
        mode = "native"
    if mode is None or profile is None:
        published = python_extension.read_publication(
            "extension", environment, root=root
        )
        if published is None:
            raise ExtensionSelectionError("missing_published_generation")
        if requirement_mode == "native_correctness" and published.mode != "native":
            raise ExtensionSelectionError(
                "publication_incompatible_with_requested_mode"
            )
        if mode is None:
            mode = published.mode
        if profile is None:
            profile = published.profile
    try:
        profile = python_extension.PROFILE.validate_python(profile, strict=True)
    except ValueError:
        raise ExtensionSelectionError("invalid_explicit_profile") from None
    if mode is None:
        raise ExtensionSelectionError("missing_recorded_mode")
    return InstalledExtensionSelection(mode, profile, validated)


def extension_currency(
    environment: Mapping[str, str],
    requirement_mode: str,
    *,
    validated: bool,
) -> python_extension.ExtensionCurrency:
    if validated is not True:
        raise ExtensionSelectionError("owned_correctness_validation_required")
    selection = selected_extension(environment, requirement_mode, validated=validated)
    # Requested compatibility is distinct from construction validity.
    return python_extension.currency_check(
        environment,
        mode=selection.mode,
        profile=selection.profile,
        validated=selection.validated,
    )
