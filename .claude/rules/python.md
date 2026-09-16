---
description: The Python package, its class systems and its boundaries
paths:
  - "python/**"
  - "conftest.py"
---

# Working in `python/pse`

## One class system per contract

There are exactly two, and they do not overlap (blueprint §21.1):

- **msgspec `Struct`** for the manifest and anything that crosses a JSON boundary.
- **attrs + cattrs** for row-shaped data. Build converters through `pse.codec.converter()`
  — it sets `forbid_extra_keys` and owns the hooks. `cattrs.Converter`,
  `GenConverter`, bare `structure`/`unstructure`, `json` and `tomllib` are banned imports.

Adding a third way to describe a record is the failure mode this rule exists to prevent.

## `Any` is banned

`typing.Any` is a banned import (ruff TID251) *and* `pse.governance` walks
`attrs.fields` at import time and fails on `Any`, a bare `dict` or a bare `list`. Both
layers matter: the lint catches the annotation, the walk catches what survives it.

`from __future__ import annotations` is forbidden under `python/pse` — PEP 563 turns
annotations into strings and the import-time walk stops seeing the real types. An
ast-grep rule enforces it.

## numpy stays at the boundary

numpy and scipy may be imported only in `python/pse/_array.py`, the parity harness and
tests; import-linter enforces it. `_array.py` passes `zero_copy_only=True` and every
copy carries a `copy_reason`. `idaes`, `pandas`, `pydantic`, `sympy`, `networkx`,
`matplotlib` and `click` never appear outside the parity harness; `pyomo` and `pint`
only in `python/pse/adapters/pyomo/`, the parity harness and tests.

Never `import pse._native` outside the package's own boundary modules — the extension is
reached through the typed surface, and `python/pse/_native.pyi` is
generated from the actual compiled PyO3 metadata by `just python-stubs`. Run
`just py-sync` after native API changes; fix the native declaration or generator
instead of editing the stub. The API surface is checked against the loaded extension.

## Parity fails, it never skips

`--parity` runs the suite against `idaes-pse==2.12.0`. A session fixture *fails* — not
skips — unless the interpreter is < 3.14, `idaes.__version__ == "2.12.0"` and `ipopt` is
on PATH. `pytest.skip` and `importorskip` are banned under the parity tree by ast-grep.
A skipped parity test is a parity claim with no evidence behind it.

## Markers and tools

Exactly one of `unit`/`component`/`integration`/`performance` per test; `conftest.py`
raises a `UsageError` at collection listing every offender. `just py-test` runs the
`unit or component` set.

Tools come from `.venv/bin`, declared with floors in `pyproject.toml`
`[dependency-groups]` and resolved by `uv.lock`, never from `$PATH` and never via
`pip install`. `just quality`
runs ruff, pyrefly, import-linter and the repository linters together.
