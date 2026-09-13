# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Adapters onto foreign modelling ecosystems (blueprint §21.2, §21.3).

An adapter is a consumer of the compiled artifact, never a second authority:
``pse.open``/``compile``/``solve`` are native, and nothing in this subpackage is
on a production path. Each adapter's third-party dependency is optional and
imported lazily, so ``import pse`` stays free of it.
"""
