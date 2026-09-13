<!--
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: 2026 Paul Heyse
-->

# `python/stubs/`

Type stubs for third-party packages that ship none, on `pyrefly`'s
`search-path` (see `[tool.pyrefly]` in `pyproject.toml`).

Empty in phase 0, deliberately. Until typed stubs exist, `pyrefly` sub-configs
replace `idaes.*`, `pyomo.*` and `pint.*` with `Any` under
`python/pse/parity/**` and `python/pse/adapters/pyomo/**` — a narrow, declared
hole rather than a repository-wide `ignore_missing_imports`.

A stub lands here when:

- the package is imported by `pse` outside those two sub-config scopes, or
- an adapter's surface is large enough that `Any` stops catching real mistakes.

Stubs are hand-written and reviewed like any other source: a wrong stub is
worse than no stub, because it makes the checker confident. Each one carries the
two-line SPDX header and states, at the top, which upstream version it was
written against and what part of the surface it covers. `pse._native`'s stub is
*not* here — it ships inside the package, beside the extension it describes, and
`test_native_stub_surface` diffs it against the real module.
