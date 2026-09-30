# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Errors the acquisition stage reports; each message says what was expected and found."""

from __future__ import annotations


class AcquireError(Exception):
    """A refusal or failure a person can act on."""


class UsageError(AcquireError):
    """The command was asked for something that does not exist, such as an unknown source id."""


class ManifestError(AcquireError):
    """One or more source manifests are invalid; `problems` has one line per problem."""

    def __init__(self, problems: list[str]) -> None:
        super().__init__("\n".join(problems))
        self.problems = problems
