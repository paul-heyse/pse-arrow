# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""The Pyomo adapter (blueprint §21.2, §21.3).

Pyomo, pint and numpy are the ``pyomo`` extra, not core dependencies: this
adapter exists so the parity harness and the Pyomo ecosystem's tools can read a
compiled artifact, not because the platform needs Pyomo to solve. ``pyomo`` is
therefore imported inside the functions that use it, never at module scope, and
an import-linter contract confines those imports to this subpackage.

The adapter runs a capability pre-flight (:func:`pse.probe_host`) and raises
*before* it constructs a ``ConcreteModel``, so a missing solver is reported as a
missing solver. Empty in phase 0: the pre-flight lands with the adapter itself.
"""
