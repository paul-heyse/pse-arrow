# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Errors the read and load-src stages report; each message says what was expected and found."""

from __future__ import annotations


class StagingError(Exception):
    """A refusal or failure a person can act on."""


class UsageError(StagingError):
    """The command was asked for something that does not exist, such as an unknown source id."""
