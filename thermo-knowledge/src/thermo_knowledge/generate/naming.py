# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Names of the PostgreSQL objects a declaration projects to."""

from __future__ import annotations

import re

_WORD_BREAK = re.compile(r"(?<=[a-z0-9])(?=[A-Z])|(?<=[A-Z])(?=[A-Z][a-z])")


def snake_case(name: str) -> str:
    """`MolarCp` becomes `molar_cp`."""
    return _WORD_BREAK.sub("_", name).lower()


def quantity_domain(name: str) -> str:
    """The domain of quantity type `name`, in schema `meta`."""
    return snake_case(name)


def scheme_domain(name: str) -> str:
    """The domain of identifier scheme `name`, in schema `meta`."""
    return f"id_{name}"
