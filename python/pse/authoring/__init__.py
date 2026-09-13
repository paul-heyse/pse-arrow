# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Authoring helpers over the generated contracts (blueprint §22).

The authoring surface builds change operations and documents against the typed
contracts and hands them to the native commit path; it never constructs Arrow
schemas or relation rows by hand. Empty in phase 0: the contracts it composes
arrive with the generator.
"""
