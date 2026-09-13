# Evidence for the Python library capability map

Reproduction material for the `[api:…]` and `[probe]` provenance markers in
`../../python-libraries.md`.

| File | What it is |
|---|---|
| `requirements-py314.txt` | the main environment: CPython **3.14.7**, every library `==`-pinned |
| `requirements-py313.txt` | the parity environment: CPython **3.13.12**, the same set plus `idaes-pse==2.12.0` (IDAES classifies 3.10–3.13 only) |
| `apidump.py` | walks a library's public API via `inspect`/`importlib` and emits JSON — the rustdoc-JSON analogue. Backs every `[api:lib@version]` marker. |
| `apiq.py` | queries those dumps (`cls` / `find` / `mod` / `has`) |
| `probe_arrow.py` | PROBES 1, 2, 8 — extension-type survival through IPC and `__arrow_c_stream__`; schema/field metadata; numpy zero-copy and the null→NaN collapse |
| `probe_pyomo.py` | PROBES 3, 4, 5 — `LinearExpression`/`Expr_if`/`ExternalFunction` lowering; units consistency; suffixes and solver availability |
| `probe_contracts.py` | PROBES 6, 7 — cattrs strictness and `Any`; msgspec constraints, located errors and JSON Schema |

To reproduce:

```sh
uv venv --python 3.14 py314
uv pip install --python py314/bin/python -r requirements-py314.txt
./py314/bin/python apidump.py api314 pyarrow attrs cattrs msgspec numpy pyomo scipy
./py314/bin/python probe_arrow.py
./py314/bin/python probe_contracts.py
./py314/bin/python probe_pyomo.py          # needs pint; see the map's §5

uv venv --python 3.13 py313
uv pip install --python py313/bin/python -r requirements-py313.txt
./py313/bin/python apidump.py api313 idaes
```

Two notes the map depends on:

- **`probe_pyomo.py` requires `pint`.** That is itself the finding: Pyomo's units machinery *is* pint,
  so §21.2's `assert_units_consistent` requirement cannot be met under §3.3's exclusion of a
  Python-side units library. Run it once without pint to see the `DeferredImportError`.
- **No solver is installed** in either environment, so PROBE 5 confirms suffix *declaration* only;
  the `dual` / `ipopt_zL_out` / `ipopt_zU_out` round-trip is recorded as `[UNVERIFIED]` in the map
  rather than asserted.
